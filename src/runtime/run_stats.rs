//! Common, reentrant native stat reconstruction boundary.
use super::{blessings, enchantments, gems, local_base, Adapter, RUNTIME};
use std::sync::atomic::{AtomicUsize, Ordering};
pub(super) static ORIGINAL:AtomicUsize=AtomicUsize::new(0);
pub(super) unsafe extern "system" fn stats(game:u64){
 let original:unsafe extern "system" fn(u64)=std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
 if let Some(r)=RUNTIME.get().filter(|r|r.rules){gems::native_stats(r,game,||enchantments::native_stats(r,game,||original(game)));}else{original(game)}
 blessings::apply_stats(game);
}

pub fn hook(adapter:Adapter)->crate::install::HookSpec{let e=adapter.blessing_entries()[2];crate::install::HookSpec{name:"official-run-stat-rebuild",target:local_base()+e.rva as usize,detour:stats as *const ()as usize,trampoline:&ORIGINAL}}
