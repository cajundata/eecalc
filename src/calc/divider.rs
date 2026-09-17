use crate::calc::require_positive;
use crate::error::EecalcError;

/// Two-resistor divider: `Vout = Vin * R2 / (R1 + R2)`.
pub fn divider(vin: f64, r1: f64, r2: f64) -> Result<f64, EecalcError> {
    require_positive(r1, "resistance")?;
    require_positive(r2, "resistance")?;
    Ok(vin * r2 / (r1 + r2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx_eq;

    #[test]
    fn readme_example() {
        let vout = divider(5.0, 10_000.0, 2_200.0).unwrap();
        assert!(approx_eq(vout, 5.0 * 2_200.0 / 12_200.0));
    }

    #[test]
    fn rejects_zero_r1() {
        assert_eq!(
            divider(5.0, 0.0, 1000.0).unwrap_err(),
            EecalcError::NonPositiveValue { what: "resistance" }
        );
    }
}
