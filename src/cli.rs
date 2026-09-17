use crate::calc::ohm::OhmGiven;
use crate::calc::power::PowerGiven;
use crate::error::EecalcError;
use crate::units::parse_value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Ohm,
    Power,
    Series,
    Parallel,
    Divider,
    Rc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Help,
    CommandHelp(CommandName),
    Ohm(OhmGiven),
    Power(PowerGiven),
    Series(Vec<f64>),
    Parallel(Vec<f64>),
    Divider { vin: f64, r1: f64, r2: f64 },
    Rc { resistance: f64, capacitance: f64 },
}

pub const GLOBAL_HELP: &str = "\
eecalc - electronics bench calculator

Usage:
  eecalc <command> [arguments]
  eecalc help | --help
  eecalc <command> --help

Commands:
  ohm       Given any two of voltage, current, resistance, compute the third
  power     Given any two of voltage, current, resistance, compute power
  series    Equivalent resistance of two or more series resistors
  parallel  Equivalent resistance of two or more parallel resistors
  divider   Output voltage of a two-resistor divider
  rc        Time constant and cutoff frequency of an RC circuit

Values:
  A decimal number with an optional SI prefix from p n u m k M.
  Examples: 470, 4.7k, 100n, 2.2M
  No space between number and prefix. u means micro.

Exit codes:
  0  success
  1  calculation error
  2  usage or parse error";

const VALUE_NOTE: &str =
    "Values: decimal plus optional prefix p n u m k M (u = micro). Examples: 470, 4.7k, 100n.";

const OHM_HELP: &str = "\
Usage: eecalc ohm --voltage <V> --current <I> --resistance <R>
  Give exactly two of --voltage/-v, --current/-i, --resistance/-r.";

const POWER_HELP: &str = "\
Usage: eecalc power --voltage <V> --current <I> --resistance <R>
  Give exactly two of --voltage/-v, --current/-i, --resistance/-r.
  Prints power P.";

const SERIES_HELP: &str = "\
Usage: eecalc series <R1> <R2> [R3 ...]
  Two or more resistances. Prints equivalent R.";

const PARALLEL_HELP: &str = "\
Usage: eecalc parallel <R1> <R2> [R3 ...]
  Two or more resistances. Prints equivalent R.";

const DIVIDER_HELP: &str = "\
Usage: eecalc divider --vin <V> --r1 <R> --r2 <R>
  All three flags required. Prints Vout.";

const RC_HELP: &str = "\
Usage: eecalc rc --resistance <R> --capacitance <C>
  Both --resistance/-r and --capacitance/-c required. Prints tau and fc.";

pub fn command_help(name: CommandName) -> String {
    let usage = match name {
        CommandName::Ohm => OHM_HELP,
        CommandName::Power => POWER_HELP,
        CommandName::Series => SERIES_HELP,
        CommandName::Parallel => PARALLEL_HELP,
        CommandName::Divider => DIVIDER_HELP,
        CommandName::Rc => RC_HELP,
    };
    format!("{usage}\n\n{VALUE_NOTE}")
}

pub fn parse(args: &[String]) -> Result<Command, EecalcError> {
    let Some(first) = args.first() else {
        return Err(EecalcError::MissingArgument("missing command".to_string()));
    };

    if first == "help" || first == "--help" {
        if let Some(extra) = args.get(1) {
            return Err(EecalcError::UnexpectedArgument(extra.clone()));
        }
        return Ok(Command::Help);
    }

    let name = parse_command_name(first)?;
    let rest = &args[1..];
    if rest.iter().any(|t| t == "--help") {
        return Ok(Command::CommandHelp(name));
    }

    match name {
        CommandName::Ohm => Ok(Command::Ohm(parse_ohm(rest)?)),
        CommandName::Power => Ok(Command::Power(parse_power(rest)?)),
        CommandName::Series => Ok(Command::Series(parse_resistors(rest, "series")?)),
        CommandName::Parallel => Ok(Command::Parallel(parse_resistors(rest, "parallel")?)),
        CommandName::Divider => parse_divider(rest),
        CommandName::Rc => parse_rc(rest),
    }
}

fn parse_command_name(name: &str) -> Result<CommandName, EecalcError> {
    match name {
        "ohm" => Ok(CommandName::Ohm),
        "power" => Ok(CommandName::Power),
        "series" => Ok(CommandName::Series),
        "parallel" => Ok(CommandName::Parallel),
        "divider" => Ok(CommandName::Divider),
        "rc" => Ok(CommandName::Rc),
        other => Err(EecalcError::UnknownCommand(other.to_string())),
    }
}

struct QuantityFlags {
    voltage: Option<f64>,
    current: Option<f64>,
    resistance: Option<f64>,
}

fn parse_ohm(args: &[String]) -> Result<OhmGiven, EecalcError> {
    let QuantityFlags {
        voltage,
        current,
        resistance,
    } = parse_vir(args)?;
    match (voltage, current, resistance) {
        (Some(voltage), Some(current), None) => Ok(OhmGiven::VoltageCurrent { voltage, current }),
        (Some(voltage), None, Some(resistance)) => Ok(OhmGiven::VoltageResistance {
            voltage,
            resistance,
        }),
        (None, Some(current), Some(resistance)) => Ok(OhmGiven::CurrentResistance {
            current,
            resistance,
        }),
        _ => Err(EecalcError::WrongArgumentCount { command: "ohm" }),
    }
}

fn parse_power(args: &[String]) -> Result<PowerGiven, EecalcError> {
    let QuantityFlags {
        voltage,
        current,
        resistance,
    } = parse_vir(args)?;
    match (voltage, current, resistance) {
        (Some(voltage), Some(current), None) => Ok(PowerGiven::VoltageCurrent { voltage, current }),
        (Some(voltage), None, Some(resistance)) => Ok(PowerGiven::VoltageResistance {
            voltage,
            resistance,
        }),
        (None, Some(current), Some(resistance)) => Ok(PowerGiven::CurrentResistance {
            current,
            resistance,
        }),
        _ => Err(EecalcError::WrongArgumentCount { command: "power" }),
    }
}

fn parse_vir(args: &[String]) -> Result<QuantityFlags, EecalcError> {
    let mut voltage = None;
    let mut current = None;
    let mut resistance = None;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--voltage" | "-v" => assign(&mut voltage, &args[idx], next_value(args, &mut idx)?)?,
            "--current" | "-i" => assign(&mut current, &args[idx], next_value(args, &mut idx)?)?,
            "--resistance" | "-r" => {
                assign(&mut resistance, &args[idx], next_value(args, &mut idx)?)?
            }
            other => return Err(EecalcError::UnexpectedArgument(other.to_string())),
        }
        idx += 1;
    }
    Ok(QuantityFlags {
        voltage,
        current,
        resistance,
    })
}

fn parse_resistors(args: &[String], command: &'static str) -> Result<Vec<f64>, EecalcError> {
    if args.is_empty() {
        return Err(EecalcError::WrongArgumentCount { command });
    }
    let mut values = Vec::new();
    for token in args {
        if is_flag(token) {
            return Err(EecalcError::UnexpectedArgument(token.clone()));
        }
        values.push(parse_value(token)?);
    }
    if values.len() < 2 {
        return Err(EecalcError::WrongArgumentCount { command });
    }
    Ok(values)
}

fn parse_divider(args: &[String]) -> Result<Command, EecalcError> {
    let mut vin = None;
    let mut r1 = None;
    let mut r2 = None;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--vin" => assign(&mut vin, &args[idx], next_value(args, &mut idx)?)?,
            "--r1" => assign(&mut r1, &args[idx], next_value(args, &mut idx)?)?,
            "--r2" => assign(&mut r2, &args[idx], next_value(args, &mut idx)?)?,
            other => return Err(EecalcError::UnexpectedArgument(other.to_string())),
        }
        idx += 1;
    }
    match (vin, r1, r2) {
        (Some(vin), Some(r1), Some(r2)) => Ok(Command::Divider { vin, r1, r2 }),
        _ => Err(EecalcError::MissingArgument(
            "divider requires --vin, --r1, and --r2".to_string(),
        )),
    }
}

fn parse_rc(args: &[String]) -> Result<Command, EecalcError> {
    let mut resistance = None;
    let mut capacitance = None;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--resistance" | "-r" => {
                assign(&mut resistance, &args[idx], next_value(args, &mut idx)?)?
            }
            "--capacitance" | "-c" => {
                assign(&mut capacitance, &args[idx], next_value(args, &mut idx)?)?
            }
            other => return Err(EecalcError::UnexpectedArgument(other.to_string())),
        }
        idx += 1;
    }
    match (resistance, capacitance) {
        (Some(resistance), Some(capacitance)) => Ok(Command::Rc {
            resistance,
            capacitance,
        }),
        _ => Err(EecalcError::MissingArgument(
            "rc requires --resistance and --capacitance".to_string(),
        )),
    }
}

fn assign(slot: &mut Option<f64>, flag: &str, raw: &str) -> Result<(), EecalcError> {
    if slot.is_some() {
        return Err(EecalcError::UnexpectedArgument(flag.to_string()));
    }
    *slot = Some(parse_value(raw)?);
    Ok(())
}

fn next_value<'a>(tokens: &'a [String], idx: &mut usize) -> Result<&'a str, EecalcError> {
    let flag = &tokens[*idx];
    let value_idx = *idx + 1;
    match tokens.get(value_idx) {
        None => Err(EecalcError::MissingArgument(format!(
            "missing value for {flag}"
        ))),
        Some(next) if is_flag(next) => Err(EecalcError::MissingArgument(format!(
            "missing value for {flag}"
        ))),
        Some(next) => {
            *idx = value_idx;
            Ok(next.as_str())
        }
    }
}

fn is_flag(token: &str) -> bool {
    if let Some(rest) = token.strip_prefix("--") {
        return !rest.is_empty();
    }
    if let Some(rest) = token.strip_prefix('-') {
        return rest.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parses_each_command() {
        assert!(matches!(
            parse(&args(&["ohm", "--voltage", "12", "--resistance", "470"])).unwrap(),
            Command::Ohm(OhmGiven::VoltageResistance { .. })
        ));
        assert!(matches!(
            parse(&args(&["power", "-v", "5", "-r", "100"])).unwrap(),
            Command::Power(PowerGiven::VoltageResistance { .. })
        ));
        let Command::Series(rs) = parse(&args(&["series", "100", "220", "330"])).unwrap() else {
            panic!("series");
        };
        assert_eq!(rs.len(), 3);
        assert!(matches!(
            parse(&args(&["parallel", "1k", "1k"])).unwrap(),
            Command::Parallel(_)
        ));
        assert!(matches!(
            parse(&args(&[
                "divider", "--vin", "5", "--r2", "2.2k", "--r1", "10k"
            ]))
            .unwrap(),
            Command::Divider { .. }
        ));
        assert!(matches!(
            parse(&args(&["rc", "-r", "10k", "-c", "1u"])).unwrap(),
            Command::Rc { .. }
        ));
    }

    #[test]
    fn help_forms() {
        assert_eq!(parse(&args(&["help"])).unwrap(), Command::Help);
        assert_eq!(parse(&args(&["--help"])).unwrap(), Command::Help);
        assert_eq!(
            parse(&args(&["ohm", "--help"])).unwrap(),
            Command::CommandHelp(CommandName::Ohm)
        );
        assert_eq!(
            parse(&args(&["ohm", "--voltage", "12", "--help"])).unwrap(),
            Command::CommandHelp(CommandName::Ohm)
        );
        assert!(matches!(
            parse(&args(&["help", "ohm"])).unwrap_err(),
            EecalcError::UnexpectedArgument(_)
        ));
        assert!(matches!(
            parse(&args(&[])).unwrap_err(),
            EecalcError::MissingArgument(_)
        ));
    }

    #[test]
    fn ohm_wrong_count_and_shorts() {
        let err = parse(&args(&["ohm", "--voltage", "12"])).unwrap_err();
        assert_eq!(
            err.to_string(),
            "ohm requires exactly two of --voltage, --current, --resistance"
        );
        assert!(matches!(
            parse(&args(&["ohm"])).unwrap_err(),
            EecalcError::WrongArgumentCount { command: "ohm" }
        ));
        assert!(matches!(
            parse(&args(&[
                "ohm",
                "--voltage",
                "12",
                "--current",
                "1",
                "--resistance",
                "10"
            ]))
            .unwrap_err(),
            EecalcError::WrongArgumentCount { command: "ohm" }
        ));
        assert!(matches!(
            parse(&args(&["ohm", "-i", "20m", "-r", "330"])).unwrap(),
            Command::Ohm(OhmGiven::CurrentResistance { .. })
        ));
    }

    #[test]
    fn unexpected_and_missing() {
        assert!(matches!(
            parse(&args(&["ohm", "--voltage", "12", "--voltage", "5"])).unwrap_err(),
            EecalcError::UnexpectedArgument(_)
        ));
        assert!(matches!(
            parse(&args(&["ohm", "--bogus", "1"])).unwrap_err(),
            EecalcError::UnexpectedArgument(_)
        ));
        assert!(matches!(
            parse(&args(&["series", "--foo", "1", "2"])).unwrap_err(),
            EecalcError::UnexpectedArgument(_)
        ));
        assert!(matches!(
            parse(&args(&["ohm", "--voltage"])).unwrap_err(),
            EecalcError::MissingArgument(_)
        ));
        assert!(matches!(
            parse(&args(&["nope"])).unwrap_err(),
            EecalcError::UnknownCommand(_)
        ));
        assert!(matches!(
            parse(&args(&["series", "100"])).unwrap_err(),
            EecalcError::WrongArgumentCount { command: "series" }
        ));
        assert!(matches!(
            parse(&args(&["divider", "--vin", "5"])).unwrap_err(),
            EecalcError::MissingArgument(_)
        ));
        assert!(matches!(
            parse(&args(&["rc", "--resistance", "10k"])).unwrap_err(),
            EecalcError::MissingArgument(_)
        ));
    }

    #[test]
    fn command_help_text() {
        assert_eq!(
            command_help(CommandName::Ohm),
            format!("{OHM_HELP}\n\n{VALUE_NOTE}")
        );
        assert!(command_help(CommandName::Power).contains("Prints power P."));
        assert!(GLOBAL_HELP.contains("p n u m k M"));
    }
}
