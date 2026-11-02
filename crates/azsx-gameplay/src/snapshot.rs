//! Immutable observations from one safe game boundary. IDs are opaque; no
//! native addresses or byte buffers can escape through this contract.
use crate::{
    support::{Member, TargetEnemy},
    EnemyId, Phase, PlayerId,
};
#[derive(Clone, Debug, PartialEq)]
pub struct EnemyState {
    pub id: EnemyId,
    pub hp: f64,
    pub position: [f64; 3],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progress {
    pub phase: Phase,
    pub level: u32,
    pub xp: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resources {
    pub gold: u32,
    pub earned: u32,
    pub next_chest: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GameSnapshot {
    pub generation: u64,
    pub observed_at: f64,
    pub local: PlayerId,
    pub players: Vec<Member>,
    pub enemies: Option<Vec<EnemyState>>,
    pub progress: Progress,
    pub resources: Option<Resources>,
}
impl GameSnapshot {
    pub fn pressure(&self) -> Option<std::collections::BTreeMap<PlayerId, u32>> {
        let e: Vec<_> = self
            .enemies
            .as_ref()?
            .iter()
            .map(|e| TargetEnemy {
                id: e.id,
                alive: e.hp > 0.,
                position: e.position,
            })
            .collect();
        Some(crate::support::pressure(&self.players, &e))
    }
    pub fn local_player(&self) -> Option<&Member> {
        self.players.iter().find(|p| p.id == self.local)
    }
}
