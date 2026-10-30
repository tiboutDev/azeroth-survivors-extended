//! Optional host-owned events over native creatures. No separate simulation/transport.
use super::event_services::Services;
use super::{io, Runtime};
use asx_game::official::{GoblinDefinition, GoblinHandle};
use asx_gameplay::treasure_goblins::{flee_candidates, whole_boss_marks, Director, Resolution};
use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};
mod appearance;
mod motion;
#[cfg(feature = "test-hooks")]
mod motion_evidence;
pub(super) mod rewards;
static AVAILABLE: AtomicBool = AtomicBool::new(false);
static ACTIVE: AtomicBool = AtomicBool::new(false);
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static SKIN_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
// Native records retain definition pointers, including after death. These twelve
// immutable, map-specific templates therefore live for the process lifetime.
static DEFINITIONS: [OnceLock<GoblinDefinition>; 12] = [const { OnceLock::new() }; 12];
thread_local! {static PENDING:Cell<Option<(u64,u64)>>=const{Cell::new(None)};}
struct State {
    game: u64,
    arena: Option<asx_gameplay::treasure_goblins::Arena>,
    owner: u64,
    clock: f32,
    rules: Director,
    handle: Option<GoblinHandle>,
    goal: Option<[f32; 3]>,
    heading: [f32; 2],
    navigation: asx_gameplay::treasure_goblins::FleeNavigation,
    next_goal: f32,
    stopped: bool,
    killed: bool,
    spawns: u32,
    members: Vec<(u32, u64)>,
    pursuers: Vec<(u32, [f32; 3])>,
}
static STATE: Mutex<Option<State>> = Mutex::new(None);
crate::context_stub_rw!(@emit definition_entry,0,definition,ORIGINAL,["mov r15, [rsp + 32 + {r15}]"]);
unsafe extern "system" fn definition(c: &mut crate::stub::Context) {
    PENDING.with(|pending| {
        if let Some((game, p)) = pending.get() {
            if c.rsi == game {
                pending.set(None);
                c.r15 = p;
            }
        }
    });
}
unsafe extern "system" fn skin_path(output: u64, name: u64, length: usize) -> u64 {
    if let Some(r) = super::RUNTIME.get() {
        let io = io::Access {
            local: &r.local,
            adapter: r.adapter,
        };
        match r.adapter.goblin_skin_path(&io, output, name, length) {
            Ok(true) => {
                #[cfg(feature = "test-hooks")]
                {
                    static ONCE: AtomicBool = AtomicBool::new(false);
                    if !ONCE.swap(true, Ordering::Relaxed) {
                        r.log.info("Treasure Goblin native body atlas selected.");
                    }
                }
                return output;
            }
            Ok(false) => {}
            Err(e) => r
                .log
                .warn(&format!("Goblin appearance fallback rejected: {e}")),
        }
    }
    type Original = unsafe extern "system" fn(u64, u64, usize) -> u64;
    let original: Original = std::mem::transmute(SKIN_ORIGINAL.load(Ordering::Acquire));
    original(output, name, length)
}
#[cfg(feature = "test-hooks")]
static SKIN_DISPATCH: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "test-hooks")]
crate::context_stub_rw!(skin_probe_entry, 0, skin_probe, SKIN_DISPATCH);
#[cfg(feature = "test-hooks")]
unsafe extern "system" fn skin_probe(c: &mut crate::stub::Context) {
    let Some(r) = super::RUNTIME.get() else {
        return;
    };
    static ONCE: AtomicBool = AtomicBool::new(false);
    if ONCE.load(Ordering::Relaxed) {
        return;
    }
    if let Ok(Some(info)) = r.adapter.goblin_skin_probe(
        &r.local,
        c.rdx,
        c.r8 as usize,
        c.rbp,
        c.return_address().saturating_sub(r.local.base as u64),
    ) {
        if !ONCE.swap(true, Ordering::Relaxed) {
            r.log
                .info(&format!("treasure goblin appearance path: {info}"));
        }
    }
}
pub fn install(r: &Runtime) {
    if !r.rules {
        return;
    }
    let disabled = std::env::var("ASX_TREASURE_GOBLINS").as_deref() == Ok("0");
    // A guest may join a host with events enabled even if local generation is off.
    // Keep this exact legacy skin association available without enabling a director.
    let skin = (|| -> Result<(), String> {
        let e = r.adapter.goblin_skin_entry();
        if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
            return Err("Optional Goblin skin boundary changed".into());
        }
        #[allow(unused_mut)]
        let mut skin_detour = skin_path as *const () as usize;
        #[cfg(feature = "test-hooks")]
        if fixture_enabled() {
            SKIN_DISPATCH.store(skin_detour, Ordering::Release);
            skin_detour = skin_probe_entry as *const () as usize;
        }
        crate::install::install_optional(crate::install::HookSpec {
            name: "optional-goblin-skin",
            target: r.local.base + e.rva as usize,
            detour: skin_detour,
            trampoline: &SKIN_ORIGINAL,
        })?;
        appearance::install(r)
    })();
    if let Err(e) = skin {
        r.log.warn(&format!(
            "Treasure Goblins unavailable: {e}. Core ASX coop is unaffected."
        ));
        return;
    }
    if disabled {
        r.log
            .info("Treasure Goblins disabled; core ASX coop remains active.");
        return;
    }
    let result = (|| -> Result<(), String> {
        for e in r.adapter.goblin_entries() {
            if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
                return Err(format!(
                    "Optional creature boundary changed at {:#x}",
                    e.rva
                ));
            }
        }
        let e = r.adapter.goblin_definition_entry();
        crate::install::install_optional(crate::install::HookSpec {
            name: "optional-treasure-goblin-definition",
            target: r.local.base + e.rva as usize,
            detour: definition_entry as *const () as usize,
            trampoline: &ORIGINAL,
        })?;
        motion::install(r)?;
        rewards::install(r)?;
        #[cfg(feature = "test-hooks")]
        if fixture_enabled() {
            let e = r.adapter.goblin_packet_probe_entry();
            if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
                return Err("Goblin diagnostic packet boundary changed".into());
            }
            crate::install::install_optional(crate::install::HookSpec {
                name: "goblin-diagnostic-packet-observer",
                target: r.local.base + e.rva as usize,
                detour: packet_probe as *const () as usize,
                trampoline: &PACKET_ORIGINAL,
            })?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            AVAILABLE.store(true, Ordering::Release);
            r.log
                .info("Treasure Goblins enabled: optional native creature integration ready.");
        }
        Err(e) => r.log.warn(&format!(
            "Treasure Goblins unavailable: {e}. Core ASX coop is unaffected."
        )),
    }
}
pub fn encounter(game: u64, normal_bosses:u32) {
    if !AVAILABLE.load(Ordering::Acquire) {
        return;
    }
    if let Ok(mut state) = STATE.lock() {
        if let Some(s) = state.as_mut().filter(|s| s.game == game) {
            s.rules.encounter(s.clock,normal_bosses);
        }
    }
}
pub fn reset() {
    rewards::reset();
    ACTIVE.store(false, Ordering::Release);
    if let Ok(mut s) = STATE.lock() {
        *s = None;
    }
}
pub fn active() -> bool {
    ACTIVE.load(Ordering::Acquire)
}
pub fn is_goblin(game: u64, enemy: u64) -> bool {
    active()
        && STATE.lock().is_ok_and(|s| {
            s.as_ref()
                .is_some_and(|s| s.game == game && s.rules.active().is_some_and(|a| a.id == enemy))
        })
}
pub(super) fn identity() -> Option<(u64, u64)> {
    if !active() {
        return None;
    }
    STATE.lock().ok().and_then(|s| {
        let s = s.as_ref()?;
        Some((s.game, s.rules.active()?.id))
    })
}
pub fn pursuit(game: u64, enemy: u64, origin: [f32; 3], owner: [f32; 3]) -> Option<[f32; 3]> {
    if !active() {
        return None;
    }
    let state = STATE.lock().ok()?;
    let s = state.as_ref()?;
    if s.game != game || s.rules.active()?.id != enemy {
        return None;
    }
    let pursuers = s
        .pursuers
        .iter()
        .map(|(slot, p)| if *slot == 0 { owner } else { *p })
        .collect::<Vec<_>>();
    Some(
        s.goal
            .filter(|goal| asx_gameplay::treasure_goblins::escapes(origin, *goal, &pursuers))
            .unwrap_or(origin),
    )
}
pub(super) fn safe_motion(game: u64, enemy: u64, origin: [f32; 3], goal: [f32; 3]) -> bool {
    let Ok(state) = STATE.lock() else {
        return false;
    };
    let Some(s) = state
        .as_ref()
        .filter(|s| s.game == game && s.rules.active().is_some_and(|a| a.id == enemy))
    else {
        return false;
    };
    let pursuers = s.pursuers.iter().map(|(_, p)| *p).collect::<Vec<_>>();
    s.arena.is_some_and(|a| a.contains(goal))
        && asx_gameplay::treasure_goblins::escapes(origin, goal, &pursuers)
}
pub fn hit_complete(r: &Runtime, game: u64, enemy: u64, spell: u32) {
    if !is_goblin(game, enemy) {
        return;
    }
    if r.adapter
        .combat_hp(&r.local, game, enemy)
        .ok()
        .is_some_and(|hp| hp.is_none_or(|v| v <= 0.))
    {
        if let Ok(mut s) = STATE.lock() {
            if let Some(s) = s.as_mut().filter(|s| s.game == game) {
                s.killed = true;
            }
        }
    }
    #[cfg(feature = "test-hooks")]
    if matches!(spell, 23922 | 853 | 5221) {
        r.log.info(&format!(
            "treasure goblin taunt rejected: enemy={enemy} spell={spell}"
        ));
    }
    #[cfg(not(feature = "test-hooks"))]
    let _ = spell;
}
fn notice(r:&dyn Services,game:u64,text:&str){if let Err(e)=r.events_goblin_notice(game,text){r.events_warn(&format!("Goblin notice rejected: {e}"));}}
pub fn finish(r: &dyn Services, game: u64) {
    if !AVAILABLE.load(Ordering::Acquire) {
        return;
    }
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tick(r, game)));
    let error = match outcome {
        Ok(Ok(())) => None,
        Ok(Err(e)) => Some(e),
        Err(_) => Some("Optional event callback panicked".into()),
    };
    if let Some(e) = error {
        AVAILABLE.store(false, Ordering::Release);
        r.events_warn(&format!(
            "Treasure Goblin event stopped: {e}; core ASX remains active."
        ));
        ACTIVE.store(false, Ordering::Release);
        let handle = STATE.lock().ok().and_then(|mut state| {
            let s = state.as_mut()?;
            s.stopped = true;
            s.goal = None;
            s.rules.abandon();
            s.handle.take()
        });
        if let Some(mut h) = handle {
            
            if let Err(error) = unsafe {
                r.events_despawn_goblin(game, &mut h)
            } {
                r.events_warn(&format!("Goblin cleanup failed once: {error}"));
            }
        }
    }
}
fn tick(r: &dyn Services, game: u64) -> Result<(), String> {
    let Some(p) = r.events_goblin_run(game)? else {
        return Ok(());
    };
    let Some(host) = p.actors.iter().find(|a| a.slot == 0) else {
        return Ok(());
    };
    r.events_finish_goblin_rewards(
        game,
        &p.actors
            .iter()
            .filter(|a| a.phase != asx_gameplay::Phase::Inactive)
            .map(|a| (a.slot, a.guid))
            .collect::<Vec<_>>(),
    )?;
    let now = p.clock;
    if !now.is_finite() {
        return Err("Invalid event clock".into());
    }
    {
        let mut state = STATE.lock().map_err(|_| "Event state unavailable")?;
        if state
            .as_ref()
            .is_none_or(|s| s.game != game || s.owner != host.guid || now < s.clock)
        {
            *state = Some(State {
                game,
                arena: None,
                owner: host.guid,
                clock: now,
                rules: Director::new(
                    host.guid
                        ^ game
                        ^ std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_nanos() as u64)
                            .unwrap_or(0),
                ),
                handle: None,
                goal: None,
                heading: [1., 0.],
                navigation: Default::default(),
                next_goal: 0.,
                stopped: false,
                killed: false,
                spawns: 0,
                members: Vec::new(),
                pursuers: Vec::new(),
            });
            ACTIVE.store(false, Ordering::Release);
        }
        state.as_mut().unwrap().clock = now;
        state.as_mut().unwrap().rules.ensure_first(now);
        state.as_mut().unwrap().pursuers = p
            .actors
            .iter()
            .filter(|a| a.hp > 0. && a.phase != asx_gameplay::Phase::Inactive)
            .map(|a| (a.slot, a.position))
            .collect();
    }
    let arena = STATE
        .lock()
        .ok()
        .and_then(|s| s.as_ref().and_then(|s| s.arena));
    let arena = match arena {
        Some(a) => a,
        None => {
            let a = r.events_goblin_arena(game)?;
            if let Ok(mut s) = STATE.lock() {
                if let Some(s) = s.as_mut() {
                    s.arena = Some(a);
                }
            }
            #[cfg(feature = "test-hooks")]
            if fixture_enabled() {
                r.events_info(&format!(
                    "treasure goblin arena: center={:?} radius={} clearance=2",
                    a.center, a.radius
                ));
            }
            a
        }
    };
    let work = STATE.lock().ok().and_then(|s| {
        s.as_ref()
            .map(|s| (s.handle, s.killed, s.next_goal, s.heading, s.stopped))
    });
    let Some((handle, killed, next_goal, heading, stopped)) = work else {
        return Ok(());
    };
    
    if let Some(mut h) = handle {
        let members = p
            .actors
            .iter()
            .filter(|a| a.phase != asx_gameplay::Phase::Inactive)
            .map(|a| (a.slot, a.guid))
            .collect::<Vec<_>>();
        let refresh = STATE.lock().ok().is_some_and(|mut state| {
            let Some(s) = state.as_mut() else {
                return false;
            };
            let fresh = members.iter().any(|m| !s.members.contains(m));
            s.members = members;
            fresh
        });
        if refresh {
            unsafe { r.events_goblin_name(game) }?;
        }
        let snapshot = r.events_goblin_snapshot(game, &mut h)?;
        #[cfg(feature = "test-hooks")]
        if fixture_enabled() {
            if let Some(v) = snapshot {
                motion_evidence::observe(h.guid(), now, v.position, &p);
            }
        }
        if snapshot.is_none() && !killed {
            ACTIVE.store(false, Ordering::Release);
            if let Ok(mut state) = STATE.lock() {
                if let Some(s) = state.as_mut() {
                    s.rules.abandon();
                    s.handle = None;
                    s.goal = None;
                }
            }
            r.events_info("Treasure Goblin removed by native lifecycle; no reward.");
            return Ok(());
        }
        let resolution = {
            let mut state = STATE.lock().map_err(|_| "Event state unavailable")?;
            let s = state.as_mut().ok_or("Event state reset")?;
            s.handle = Some(h);
            s.rules
                .resolve(now, !killed && snapshot.is_some_and(|a| a.alive))
        };
        if let Some(result) = resolution {
            #[cfg(feature = "test-hooks")]
            if fixture_enabled() {
                motion_evidence::report(r, h.guid());
            }
            ACTIVE.store(false, Ordering::Release);
            {
                let mut state = STATE.lock().map_err(|_| "Event state unavailable")?;
                if let Some(s) = state.as_mut() {
                    s.handle = None;
                    s.goal = None;
                    s.killed = false;
                }
            }
            match result {
                Resolution::Killed { id } => {
                    let ids = p
                        .actors
                        .iter()
                        .filter(|a| a.phase != asx_gameplay::Phase::Inactive)
                        .map(|a| (a.slot, a.guid))
                        .collect::<Vec<_>>();
                    let marks =
                        r.events_goblin_marks_amount(game)?;
                    let marks =
                        whole_boss_marks(marks).ok_or("Invalid goblin boss mark amount")? as f32;
                    rewards::queue(game, id, marks, &ids);
                    r.events_info(&format!(
                        "treasure goblin rewarded: guid={id} recipients={}",
                        ids.len()
                    ));
                    notice(r, game, "Venture Co. Treasurer defeated!");
                }
                Resolution::Escaped { id } => {
                    unsafe {
                        r.events_despawn_goblin(game, &mut h)
                    }?;
                    r.events_info(&format!("treasure goblin escaped: guid={id}"));
                    notice(r, game, "The Venture Co. Treasurer escaped!");
                }
            }
            return Ok(());
        }
        #[cfg(feature = "test-hooks")]
        r.events_fixture_goblin(game, now, h, &p);
        if now >= next_goal {
            if let Some(snapshot) = snapshot {
                let pursuers = p
                    .actors
                    .iter()
                    .filter(|a| a.hp > 0. && a.phase != asx_gameplay::Phase::Inactive)
                    .map(|a| a.position)
                    .collect::<Vec<_>>();
                let (mut navigation, previous_goal) = STATE
                    .lock()
                    .ok()
                    .and_then(|s| s.as_ref().map(|s| (s.navigation, s.goal)))
                    .ok_or("Missing flee navigation")?;
                navigation.observe(now, snapshot.position, previous_goal);
                let mut reachable = Vec::with_capacity(7);
                for candidate in flee_candidates(snapshot.position, &pursuers, heading) {
                    let clipped = unsafe {
                        r.events_goblin_clip(game, snapshot.position, candidate)
                    }?;
                    if let Some(clipped) = arena.clip(snapshot.position, clipped) {
                        reachable.push(clipped);
                    }
                }
                let goal = navigation.choose(snapshot.position, &pursuers, heading, &reachable);
                if let Ok(mut s) = STATE.lock() {
                    if let Some(s) = s.as_mut().filter(|s| {
                        s.game == game && s.handle.is_some_and(|active| active.guid() == h.guid())
                    }) {
                        s.navigation = navigation;
                        s.next_goal = now + 0.4;
                        s.goal = goal;
                        if let Some(goal) = goal {
                            s.heading = [
                                goal[0] - snapshot.position[0],
                                goal[1] - snapshot.position[1],
                            ];
                            s.goal = Some(goal);
                        }
                    }
                }
            }
        }
        return Ok(());
    }
    if stopped
        || super::pause::active()
        || !p
            .actors
            .iter()
            .any(|a| a.phase == asx_gameplay::Phase::Playing && a.hp > 0.)
    {
        return Ok(());
    }
    #[cfg(feature="test-hooks")]
    let observed_encounter_ready = !super::scenario_steps::enabled() ||
        (p.actors.iter().all(|a|a.phase==asx_gameplay::Phase::Playing)&&r.events_blessing_fixture_targets(game)?.is_empty());
    #[cfg(feature="test-hooks")]
    let observed_rewards_ready = !super::scenario_steps::enabled() || super::world_events::coop::observed_rewards_complete(game);
    let due = {
        let mut state = STATE.lock().map_err(|_| "Event state unavailable")?;
        let s = state.as_mut().ok_or("Event state reset")?;
        #[cfg(feature = "test-hooks")]
        let forced = std::env::var("ASX_TEST_TREASURE_GOBLINS").as_deref() == Ok("1")
            && s.spawns < 2
            && if super::scenario_steps::enabled(){observed_encounter_ready&&s.rules.encounters()>0&&s.spawns<2&&(s.spawns==0||observed_rewards_ready)}else{now >= if s.spawns == 0 {21.}else{48.}};
        #[cfg(not(feature = "test-hooks"))]
        let forced = false;
        if !s.rules.due(now) && !forced {
            false
        } else {
            s.rules.reserve(now);
            true
        }
    };
    if !due {
        return Ok(());
    }
    let (map, template) = r.events_goblin_template(game)?;
    let definition = DEFINITIONS[map].get_or_init(|| template);
    #[cfg(feature = "test-hooks")]
    if fixture_enabled() {
        r.events_info(&format!(
            "treasure goblin template: map={map} base_hp={}",
            definition.base_health()
        ));
    }
    unsafe { r.events_goblin_name(game) }?;
    struct PendingGuard(Option<(u64, u64)>);
    impl Drop for PendingGuard {
        fn drop(&mut self) {
            PENDING.with(|p| p.set(self.0));
        }
    }
    let guard = PendingGuard(PENDING.with(|p| p.replace(Some((game, definition.pointer())))));
    let spawned = unsafe { r.events_spawn_goblin(game) };
    drop(guard);
    let Some(handle) = spawned? else {
        r.events_info("Treasure Goblin opportunity skipped: native capacity/placement rejected it.");
        return Ok(());
    };
    let mut initial_handle = handle;
    let initial_position = r.events_goblin_snapshot(game, &mut initial_handle)?
        .map(|v| v.position);
    {
        let mut state = STATE.lock().map_err(|_| "Event state unavailable")?;
        let s = state.as_mut().ok_or("Event state reset")?;
        if !s.rules.spawned(handle.guid(), now) {
            return Err("Event creation lost its reservation".into());
        }
        s.handle = Some(handle);
        s.navigation = Default::default();
        s.spawns += 1;
        s.members = p
            .actors
            .iter()
            .filter(|a| a.phase != asx_gameplay::Phase::Inactive)
            .map(|a| (a.slot, a.guid))
            .collect();
        s.goal = initial_position;
        s.next_goal = now + asx_gameplay::treasure_goblins::ARRIVAL_SECONDS;
    }
    r.events_goblin_elite(game, &mut initial_handle)?;
    #[cfg(feature="test-hooks")]
    if super::scenario_steps::enabled(){
        let second=STATE.lock().ok().is_some_and(|s|s.as_ref().is_some_and(|s|s.spawns==2));
        if second{if let Some(position)=initial_position{
            let dx=host.position[0]-position[0];let dy=host.position[1]-position[1];let length=dx.hypot(dy).max(0.1);
            let point=[position[0]+dx/length*40.,position[1]+dy/length*40.];
            let safe=unsafe{r.events_ground(game,point,0.3,2.)}?.ok_or("Observed goblin motion needs a walkable witness position")?;
            for actor in &p.actors{r.events_fixture_event_pose(game,actor.slot,actor.guid,safe)?;}
            r.events_info(&format!("scenario step: goblin motion witnesses relocated position={safe:?} before_sampling=true"));
        }}
    }
    ACTIVE.store(true, Ordering::Release);
    r.events_info(&format!(
        "treasure goblin spawned: game={game:x} guid={} clock={now} map={map}",
        handle.guid()
    ));
    notice(
        r,
        game,
        "A Venture Co. Treasurer is fleeing with stolen goods!",
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[unsafe(naked)]
    unsafe extern "C" fn target() {
        core::arch::naked_asm!("mov rax,r15", "ret");
    }
    #[unsafe(naked)]
    unsafe extern "system" fn invoke(_game: u64, _native_definition: u64) -> u64 {
        core::arch::naked_asm!("push r15","push rsi","sub rsp,40","mov rsi,rcx","mov r15,rdx","call {stub}","add rsp,40","pop rsi","pop r15","ret",stub=sym definition_entry);
    }
    unsafe extern "system" fn native_skin(output: u64, name: u64, len: usize) -> u64 {
        output.wrapping_add(name).wrapping_add(len as u64)
    }
    #[test]
    fn skin_wrapper_preserves_native_arguments_and_return_without_runtime() {
        let _lock = crate::test_support::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(super::super::RUNTIME.get().is_none());
        SKIN_ORIGINAL.store(native_skin as *const () as usize, Ordering::Release);
        assert_eq!(unsafe { skin_path(0x10000, 0x20000, 31) }, 0x3001f);
    }
    #[test]
    fn creature_definition_stub_changes_r15_only_for_one_scoped_native_creation() {
        let _lock = crate::test_support::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        ORIGINAL.store(target as *const () as usize, Ordering::Release);
        PENDING.with(|p| p.set(None));
        assert_eq!(unsafe { invoke(7, 100) }, 100);
        PENDING.with(|p| p.set(Some((7, 900))));
        assert_eq!(unsafe { invoke(8, 100) }, 100); // Another native owner does not consume it.
        assert_eq!(unsafe { invoke(7, 100) }, 900);
        assert_eq!(unsafe { invoke(7, 100) }, 100); // Nested/unrelated creation stays native.
        assert!(PENDING.with(|p| p.get().is_none()));
    }
}

#[cfg(feature = "test-hooks")]
fn fixture_enabled() -> bool {
    std::env::var("ASX_TEST_TREASURE_GOBLINS").as_deref() == Ok("1")
}
#[cfg(feature = "test-hooks")]
static PACKET_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "test-hooks")]
unsafe extern "system" fn packet_probe(server: u64, message: u64, now: f64) {
    let observed = RUNTIME_FOR_PROBE().and_then(|r| {
        r.adapter
            .goblin_packet_probe(&r.local, message)
            .ok()
            .flatten()
            .map(|v| (r, v))
    });
    type Original = unsafe extern "system" fn(u64, u64, f64);
    let original: Original = std::mem::transmute(PACKET_ORIGINAL.load(Ordering::Acquire));
    original(server, message, now);
    if let Some((r, (opcode, guid, position))) = observed {
        static COUNTS: Mutex<std::collections::BTreeMap<(u64, u16), (usize, std::time::Instant)>> =
            Mutex::new(std::collections::BTreeMap::new());
        let log = if let Ok(mut counts) = COUNTS.lock() {
            if counts.len() > 64 {
                false
            } else {
                let now = std::time::Instant::now();
                let (count, last) = counts.entry((guid, opcode)).or_insert((0, now));
                if opcode == 0xdd {
                    if *count >= 16 || *count > 0 && now.duration_since(*last).as_secs_f32() < 2. {
                        false
                    } else {
                        *count += 1;
                        *last = now;
                        true
                    }
                } else {
                    *count += 1;
                    *count <= 2
                }
            }
        } else {
            false
        };
        if log {
            r.log.info(&format!(
                "treasure goblin received: opcode={opcode:x} guid={guid} position={position:?}"
            ));
        }
    }
}
#[cfg(feature = "test-hooks")]
#[allow(non_snake_case)]
fn RUNTIME_FOR_PROBE() -> Option<&'static Runtime> {
    super::RUNTIME.get().filter(|r| r.rules)
}
#[cfg(feature = "test-hooks")]
pub(super) fn fixture_active(
    r: &Runtime,
    game: u64,
    now: f32,
    h: GoblinHandle,
    party: &asx_game::official::PartyObservation,
) {
    if !fixture_enabled() {
        return;
    }
    static STAGES: Mutex<std::collections::BTreeMap<u64, u8>> =
        Mutex::new(std::collections::BTreeMap::new());
    let current = STATE
        .lock()
        .ok()
        .and_then(|s| s.as_ref().and_then(|s| Some((s.rules.active()?, s.spawns))));
    let Some((a, spawns)) = current else {
        return;
    };
    let elapsed = now - (a.expires - 25.);
    let observed_control_ready = !super::scenario_steps::enabled() || r.adapter.combat_enemies(&r.local,game).ok().is_some_and(|enemies|enemies.iter().any(|e|e.alive&&e.id!=h.guid()));
    let action = {
        let Ok(mut stages) = STAGES.lock() else {
            return;
        };
        let n = stages.entry(h.guid()).or_default();
        if super::scenario_steps::enabled(){
            let action=super::scenario_steps::goblin_action(spawns,*n,motion_evidence::ready(h.guid()),observed_control_ready);
            if action>0{*n=action;}action
        }else{
        if *n == 0 && observed_control_ready && (if super::scenario_steps::enabled(){motion_evidence::ready(h.guid())}else{elapsed>=2.}) {
            *n = 1;
            1
        } else if *n == 1 && spawns == 2 && (if super::scenario_steps::enabled(){motion_evidence::ready(h.guid())}else{elapsed>=18.}) {
            *n = 2;
            2
        } else {
            0
        }
        }
    };
    if action>0&&super::scenario_steps::enabled(){r.log.info(&format!("scenario step: goblin phase={spawns} action={action} clock={now}"));}
    if action == 1 {
        super::combat::fixture_goblin_taunt(r, game, h.guid());
    }
    if action == 2 {
        use asx_game::official::SupportCalls;
        let io = io::Access {
            local: &r.local,
            adapter: r.adapter,
        };
        let _ = io.support_hit(game, h.guid(), 1e9, 23922);
    }
    static NEXT: Mutex<f32> = Mutex::new(0.);
    let log = if let Ok(mut next) = NEXT.lock() {
        if now >= *next {
            *next = now + 2.;
            true
        } else {
            false
        }
    } else {
        false
    };
    if log {
        let mut handle = h;
        if let Ok(Some(v)) = r.adapter.goblin_snapshot(&r.local, game, &mut handle) {
            r.log.info(&format!(
                "treasure goblin native: guid={} clock={now} hp={} elite={} pursuers={:?} position={:?}",
                h.guid(),
                v.hp,
                v.elite,
                party.actors.iter().filter(|a|a.hp>0.&&a.phase!=asx_gameplay::Phase::Inactive).map(|a|a.position).collect::<Vec<_>>(),
                v.position
            ));
        }
    }
}

pub(super) fn deliver_rewards(r:&Runtime,game:u64,actors:&[(u32,u64)])->Result<(),String>{rewards::finish(r,game,actors)}
