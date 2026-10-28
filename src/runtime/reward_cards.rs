//! Common native reward preparation and acceptance; feature state remains private.
use super::{enchantments, gems, glyphs, local_base, Adapter, RUNTIME};
#[cfg(feature="test-hooks")]
use super::{fixture, wallet};
use std::sync::atomic::{AtomicUsize, Ordering};
pub(super) static CARD:AtomicUsize=AtomicUsize::new(0);
pub(super) static ACCEPT:AtomicUsize=AtomicUsize::new(0);
pub(super) unsafe extern "system" fn card(output:u64,game:u64,descriptor:u64,rarity:u64)->u64{
 let original:unsafe extern "system" fn(u64,u64,u64,u64)->u64=std::mem::transmute(CARD.load(Ordering::Acquire));
 let result=original(output,game,descriptor,rarity);
 if let Some(r)=RUNTIME.get().filter(|r|r.rules){let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{enchantments::card(r,output,game);gems::card(r,output,game);}));}
 glyphs::card(output,game);result
}
pub(super) unsafe extern "system" fn accept(game:u64,index:u64){
 #[cfg(feature="test-hooks")]if let Some(r)=RUNTIME.get(){if wallet::fixture_choice(r,game){return}}
 let selected=glyphs::selecting(game,index);let index=selected.index;if index==u64::MAX{return}
 #[cfg(feature="test-hooks")]let index=if fixture::compat121(){RUNTIME.get().and_then(|r|fixture::compat121_pick(r,game)).unwrap_or(index)}else if enchantments::fixture_enabled(){RUNTIME.get().and_then(|r|r.adapter.ring_fixture_pick(&r.local,r.local.base as u64,game).ok().flatten()).unwrap_or(index)}else if gems::fixture_enabled(){RUNTIME.get().and_then(|r|r.adapter.amulet_fixture_pick(&r.local,r.local.base as u64,game).ok().flatten()).unwrap_or(index)}else{index};
 let r=RUNTIME.get().filter(|r|r.rules);
 let enchantment=r.map(|r|enchantments::selecting(r,game,index));let gem=r.map(|r|gems::selecting(r,game,index));
 let original:unsafe extern "system" fn(u64,u64)=std::mem::transmute(ACCEPT.load(Ordering::Acquire));original(game,index);
 if let(Some(r),Some(e))=(r,enchantment.as_ref()){enchantments::accepted(r,e)}
 if let(Some(r),Some(g))=(r,gem.as_ref()){gems::accepted(r,g)}
 glyphs::accepted(game,&selected);
}
pub fn hooks(adapter:Adapter)->Vec<crate::install::HookSpec>{
 let entries=adapter.reward_entries();if entries.len()<2{return vec![]}
 vec![crate::install::HookSpec{name:"official-reward-card",target:local_base()+entries[0].rva as usize,detour:card as *const ()as usize,trampoline:&CARD},crate::install::HookSpec{name:"official-reward-accept",target:local_base()+entries[1].rva as usize,detour:accept as *const ()as usize,trampoline:&ACCEPT}]
}
