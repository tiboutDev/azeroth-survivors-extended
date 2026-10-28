//! One immutable gem per run/actor, over existing native choice and stat callbacks.
use super::reward_services::Services;
#[cfg(feature="test-hooks")]
use super::{io, Runtime};
#[cfg(feature="test-hooks")]
use std::sync::atomic::{AtomicBool, Ordering};
use asx_game::official::{AmuletChoice,RingStatLoan};
use asx_gameplay::gems::{Gem,Gems};
#[derive(Default)]struct State{game:u64,clock:f32,rules:Gems,offers:Vec<(AmuletChoice,Gem)>,rng:u64}
static STATE:std::sync::Mutex<Option<State>>=std::sync::Mutex::new(None);
thread_local!{static PENDING:std::cell::RefCell<Vec<(AmuletChoice,Gem)>>=const{std::cell::RefCell::new(Vec::new())};static STATS:std::cell::RefCell<Vec<u64>>=const{std::cell::RefCell::new(Vec::new())};}
pub fn reset(){if let Ok(mut s)=STATE.lock(){*s=None;}}
pub fn observe(game:u64,p:Option<&asx_game::official::PartyObservation>){
 let Some(p)=p else{reset();return};
 if let Ok(mut guard)=STATE.lock(){let s=guard.get_or_insert_with(State::default);if s.game!=game||p.clock+0.01<s.clock{*s=State{game,..Default::default()};}s.clock=p.clock;let ids=p.actors.iter().map(|a|(a.slot,a.guid)).collect::<Vec<_>>();s.rules.retain(&ids);s.offers.retain(|(c,_)|ids.contains(&(c.slot,c.guid)));}
}
fn offer(c:AmuletChoice)->Option<(Gem,bool)>{
 if let Some(e)=PENDING.with(|p|p.borrow().iter().rev().find(|(old,_)|old.game==c.game&&old.slot==c.slot&&old.guid==c.guid&&c.previous==Some(old.amulet)).map(|(_,e)|*e)){return Some((e,false))}
 let mut guard=STATE.lock().ok()?;let s=guard.get_or_insert_with(||State{game:c.game,..Default::default()});if s.game!=c.game{*s=State{game:c.game,..Default::default()};}
 if let Some(e)=s.rules.get(c.slot,c.guid){return Some((e,false))}
 if let Some((_,e))=s.offers.iter().find(|(old,_)|*old==c){return Some((*e,true))}
 if s.rng==0{s.rng=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()as u64^c.guid^c.game;}
 s.rng=s.rng.wrapping_add(0x9e3779b97f4a7c15);let mut z=s.rng;z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);z^=z>>31;
 let gem=Gem::roll(z as u32);if s.offers.len()>=64{s.offers.remove(0);}s.offers.push((c,gem));Some((gem,true))
}
pub fn card(r:&dyn Services,output:u64,game:u64){
 let c=match r.amulet_card(game,output){Ok(Some(c))=>c,Ok(None)=>return,Err(e)=>{r.warn(&format!("Gem card rejected: {e}"));return}};
 let Some((gem,first))=offer(c)else{return};if let Err(e)=r.reward_card_note(output,&gem.note(first)){r.warn(&format!("Gem card text rejected: {e}"));}
}
pub struct Selection(Option<(AmuletChoice,Gem)>);
impl Drop for Selection{fn drop(&mut self){if self.0.is_some(){PENDING.with(|p|{p.borrow_mut().pop();});}}}
pub fn selecting(r:&dyn Services,game:u64,index:u64)->Selection{
 let chosen=r.amulet_selected(game,index).ok().flatten().and_then(|c|STATE.lock().ok().and_then(|s|{let s=s.as_ref().filter(|s|s.game==game)?;s.rules.get(c.slot,c.guid).or_else(||s.offers.iter().find(|(old,_)|*old==c).map(|(_,e)|*e)).map(|e|(c,e))}));
 if let Some(e)=chosen{PENDING.with(|p|p.borrow_mut().push(e));}Selection(chosen)
}
fn commit(s:&mut State,c:AmuletChoice,owned:Option<u64>,rank:u32,gem:Gem)->bool{
 let committed=s.rules.accept(c.slot,c.guid,owned==Some(c.amulet)&&(c.previous!=owned||rank>c.rank),gem);
 // A newly constructed reward may already exist on native acceptance return.
 s.offers.retain(|(old,_)|old.slot!=c.slot||old.guid!=c.guid||old.previous!=c.previous);
 committed
}
pub fn accepted(r:&dyn Services,selection:&Selection){
 let Some((c,gem))=selection.0 else{return};let Ok(owned)=r.amulet_owned(c.game)else{return};let Ok(rank)=r.amulet_rank(c.game)else{return};if owned!=Some(c.amulet)||(owned==c.previous&&rank<=c.rank){return}
 let committed=STATE.lock().ok().is_some_and(|mut s|s.as_mut().filter(|s|s.game==c.game).is_some_and(|s|commit(s,c,owned,rank,gem)));
 if let Err(e)=r.refresh_stats(c.game){r.warn(&format!("Gem rebuild rejected: {e}"));}
 r.info(&format!("gem {}: slot={} guid={} amulet={} previous={:?} name={}",if committed{"locked"}else{"retained"},c.slot,c.guid,c.amulet,c.previous,gem.name()));
}
struct StatsScope<'a>{r:&'a dyn Services,loan:Option<RingStatLoan>}
impl Drop for StatsScope<'_>{fn drop(&mut self){if let Some(loan)=self.loan.take(){if let Err(e)=self.r.restore_stats(loan){self.r.warn(&format!("Gem stat restoration rejected: {e}"));}}STATS.with(|s|{s.borrow_mut().pop();});}}
pub fn native_stats(r:&dyn Services,game:u64,native:impl FnOnce()){
 if STATS.with(|s|s.borrow().contains(&game)){native();return}
 let Some((slot,guid))=r.run_source(game).ok().flatten()else{native();return};
 let mut gem=STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game).and_then(|s|s.rules.get(slot,guid)));
 if gem.is_none(){let pending=PENDING.with(|p|p.borrow().iter().rev().find(|(c,_)|c.game==game&&c.slot==slot&&c.guid==guid).copied());if let Some((c,e))=pending{if r.amulet_owned(game).ok().flatten()==Some(c.amulet)&&(c.previous!=Some(c.amulet)||r.amulet_rank(game).ok().is_some_and(|rank|rank>c.rank)){gem=Some(e);}}}
 #[cfg(feature="test-hooks")]
 if let Some(e)=OVERRIDE.with(|v|v.get()){gem=Some(e);}
 let Some(gem)=gem else{native();return};
 match r.gem_stats(game,gem){Ok(loan)=>{STATS.with(|s|s.borrow_mut().push(game));let _scope=StatsScope{r,loan};native();},Err(e)=>{r.warn(&format!("Gem stats rejected: {e}"));native();}}
}
pub fn annotation(content:&str)->Option<(Gem,usize)>{let at=content.find("\n\nGem:").or_else(||content.find("\n\nActive gem:"));let(rest,prefix)=at.map(|at|(&content[at+2..],at)).unwrap_or((content,0));let name=rest.strip_prefix("Gem: ").or_else(||rest.strip_prefix("Active gem: "))?.lines().next()?;Gem::ALL.into_iter().find(|g|g.name()==name).map(|g|(g,prefix))}
#[cfg(feature="test-hooks")]thread_local!{static OVERRIDE:std::cell::Cell<Option<Gem>>=const{std::cell::Cell::new(None)};}
#[cfg(feature="test-hooks")]pub fn fixture_enabled()->bool{std::env::var("AZCOOP_TEST_GEMS").as_deref()==Ok("1")}
#[cfg(feature="test-hooks")]
pub fn fixture_tick(r:&Runtime,game:u64){
 if !fixture_enabled(){return}
 let Some(p)=r.extension_party()else{return};if p.actors.is_empty()||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 let io=io::Access{local:&r.local,adapter:r.adapter};
 static MATRIX:AtomicBool=AtomicBool::new(false);
 if p.clock>=3.&&!MATRIX.swap(true,Ordering::Relaxed){
  let before=r.adapter.ring_input_header(&r.local,game);let _=asx_game::official::SupportCalls::refresh_support(&io,game);let baseline=r.adapter.ring_stat_values(&r.local,game);
  for gem in Gem::ALL{OVERRIDE.with(|v|v.set(Some(gem)));let _=asx_game::official::SupportCalls::refresh_support(&io,game);let actual=r.adapter.ring_stat_values(&r.local,game);let _=asx_game::official::SupportCalls::refresh_support(&io,game);let repeat=r.adapter.ring_stat_values(&r.local,game);OVERRIDE.with(|v|v.set(None));r.log.info(&format!("gem matrix: kind={} base={baseline:?} actual={actual:?} repeat={repeat:?} restored={}",gem as u8,r.adapter.ring_input_header(&r.local,game)==before));}
  let _=asx_game::official::SupportCalls::refresh_support(&io,game);
 }
 static NEXT:std::sync::Mutex<(f32,u32)>=std::sync::Mutex::new((4.,0));
 let due=NEXT.lock().ok().is_some_and(|mut n|{if p.clock<n.0||n.1>=40{return false}n.0=p.clock+0.6;n.1+=1;true});
 if due{type Gold=unsafe extern "system" fn(u64,u32)->usize;type Chest=unsafe extern "system" fn(u64)->bool;let gold:Gold=unsafe{std::mem::transmute(r.local.base+r.adapter.gold_entry().rva as usize)};let chest:Chest=unsafe{std::mem::transmute(r.local.base+r.adapter.chest_entry().rva as usize)};let ok=unsafe{gold(game,1000);chest(game)};r.log.info(&format!("gem fixture native cache: {ok}"));}
 static SAMPLE:std::sync::Mutex<Option<std::time::Instant>>=std::sync::Mutex::new(None);
 if SAMPLE.lock().ok().is_some_and(|mut s|{if s.is_some_and(|t|t.elapsed()<std::time::Duration::from_secs(3)){return false}*s=Some(std::time::Instant::now());true}){let before=r.adapter.ring_input_header(&r.local,game);let _=asx_game::official::SupportCalls::refresh_support(&io,game);let first=r.adapter.ring_stat_values(&r.local,game);let _=asx_game::official::SupportCalls::refresh_support(&io,game);r.log.info(&format!("gem fixture rebuild: stable={} restored={}",first==r.adapter.ring_stat_values(&r.local,game),before==r.adapter.ring_input_header(&r.local,game)));}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn stable_offers_reentrant_replacement_and_immutable_actor_gem(){
 let _guard=crate::test_support::TEST_LOCK.lock().unwrap();
 reset();let c=AmuletChoice{game:10,slot:0,guid:11,amulet:1,previous:None,rank:0,fingerprint:1};let e=offer(c).unwrap();assert!(e.1);assert_eq!(offer(c),Some(e));
 let next=AmuletChoice{amulet:2,previous:Some(1),rank:1,fingerprint:2,..c};PENDING.with(|p|p.borrow_mut().push((c,e.0)));assert_eq!(offer(next),Some((e.0,false)));
 {let mut s=STATE.lock().unwrap();let s=s.as_mut().unwrap();assert!(!commit(s,c,None,0,e.0));assert!(commit(s,c,Some(1),1,e.0));assert!(!commit(s,next,Some(2),1,Gem::Ruby));assert_eq!(s.rules.get(0,11),Some(e.0));}
 PENDING.with(|p|{p.borrow_mut().pop();});assert_eq!(offer(next),Some((e.0,false)));assert_eq!(annotation(&e.0.note(true)),Some((e.0,0)));reset();
 }
}

#[cfg(feature="test-hooks")]pub fn fixture_has_gem(slot:u32,guid:u64)->bool{STATE.lock().ok().is_some_and(|s|s.as_ref().is_some_and(|s|s.rules.get(slot,guid).is_some()))}
