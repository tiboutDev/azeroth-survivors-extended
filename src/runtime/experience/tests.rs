use super::*;
use crate::test_support::TEST_LOCK;
use std::cell::RefCell;
static OVERRIDE: AtomicUsize = AtomicUsize::new(0);
thread_local! { static SEEN: RefCell<Vec<(usize, [u64;6], u64)>> = const {RefCell::new(Vec::new())}; }
pub(super) fn override_target(at: usize, kind: usize, c: &crate::stub::Context) -> usize {
    SEEN.with(|s| s.borrow_mut().push((kind, [c.r13,c.r9,c.rsi,c.rbx,c.rdi,c.rbp], c.rflags)));
    match OVERRIDE.load(Ordering::Acquire) { 0 => at, target => target }
}
#[unsafe(naked)] unsafe extern "C" fn original() { core::arch::naked_asm!("mov rax,0x100", "ret"); }
#[unsafe(naked)] unsafe extern "C" fn allowed() { core::arch::naked_asm!("mov rax,0x200", "ret"); }
#[unsafe(naked)] unsafe extern "system" fn probe(_entry: u64, _host: u64, _remote: u64, _amount: u64) -> u64 {
    core::arch::naked_asm!("push r13", "push rsi", "push rbx", "push rdi", "sub rsp,40", "mov rax,rcx", "mov r13,rdx", "mov rsi,r9", "mov r9,r8", "mov rbx,0x44", "mov rdi,0x55", "stc", "call rax", "xor rax,r13", "xor rax,r9", "xor rax,rsi", "xor rax,rbx", "xor rax,rdi", "add rsp,40", "pop rdi", "pop rbx", "pop rsi", "pop r13", "ret");
}
#[test] fn all_seven_xp_edges_preserve_host_remote_amount_flags_and_select_the_continuation() {
    let _guard=TEST_LOCK.lock().unwrap();
    let old_h=HOST_ORIGINAL.swap(original as *const () as usize,Ordering::AcqRel);
    let old_r=REMOTE_ORIGINAL.swap(original as *const () as usize,Ordering::AcqRel);
    let old_p:Vec<_>=[&PENDING_HOST_ORIGINAL,&PENDING_REMOTE_ORIGINAL,&PENDING_COMMAND_ORIGINAL,&PRODUCER_HOST_ORIGINAL,&PRODUCER_REMOTE_ORIGINAL].iter().map(|a|a.swap(original as *const () as usize,Ordering::AcqRel)).collect();
    SEEN.with(|s|s.borrow_mut().clear());
    for (kind,entry) in [host_entry as *const () as usize,remote_entry as *const () as usize,pending_host_entry as *const () as usize,pending_remote_entry as *const () as usize,pending_command_entry as *const () as usize,producer_host_entry as *const () as usize,producer_remote_entry as *const () as usize].into_iter().enumerate() {
        for bypass in [false,true] {
            OVERRIDE.store(if bypass {allowed as *const () as usize} else {0},Ordering::Release);
            let result=unsafe{probe(entry as u64,0x1122334455667788,0x8877665544332211,0x123456)};
            assert_eq!(result,0x1122334455667788^0x8877665544332211^0x123456^0x44^0x55^if bypass{0x200}else{0x100});
            let seen=SEEN.with(|s|*s.borrow().last().unwrap());
            assert_eq!((seen.0,&seen.1[..5]),(kind,&[0x1122334455667788,0x8877665544332211,0x123456,0x44,0x55][..]));
            assert_ne!(seen.2&1,0,"entry carry flag survives until native continuation");
        }
    }
    OVERRIDE.store(0,Ordering::Release);HOST_ORIGINAL.store(old_h,Ordering::Release);REMOTE_ORIGINAL.store(old_r,Ordering::Release);
    for (a,old) in [&PENDING_HOST_ORIGINAL,&PENDING_REMOTE_ORIGINAL,&PENDING_COMMAND_ORIGINAL,&PRODUCER_HOST_ORIGINAL,&PRODUCER_REMOTE_ORIGINAL].into_iter().zip(old_p){a.store(old,Ordering::Release);}
}
struct Fake(u8);
impl Services for Fake {
    fn continuation(&self,kind:usize,registers:[u64;6])->Result<Option<usize>,String>{
        assert_eq!((kind,registers),(1,[11,22,33,44,55,66]));
        match self.0 {0=>Ok(Some(1234)),1=>Ok(None),2=>Err("invalid actor".into()),_=>unreachable!()}
    }
    fn info(&self,_:&str){} fn warn(&self,_:&str){}
}
#[test] fn guard_target_falls_back_on_unqualified_or_invalid_native_context() {
    let mut c:crate::stub::Context=unsafe{std::mem::zeroed()};c.r13=11;c.r9=22;c.rsi=33;c.rbx=44;c.rdi=55;c.rbp=66;
    for (mode,expected) in [(0,1234),(1,456),(2,456)] {assert_eq!(target(&Fake(mode),1,&c,456),expected);}
    assert_eq!((c.r13,c.r9,c.rsi,c.skip),(11,22,33,0));
}
