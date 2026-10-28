//! Native menu ownership; existing native coop stream and retained menu only.
use super::{local_base, pause, Adapter};
#[cfg(feature = "test-hooks")]
use super::Runtime;
use services::Services;
mod services;
use std::sync::{atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}, Mutex};
static ACTIVE: AtomicBool = AtomicBool::new(false);
static KEY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static KEY_NEXT: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(key_entry, 0, key, KEY_NEXT);
unsafe extern "system" fn key(_c: &mut crate::stub::Context) {
    let target = services::active()
        .filter(|_| ACTIVE.load(Ordering::Acquire))
        .and_then(|services| services.key_target())
        .unwrap_or_else(|| KEY_ORIGINAL.load(Ordering::Acquire));
    KEY_NEXT.store(target, Ordering::Release);
}
pub fn observe_role(role: asx_game::official::Role) {
    ACTIVE.store(
        matches!(
            role,
            asx_game::official::Role::Host | asx_game::official::Role::Guest
        ),
        Ordering::Release,
    );
}
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static REMOTE_EPOCH: AtomicU64 = AtomicU64::new(0);
static LOCAL_OPEN: AtomicBool = AtomicBool::new(false);
#[derive(Default)]
struct Menus {
    key: (u64, u64),
    clock: f32,
    epoch: u64,
    owners: asx_gameplay::official::MenuOwners,
}
static STATE: std::sync::LazyLock<Mutex<Menus>> =
    std::sync::LazyLock::new(|| Mutex::new(Menus::default()));
impl Menus {
    fn reconcile(&mut self, game: u64, p: &asx_game::official::PartyObservation) {
        let key = (
            game,
            p.actors.iter().find(|a| a.slot == 0).map_or(0, |a| a.guid),
        );
        if self.key != key || p.clock + 0.001 < self.clock {
            *self = Self {
                key,
                epoch: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(1, |d| d.as_nanos() as u64)
                    .max(1),
                ..Default::default()
            };
        }
        self.clock = p.clock;
        self.owners
            .retain(p.actors.iter().map(|a| (a.slot, a.guid, a.phase)));
    }
    fn set(&mut self, slot: u32, guid: u64, open: bool) {
        self.owners.set(slot, guid, open);
    }
    fn local(&self, slot: u32, guid: u64) -> bool {
        self.owners.contains(slot, guid)
    }
}
pub fn reset() {
    ACTIVE.store(false, Ordering::Release);
    if let Ok(mut s) = STATE.lock() {
        *s = Menus::default();
    }
    REMOTE_EPOCH.store(0, Ordering::Release);
    LOCAL_OPEN.store(false, Ordering::Release);
}
pub fn reason(
    r: &dyn Services,
    game: u64,
    p: &asx_game::official::PartyObservation,
) -> Option<asx_gameplay::official::PauseReason> {
    if !r.has_menu() {
        return asx_gameplay::official::group_pause(
            p.actors.iter().map(|a| (a.slot + 1, a.phase)),
        );
    }
    let Ok(mut s) = STATE.lock() else { return None };
    s.reconcile(game, p);
    asx_gameplay::official::group_pause(p.actors.iter().map(|a| {
        (
            a.slot + 1,
            if s.local(a.slot, a.guid) && a.phase == asx_gameplay::Phase::Playing {
                asx_gameplay::Phase::Paused
            } else {
                a.phase
            },
        )
    }))
}
pub fn local_menu(slot: u32, guid: u64) -> bool {
    STATE.lock().is_ok_and(|s| s.local(slot, guid))
}
pub fn epoch() -> u64 {
    STATE.lock().map_or(0, |s| s.epoch)
}
pub fn session(v: &serde_json::Value) {
    if let Some(epoch) = v["_azcoop"]["pause"]["session"].as_u64() {
        if REMOTE_EPOCH.swap(epoch, Ordering::AcqRel) != epoch {
            LOCAL_OPEN.store(false, Ordering::Release);
        }
    }
}
fn request(r: &dyn Services, server: u64, open: bool) -> Result<(), String> {
    let role = r.role(server)?;
    if !matches!(
        role,
        asx_game::official::Role::Host | asx_game::official::Role::Guest
    ) {
        return Ok(());
    }
    // Escape on a waiting panel opens this participant own native menu. Closing
    // an owned menu removes only that ownership; the group barrier stays separate.
    let open = open || (!LOCAL_OPEN.load(Ordering::Acquire) && pause::active());
    match role {
        asx_game::official::Role::Host => {
            let p = r.cached_party()
                .ok_or("Missing menu run")?;
            let a = p
                .actors
                .iter()
                .find(|a| a.slot == 0 && a.phase != asx_gameplay::Phase::Inactive)
                .ok_or("No running menu owner")?;
            let mut s = STATE.lock().map_err(|_| "Menu state unavailable")?;
            s.reconcile(r.game_address(server)?, &p);
            s.set(a.slot, a.guid, open);
        }
        asx_game::official::Role::Guest => {
            let epoch = REMOTE_EPOCH.load(Ordering::Acquire);
            if epoch == 0 {
                return Ok(());
            }
            let mut bytes = epoch.to_le_bytes().to_vec();
            bytes.push(u8::from(open));
            r.send(server, &bytes)?;
        }
        _ => {}
    }
    LOCAL_OPEN.store(open, Ordering::Release);
    r.info(&format!("official menu request: open={open} role={role:?}"));
    Ok(())
}
type Input = unsafe extern "system" fn(u64, u64);
unsafe extern "system" fn input(server: u64, action: u64) {
    let original: Input = std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
    if let Some(r) = services::active() {
        if let Ok(Some(open)) = r.request(action) {
            if let Err(e) = request(r, server, open) {
                r.warn(&format!("Official menu request rejected: {e}"));
            }
        }
    }
    original(server, action);
}
pub fn command(r: &dyn Services, c: &mut crate::stub::Context) -> bool {
    if r.opcode() != Some(c.r9 as u16) {
        return false;
    }
    c.skip = 1;
    c.result = 0;
    let result = (|| -> Result<(), String> {
        if c.rdx != 1 {
            return Err("Menu request is not from a native peer".into());
        }
        let slot = r.peer(c.rcx, c.r8)?;
        let (epoch, open) = r.wire(c.rsp_entry)?;
        let p = r.party(c.rcx)?
            .ok_or("Missing menu party")?;
        let a = p
            .actors
            .iter()
            .find(|a| a.slot == slot && a.phase != asx_gameplay::Phase::Inactive)
            .ok_or("Menu owner departed")?;
        let mut s = STATE.lock().map_err(|_| "Menu state unavailable")?;
        s.reconcile(r.game_address(c.rcx)?, &p);
        if epoch != s.epoch {
            return Err("Stale menu session".into());
        }
        s.set(slot, a.guid, open);
        r.info(&format!("official peer menu: slot={slot} open={open}"));
        Ok(())
    })();
    if let Err(e) = result {
        r.warn(&format!("Official menu request rejected: {e}"));
    }
    true
}
pub fn hooks(a: Adapter) -> Vec<crate::install::HookSpec> {
    let mut hooks = Vec::new();
    if let Some(e) = a.menu_entry() {
        hooks.push(crate::install::HookSpec {
            name: "official-menu-input",
            target: local_base() + e.rva as usize,
            detour: input as *const () as usize,
            trampoline: &ORIGINAL,
        });
    }
    if let Some(e) = a.menu_key_entry() {
        hooks.push(crate::install::HookSpec {
            name: "official-native-escape",
            target: local_base() + e.rva as usize,
            detour: key_entry as *const () as usize,
            trampoline: &KEY_ORIGINAL,
        });
    }
    hooks
}
#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "system" fn native(server: u64, action: u64) {
        assert_eq!((server, action), (0x12340, 0x56780));
    }
    #[test]
    fn input_forwards_borrowed_native_action_and_receiver() {
        ORIGINAL.store(native as *const () as usize, Ordering::Release);
        unsafe { input(0x12340, 0x56780) }
    }
}

#[cfg(feature = "test-hooks")]
static MENU_SEEN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
#[cfg(feature = "test-hooks")]
pub fn fixture_view(v: &asx_gameplay::official::PausePresentation) {
    if v.active && v.title == "Game paused" {
        let _ = MENU_SEEN.set(std::time::Instant::now());
    }
}
#[cfg(feature = "test-hooks")]
pub fn fixture(r: &Runtime, server: u64) {
    if !super::fixture::pause_enabled() {
        return;
    }
    static STAGE: Mutex<(u8, Option<std::time::Instant>)> = Mutex::new((0, None));
    let host = std::env::var("AZCOOP_TEST_OFFICIAL_PAUSE_HOST").as_deref() == Ok("1");
    let event = {
        let Ok(mut stage) = STAGE.lock() else { return };
        let ready = if host {
            r.party.lock().ok().is_some_and(|p| {
                p.as_ref().is_some_and(|p| {
                    p.clock > 15.
                        && p.actors
                            .iter()
                            .all(|a| a.phase == asx_gameplay::Phase::Playing)
                })
            })
        } else {
            MENU_SEEN
                .get()
                .is_some_and(|at| at.elapsed().as_secs_f32() > 2.)
        };
        if stage.0 == 0 && ready {
            *stage = (1, Some(std::time::Instant::now()));
            Some(true)
        } else if stage.0 == 1
            && stage
                .1
                .is_some_and(|at| at.elapsed().as_secs_f32() > if host { 12. } else { 18. })
        {
            stage.0 = 2;
            Some(false)
        } else {
            None
        }
    };
    if let Some(open) = event {
        // The reviewed native Input::Pause variant has no owned payload. Its native
        // drop routine is a no-op for tag 7; the borrowed stack action stays valid.
        let mut action = [0u64; 8];
        let bytes = unsafe { std::slice::from_raw_parts_mut(action.as_mut_ptr() as *mut u8, 64) };
        bytes[..4].copy_from_slice(&7u32.to_le_bytes());
        bytes[4] = u8::from(open);
        unsafe { input(server, action.as_ptr() as u64) };
        r.log
            .info(&format!("native menu fixture: host={host} open={open}"));
    }
}

#[cfg(test)]
mod escape_abi {
    use super::*;
    unsafe extern "system" fn original(a: u64, b: u64, c: u64, d: u64, e: u64) -> u64 {
        assert_eq!([a, b, c, d, e], [11, 22, 33, 44, 55]);
        77
    }
    #[test]
    fn native_escape_fallback_keeps_registers_and_stack() {
        KEY_ORIGINAL.store(original as *const () as usize, Ordering::Release);
        let call: unsafe extern "system" fn(u64, u64, u64, u64, u64) -> u64 =
            unsafe { std::mem::transmute(key_entry as *const ()) };
        assert_eq!(unsafe { call(11, 22, 33, 44, 55) }, 77);
    }
}

#[cfg(test)]
mod coordination_tests;
