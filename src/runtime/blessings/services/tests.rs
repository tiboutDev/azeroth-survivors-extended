use super::*;
use super::super::{finish, reset, view, State, STATE};
use crate::test_support::TEST_LOCK;
use asx_game::official::{ActorObservation, PartyObservation};
use asx_gameplay::Phase;
use std::cell::{Cell, RefCell};

struct Fake {
    party: PartyObservation,
    reenter: Cell<bool>,
    refreshes: RefCell<Vec<(u64, Vec<(u32, u64)>)>>,
}

impl Services for Fake {
    fn extension_party(&self) -> Option<PartyObservation> { Some(self.party.clone()) }
    fn event_context(&self, _: [u64; 3]) -> Result<Option<(u64, u64)>, String> { Ok(None) }
    fn duplicate_loot_enabled(&self) -> bool { false }
    fn fixture_loot(&self, _: u64) {}
    fn pool_index(&self, _: u64, index: u64) -> Result<u64, String> { Ok(index) }
    fn refresh_blessings(&self, game: u64, actors: &[(u32, u64)]) -> Result<(), String> {
        self.refreshes.borrow_mut().push((game, actors.to_vec()));
        if self.reenter.replace(false) { finish(self, game); }
        Ok(())
    }
    fn run_source(&self, _: u64) -> Result<Option<(u32, u64)>, String> { Ok(None) }
    fn apply_blessing_stats(&self, _: u64, _: Option<Blessing>) -> Result<(), String> { Ok(()) }
    fn blessing_stat_values(&self, _: u64) -> Result<[f32; 5], String> { Ok([0.0; 5]) }
    fn info(&self, _: &str) {}
    fn warn(&self, _: &str) {}
}

#[test]
fn native_refresh_reentry_observes_committed_blessing_state() {
    let _guard = TEST_LOCK.lock().unwrap();
    reset();
    let party = PartyObservation {
        clock: 1.0,
        actors: vec![ActorObservation {
            slot: 0, guid: 10, phase: Phase::Playing, gold: 0, level: 1, xp: 0,
            class_id: 0, race_id: 0, xp_multiplier: 1.0, gold_multiplier: 1.0,
            hp: 1.0, maximum: 1.0, position: [0.0; 3],
        }],
    };
    let service = Fake { party, reenter: Cell::new(true), refreshes: RefCell::new(Vec::new()) };
    {
        let mut state = STATE.lock().unwrap();
        *state = Some(State { game: 7, pending: [1].into_iter().collect(), next_event: 1, ..Default::default() });
    }

    finish(&service, 7);

    assert_eq!(&*service.refreshes.borrow(), &vec![(7, vec![(0, 10)])]);
    assert!(view(0, 10).is_some());
    assert!(STATE.lock().unwrap().as_ref().unwrap().pending.is_empty());
    reset();
}
