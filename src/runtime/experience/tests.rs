use super::*;
use crate::test_support::TEST_LOCK;
use asx_game::official::{ActorObservation, PartyObservation};
use asx_gameplay::Phase;
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

pub static GAME: AtomicU64 = AtomicU64::new(0);
pub static AMOUNT: AtomicU64 = AtomicU64::new(0);

#[unsafe(naked)]
unsafe extern "C" fn continuation() { core::arch::naked_asm!("ret"); }
#[unsafe(naked)]
unsafe extern "system" fn probe(_game: u64, _amount: u64) -> u64 {
    core::arch::naked_asm!("push rsi","push r13","sub rsp,40","mov r13,rcx","mov rsi,rdx","call {entry}","mov rax,r13","xor rax,rsi","add rsp,40","pop r13","pop rsi","ret",entry=sym host_entry);
}
#[test]
fn host_dead_guard_reads_and_preserves_both_native_nonvolatile_registers() {
    let _guard = TEST_LOCK.lock().unwrap();
    let old = HOST_ORIGINAL.swap(continuation as *const () as usize, Ordering::SeqCst);
    let result = unsafe { probe(0x1122334455667788, 0x345678) };
    HOST_ORIGINAL.store(old, Ordering::SeqCst);
    assert_eq!(GAME.load(Ordering::SeqCst), 0x1122334455667788);
    assert_eq!(AMOUNT.load(Ordering::SeqCst), 0x345678);
    assert_eq!(result, 0x1122334455667788 ^ 0x345678);
}

fn actor(slot: u32, guid: u64, phase: Phase) -> ActorObservation {
    ActorObservation { slot, guid, phase, gold: 0, level: 1, xp: 0, class_id: 0, race_id: 0,
        xp_multiplier: 1.0, gold_multiplier: 1.0, hp: 1.0, maximum: 1.0, position: [0.0; 3] }
}

#[derive(Clone, Copy, PartialEq, Eq)] enum Replay { Success, SafeReject, Uncertain }
struct Fake {
    game: Cell<u64>, source: Cell<Option<(u32, u64)>>, party: std::cell::RefCell<PartyObservation>,
    replay: Cell<Replay>, reenter: Cell<bool>, calls: Cell<u32>, logs: Cell<u32>, expected: std::cell::RefCell<Vec<(u64,u32,u64,u32)>>,
}
impl Fake {
    fn new() -> Self { Self { game: Cell::new(10), source: Cell::new(Some((0, 100))),
        party: std::cell::RefCell::new(PartyObservation { clock: 1.0, actors: vec![actor(0, 100, Phase::Playing)] }),
        replay: Cell::new(Replay::Success), reenter: Cell::new(false), calls: Cell::new(0), logs: Cell::new(0), expected: std::cell::RefCell::new(Vec::new()) } }
    fn capture(&self, amount: u32) { if let Some((slot,guid))=self.source.get(){self.expected.borrow_mut().push((self.game.get(),slot,guid,amount));} capture(self, self.game.get(), amount); }
}
impl Services for Fake {
    fn deferred_actor(&self, _: u64) -> Result<Option<(u32,u64)>, String> { Ok(self.source.get()) }
    fn world_party(&self, game: u64) -> Result<Option<PartyObservation>, String> { let party=self.party.borrow().clone();let live=party.actors.iter().map(|a|(a.slot,a.guid)).collect::<Vec<_>>();self.expected.borrow_mut().retain(|(g,slot,guid,_)|*g==game&&live.contains(&(*slot,*guid)));Ok(Some(party)) }
    fn replay(&self, game:u64, slot:u32, guid:u64, amount:u32)->Result<bool,String> {
        let expected=self.expected.borrow_mut().remove(0);
        assert_eq!((game,slot,guid,amount),expected);
        self.calls.set(self.calls.get()+1);
        if self.reenter.replace(false) { finish(self,game); }
        match self.replay.get() { Replay::Success=>Ok(true),Replay::SafeReject=>{self.expected.borrow_mut().insert(0,expected);Ok(false)},Replay::Uncertain=>Err("commit status uncertain".into()) }
    }
    fn info(&self,_:&str){self.logs.set(self.logs.get()+1)}
    fn warn(&self,_:&str){self.logs.set(self.logs.get()+1)}
}
fn clean() { reset(); }

#[test]
fn deferred_entries_are_taken_before_reentrant_replay() {
    let _guard=TEST_LOCK.lock().unwrap(); clean();
    let fake=Fake::new();fake.capture(40);fake.reenter.set(true);
    finish(&fake,10);
    assert_eq!(fake.calls.get(),1,"nested finish sees the entry already removed");
    assert_eq!(STATE.lock().unwrap().bank.take(1,100),0);
    clean();
}

#[test]
fn only_a_certain_safe_rejection_is_recredited() {
    let _guard=TEST_LOCK.lock().unwrap();clean();let fake=Fake::new();fake.capture(40);
    fake.replay.set(Replay::SafeReject);finish(&fake,10);
    assert_eq!(fake.calls.get(),1);
    fake.replay.set(Replay::Success);finish(&fake,10);
    assert_eq!(fake.calls.get(),2);assert_eq!(STATE.lock().unwrap().bank.take(1,100),0);
    fake.capture(40);fake.replay.set(Replay::Uncertain);finish(&fake,10);
    assert_eq!(fake.calls.get(),3);fake.replay.set(Replay::Success);finish(&fake,10);
    assert_eq!(fake.calls.get(),3,"uncertain native commit is never retried");
    clean();
}

#[test]
fn game_clock_and_actor_life_changes_retire_pending_credits() {
    let _guard=TEST_LOCK.lock().unwrap();clean();let fake=Fake::new();fake.capture(40);
    fake.party.borrow_mut().actors[0]=actor(0,101,Phase::Playing);finish(&fake,10);
    assert_eq!(fake.calls.get(),0,"reused slot with a different life GUID receives no old XP");
    fake.game.set(11);fake.source.set(Some((0,200)));fake.party.replace(PartyObservation{clock:1.0,actors:vec![actor(0,200,Phase::Playing)]});fake.capture(7);finish(&fake,11);
    assert_eq!(fake.calls.get(),1,"new game delivers only its own identity and amount");
    fake.game.set(12);fake.source.set(Some((0,300)));fake.party.replace(PartyObservation{clock:1.0,actors:vec![actor(0,300,Phase::Playing)]});fake.capture(13);finish(&fake,12);
    assert_eq!(fake.calls.get(),2);
    clean();
}

#[test]
fn downed_credits_wait_for_playing_and_clock_rewind_resets_them() {
    let _guard=TEST_LOCK.lock().unwrap();clean();let fake=Fake::new();fake.capture(40);
    fake.party.borrow_mut().actors[0]=actor(0,100,Phase::Downed);finish(&fake,10);
    assert_eq!(fake.calls.get(),0);
    fake.party.borrow_mut().actors[0]=actor(0,100,Phase::Playing);finish(&fake,10);
    assert_eq!(fake.calls.get(),1);
    fake.capture(40);fake.party.borrow_mut().clock=0.5;finish(&fake,10);
    fake.party.borrow_mut().clock=1.0;finish(&fake,10);
    assert_eq!(fake.calls.get(),1,"clock rewind retired the pending entry");
    clean();
}
