//! Host-owned pursuit/basic-attack ownership. Special spell targeting is absent
//! from this API: those spells deliberately retain the game's own selection.
use crate::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub enemy: EnemyId,
    pub player: PlayerId,
    pub spell: TauntSpell,
    pub life: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub player: PlayerId,
    pub life: u64,
    pub spell: TauntSpell,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Applied {
        hit: Hit,
        previous: Option<PlayerId>,
    },
    Cleared {
        enemy: EnemyId,
        player: PlayerId,
    },
}
#[derive(Clone, Default)]
pub struct Taunts {
    pub bindings: BTreeMap<EnemyId, Binding>,
}
impl Taunts {
    pub fn eligible(talents: Talents, spell: TauntSpell) -> bool {
        talents.allows(spell)
    }
    pub fn target(&self, enemy: EnemyId, party: &[Player]) -> Option<PlayerId> {
        self.target_if(enemy, |owner| {
            party.iter().any(|p| p.id == owner && p.phase.can_tank())
        })
    }
    /// Allocation-free synchronous predicate for native pursuit/basic-hit callbacks.
    pub fn target_if(
        &self,
        enemy: EnemyId,
        eligible: impl FnOnce(PlayerId) -> bool,
    ) -> Option<PlayerId> {
        self.bindings
            .get(&enemy)
            .and_then(|b| eligible(b.player).then_some(b.player))
    }
    pub fn settle(
        &mut self,
        party: &[Player],
        enemies: &[Enemy],
        hits: impl IntoIterator<Item = Hit>,
    ) -> Vec<Change> {
        let live: BTreeSet<_> = enemies.iter().filter(|e| e.alive).map(|e| e.id).collect();
        let valid = |owner: PlayerId, life: u64| {
            party
                .iter()
                .any(|p| p.id == owner && p.phase.can_tank() && p.life == life)
        };
        let mut changes = vec![];
        self.bindings.retain(|enemy, b| {
            let keep = live.contains(enemy) && valid(b.player, b.life);
            if !keep {
                changes.push(Change::Cleared {
                    enemy: *enemy,
                    player: b.player,
                });
            }
            keep
        });
        for hit in hits {
            if !live.contains(&hit.enemy) || !valid(hit.player, hit.life) {
                continue;
            }
            let old = self.bindings.insert(
                hit.enemy,
                Binding {
                    player: hit.player,
                    life: hit.life,
                    spell: hit.spell,
                },
            );
            if old.map(|b| b.player) != Some(hit.player) {
                changes.push(Change::Applied {
                    hit,
                    previous: old.map(|b| b.player),
                });
            }
        }
        changes
    }
    pub fn pursuit(&self, party: &[Player]) -> Vec<Pursuit> {
        self.bindings
            .keys()
            .filter_map(|enemy| {
                self.target(*enemy, party).map(|player| Pursuit {
                    enemy: *enemy,
                    player,
                })
            })
            .collect()
    }
}
