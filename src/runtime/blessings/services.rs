//! Bounded native calls used by blessing reward and stat coordination.
#[cfg(feature = "test-hooks")]
use super::super::duplicate_loot;
use super::super::{io, Runtime, RUNTIME};
use asx_game::official::PartyObservation;
use asx_gameplay::blessings::Blessing;

pub(in crate::runtime) trait Services {
    fn extension_party(&self) -> Option<PartyObservation>;
    fn event_context(&self, registers: [u64; 3]) -> Result<Option<(u64, u64)>, String>;
    fn duplicate_loot_enabled(&self) -> bool;
    fn fixture_loot(&self, game: u64);
    fn pool_index(&self, frame: u64, index: u64) -> Result<u64, String>;
    fn refresh_blessings(&self, game: u64, actors: &[(u32, u64)]) -> Result<(), String>;
    fn run_source(&self, game: u64) -> Result<Option<(u32, u64)>, String>;
    fn apply_blessing_stats(&self, game: u64, blessing: Option<Blessing>) -> Result<(), String>;
    fn blessing_stat_values(&self, game: u64) -> Result<[f32; 5], String>;
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
    fn extension_party(&self) -> Option<PartyObservation> { Runtime::extension_party(self) }

    fn event_context(&self, registers: [u64; 3]) -> Result<Option<(u64, u64)>, String> {
        self.adapter.blessing_event_context(&self.local, registers, self.local.base as u64)
    }

    fn duplicate_loot_enabled(&self) -> bool { self.adapter.duplicate_loot_enabled() }

    #[cfg(feature = "test-hooks")]
    fn fixture_loot(&self, game: u64) { duplicate_loot::fixture_loot(self, game); }
    #[cfg(not(feature = "test-hooks"))]
    fn fixture_loot(&self, _: u64) {}

    fn pool_index(&self, frame: u64, index: u64) -> Result<u64, String> {
        self.adapter.blessing_pool_index(&self.local, frame, index)
    }

    fn refresh_blessings(&self, game: u64, actors: &[(u32, u64)]) -> Result<(), String> {
        self.adapter.refresh_blessings(
            &io::Access { local: &self.local, adapter: self.adapter },
            game,
            actors,
        )
    }

    fn run_source(&self, game: u64) -> Result<Option<(u32, u64)>, String> {
        self.adapter.run_source(&self.local, game)
    }

    fn apply_blessing_stats(&self, game: u64, blessing: Option<Blessing>) -> Result<(), String> {
        self.adapter.blessing_stats(
            &io::Access { local: &self.local, adapter: self.adapter },
            game,
            blessing,
        )
    }

    fn blessing_stat_values(&self, game: u64) -> Result<[f32; 5], String> {
        self.adapter.blessing_stat_values(&self.local, game)
    }

    fn info(&self, message: &str) { self.log.info(message); }
    fn warn(&self, message: &str) { self.log.warn(message); }
}

#[cfg(test)]
mod tests;
