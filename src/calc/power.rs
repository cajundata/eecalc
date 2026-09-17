use crate::calc::require_positive;
use crate::error::EecalcError;

/// Valid power input: any two of voltage, current, resistance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerGiven {
    VoltageCurrent { voltage: f64, current: f64 },
    VoltageResistance { voltage: f64, resistance: f64 },
    CurrentResistance { current: f64, resistance: f64 },
}

/// Computes power: `P = V * I`, `P = I² * R`, or `P = V² / R`.
pub fn power(given: PowerGiven) -> Result<f64, EecalcError> {
    match given {
        PowerGiven::VoltageCurrent { voltage, current } => Ok(voltage * current),
        PowerGiven::VoltageResistance {
            voltage,
            resistance,
        } => {
            require_positive(resistance, "resistance")?;
            Ok(voltage * voltage / resistance)
        }
        PowerGiven::CurrentResistance {
            current,
            resistance,
        } => {
            require_positive(resistance, "resistance")?;
            Ok(current * current * resistance)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx_eq;

    #[test]
    fn from_voltage_and_resistance() {
        let p = power(PowerGiven::VoltageResistance {
            voltage: 5.0,
            resistance: 100.0,
        })
        .unwrap();
        assert!(approx_eq(p, 0.25));
    }

    #[test]
    fn from_voltage_and_current() {
        let p = power(PowerGiven::VoltageCurrent {
            voltage: 5.0,
            current: 0.05,
        })
        .unwrap();
        assert!(approx_eq(p, 0.25));
    }

    #[test]
    fn from_current_and_resistance() {
        let p = power(PowerGiven::CurrentResistance {
            current: 0.05,
            resistance: 100.0,
        })
        .unwrap();
        assert!(approx_eq(p, 0.25));
    }

    #[test]
    fn rejects_zero_resistance() {
        let err = power(PowerGiven::VoltageResistance {
            voltage: 5.0,
            resistance: 0.0,
        })
        .unwrap_err();
        assert_eq!(err, EecalcError::NonPositiveValue { what: "resistance" });
    }
}
