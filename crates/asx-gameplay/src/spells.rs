//! Normalized local spell events. Native stack identity is an adapter concern.
//! This ledger neither sends packets nor guesses the outcome of failed casts.
use crate::{support::Spell, EnemyId, PlayerId, Result, Talents, TauntSpell};
use std::collections::{BTreeMap, VecDeque};
pub type CastId = u64;
pub const LIMIT: usize = 128;
const HISTORY: usize = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cast {
    pub id: CastId,
    pub caster: PlayerId,
    pub target: PlayerId,
    pub spell: Spell,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Healing {
    pub caster: PlayerId,
    pub target: PlayerId,
    pub spell: Spell,
    pub raw: f32,
    pub effective: f32,
    pub periodic: bool,
}
/// A confirmed successful target entry; not proof of damage amount or a cast ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Impact {
    pub caster: PlayerId,
    pub enemy: EnemyId,
    pub spell: TauntSpell,
    pub life: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpellEvent {
    Started(Cast),
    Completed(Cast),
    Failed(Cast),
    Replaced(Cast),
    Healed(Healing),
    Hit(Impact),
    OffensiveRequested(crate::offensive::Request),
    OffensivePrepared(crate::offensive::Command),
    PeriodicBudgeted(crate::periodic::Plan),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Completion {
    pub cast: Cast,
    pub clear_self_owner: bool,
    pub group_heal: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contribution {
    pub caster: PlayerId,
    pub target: PlayerId,
    pub raw: f32,
    pub effective: f32,
    pub foreign_periodic: f32,
}
#[derive(Default, Clone, Debug)]
pub struct Events {
    next: CastId,
    active: BTreeMap<CastId, Cast>,
    pub recent: VecDeque<SpellEvent>,
    pub started: u64,
    pub completed: u64,
    pub failed: u64,
    pub replaced: u64,
    pub healed: u64,
    pub impacts: u64,
    pub effect_requests: u64,
    pub effect_commands: u64,
    pub periodic_batches: u64,
}
impl Events {
    fn record(&mut self, event: SpellEvent) {
        self.recent.push_back(event);
        while self.recent.len() > HISTORY {
            self.recent.pop_front();
        }
    }
    pub fn begin(&mut self, caster: PlayerId, target: PlayerId, spell: Spell) -> Result<CastId> {
        if caster == 0 || target == 0 {
            return Err("Invalid spell event player".into());
        }
        if self.active.len() >= LIMIT {
            return Err("Active spell event bound exceeded".into());
        }
        let id = self
            .next
            .checked_add(1)
            .filter(|id| *id <= 1_000_000_000)
            .ok_or("Spell event identity exhausted")?;
        self.next = id;
        let cast = Cast {
            id,
            caster,
            target,
            spell,
        };
        self.active.insert(id, cast);
        self.started += 1;
        self.record(SpellEvent::Started(cast));
        Ok(id)
    }
    pub fn get(&self, id: CastId) -> Option<Cast> {
        self.active.get(&id).copied()
    }
    pub fn finish(&mut self, id: CastId) -> Option<Completion> {
        let cast = self.active.remove(&id)?;
        self.completed += 1;
        self.record(SpellEvent::Completed(cast));
        Some(Completion {
            cast,
            clear_self_owner: cast.caster == cast.target,
            group_heal: cast.spell.hybrid(),
        })
    }
    /// A confirmed native false return, distinct from an unobserved outcome.
    pub fn fail(&mut self, id: CastId) -> Option<Cast> {
        let cast = self.active.remove(&id)?;
        self.failed += 1;
        self.record(SpellEvent::Failed(cast));
        Some(cast)
    }
    pub fn replace(&mut self, id: CastId) -> Option<Cast> {
        let cast = self.active.remove(&id)?;
        self.replaced += 1;
        self.record(SpellEvent::Replaced(cast));
        Some(cast)
    }
    pub fn healing(&mut self, event: Healing) -> Result<Contribution> {
        if event.caster == 0
            || event.target == 0
            || ![event.raw, event.effective]
                .iter()
                .all(|v| v.is_finite() && (0. ..=1e12).contains(v))
        {
            return Err("Invalid native support meter contribution".into());
        }
        self.healed += 1;
        self.record(SpellEvent::Healed(event));
        Ok(Contribution {
            caster: event.caster,
            target: event.target,
            raw: event.raw,
            effective: event.effective,
            foreign_periodic: if event.periodic && event.caster != event.target { event.raw } else { 0. },
        })
    }
    /// Record every reviewed impact; only the agreed talent grants a taunt.
    /// Distinct native observations remain distinct, even for the same target.
    /// Reliable inter-player delivery, not this local journal, handles replay.
    pub fn impact(
        &mut self,
        event: Impact,
        talents: Talents,
    ) -> Result<Option<crate::taunts::Hit>> {
        if event.caster == 0 || event.enemy == 0 || event.life > 1_000_000 {
            return Err("Invalid spell impact identity/life".into());
        }
        self.impacts += 1;
        self.record(SpellEvent::Hit(event));
        Ok(
            crate::taunts::Taunts::eligible(talents, event.spell).then_some(crate::taunts::Hit {
                player: event.caster,
                enemy: event.enemy,
                spell: event.spell,
                life: event.life,
            }),
        )
    }
    pub fn offensive_request(&mut self, request: crate::offensive::Request) -> Result<crate::offensive::Command> {
        let request = request.validate()?;
        self.effect_requests += 1;
        self.record(SpellEvent::OffensiveRequested(request));
        Ok(request.command)
    }
    pub fn offensive_command(&mut self, command: crate::offensive::Command) -> Result<crate::offensive::Command> {
        let command = command.validate()?;
        self.effect_commands += 1;
        self.record(SpellEvent::OffensivePrepared(command));
        Ok(command)
    }
    pub fn periodic_budget(&mut self, batch: &crate::periodic::Batch, effective: f32, overheal: f32) -> Result<crate::periodic::Plan> {
        let plan = batch.plan(effective, overheal)?;
        self.periodic_batches += 1;
        self.record(SpellEvent::PeriodicBudgeted(plan));
        Ok(plan)
    }
    pub fn active(&self) -> usize {
        self.active.len()
    }
}
