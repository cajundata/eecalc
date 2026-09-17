use crate::error::EecalcError;

/// Parses a decimal number with an optional SI prefix from `p n u m k M`.
pub fn parse_value(token: &str) -> Result<f64, EecalcError> {
    if token.is_empty() {
        return invalid(token, "not a number");
    }

    let mut start = 0;
    let mut chars = token.char_indices();
    if let Some((_, c)) = chars.next()
        && (c == '+' || c == '-')
    {
        start = c.len_utf8();
    }

    let body = &token[start..];
    if body.eq_ignore_ascii_case("inf")
        || body.eq_ignore_ascii_case("infinity")
        || body.eq_ignore_ascii_case("nan")
    {
        return invalid(token, "not a number");
    }

    let bytes = token.as_bytes();
    let mut i = start;
    let mut seen_dot = false;
    let mut mantissa_end = start;

    while i < bytes.len() {
        match bytes[i] {
            b'0'..=b'9' => {
                i += 1;
                mantissa_end = i;
            }
            b'.' => {
                if seen_dot {
                    return invalid(token, "not a number");
                }
                seen_dot = true;
                i += 1;
                mantissa_end = i;
            }
            _ => break,
        }
    }

    let mantissa_slice = &token[start..mantissa_end];
    if mantissa_slice.is_empty() || mantissa_slice.chars().all(|c| c == '.') {
        return invalid(token, "not a number");
    }

    let mantissa: f64 = match token[..mantissa_end].parse::<f64>() {
        Ok(v) if v.is_finite() => v,
        _ => return invalid(token, "not a number"),
    };

    let suffix = &token[mantissa_end..];
    let factor = match suffix {
        "" => 1.0,
        "p" => 1e-12,
        "n" => 1e-9,
        "u" => 1e-6,
        "m" => 1e-3,
        "k" => 1e3,
        "M" => 1e6,
        other => {
            return invalid(token, &format!("unknown SI prefix '{other}'"));
        }
    };

    let value = mantissa * factor;
    if !value.is_finite() {
        return invalid(token, "not a number");
    }
    Ok(value)
}

/// Formats `value` in engineering notation with four significant figures.
pub fn format_eng(value: f64, unit: &str) -> String {
    if value == 0.0 {
        return format!("0 {unit}");
    }

    let sign = if value < 0.0 { "-" } else { "" };
    let abs = value.abs();
    let mut exp = (abs.log10() / 3.0).floor() as i32 * 3;
    exp = exp.clamp(-12, 6);

    let mut mantissa = abs / 10f64.powi(exp);
    mantissa = round_sig(mantissa, 4);

    if mantissa >= 1000.0 && exp < 6 {
        mantissa /= 1000.0;
        exp += 3;
        mantissa = round_sig(mantissa, 4);
    }

    let prefix = match exp {
        -12 => "p",
        -9 => "n",
        -6 => "u",
        -3 => "m",
        0 => "",
        3 => "k",
        6 => "M",
        _ => "",
    };

    format!("{sign}{} {prefix}{unit}", format_mantissa(mantissa))
}

fn round_sig(x: f64, sig: i32) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    let order = x.log10().floor();
    let factor = 10f64.powf(f64::from(sig - 1) - order);
    (x * factor).round() / factor
}

fn format_mantissa(m: f64) -> String {
    if m == 0.0 {
        return "0".to_string();
    }
    let order = m.log10().floor() as i32;
    let decimals = 3 - order;
    let raw = if decimals > 0 {
        format!("{m:.prec$}", prec = decimals as usize)
    } else {
        format!("{m:.0}")
    };
    trim_trailing_zeros(&raw)
}

fn trim_trailing_zeros(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_string()
}

fn invalid(value: &str, message: &str) -> Result<f64, EecalcError> {
    Err(EecalcError::InvalidNumber {
        value: value.to_string(),
        message: message.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_ok(token: &str, expected: f64) {
        let got = parse_value(token).expect(token);
        assert!(
            crate::approx_eq(got, expected),
            "{token}: got {got}, expected {expected}"
        );
    }

    fn assert_err(token: &str, contains: &str) {
        let err = parse_value(token).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(contains),
            "{token}: expected {contains:?} in {msg:?}"
        );
    }

    #[test]
    fn parses_prefixes_and_plain_decimals() {
        assert_ok("470", 470.0);
        assert_ok("4.7k", 4700.0);
        assert_ok("100n", 1e-7);
        assert_ok("2.2M", 2.2e6);
        assert_ok("1p", 1e-12);
        assert_ok("1u", 1e-6);
        assert_ok("20m", 0.02);
        assert_ok(".5", 0.5);
        assert_ok("10.", 10.0);
        assert_ok("-5", -5.0);
        assert_ok("+12", 12.0);
        assert_ok("1k", 1000.0);
    }

    #[test]
    fn rejects_malformed_values() {
        assert_err("10x", "unknown SI prefix 'x'");
        assert_eq!(
            parse_value("10x").unwrap_err().to_string(),
            "invalid value '10x': unknown SI prefix 'x'"
        );
        assert_err("k", "not a number");
        assert_err("10kk", "unknown SI prefix 'kk'");
        assert_err("1e3", "unknown SI prefix 'e3'");
        assert_err("10µ", "unknown SI prefix");
        assert_err("", "not a number");
        assert_err("4.7.0k", "not a number");
        assert_err("inf", "not a number");
        assert_err("NaN", "not a number");
        assert_err("-k", "not a number");
    }

    #[test]
    fn formats_readme_examples() {
        assert_eq!(format_eng(12.0 / 470.0, "A"), "25.53 mA");
        assert_eq!(format_eng(0.02 * 330.0, "V"), "6.6 V");
        assert_eq!(format_eng(5.0 * 5.0 / 100.0, "W"), "250 mW");
        assert_eq!(format_eng(650.0, "Ω"), "650 Ω");
        assert_eq!(format_eng(0.0, "V"), "0 V");
        assert_eq!(format_eng(-6.6, "V"), "-6.6 V");
        assert_eq!(format_eng(1e-6, "s"), "1 us");
    }
}
