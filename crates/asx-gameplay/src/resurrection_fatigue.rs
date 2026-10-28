//! Personal resurrection penalty. Times are authoritative simulation seconds.
use crate::Phase;
pub const RECOVERY_SECONDS: f32 = 60.0;
pub const MAX_STACKS: u8 = 3;
pub const PENALTY: f32 = 0.05;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fatigue {
    pub stacks: u8,
    pub remaining: f32,
    phase: Phase,
}
impl Fatigue {
    pub fn multiplier(self) -> f32 {
        1.0 - PENALTY * self.stacks as f32
    }
    /// Observe each world boundary. A revival starts a fresh recovery interval;
    /// time spent dead, paused or choosing never counts towards recovery.
    pub fn observe(&mut self, phase: Phase, elapsed: f32) {
        if phase == Phase::Inactive {
            *self = Self::default();
            return;
        }
        if self.phase == Phase::Downed && phase.can_tank() {
            self.stacks = (self.stacks + 1).min(MAX_STACKS);
            self.remaining = RECOVERY_SECONDS;
        } else if self.phase == Phase::Playing
            && phase == Phase::Playing
            && elapsed.is_finite()
            && elapsed > 0.0
            && self.stacks > 0
        {
            self.remaining -= elapsed;
            while self.remaining <= 0.0 && self.stacks > 0 {
                self.stacks -= 1;
                if self.stacks > 0 {
                    self.remaining += RECOVERY_SECONDS;
                } else {
                    self.remaining = 0.0;
                }
            }
        }
        self.phase = phase;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn revive(f: &mut Fatigue) {
        f.observe(Phase::Downed, 2.0);
        f.observe(Phase::Playing, 10.0);
    }
    #[test]
    fn repeated_deaths_cap_and_restart_recovery() {
        let mut f = Fatigue::default();
        f.observe(Phase::Playing, 100.0);
        assert_eq!(f.stacks, 0);
        for expected in [1, 2, 3, 3] {
            revive(&mut f);
            assert_eq!(f.stacks, expected);
            assert_eq!(f.remaining, 60.0);
            assert!((f.multiplier() - (1.0 - expected as f32 * 0.05)).abs() < 0.00001);
            f.observe(Phase::Playing, 30.0);
        }
        f.observe(Phase::Playing, 30.0);
        assert_eq!((f.stacks, f.remaining), (2, 60.0));
        f.observe(Phase::Playing, 119.0);
        assert_eq!((f.stacks, f.remaining), (1, 1.0));
        f.observe(Phase::Playing, 1.0);
        assert_eq!((f.stacks, f.remaining), (0, 0.0));
    }
    #[test]
    fn pauses_death_and_invalid_clock_do_not_drain() {
        let mut f = Fatigue::default();
        revive(&mut f);
        for phase in [Phase::Paused, Phase::Choosing, Phase::Playing] {
            f.observe(phase, 1000.0);
        }
        f.observe(Phase::Playing, f32::NAN);
        f.observe(Phase::Playing, -10.0);
        assert_eq!(f.remaining, 60.0);
        f.observe(Phase::Downed, 120.0);
        f.observe(Phase::Downed, 120.0);
        assert_eq!(f.remaining, 60.0);
        f.observe(Phase::Inactive, 0.0);
        assert_eq!(f, Fatigue::default());
        f.observe(Phase::Playing, 0.0);
        assert_eq!(f.stacks, 0);
    }
}
