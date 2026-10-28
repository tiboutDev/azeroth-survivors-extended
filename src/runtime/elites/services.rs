//! Bounded native calls used by elite item reward coordination.
use super::super::{io, Runtime, RUNTIME};

pub(in crate::runtime) trait Services {
    fn elite_game(&self, registers: [u64; 3]) -> Result<Option<u64>, String>;
    fn party_actors(&self) -> Vec<(u32, u64)>;
    fn deliver_item(&self, game: u64, slot: u32, guid: u64) -> Result<bool, String>;
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
}

pub(super) fn active() -> Option<&'static dyn Services> {
    RUNTIME
        .get()
        .filter(|runtime| runtime.rules)
        .map(|runtime| runtime as &dyn Services)
}

impl Services for Runtime {
    fn elite_game(&self, registers: [u64; 3]) -> Result<Option<u64>, String> {
        self.adapter.elite_game(&self.local, registers)
    }

    fn party_actors(&self) -> Vec<(u32, u64)> {
        self.party
            .lock()
            .ok()
            .and_then(|party| party.as_ref().map(|party| party.actors.iter().map(|a| (a.slot, a.guid)).collect()))
            .unwrap_or_default()
    }

    fn deliver_item(&self, game: u64, slot: u32, guid: u64) -> Result<bool, String> {
        self.adapter.deliver_elite_item(
            &io::Access { local: &self.local, adapter: self.adapter },
            game,
            slot,
            guid,
        )
    }

    fn info(&self, message: &str) { self.log.info(message); }
    fn warn(&self, message: &str) { self.log.warn(message); }
}
