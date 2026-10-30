//! Host-owned fatigue. Reuses world, damage, healing and addon boundaries.
use super::*;
use asx_gameplay::resurrection_fatigue::Fatigue;
use std::collections::BTreeMap;
#[derive(Default)]
struct State {
    game: u64,
    clock: f32,
    actors: BTreeMap<(u32, u64), Fatigue>,
}
static STATE: std::sync::Mutex<Option<State>> = std::sync::Mutex::new(None);
static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
thread_local! { static SOURCE: std::cell::Cell<Option<(u64,u32,u64)>> = const { std::cell::Cell::new(None) }; }
pub fn reset() {
    if let Ok(mut state) = STATE.lock() {
        *state = None;
        ACTIVE.store(false, Ordering::Release);
    }
}
pub fn observe(game: u64, party: Option<&asx_game::official::PartyObservation>) {
    let Some(party) = party.filter(|p| p.clock.is_finite()) else {
        reset();
        return;
    };
    let Ok(mut guard) = STATE.lock() else {
        return;
    };
    let state = guard.get_or_insert_with(State::default);
    if state.game != game || party.clock < state.clock {
        *state = State {
            game,
            clock: party.clock,
            ..State::default()
        };
    }
    let elapsed = (party.clock - state.clock).max(0.0);
    state.clock = party.clock;
    state.actors.retain(|(slot, guid), _| {
        party
            .actors
            .iter()
            .any(|a| a.slot == *slot && a.guid == *guid)
    });
    for actor in &party.actors {
        state
            .actors
            .entry((actor.slot, actor.guid))
            .or_default()
            .observe(actor.phase, elapsed);
    }
    ACTIVE.store(
        state.actors.values().any(|f| f.stacks > 0),
        Ordering::Release,
    );
}
pub fn view(slot: u32, guid: u64) -> Fatigue {
    STATE
        .lock()
        .ok()
        .and_then(|s| s.as_ref()?.actors.get(&(slot, guid)).copied())
        .unwrap_or_default()
}
pub fn for_source(game: u64, slot: u32, guid: u64) -> f32 {
    STATE
        .lock()
        .ok()
        .and_then(|s| {
            let s = s.as_ref().filter(|s| s.game == game)?;
            s.actors.get(&(slot, guid)).map(|f| f.multiplier())
        })
        .unwrap_or(1.0)
}
pub fn multiplier(runtime: &Runtime, game: u64) -> f32 {
    if !ACTIVE.load(Ordering::Acquire) {
        return 1.0;
    }
    if let Some((_, slot, guid)) = SOURCE.with(|s| s.get()).filter(|(g, _, _)| *g == game) {
        return for_source(game, slot, guid);
    }
    runtime
        .adapter
        .effect_source(&runtime.local, game)
        .ok()
        .flatten()
        .map(|(slot, guid)| for_source(game, slot, guid))
        .unwrap_or(1.0)
}
/// Scope the ORIGINAL producer, not the currently swapped recipient. Restore
/// even on unwind; no cache lock crosses a potentially reentrant native call.
pub fn with_source<T>(game: u64, slot: u32, guid: u64, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<(u64, u32, u64)>);
    impl Drop for Restore {
        fn drop(&mut self) {
            SOURCE.with(|s| s.set(self.0));
        }
    }
    let _restore = Restore(SOURCE.with(|s| s.replace(Some((game, slot, guid)))));
    f()
}

#[cfg(feature = "test-hooks")]
static TRACE: [AtomicUsize; 4] = [const { AtomicUsize::new(0) }; 4];
#[cfg(feature = "test-hooks")]
pub fn fixture_arm() {
    for budget in &TRACE {
        budget.store(12, Ordering::Release);
    }
}
#[cfg(feature = "test-hooks")]
pub fn fixture_trace(runtime: &Runtime, game: u64, kind: usize, raw: f32, scaled: f32) {
    // This fixture penalizes slot 1. Do not spend its small periodic-readout
    // budget on healthy slot-0 effects left by an earlier support case.
    if matches!(kind,2|3) && !SOURCE.with(|s|s.get().is_some_and(|(_,slot,_)|slot==1)){return}

    if TRACE[kind]
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |v| v.checked_sub(1))
        .is_ok()
    {
        let identity = SOURCE
            .with(|s| s.get())
            .filter(|(g, _, _)| *g == game)
            .map(|(_, s, g)| (s, g))
            .or_else(|| {
                runtime
                    .adapter
                    .effect_source(&runtime.local, game)
                    .ok()
                    .flatten()
            });
        runtime.log.info(&format!(
            "fatigue fixture effect: kind={kind} source={identity:?} raw={raw} scaled={scaled}"
        ));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fatigue_identity_lifecycle_and_scoped_healing_owner() {
        use asx_game::official::{ActorObservation, PartyObservation};
        let actor = |slot, guid, phase| ActorObservation {
            slot,
            guid,
            phase,
            gold: 0,
            level: 1,
            xp: 0,
            class_id: 1,
            race_id: 1,
            xp_multiplier: 1.0,
            gold_multiplier: 1.0,
            hp: 100.0,
            maximum: 100.0,
            position: [0.0; 3],
        };
        let mut p = PartyObservation {
            clock: 1.0,
            actors: vec![
                actor(0, 10, asx_gameplay::Phase::Playing),
                actor(1, 20, asx_gameplay::Phase::Downed),
            ],
        };
        reset();
        observe(123, Some(&p));
        p.clock = 2.0;
        p.actors[1].phase = asx_gameplay::Phase::Playing;
        observe(123, Some(&p));
        assert_eq!(for_source(123, 1, 20), 0.95);
        assert_eq!(for_source(123, 0, 10), 1.0);
        assert_eq!(for_source(124, 1, 20), 1.0);
        assert_eq!(for_source(123, 1, 21), 1.0);
        with_source(123, 1, 20, || {
            assert_eq!(SOURCE.with(|s| s.get()), Some((123, 1, 20)));
            with_source(123, 0, 10, || {
                assert_eq!(SOURCE.with(|s| s.get()), Some((123, 0, 10)))
            });
            assert_eq!(SOURCE.with(|s| s.get()), Some((123, 1, 20)));
        });
        assert_eq!(SOURCE.with(|s| s.get()), None);
        with_scaled_heal(123, true, || {
            assert_eq!(PRE_SCALED_HEAL.with(|s| s.get()), Some(123));
            with_scaled_heal(124, false, || {
                assert_eq!(PRE_SCALED_HEAL.with(|s| s.get()), None)
            });
            assert_eq!(PRE_SCALED_HEAL.with(|s| s.get()), Some(123));
        });
        assert_eq!(PRE_SCALED_HEAL.with(|s| s.get()), None);
        p.actors[1].guid = 21;
        observe(123, Some(&p));
        assert_eq!(for_source(123, 1, 21), 1.0);
        p.clock = 0.0;
        observe(123, Some(&p));
        assert_eq!(view(1, 21).stacks, 0);
        observe(123, None);
        assert!(!ACTIVE.load(Ordering::Acquire));
    }
}

thread_local! { static PRE_SCALED_HEAL: std::cell::Cell<Option<u64>> = const {std::cell::Cell::new(None)}; }
/// The descriptor has already received all source-owned outgoing-heal factors.
pub fn heal_pre_scaled(game:u64)->bool{PRE_SCALED_HEAL.with(|v|v.get())==Some(game)}
pub fn healing_multiplier(runtime: &Runtime, game: u64) -> f32 {
    if heal_pre_scaled(game) {
        1.0
    } else {
        multiplier(runtime, game)
    }
}
pub fn with_scaled_heal<T>(game: u64, scaled: bool, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<u64>);
    impl Drop for Restore {
        fn drop(&mut self) {
            PRE_SCALED_HEAL.with(|v| v.set(self.0));
        }
    }
    let _restore =
        Restore(PRE_SCALED_HEAL.with(|v| v.replace(if scaled { Some(game) } else { None })));
    f()
}

static AURA_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(aura_entry, 0, aura, AURA_ORIGINAL);
unsafe extern "system" fn aura(c: &mut crate::stub::Context) {
    if !ACTIVE.load(Ordering::Acquire) && !blessings::ACTIVE.load(Ordering::Acquire)&&!super::supplies::active(){return;}
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let regs = [
        c.rax, c.rcx, c.rdx, c.r8, c.r9, c.r10, c.r11, c.rbx, c.rbp, c.rsi, c.rdi, c.r12, c.r13,
        c.r14, c.r15,
    ];
    let Ok(Some(frame)) = r.adapter.fatigue_aura_frame(&r.local, regs) else {
        return;
    };
    if blessings::view(frame.slot,frame.guid).is_some(){
        let io=io::Access{local:&r.local,adapter:r.adapter};
        match r.adapter.project_blessing_aura(&io,&frame){
            Err(e)=>r.log.warn(&format!("Blessing aura rejected: {e}")),
            Ok(true)=>{#[cfg(feature="test-hooks")]if std::env::var("AZCOOP_TEST_EXTENSIONS").as_deref()==Ok("1"){
                static SEEN:std::sync::Mutex<std::collections::BTreeSet<(u32,u32)>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
                if let Some(b)=blessings::view(frame.slot,frame.guid){if SEEN.lock().ok().is_some_and(|mut s|s.insert((frame.slot,b.boss))){r.log.info(&format!("extensions native fortune aura: slot={} boss={}",frame.slot,b.boss));}}
            }},Ok(false)=>{}
        }
    }
    let consumables=super::supplies::buffs(frame.game,frame.guid).into_iter().filter(|(_,left,_)|*left>0.).map(|(c,left,_)|(c.family,left)).collect::<Vec<_>>();
    if !consumables.is_empty()||super::supplies::has_aura_projection(frame.game,frame.guid){
        let io=io::Access{local:&r.local,adapter:r.adapter};
        match r.adapter.project_consumable_auras(&io,&frame,&consumables){Ok(rows)=>super::supplies::aura(frame.game,frame.guid,&rows),Err(e)=>r.log.warn(&format!("Consumable aura rejected: {e}"))}
    }
    #[cfg(feature="test-hooks")]if std::env::var("ASX_TEST_LOST_SUPPLIES").as_deref()==Ok("1"){
        static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
        let count=consumables.len();let bit=1usize<<count;
        if SEEN.fetch_or(bit,Ordering::Relaxed)&bit==0{r.log.info(&format!("supplies native aura rows: families={count} rows={:?}",r.adapter.event_aura_readback(&r.local,&frame)));}
    }
    let fatigue = view(frame.slot, frame.guid);
    if fatigue.stacks == 0 {
        return;
    }
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    match r.adapter.project_fatigue_aura(&io,&frame,fatigue.stacks,fatigue.remaining){
        Err(e)=>r.log.warn(&format!("Native fatigue aura rejected: {e}")),
        Ok(true)=>{#[cfg(feature="test-hooks")]if std::env::var("AZCOOP_TEST_OFFICIAL_FATIGUE").as_deref()==Ok("1"){
            static SEEN:std::sync::Mutex<std::collections::BTreeSet<(u64,u8)>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
            if SEEN.lock().ok().is_some_and(|mut s|s.insert((frame.guid,fatigue.stacks))){r.log.info(&format!("fatigue121 native aura: slot={} stacks={} remaining={}",frame.slot,fatigue.stacks,fatigue.remaining));}
        }},Ok(false)=>{}
    }
}
pub fn hooks(adapter: Adapter) -> Vec<crate::install::HookSpec> {
    adapter
        .fatigue_aura_entry()
        .map(|e| crate::install::HookSpec {
            name: "official-fatigue-aura",
            target: local_base() + e.rva as usize,
            detour: aura_entry as *const () as usize,
            trampoline: &AURA_ORIGINAL,
        })
        .into_iter()
        .collect()
}
