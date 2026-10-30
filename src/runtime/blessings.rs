//! Host-owned event rewards; no per-entity polling or additional network channel.
use super::{local_base, Adapter};
use asx_gameplay::blessings::{Blessing, Blessings};
use services::Services;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
mod services;
#[derive(Default)]
struct State {
    game: u64,
    clock: f32,
    rules: Blessings,
    pending: BTreeSet<u64>,
    next_event: u64,
    rng: u64,
}
static STATE: std::sync::Mutex<Option<State>> = std::sync::Mutex::new(None);
pub static ACTIVE: AtomicBool = AtomicBool::new(false);
pub fn reset() {
    ACTIVE.store(false, Ordering::Release);
    if let Ok(mut s) = STATE.lock() {
        *s = None;
    }
}
pub fn observe(game: u64, p: Option<&asx_game::official::PartyObservation>) {
    let Some(p) = p else {
        reset();
        return;
    };
    if let Ok(mut guard) = STATE.lock() {
        let s = guard.get_or_insert_with(State::default);
        if s.game != game || p.clock < s.clock {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0)
                ^ game;
            *s = State {
                game,
                rng: seed,
                ..Default::default()
            };
        }
        s.clock = p.clock;
        s.rules.retain(
            &p.actors
                .iter()
                .map(|a| (a.slot, a.guid))
                .collect::<Vec<_>>(),
        );
        ACTIVE.store(!s.rules.actors.is_empty(), Ordering::Release);
    }
}
pub fn view(slot: u32, guid: u64) -> Option<Blessing> {
    STATE
        .lock()
        .ok()
        .and_then(|s| s.as_ref()?.rules.actors.get(&(slot, guid)).copied())
}
pub fn personal(slot: u32, guid: u64, kind: u8) -> f32 {
    view(slot, guid)
        .filter(|b| b.category == kind)
        .map_or(0.0, |b| b.factor())
}
pub fn strongest(kind: u8) -> f32 {
    STATE
        .lock()
        .ok()
        .and_then(|s| s.as_ref().map(|s| s.rules.strongest(kind)))
        .unwrap_or(0.0)
}
fn random(seed: &mut u64, n: u32) -> u32 {
    // Rejection sampling keeps category and rarity probabilities exact.
    loop {
        *seed = seed.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = *seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        let max = u64::MAX - u64::MAX % u64::from(n);
        if z < max {
            return (z % u64::from(n)) as u32;
        }
    }
}
pub fn finish(r: &dyn Services, game: u64) {
    let Some(p) = r.extension_party() else {
        return;
    };
    let ids = p
        .actors
        .iter()
        .map(|a| (a.slot, a.guid))
        .collect::<Vec<_>>();
    let changed = if let Ok(mut guard) = STATE.lock() {
        let Some(s) = guard.as_mut().filter(|s| s.game == game) else {
            return;
        };
        let mut changed = false;
        for event in std::mem::take(&mut s.pending) {
            if s.rules
                .complete(event, &ids, |n| random(&mut s.rng, n))
            {
                changed = true;
                #[cfg(feature = "test-hooks")]
                if std::env::var("AZCOOP_TEST_BLESSINGS").as_deref() == Ok("1") {
                    for ((slot, _), b) in &mut s.rules.actors {
                        let category = match b.boss {
                            1 => 1, // Shared XP: strongest only.
                            2 => 2, // Shared gold: strongest only.
                            3 => 4, // Personal speed on both owners.
                            4 => if *slot == 0 {3} else {0}, // Replace speed: pickup / luck.
                            _ => 4, // Same category/rarity at bosses 5 and 6: fixed cap.
                        };
                        *b = Blessing::roll(
                            b.boss,
                            category,
                            if *slot == 0 { 0 } else { 85 },
                        );
                    }
                }
                for (id, b) in &s.rules.actors {
                    r.info(&format!("blessing granted: event={event} slot={} guid={} boss={} category={} rarity={} amount={}",id.0,id.1,b.boss,b.category,b.rarity,b.amount));
                }
            }
        }
        changed
    } else {
        false
    };
    if changed {
        ACTIVE.store(true, Ordering::Release);
        if let Err(e) = r.refresh_blessings(game, &ids) {
            r.warn(&format!("Blessing refresh rejected: {e}"));
        }
        r.info(&format!(
            "blessing group: xp={} gold={}",
            strongest(1),
            strongest(2)
        ));
    }
}
static POOL: AtomicUsize = AtomicUsize::new(0);

crate::context_stub_rw!(pool_entry, 0, pool, POOL, write_r14);
pub(super) fn encounter(game:u64){
    if super::rituals::replaces_boss_fortunes(game){return}
    if let Ok(mut guard)=STATE.lock(){if let Some(s)=guard.as_mut().filter(|s|s.game==game){if s.pending.len()<256{s.next_event+=1;s.pending.insert(s.next_event);}}}
}
pub(super) fn ritual_rewards(r:&dyn Services,game:u64,rewards:&[((u32,u64),Blessing)])->Result<(),String>{
    {let mut guard=STATE.lock().map_err(|_|"Fortune state unavailable")?;let s=guard.as_mut().filter(|s|s.game==game).ok_or("Fortune run changed")?;for(id,b)in rewards{s.rules.actors.insert(*id,*b);}}
    ACTIVE.store(true,Ordering::Release);r.refresh_blessings(game,&rewards.iter().map(|(id,_)|*id).collect::<Vec<_>>())?;Ok(())
}
pub(super) fn ritual_reward(r:&dyn Services,game:u64,owner:u64,b:Blessing)->Result<(),String>{
    {let mut guard=STATE.lock().map_err(|_|"Fortune state unavailable")?;let s=guard.as_mut().filter(|s|s.game==game).ok_or("Fortune run changed")?;s.rules.actors.insert((0,owner),b);}
    ACTIVE.store(true,Ordering::Release);r.refresh_blessings(game,&[(0,owner)])?;
    r.info(&format!("ritual fortune: owner={owner} stage={} category={} amount={}",b.boss,b.category,b.amount));Ok(())
}
unsafe extern "system" fn pool(c: &mut crate::stub::Context) {
    #[cfg(test)]
    {
        tests::INDEX.store(c.r14, Ordering::SeqCst);
        c.r14 = 19 * 72;
    }
    let Some(r) = services::active() else {
        return;
    };
    if let Ok(index) = r.pool_index(c.rbp, c.r14) {
        c.r14 = index;
    }
}
pub fn apply_stats(game:u64){
    let Some(r) = services::active() else {
        return;
    };
    let b = r.run_source(game).ok().flatten()
        .and_then(|(slot, guid)| {
            STATE.lock().ok().and_then(|s| {
                s.as_ref()
                    .filter(|s| s.game == game)?
                    .rules
                    .actors
                    .get(&(slot, guid))
                    .copied()
            })
        });
    #[cfg(feature = "test-hooks")]
    let baseline = if b.is_some() && std::env::var("AZCOOP_TEST_BLESSINGS").as_deref() == Ok("1") {
        r.blessing_stat_values(game).ok()
    } else {
        None
    };
    if b.is_some() {
        if let Err(e) = r.apply_blessing_stats(game, b) {
            r.warn(&format!("Blessing stats rejected: {e}"));
        }
    }
    #[cfg(feature = "test-hooks")]
    if let (Some(b), Some(base)) = (b, baseline) {
        static SEEN: std::sync::Mutex<BTreeSet<(u32, u32)>> =
            std::sync::Mutex::new(BTreeSet::new());
        if let Ok(Some((slot, _))) = r.run_source(game) {
            if SEEN
                .lock()
                .ok()
                .is_some_and(|mut s| s.insert((slot, b.boss)))
            {
                r.info(&format!("blessing fixture stats: slot={slot} boss={} category={} base={base:?} actual={:?}",b.boss,b.category,r.blessing_stat_values(game)));
            }
        }
    }
}
pub fn hooks(adapter:Adapter)->Vec<crate::install::HookSpec>{
 let e=adapter.blessing_entries()[1];vec![crate::install::HookSpec{name:"official-blessing-pool",target:local_base()+e.rva as usize,detour:pool_entry as *const ()as usize,trampoline:&POOL}]
}

#[cfg(test)]
pub(super) mod tests;
