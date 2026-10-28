//! Bounded selected-native calls used by the deferred XP coordinator.
use super::super::{io, Runtime, RUNTIME};
use asx_game::official::PartyObservation;

pub(in crate::runtime) trait Services {
    fn deferred_actor(&self, game: u64) -> Result<Option<(u32, u64)>, String>;
    fn world_party(&self, game: u64) -> Result<Option<PartyObservation>, String>;
    fn replay(&self, game: u64, slot: u32, guid: u64, amount: u32) -> Result<bool, String>;
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
}

pub(super) fn active() -> Option<&'static dyn Services> {
    RUNTIME.get().filter(|runtime| runtime.rules).map(|runtime| runtime as &dyn Services)
}

impl Services for Runtime {
    fn deferred_actor(&self, game: u64) -> Result<Option<(u32, u64)>, String> {
        self.adapter.deferred_xp_actor(&self.local, game)
    }
    fn world_party(&self, game: u64) -> Result<Option<PartyObservation>, String> {
        self.adapter.world_party(&self.local, game)
    }
    fn replay(&self, game: u64, slot: u32, guid: u64, amount: u32) -> Result<bool, String> {
        self.adapter.replay_experience(
            &io::Access { local: &self.local, adapter: self.adapter }, game, slot, guid, amount,
        )
    }
    fn info(&self, message: &str) { self.log.info(message); }
    fn warn(&self, message: &str) { self.log.warn(message); }
}
