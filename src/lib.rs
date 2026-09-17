pub mod calc;
pub mod cli;
pub mod error;
pub mod units;

use crate::calc::ohm::OhmResult;
use crate::cli::{Command, command_help};
use crate::error::EecalcError;
use crate::units::format_eng;

/// Parses `args` (with the program name already stripped) and returns formatted stdout.
pub fn run(args: &[String]) -> Result<String, EecalcError> {
    match cli::parse(args)? {
        Command::Help => Ok(cli::GLOBAL_HELP.to_string()),
        Command::CommandHelp(name) => Ok(command_help(name)),
        Command::Ohm(given) => match calc::ohm::ohm(given)? {
            OhmResult::Voltage(v) => Ok(format!("V = {}", format_eng(v, "V"))),
            OhmResult::Current(i) => Ok(format!("I = {}", format_eng(i, "A"))),
            OhmResult::Resistance(r) => Ok(format!("R = {}", format_eng(r, "Ω"))),
        },
        Command::Power(given) => {
            let p = calc::power::power(given)?;
            Ok(format!("P = {}", format_eng(p, "W")))
        }
        Command::Series(values) => {
            let r = calc::resistance::series(&values)?;
            Ok(format!("R = {}", format_eng(r, "Ω")))
        }
        Command::Parallel(values) => {
            let r = calc::resistance::parallel(&values)?;
            Ok(format!("R = {}", format_eng(r, "Ω")))
        }
        Command::Divider { vin, r1, r2 } => {
            let vout = calc::divider::divider(vin, r1, r2)?;
            Ok(format!("Vout = {}", format_eng(vout, "V")))
        }
        Command::Rc {
            resistance,
            capacitance,
        } => {
            let (tau, fc) = calc::rc::rc(resistance, capacitance)?;
            Ok(format!(
                "tau = {}\nfc  = {}",
                format_eng(tau, "s"),
                format_eng(fc, "Hz")
            ))
        }
    }
}

#[cfg(test)]
pub(crate) fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn golden_success_outputs() {
        assert_eq!(
            run(&args(&["ohm", "--voltage", "12", "--resistance", "470"])).unwrap(),
            "I = 25.53 mA"
        );
        assert_eq!(
            run(&args(&["ohm", "--current", "20m", "--resistance", "330"])).unwrap(),
            "V = 6.6 V"
        );
        assert_eq!(
            run(&args(&["power", "--voltage", "5", "--resistance", "100"])).unwrap(),
            "P = 250 mW"
        );
        assert_eq!(
            run(&args(&["series", "100", "220", "330"])).unwrap(),
            "R = 650 Ω"
        );
        assert_eq!(
            run(&args(&["parallel", "1k", "1k", "2.2k"])).unwrap(),
            "R = 407.4 Ω"
        );
        assert_eq!(
            run(&args(&[
                "divider", "--vin", "5", "--r1", "10k", "--r2", "2.2k"
            ]))
            .unwrap(),
            "Vout = 901.6 mV"
        );
        assert_eq!(
            run(&args(&["rc", "--resistance", "10k", "--capacitance", "1u"])).unwrap(),
            "tau = 10 ms\nfc  = 15.92 Hz"
        );
    }
}
