use super::*;
use crate::test_support::TEST_LOCK;
use asx_game::official::{ActorObservation, PartyObservation, Role};
use asx_gameplay::{official::PauseReason, Phase};
use std::cell::{Cell, RefCell};

fn actor(slot: u32, guid: u64) -> ActorObservation {
    ActorObservation { slot, guid, phase: Phase::Playing, gold: 0, level: 1, xp: 0,
        class_id: 0, race_id: 0, xp_multiplier: 1.0, gold_multiplier: 1.0,
        hp: 1.0, maximum: 1.0, position: [0.0; 3] }
}
struct Fake {
    role: Cell<Role>, party: RefCell<PartyObservation>, wire: Cell<(u64, bool)>,
    sends: RefCell<Vec<Vec<u8>>>, fail_send: Cell<bool>, warnings: Cell<usize>, peers: Cell<usize>,
}
impl Fake {
    fn new(role: Role) -> Self {
        Self { role: Cell::new(role), party: RefCell::new(PartyObservation { clock: 1.0,
            actors: vec![actor(0, 100), actor(1, 200)] }), wire: Cell::new((0, true)),
            sends: RefCell::new(vec![]), fail_send: Cell::new(false), warnings: Cell::new(0), peers: Cell::new(0) }
    }
}
impl Services for Fake {
    fn key_target(&self) -> Option<usize> { None }
    fn has_menu(&self) -> bool { true }
    fn role(&self, _: u64) -> Result<Role, String> { Ok(self.role.get()) }
    fn cached_party(&self) -> Option<PartyObservation> { Some(self.party.borrow().clone()) }
    fn game_address(&self, _: u64) -> Result<u64, String> { Ok(10) }
    fn send(&self, _: u64, bytes: &[u8]) -> Result<(), String> {
        // Native transport can call back; no menu-state lock may span it.
        assert!(STATE.try_lock().is_ok());
        let party = self.party.borrow().clone();
        let _ = reason(self, 10, &party);
        self.sends.borrow_mut().push(bytes.to_vec());
        if self.fail_send.get() { Err("transport rejected".into()) } else { Ok(()) }
    }
    fn request(&self, _: u64) -> Result<Option<bool>, String> { Ok(None) }
    fn opcode(&self) -> Option<u16> { Some(0xabc) }
    fn peer(&self, _: u64, _: u64) -> Result<u32, String> { self.peers.set(self.peers.get()+1); Ok(1) }
    fn wire(&self, _: u64) -> Result<(u64, bool), String> { Ok(self.wire.get()) }
    fn party(&self, _: u64) -> Result<Option<PartyObservation>, String> { Ok(self.cached_party()) }
    fn info(&self, _: &str) {}
    fn warn(&self, _: &str) { self.warnings.set(self.warnings.get()+1); }
}
fn clear() { reset(); pause::set_active(false); }

#[test]
fn host_menu_ownership_is_personal_and_retired_with_actor_identity() {
    let _guard=TEST_LOCK.lock().unwrap();clear();
    let fake=Fake::new(Role::Host);
    request(&fake,1,true).unwrap();
    assert!(local_menu(0,100));assert!(!local_menu(1,200));
    assert_eq!(reason(&fake,10,&fake.party.borrow()),Some(PauseReason::Menus{players:vec![1]}));
    request(&fake,1,false).unwrap();assert!(!local_menu(0,100));
    request(&fake,1,true).unwrap();
    fake.party.borrow_mut().actors[0].guid=101;
    assert_eq!(reason(&fake,10,&fake.party.borrow()),None);
    assert!(!local_menu(0,100));assert!(!local_menu(0,101));clear();
}

#[test]
fn guest_menu_waits_for_epoch_and_commits_only_after_reentrant_send_succeeds() {
    let _guard=TEST_LOCK.lock().unwrap();clear();
    let fake=Fake::new(Role::Guest);
    request(&fake,1,true).unwrap();assert!(fake.sends.borrow().is_empty());assert!(!LOCAL_OPEN.load(Ordering::Acquire));
    session(&serde_json::json!({"_azcoop":{"pause":{"session":5}}}));
    pause::set_active(true);fake.fail_send.set(true);
    assert!(request(&fake,1,false).is_err());assert!(!LOCAL_OPEN.load(Ordering::Acquire));
    fake.fail_send.set(false);request(&fake,1,false).unwrap();
    let mut expected=5u64.to_le_bytes().to_vec();expected.push(1);
    assert_eq!(*fake.sends.borrow(),vec![expected.clone(),expected]);assert!(LOCAL_OPEN.load(Ordering::Acquire));
    request(&fake,1,false).unwrap();assert!(!LOCAL_OPEN.load(Ordering::Acquire));
    assert_eq!(fake.sends.borrow().last().unwrap().last(),Some(&0));
    clear();
}

#[test]
fn peer_menu_requires_native_origin_current_epoch_and_live_owner() {
    let _guard=TEST_LOCK.lock().unwrap();clear();
    let fake=Fake::new(Role::Host);let _=reason(&fake,10,&fake.party.borrow());
    let mut c:crate::stub::Context=unsafe{std::mem::zeroed()};c.r9=0xabc;c.rdx=0;
    assert!(command(&fake,&mut c));assert_eq!((c.skip,c.result),(1,0));assert_eq!(fake.peers.get(),0);
    c.rdx=1;fake.wire.set((epoch().wrapping_add(1),true));assert!(command(&fake,&mut c));assert!(!local_menu(1,200));
    fake.wire.set((epoch(),true));assert!(command(&fake,&mut c));assert!(local_menu(1,200));
    fake.party.borrow_mut().actors[1].phase=Phase::Inactive;
    assert!(command(&fake,&mut c));let _=reason(&fake,10,&fake.party.borrow());assert!(!local_menu(1,200));
    let warnings=fake.warnings.get();c.r9=0xff;
    assert!(!command(&fake,&mut c));assert_eq!(fake.warnings.get(),warnings);clear();
}
