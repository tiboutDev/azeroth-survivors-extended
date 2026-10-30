//! One native completed-encounter hook; optional features subscribe independently.
use super::{Adapter,Runtime,RUNTIME,local_base};
use std::sync::atomic::AtomicUsize;
#[cfg(test)]use std::sync::atomic::Ordering;
pub(super) static EVENT:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_rw!(event_entry,0,event,EVENT);
unsafe extern "system" fn event(c:&mut crate::stub::Context){
 #[cfg(test)]{super::blessings::tests::FRAME.store(c.rbp,Ordering::SeqCst);super::blessings::tests::DEF.store(c.rbx,Ordering::SeqCst);}
 let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};
 if let Ok(Some((game,count)))=r.adapter.blessing_event_context(&r.local,[c.rbp,c.rbx,c.r13],r.local.base as u64){complete(r,game,count);}
}
pub(super) fn complete(r:&Runtime,game:u64,count:u64){
 super::treasure_goblins::encounter(game,count as u32);super::supplies::encounter(game);super::world_events::coop::encounter(r,game,count);super::rituals::encounter(r,game,count);
 if r.adapter.duplicate_loot_enabled(){super::duplicate_loot::encounter(game);}
 #[cfg(feature="test-hooks")]super::duplicate_loot::fixture_loot(r,game);
 super::blessings::encounter(game);
}
pub(super) fn hook(adapter:Adapter)->crate::install::HookSpec {let e=adapter.blessing_entries()[0];crate::install::HookSpec{name:"official-native-encounter",target:local_base()+e.rva as usize,detour:event_entry as *const ()as usize,trampoline:&EVENT}}
#[cfg(test)]#[unsafe(naked)]pub(super) unsafe extern "C" fn test_entry(){core::arch::naked_asm!("jmp {entry}",entry=sym event_entry);}
