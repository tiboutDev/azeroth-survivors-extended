//! Cooperative scaling of the global area/range multiplier, not spell base radii.
pub const COOP_BONUS_SHARE: f32 = 0.5;
pub fn cooperative_factor(native: f32) -> crate::Result<f32> {
    if !native.is_finite() || native < 0.0 { return Err("Invalid native area factor".into()); }
    Ok(if native > 1.0 { 1.0 + (native - 1.0) * COOP_BONUS_SHARE } else { native })
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn only_positive_bonus_is_reduced() {
        for (input,expected) in [(0.,0.),(0.8,0.8),(1.,1.),(1.4,1.2),(2.,1.5),(5.,3.)] {
            assert!((cooperative_factor(input).unwrap()-expected).abs()<1e-6);
        }
        for input in [f32::NAN,f32::INFINITY,-1.] { assert!(cooperative_factor(input).is_err()); }
    }
}
