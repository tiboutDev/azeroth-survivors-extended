//! Route only the selected NPC appearance through the native armor compositor.
use super::super::{io, Runtime, RUNTIME};
use std::{
    cell::Cell,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
static METADATA: AtomicUsize = AtomicUsize::new(0);
static KEY: AtomicUsize = AtomicUsize::new(0);
thread_local! {static PENDING:Cell<[Option<(u64,u64)>;4]>=const{Cell::new([None;4])};}
crate::context_stub_rw!(metadata_entry, 0, metadata, METADATA);
crate::context_stub_rw!(key_entry, 0, key, KEY);
unsafe extern "system" fn metadata(c: &mut crate::stub::Context) {
    let Some(r) = RUNTIME.get() else { return };
    if PENDING.with(|p| p.get().iter().all(Option::is_some)) {
        return;
    }
    let memory = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    match r.adapter.goblin_outfit_metadata(&memory, c.rbp) {
        Ok(Some(entity)) => {
            PENDING.with(|p| {
                let mut rows = p.get();
                if let Some(i) = rows.iter().position(Option::is_none) {
                    rows[i] = Some((c.rbp, entity));
                    p.set(rows);
                }
            });
        }
        Ok(None) => {}
        Err(e) => warn(r, &e),
    }
}
unsafe extern "system" fn key(c: &mut crate::stub::Context) {
    let entity = PENDING.with(|p| {
        let mut rows = p.get();
        let i = rows.iter().rposition(|v| v.is_some_and(|v| v.0 == c.rbp))?;
        let value = rows[i].take()?;
        p.set(rows);
        Some(value.1)
    });
    let Some(entity) = entity else { return };
    if let Some(r) = RUNTIME.get() {
        let memory = io::Access {
            local: &r.local,
            adapter: r.adapter,
        };
        if let Err(e) = r.adapter.goblin_outfit_key(&memory, c.rbp, entity) {
            warn(r, &e);
            return;
        }
        #[cfg(feature = "test-hooks")]
        {
            static ONCE: AtomicBool = AtomicBool::new(false);
            if !ONCE.swap(true, Ordering::Relaxed) {
                r.log
                    .info("treasure goblin equipment: native armor compositor selected");
            }
        }
    }
    c.rax &= !0xff; // Native bool: an actual armor key is available.
}
fn warn(r: &Runtime, e: &str) {
    static ONCE: AtomicBool = AtomicBool::new(false);
    if !ONCE.swap(true, Ordering::Relaxed) {
        r.log.warn(&format!("Goblin equipment unavailable: {e}"));
    }
}
pub(super) fn install(r: &Runtime) -> Result<(), String> {
    let entries = r.adapter.goblin_outfit_entries();
    for e in entries {
        if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
            return Err(format!("Optional NPC armor boundary changed: {:#x}", e.rva));
        }
    }
    // Consumer is installed before the producer can reserve an appearance frame.
    for (i, detour, original, name) in [
        (1, key_entry as *const () as usize, &KEY, "goblin-armor-key"),
        (
            0,
            metadata_entry as *const () as usize,
            &METADATA,
            "goblin-npc-armor-path",
        ),
    ] {
        crate::install::install_optional(crate::install::HookSpec {
            name,
            target: r.local.base + entries[i].rva as usize,
            detour,
            trampoline: original,
        })?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[unsafe(naked)]
    unsafe extern "C" fn target() {
        core::arch::naked_asm!("ret");
    }
    #[unsafe(naked)]
    unsafe extern "system" fn invoke(_frame: u64, _rax: u64) -> u64 {
        core::arch::naked_asm!("push rbp","sub rsp,32","mov rbp,rcx","mov rax,rdx","call {stub}","add rsp,32","pop rbp","ret",stub=sym key_entry);
    }
    #[test]
    fn armor_key_stub_changes_only_the_expected_native_boolean_for_one_owned_frame() {
        let _lock = crate::test_support::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        KEY.store(target as *const () as usize, Ordering::Release);
        PENDING.with(|p| p.set([Some((7, 9)), None, None, None]));
        assert_eq!(unsafe { invoke(8, 0x12ff) }, 0x12ff);
        assert_eq!(unsafe { invoke(7, 0x12ff) }, 0x1200);
        assert_eq!(unsafe { invoke(7, 0x12ff) }, 0x12ff);
    }
}
