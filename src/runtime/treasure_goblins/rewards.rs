//! Personal native reward requests; reserve each recipient before native callbacks.
use super::super::{io, Runtime, RUNTIME};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};
#[derive(Clone, Copy)]
struct Request {
    game: u64,
    event: u64,
    slot: u32,
    guid: u64,
    marks: f32,
    marks_sent: bool,
}
#[derive(Default)]
struct State {
    pending: VecDeque<Request>,
    choices: BTreeMap<(u64, u32, u64), bool>,
}
static STATE: Mutex<State> = Mutex::new(State {
    pending: VecDeque::new(),
    choices: BTreeMap::new(),
});
#[derive(Clone)]
struct Scope {
    game: u64,
    keys: Vec<String>,
}
thread_local! {static SCOPE:RefCell<Option<Scope>>=const{RefCell::new(None)};}
static POOL: AtomicUsize = AtomicUsize::new(0);
static EXCLUDE: AtomicUsize = AtomicUsize::new(0);
static CHOICE: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(@emit pool_entry,0,pool,POOL,["mov rbx, [rsp + 32 + {rbx}]","mov rdi, [rsp + 32 + {rdi}]"]);
unsafe extern "system" fn pool(c: &mut crate::stub::Context) {
    let scope = SCOPE.with(|s| s.borrow().as_ref().map(|s| s.game));
    let Some(game) = scope.filter(|g| *g == c.rsi) else {
        return;
    };
    let Some(r) = RUNTIME.get() else { return };
    let m = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    match r.adapter.goblin_amulet_pool(&m, c.rbp, game) {
        Ok(()) => {
            c.rbx = 3;
            c.rdi = 3;
        }
        Err(e) => r.log.warn(&format!("Goblin amulet pool rejected: {e}")),
    }
}
unsafe extern "system" fn exclude(pool: u64, owned: u64, excluded: u64) {
    type F = unsafe extern "system" fn(u64, u64, u64);
    let original: F = std::mem::transmute(EXCLUDE.load(Ordering::Acquire));
    let keys = SCOPE.with(|s| {
        s.borrow()
            .as_ref()
            .map(|s| s.keys.clone())
            .unwrap_or_default()
    });
    if keys.is_empty() {
        original(pool, owned, excluded);
        return;
    }
    let Some(r) = RUNTIME.get() else {
        original(pool, owned, excluded);
        return;
    };
    let Ok(mut rows) = r.adapter.goblin_exclusion_keys(&r.local, excluded) else {
        original(pool, owned, excluded);
        return;
    };
    rows.extend(
        keys.iter()
            .map(|s| [s.len() as u64, s.as_ptr() as u64, s.len() as u64]),
    );
    let borrowed = [rows.len() as u64, rows.as_ptr() as u64, rows.len() as u64];
    original(pool, owned, borrowed.as_ptr() as u64);
}
unsafe extern "system" fn choice(game: u64, kind: u8, notify: bool) {
    type F = unsafe extern "system" fn(u64, u8, bool);
    let original: F = std::mem::transmute(CHOICE.load(Ordering::Acquire));
    let identity = RUNTIME
        .get()
        .and_then(|r| r.adapter.goblin_reward_actor(&r.local, game).ok().flatten())
        .map(|(s, id)| (game, s, id));
    let active = identity.and_then(|key| {
        STATE
            .lock()
            .ok()
            .and_then(|s| s.choices.get(&key).copied().map(|first| (key, first)))
    });
    let valid = active.is_some_and(|(_, first)| {
        first
            || RUNTIME.get().is_some_and(|r| {
                r.adapter.reward_choice_kind(&r.local, game).ok().flatten() == Some(1)
            })
    });
    if !valid || kind != 1 {
        if let Some((key, _)) = active {
            if let Ok(mut s) = STATE.lock() {
                s.choices.remove(&key);
            }
        }
        original(game, kind, notify);
        return;
    }
    let old = SCOPE.with(|s| s.replace(Some(Scope { game, keys: vec![] })));
    original(game, kind, notify);
    SCOPE.with(|s| s.replace(old));
    if let Some((key, _)) = active {
        if let Ok(mut s) = STATE.lock() {
            if let Some(first) = s.choices.get_mut(&key) {
                *first = false;
            }
        }
    }
}
pub(in crate::runtime) fn card(game: u64, output: u64) {
    if !SCOPE.with(|s| s.borrow().as_ref().is_some_and(|s| s.game == game)) {
        return;
    }
    let Some(r) = RUNTIME.get() else { return };
    if let Ok(Some(key)) = r.adapter.goblin_amulet_key(&r.local, game, output) {
        SCOPE.with(|s| {
            if let Some(s) = s.borrow_mut().as_mut() {
                s.keys.push(key);
            }
        });
    }
}
pub(in crate::runtime) fn accepting(game: u64, index: u64) {
    let Some(r) = RUNTIME.get() else { return };
    let Ok(Some((slot, id))) = r.adapter.goblin_reward_actor(&r.local, game) else {
        return;
    };
    if r.adapter
        .amulet_selected(&r.local, r.local.base as u64, game, index)
        .ok()
        .flatten()
        .is_none()
    {
        return;
    }
    if let Ok(mut s) = STATE.lock() {
        s.choices.remove(&(game, slot, id));
    }
}
pub(super) fn reset() {
    if let Ok(mut s) = STATE.lock() {
        *s = State::default();
    }
}
pub(super) fn queue(game: u64, event: u64, marks: f32, actors: &[(u32, u64)]) {
    if let Ok(mut s) = STATE.lock() {
        for &(slot, guid) in actors {
            s.pending.push_back(Request {
                game,
                event,
                slot,
                guid,
                marks,
                marks_sent: marks == 0.,
            });
        }
    }
}
pub(super) fn finish(r: &Runtime, game: u64, actors: &[(u32, u64)]) -> Result<(), String> {
    if let Ok(mut s) = STATE.lock() {
        s.choices
            .retain(|(g, slot, id), _| *g == game && actors.contains(&(*slot, *id)));
    }
    let pending = STATE
        .lock()
        .map_err(|_| "Goblin reward state")?
        .pending
        .iter()
        .filter(|a| a.game == game)
        .copied()
        .collect::<Vec<_>>();
    for request in pending {
        let present = actors.contains(&(request.slot, request.guid));
        if present && !request.marks_sent {
            let reserve = STATE.lock().ok().is_some_and(|mut s| {
                let Some(a) = s.pending.iter_mut().find(|a| {
                    a.game == game
                        && a.event == request.event
                        && a.slot == request.slot
                        && a.guid == request.guid
                }) else {
                    return false;
                };
                if a.marks_sent {
                    return false;
                }
                a.marks_sent = true;
                true
            });
            if reserve {
                let m = io::Access {
                    local: &r.local,
                    adapter: r.adapter,
                };
                r.adapter.goblin_credit_marks(
                    &m,
                    game,
                    request.slot,
                    request.guid,
                    request.marks,
                )?;
                r.log.info(&format!(
                    "treasure goblin marks: event={} slot={} guid={} amount={}",
                    request.event, request.slot, request.guid, request.marks
                ));
            }
        }
        let ready = present
            && r.adapter
                .goblin_reward_ready(&r.local, game, request.slot, request.guid)?;
        if present && !ready {
            continue;
        }
        {
            let mut s = STATE.lock().map_err(|_| "Goblin reward state")?;
            let Some(i) = s.pending.iter().position(|a| {
                a.game == game
                    && a.event == request.event
                    && a.slot == request.slot
                    && a.guid == request.guid
            }) else {
                continue;
            };
            s.pending.remove(i);
            if ready {
                s.choices.insert((game, request.slot, request.guid), true);
            }
        }
        if !ready {
            continue;
        }
        let m = io::Access {
            local: &r.local,
            adapter: r.adapter,
        };
        let (keys, marks) = unsafe {
            r.adapter.goblin_open_amulets(
                &m,
                r.local.base as u64,
                game,
                request.slot,
                request.guid,
                request.marks,
            )
        }?;
        r.log.info(&format!(
            "treasure goblin amulets: event={} slot={} guid={} count={} marks={marks} keys={keys:?}",
            request.event,
            request.slot,
            request.guid,
            keys.len()
        ));
    }
    Ok(())
}
pub(super) fn install(r: &Runtime) -> Result<(), String> {
    let es = r.adapter.goblin_reward_entries();
    for e in es {
        if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
            return Err("Optional goblin reward boundary changed".into());
        }
    }
    for (e, detour, trampoline, name) in [
        (
            es[0],
            pool_entry as *const () as usize,
            &POOL,
            "goblin-amulet-pool",
        ),
        (
            es[1],
            exclude as *const () as usize,
            &EXCLUDE,
            "goblin-distinct-amulets",
        ),
        (
            es[2],
            choice as *const () as usize,
            &CHOICE,
            "goblin-native-reward",
        ),
    ] {
        crate::install::install_optional(crate::install::HookSpec {
            name,
            target: r.local.base + e.rva as usize,
            detour,
            trampoline,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "system" fn native_choice(g: u64, k: u8, b: bool) {
        assert_eq!((g, k, b), (7, 1, true));
    }
    unsafe extern "system" fn native_exclusion(p: u64, o: u64, e: u64) {
        assert_eq!((p, o, e), (7, 9, 11));
    }
    #[test]
    fn ordinary_native_rewards_keep_their_arguments() {
        let _lock = crate::test_support::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        reset();
        SCOPE.with(|s| s.replace(None));
        CHOICE.store(native_choice as *const () as usize, Ordering::Release);
        EXCLUDE.store(native_exclusion as *const () as usize, Ordering::Release);
        unsafe {
            choice(7, 1, true);
            exclude(7, 9, 11);
        }
    }
}

