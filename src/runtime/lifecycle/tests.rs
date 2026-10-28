use super::*;
use crate::test_support::TEST_LOCK;
use asx_game::official::PartyObservation;
use std::cell::{Cell, RefCell};

struct Fake {
    snapshot: RefCell<Result<Option<PartyObservation>,String>>, coop: Cell<bool>,
    events: RefCell<Vec<&'static str>>, stored: RefCell<Option<PartyObservation>>,
}
impl Services for Fake {
    fn solo_run(&self, _:u64)->Result<Option<PartyObservation>,String>{self.events.borrow_mut().push("observe");self.snapshot.borrow().clone()}
    fn detach_coop(&self)->bool{self.events.borrow_mut().push("detach");self.coop.replace(false)}
    fn store_solo(&self,p:Option<PartyObservation>){self.events.borrow_mut().push("store");*self.stored.borrow_mut()=p;}
}
#[test]
fn only_a_confirmed_solo_snapshot_detaches_coop_before_storing() {
    let _guard=TEST_LOCK.lock().unwrap();
    let fake=Fake{snapshot:RefCell::new(Ok(None)),coop:Cell::new(true),events:RefCell::new(vec![]),stored:RefCell::new(None)};
    refresh_solo(&fake,10);assert!(fake.coop.get());assert_eq!(*fake.events.borrow(),["observe","store"]);
    fake.events.borrow_mut().clear();*fake.snapshot.borrow_mut()=Err("unavailable".into());
    refresh_solo(&fake,10);assert!(fake.coop.get());assert!(fake.stored.borrow().is_none());
    fake.events.borrow_mut().clear();*fake.snapshot.borrow_mut()=Ok(Some(PartyObservation{clock:1.0,actors:vec![]}));
    refresh_solo(&fake,10);assert!(!fake.coop.get());assert_eq!(*fake.events.borrow(),["observe","detach","store"]);
    assert_eq!(fake.stored.borrow().as_ref().unwrap().clock,1.0);
    end(&fake);assert!(fake.stored.borrow().is_none());
}
