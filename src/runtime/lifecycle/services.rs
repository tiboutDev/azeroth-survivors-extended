//! Owned observations and cache mutations at feature lifetime boundaries.
use super::super::Runtime;
use asx_game::official::PartyObservation;

pub(in crate::runtime) trait Services {
    fn solo_run(&self, game: u64) -> Result<Option<PartyObservation>, String>;
    fn detach_coop(&self) -> bool;
    fn store_solo(&self, snapshot: Option<PartyObservation>);
}

impl Services for Runtime {
    fn solo_run(&self, game: u64) -> Result<Option<PartyObservation>, String> {
        self.adapter.solo_run(&self.local, game)
    }
    fn detach_coop(&self) -> bool {
        self.party.lock().ok().is_some_and(|mut party| party.take().is_some())
    }
    fn store_solo(&self, snapshot: Option<PartyObservation>) {
        if let Ok(mut solo) = self.solo.lock() { *solo = snapshot; }
    }
}
