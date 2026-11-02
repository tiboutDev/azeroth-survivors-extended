//! Automatic player-only support policy. Native callbacks still decide when
//! a solo spell can trigger; this module supplies its valid recipient and scale.
use crate::{EnemyId, Phase, PlayerId, Result};
use std::collections::BTreeMap;
pub const RANGE: f64 = 30.;
pub const FRESH: f64 = 0.6;
pub const PRESSURE_RADIUS: f64 = 10.;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spell {
    HolyLight,
    HealingWave,
    Renew,
    Rejuvenation,
    Regrowth,
    PowerWordShield,
    LightningShield,
    Thorns,
    BattleShout,
    HolyNova,
    HolyShock,
}
impl Spell {
    pub fn periodic(self) -> bool {
        matches!(self, Self::Renew | Self::Rejuvenation)
    }
    pub fn pressure_targeted(self) -> bool {
        matches!(
            self,
            Self::PowerWordShield | Self::LightningShield | Self::Thorns
        )
    }
    pub fn heals(self) -> bool {
        !matches!(
            self,
            Self::PowerWordShield | Self::LightningShield | Self::Thorns | Self::BattleShout
        )
    }
    pub fn hybrid(self) -> bool {
        matches!(self, Self::HolyNova | Self::HolyShock)
    }
    pub fn prevents_refresh(self) -> bool {
        self.periodic() || self.pressure_targeted()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Active {
    pub shield: bool,
    pub renew: bool,
    pub rejuvenation: bool,
    pub lightning: bool,
    pub thorns: bool,
}
impl Active {
    pub fn contains(self, s: Spell) -> bool {
        match s {
            Spell::PowerWordShield => self.shield,
            Spell::Renew => self.renew,
            Spell::Rejuvenation => self.rejuvenation,
            Spell::LightningShield => self.lightning,
            Spell::Thorns => self.thorns,
            _ => false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    Player,
    Pet,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Member {
    pub id: PlayerId,
    pub kind: UnitKind,
    pub hp: f64,
    pub max: f64,
    pub position: [f64; 3],
    pub phase: Phase,
    pub seen: f64,
    pub life: u64,
    pub active: Active,
}
impl Member {
    pub fn valid(self) -> bool {
        self.hp.is_finite()
            && self.max.is_finite()
            && self.max > 0.
            && self.max <= 1e12
            && (0. ..=self.max).contains(&self.hp)
            && self.position.iter().all(|v| v.is_finite() && v.abs() < 1e7)
            && self.life <= 1_000_000
    }
    pub fn eligible(self, origin: [f64; 3], radius: f64, now: f64, wounded: bool) -> bool {
        self.kind == UnitKind::Player
            && self.valid()
            && self.phase == Phase::Playing
            && self.hp > 0.
            && (0. ..=FRESH).contains(&(now - self.seen))
            && (!wounded || self.hp < self.max)
            && distance(self.position, origin) <= radius * radius
    }
}
pub fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum()
}
pub fn select(
    party: &[Member],
    pressure: &BTreeMap<PlayerId, u32>,
    caster: PlayerId,
    spell: Spell,
    now: f64,
) -> Option<PlayerId> {
    let origin = party.iter().find(|v| v.id == caster)?;
    let mut candidates: Vec<_> = party
        .iter()
        .filter(|v| {
            v.eligible(origin.position, RANGE, now, spell.heals())
                && (!spell.prevents_refresh() || !v.active.contains(spell))
        })
        .collect();
    if spell.pressure_targeted() {
        if let Some(peak) = candidates
            .iter()
            .map(|v| pressure.get(&v.id).copied().unwrap_or(0))
            .max()
        {
            candidates.retain(|v| pressure.get(&v.id).copied().unwrap_or(0) == peak);
        }
    }
    let health = spell.heals() || spell == Spell::PowerWordShield;
    candidates
        .into_iter()
        .min_by(|a, b| {
            let order = if health {
                (a.hp / a.max).total_cmp(&(b.hp / b.max))
            } else {
                std::cmp::Ordering::Equal
            };
            order
                .then_with(|| (u8::from(a.id != caster)).cmp(&u8::from(b.id != caster)))
                .then(a.id.cmp(&b.id))
        })
        .map(|v| v.id)
}
/// Battle Shout affects every in-range player. Hybrids heal once per allied
/// unit using the native per-unit value; they never consume offensive targets.
pub fn group_targets(party: &[Member], caster: PlayerId, spell: Spell, now: f64) -> Vec<PlayerId> {
    let Some(origin) = party.iter().find(|v| v.id == caster) else {
        return vec![];
    };
    let radius = if spell == Spell::HolyNova { 25. } else { RANGE };
    party
        .iter()
        .filter(|v| {
            v.id != caster && v.eligible(origin.position, radius, now, spell != Spell::BattleShout)
        })
        .map(|v| v.id)
        .collect()
}
pub fn pressure(party: &[Member], enemies: &[TargetEnemy]) -> BTreeMap<PlayerId, u32> {
    party
        .iter()
        .map(|v| {
            (
                v.id,
                enemies
                    .iter()
                    .filter(|e| {
                        e.alive
                            && distance(e.position, v.position) <= PRESSURE_RADIUS * PRESSURE_RADIUS
                    })
                    .count() as u32,
            )
        })
        .collect()
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetEnemy {
    pub id: EnemyId,
    pub alive: bool,
    pub position: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Buff {
    pub spell: Spell,
    pub value: f32,
    pub auxiliary: f32,
    pub expires: f32,
    pub duration: f32,
}
impl Buff {
    pub fn live(self, clock: f32) -> bool {
        clock.is_finite()
            && self.expires.is_finite()
            && self.expires > clock
            && self.expires <= clock + 600.
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub caster: PlayerId,
    pub recipient: PlayerId,
    pub spell: Spell,
    pub amount: f32,
    pub buff: Option<Buff>,
    pub expires: f64,
    pub life: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shield {
    pub current: f32,
    pub total: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecipientState {
    pub clock: f32,
    pub active: Active,
    pub shield: Shield,
    pub retaliation_scale: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Apply {
    pub request: Request,
    pub shield: Option<Shield>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Damage {
    pub enemy: EnemyId,
    pub amount: f32,
}
/// Implementations own native allocations, register/stack contracts, visuals,
/// counters and native proc execution. Commands run at the existing safe boundary.
pub trait SupportPort {
    fn recipient(&mut self) -> Result<RecipientState>;
    fn apply(&mut self, command: Apply) -> Result<bool>;
}
pub fn deliver(
    port: &mut dyn SupportPort,
    request: Request,
    party: &[Member],
    now: f64,
) -> Result<bool> {
    let Some(me) = party.iter().find(|v| v.id == request.recipient) else {
        return Ok(false);
    };
    let Some(caster) = party.iter().find(|v| v.id == request.caster) else {
        return Ok(false);
    };
    if !now.is_finite()
        || !request.expires.is_finite()
        || !me.valid()
        || !caster.valid()
        || !(0. ..=FRESH).contains(&(now - caster.seen))
        || request.expires <= now
        || me.phase != Phase::Playing
        || me.hp <= 0.
        || me.life != request.life
        || caster.phase != Phase::Playing
        || now - caster.seen > FRESH
        || distance(caster.position, me.position) > RANGE * RANGE
        || me.kind != UnitKind::Player
        || caster.kind != UnitKind::Player
    {
        return Ok(false);
    }
    if !request.amount.is_finite() || !(0. ..=1e12).contains(&request.amount) {
        return Err("Invalid support command amount".into());
    }
    let mut prepared = Apply {
        request,
        shield: None,
    };
    if let Some(mut buff) = prepared.request.buff {
        let state = port.recipient()?;
        if !state.clock.is_finite()
            || !buff.value.is_finite()
            || !buff.auxiliary.is_finite()
            || !buff.duration.is_finite()
        {
            return Ok(false);
        }
        if buff.spell == Spell::PowerWordShield
            && (!state.shield.current.is_finite()
                || !state.shield.total.is_finite()
                || state.shield.current.abs() > 1e12
                || state.shield.total.abs() > 1e12)
        {
            return Ok(false);
        }
        if matches!(buff.spell, Spell::LightningShield | Spell::Thorns)
            && !state.retaliation_scale.is_finite()
        {
            return Ok(false);
        }

        if buff.spell != prepared.request.spell {
            return Err("Support buff/spell mismatch".into());
        }
        if (buff.spell.prevents_refresh() && state.active.contains(buff.spell))
            || !buff.live(state.clock)
        {
            return Ok(false);
        }
        if buff.spell == Spell::PowerWordShield {
            if state.shield.current.min(state.shield.total) > prepared.request.amount {
                return Ok(false);
            }
            prepared.shield = Some(Shield {
                current: prepared.request.amount,
                total: (state.shield.total - state.shield.current).max(0.)
                    + prepared.request.amount,
            });
        }
        if matches!(buff.spell, Spell::LightningShield | Spell::Thorns)
            && state.retaliation_scale > 0.
        {
            buff.value = buff.value * prepared.request.amount / state.retaliation_scale;
        }
        if buff.spell.periodic() {
            buff.value = buff.value * prepared.request.amount / me.max as f32;
        }
        if !buff.value.is_finite() || buff.value.abs() > 1e12 {
            return Ok(false);
        }
        prepared.request.buff = Some(buff);
    }
    port.apply(prepared)
}
pub fn redirect_heal(caster: PlayerId, target: PlayerId, spell: Spell, amount: f32) -> bool {
    caster != target && !spell.hybrid() && amount.is_finite() && amount > 0.
}
pub fn threshold_hp(target: &Member, caster_max: f32) -> f32 {
    (target.hp / target.max * caster_max as f64) as f32
}
pub fn elementary_heal(fraction: f32, caster_max: f64) -> f32 {
    fraction * caster_max as f32
}
pub fn credit_delta(old: f64, total: f64) -> Option<f32> {
    (total > old).then_some((total - old) as f32)
}
/// Damage uses the original caster's amount/coefficient, centred on the healed
/// player's position. Existing native global throttle remains in the adapter.
pub fn nature_burst(
    amount: f32,
    coefficient: f32,
    radius: f32,
    origin: [f32; 3],
    enemies: &[TargetEnemy],
) -> Vec<Damage> {
    enemies
        .iter()
        .filter(|e| {
            e.alive && distance(e.position, origin.map(|v| v as f64)) <= (radius * radius) as f64
        })
        .map(|e| Damage {
            enemy: e.id,
            amount: amount * coefficient,
        })
        .collect()
}
