//! Production callbacks with native stand-ins and no installed runtime.
use super::*;
use crate::test_support::TEST_LOCK;
use std::cell::{Cell, RefCell};

thread_local! {
    static OBSERVED: RefCell<Vec<(usize, u64)>> = const { RefCell::new(Vec::new()) };
}

unsafe extern "system" fn native_world(game: *mut u8, dt: f32) {
    OBSERVED.with(|seen| seen.borrow_mut().push((game as usize, dt.to_bits() as u64)));
}
#[test]
fn world_pass_forwards_native_game_and_only_zeroes_paused_time() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert!(services::world().is_none() && services::hud().is_none() && services::command().is_none());
    let prior_pause = pause::set_active(false);
    let prior_host = pause::host_authority(); pause::set_host_authority(false);
    let prior = WORLD_ORIGINAL.swap(native_world as *const () as usize, Ordering::AcqRel);
    OBSERVED.with(|seen| seen.borrow_mut().clear());
    unsafe { world(0x12340 as *mut u8, 0.125); }
    pause::set_active(true);
    unsafe { world(0x12340 as *mut u8, 0.25); }
    WORLD_ORIGINAL.store(prior, Ordering::Release);
    pause::set_active(prior_pause); pause::set_host_authority(prior_host);
    OBSERVED.with(|seen| assert_eq!(*seen.borrow(), [(0x12340, 0.125f32.to_bits() as u64), (0x12340, 0)]));
}

unsafe extern "system" fn native_update(server: *mut u8, now: f64) {
    OBSERVED.with(|seen| seen.borrow_mut().push((server as usize, now.to_bits())));
}
#[test]
fn server_update_preserves_native_arguments_and_counts_completed_calls() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert!(services::world().is_none() && services::hud().is_none() && services::command().is_none());
    let prior = ORIGINAL.swap(native_update as *const () as usize, Ordering::AcqRel);
    let count = CALLS.load(Ordering::Relaxed);
    OBSERVED.with(|seen| seen.borrow_mut().clear());
    unsafe { update(0x23450 as *mut u8, 13.75); }
    ORIGINAL.store(prior, Ordering::Release);
    assert_eq!(CALLS.load(Ordering::Relaxed), count + 1);
    OBSERVED.with(|seen| assert_eq!(*seen.borrow(), [(0x23450, 13.75f64.to_bits())]));
}

unsafe extern "system" fn native_hud(output: *mut u8, game: *mut u8) -> *mut u8 {
    OBSERVED.with(|seen| seen.borrow_mut().push((output as usize, game as u64)));
    0x98760 as *mut u8
}
#[test]
fn hud_pass_preserves_borrowed_pointers_and_native_result() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert!(services::world().is_none() && services::hud().is_none() && services::command().is_none());
    let prior = HUD_ORIGINAL.swap(native_hud as *const () as usize, Ordering::AcqRel);
    OBSERVED.with(|seen| seen.borrow_mut().clear());
    let result = unsafe { hud(0x34560 as *mut u8, 0x45670 as *mut u8) };
    HUD_ORIGINAL.store(prior, Ordering::Release);
    assert_eq!(result as usize, 0x98760);
    OBSERVED.with(|seen| assert_eq!(*seen.borrow(), [(0x34560, 0x45670)]));
}

unsafe extern "system" fn native_command(a: u64, b: u64, c: u64, d: u64, e: u64) -> u64 {
    a + 10 * b + 100 * c + 1000 * d + 10000 * e
}
#[test]
fn moved_command_stub_preserves_register_and_stack_arguments() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert!(services::world().is_none() && services::hud().is_none() && services::command().is_none());
    let prior = COMMAND_ORIGINAL.swap(native_command as *const () as usize, Ordering::AcqRel);
    let call: unsafe extern "system" fn(u64, u64, u64, u64, u64) -> u64 =
        unsafe { std::mem::transmute(command_entry as *const ()) };
    let result = unsafe { call(1, 2, 3, 4, 5) };
    COMMAND_ORIGINAL.store(prior, Ordering::Release);
    assert_eq!(result, 54321);
}

struct WorldFake {
    enabled: bool,
    observation: RefCell<Result<Option<asx_gameplay::official::PauseReason>, String>>,
    calls: RefCell<Vec<&'static str>>,
    warnings: Cell<u32>,
}
impl WorldControl for WorldFake {
    fn rules_enabled(&self) -> bool { self.enabled }
    fn observe_world(&self, _: u64) -> Result<Option<asx_gameplay::official::PauseReason>, String> {
        self.calls.borrow_mut().push("observe");
        self.observation.borrow().clone()
    }
    fn capture_world(&self, _: u64) { self.calls.borrow_mut().push("capture"); }
    fn finish_world(&self, _: u64) { self.calls.borrow_mut().push("finish"); }
    fn log_group_pause(&self, _: &Option<asx_gameplay::official::PauseReason>) { self.calls.borrow_mut().push("pause-log"); }
    fn warn(&self, _: &str) { self.warnings.set(self.warnings.get() + 1); }
}
fn world_fake(enabled: bool, observation: Result<Option<asx_gameplay::official::PauseReason>, String>) -> WorldFake {
    WorldFake { enabled, observation: RefCell::new(observation), calls: RefCell::new(Vec::new()), warnings: Cell::new(0) }
}

#[test]
fn world_control_preserves_observe_capture_native_finish_order_and_host_gate() {
    let _guard = TEST_LOCK.lock().unwrap();
    let prior_active = pause::set_active(false);
    let prior_host = pause::host_authority();
    pause::set_host_authority(true);
    let runtime = world_fake(true, Ok(Some(asx_gameplay::official::PauseReason::Menus { players: vec![] })));
    let delta = pre_world(Some(&runtime), 100, 0.25);
    assert_eq!(delta, 0.0);
    runtime.calls.borrow_mut().push("native");
    finish_world(Some(&runtime), 100);
    assert_eq!(*runtime.calls.borrow(), ["observe", "pause-log", "capture", "native", "finish"]);

    runtime.calls.borrow_mut().clear();
    pause::set_host_authority(false);
    let delta = pre_world(Some(&runtime), 100, 0.5);
    assert_eq!(delta, 0.0, "host-only observation stays gated while the existing group pause remains active");
    assert_eq!(*runtime.calls.borrow(), ["capture"], "host-only pause observation stays gated");
    pause::set_active(prior_active);
    pause::set_host_authority(prior_host);
}

#[test]
fn world_observation_failure_releases_pause_but_keeps_native_pass_order() {
    let _guard = TEST_LOCK.lock().unwrap();
    let prior_active = pause::set_active(true);
    let prior_host = pause::host_authority(); pause::set_host_authority(true);
    let runtime = world_fake(true, Err("bad snapshot".into()));
    let delta = pre_world(Some(&runtime), 101, 0.75);
    assert_eq!(delta, 0.0, "failed observation forwards the pause value sampled at entry");
    runtime.calls.borrow_mut().push("native");
    finish_world(Some(&runtime), 101);
    assert_eq!(*runtime.calls.borrow(), ["observe", "capture", "native", "finish"]);
    assert_eq!(runtime.warnings.get(), 1);
    assert!(!pause::active(), "the group barrier is cleared after a failed host observation");
    pause::set_active(prior_active); pause::set_host_authority(prior_host);
}

#[derive(Default)]
struct CommandFake {
    rules: bool, duplicate: bool, menu_intercepts: bool, loot_intercepts: bool,
    movement: bool, active: bool, calls: RefCell<Vec<&'static str>>,
}
impl CommandServices for CommandFake {
    fn rules_enabled(&self) -> bool { self.rules }
    fn duplicate_enabled(&self) -> bool { self.duplicate }
    fn menu_command(&self, context: &mut crate::stub::Context) -> bool {
        self.calls.borrow_mut().push("menu");
        if self.menu_intercepts { context.skip = 1; context.result = 0; }
        self.menu_intercepts
    }
    fn duplicate_command(&self, context: &mut crate::stub::Context) -> bool {
        self.calls.borrow_mut().push("loot");
        if self.loot_intercepts { context.skip = 1; context.result = 0; }
        self.loot_intercepts
    }
    fn movement_opcode(&self, _: u16) -> bool { self.calls.borrow_mut().push("movement"); self.movement }
    fn active_role(&self, _: u64) -> bool { self.calls.borrow_mut().push("role"); self.active }
}

#[test]
fn command_filter_keeps_menu_then_loot_then_paused_movement_priority() {
    let _guard = TEST_LOCK.lock().unwrap();
    let prior = pause::set_active(true);
    let menu = CommandFake { rules: true, duplicate: true, menu_intercepts: true, movement: true, active: true, ..Default::default() };
    let mut context = new_context(); context.r9 = 0xb5;
    assert!(dispatch_command(&menu, &mut context));
    assert_eq!(*menu.calls.borrow(), ["menu"]);
    assert_eq!((context.skip, context.result), (1, 0));

    let loot = CommandFake { rules: true, duplicate: true, loot_intercepts: true, movement: true, active: true, ..Default::default() };
    let mut context = new_context(); context.r9 = 0xb5;
    assert!(dispatch_command(&loot, &mut context));
    assert_eq!(*loot.calls.borrow(), ["menu", "loot"]);

    let movement = CommandFake { rules: true, movement: true, active: true, ..Default::default() };
    let mut context = new_context(); context.r9 = 0xb5;
    assert!(dispatch_command(&movement, &mut context));
    assert_eq!(*movement.calls.borrow(), ["menu", "movement", "role"]);
    assert_eq!((context.skip, context.result), (1, 0));

    let ordinary = CommandFake { rules: true, movement: false, active: true, ..Default::default() };
    let mut context = new_context(); context.r9 = 0x100;
    assert!(!dispatch_command(&ordinary, &mut context));
    assert_eq!(*ordinary.calls.borrow(), ["menu", "movement"]);
    pause::set_active(prior);
}

fn new_context() -> crate::stub::Context { unsafe { std::mem::zeroed() } }

struct UpdateFake {
    enabled: bool,
    role: Cell<asx_game::official::Role>,
    events: RefCell<Vec<&'static str>>,
}
impl WorldControl for UpdateFake {
    fn rules_enabled(&self) -> bool { self.enabled }
    fn observe_world(&self, _:u64)->Result<Option<asx_gameplay::official::PauseReason>,String>{unreachable!()}
    fn capture_world(&self,_:u64){unreachable!()}
    fn finish_world(&self,_:u64){unreachable!()}
    fn log_group_pause(&self,_:&Option<asx_gameplay::official::PauseReason>){unreachable!()}
    fn warn(&self,_:&str){unreachable!()}
}
impl WorldServices for UpdateFake {
    fn duplicate_enabled(&self)->bool{true}
    fn observe_role(&self,_:u64)->Result<asx_game::official::Observation,String>{
        self.events.borrow_mut().push("role");
        Ok(asx_game::official::Observation{role:self.role.get(),guest_game_stream:None,remote_connections:0})
    }
    fn begin_duplicates(&self,_:u64){self.events.borrow_mut().push("dup-begin");}
    fn refresh_host(&self,_:u64){self.events.borrow_mut().push("refresh");}
    fn finish_experience(&self,_:u64){self.events.borrow_mut().push("xp");}
    #[cfg(feature="test-hooks")]
    fn fixture_before_update(&self,_:u64){}
    #[cfg(feature="test-hooks")]
    fn fixture_after_update(&self,_:u64){}
    fn finish_duplicates(&self,_:u64){self.events.borrow_mut().push("dup-finish");}
    fn emit_diagnostics(&self,_:u64){self.events.borrow_mut().push("diagnostic");}
}

#[test]
fn update_observes_role_again_after_native_before_host_work() {
    let _guard=TEST_LOCK.lock().unwrap();
    use asx_game::official::Role::{Guest,Host};
    let old=pause::host_authority();
    let fake=UpdateFake{enabled:true,role:Cell::new(Guest),events:RefCell::new(vec![])};
    update_pass(Some(&fake),10,||{
        assert!(!pause::host_authority());
        fake.events.borrow_mut().push("native");fake.role.set(Host);
    });
    assert_eq!(*fake.events.borrow(),["role","dup-begin","native","role","refresh","xp","dup-finish","diagnostic"]);
    fake.events.borrow_mut().clear();
    update_pass(Some(&fake),10,||{
        assert!(pause::host_authority());
        fake.events.borrow_mut().push("native");fake.role.set(Guest);
    });
    assert_eq!(*fake.events.borrow(),["role","dup-begin","native","role","dup-finish","diagnostic"]);
    pause::set_host_authority(old);
}

#[test]
fn disabled_session_rules_keep_native_update_and_observation_diagnostics_only() {
    let _guard=TEST_LOCK.lock().unwrap();
    let old=pause::host_authority();
    let fake=UpdateFake{enabled:false,role:Cell::new(asx_game::official::Role::Host),events:RefCell::new(vec![])};
    update_pass(Some(&fake),10,||fake.events.borrow_mut().push("native"));
    assert_eq!(*fake.events.borrow(),["native","diagnostic"]);
    assert_eq!(pause::host_authority(),old);
}

struct HudFake {
    enabled: bool, captures: Cell<usize>, cache_updates: Cell<usize>,
    written: RefCell<Vec<serde_json::Value>>, views: RefCell<Vec<asx_gameplay::official::PausePresentation>>,
}
impl HudServices for HudFake {
    fn rules_enabled(&self)->bool{self.enabled}
    fn solo_active(&self)->bool{true}
    fn hud_snapshot(&self,game:u64,output:u64)->Result<Option<(u32,serde_json::Value)>,String>{
        assert_eq!((game,output),(10,20));Ok(Some((1,serde_json::json!({"phase":"playing","native":"retained"}))))
    }
    fn capture_party(&self,_:u32,_:&serde_json::Value){self.captures.set(self.captures.get()+1);}
    fn effect_owner(&self,_:u64)->Option<(u32,u64)>{Some((1,200))}
    fn hud_phase(&self,_:&serde_json::Value)->Result<asx_gameplay::Phase,String>{Ok(asx_gameplay::Phase::Playing)}
    fn update_pause_party(&self,game:u64,owner:Option<(u32,u64)>,phase:Option<asx_gameplay::Phase>)
        ->(Option<asx_gameplay::official::PauseReason>,Vec<(u32,u64)>){
        assert_eq!((game,owner,phase),(10,Some((1,200)),Some(asx_gameplay::Phase::Playing)));
        self.cache_updates.set(self.cache_updates.get()+1);
        (Some(asx_gameplay::official::PauseReason::Menus{players:vec![1]}),vec![(1,100),(2,200)])
    }
    fn decorate_pause(&self,value:&mut serde_json::Value,view:&asx_gameplay::official::PausePresentation){
        self.views.borrow_mut().push(view.clone());value["pause_active"]=view.active.into();
    }
    fn replace_hud(&self,output:u64,value:&serde_json::Value)->Result<(),String>{
        assert_eq!(output,20);self.written.borrow_mut().push(value.clone());Ok(())
    }
    fn first_hud_phase(&self,_:u32,_:&serde_json::Value)->bool{false}
    fn info(&self,_:&str){}
    fn warn(&self,_:&str){panic!("unexpected HUD warning")}
}
fn hud_fake(enabled:bool)->HudFake{HudFake{enabled,captures:Cell::new(0),cache_updates:Cell::new(0),written:RefCell::new(vec![]),views:RefCell::new(vec![])}}
#[test]
fn host_hud_keeps_native_fields_and_commits_owned_pause_decoration() {
    let _guard=TEST_LOCK.lock().unwrap();super::super::menu_pause::reset();
    let old=pause::host_authority();pause::set_host_authority(true);
    let fake=hud_fake(true);enrich_hud(&fake,20,10);
    assert_eq!(fake.captures.get(),2,"solo and host capture retain separate gates");assert_eq!(fake.cache_updates.get(),1);
    assert_eq!(fake.written.borrow()[0]["native"],"retained");assert_eq!(fake.written.borrow()[0]["pause_active"],true);
    assert!(fake.views.borrow()[0].active);
    pause::set_host_authority(old);super::super::menu_pause::reset();
}
#[test]
fn observation_only_hud_does_not_capture_or_replace_gameplay_presentation() {
    let _guard=TEST_LOCK.lock().unwrap();
    let old=pause::host_authority();pause::set_host_authority(true);
    let fake=hud_fake(false);enrich_hud(&fake,20,10);
    assert_eq!(fake.captures.get(),0);assert_eq!(fake.cache_updates.get(),0);assert!(fake.written.borrow().is_empty());
    pause::set_host_authority(old);
}
