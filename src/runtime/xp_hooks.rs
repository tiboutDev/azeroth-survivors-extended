//! Shared-XP capture and award interception, separate from session callbacks.
//! Native translation stays in the selected adapter; deferred award bookkeeping
//! remains in experience. Preserve TLS nesting around the native shared wrapper.
use super::Adapter;
use services::Services;
mod services;
use crate::install::HookSpec;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[cfg(test)]
mod tests;

static SHARED_XP_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
thread_local! { static SHARED_XP_INPUT: std::cell::Cell<Option<(u64,u32)>> = const {std::cell::Cell::new(None)}; }
type SharedXp = unsafe extern "system" fn(u64, u64, u64, u64) -> usize;
unsafe extern "system" fn shared_xp(language: u64, length: u64, request: u64, game: u64) -> usize {
    // Copy normalized integers before the native callback. No native vector or
    // mutable state borrow is retained across its original language wrapper.
    let input = services::active().and_then(|services| capture_input(services, request, game));
    let previous = SHARED_XP_INPUT.with(|value| value.replace(input));
    let original: SharedXp = std::mem::transmute(SHARED_XP_ORIGINAL.load(Ordering::Acquire));
    let result = original(language, length, request, game);
    SHARED_XP_INPUT.with(|value| value.set(previous));
    result
}
static XP_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static XP_SAMPLES: AtomicU64 = AtomicU64::new(0);
crate::context_stub_rw!(xp_entry, 0, xp_award, XP_ORIGINAL);
unsafe extern "system" fn xp_award(context: &mut crate::stub::Context) {
    let Some(services) = services::active() else { return };
    award(services, context);
}

fn capture_input(services: &dyn Services, request: u64, game: u64) -> Option<(u64, u32)> {
    let sum = services.shared_input(request, game).ok()?;
    services.capture_shared(game, sum);
    Some((game, sum))
}

fn award(services: &dyn Services, context: &mut crate::stub::Context) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let Some(multiplier) = services.multiplier() else { return };
        let Some(site) = services.return_site(context.rsp_entry) else { return };
        match services.normalize(
            context.rcx,
            site,
            [context.rbx, context.rsi, context.r12],
            multiplier,
            SHARED_XP_INPUT.with(|value| {
                value
                    .get()
                    .filter(|(game, _)| *game == context.rcx)
                    .map(|(_, sum)| sum)
            }),
        ) {
            Ok(Some(award)) => {
                let previous = context.rdx as u32;
                context.rdx = award.amount.into();
                if XP_SAMPLES.fetch_add(1, Ordering::Relaxed) < 256 {
                    services.info(&format!(
                        "official XP: source={} raw={} native={} shared={} multiplier={multiplier}",
                        award.source, award.raw, previous, award.amount
                    ));
                }
            }
            Ok(None) => {}
            Err(error) => services.warn(&format!("Official XP adaptation rejected: {error}")),
        }
    }));
}

/// Return both entries to bootstrap's existing single installation batch.
pub(super) fn hooks(adapter: Adapter, base: usize) -> [HookSpec; 2] {
    [
        HookSpec {
            name: "official-shared-xp",
            target: base + adapter.shared_xp().rva as usize,
            detour: shared_xp as *const () as usize,
            trampoline: &SHARED_XP_ORIGINAL,
        },
        HookSpec {
            name: "official-award-xp",
            target: base + adapter.award_xp().rva as usize,
            detour: xp_entry as *const () as usize,
            trampoline: &XP_ORIGINAL,
        },
    ]
}
