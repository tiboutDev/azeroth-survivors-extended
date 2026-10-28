use super::*;
use crate::test_support::TEST_LOCK;
use asx_gameplay::{official::PausePresentation, Phase};
use serde_json::Value;
use std::{cell::{Cell, RefCell}, sync::atomic::Ordering};

struct FakeServices {
    read: RefCell<Result<Value, String>>,
    decoded: RefCell<Result<Option<PausePresentation>, String>>,
    notice: Option<&'static str>,
    phase: Phase,
    duplicate: bool,
    ended: Cell<Option<(bool, bool, bool)>>,
    warnings: Cell<u32>,
    #[cfg(feature = "test-hooks")]
    fixture_calls: Cell<u32>,
}
impl FakeServices {
    fn new(value: Value, decoded: Result<Option<PausePresentation>, String>) -> Self {
        Self { read: RefCell::new(Ok(value)), decoded: RefCell::new(decoded), notice: Some("native pause"),
            phase: Phase::Playing, duplicate: false, ended: Cell::new(None), warnings: Cell::new(0),
            #[cfg(feature = "test-hooks")] fixture_calls: Cell::new(0) }
    }
}
impl Services for FakeServices {
    fn read_ui(&self, _: u64, _: u64) -> Result<Value, String> { self.read.borrow().clone() }
    fn decode_pause(&self, _: &Value) -> Result<Option<PausePresentation>, String> { self.decoded.borrow().clone() }
    fn pause_notice(&self, _: bool) -> Option<&'static str> { self.notice }
    fn hud_phase(&self, _: &Value) -> Result<Phase, String> { Ok(self.phase) }
    fn duplicate_loot_enabled(&self) -> bool { self.duplicate }
    fn end_session(&self) { self.ended.set(Some((PAUSED.load(Ordering::Acquire), UI_INPUT_PAUSED.load(Ordering::Acquire), HOST_PAUSE_AUTHORITY.load(Ordering::Acquire)))); }
    fn warn(&self, _: &str) { self.warnings.set(self.warnings.get()+1); }
    #[cfg(feature = "test-hooks")]
    fn economy_hud(&self, _: &Value) { self.fixture_calls.set(self.fixture_calls.get()+1); }
    #[cfg(feature = "test-hooks")]
    fn fixture_event(&self, _: &mut crate::stub::Context, _: &Value) { self.fixture_calls.set(self.fixture_calls.get()+1); }
}
fn clear_flags() {
    PAUSED.store(false, Ordering::Release);
    UI_INPUT_PAUSED.store(false, Ordering::Release);
    HOST_PAUSE_AUTHORITY.store(false, Ordering::Release);
}
fn view(active: bool) -> PausePresentation {
    PausePresentation { active, visible: active, ..Default::default() }
}
fn context() -> crate::stub::Context {
    let mut c: crate::stub::Context = unsafe { std::mem::zeroed() };
    c.rdx = 10; c.r8 = 20; c
}

#[test]
fn partial_hud_does_not_release_pause_and_authoritative_guest_resume_does() {
    let _guard=TEST_LOCK.lock().unwrap(); clear_flags();
    PAUSED.store(true, Ordering::Release);
    let partial=FakeServices::new(Value::Null,Ok(None));
    handle_hud(&partial,&context());
    assert!(PAUSED.load(Ordering::Acquire),"partial HUD has no resume authority");

    UI_INPUT_PAUSED.store(true,Ordering::Release);
    let guest=FakeServices::new(Value::Null,Ok(Some(view(false))));
    handle_hud(&guest,&context());
    assert!(!PAUSED.load(Ordering::Acquire),"complete guest HUD resumes its group barrier");
    assert!(!UI_INPUT_PAUSED.load(Ordering::Acquire));

    clear_flags();
    HOST_PAUSE_AUTHORITY.store(true,Ordering::Release);
    PAUSED.store(true,Ordering::Release);
    UI_INPUT_PAUSED.store(true,Ordering::Release);
    let host=FakeServices::new(Value::Null,Ok(Some(view(false))));
    handle_hud(&host,&context());
    assert!(PAUSED.load(Ordering::Acquire),"host HUD cannot release host-owned group barrier");
    clear_flags();
}

#[test]
fn local_unpause_is_consumed_before_fixture_and_policy_work() {
    let _guard=TEST_LOCK.lock().unwrap();clear_flags();
    PAUSED.store(true,Ordering::Release);UI_INPUT_PAUSED.store(true,Ordering::Release);
    let service=FakeServices::new(serde_json::json!({"type":"unpaused"}),Ok(None));
    let mut c=context();c.result=99;
    handle_event(&service,&mut c);
    assert_eq!((c.skip,c.result),(1,0));
    #[cfg(feature = "test-hooks")]
    assert_eq!(service.fixture_calls.get(),0,"local unpause exits before fixture dispatch");
    assert!(PAUSED.load(Ordering::Acquire));assert!(UI_INPUT_PAUSED.load(Ordering::Acquire));
    clear_flags();
}

#[test]
fn session_end_observes_pause_authority_before_owner_clears_flags() {
    let _guard=TEST_LOCK.lock().unwrap();clear_flags();
    PAUSED.store(true,Ordering::Release);UI_INPUT_PAUSED.store(true,Ordering::Release);HOST_PAUSE_AUTHORITY.store(true,Ordering::Release);
    let service=FakeServices::new(serde_json::json!({"type":"coop","state":"ended"}),Ok(None));
    handle_event(&service,&mut context());
    assert_eq!(service.ended.get(),Some((true,true,true)));
    assert!(!PAUSED.load(Ordering::Acquire));assert!(!UI_INPUT_PAUSED.load(Ordering::Acquire));assert!(!HOST_PAUSE_AUTHORITY.load(Ordering::Acquire));
    clear_flags();
}

#[test]
fn read_and_decode_errors_preserve_pause_authority() {
    let _guard=TEST_LOCK.lock().unwrap();clear_flags();
    PAUSED.store(true,Ordering::Release);HOST_PAUSE_AUTHORITY.store(true,Ordering::Release);
    let service=FakeServices::new(Value::Null,Err("invalid HUD".into()));
    handle_hud(&service,&context());
    assert!(PAUSED.load(Ordering::Acquire));
    assert_eq!(service.warnings.get(),1);
    *service.read.borrow_mut()=Err("invalid event".into());
    handle_event(&service,&mut context());
    assert!(PAUSED.load(Ordering::Acquire));assert_eq!(service.ended.get(),None);
    clear_flags();
}
