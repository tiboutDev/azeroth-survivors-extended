use super::*;
use crate::test_support::TEST_LOCK;
use std::cell::{Cell, RefCell};

struct Fake {
    calls: RefCell<Vec<(u32, u64)>>,
    reenter: Cell<bool>,
    rejected_guid: Cell<Option<u64>>,
}

impl Services for Fake {
    fn elite_game(&self, _: [u64; 3]) -> Result<Option<u64>, String> { Ok(None) }
    fn party_actors(&self) -> Vec<(u32, u64)> { Vec::new() }
    fn deliver_item(&self, game: u64, slot: u32, guid: u64) -> Result<bool, String> {
        assert_eq!(game, 7);
        self.calls.borrow_mut().push((slot, guid));
        if self.reenter.replace(false) {
            finish(self, game);
        }
        Ok(self.rejected_guid.get() != Some(guid))
    }
    fn info(&self, _: &str) {}
    fn warn(&self, _: &str) {}
}

fn fake() -> Fake {
    Fake { calls: RefCell::new(Vec::new()), reenter: Cell::new(false), rejected_guid: Cell::new(None) }
}

fn enqueue() {
    let mut pending = PENDING.lock().unwrap();
    pending.clear();
    pending.push(Reward { game: 7, players: vec![(0, 10), (1, 20)] });
}

#[test]
fn native_delivery_can_reenter_after_pending_work_is_detached() {
    let _guard = TEST_LOCK.lock().unwrap();
    enqueue();
    let service = fake();
    service.reenter.set(true);

    finish(&service, 7);

    assert_eq!(service.calls.borrow().as_slice(), &[(0, 10), (1, 20)]);
    assert!(PENDING.lock().unwrap().is_empty());
}

#[test]
fn successful_actor_commit_is_kept_when_the_next_actor_is_rejected() {
    let _guard = TEST_LOCK.lock().unwrap();
    enqueue();
    let service = fake();
    service.rejected_guid.set(Some(20));

    finish(&service, 7);
    assert_eq!(service.calls.borrow().as_slice(), &[(0, 10), (1, 20)]);
    assert_eq!(PENDING.lock().unwrap()[0].players.as_slice(), &[(1, 20)]);

    service.rejected_guid.set(None);
    finish(&service, 7);
    assert_eq!(service.calls.borrow().as_slice(), &[(0, 10), (1, 20), (1, 20)]);
    assert!(PENDING.lock().unwrap().is_empty());
}
