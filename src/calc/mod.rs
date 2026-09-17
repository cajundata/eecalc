pub mod divider;
pub mod ohm;
pub mod power;
pub mod rc;
pub mod resistance;

use crate::error::EecalcError;

pub(crate) fn require_positive(value: f64, what: &'static str) -> Result<(), EecalcError> {
    if value > 0.0 {
        Ok(())
    } else {
        Err(EecalcError::NonPositiveValue { what })
    }
}
