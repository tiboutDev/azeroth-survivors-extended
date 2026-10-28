//! Host-owned policies for the official multiplayer engine. No native layout,
//! transport or process identity belongs here. Inputs are normalized by an adapter.
use crate::{PlayerId, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberState {
    Alive,
    Downed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Award {
    World,
    ConvertedReward,
    /// A player voluntarily skips one reward; credit its full gold to the group.
    SkippedReward,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceKind {
    Reward,
    Level,
    Chest,
    EliteSkill,
    EliteItem,
    Augment,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub id: u64,
    pub kind: ChoiceKind,
    pub pending: BTreeSet<PlayerId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Gold { amount: u32, source: Award },
    Spend { actor: PlayerId, amount: u32 },
    Experience { amount: u32 },
    OpenChoice { id: u64, kind: ChoiceKind },
    CompleteChoice { id: u64, actor: PlayerId },
    SetState { actor: PlayerId, state: MemberState },
    SetMenu { actor: PlayerId, open: bool },
    Leave { actor: PlayerId },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    generation: u64,
    sequence: u64,
    host: PlayerId,
    members: BTreeMap<PlayerId, MemberState>,
    gold: u32,
    experience: u64,
    choice_serial: u64,
    choices: BTreeMap<u64, Choice>,
    menus: BTreeSet<PlayerId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PauseReason {
    Choices {
        kind: ChoiceKind,
        waiting: Vec<PlayerId>,
    },
    Menus {
        players: Vec<PlayerId>,
    },
}
impl Session {
    pub fn new(
        generation: u64,
        host: PlayerId,
        members: impl IntoIterator<Item = PlayerId>,
        gold: u32,
    ) -> Result<Self> {
        let mut roster = BTreeMap::new();
        for actor in members {
            if actor == 0 || roster.insert(actor, MemberState::Alive).is_some() {
                return Err("Invalid official party identity".into());
            }
        }
        if generation == 0 || !(2..=64).contains(&roster.len()) || !roster.contains_key(&host) {
            return Err("Invalid official party roster".into());
        }
        Ok(Self {
            generation,
            sequence: 0,
            host,
            members: roster,
            gold,
            experience: 0,
            choice_serial: 0,
            choices: BTreeMap::new(),
            menus: BTreeSet::new(),
        })
    }
    pub fn gold(&self) -> u32 {
        self.gold
    }
    pub fn experience(&self) -> u64 {
        self.experience
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn members(&self) -> &BTreeMap<PlayerId, MemberState> {
        &self.members
    }
    /// Prepare a complete replacement state; a failed native transaction must not
    /// commit it. Duplicate observations and stale run IDs cannot award twice.
    pub fn prepare(&self, generation: u64, sequence: u64, event: Event) -> Result<Self> {
        if generation != self.generation || self.sequence.checked_add(1) != Some(sequence) {
            return Err("Stale or out-of-order official event".into());
        }
        let mut next = self.clone();
        match event {
            Event::Gold { amount, source } => {
                let amount = match source {
                    Award::World | Award::SkippedReward => amount,
                    Award::ConvertedReward => amount / (next.members.len() as u32),
                };
                next.gold = next
                    .gold
                    .checked_add(amount)
                    .ok_or("Shared gold overflow")?;
            }
            Event::Spend { actor, amount } => {
                next.require_member(actor)?;
                next.gold = next
                    .gold
                    .checked_sub(amount)
                    .ok_or("Insufficient shared gold")?;
            }
            Event::Experience { amount } => {
                next.experience = next
                    .experience
                    .checked_add(amount.into())
                    .ok_or("Shared experience overflow")?
            }
            Event::OpenChoice { id, kind } => {
                if id <= next.choice_serial {
                    return Err("Reused official choice identity".into());
                }
                next.choice_serial = id;
                let pending = next
                    .members
                    .iter()
                    .filter_map(|(&id, &state)| (state == MemberState::Alive).then_some(id))
                    .collect::<BTreeSet<_>>();
                if !pending.is_empty() {
                    next.choices.insert(id, Choice { id, kind, pending });
                }
            }
            Event::CompleteChoice { id, actor } => {
                next.require_member(actor)?;
                let choice = next.choices.get_mut(&id).ok_or("Unknown official choice")?;
                if !choice.pending.remove(&actor) {
                    return Err("Choice already completed or player ineligible".into());
                }
                if choice.pending.is_empty() {
                    next.choices.remove(&id);
                }
            }
            Event::SetState { actor, state } => {
                *next
                    .members
                    .get_mut(&actor)
                    .ok_or("Unknown official actor")? = state;
                if state == MemberState::Downed {
                    next.release(actor);
                }
            }
            Event::SetMenu { actor, open } => {
                next.require_member(actor)?;
                if open {
                    next.menus.insert(actor);
                } else {
                    next.menus.remove(&actor);
                }
            }
            Event::Leave { actor } => {
                next.require_member(actor)?;
                if actor == next.host {
                    return Err("Host departure requires group session termination".into());
                }
                next.members.remove(&actor);
                next.release(actor);
            }
        }
        next.sequence = sequence;
        Ok(next)
    }
    fn require_member(&self, actor: PlayerId) -> Result<()> {
        if self.members.contains_key(&actor) {
            Ok(())
        } else {
            Err("Unknown official actor".into())
        }
    }
    fn release(&mut self, actor: PlayerId) {
        self.menus.remove(&actor);
        self.choices.retain(|_, choice| {
            choice.pending.remove(&actor);
            !choice.pending.is_empty()
        });
    }
    pub fn pause(&self) -> Option<PauseReason> {
        if let Some(choice) = self.choices.values().next() {
            return Some(PauseReason::Choices {
                kind: choice.kind,
                waiting: choice.pending.iter().copied().collect(),
            });
        }
        (!self.menus.is_empty()).then(|| PauseReason::Menus {
            players: self.menus.iter().copied().collect(),
        })
    }
}
/// Multipliers remain individual observations. Resource policy only selects the
/// best racial contribution; equipment bonuses must not be folded into this list.
pub fn best_racial_bonus(bonuses: impl IntoIterator<Item = f64>) -> Result<f64> {
    bonuses.into_iter().try_fold(0.0_f64, |best, value| {
        if !value.is_finite() || !(0.0..=10.0).contains(&value) {
            return Err("Invalid racial resource bonus".into());
        }
        Ok(best.max(value))
    })
}

/// Derive the current group barrier from native-owned actor states. Downed and
/// inactive actors never hold a reward barrier; menu waits remain distinct.
pub fn group_pause(
    actors: impl IntoIterator<Item = (PlayerId, crate::Phase)>,
) -> Option<PauseReason> {
    let mut waiting = Vec::new();
    let mut menus = Vec::new();
    for (id, phase) in actors {
        match phase {
            crate::Phase::Choosing => waiting.push(id),
            crate::Phase::Paused => menus.push(id),
            _ => {}
        }
    }
    if !waiting.is_empty() {
        Some(PauseReason::Choices {
            kind: ChoiceKind::Reward,
            waiting,
        })
    } else if !menus.is_empty() {
        Some(PauseReason::Menus { players: menus })
    } else {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PausePresentation {
    pub active: bool,
    pub visible: bool,
    pub local_menu: bool,
    pub title: String,
    pub detail: String,
}
/// Retain the native owner's choice/menu. Show a retained explanation only on
/// players waiting for someone else; downed actors keep their rescue interface.
pub fn pause_presentation(
    reason: Option<PauseReason>,
    local: crate::Phase,
    name_for: impl Fn(PlayerId) -> Option<String>,
) -> PausePresentation {
    let local_menu = local == crate::Phase::Paused;
    let Some(reason) = reason else {
        return PausePresentation::default();
    };
    let (title, players, action) = match reason {
        PauseReason::Choices { waiting, .. } => {
            ("Waiting for players", waiting, "choosing a reward")
        }
        PauseReason::Menus { players } => ("Game paused", players, "in the menu"),
    };
    let names = players
        .iter()
        .map(|id| {
            name_for(*id)
                .filter(|name| !name.trim().is_empty() && !name.chars().any(char::is_control))
                .map(|name| {
                    // Bound the native pause payload; preserve UTF-8 and render
                    // player text literally rather than as WoW color escapes.
                    let mut text = String::new();
                    for ch in name.trim().chars() {
                        let size = if ch == '|' { 2 } else { ch.len_utf8() };
                        if text.len() + size > 64 { break; }
                        text.push(ch);
                        if ch == '|' { text.push('|'); }
                    }
                    text
                })
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| format!("Player {id}"))
        })
        .collect::<Vec<_>>()
        .join(", ");
    PausePresentation {
        active: true,
        visible: !matches!(
            local,
            crate::Phase::Choosing
                | crate::Phase::Downed
                | crate::Phase::Inactive
                | crate::Phase::Paused
        ),
        local_menu,
        title: title.into(),
        detail: format!(
            "{names} {} {action}.",
            if players.len() == 1 { "is" } else { "are" }
        ),
    }
}

/// Personal non-racial XP bonuses remain additive; repeated innate bonuses
/// contribute only the strongest racial. Round the innate baseline as native f32
/// before subtraction, avoiding residual bonuses from identical Human actors.
pub fn shared_xp_multiplier(bonuses: impl IntoIterator<Item = (f32, f32)>) -> Result<f32> {
    let mut sum = 0.0f64;
    let mut racial = 0.0f32;
    let mut count = 0;
    for (native, innate) in bonuses {
        if !native.is_finite() || native < 0.0 || !innate.is_finite() || innate < 0.0 {
            return Err("Invalid XP bonus provenance".into());
        }
        count += 1;
        sum += (native - (1.0f32 + innate)) as f64;
        racial = racial.max(innate);
    }
    if !(1..=64).contains(&count) {
        return Err("Invalid shared XP membership".into());
    }
    let result = (1.0 + sum + f64::from(racial)).max(0.0) as f32;
    if !result.is_finite() {
        return Err("Shared XP multiplier overflow".into());
    }
    Ok(result)
}

/// Preserve the native award's rounding and one-point minimum. The caller must
/// supply an unscaled receipt, never reverse a previously rounded native award.
pub fn shared_xp_award(raw: u32, multiplier: f32) -> Result<u32> {
    if !multiplier.is_finite() || multiplier < 0.0 {
        return Err("Invalid shared XP multiplier".into());
    }
    let scaled = (raw as f32) * multiplier;
    if !scaled.is_finite() || f64::from(scaled) > f64::from(u32::MAX) {
        return Err("Shared XP award overflow".into());
    }
    Ok(scaled.round().max(1.0) as u32)
}

/// One authoritative wallet; native actor balances are projections of this value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedWallet {
    pub gold: u32,
    pub earned: u32,
}
impl SharedWallet {
    pub fn credit(self, amount: u32) -> Result<Self> {
        let gold = self
            .gold
            .checked_add(amount)
            .ok_or("Shared gold overflow")?;
        // Native coinage is a u32 measured in copper (10,000 per gold).
        gold.checked_mul(10000).ok_or("Native coinage overflow")?;
        Ok(Self {
            gold,
            earned: self
                .earned
                .checked_add(amount)
                .ok_or("Shared earned gold overflow")?,
        })
    }
    pub fn spend(self, amount: u32) -> Result<Self> {
        Ok(Self {
            gold: self
                .gold
                .checked_sub(amount)
                .ok_or("Insufficient shared gold")?,
            ..self
        })
    }
    pub fn native_debit(self, balance: u32) -> Result<Self> {
        self.spend(
            self.gold
                .checked_sub(balance)
                .ok_or("Unclassified native gold gain")?,
        )
    }
}
/// Replace only the collector's permanent equipment contribution by the best
/// participant's contribution. Racial, jewelry and fortune handling stays separate.
pub fn shared_gold_equipment_multiplier(
    collector:f32,collector_equipment:f32,participants:impl IntoIterator<Item=f32>,
)->Result<f32>{
    if !collector.is_finite() || collector<0.0 || !collector_equipment.is_finite(){return Err("Invalid equipment gold provenance".into())}
    let mut best=f32::NEG_INFINITY;let mut count=0;
    for bonus in participants{
        if !bonus.is_finite(){return Err("Invalid participant equipment gold bonus".into())}
        best=best.max(bonus);count+=1;
        if count>64{return Err("Invalid equipment gold membership".into())}
    }
    if count==0{return Err("Missing equipment gold membership".into())}
    let result=(f64::from(collector)-f64::from(collector_equipment)+f64::from(best)).max(0.0)as f32;
    if !result.is_finite(){return Err("Equipment gold multiplier overflow".into())}
    Ok(result)
}

pub fn shared_gold_award(
    raw: u32,
    collector: f32,
    collector_racial: f32,
    best_racial: f32,
    members: usize,
    source: Award,
) -> Result<u32> {
    if !(1..=64).contains(&members)
        || [collector, collector_racial, best_racial]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err("Invalid shared gold provenance".into());
    }
    let effective = (f64::from(collector) - f64::from(collector_racial) + f64::from(best_racial))
        .max(0.0) as f32;
    let amount = (raw as f32) * effective;
    if !amount.is_finite() || f64::from(amount) > f64::from(u32::MAX) {
        return Err("Shared gold award overflow".into());
    }
    let amount = amount.round() as u32;
    Ok(if source == Award::ConvertedReward {
        amount / (members as u32)
    } else {
        amount
    })
}

/// Already normalized shared XP withheld only while an actor is downed.
/// Keys include actor life identity; the native adapter owns all context binding.
#[derive(Default)]
pub struct DeferredExperience {
    pending: BTreeMap<(PlayerId, u64), u32>,
}
impl DeferredExperience {
    pub fn credit(&mut self, player: PlayerId, life: u64, amount: u32) -> Result<u32> {
        if !(1..=64).contains(&player) || life == 0 {
            return Err("Invalid deferred XP identity".into());
        }
        let current = self.pending.get(&(player, life)).copied().unwrap_or(0);
        let next = current.checked_add(amount).ok_or("Deferred XP overflow")?;
        if next > 0 {
            self.pending.insert((player, life), next);
        }
        Ok(next)
    }
    pub fn retain(&mut self, roster: &[(PlayerId, u64)]) {
        self.pending.retain(|key, _| roster.contains(key));
    }
    pub fn take(&mut self, player: PlayerId, life: u64) -> u32 {
        self.pending.remove(&(player, life)).unwrap_or(0)
    }
}

/// Participant-scoped menu ownership, separate from native reward phases.
#[derive(Default)]
pub struct MenuOwners(std::collections::BTreeSet<(u32,u64)>);
impl MenuOwners {
 pub fn set(&mut self,id:u32,guid:u64,open:bool){if id>=5||guid==0{return}if open{self.0.insert((id,guid));}else{self.0.remove(&(id,guid));}}
 pub fn contains(&self,id:u32,guid:u64)->bool{self.0.contains(&(id,guid))}
 pub fn retain(&mut self,actors:impl IntoIterator<Item=(u32,u64,crate::Phase)>){if self.0.is_empty(){return}let live:std::collections::BTreeSet<_>=actors.into_iter().filter(|(_,_,p)|matches!(p,crate::Phase::Playing|crate::Phase::Choosing|crate::Phase::Paused)).map(|(id,g,_)|(id,g)).collect();self.0.retain(|a|live.contains(a));}
}
#[cfg(test)]mod menu_owner_tests{
 use super::*;
 #[test]fn menus_close_independently_and_repeated_requests_are_idempotent(){let mut m=MenuOwners::default();m.set(0,10,true);m.set(1,20,true);m.set(1,20,true);m.set(0,10,false);assert!(!m.contains(0,10));assert!(m.contains(1,20));m.set(1,20,false);assert!(!m.contains(1,20));}
 #[test]fn departed_downed_and_reused_slots_cannot_keep_the_group_paused(){let mut m=MenuOwners::default();for(id,g)in[(0,10),(1,20),(2,30)]{m.set(id,g,true)}m.retain([(0,10,crate::Phase::Downed),(1,21,crate::Phase::Playing)]);assert!(!m.contains(0,10));assert!(!m.contains(1,20));assert!(!m.contains(2,30));}
 #[test]fn choices_keep_priority_when_a_menu_is_open(){assert!(matches!(group_pause([(1,crate::Phase::Paused),(2,crate::Phase::Choosing)]),Some(PauseReason::Choices{..})));}
}
