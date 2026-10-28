//! Native stand-ins exercise the production entries without installing a game.
use super::*;
use crate::test_support::TEST_LOCK;

thread_local! {
    static SEEN: std::cell::RefCell<Vec<[u64; 4]>> = const { std::cell::RefCell::new(Vec::new()) };
}

unsafe extern "system" fn shared_original(language: u64, length: u64, request: u64, game: u64) -> usize {
    SEEN.with(|seen| seen.borrow_mut().push([language, length, request, game]));
    // With no active runtime the production entry must suppress a stale input
    // during the original call, then restore its caller's thread-local value.
    assert_eq!(SHARED_XP_INPUT.with(|input| input.get()), None);
    if language == 11 {
        SHARED_XP_INPUT.with(|input| input.set(Some((game, 37))));
        let nested = shared_xp(21, 22, 23, 24);
        assert_eq!(nested, 90);
        assert_eq!(SHARED_XP_INPUT.with(|input| input.get()), Some((game, 37)));
    }
    (language + length + request + game) as usize
}

#[test]
fn shared_entry_preserves_arguments_return_and_nested_tls() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert!(services::active().is_none());
    SEEN.with(|seen| seen.borrow_mut().clear());
    let prior_input = SHARED_XP_INPUT.with(|input| input.replace(Some((100, 55))));
    let prior_original = SHARED_XP_ORIGINAL.swap(shared_original as *const () as usize, Ordering::AcqRel);
    let result = unsafe { shared_xp(11, 12, 13, 14) };
    let restored = SHARED_XP_INPUT.with(|input| input.replace(prior_input));
    SHARED_XP_ORIGINAL.store(prior_original, Ordering::Release);
    assert_eq!(result, 50);
    assert_eq!(restored, Some((100, 55)));
    SEEN.with(|seen| assert_eq!(*seen.borrow(), [[11, 12, 13, 14], [21, 22, 23, 24]]));
}

unsafe extern "system" fn award_original(a: u64, b: u64, c: u64, d: u64) -> u64 {
    a + 10 * b + 100 * c + 1000 * d
}

#[test]
fn moved_award_stub_preserves_native_fallback_call() {
    let _guard = TEST_LOCK.lock().unwrap();
    assert!(services::active().is_none());
    let prior = XP_ORIGINAL.swap(award_original as *const () as usize, Ordering::AcqRel);
    let call: unsafe extern "system" fn(u64, u64, u64, u64) -> u64 =
        unsafe { std::mem::transmute(xp_entry as *const ()) };
    let result = unsafe { call(1, 2, 3, 4) };
    XP_ORIGINAL.store(prior, Ordering::Release);
    assert_eq!(result, 4321);
}

use asx_game::official::ExperienceAward;
use std::cell::{Cell, RefCell};

#[derive(Default)]
struct Probe {
    mode: Cell<u8>,
    seen: RefCell<Vec<(u64, u64, [u64; 3], f32, Option<u32>)>>,
    captures: RefCell<Vec<(u64, u32, Option<(u64, u32)>)>>,
    warnings: Cell<usize>,
}
impl Services for Probe {
    fn shared_input(&self, request: u64, game: u64) -> Result<u32, String> {
        assert_eq!((request, game), (17, 42));
        if self.mode.get() == 1 { Err("invalid request".into()) } else { Ok(73) }
    }
    fn capture_shared(&self, game: u64, sum: u32) {
        self.captures.borrow_mut().push((game, sum, SHARED_XP_INPUT.with(|input| input.get())));
    }
    fn multiplier(&self) -> Option<f32> { (self.mode.get() != 4).then_some(1.25) }
    fn return_site(&self, stack: u64) -> Option<u64> {
        assert_eq!(stack, 99);
        (self.mode.get() != 5).then_some(120)
    }
    fn normalize(&self, game: u64, site: u64, registers: [u64; 3], multiplier: f32, shared: Option<u32>)
        -> Result<Option<ExperienceAward>, String>
    {
        self.seen.borrow_mut().push((game, site, registers, multiplier, shared));
        match self.mode.get() {
            1 => Ok(None),
            2 => Err("rejected input".into()),
            3 => panic!("controlled service panic"),
            _ => Ok(Some(ExperienceAward { source: "shared", raw: 73, amount: 91 })),
        }
    }
    fn info(&self, _: &str) {}
    fn warn(&self, _: &str) { self.warnings.set(self.warnings.get() + 1); }
}
fn context() -> crate::stub::Context {
    // Context contains only integer register/stack snapshots.
    let mut context: crate::stub::Context = unsafe { std::mem::zeroed() };
    context.rcx = 42; context.rdx = 7; context.rsp_entry = 99;
    context.rbx = 10; context.rsi = 20; context.r12 = 30;
    context.r8 = 40; context.r9 = 50; context.rflags = 0x202;
    context
}

#[test]
fn detached_award_uses_live_input_and_only_rewrites_the_amount() {
    let _guard = TEST_LOCK.lock().unwrap();
    let prior = SHARED_XP_INPUT.with(|input| input.replace(Some((42, 73))));
    let services = Probe::default(); let mut context = context();
    award(&services, &mut context);
    assert_eq!(context.rdx, 91);
    assert_eq!(services.seen.borrow()[0], (42, 120, [10, 20, 30], 1.25, Some(73)));
    assert_eq!((context.rcx, context.rbx, context.rsi, context.r12, context.r8, context.r9, context.rflags, context.skip),
        (42, 10, 20, 30, 40, 50, 0x202, 0));
    // A cached sum from another game must never be normalized as this award.
    SHARED_XP_INPUT.with(|input| input.set(Some((43, 900))));
    award(&services, &mut context);
    assert_eq!(services.seen.borrow()[1].4, None);
    SHARED_XP_INPUT.with(|input| input.set(prior));
}

#[test]
fn detached_award_rejections_and_panics_preserve_native_fallback() {
    let _guard = TEST_LOCK.lock().unwrap();
    let services = Probe::default();
    for mode in 1..=5 {
        services.mode.set(mode); let mut context = context();
        let before = services.seen.borrow().len();
        award(&services, &mut context);
        assert_eq!(context.rdx, 7);
        assert_eq!(context.skip, 0);
        assert_eq!(services.seen.borrow().len(), before + usize::from(mode < 4));
    }
    assert_eq!(services.warnings.get(), 1);
    services.mode.set(0); let mut context = context();
    award(&services, &mut context);
    assert_eq!(context.rdx, 91, "a rejected callback must not block later awards");
}

#[test]
fn detached_shared_capture_precedes_tls_replacement_and_rejects_invalid_input() {
    let _guard = TEST_LOCK.lock().unwrap();
    let prior = SHARED_XP_INPUT.with(|input| input.replace(Some((9, 8))));
    let services = Probe::default();
    assert_eq!(capture_input(&services, 17, 42), Some((42, 73)));
    assert_eq!(*services.captures.borrow(), [(42, 73, Some((9, 8)))]);
    services.mode.set(1);
    assert_eq!(capture_input(&services, 17, 42), None);
    assert_eq!(services.captures.borrow().len(), 1);
    assert_eq!(SHARED_XP_INPUT.with(|input| input.replace(prior)), Some((9, 8)));
}
