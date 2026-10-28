//! Mark lifetime, non-stacking strength and bounded presentation requests.
use crate::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Default)]
pub struct Marks {
    pub players: BTreeMap<PlayerId, Mark>,
    applied: BTreeSet<EnemyId>,
    visuals: BTreeMap<EnemyId, bool>,
}
impl Marks {
    pub fn effective(&self, party: &[Player]) -> BTreeMap<EnemyId, f32> {
        let mut out = BTreeMap::<EnemyId, f32>::new();
        for (owner, mark) in &self.players {
            if let Some(enemy) = mark.enemy {
                if party
                    .iter()
                    .any(|p| p.id == *owner && p.phase.retains_mark())
                {
                    let value = out.entry(enemy).or_default();
                    *value = value.max(mark.strength);
                }
            }
        }
        out
    }
    pub fn needs_world(&self, party: &[Player]) -> bool {
        !self.effective(party).is_empty() || !self.applied.is_empty() || !self.visuals.is_empty()
    }
    pub fn reconcile(&mut self, party: &[Player], enemies: &[Enemy]) -> Vec<MarkWrite> {
        let effective = self.effective(party);
        let mut writes = Vec::new();
        for enemy in enemies {
            if !enemy.alive {
                self.visuals.remove(&enemy.id);
                continue;
            }
            let desired = effective.contains_key(&enemy.id);
            if desired && !enemy.marked && enemy.aura_full {
                self.visuals.remove(&enemy.id);
            } else if enemy.marked != desired
                && (self.visuals.contains_key(&enemy.id) || self.visuals.len() < 128)
            {
                self.visuals.insert(enemy.id, desired);
            } else if enemy.marked == desired {
                self.visuals.remove(&enemy.id);
            }
            if !desired && !self.applied.contains(&enemy.id) {
                continue;
            }
            let strength = effective.get(&enemy.id).copied().unwrap_or(0.);
            if enemy.vulnerability != strength {
                writes.push(MarkWrite {
                    enemy: enemy.id,
                    strength,
                });
            }
        }
        self.applied = effective.keys().copied().collect();
        writes
    }
    pub fn next_visual(&mut self) -> Option<MarkVisual> {
        self.visuals
            .pop_first()
            .map(|(enemy, active)| MarkVisual { enemy, active })
    }
}
