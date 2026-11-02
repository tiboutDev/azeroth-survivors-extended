//! Shared rounding for an already-selected gold bonus multiplier.
pub fn multiply(base: u32, multiplier: f32) -> crate::Result<u32> {
    if !multiplier.is_finite() || multiplier < 0. {
        return Err("Invalid gold multiplier".into());
    }
    let value = (f64::from(base) * f64::from(multiplier)).round();
    if value > f64::from(u32::MAX) {
        return Err("Gold reward overflow".into());
    }
    Ok(value as u32)
}
