//! Keep the event out of the native horde's distant-enemy recycling list.
use super::super::{io, Runtime, RUNTIME};
use std::sync::atomic::{AtomicUsize, Ordering};
static FINAL_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RETURN: AtomicUsize = AtomicUsize::new(0);
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
unsafe extern "system" fn recycle(output: u64, iterator: u64) -> u64 {
    type Original = unsafe extern "system" fn(u64, u64) -> u64;
    let original: Original = std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
    let result = original(output, iterator);
    if let (Some(r), Some((game, guid))) = (RUNTIME.get(), super::identity()) {
        let memory = io::Access {
            local: &r.local,
            adapter: r.adapter,
        };
        match r
            .adapter
            .goblin_filter_recycle(&memory, iterator, output, game, guid)
        {
            #[cfg(feature = "test-hooks")]
            Ok(true) => {
                static LAST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                if LAST.swap(guid, Ordering::Relaxed) != guid {
                    r.log.info(&format!(
                        "treasure goblin native recycle excluded: guid={guid}"
                    ));
                }
            }
            Ok(_) => {}
            Err(e) => r.log.warn(&format!("Goblin recycle exclusion failed: {e}")),
        }
    }
    result
}
pub(super) fn install(r: &Runtime) -> Result<(), String> {
    let e = r.adapter.goblin_recycle_entry();
    if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
        return Err("Optional native recycling boundary changed".into());
    }
    crate::install::install_optional(crate::install::HookSpec {
        name: "goblin-native-recycle-filter",
        target: r.local.base + e.rva as usize,
        detour: recycle as *const () as usize,
        trampoline: &ORIGINAL,
    })?;
    let e = r.adapter.goblin_final_motion_entry();
    if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
        return Err("Optional native final motion boundary changed".into());
    }
    RETURN.store(return_to_entry as *const () as usize, Ordering::Release);
    crate::install::install_optional(crate::install::HookSpec {
        name: "goblin-final-flee-direction",
        target: r.local.base + e.rva as usize,
        detour: final_entry as *const () as usize,
        trampoline: &FINAL_ORIGINAL,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "system" fn native(out: u64, it: u64) -> u64 {
        assert_eq!((out, it), (7, 9));
        81
    }
    #[test]
    fn recycle_wrapper_preserves_the_native_result_and_arguments() {
        let _lock = crate::test_support::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        ORIGINAL.store(native as *const () as usize, Ordering::Release);
        assert_eq!(unsafe { recycle(7, 9) }, 81);
    }
}

// Save the native nonvolatile candidate registers separately. The context stub
// preserves flags and volatile arguments; only the scoped candidate may change.
#[unsafe(naked)]
unsafe extern "C" fn final_entry() {
    core::arch::naked_asm!(
        "lea rsp, [rsp-72]", "movups [rsp+32], xmm13", "movups [rsp+48], xmm14",
        "call {guard}", "movups xmm13, [rsp+32]", "movups xmm14, [rsp+48]",
        "lea rsp, [rsp+72]", "jmp qword ptr [rip+{original}]",
        guard=sym final_guard_entry, original=sym FINAL_ORIGINAL);
}
#[unsafe(naked)]
unsafe extern "C" fn return_to_entry() {
    core::arch::naked_asm!("ret");
}
crate::context_stub_rw!(final_guard_entry, 0, final_guard, RETURN);
unsafe extern "system" fn final_guard(c: &mut crate::stub::Context) {
    let Some(r) = RUNTIME.get() else { return };
    let Some((game, guid)) = super::identity() else {
        return;
    };
    let memory = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    let saved = c.rsp_entry + 40;
    if let Ok(Some((origin, candidate))) = r
        .adapter
        .goblin_final_motion(&memory, c.rbp, saved, game, guid)
    {
        if !super::safe_motion(game, guid, origin, candidate) {
            if let Err(e) = r.adapter.goblin_hold_motion(&memory, saved, origin) {
                r.log.warn(&format!("Goblin final motion rejected: {e}"));
            }
        }
    }
}

#[cfg(test)]
mod final_tests {
    use super::*;
    #[unsafe(naked)]
    unsafe extern "C" fn native_candidate() {
        core::arch::naked_asm!(
            "movups [rcx],xmm13",
            "movups [rcx+16],xmm14",
            "mov rax,rdx",
            "ret"
        );
    }
    #[unsafe(naked)]
    unsafe extern "system" fn invoke(_out: *mut u128, _input: *const u128) -> u64 {
        core::arch::naked_asm!(
            "push rbp","sub rsp,64","movups [rsp+32],xmm13","movups [rsp+48],xmm14",
            "movups xmm13,[rdx]","movups xmm14,[rdx+16]","mov edx,77","call {entry}",
            "movups xmm13,[rsp+32]","movups xmm14,[rsp+48]","add rsp,64","pop rbp","ret",entry=sym final_entry);
    }
    #[test]
    fn final_motion_wrapper_forwards_both_full_nonvolatile_candidates_and_arguments() {
        let _lock = crate::test_support::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        RETURN.store(return_to_entry as *const () as usize, Ordering::Release);
        FINAL_ORIGINAL.store(native_candidate as *const () as usize, Ordering::Release);
        let input = [0x123456789abcdef0_u128, 0xfedcba9876543210_u128];
        let mut out = [0u128; 2];
        assert_eq!(unsafe { invoke(out.as_mut_ptr(), input.as_ptr()) }, 77);
        assert_eq!(out, input);
    }
}
