//! Deliver a native corpse snapshot to the owner of an active eight-piece DoT.
use super::super::{Adapter, RUNTIME, Runtime, io, local_base};
use crate::install::HookSpec;
use azsx_game::official::SetDeath;
use azsx_gameplay::Phase;
use std::{
    cell::Cell,
    collections::{BTreeMap, VecDeque},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Clone, Copy)]
struct Filter {
    game: u64,
    event: u64,
    mask: u16,
    only: bool,
}
thread_local! { static FILTER: Cell<Option<Filter>> = const { Cell::new(None) }; }
fn filtered(f: Filter, run: impl FnOnce()) {
    let old = FILTER.with(|v| v.replace(Some(f)));
    run();
    FILTER.with(|v| v.set(old));
}
struct Pending {
    game: u64,
    slot: u32,
    guid: u64,
    death: SetDeath,
}
static DEATHS: Mutex<VecDeque<Pending>> = Mutex::new(VecDeque::new());
#[derive(Default, Clone, Copy)]
struct Vulnerability {
    ends: [f32; 2],
    published: [f32; 2],
}
impl Vulnerability {
    fn strongest(self, clock: f32) -> [f32; 2] {
        if self.ends[1] > clock {
            [0.30, self.ends[1]]
        } else if self.ends[0] > clock {
            [0.20, self.ends[0]]
        } else {
            [0., 0.]
        }
    }
}
static VULN: Mutex<BTreeMap<(u64, u64), Vulnerability>> = Mutex::new(BTreeMap::new());
static PROC: AtomicUsize = AtomicUsize::new(0);
static CLONE: AtomicUsize = AtomicUsize::new(0);
static WRITE: AtomicUsize = AtomicUsize::new(0);
static DOT: AtomicUsize = AtomicUsize::new(0);
type Proc = unsafe extern "system" fn(u64, u64, u64);
unsafe extern "system" fn proc(game: u64, event: u64, corpse: u64) {
    let original: Proc = std::mem::transmute(PROC.load(Ordering::Acquire));
    if corpse == 0 {
        return super::super::support::with_necklace_event(game,event,||original(game,event,corpse));
    }
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return original(game, event, corpse);
    };
    let Some(party) = r.party.lock().ok().and_then(|p| p.clone()) else {
        return original(game, event, corpse);
    };
    let Ok(Some((killer, guid))) = r.adapter.effect_source(&r.local, game) else {
        return original(game, event, corpse);
    };
    if !party
        .actors
        .iter()
        .any(|a| a.slot == killer && a.guid == guid && a.phase == Phase::Playing)
    {
        return original(game, event, corpse);
    }
    let Ok((mask, groups)) = r.adapter.set_deaths(&r.local, event, corpse, killer) else {
        return original(game, event, corpse);
    };
    if mask == 0 {
        return original(game, event, corpse);
    }
    let work = groups.into_iter().filter_map(|(slot, death)| {
        let actor = party
            .actors
            .iter()
            .find(|a| a.slot == slot && a.phase == Phase::Playing)?;
        Some(Pending {
            game,
            slot,
            guid: actor.guid,
            death,
        })
    });
    let accepted = DEATHS.lock().is_ok_and(|mut queue| {
        if queue.len() + party.actors.len() > 512 {
            return false;
        }
        queue.extend(work);
        true
    });
    if !accepted {
        return original(game, event, corpse);
    }
    filtered(
        Filter {
            game,
            event,
            mask,
            only: false,
        },
        || original(game, event, corpse),
    );
}
crate::context_stub_rw!(clone_entry, 0, filter_clone, CLONE);
unsafe extern "system" fn filter_clone(c: &mut crate::stub::Context) {
    let Some(f) = FILTER.with(Cell::get) else { return };
    let Some(r) = RUNTIME.get() else { return };
    let Ok((game, event, pointer, count)) =
        r.adapter.set_clone_context(&r.local, c.rbp, [c.r15, c.r13, c.r14, c.rbx])
    else { return };
    if (f.game, f.event) != (game, event) { return; }
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    if let Err(e) = r
        .adapter
        .set_filter_clone(&io, pointer, count, f.mask, f.only)
    {
        r.log.warn(&format!("Native set perk clone rejected: {e}"));
    }
}
crate::context_stub_rw!(dot_entry, 0, spread_owner, DOT);
unsafe extern "system" fn spread_owner(c: &mut crate::stub::Context) {
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    if let Err(e) = r.adapter.set_stamp_dot(
        &io,
        c.rsp_entry,
        r.local.base as u64,
        [c.rax, c.rcx, c.rdx, c.rdi],
    ) {
        r.log.warn(&format!("Native set DoT owner rejected: {e}"));
    }
}
crate::context_stub_rw!(write_entry, 0, vulnerability, WRITE);
unsafe extern "system" fn vulnerability(c: &mut crate::stub::Context) {
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let Ok((game, enemy, spell, now, values)) = r.adapter.set_vulnerability(&r.local, c.rbp, c.rcx)
    else {
        return;
    };
    if !now.is_finite()
        || r.adapter
            .effect_source(&r.local, game)
            .ok()
            .flatten()
            .is_none()
    {
        return;
    }
    let next = {
        let Ok(mut all) = VULN.lock() else { return };
        if !matches!(spell, 12294 | 8042) || !matches!(values[0], 0.20 | 0.30) {
            all.remove(&(game, enemy));
            return;
        }
        let v = all.entry((game, enemy)).or_default();
        let tier = usize::from(values[0] > 0.20);
        v.ends[tier] = v.ends[tier].max(values[1]);
        v.published = v.strongest(now);
        v.published
    };
    if next != values {
        let io = io::Access {
            local: &r.local,
            adapter: r.adapter,
        };
        let _ = r
            .adapter
            .set_vulnerability_write(&io, game, enemy, values, next);
    }
}
pub(super) fn before_damage(r: &Runtime, game: u64, enemy: u64) {
    let state = VULN
        .lock()
        .ok()
        .and_then(|m| m.get(&(game, enemy)).copied());
    let Some(state) = state else { return };
    let Ok(now) = r.adapter.set_clock(&r.local, game) else {
        return;
    };
    let next = state.strongest(now);
    if next == state.published || next[0] == 0. {
        return;
    }
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    if r.adapter
        .set_vulnerability_write(&io, game, enemy, state.published, next)
        == Ok(true)
    {
        if let Ok(mut map) = VULN.lock() {
            if let Some(v) = map.get_mut(&(game, enemy)) {
                v.published = next;
            }
        }
    }
}
pub(super) fn reset() {
    if let Ok(mut q) = DEATHS.lock() {
        q.clear();
    }
    if let Ok(mut q) = VULN.lock() {
        q.clear();
    }
}
pub(in crate::runtime) fn finish(r: &Runtime, game: u64) {
    let Some(party) = r.party.lock().ok().and_then(|p| p.clone()) else {
        return;
    };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    let count = DEATHS.lock().map(|q| q.len()).unwrap_or(0);
    for _ in 0..count {
        let Some(work) = DEATHS.lock().ok().and_then(|mut q| q.pop_front()) else {
            break;
        };
        if work.game != game
            || !party
                .actors
                .iter()
                .any(|a| a.slot == work.slot && a.guid == work.guid && a.phase == Phase::Playing)
        {
            continue;
        }
        let result = r.adapter.set_actor(&io, game, work.slot, work.guid, || {
            let original: Proc = unsafe { std::mem::transmute(PROC.load(Ordering::Acquire)) };
            work.death.invoke(|event, corpse| {
                filtered(
                    Filter {
                        game,
                        event,
                        mask: work.death.mask,
                        only: true,
                    },
                    || unsafe { original(game, event, corpse) },
                )
            });
            Ok(())
        });
        if let Err(e) = result {
            r.log
                .warn(&format!("Native set death dispatch rejected: {e}"));
        }
    }
    let mut updates = Vec::new();
    if let Ok(mut records) = VULN.lock() {
        records.retain(|&(g, enemy), v| {
            if g != game {
                return false;
            }
            let next = v.strongest(party.clock);
            if next[0] == 0. {
                return false;
            }
            if next != v.published {
                updates.push((enemy, v.published, next));
                v.published = next;
            }
            true
        });
    }
    for (enemy, last, next) in updates {
        let _ = r
            .adapter
            .set_vulnerability_write(&io, game, enemy, last, next);
    }
}
pub(super) fn hooks(adapter: Adapter) -> Vec<HookSpec> {
    let [clone, write, dot] = adapter.set_entries();
    [
        (
            adapter.support_proc_entries()[1],
            proc as *const (),
            "official-set-death",
            &PROC,
        ),
        (
            clone,
            clone_entry as *const (),
            "official-set-perk-clone",
            &CLONE,
        ),
        (
            write,
            write_entry as *const (),
            "official-set-vulnerability",
            &WRITE,
        ),
        (
            dot,
            dot_entry as *const (),
            "official-set-spread-owner",
            &DOT,
        ),
    ]
    .into_iter()
    .map(|(e, handler, name, trampoline)| HookSpec {
        name,
        target: local_base() + e.rva as usize,
        detour: handler as usize,
        trampoline,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    static OBSERVED: Mutex<Option<[u64; 3]>> = Mutex::new(None);
    unsafe extern "system" fn native(a: u64, b: u64, c: u64) {
        *OBSERVED.lock().unwrap() = Some([a, b, c]);
    }
    #[test]
    fn new_seams_forward_original_abi_when_inactive() {
        let _guard = crate::test_support::TEST_LOCK.lock().unwrap();
        for (slot, entry) in [
            (&PROC, proc as *const ()),
            (&CLONE, clone_entry as *const ()),
            (&WRITE, write_entry as *const ()),
            (&DOT, dot_entry as *const ()),
        ] {
            let old = slot.swap(native as *const () as usize, Ordering::SeqCst);
            let call: Proc = unsafe { std::mem::transmute(entry) };
            unsafe { call(0xabc1, 0xabc2, 0xabc3) };
            slot.store(old, Ordering::SeqCst);
            assert_eq!(
                OBSERVED.lock().unwrap().take(),
                Some([0xabc1, 0xabc2, 0xabc3])
            );
        }
    }
    #[test]
    fn overlapping_tiers_return_to_the_weaker_active_value() {
        let state = Vulnerability {
            ends: [17., 15.],
            published: [0.30, 15.],
        };
        assert_eq!(state.strongest(14.), [0.30, 15.]);
        assert_eq!(state.strongest(15.), [0.20, 17.]);
        assert_eq!(state.strongest(17.), [0., 0.]);
    }
}
