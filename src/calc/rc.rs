use crate::calc::require_positive;
use crate::error::EecalcError;

/// RC time constant and first-order cutoff: `tau = R * C`, `fc = 1 / (2π R C)`.
pub fn rc(resistance: f64, capacitance: f64) -> Result<(f64, f64), EecalcError> {
    require_positive(resistance, "resistance")?;
    require_positive(capacitance, "capacitance")?;
    let tau = resistance * capacitance;
    let fc = 1.0 / (2.0 * std::f64::consts::PI * resistance * capacitance);
    Ok((tau, fc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx_eq;

    #[test]
    fn readme_example() {
        let (tau, fc) = rc(10_000.0, 1e-6).unwrap();
        assert!(approx_eq(tau, 0.01));
        assert!(approx_eq(
            fc,
            1.0 / (2.0 * std::f64::consts::PI * 10_000.0 * 1e-6)
        ));
    }

    #[test]
    fn rejects_zero_capacitance() {
        assert_eq!(
            rc(1000.0, 0.0).unwrap_err(),
            EecalcError::NonPositiveValue {
                what: "capacitance"
            }
        );
    }
}
