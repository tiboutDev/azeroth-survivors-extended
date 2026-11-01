//! Common, reentrant native stat reconstruction boundary.
use super::{blessings, enchantments, crystals, local_base, Adapter, RUNTIME};
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
thread_local!{static REBUILDS:RefCell<Vec<u64>>=const{RefCell::new(Vec::new())};}
struct RebuildScope{game:u64,outer:bool}
impl RebuildScope{
 fn enter(game:u64)->Self{let outer=REBUILDS.with(|s|{let mut s=s.borrow_mut();let outer=!s.contains(&game);s.push(game);outer});Self{game,outer}}
}
impl Drop for RebuildScope{fn drop(&mut self){REBUILDS.with(|s|{let popped=s.borrow_mut().pop();debug_assert_eq!(popped,Some(self.game));});}}
pub(super) static ORIGINAL:AtomicUsize=AtomicUsize::new(0);
pub(super) unsafe extern "system" fn stats(game:u64){
 let scope=RebuildScope::enter(game);
 let original:unsafe extern "system" fn(u64)=std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
 if let Some(r)=RUNTIME.get().filter(|r|r.rules){super::supplies::native_stats(r,game,||crystals::native_stats(r,game,||enchantments::native_stats(r,game,||original(game))));}else{original(game)}
 blessings::apply_stats(game);
 if scope.outer {if let Some(r)=RUNTIME.get().filter(|r|r.rules){
  if let Err(e)=r.adapter.apply_coop_area(&super::io::Access{local:&r.local,adapter:r.adapter},game){r.log.warn(&format!("Cooperative area adjustment rejected: {e}"));}
 }}
}

pub fn hook(adapter:Adapter)->crate::install::HookSpec{let e=adapter.blessing_entries()[2];crate::install::HookSpec{name:"official-run-stat-rebuild",target:local_base()+e.rva as usize,detour:stats as *const ()as usize,trampoline:&ORIGINAL}}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn nested_same_actor_rebuild_scales_only_at_outer_boundary(){
  let outer=RebuildScope::enter(10);assert!(outer.outer);
  {let inner=RebuildScope::enter(10);assert!(!inner.outer);let other=RebuildScope::enter(20);assert!(other.outer);}
  assert!(!RebuildScope::enter(10).outer);drop(outer);assert!(RebuildScope::enter(10).outer);
 }
 #[test]fn failed_rebuild_does_not_leave_a_stale_scope(){
  let _=std::panic::catch_unwind(||{let _scope=RebuildScope::enter(30);panic!("controlled rebuild failure")});assert!(RebuildScope::enter(30).outer);
 }
}
