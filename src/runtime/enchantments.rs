//! Run-local ring enchantment identity and native-rank progression over the existing choice/stat boundaries.
use super::reward_services::Services;
#[cfg(feature="test-hooks")]
use super::{io, Runtime};
#[cfg(feature="test-hooks")]
use std::sync::atomic::{AtomicBool, Ordering};
use asx_game::official::{RingChoice,RingStatLoan};
use asx_gameplay::enchantments::{Enchantment,Enchantments,RankedEnchantment,Rank};
#[derive(Default)]struct State{game:u64,clock:f32,rules:Enchantments,offers:Vec<(RingChoice,Enchantment)>,rng:u64}
static STATE:std::sync::Mutex<Option<State>>=std::sync::Mutex::new(None);
thread_local!{static PENDING:std::cell::RefCell<Vec<(RingChoice,Enchantment)>>=const{std::cell::RefCell::new(Vec::new())};static STATS:std::cell::RefCell<Vec<u64>>=const{std::cell::RefCell::new(Vec::new())};}
pub fn reset(){if let Ok(mut s)=STATE.lock(){*s=None;}}
pub fn observe(game:u64,p:Option<&asx_game::official::PartyObservation>){
 let Some(p)=p else{reset();return};
 if let Ok(mut guard)=STATE.lock(){let s=guard.get_or_insert_with(State::default);if s.game!=game||p.clock+0.01<s.clock{*s=State{game,..Default::default()};}s.clock=p.clock;let ids=p.actors.iter().map(|a|(a.slot,a.guid)).collect::<Vec<_>>();s.rules.retain(&ids);s.offers.retain(|(c,_)|ids.contains(&(c.slot,c.guid)));}
}
fn offer(c:RingChoice)->Option<(Enchantment,bool)>{
 if let Some(e)=PENDING.with(|p|p.borrow().iter().rev().find(|(o,_)|o.game==c.game&&o.slot==c.slot&&o.guid==c.guid&&o.ring==c.ring&&c.rank>o.rank).map(|(_,e)|*e)){return Some((e,false))}

 let mut guard=STATE.lock().ok()?;let s=guard.get_or_insert_with(||State{game:c.game,..Default::default()});if s.game!=c.game{*s=State{game:c.game,..Default::default()};}
 if let Some(e)=s.rules.get(c.slot,c.guid,c.ring){return Some((e,false))}
 if let Some((_,e))=s.offers.iter().find(|(o,_)|*o==c){return Some((*e,true))}
 if s.rng==0{s.rng=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()as u64^c.guid^c.game;}
 s.rng=s.rng.wrapping_add(0x9e3779b97f4a7c15);let mut z=s.rng;z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);z^=z>>31;
 let e=Enchantment::ALL[(z&7)as usize];
 if s.offers.len()>=64{s.offers.remove(0);}s.offers.push((c,e));Some((e,true))
}
pub fn card(r:&dyn Services,output:u64,game:u64){
 let Ok(Some(c))=r.ring_card(game,output)else{return};
 let Some((e,first))=offer(c)else{return};
 let Some(preview)=c.rank.checked_add(1).and_then(|rank|e.at_ring_rank(rank))else{return};let previous=if first{None}else{e.at_ring_rank(c.rank)};
 if let Err(err)=r.reward_card_note(output,&preview.note(previous)){r.warn(&format!("Enchantment card rejected: {err}"));}
}
pub struct Selection(Option<(RingChoice,Enchantment)>);
impl Drop for Selection{fn drop(&mut self){if self.0.is_some(){PENDING.with(|p|{p.borrow_mut().pop();});}}}
pub fn selecting(r:&dyn Services,game:u64,index:u64)->Selection{
 let selected=r.ring_selected(game,index).ok().flatten().and_then(|c|STATE.lock().ok().and_then(|s|{let s=s.as_ref().filter(|s|s.game==game)?;if let Some(e)=s.rules.get(c.slot,c.guid,c.ring){return Some((c,e))}s.offers.iter().find(|(o,_)|*o==c).map(|(_,e)|(c,*e))}));
 if let Some(v)=selected{PENDING.with(|p|p.borrow_mut().push(v));}Selection(selected)
}
fn commit(s:&mut State,c:RingChoice,rank:u32,e:Enchantment)->bool{
 let accepted=s.rules.accept(c.slot,c.guid,c.ring,c.rank,rank,e);
 // Native acceptance can synchronously construct the next reward. Keep offers
 // for other rings and newer ranks created by those reentrant callbacks.
 s.offers.retain(|(o,_)|o.slot!=c.slot||o.guid!=c.guid||o.ring!=c.ring||o.rank>=rank);
 accepted
}
pub fn accepted(r:&dyn Services,selection:&Selection){
 let Some((c,e))=selection.0 else{return};
 let Ok(Some((rank,_)))=r.ring_owned(c.game,c.ring)else{return};
 let committed=STATE.lock().ok().is_some_and(|mut guard|{let Some(s)=guard.as_mut().filter(|s|s.game==c.game)else{return false};commit(s,c,rank,e)});
 let promoted=c.rank.checked_add(1)==Some(rank)&&Rank::from_ring_rank(c.rank)!=Rank::from_ring_rank(rank);
 if committed||promoted{if let Err(err)=r.refresh_stats(c.game){r.warn(&format!("Enchantment rebuild rejected: {err}"));}if committed{r.info(&format!("enchantment locked: slot={} guid={} ring={} rank={rank} name={}",c.slot,c.guid,c.ring,e.name()));}else{r.info(&format!("enchantment promoted: slot={} guid={} ring={} rank={rank} name={}",c.slot,c.guid,c.ring,e.at_ring_rank(rank).unwrap().name()));}}
}
struct StatsScope<'a>{r:&'a dyn Services,loan:Option<RingStatLoan>}
impl Drop for StatsScope<'_>{fn drop(&mut self){if let Some(loan)=self.loan.take(){if let Err(e)=self.r.restore_stats(loan){self.r.warn(&format!("Enchantment stat restoration rejected: {e}"));}}STATS.with(|s|{let _=s.borrow_mut().pop();});}}
pub fn native_stats(r:&dyn Services,game:u64,native:impl FnOnce()){
 if STATS.with(|s|s.borrow().contains(&game)){native();return}
 let Some((slot,guid))=r.run_source(game).ok().flatten()else{native();return};
 let entries=STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game).map(|s|s.rules.bonuses(slot,guid))).unwrap_or_default();
 let mut bonuses=entries.into_iter().filter_map(|(ring,e)|r.ring_owned(game,ring).ok().flatten().and_then(|(rank,_)|e.at_ring_rank(rank))).collect::<Vec<_>>();
 let pending=PENDING.with(|p|p.borrow().iter().rev().find(|(c,_)|c.game==game&&c.slot==slot&&c.guid==guid).copied());
 if let Some((c,e))=pending{let locked=STATE.lock().ok().is_some_and(|s|s.as_ref().is_some_and(|s|s.rules.get(slot,guid,c.ring).is_some()));if !locked&&r.ring_owned(game,c.ring).ok().flatten().is_some_and(|(rank,_)|rank==c.rank+1){if let Some(e)=e.at_ring_rank(c.rank+1){bonuses.push(e);}}}
 #[cfg(feature="test-hooks")]
 if let Some(e)=OVERRIDE.with(|v|v.get()){bonuses=vec![e];}
 if bonuses.is_empty(){native();return}
 match r.ring_stats(game,&bonuses){
  Ok(loan)=>{STATS.with(|s|s.borrow_mut().push(game));let _scope=StatsScope{r,loan};native();},
  Err(e)=>{r.warn(&format!("Enchantment stats rejected: {e}"));native();}
 }
}
pub fn annotation(content:&str)->Option<(RankedEnchantment,usize)>{let at=content.find("\n\nEnchantment:").or_else(||content.find("\n\nActive enchantment:"));let(rest,prefix)=at.map(|at|(&content[at+2..],at)).unwrap_or((content,0));let name=rest.strip_prefix("Enchantment: ").or_else(||rest.strip_prefix("Active enchantment: "))?.lines().next()?;for kind in Enchantment::ALL{for rank in Rank::ALL{let e=RankedEnchantment{kind,rank};if e.name()==name{return Some((e,prefix))}}}None}

#[cfg(feature="test-hooks")]thread_local!{static OVERRIDE:std::cell::Cell<Option<RankedEnchantment>>=const{std::cell::Cell::new(None)};}
#[cfg(feature="test-hooks")]
pub fn fixture_enabled()->bool{std::env::var("AZCOOP_TEST_ENCHANTMENTS").as_deref()==Ok("1")}
#[cfg(feature="test-hooks")]
pub fn fixture_tick(r:&Runtime,game:u64){
 if !fixture_enabled(){return}
 static NEXT:std::sync::Mutex<(f32,u32)>=std::sync::Mutex::new((4.,0));
 static MATRIX:AtomicBool=AtomicBool::new(false);
 let Some(p)=r.extension_party()else{return};if p.actors.is_empty()||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 let io=io::Access{local:&r.local,adapter:r.adapter};
 if p.clock>=3. && !MATRIX.swap(true,Ordering::Relaxed){
  let before=r.adapter.ring_input_header(&r.local,game);
  let _=asx_game::official::SupportCalls::refresh_support(&io,game);let baseline=r.adapter.ring_stat_values(&r.local,game);
  for kind in Enchantment::ALL{for rank in Rank::ALL{let e=RankedEnchantment{kind,rank};
   OVERRIDE.with(|v|v.set(Some(e)));let _=asx_game::official::SupportCalls::refresh_support(&io,game);let actual=r.adapter.ring_stat_values(&r.local,game);let _=asx_game::official::SupportCalls::refresh_support(&io,game);let repeat=r.adapter.ring_stat_values(&r.local,game);OVERRIDE.with(|v|v.set(None));
   r.log.info(&format!("enchantment ranked matrix: tier={} name={} base={baseline:?} actual={actual:?} repeat={repeat:?} restored={}",rank as u8,kind.name(),r.adapter.ring_input_header(&r.local,game)==before));
  }
  }
  let _=asx_game::official::SupportCalls::refresh_support(&io,game);
 }
 let due=NEXT.lock().ok().is_some_and(|mut n|{if p.clock<n.0||n.1>=40{return false}n.0=p.clock+0.6;n.1+=1;true});
 if due{
  type Gold=unsafe extern "system" fn(u64,u32)->usize;type Chest=unsafe extern "system" fn(u64)->bool;
  let gold:Gold=unsafe{std::mem::transmute(r.local.base+r.adapter.gold_entry().rva as usize)};let chest:Chest=unsafe{std::mem::transmute(r.local.base+r.adapter.chest_entry().rva as usize)};
  let ok=unsafe{gold(game,1000);chest(game)};r.log.info(&format!("enchantment fixture native cache: {ok}"));
 }
 static SAMPLED:std::sync::Mutex<Option<std::time::Instant>>=std::sync::Mutex::new(None);
 if SAMPLED.lock().ok().is_some_and(|mut s|{if s.is_some_and(|t|t.elapsed()<std::time::Duration::from_secs(3)){return false}*s=Some(std::time::Instant::now());true}){
  let owned=STATE.lock().ok().and_then(|guard|guard.as_ref().map(|s|{
      let mut owned=Vec::new();for a in &p.actors{for ring in 0..9u8{if let Some(e)=s.rules.get(a.slot,a.guid,ring){owned.push((a.slot,a.guid,ring,e.name()));}}}owned
  })).unwrap_or_default();
  let header=r.adapter.ring_input_header(&r.local,game);let _=asx_game::official::SupportCalls::refresh_support(&io,game);let first=r.adapter.ring_stat_values(&r.local,game);let _=asx_game::official::SupportCalls::refresh_support(&io,game);let repeat=r.adapter.ring_stat_values(&r.local,game);
  r.log.info(&format!("enchantment fixture rebuild: stable={} restored={}",first==repeat,header==r.adapter.ring_input_header(&r.local,game)));
  let rows=owned.into_iter().map(|(slot,guid,ring,name)|{let row=r.adapter.ring_actor_owned(&r.local,game,slot,guid,ring,r.local.base as u64);(slot,guid,ring,name,row)}).collect::<Vec<_>>();
  r.log.info(&format!("enchantment fixture retained: {rows:?}"));
 }
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn reentrant_reward_build_keeps_the_locked_preview_and_other_ring_offer(){
 let _guard=crate::test_support::TEST_LOCK.lock().unwrap();
  reset();let c=RingChoice{game:10,slot:0,guid:111,ring:0,rank:1,rarity:0,value:1};let other=RingChoice{ring:1,..c};let(e,_)=offer(c).unwrap();let second=offer(other).unwrap();
  PENDING.with(|p|p.borrow_mut().push((c,e)));assert_eq!(offer(RingChoice{rank:2,value:2,..c}),Some((e,false)));
  {let mut s=STATE.lock().unwrap();let s=s.as_mut().unwrap();assert!(commit(s,c,2,e));assert!(s.offers.iter().any(|(o,x)|*o==other&&*x==second.0));}
  PENDING.with(|p|{p.borrow_mut().pop();});assert!(Enchantment::Might.at_ring_rank(4).unwrap().note(Enchantment::Might.at_ring_rank(3)).contains("Improves enchantment"));assert_eq!(offer(RingChoice{rank:3,value:3,..c}),Some((e,false)));reset();
 }
}
