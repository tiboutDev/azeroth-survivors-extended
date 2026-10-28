//! Reviewed offensive effect requests, independent of native enum/layout values.
//! A request or prepared command does not prove an enemy accepted the effect.
use crate::{EnemyId, PlayerId, Result};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    MovementRate { strength: f32, seconds: f32 },
    ActionControl { seconds: f32 },
    MovementControl { seconds: f32 },
    LegacyTimed { strength: f32, seconds: f32 },
}
impl Effect {
    pub fn valid(self) -> bool {
        let duration = |v: f32| v.is_finite() && (0. ..=3600.).contains(&v);
        match self {
            Self::MovementRate { strength, seconds } | Self::LegacyTimed { strength, seconds } => {
                strength.is_finite() && (0. ..=1.).contains(&strength) && duration(seconds)
            }
            Self::ActionControl { seconds } | Self::MovementControl { seconds } => duration(seconds),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Command {
    pub enemy: EnemyId,
    /// Opaque game spell identity; no spell-number policy lives here.
    pub spell: u32,
    pub effect: Effect,
}
impl Command {
    pub fn validate(self) -> Result<Self> {
        if self.enemy == 0 || !self.effect.valid() { return Err("Invalid offensive effect command".into()); }
        Ok(self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Request {
    /// Local player whose native path emitted the request, not proven original
    /// caster ownership (e.g. foreign retaliation buffs can trigger locally).
    pub observed_by: PlayerId,
    pub command: Command,
}
impl Request {
    pub fn validate(self) -> Result<Self> {
        if self.observed_by == 0 { return Err("Invalid offensive effect observer".into()); }
        self.command.validate()?;
        Ok(self)
    }
}
