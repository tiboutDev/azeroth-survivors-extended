//! Solo supplies director. All native work happens after short state reservations.
use super::event_services::Services;
use super::RUNTIME;
use asx_game::official::{EventObject,StatInputLoan};
use asx_gameplay::{Phase,supplies::{Supply,Reward,Buffs,Consumable,Family,Discovery}};
use std::{cell::{Cell,RefCell},collections::BTreeMap,sync::{Mutex,atomic::{AtomicBool,Ordering}}};
static ENABLED:AtomicBool=AtomicBool::new(false);
struct Reserve{rules:Supply,object:EventObject}
struct State{game:u64,owner:u64,clock:f32,next:f32,sequence:u64,rng:u64,bosses:u32,reserves:Vec<Reserve>,buffs:Buffs,discovery:Option<Discovery>,auras:BTreeMap<Family,u32>,fixture:u32}
static STATE:Mutex<Option<State>>=Mutex::new(None);
impl State{fn draw(&mut self,n:u32)->u32{loop{self.rng=self.rng.wrapping_add(0x9e3779b97f4a7c15);let mut z=self.rng;z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);z^=z>>31;let max=u64::MAX-u64::MAX%u64::from(n);if z<max{return(z%u64::from(n))as u32}}}}
pub fn install(r:&dyn Services){
 if std::env::var("ASX_LOST_SUPPLIES").as_deref()==Ok("0"){return}
 if r.events_verify_supplies().is_err(){r.events_warn("Lost Supplies unavailable: optional native contact changed; core ASX unchanged.");return}
 ENABLED.store(true,Ordering::Release);r.events_info("Lost Supplies: map-wide generation and personal cooperative claims enabled.");
}
pub fn enabled()->bool{ENABLED.load(Ordering::Acquire)}
pub fn reset(){let old=STATE.lock().ok().and_then(|mut s|s.take());if let(Some(r),Some(s))=(RUNTIME.get(),old){
 cleanup(r,&s);
 if !s.buffs.is_empty(){if let Ok(Some(p))=r.events_solo_run(s.game){if let Some(actor)=p.actors.first().filter(|a|a.guid==s.owner){let _=r.events_refresh_stats(s.game);let _=r.events_clamp_event_health(s.game, s.owner, actor.hp);}}}
}}
fn cleanup(r:&dyn Services,s:&State){if r.events_solo_run(s.game).ok().flatten().is_some_and(|p|p.actors.first().is_some_and(|a|a.guid==s.owner)){for reserve in &s.reserves{let _=unsafe{r.events_remove_event_object(s.game, reserve.object)};}}}
pub fn encounter(game:u64){if let Ok(mut s)=STATE.lock(){if let Some(s)=s.as_mut().filter(|s|s.game==game){s.bosses=s.bosses.saturating_add(1)}}}
pub fn occupied(game:u64)->Vec<([f32;3],f32)>{STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game).map(|s|s.reserves.iter().map(|v|(v.rules.position,2.)).collect())).unwrap_or_default()}
pub fn markers(game:u64)->Vec<super::world_events::Marker>{STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game).map(|s|s.reserves.iter().map(|v|super::world_events::Marker{id:v.rules.event,kind:0,position:v.rules.position,remaining:(v.rules.expires-s.clock).max(0.)}).collect())).unwrap_or_default()}
pub fn buffs(game:u64,owner:u64)->Vec<(Consumable,f32,u32)>{let coop=super::world_events::coop::buffs(game,owner);if !coop.is_empty(){return coop}STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game&&s.owner==owner).map(|s|s.buffs.values().into_iter().map(|b|(b.consumable,(b.expires-s.clock).max(0.),s.auras.get(&b.consumable.family).copied().unwrap_or(32))).collect())).unwrap_or_default()}
pub fn has_aura_projection(game:u64,owner:u64)->bool{if super::world_events::coop::has_aura(game,owner){return true}STATE.lock().is_ok_and(|s|s.as_ref().is_some_and(|s|s.game==game&&s.owner==owner&&!s.auras.is_empty()))}
pub fn discovery(game:u64,owner:u64)->Option<Discovery>{if let Some(d)=super::world_events::coop::discovery(game,owner){return Some(d)}STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game&&s.owner==owner)?.discovery)}
pub fn aura(game:u64,owner:u64,rows:&[(Family,u32)]){super::world_events::coop::aura(game,owner,rows);if let Ok(mut s)=STATE.lock(){if let Some(s)=s.as_mut().filter(|s|s.game==game&&s.owner==owner){s.auras=rows.iter().copied().collect();}}}
pub fn active()->bool{STATE.lock().is_ok_and(|s|s.as_ref().is_some_and(|s|!s.buffs.is_empty()))}
pub fn finish(r:&dyn Services,game:u64){if !ENABLED.load(Ordering::Acquire){return}if let Err(e)=tick(r,game){r.events_warn(&format!("Lost Supplies stopped: {e}; core ASX remains active."));ENABLED.store(false,Ordering::Release);reset();}}
fn notice(r:&dyn Services,game:u64,text:&str){if let Err(e)=unsafe{r.events_notice(game, "Lost Supplies", text)}{r.events_warn(&format!("Supply announcement rejected: {e}"));}}
fn tick(r:&dyn Services,game:u64)->Result<(),String>{
 let Some(p)=r.events_solo_run(game)?else{return Ok(())};let Some(actor)=p.actors.first()else{return Ok(())};let now=p.clock;
 let old={let mut state=STATE.lock().map_err(|_|"Supplies state unavailable")?;if state.as_ref().is_none_or(|s|s.game!=game||s.owner!=actor.guid||now<s.clock){let old=state.take();*state=Some(State{game,owner:actor.guid,clock:now,next:now+30.,sequence:0,rng:game^actor.guid^std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()as u64,bosses:0,reserves:vec![],buffs:Buffs::default(),discovery:None,auras:BTreeMap::new(),fixture:0});old}else{None}};if let Some(old)=old{cleanup(r,&old)}
 let(remove,mut discovery,refresh,spawn)={
  let mut state=STATE.lock().map_err(|_|"Supplies state unavailable")?;let s=state.as_mut().ok_or("Supplies state missing")?;s.clock=now;let ended=actor.phase==Phase::Inactive||actor.hp<=0.;let mut refresh=s.buffs.expire(now);if ended{refresh|=s.buffs.clear()}
  let mut discovery=None;if !ended&&actor.phase==Phase::Playing{for reserve in &mut s.reserves{if let Some(d)=reserve.rules.claim(now,(0,actor.guid),actor.position,true){discovery=Some(d);break}}}
  if let Some(d)=discovery{refresh|=s.buffs.accept(now,d.reward)}// Keep the last owned slots until the native aura boundary retires expired rows.
  let mut remove=vec![];let mut i=0;while i<s.reserves.len(){if ended||now>=s.reserves[i].rules.expires||s.reserves[i].rules.complete(){let item=s.reserves.remove(i);remove.push(item.object)}else{i+=1}}
  let mut spawn=false;if !ended&&actor.phase==Phase::Playing&&now>=s.next{s.next=now+30.+s.draw(21)as f32;spawn=s.draw(100)<50&&!fixture_enabled()}
  #[cfg(feature="test-hooks")]if fixture_enabled()&&!ended&&actor.phase==Phase::Playing{let times=[6.,8.,20.,20.25,20.5,20.75,21.,21.25,24.,25.,28.];let ready=if super::scenario_steps::enabled(){super::scenario_steps::supply_ready(s.fixture,s.reserves.len(),s.discovery.map(|d|d.event))}else{(s.fixture as usize)<times.len()&&now>=times[s.fixture as usize]};if ready{s.fixture+=1;spawn=true}}
  let spawn=if spawn&&s.reserves.len()<8{s.sequence=s.sequence.saturating_add(1);let draws=(0..32).map(|_|(s.draw(u32::MAX),s.draw(u32::MAX))).collect::<Vec<_>>();let(a,b,c)=(s.draw(100),s.draw(100),s.draw(u32::MAX));let mut reward=Reward::roll(s.bosses,a,b,c);
   #[cfg(feature="test-hooks")]if fixture_enabled(){reward=match s.fixture{3..=8=>Reward::Buff(Consumable{family:Family::ALL[(s.fixture-3)as usize],tier:((s.fixture-3)%3+1)as u8}),9=>Reward::Material{rarity:0,variant:0},10=>Reward::Material{rarity:3,variant:1},11=>Reward::Buff(Consumable{family:Family::Recovery,tier:3}),_=>Reward::Material{rarity:0,variant:1}};}
   Some((s.sequence,s.bosses,s.fixture,reward,draws))}else{None};(remove,discovery,refresh,spawn)
 };
 for object in remove{unsafe{r.events_remove_event_object(game, object)}?;r.events_info(&format!("supplies removed: guid={}",object.guid()));}
 if let Some(ref mut d)=discovery{if d.gold>0{d.gold=r.events_credit_supply(game,actor.guid,d.gold)?;}}
 if refresh{
  let before=r.events_test_stat_input(game)?;
  #[cfg(feature="test-hooks")]let mut base=None;
  #[cfg(feature="test-hooks")]if fixture_enabled(){let _skip=Probe::new();r.events_refresh_stats(game)?;base=Some(r.events_ring_stat_values(game)?);}
  r.events_refresh_stats(game)?;
  #[cfg(feature="test-hooks")]if fixture_enabled(){let actual=r.events_ring_stat_values(game)?;r.events_refresh_stats(game)?;let repeat=r.events_ring_stat_values(game)?;
   let current=buffs(game,actor.guid);let values=current.iter().map(|(c,_,_)|(c.family as u8,c.tier,c.value())).collect::<Vec<_>>();r.events_info(&format!("supplies stacked stats: buffs={values:?} base={base:?} actual={actual:?} stable={}",actual==repeat));}
  r.events_clamp_event_health(game, actor.guid, actor.hp)?;
  r.events_info(&format!("supplies stats rebuilt: active={} inputs_restored={} families={}",active(),r.events_test_stat_input(game)?==before,buffs(game,actor.guid).len()));
 }
 if let Some(d)=discovery{if let Ok(mut state)=STATE.lock(){if let Some(s)=state.as_mut().filter(|s|s.game==game&&s.owner==actor.guid){s.discovery=Some(d)}}r.events_info(&format!("supplies discovered: event={} reward={:?} name={} gold={} clock={now}",d.event,d.reward,d.reward.name(),d.gold));}
 if let Some((sequence,bosses,fixture,reward,draws))=spawn{
  let area=r.events_area(game)?;
  let point=if fixture_enabled()&&fixture>1{let distance=if fixture==2{4.5}else{1.};
   #[cfg(feature="test-hooks")]let distance=if super::scenario_steps::enabled(){1.}else{distance};unsafe{r.events_ground(game, [actor.position[0]+distance,actor.position[1]], 0.3, 2.)}?}else{super::world_events::place(r,game,area,2.,&draws)?};
  let Some(point)=point else{r.events_info("supplies attempt: no valid world location");return Ok(())};
  let object=unsafe{r.events_object(game, 0, point)}?;let rules=Supply::new(sequence,now,point,&[(0,actor.guid)],reward,bosses).ok_or("Invalid supplies location")?;
  let mut owned=Some(Reserve{rules,object});{let mut state=STATE.lock().map_err(|_|"Supplies state unavailable")?;if let Some(s)=state.as_mut().filter(|s|s.game==game&&s.owner==actor.guid&&s.clock==now){s.reserves.push(owned.take().unwrap());r.events_info(&format!("supplies active reserves: count={}",s.reserves.len()));}}
  if owned.is_some(){unsafe{r.events_remove_event_object(game, object)}?;return Ok(())}
  #[cfg(feature="test-hooks")]if super::scenario_steps::enabled(){r.events_info(&format!("scenario step: supplies phase={fixture} created={sequence} clock={now}"));}
  notice(r,game,"Lost supplies have been spotted in the area!");r.events_info(&format!("supplies spawned: event={sequence} guid={} position={point:?} clock={now} world_wide={} display=36",object.guid(),!fixture_enabled()||fixture==1));
 }
 Ok(())
}
thread_local!{static STATS:RefCell<Vec<u64>>=const{RefCell::new(Vec::new())};static PROBE:Cell<bool>=const{Cell::new(false)};}
#[cfg(feature="test-hooks")]struct Probe(bool);
#[cfg(feature="test-hooks")]impl Probe{fn new()->Self{Self(PROBE.with(|p|p.replace(true)))}}
#[cfg(feature="test-hooks")]impl Drop for Probe{fn drop(&mut self){PROBE.with(|p|p.set(self.0));}}
struct Scope<'a>{r:&'a dyn Services,loan:Option<StatInputLoan>}
impl Drop for Scope<'_>{fn drop(&mut self){if let Some(loan)=self.loan.take(){if let Err(e)=self.r.events_restore_stat_input(loan){self.r.events_warn(&format!("Consumable stat restoration rejected: {e}"));}}STATS.with(|s|{s.borrow_mut().pop();});}}
pub fn native_stats(r:&dyn Services,game:u64,native:impl FnOnce()){
 if PROBE.with(|p|p.get())||STATS.with(|s|s.borrow().contains(&game)){native();return}
 let Some((_,owner))=r.events_run_source(game).ok().flatten()else{native();return};let current=buffs(game,owner).into_iter().filter(|(_,left,_)|*left>0.).map(|(c,_,_)|c).collect::<Vec<_>>();if current.is_empty(){native();return}
 match r.events_consumable_stats(game, &current){Ok(loan)=>{STATS.with(|s|s.borrow_mut().push(game));let _scope=Scope{r,loan};native()},Err(e)=>{r.events_warn(&format!("Consumable inputs rejected: {e}"));native()}}
}
#[cfg(feature="test-hooks")]fn fixture_enabled()->bool{std::env::var("ASX_TEST_LOST_SUPPLIES").as_deref()==Ok("1")}
#[cfg(not(feature="test-hooks"))]fn fixture_enabled()->bool{false}
