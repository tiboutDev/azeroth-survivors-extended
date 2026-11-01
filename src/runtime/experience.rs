//! Let native shared XP reach dead coop actors immediately. No bank or replay.
use super::{local_base, Adapter};
use services::Services;
use std::sync::atomic::{AtomicUsize, Ordering};
mod services;
#[cfg(test)] mod tests;
static HOST_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static HOST_NEXT: AtomicUsize = AtomicUsize::new(0);
static REMOTE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static REMOTE_NEXT: AtomicUsize = AtomicUsize::new(0);
static PENDING_HOST_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PENDING_HOST_NEXT: AtomicUsize = AtomicUsize::new(0);
static PENDING_REMOTE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PENDING_REMOTE_NEXT: AtomicUsize = AtomicUsize::new(0);
static PENDING_COMMAND_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PENDING_COMMAND_NEXT: AtomicUsize = AtomicUsize::new(0);
static PRODUCER_HOST_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PRODUCER_HOST_NEXT: AtomicUsize = AtomicUsize::new(0);
static PRODUCER_REMOTE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PRODUCER_REMOTE_NEXT: AtomicUsize = AtomicUsize::new(0);
static SAMPLES: AtomicUsize = AtomicUsize::new(0);
// Native host: R13=Game; native borrowed remote: R9=Game; both RSI=raw XP.
// Full stubs preserve volatile/nonvolatile registers, SIMD and entry flags.
crate::context_stub_flags!(host_entry, host_shared, HOST_NEXT);
crate::context_stub_flags!(remote_entry, remote_shared, REMOTE_NEXT);
crate::context_stub_flags!(pending_host_entry, pending_host, PENDING_HOST_NEXT);
crate::context_stub_flags!(pending_remote_entry, pending_remote, PENDING_REMOTE_NEXT);
crate::context_stub_flags!(pending_command_entry, pending_command, PENDING_COMMAND_NEXT);
crate::context_stub_flags!(producer_host_entry, producer_host, PRODUCER_HOST_NEXT);
crate::context_stub_flags!(producer_remote_entry, producer_remote, PRODUCER_REMOTE_NEXT);
fn target(r: &dyn Services, kind: usize, c: &crate::stub::Context, fallback: usize) -> usize {
    match r.continuation(kind, [c.r13, c.r9, c.rsi, c.rbx, c.rdi, c.rbp]) {
        Ok(Some(at)) => {
            if SAMPLES.fetch_add(1, Ordering::Relaxed) < 128 {
                r.info(&format!("official downed XP: path={kind} admitted=true"));
            }
            at
        },
        Ok(None) => fallback,
        Err(error) => { r.warn(&format!("Official downed XP guard rejected: {error}")); fallback }
    }
}
fn select(kind: usize, c: &crate::stub::Context, original: &AtomicUsize, next: &AtomicUsize) {
    let fallback = original.load(Ordering::Acquire);
    // The continuation is selected only after bounded reads, without native
    // calls, retained borrowed containers, phase changes or HP writes.
    let at = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        services::active().map_or(fallback, |r| target(r, kind, c, fallback))
    })).unwrap_or(fallback);
    #[cfg(test)] let at = tests::override_target(at, kind, c);
    next.store(at, Ordering::Release);
}
unsafe extern "system" fn host_shared(c: &mut crate::stub::Context) { select(0, c, &HOST_ORIGINAL, &HOST_NEXT); }
unsafe extern "system" fn remote_shared(c: &mut crate::stub::Context) { select(1, c, &REMOTE_ORIGINAL, &REMOTE_NEXT); }
unsafe extern "system" fn pending_host(c: &mut crate::stub::Context) { select(2, c, &PENDING_HOST_ORIGINAL, &PENDING_HOST_NEXT); }
unsafe extern "system" fn pending_remote(c: &mut crate::stub::Context) { select(3, c, &PENDING_REMOTE_ORIGINAL, &PENDING_REMOTE_NEXT); }
unsafe extern "system" fn pending_command(c: &mut crate::stub::Context) { select(4, c, &PENDING_COMMAND_ORIGINAL, &PENDING_COMMAND_NEXT); }
unsafe extern "system" fn producer_host(c: &mut crate::stub::Context) { select(5, c, &PRODUCER_HOST_ORIGINAL, &PRODUCER_HOST_NEXT); }
unsafe extern "system" fn producer_remote(c: &mut crate::stub::Context) { select(6, c, &PRODUCER_REMOTE_ORIGINAL, &PRODUCER_REMOTE_NEXT); }
pub fn hooks(adapter: Adapter) -> [crate::install::HookSpec; 7] {
    let entries = adapter.downed_xp_entries();
    let specs=[("official-host-downed-xp",host_entry as *const () as usize,&HOST_ORIGINAL),
        ("official-remote-downed-xp",remote_entry as *const () as usize,&REMOTE_ORIGINAL),
        ("official-host-downed-level",pending_host_entry as *const () as usize,&PENDING_HOST_ORIGINAL),
        ("official-remote-downed-level",pending_remote_entry as *const () as usize,&PENDING_REMOTE_ORIGINAL),
        ("official-command-downed-level",pending_command_entry as *const () as usize,&PENDING_COMMAND_ORIGINAL),
        ("official-host-downed-level-receipt",producer_host_entry as *const () as usize,&PRODUCER_HOST_ORIGINAL),
        ("official-remote-downed-level-receipt",producer_remote_entry as *const () as usize,&PRODUCER_REMOTE_ORIGINAL)];
    std::array::from_fn(|i|crate::install::HookSpec{name:specs[i].0,target:local_base()+entries[2*i].rva as usize,detour:specs[i].1,trampoline:specs[i].2})
}
