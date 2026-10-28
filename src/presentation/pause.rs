//! Retained official Azeroth Survivors pause UI. Only presentation inputs change.
//! Simulation and reward menus remain owned by the native game.
#[derive(Clone, Default, PartialEq, Eq, serde::Serialize, Debug)]
pub(crate) struct PauseView {
    pub active: bool,
    pub visible: bool,
    pub local_menu: bool,
    pub title: String,
    pub detail: String,
    pub timeout: bool,
    pub epoch: u64,
}
use std::{
    cell::RefCell,
    ffi::c_void,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};
use asx_game::official::PauseLayout;
static LAYOUT: Mutex<Option<PauseLayout>>=Mutex::new(None);
fn rva(old:usize)->usize{LAYOUT.lock().unwrap_or_else(|e|e.into_inner()).expect("Pause layout not installed").rva(old)}
static BASE: AtomicUsize = AtomicUsize::new(0);
static VERSION: AtomicUsize = AtomicUsize::new(0);
static DISPLAY: AtomicUsize = AtomicUsize::new(0);
static REDRAW: AtomicUsize = AtomicUsize::new(0);
static CLEARING: AtomicUsize = AtomicUsize::new(0);
static VIEW: Mutex<Option<PauseView>> = Mutex::new(None);
static CHECK: AtomicUsize = AtomicUsize::new(0);
static CHECK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static BOOL: AtomicUsize = AtomicUsize::new(0);
static BUILD: AtomicUsize = AtomicUsize::new(0);
static TITLE: AtomicUsize = AtomicUsize::new(0);
static DETAIL: AtomicUsize = AtomicUsize::new(0);
static PANEL: AtomicUsize = AtomicUsize::new(0);
static FONT: AtomicUsize = AtomicUsize::new(0);
static CLICK: AtomicUsize = AtomicUsize::new(0);
static CLICK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static BUTTON: AtomicUsize = AtomicUsize::new(0);
static RECOVERY_ROOT: Mutex<Option<u64>> = Mutex::new(None);
static PENDING_ACTION: Mutex<Option<String>> = Mutex::new(None);
static BUILDS: AtomicUsize = AtomicUsize::new(0);
thread_local! {static BUILD_VIEW:RefCell<PauseView>=RefCell::new(PauseView::default());}
crate::context_stub_rw!(check_entry, 0, check, CHECK);
crate::context_stub_rw!(bool_entry, 0, visible, BOOL);
crate::context_stub_rw!(build_entry, 0, build, BUILD);
crate::context_stub_rw!(title_entry, 0, text, TITLE);
crate::context_stub_rw!(detail_entry, 1, text, DETAIL);
crate::context_stub_rw!(panel_entry, 0, panel, PANEL);
crate::context_stub_rw!(font_entry, 0, font, FONT);
crate::context_stub_rw!(click_entry, 0, click, CLICK);
crate::context_stub_rw!(button_entry, 0, button, BUTTON);
pub fn publish(view: &PauseView) {
    let mut slot = VIEW.lock().unwrap_or_else(|e| e.into_inner());
    if slot.as_ref() != Some(view) {
        let redraw = view.active || slot.as_ref().is_some_and(|v| v.active);
        *slot = Some(view.clone());
        VERSION.fetch_add(1, Ordering::Release);
        DISPLAY.store(
            usize::from(view.active)
                | (usize::from(view.visible || view.local_menu) << 1)
                | (usize::from(view.local_menu) << 2),
            Ordering::Release,
        );
        if redraw {
            REDRAW.store(2, Ordering::Release);
        }
    }
}
fn snapshot() -> PauseView {
    VIEW.lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default()
}
unsafe extern "system" fn check(_c: &mut crate::stub::Context) {
    let redraw = REDRAW
        .try_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_sub(1))
        .unwrap_or(0);
    CLEARING.store(usize::from(redraw == 2), Ordering::Release);
    // Despawn the previous retained native root, then build once with the new
    // reason/roster. Native deferred ECS destruction completes between these calls.
    CHECK.store(
        if redraw > 0 || DISPLAY.load(Ordering::Acquire) & 5 == 1 {
            BASE.load(Ordering::Relaxed) + rva(0x1bfc1)
        } else {
            CHECK_ORIGINAL.load(Ordering::Relaxed)
        },
        Ordering::Release,
    );
}
unsafe extern "system" fn visible(c: &mut crate::stub::Context) {
    let display = DISPLAY.load(Ordering::Acquire);
    let clearing = CLEARING.load(Ordering::Acquire) != 0;
    if clearing || display & 5 == 1 {
        c.rdx = (c.rdx & !255) | u64::from(!clearing && display & 2 != 0);
        // These are the native builder's borrowed pause-vs-menu presentation flag
        // and matching Resume label selector, not the simulation's phase/resource.
        *((c.rbp + 0x2fe) as *mut u8) = 1;
        c.rsi = (c.rsi & !255) | 1;
    }
}
unsafe extern "system" fn build(c: &mut crate::stub::Context) {
    let view = snapshot();
    *RECOVERY_ROOT.lock().unwrap_or_else(|e| e.into_inner()) = if view.visible && view.timeout {
        Some(view.epoch)
    } else {
        None
    };
    if view.visible {
        let input = c.rdx as *mut usize;
        let cap = *input;
        let pointer = *input.add(1);
        let len = *input.add(2);
        if len <= cap && cap <= 16 {
            if view.timeout && len >= 2 {
                for (i, label) in ["Keep waiting", "Continue without missing players"]
                    .iter()
                    .enumerate()
                {
                    let row = (pointer + i * 24) as *mut u8;
                    *row = if i == 0 { 0 } else { 2 };
                    std::ptr::write_unaligned(row.add(8) as *mut usize, label.as_ptr() as usize);
                    std::ptr::write_unaligned(row.add(16) as *mut usize, label.len());
                }
                *input.add(2) = 2;
            } else {
                *input.add(2) = 0;
            }
        }
        BUILDS.fetch_add(1, Ordering::Relaxed);
    }
    BUILD_VIEW.with(|v| *v.borrow_mut() = view);
}
unsafe extern "system" fn text(c: &mut crate::stub::Context) {
    BUILD_VIEW.with(|v| {
        let v = v.borrow();
        if v.visible {
            let text = if c.id == 0 { &v.title } else { &v.detail };
            c.rax = text.as_ptr() as u64;
            c.rdx = text.len() as u64;
        }
    });
}
unsafe extern "system" fn panel(c: &mut crate::stub::Context) {
    if c.return_address() as usize != BASE.load(Ordering::Relaxed) + rva(0x346ac3) {
        return;
    }
    BUILD_VIEW.with(|v| {
        if v.borrow().visible {
            let node = (c.r8 as *mut u8).add(8);
            for (offset, tag, value) in [(0x88, 1u32, 600f32), (0xa8, 2, 90.), (0x90, 0, 0.)] {
                std::ptr::write_unaligned(node.add(offset) as *mut u32, tag);
                std::ptr::write_unaligned(node.add(offset + 4) as *mut f32, value);
            }
        }
    });
}
unsafe extern "system" fn font(c: &mut crate::stub::Context) {
    BUILD_VIEW.with(|v| {
        if v.borrow().visible {
            std::ptr::write_unaligned((c.rbp + 0x470) as *mut f32, 28.);
        }
    });
}
pub fn take_action() -> Option<String> {
    PENDING_ACTION
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
}
unsafe extern "system" fn click(c: &mut crate::stub::Context) {
    let generation = *RECOVERY_ROOT.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(epoch) = generation {
        let action = *(c.rdx as *const u8);
        if let Some(action) = match action {
            0 => Some("wait"),
            2 => Some("continue"),
            _ => None,
        } {
            let mut pending = PENDING_ACTION.lock().unwrap_or_else(|e| e.into_inner());
            if pending.is_none() {
                *pending = Some(format!("{epoch}|{action}"));
            }
        }
        // Intercept before the native confirmation/destructive menu paths. Even a
        // stale button is consumed here; the game-thread epoch check rejects it.
        CLICK.store(BASE.load(Ordering::Relaxed) + rva(0x1b6d4), Ordering::Release);
    } else {
        CLICK.store(CLICK_ORIGINAL.load(Ordering::Relaxed), Ordering::Release);
    }
}
unsafe extern "system" fn button(c: &mut crate::stub::Context) {
    if c.return_address() as usize != BASE.load(Ordering::Relaxed) + rva(0x347403) {
        return;
    }
    BUILD_VIEW.with(|v| {
        if v.borrow().visible {
            let node = (c.r8 as *mut u8).add(8);
            std::ptr::write_unaligned(node.add(0x88) as *mut u32, 2);
            std::ptr::write_unaligned(node.add(0x8c) as *mut f32, 100.);
        }
    });
}
pub fn evidence() -> serde_json::Value {
    serde_json::json!({"installed":BASE.load(Ordering::Acquire)!=0,"version":VERSION.load(Ordering::Relaxed),"native_builds":BUILDS.load(Ordering::Relaxed),"view":snapshot()})
}
pub fn install_selected(base:usize,layout:PauseLayout)->Result<(),String>{
    if BASE.load(Ordering::Acquire) != 0 {
        return Ok(());
    }
    *LAYOUT.lock().unwrap_or_else(|e|e.into_inner())=Some(layout);
    let sites: [(usize, &[u8], *mut c_void, &AtomicUsize); 9] = [
        (
            0x1bf9c,
            b"\x41\x80\xbc\x24\xea\x00\x00\x00\x01",
            check_entry as *const () as _,
            &CHECK,
        ),
        (
            0x1bfcd,
            b"\x48\x8b\x4d\x48\x48\x8b\x5d\x50",
            bool_entry as *const () as _,
            &BOOL,
        ),
        (
            0x346580,
            b"\x55\x41\x57\x41\x56\x41\x55\x41\x54\x56\x57\x53",
            build_entry as *const () as _,
            &BUILD,
        ),
        (
            0x346b4c,
            b"\x48\x89\xd3\x48\x85\xd2",
            title_entry as *const () as _,
            &TITLE,
        ),
        (
            0x346d29,
            b"\x48\x89\xd3\x48\x85\xd2",
            detail_entry as *const () as _,
            &DETAIL,
        ),
        (
            0x346cc9,
            b"\x4c\x89\xad\x20\x04\x00\x00",
            font_entry as *const () as _,
            &FONT,
        ),
        (
            0x1bbce,
            b"\x48\x85\xc9\x74\x2f",
            click_entry as *const () as _,
            &CLICK,
        ),
        (
            0x3227b0,
            b"\x55\x41\x57\x41\x56\x41\x55\x41\x54\x56\x57\x53",
            button_entry as *const () as _,
            &BUTTON,
        ),
        (
            0x31ddd0,
            b"\x55\x41\x57\x41\x56\x41\x55\x41\x54\x56\x57\x53",
            panel_entry as *const () as _,
            &PANEL,
        ),
    ];
    for (rva, bytes, _, _) in &sites {
        if unsafe { std::slice::from_raw_parts((base + self::rva(*rva)) as *const u8, bytes.len()) } != *bytes {
            return Err(format!("Native pause UI contract changed: {rva:x}"));
        }
    }
    BASE.store(base, Ordering::Release);
    for (rva, _, detour, slot) in sites {
        let target = (base + self::rva(rva)) as *mut c_void;
        let original = unsafe { minhook::MinHook::create_hook(target, detour) }
            .map_err(|e| format!("Native pause hook: {e:?}"))?;
        slot.store(original as usize, Ordering::Release);
        if rva == 0x1bbce {
            CLICK_ORIGINAL.store(original as usize, Ordering::Release);
        }
        if rva == 0x1bf9c {
            CHECK_ORIGINAL.store(original as usize, Ordering::Release);
        }
        unsafe { minhook::MinHook::queue_enable_hook(target) }
            .map_err(|e| format!("Native pause enable: {e:?}"))?;
    }
    unsafe { minhook::MinHook::apply_queued() }
        .map_err(|e| format!("Native pause apply: {e:?}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timeout_buttons_never_reach_native_destructive_actions_and_keep_their_generation() {
        let _turn = crate::test_support::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *LAYOUT.lock().unwrap()=Some(asx_game::official::SelectedAdapter::select(asx_game::official125::SHA256).unwrap().pause_layout());
        BASE.store(0x10000000, Ordering::Release);
        CLICK_ORIGINAL.store(0x1234, Ordering::Release);
        *RECOVERY_ROOT.lock().unwrap() = Some(7);
        *PENDING_ACTION.lock().unwrap() = None;
        for (tag, want) in [(0u8, "wait"), (2, "continue")] {
            let mut c: crate::stub::Context = unsafe { std::mem::zeroed() };
            c.rdx = (&tag as *const u8) as u64;
            unsafe { click(&mut c) };
            assert_eq!(take_action(), Some(format!("7|{want}")));
            assert_eq!(CLICK.load(Ordering::Acquire), 0x10000000 + asx_game::official::SelectedAdapter::select(asx_game::official125::SHA256).unwrap().pause_addresses()[10]);
        }
        // The rendered root retains its original generation even after a newer
        // recovery starts; the game-thread authority check will reject this action.
        publish(&PauseView {
            active: true,
            epoch: 8,
            ..Default::default()
        });
        let tag = 2u8;
        let mut c: crate::stub::Context = unsafe { std::mem::zeroed() };
        c.rdx = (&tag as *const u8) as u64;
        unsafe { click(&mut c) };
        assert_eq!(take_action().as_deref(), Some("7|continue"));
        *RECOVERY_ROOT.lock().unwrap() = None;
        unsafe { click(&mut c) };
        assert_eq!(CLICK.load(Ordering::Acquire), 0x1234);
        assert!(take_action().is_none());
        BASE.store(0, Ordering::Release);
        publish(&PauseView::default());
    }
    #[test]
    fn modern_timeout_action_uses_retained_root_epoch_and_bypasses_destructive_dispatch() {
        let _turn=crate::test_support::TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());
        *LAYOUT.lock().unwrap()=Some(asx_game::official::SelectedAdapter::select(asx_game::official125::SHA256).unwrap().pause_layout());BASE.store(0x10000000,Ordering::Release);
        *RECOVERY_ROOT.lock().unwrap()=Some(42);*PENDING_ACTION.lock().unwrap()=None;
        let tag=2u8;let mut c:crate::stub::Context=unsafe{std::mem::zeroed()};c.rdx=(&tag as *const u8)as u64;
        unsafe{click(&mut c)};
        assert_eq!(take_action().as_deref(),Some("42|continue"));
        assert_eq!(CLICK.load(Ordering::Acquire),0x10000000 + asx_game::official::SelectedAdapter::select(asx_game::official125::SHA256).unwrap().pause_addresses()[10]);
        *RECOVERY_ROOT.lock().unwrap()=None;BASE.store(0,Ordering::Release);*LAYOUT.lock().unwrap()=None;
    }
    #[test]
    fn unchanged_presentation_does_not_request_a_new_native_tree() {
        let _turn = crate::test_support::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let v = PauseView {
            active: true,
            visible: true,
            title: "Game paused".into(),
            ..Default::default()
        };
        publish(&v);
        REDRAW.store(0, Ordering::Release);
        let version = VERSION.load(Ordering::Acquire);
        for _ in 0..1000 {
            publish(&v);
        }
        assert_eq!(VERSION.load(Ordering::Acquire), version);
        assert_eq!(REDRAW.load(Ordering::Acquire), 0);
        publish(&PauseView::default());
    }
}
