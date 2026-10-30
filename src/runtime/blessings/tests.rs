use super::*;
use super::super::encounters::{EVENT,test_entry as event_entry};
use super::super::run_stats;
use crate::test_support::TEST_LOCK;
use std::sync::atomic::{AtomicU64, Ordering};

pub static FRAME: AtomicU64 = AtomicU64::new(0);
pub static DEF: AtomicU64 = AtomicU64::new(0);
pub static INDEX: AtomicU64 = AtomicU64::new(0);
static GAME: AtomicU64 = AtomicU64::new(0);

#[unsafe(naked)]
unsafe extern "C" fn continuation() { core::arch::naked_asm!("ret"); }

#[unsafe(naked)]
unsafe extern "system" fn event_probe(_frame: u64, _def: u64) -> u64 {
    core::arch::naked_asm!(
        "push rbp", "push rbx", "sub rsp,40", "mov rbp,rcx", "mov rbx,rdx",
        "call {entry}", "mov rax,rbp", "xor rax,rbx", "add rsp,40", "pop rbx", "pop rbp", "ret",
        entry = sym event_entry
    );
}

#[unsafe(naked)]
unsafe extern "system" fn pool_probe(_index: u64) -> u64 {
    core::arch::naked_asm!(
        "push r14", "sub rsp,32", "mov r14,rcx", "call {entry}", "mov rax,r14",
        "add rsp,32", "pop r14", "ret", entry = sym pool_entry
    );
}

unsafe extern "system" fn native_stats(game: u64) { GAME.store(game, Ordering::SeqCst); }

#[test]
fn native_encounter_pool_and_stat_abi() {
    let _guard = TEST_LOCK.lock().unwrap();
    EVENT.store(continuation as *const () as usize, Ordering::SeqCst);
    POOL.store(continuation as *const () as usize, Ordering::SeqCst);
    run_stats::ORIGINAL.store(native_stats as *const () as usize, Ordering::SeqCst);
    let frame = 0x1122334455667788;
    let def = 0x8877665544332211;
    assert_eq!(unsafe { event_probe(frame, def) }, frame ^ def);
    assert_eq!(FRAME.load(Ordering::SeqCst), frame);
    assert_eq!(DEF.load(Ordering::SeqCst), def);
    assert_eq!(unsafe { pool_probe(15 * 72) }, 19 * 72);
    assert_eq!(INDEX.load(Ordering::SeqCst), 15 * 72);
    unsafe { run_stats::stats(frame) };
    assert_eq!(GAME.load(Ordering::SeqCst), frame);
}
