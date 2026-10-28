//! Deferred, already-normalized native shared awards. No custom XP multiplier.
use super::{local_base, Adapter};
use asx_gameplay::official::DeferredExperience;
use services::Services;
use std::sync::{atomic::AtomicUsize, LazyLock, Mutex};
#[cfg(test)]
use std::sync::atomic::Ordering;
mod services;

#[cfg(test)]
mod tests;
#[derive(Default)]
struct State {
    game: u64,
    clock: f32,
    bank: DeferredExperience,
}
static STATE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::default()));
static HOST_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
pub fn reset() {
    if let Ok(mut s) = STATE.lock() {
        *s = State::default();
    }
}
pub fn capture(r: &dyn Services, game: u64, amount: u32) {
    if amount == 0 {
        return;
    }
    match r.deferred_actor(game) {
        Ok(Some((slot, guid))) => {
            let result = STATE
                .lock()
                .map_err(|_| "XP bank poisoned".to_owned())
                .and_then(|mut s| {
                    if s.game != game {
                        *s = State {
                            game,
                            ..Default::default()
                        };
                    }
                    s.bank.credit(slot + 1, guid, amount)
                });
            match result {
                Ok(total) => r.info(&format!(
                    "official XP deferred: slot={slot} guid={guid} amount={amount} pending={total}"
                )),
                Err(e) => r.warn(&format!("Official deferred XP rejected: {e}")),
            }
        }
        Ok(None) => {}
        Err(e) => r.warn(&format!("Official deferred XP actor rejected: {e}")),
    }
}
// R13/RSI are callee-saved: only the full-context stub snapshots them.
crate::context_stub_rw!(host_entry, 0, host_shared, HOST_ORIGINAL);
unsafe extern "system" fn host_shared(c: &mut crate::stub::Context) {
    #[cfg(test)]
    {
        tests::GAME.store(c.r13, Ordering::SeqCst);
        tests::AMOUNT.store(c.rsi, Ordering::SeqCst);
    }
    if let Some(services) = services::active() {
        capture(services, c.r13, c.rsi as u32);
    }
}
pub fn hook(adapter: Adapter) -> crate::install::HookSpec {
    crate::install::HookSpec {
        name: "official-downed-xp",
        target: local_base() + adapter.deferred_xp_entry().rva as usize,
        detour: host_entry as *const () as usize,
        trampoline: &HOST_ORIGINAL,
    }
}
pub fn finish(r: &dyn Services, game: u64) {
    let Ok(Some(party)) = r.world_party(game) else {
        return;
    };
    let deliveries = if let Ok(mut s) = STATE.lock() {
        if s.game != game || party.clock + 0.01 < s.clock {
            *s = State {
                game,
                ..Default::default()
            };
        }
        s.clock = party.clock;
        s.bank.retain(
            &party
                .actors
                .iter()
                .map(|a| (a.slot + 1, a.guid))
                .collect::<Vec<_>>(),
        );
        party
            .actors
            .iter()
            .filter(|a| a.phase == asx_gameplay::Phase::Playing)
            .filter_map(|a| {
                let amount = s.bank.take(a.slot + 1, a.guid);
                (amount > 0).then_some((a.slot, a.guid, amount))
            })
            .collect::<Vec<_>>()
    } else {
        return;
    };
    for (slot, guid, amount) in deliveries {
        match r.replay(game, slot, guid, amount) {
            Ok(true) => r.info(&format!(
                "official XP replayed: slot={slot} guid={guid} amount={amount}"
            )),
            Ok(false) => {
                if let Ok(mut s) = STATE.lock() {
                    if let Err(e) = s.bank.credit(slot + 1, guid, amount) {
                        r.warn(&format!("Official deferred XP restore rejected: {e}"));
                    }
                }
            }
            // Never retry after an uncertain native commit/flush; report loudly.
            Err(e) => r.warn(&format!(
                "Official XP replay rejected: slot={slot} amount={amount}: {e}"
            )),
        }
    }
}
