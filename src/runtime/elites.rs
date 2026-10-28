//! Native guardian strength/level reward is retained exactly once; add only
//! the missing personal item choice. No cache purchase, price or wallet mutation.
use super::{local_base, Adapter};
use services::Services;
use std::sync::atomic::AtomicUsize;
mod services;
#[derive(Clone)]
struct Reward {
    game: u64,
    players: Vec<(u32, u64)>,
}
static PENDING: std::sync::Mutex<Vec<Reward>> = std::sync::Mutex::new(vec![]);
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(entry, 0, observe, ORIGINAL);
unsafe extern "system" fn observe(c: &mut crate::stub::Context) {
    let Some(r) = services::active() else {
        return;
    };
    let Ok(Some(game)) = r.elite_game([c.rax, c.rcx, c.rdi]) else {
        return;
    };
    let players = r.party_actors();
    if let Ok(mut q) = PENDING.lock() {
        if q.len() < 64 {
            q.push(Reward { game, players });
            r.info("official elite reward: native shared skill retained, personal items pending");
        }
    }
}
pub fn finish(r: &dyn Services, game: u64) {
    let pending = PENDING
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default();
    for mut reward in pending {
        if reward.game != game {
            continue;
        }
        reward.players.retain(|(slot, guid)| {
            match r.deliver_item(game, *slot, *guid) {
                Ok(true) => {
                    r.info(&format!(
                        "official elite item: slot={slot} guid={guid} count=1"
                    ));
                    false
                }
                Ok(false) => true,
                Err(e) => {
                    r.warn(&format!("Official elite item rejected: {e}"));
                    true
                }
            }
        });
        if !reward.players.is_empty() {
            if let Ok(mut q) = PENDING.lock() {
                if q.len() < 64 {
                    q.push(reward);
                }
            }
        }
    }
}
pub fn reset() {
    if let Ok(mut q) = PENDING.lock() {
        q.clear();
    }
}
pub fn hook(adapter: Adapter) -> crate::install::HookSpec {
    crate::install::HookSpec {
        name: "official-elite-item",
        target: local_base() + adapter.elite_entry().rva as usize,
        detour: entry as *const () as usize,
        trampoline: &ORIGINAL,
    }
}

#[cfg(test)]
mod tests;
