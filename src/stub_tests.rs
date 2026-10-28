//! Shared native-stub ABI tests retained from the former combined engine suite.
//! They use stand-in functions, not a game or a legacy simulation.
use std::hint::black_box;
use std::sync::atomic::Ordering::Relaxed;
fn begin(name: &str) -> std::time::Instant {
    println!("[ .... ] {name}");
    std::time::Instant::now()
}
fn ok(name: &str, started: std::time::Instant, metric: &str) {
    println!("[  OK  ] {name} ({:.1} ms, {metric})", started.elapsed().as_secs_f64() * 1000.0);
}

// ---- read-write context stub ------------------------------------------------

static RW_TRAMPOLINE_MODIFY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static RW_TRAMPOLINE_SKIP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static RW_ORIGINAL_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[inline(never)]
unsafe extern "system" fn rw_modify_target(a: u64, scale: f64, flag: u64) -> f64 {
    black_box(a) as f64 * black_box(scale) + black_box(flag) as f64
}

#[inline(never)]
unsafe extern "system" fn rw_skip_target(a: u64, b: u64) -> u64 {
    RW_ORIGINAL_CALLS.fetch_add(1, Relaxed);
    black_box(a).wrapping_mul(7).wrapping_add(black_box(b))
}

extern "C" fn rw_modify_handler(ctx: *mut crate::stub::Context) {
    let ctx = unsafe { &mut *ctx };
    assert_eq!(ctx.id, 90);
    ctx.rcx = 5; // a
    ctx.xmm[1] = 2.0f64.to_bits() as u128; // scale
    ctx.r8 = 1; // flag
}

extern "C" fn rw_skip_handler(ctx: *mut crate::stub::Context) {
    let ctx = unsafe { &mut *ctx };
    assert_eq!(ctx.id, 91);
    if ctx.rcx == 1 {
        ctx.skip = 1;
        ctx.result = 0x1234_5678_9ABC;
    }
}

crate::context_stub_rw!(rw_modify_stub, 90u64, rw_modify_handler, RW_TRAMPOLINE_MODIFY);
crate::context_stub_rw!(rw_skip_stub, 91u64, rw_skip_handler, RW_TRAMPOLINE_SKIP);

#[test]
fn read_write_stub_rewrites_arguments() {
    // MinHook freezes the other threads while it patches: tests that install hooks take turns
    let _turn = crate::test_support::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let name = "read-write stub: the handler rewrites rcx, xmm1 and r8 before the original runs";
    let started = begin(name);
    let target = rw_modify_target as *mut std::ffi::c_void;
    let direct = unsafe { rw_modify_target(1, 3.0, 0) };
    assert_eq!(direct, 3.0);
    let original = unsafe { minhook::MinHook::create_hook(target, rw_modify_stub as *mut _).unwrap() };
    RW_TRAMPOLINE_MODIFY.store(original as usize, Relaxed);
    unsafe { minhook::MinHook::enable_hook(target).unwrap() };
    let hooked = unsafe { rw_modify_target(1, 3.0, 0) };
    unsafe { minhook::MinHook::disable_hook(target).unwrap() };
    unsafe { minhook::MinHook::remove_hook(target).unwrap() };
    assert_eq!(hooked, 11.0, "the original saw a=5, scale=2.0, flag=1");
    ok(name, started, "5 * 2.0 + 1 = 11");
}

#[test]
fn read_write_stub_skips_the_original() {
    // MinHook freezes the other threads while it patches: tests that install hooks take turns
    let _turn = crate::test_support::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let name = "read-write stub: skip returns the handler's result without entering the original";
    let started = begin(name);
    let target = rw_skip_target as *mut std::ffi::c_void;
    let original = unsafe { minhook::MinHook::create_hook(target, rw_skip_stub as *mut _).unwrap() };
    RW_TRAMPOLINE_SKIP.store(original as usize, Relaxed);
    unsafe { minhook::MinHook::enable_hook(target).unwrap() };
    RW_ORIGINAL_CALLS.store(0, Relaxed);
    let skipped = unsafe { rw_skip_target(1, 2) };
    assert_eq!(skipped, 0x1234_5678_9ABC, "result from the handler");
    assert_eq!(RW_ORIGINAL_CALLS.load(Relaxed), 0, "the original was not entered");
    let passed = unsafe { rw_skip_target(2, 3) };
    assert_eq!(passed, 17, "without skip the original runs with its own arguments");
    assert_eq!(RW_ORIGINAL_CALLS.load(Relaxed), 1);
    // many alternating calls: the stack stays balanced and the callee-saved state intact
    let mut sum = 0u64;
    for i in 0..10_000u64 {
        sum = sum.wrapping_add(unsafe { rw_skip_target(1 + (i & 1), i) });
    }
    unsafe { minhook::MinHook::disable_hook(target).unwrap() };
    unsafe { minhook::MinHook::remove_hook(target).unwrap() };
    let expected: u64 = (0..10_000u64).fold(0u64, |acc, i| acc.wrapping_add(if i & 1 == 0 { 0x1234_5678_9ABC } else { (2u64 * 7).wrapping_add(i) }));
    assert_eq!(sum, expected);
    ok(name, started, "10 000 alternating calls balanced");
}

static RW_TRAMPOLINE_NV: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static RW_NV_SEEN: [std::sync::atomic::AtomicU64; 8] = [const { std::sync::atomic::AtomicU64::new(0) }; 8];

#[inline(never)]
unsafe extern "system" fn rw_nv_target() -> u64 {
    black_box(7)
}

extern "C" fn rw_nv_handler(ctx: *mut crate::stub::Context) {
    let ctx = unsafe { &*ctx };
    for (slot, value) in RW_NV_SEEN.iter().zip([ctx.rbx, ctx.rbp, ctx.rsi, ctx.rdi, ctx.r12, ctx.r13, ctx.r14, ctx.r15]) {
        slot.store(value, Relaxed);
    }
}

crate::context_stub_rw!(rw_nv_stub, 92u64, rw_nv_handler, RW_TRAMPOLINE_NV);

#[test]
fn read_write_stub_reports_the_callee_saved_registers_of_the_caller() {
    // MinHook freezes the other threads while it patches: tests that install hooks take turns
    let _turn = crate::test_support::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let name = "read-write stub: rbx, rbp, rsi, rdi, r12-r15 are the caller's values and survive the call";
    let started = begin(name);
    let target = rw_nv_target as *mut std::ffi::c_void;
    let original = unsafe { minhook::MinHook::create_hook(target, rw_nv_stub as *mut _).unwrap() };
    RW_TRAMPOLINE_NV.store(original as usize, Relaxed);
    unsafe { minhook::MinHook::enable_hook(target).unwrap() };
    let (result, after): (u64, [u64; 8]);
    let mut out = [0u64; 8];
    unsafe {
        let out_ptr = out.as_mut_ptr();
        let rax: u64;
        std::arch::asm!(
            "push rbx", "push rbp", "push rsi", "push rdi", "push r12", "push r13", "push r14", "push r15",
            "mov rbx, 0xB1", "mov rbp, 0xB2", "mov rsi, 0xB3", "mov rdi, 0xB4", "mov r12, 0xB5", "mov r13, 0xB6", "mov r14, 0xB7", "mov r15, 0xB8",
            "sub rsp, 32",
            "call {t}",
            "add rsp, 32",
            "mov [{o}], rbx", "mov [{o} + 8], rbp", "mov [{o} + 16], rsi", "mov [{o} + 24], rdi",
            "mov [{o} + 32], r12", "mov [{o} + 40], r13", "mov [{o} + 48], r14", "mov [{o} + 56], r15",
            "pop r15", "pop r14", "pop r13", "pop r12", "pop rdi", "pop rsi", "pop rbp", "pop rbx",
            t = in(reg) rw_nv_target as usize,
            o = in(reg) out_ptr,
            out("rax") rax,
            clobber_abi("C"),
        );
        result = rax;
        after = out;
    }
    unsafe { minhook::MinHook::disable_hook(target).unwrap() };
    unsafe { minhook::MinHook::remove_hook(target).unwrap() };
    let seen: Vec<u64> = RW_NV_SEEN.iter().map(|s| s.load(Relaxed)).collect();
    assert_eq!(seen, vec![0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8], "the handler saw the caller's registers");
    assert_eq!(after, [0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8], "and they are unchanged afterwards");
    assert_eq!(result, 7);
    ok(name, started, "8 registers");
}
