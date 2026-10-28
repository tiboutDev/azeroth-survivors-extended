//! Native session callbacks consume bounded command, world, HUD and update ports.
mod services;
use services::{CommandServices, HudServices, WorldControl, WorldServices};
use super::{pause, Adapter};
#[cfg(feature = "test-hooks")]
use super::performance;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[cfg(test)]
mod tests;

type Update = unsafe extern "system" fn(*mut u8, f64);
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicU64 = AtomicU64::new(0);
static HUD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static WORLD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static COMMAND_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(command_entry, 0, command, COMMAND_ORIGINAL);
pub(super) unsafe extern "system" fn command(context: &mut crate::stub::Context) {
    let Some(runtime) = services::command() else { return };
    dispatch_command(runtime, context);
}

fn dispatch_command(runtime: &dyn CommandServices, context: &mut crate::stub::Context) -> bool {
    if runtime.rules_enabled() && runtime.menu_command(context) { return true; }
    if runtime.rules_enabled() && runtime.duplicate_enabled() && runtime.duplicate_command(context) { return true; }
    if !runtime.rules_enabled() || !pause::active() || !runtime.movement_opcode(context.r9 as u16) { return false; }
    if runtime.active_role(context.rcx) { context.skip = 1; context.result = 0; return true; }
    false
}
type World = unsafe extern "system" fn(*mut u8, f32);
unsafe extern "system" fn world(game: *mut u8, dt: f32) {
    let original: World = std::mem::transmute(WORLD_ORIGINAL.load(Ordering::Acquire));
    let now = pre_world(services::world_control(), game as u64, dt);
    original(game, now);
    finish_world(services::world_control(), game as u64);
}

fn pre_world(runtime: Option<&dyn WorldControl>, game: u64, dt: f32) -> f32 {
    let mut delta = if pause::active() { 0.0 } else { dt };
    if let Some(runtime) = runtime.filter(|r| r.rules_enabled() && pause::host_authority()) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.observe_world(game)));
        if let Ok(Ok(reason)) = result {
            let paused = reason.is_some();
            delta = if paused { 0.0 } else { dt };
            if pause::set_active(paused) != paused {
                runtime.log_group_pause(&reason);
            }
        } else if pause::set_active(false) {
            runtime.warn("Official pause observation failed; native simulation resumed and mod barrier deactivated");
        }
    }
    if let Some(runtime) = runtime.filter(|r| r.rules_enabled()) { runtime.capture_world(game); }
    delta
}

fn finish_world(runtime: Option<&dyn WorldControl>, game: u64) {
    if let Some(runtime) = runtime.filter(|r| r.rules_enabled()) { runtime.finish_world(game); }
}
type Hud = unsafe extern "system" fn(*mut u8, *mut u8) -> *mut u8;
unsafe extern "system" fn hud(output: *mut u8, game: *mut u8) -> *mut u8 {
    let original: Hud = std::mem::transmute(HUD_ORIGINAL.load(Ordering::Acquire));
    let result = original(output, game);
    #[cfg(feature = "test-hooks")]
    let _timing = performance::start(5);
    if let Some(runtime) = services::hud() {
        enrich_hud(runtime, output as u64, game as u64);
    }
    result
}

fn enrich_hud(runtime: &dyn HudServices, output: u64, game: u64) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Ok(Some((actor, value))) = runtime.hud_snapshot(game as u64, output as u64) else {
                return;
            };
            if runtime.rules_enabled() && runtime.solo_active() {
                runtime.capture_party(actor, &value);
            }
            if runtime.rules_enabled() && pause::host_authority() {
                runtime.capture_party(actor, &value);
                let owner = runtime.effect_owner(game as u64);
                let current_phase = runtime.hud_phase(&value).ok();
                let (reason, identities) = runtime.update_pause_party(game as u64, owner, current_phase);
                if let Ok(phase) = runtime.hud_phase(&value) {
                    let mut view = super::party_ui::pause_view(reason, phase, &identities);
                    if owner.is_some_and(|(slot, guid)| super::menu_pause::local_menu(slot, guid)) {
                        view.local_menu = true;
                        view.visible = false;
                    }
                    let mut decorated = value.clone();
                    runtime.decorate_pause(&mut decorated, &view);
                    let epoch = super::menu_pause::epoch();
                    if epoch != 0 { decorated["_azcoop"]["pause"]["session"] = epoch.into(); }
                    if let Err(error) = runtime.replace_hud(output as u64, &decorated) {
                        runtime.warn(&format!("Mod HUD update failed: {error}"));
                    }
                }
            }
            if runtime.first_hud_phase(actor, &value) {
                runtime.info(&format!("official HUD: actor={actor} data={value}"));
            }
        }));
}
unsafe extern "system" fn update(server: *mut u8, now: f64) {
    let original: Update = std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
    update_pass(services::world(), server as u64, || original(server, now));
}

fn update_pass(runtime: Option<&dyn WorldServices>, server_address: u64, native: impl FnOnce()) {
    #[cfg(feature = "test-hooks")]
    if let Some(runtime) = runtime.filter(|runtime| runtime.rules_enabled()) {
        runtime.fixture_before_update(server_address);
    }
    if let Some(runtime) = runtime.filter(|runtime| runtime.rules_enabled()) {
        if let Ok(state) = runtime.observe_role(server_address) {
            pause::set_host_authority(state.role == asx_game::official::Role::Host);
            super::menu_pause::observe_role(state.role);
        }
    }
    if let Some(runtime) = runtime.filter(|runtime| runtime.rules_enabled() && runtime.duplicate_enabled()) {
        runtime.begin_duplicates(server_address);
    }
    native();
    if let Some(runtime) = runtime.filter(|runtime| runtime.rules_enabled()) {
        if runtime.observe_role(server_address).is_ok_and(|state| state.role == asx_game::official::Role::Host) {
            runtime.refresh_host(server_address);
            runtime.finish_experience(server_address);
        }
    }
    if let Some(runtime) = runtime.filter(|runtime| runtime.rules_enabled() && runtime.duplicate_enabled()) {
        runtime.finish_duplicates(server_address);
    }
    CALLS.fetch_add(1, Ordering::Relaxed);
    #[cfg(feature = "test-hooks")]
    if let Some(runtime) = runtime {
        runtime.fixture_after_update(server_address);
    }
    if let Some(runtime) = runtime {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime.emit_diagnostics(server_address);
        }));
    }
}

/// Included after XP and UI entries in bootstrap's existing installation batch.
pub(super) fn hooks(adapter: Adapter, base: usize) -> [crate::install::HookSpec; 4] {
    [
        crate::install::HookSpec { name: "official-server-observe", target: base + adapter.server_update().rva as usize,
            detour: update as *const () as usize, trampoline: &ORIGINAL },
        crate::install::HookSpec { name: "official-hud-observe", target: base + adapter.hud().rva as usize,
            detour: hud as *const () as usize, trampoline: &HUD_ORIGINAL },
        crate::install::HookSpec { name: "official-world", target: base + adapter.world_update().rva as usize,
            detour: world as *const () as usize, trampoline: &WORLD_ORIGINAL },
        crate::install::HookSpec { name: "official-command", target: base + adapter.command().rva as usize,
            detour: command_entry as *const () as usize, trampoline: &COMMAND_ORIGINAL },
    ]
}
