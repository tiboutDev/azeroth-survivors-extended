//! Native pause notices and UI event authority. Owns pause flags and trampolines.
use super::{duplicate_loot, menu_pause, party_ui, Adapter};
use services::Services;
mod services;
#[cfg(feature = "test-hooks")]
use super::{fixture, performance};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static PAUSED: AtomicBool = AtomicBool::new(false);
static UI_INPUT_PAUSED: AtomicBool = AtomicBool::new(false);
// HUD reception drives guests; host simulation owns its barrier independently.
static HOST_PAUSE_AUTHORITY: AtomicBool = AtomicBool::new(false);
static UI_HUD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static UI_EVENT_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]static UI_EVENT_TEST_LOCK:std::sync::Mutex<()>=std::sync::Mutex::new(());
crate::context_stub!(ui_hud_entry, 0, ui_hud, UI_HUD_ORIGINAL);
crate::context_stub_rw!(ui_event_entry, 0, ui_event, UI_EVENT_ORIGINAL);
fn publish_pause(view: asx_gameplay::official::PausePresentation) {
    if !HOST_PAUSE_AUTHORITY.load(Ordering::Acquire) {
        PAUSED.store(view.active, Ordering::Release);
    }
    crate::presentation::pause::publish(&crate::presentation::pause::PauseView {
        active: view.active,
        visible: view.visible,
        local_menu: view.local_menu,
        title: view.title,
        detail: view.detail,
        timeout: false,
        epoch: 0,
    });
}
fn native_pause_transition(owned:&AtomicBool,active:bool,phase:asx_gameplay::Phase)->Option<bool>{
 if active {return (!owned.swap(true,Ordering::AcqRel)).then_some(true)}
 if phase!=asx_gameplay::Phase::Choosing&&owned.swap(false,Ordering::AcqRel){Some(false)}else{None}
}
unsafe fn native_input_pause(receiver:u64,active:bool){
 let Some(runtime)=services::available()else{return};
 let Some(event)=runtime.pause_notice(active)else{return};
 native_pause_emit(receiver,event.as_bytes());
}
unsafe fn native_pause_emit(receiver:u64,event:&[u8]){
 type Event=unsafe extern "system" fn(u64,*const u8,usize);
 let original=UI_EVENT_ORIGINAL.load(Ordering::Acquire);
 if original!=0{let call:Event=std::mem::transmute(original);call(receiver,event.as_ptr(),event.len());}
}
unsafe extern "system" fn ui_hud(c: &crate::stub::Context) {
    #[cfg(feature = "test-hooks")]
    let _timing = performance::start(3);
    let Some(runtime) = services::active() else { return };
    handle_hud(runtime, c);
}

fn handle_hud(runtime: &dyn Services, c: &crate::stub::Context) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Ok(value) = runtime.read_ui(c.rdx, c.r8) {
            party_ui::roster(&value);
            menu_pause::session(&value);
            #[cfg(feature = "test-hooks")]
            runtime.economy_hud(&value);
            match runtime.decode_pause(&value) {
                Ok(Some(view)) => {
                    #[cfg(feature = "test-hooks")]
                    if fixture::pause_enabled() { menu_pause::fixture_view(&view); }
                    let transition = runtime.pause_notice(true)
                        .and_then(|_| runtime.hud_phase(&value).ok())
                        .and_then(|phase| native_pause_transition(&UI_INPUT_PAUSED, view.active, phase));
                    publish_pause(view);
                    if let Some(active) = transition {
                        // The selected notice still comes through the runtime-present
                        // service gate, including observation-only runtimes.
                        unsafe { native_input_pause(c.rcx, active); }
                    }
                }
                // A native/partial HUD carries no group-resume authority.
                Ok(None) => {}
                Err(error) => runtime.warn(&format!("Mod pause presentation rejected: {error}")),
            }
        }
    }));
}

unsafe extern "system" fn ui_event(c: &mut crate::stub::Context) {
    #[cfg(feature = "test-hooks")]
    let _timing = performance::start(4);
    let Some(runtime) = services::active() else { return };
    handle_event(runtime, c);
}

fn handle_event(runtime: &dyn Services, c: &mut crate::stub::Context) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Ok(value) = runtime.read_ui(c.rdx, c.r8) {
            if value["type"] == "unpaused" && PAUSED.load(Ordering::Acquire) && UI_INPUT_PAUSED.load(Ordering::Acquire) {
                c.skip = 1;
                c.result = 0;
                return;
            }
            if runtime.duplicate_loot_enabled() { duplicate_loot::presentation(&value); }
            #[cfg(feature = "test-hooks")]
            runtime.fixture_event(c, &value);
            party_ui::roster(&value);
            let reset = pause_session_ended(&value);
            if reset {
                runtime.end_session();
                HOST_PAUSE_AUTHORITY.store(false, Ordering::Release);
                PAUSED.store(false, Ordering::Release);
                UI_INPUT_PAUSED.store(false, Ordering::Release);
                publish_pause(Default::default());
            }
        }
    }));
}

// A pick acknowledges only one player's choice. It cannot release a group
// barrier; only the authoritative complete HUD/snapshot or teardown can do so.
fn pause_session_ended(value:&serde_json::Value)->bool {
    matches!(value["type"].as_str(),Some("menu"|"logout")) || value["type"]=="coop" && matches!(value["state"].as_str(),Some("ended"|"closed"|"unavailable"))
}
#[cfg(test)]
mod pause_regression_tests {
 use super::*;
 #[test]fn complete_group_state_alone_resumes_guests_and_host_ui_cannot_release_the_barrier(){
  let _guard=crate::test_support::TEST_LOCK.lock().unwrap();
  HOST_PAUSE_AUTHORITY.store(false,Ordering::Release);
  let view=asx_gameplay::official::PausePresentation{active:true,visible:true,..Default::default()};
  publish_pause(view);assert!(PAUSED.load(Ordering::Acquire));
  for event in [serde_json::json!({"type":"picked"}),serde_json::json!({"type":"ready"}),serde_json::json!({"type":"choices","why":"cache"})]{assert!(!pause_session_ended(&event));assert!(PAUSED.load(Ordering::Acquire));}
  publish_pause(Default::default());assert!(!PAUSED.load(Ordering::Acquire));
  HOST_PAUSE_AUTHORITY.store(true,Ordering::Release);PAUSED.store(true,Ordering::Release);
  publish_pause(Default::default());assert!(PAUSED.load(Ordering::Acquire),"A local HUD must not release the host's group pause");
  assert!(pause_session_ended(&serde_json::json!({"type":"coop","state":"ended"})));
  HOST_PAUSE_AUTHORITY.store(false,Ordering::Release);publish_pause(Default::default());
 }
}

#[cfg(test)]mod native_pause_tests{
 use super::*;
 #[test]fn group_pause_owns_native_transition_until_everyone_has_finished(){use asx_gameplay::Phase::*;let owned=AtomicBool::new(false);assert_eq!(native_pause_transition(&owned,false,Playing),None);assert_eq!(native_pause_transition(&owned,true,Playing),Some(true));assert_eq!(native_pause_transition(&owned,true,Choosing),None);assert_eq!(native_pause_transition(&owned,false,Choosing),None);assert_eq!(native_pause_transition(&owned,false,Playing),Some(false));assert_eq!(native_pause_transition(&owned,false,Playing),None);for phase in [Downed,Inactive]{assert_eq!(native_pause_transition(&owned,true,phase),Some(true));assert_eq!(native_pause_transition(&owned,false,phase),Some(false));}}
 unsafe extern "system" fn event(receiver:u64,ptr:*const u8,len:usize){assert_eq!(receiver,0x12340);assert_eq!(std::slice::from_raw_parts(ptr,len),b"{\"type\":\"unpaused\"}");}
 #[test]fn native_notification_preserves_recipient_and_borrowed_event(){let _guard=UI_EVENT_TEST_LOCK.lock().unwrap();let old=UI_EVENT_ORIGINAL.swap(event as *const ()as usize,Ordering::AcqRel);unsafe{native_pause_emit(0x12340,b"{\"type\":\"unpaused\"}")};UI_EVENT_ORIGINAL.store(old,Ordering::Release);}
}

#[cfg(test)]mod pause_event_abi_test{use super::*;unsafe extern "system" fn passthrough(a:u64,b:u64,c:u64)->u64{assert_eq!([a,b,c],[11,22,33]);77}
#[test]fn mutable_event_stub_preserves_native_arguments(){let _guard=UI_EVENT_TEST_LOCK.lock().unwrap();let old=UI_EVENT_ORIGINAL.swap(passthrough as *const ()as usize,Ordering::AcqRel);let call:unsafe extern "system" fn(u64,u64,u64)->u64=unsafe{std::mem::transmute(ui_event_entry as *const ())};assert_eq!(unsafe{call(11,22,33)},77);UI_EVENT_ORIGINAL.store(old,Ordering::Release);}}

// Only the pause owner mutates authority flags. The returned prior state keeps
// transition logging at the existing coordinator sites.
pub(super) fn active() -> bool { PAUSED.load(Ordering::Acquire) }
pub(super) fn host_authority() -> bool { HOST_PAUSE_AUTHORITY.load(Ordering::Acquire) }
pub(super) fn set_active(active: bool) -> bool { PAUSED.swap(active, Ordering::AcqRel) }
pub(super) fn set_host_authority(host: bool) { HOST_PAUSE_AUTHORITY.store(host, Ordering::Release); }

#[cfg(feature = "test-hooks")]
pub(super) unsafe fn replay_event(receiver: u64, event: &[u8]) {
    type Event = unsafe extern "system" fn(u64, *const u8, usize);
    let original: Event = std::mem::transmute(UI_EVENT_ORIGINAL.load(Ordering::Acquire));
    original(receiver, event.as_ptr(), event.len());
}

pub(super) fn hooks(adapter: Adapter, base: usize) -> [crate::install::HookSpec; 2] {
    [
        crate::install::HookSpec { name: "official-ui-hud", target: base + adapter.ui_hud().rva as usize,
            detour: ui_hud_entry as *const () as usize, trampoline: &UI_HUD_ORIGINAL },
        crate::install::HookSpec { name: "official-ui-event", target: base + adapter.ui_event().rva as usize,
            detour: ui_event_entry as *const () as usize, trampoline: &UI_EVENT_ORIGINAL },
    ]
}

#[cfg(test)]
mod tests;
