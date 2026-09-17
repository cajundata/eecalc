use crate::calc::require_positive;
use crate::error::EecalcError;

/// Valid Ohm's-law input: any two of voltage, current, resistance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OhmGiven {
    VoltageCurrent { voltage: f64, current: f64 },
    VoltageResistance { voltage: f64, resistance: f64 },
    CurrentResistance { current: f64, resistance: f64 },
}

/// The quantity that was not supplied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OhmResult {
    Voltage(f64),
    Current(f64),
    Resistance(f64),
}

/// Computes the missing Ohm's-law quantity: `V = I * R`, `I = V / R`, or `R = V / I`.
pub fn ohm(given: OhmGiven) -> Result<OhmResult, EecalcError> {
    match given {
        OhmGiven::VoltageResistance {
            voltage,
            resistance,
        } => {
            require_positive(resistance, "resistance")?;
            Ok(OhmResult::Current(voltage / resistance))
        }
        OhmGiven::VoltageCurrent { voltage, current } => {
            if current == 0.0 {
                return Err(EecalcError::DivisionByZero);
            }
            Ok(OhmResult::Resistance(voltage / current))
        }
        OhmGiven::CurrentResistance {
            current,
            resistance,
        } => {
            require_positive(resistance, "resistance")?;
            Ok(OhmResult::Voltage(current * resistance))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx_eq;

    #[test]
    fn current_from_voltage_and_resistance() {
        let result = ohm(OhmGiven::VoltageResistance {
            voltage: 12.0,
            resistance: 470.0,
        })
        .unwrap();
        match result {
            OhmResult::Current(i) => assert!(approx_eq(i, 12.0 / 470.0)),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn voltage_from_current_and_resistance() {
        let result = ohm(OhmGiven::CurrentResistance {
            current: 0.02,
            resistance: 330.0,
        })
        .unwrap();
        match result {
            OhmResult::Voltage(v) => assert!(approx_eq(v, 6.6)),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn resistance_from_voltage_and_current() {
        let result = ohm(OhmGiven::VoltageCurrent {
            voltage: 5.0,
            current: 0.01,
        })
        .unwrap();
        match result {
            OhmResult::Resistance(r) => assert!(approx_eq(r, 500.0)),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn allows_negative_voltage() {
        let result = ohm(OhmGiven::VoltageResistance {
            voltage: -12.0,
            resistance: 470.0,
        })
        .unwrap();
        match result {
            OhmResult::Current(i) => assert!(approx_eq(i, -12.0 / 470.0)),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn zero_voltage_yields_zero_current() {
        let result = ohm(OhmGiven::VoltageResistance {
            voltage: 0.0,
            resistance: 100.0,
        })
        .unwrap();
        match result {
            OhmResult::Current(i) => assert!(approx_eq(i, 0.0)),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn zero_current_when_computing_resistance_is_division_by_zero() {
        let err = ohm(OhmGiven::VoltageCurrent {
            voltage: 5.0,
            current: 0.0,
        })
        .unwrap_err();
        assert_eq!(err, EecalcError::DivisionByZero);
    }

    #[test]
    fn rejects_non_positive_resistance() {
        let err = ohm(OhmGiven::VoltageResistance {
            voltage: 12.0,
            resistance: 0.0,
        })
        .unwrap_err();
        assert_eq!(err, EecalcError::NonPositiveValue { what: "resistance" });
    }
}
