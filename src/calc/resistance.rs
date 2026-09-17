use crate::calc::require_positive;
use crate::error::EecalcError;

/// Equivalent resistance of series resistors: `R = Σ Rn`.
pub fn series(resistances: &[f64]) -> Result<f64, EecalcError> {
    let mut sum = 0.0;
    for r in resistances {
        require_positive(*r, "resistance")?;
        sum += r;
    }
    Ok(sum)
}

/// Equivalent resistance of parallel resistors: `R = 1 / Σ(1 / Rn)`.
pub fn parallel(resistances: &[f64]) -> Result<f64, EecalcError> {
    let mut recip = 0.0;
    for r in resistances {
        require_positive(*r, "resistance")?;
        recip += 1.0 / r;
    }
    Ok(1.0 / recip)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approx_eq;

    #[test]
    fn series_sum() {
        assert!(approx_eq(series(&[100.0, 220.0, 330.0]).unwrap(), 650.0));
    }

    #[test]
    fn parallel_two_equal() {
        assert!(approx_eq(parallel(&[1000.0, 1000.0]).unwrap(), 500.0));
    }

    #[test]
    fn parallel_readme() {
        let r = parallel(&[1000.0, 1000.0, 2200.0]).unwrap();
        let expected = 1.0 / (1.0 / 1000.0 + 1.0 / 1000.0 + 1.0 / 2200.0);
        assert!(approx_eq(r, expected));
    }

    #[test]
    fn rejects_zero() {
        assert_eq!(
            series(&[100.0, 0.0]).unwrap_err(),
            EecalcError::NonPositiveValue { what: "resistance" }
        );
        assert_eq!(
            parallel(&[-10.0, 100.0]).unwrap_err(),
            EecalcError::NonPositiveValue { what: "resistance" }
        );
    }
}
