//! Host-owned crystal scenes and personal run-local bonuses; no new transport.
use super::{event_services::Services,RUNTIME};
use asx_game::official::{EventObject,PartyObservation,StatInputLoan};
use asx_gameplay::{Phase,crystals::{Collection,Color,Crystal,Director,Owner,Totals,MAX_PRESENT}};
use std::{cell::RefCell,collections::{BTreeMap,BTreeSet},sync::{Mutex,atomic::{AtomicBool,Ordering}}};
struct Reserve{rules:Crystal,object:EventObject}
struct State{game:u64,owner:u64,clock:f32,last:Option<f32>,director:Director,sequence:u64,reserves:Vec<Reserve>,collection:Collection,auras:BTreeMap<Owner,u32>,pending:BTreeSet<Owner>,fixture:u32,#[cfg(feature="test-hooks")]fixture_pose:u8,#[cfg(feature="test-hooks")]fixture_setup:bool,#[cfg(feature="test-hooks")]fixture_home:[f32;3]}
static STATE:Mutex<Option<State>>=Mutex::new(None);
static ENABLED:AtomicBool=AtomicBool::new(false);
pub fn install(r:&dyn Services){if std::env::var("ASX_CRYSTALS").as_deref()==Ok("0"){return}if r.events_verify_supplies().is_err(){r.events_warn("Crystal Resonance unavailable: native scene contact changed.");return}ENABLED.store(true,Ordering::Release);r.events_info("Crystal Resonance: independent native crystals and personal run-local totals enabled; amulet gems retired.");}
fn authority(r:&dyn Services,game:u64)->Result<Option<PartyObservation>,String>{if r.events_run_source(game)?.is_none_or(|(slot,_)|slot!=0){return Ok(None)}if let Some(p)=r.events_solo_run(game)?{return Ok(Some(p))}r.events_world_party(game)}
pub fn reset(){let old=STATE.lock().ok().and_then(|mut s|s.take());if let(Some(r),Some(s))=(RUNTIME.get(),old){if authority(r,s.game).ok().flatten().is_some_and(|p|p.clock>=s.clock&&p.actors.first().is_some_and(|a|a.guid==s.owner)){for v in s.reserves{let _=unsafe{r.events_remove_event_object(s.game,v.object)};}}}}
pub fn active()->bool{STATE.lock().is_ok_and(|s|s.as_ref().is_some_and(|s|!s.collection.people.is_empty()||!s.auras.is_empty()))}
pub fn totals(game:u64,owner:Owner)->Totals{STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game).and_then(|s|s.collection.people.get(&owner).copied())).unwrap_or_default()}
pub fn aura(game:u64,owner:Owner,index:Option<u32>){if let Ok(mut guard)=STATE.lock(){if let Some(s)=guard.as_mut().filter(|s|s.game==game){if let Some(index)=index{s.auras.insert(owner,index);}else{s.auras.remove(&owner);}}}}
pub fn wire(game:u64,owner:Owner)->Option<String>{STATE.lock().ok().and_then(|s|{let s=s.as_ref().filter(|s|s.game==game)?;let c=s.collection.people.get(&owner)?;let index=s.auras.get(&owner).copied().unwrap_or(32);Some(format!("G|{}|{}|{}|{}|{}|{index}\n",owner.1,c.0[0],c.0[1],c.0[2],c.0[3]))})}
pub fn member_wire(owner:Owner)->Option<String>{let game=STATE.lock().ok().and_then(|s|s.as_ref().map(|s|s.game))?;wire(game,owner)}
pub fn occupied(game:u64)->Vec<([f32;3],f32)>{STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game).map(|s|s.reserves.iter().map(|v|(v.rules.position,1.)).collect())).unwrap_or_default()}
pub fn finish(r:&dyn Services,game:u64){if !ENABLED.load(Ordering::Acquire){return}if let Err(e)=tick(r,game){r.events_warn(&format!("Crystal Resonance stopped: {e}; core ASX remains active."));ENABLED.store(false,Ordering::Release);reset();}}
fn tick(r:&dyn Services,game:u64)->Result<(),String>{
 let Some(p)=authority(r,game)?else{return Ok(())};let Some(host)=p.actors.first().filter(|a|a.slot==0)else{return Ok(())};let now=p.clock;let ids=p.actors.iter().filter(|a|a.phase!=Phase::Inactive).map(|a|(a.slot,a.guid)).collect::<Vec<_>>();
 if !now.is_finite()||now<0.{return Err("Invalid crystal clock".into())}
 {let mut guard=STATE.lock().map_err(|_|"Crystal state unavailable")?;if guard.as_ref().is_none_or(|s|s.game!=game||s.owner!=host.guid||now+0.01<s.clock){let seed=game^host.guid^std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()as u64;*guard=Some(State{game,owner:host.guid,clock:now,last:None,director:Director::new(seed),sequence:0,reserves:vec![],collection:Collection::default(),auras:BTreeMap::new(),pending:BTreeSet::new(),fixture:0,#[cfg(feature="test-hooks")]fixture_pose:0,#[cfg(feature="test-hooks")]fixture_setup:false,#[cfg(feature="test-hooks")]fixture_home:host.position});}}
 #[cfg(feature="test-hooks")]if fixture_enabled(){
  let expected=std::env::var("ASX_TEST_CRYSTAL_PLAYERS").ok().and_then(|v|v.parse::<usize>().ok()).unwrap_or(1);
  if now<5.||p.actors.len()<expected||p.actors.iter().any(|a|a.phase!=Phase::Playing||a.hp<=0.){return Ok(())}
  let align={let mut guard=STATE.lock().map_err(|_|"Crystal fixture setup unavailable")?;let s=guard.as_mut().ok_or("Crystal fixture missing")?;if !s.fixture_setup{s.fixture_setup=true;if p.actors.len()>1{Some([p.actors.iter().map(|a|a.position[0]).sum::<f32>()/p.actors.len()as f32,p.actors.iter().map(|a|a.position[1]).sum::<f32>()/p.actors.len()as f32,host.position[2]])}else{None}}else{None}};
  if let Some(point)=align{r.events_fixture_event_pose(game,0,host.guid,point)?;r.events_info(&format!("crystal fixture aligned owned host with group center: point={point:?}"));return Ok(())}
 }
 if super::pause::active(){return Ok(())}
 let endless=r.events_endless(game)?;
 let (remove,refresh,spawn)={let mut guard=STATE.lock().map_err(|_|"Crystal state unavailable")?;let s=guard.as_mut().ok_or("Crystal state missing")?;
  if s.last==Some(now){return Ok(())}s.last=Some(now);s.clock=now;s.collection.retain(&ids);s.auras.retain(|id,_|ids.contains(id));s.pending.retain(|id|ids.contains(id));
  let mut grants=vec![];
  for v in &mut s.reserves{v.rules.retain(&ids);for a in &p.actors{if v.rules.claim(now,(a.slot,a.guid),a.position,a.phase==Phase::Playing&&a.hp>0.){grants.push(((a.slot,a.guid),v.rules.color,v.rules.id));}}}
  for(id,color,event)in grants{if s.collection.grant(id,color){s.pending.insert(id);r.events_info(&format!("crystal collected: event={event} slot={} owner={} color={} counts={:?} clock={now}",id.0,id.1,color as u8,s.collection.people[&id].0));}}
  let mut remove=vec![];let mut i=0;while i<s.reserves.len(){if s.reserves[i].rules.done(now){let v=s.reserves.remove(i);remove.push((v.rules.id,v.object));}else{i+=1}}
  let available=p.actors.iter().any(|a|a.phase==Phase::Playing&&a.hp>0.);
  let mut due=available&&s.director.due(now,endless,s.reserves.len());
  #[cfg(feature="test-hooks")]if fixture_enabled(){due=available&&s.reserves.len()<MAX_PRESENT&&s.fixture<6&&now>=3.+s.fixture as f32*4.;if due{s.fixture+=1;}}
  let spawn=if due&&s.reserves.len()<MAX_PRESENT{s.sequence=s.sequence.checked_add(1).ok_or("Crystal identity overflow")?;let color=Color::roll(s.director.draw(4));
   #[cfg(feature="test-hooks")]let color=if fixture_enabled(){Color::ALL[((s.fixture-1)%4)as usize]}else{color};let draws=(0..32).map(|_|(s.director.draw(u32::MAX),s.director.draw(u32::MAX))).collect::<Vec<_>>();Some((s.sequence,color,draws,s.fixture))}else{None};(remove,s.pending.iter().copied().collect::<Vec<_>>(),spawn)
 };
 for(event,object)in remove{unsafe{r.events_remove_event_object(game,object)}?;r.events_info(&format!("crystal removed: event={event} guid={} clock={now}",object.guid()));}
 for id in refresh{
  let result=if p.actors.len()==1{r.events_refresh_stats(game)}else{r.events_refresh_blessings(game,&[id])};
  if result.is_ok(){if let Ok(mut guard)=STATE.lock(){if let Some(s)=guard.as_mut().filter(|s|s.game==game&&s.owner==host.guid){s.pending.remove(&id);}}}else{r.events_warn(&format!("Crystal rebuild deferred: slot={} owner={} result={result:?}",id.0,id.1));}
 }
 #[cfg(feature="test-hooks")]if fixture_enabled(){
  let pose={let mut guard=STATE.lock().map_err(|_|"Crystal fixture state unavailable")?;let s=guard.as_mut().ok_or("Crystal fixture missing")?;
   if s.fixture>=6&&now>=30.&&s.fixture_pose==0{let point=s.reserves.iter().find(|v|v.rules.id==1).map(|v|v.rules.position);if point.is_some(){s.fixture_pose=1;}point}
   else if s.fixture_pose==1&&s.collection.people.get(&(0,host.guid)).is_some_and(|v|v.count()>=6){s.fixture_pose=2;Some(s.fixture_home)}else{None}};
  if let Some(point)=pose{r.events_fixture_event_pose(game,0,host.guid,point)?;r.events_info(&format!("crystal fixture owned host pose: point={point:?} clock={now}"));}
 }
 if let Some((id,color,draws,fixture))=spawn{
  let area=r.events_area(game)?;let point=super::world_events::place(r,game,area,1.,&draws)?;
  #[cfg(feature="test-hooks")]let point=if fixture_enabled(){let distance=if fixture==1{8.}else{0.5};unsafe{r.events_ground(game,[host.position[0]+distance,host.position[1]],0.3,2.)}?}else{point};
  let Some(point)=point else{r.events_info("crystal placement: no accessible site; opportunity consumed");return Ok(())};
  let object=unsafe{r.events_object(game,4+color as u8,point)}?;let rules=Crystal::new(id,color,point,&ids).ok_or("Invalid crystal scene")?;
  let mut reserve=Some(Reserve{rules,object});{let mut guard=STATE.lock().map_err(|_|"Crystal state unavailable")?;if let Some(s)=guard.as_mut().filter(|s|s.game==game&&s.owner==host.guid&&s.clock==now){s.reserves.push(reserve.take().unwrap());}}
  if reserve.is_some(){unsafe{r.events_remove_event_object(game,object)}?;}else{r.events_info(&format!("crystal spawned: event={id} guid={} color={} position={point:?} display={} clock={now}",object.guid(),color as u8,[2973,2971,2972,2974][color as usize]));}
  #[cfg(not(feature="test-hooks"))]let _=fixture;
 }
 Ok(())
}
thread_local!{static STATS:RefCell<Vec<u64>>=const{RefCell::new(Vec::new())};static PROBE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
struct Scope<'a>{r:&'a dyn Services,loan:Option<StatInputLoan>}
impl Drop for Scope<'_>{fn drop(&mut self){if let Some(loan)=self.loan.take(){if let Err(e)=self.r.events_restore_stat_input(loan){self.r.events_warn(&format!("Crystal inputs restoration rejected: {e}"));}}STATS.with(|v|{v.borrow_mut().pop();});}}
pub fn native_stats(r:&dyn Services,game:u64,native:impl FnOnce()){
 if PROBE.with(|v|v.get())||STATS.with(|v|v.borrow().contains(&game)){native();return}let Some(id)=r.events_run_source(game).ok().flatten()else{native();return};let counts=totals(game,id);if counts.count()==0{native();return}
 #[cfg(feature="test-hooks")]
 let base=if fixture_enabled(){
  static SEEN:Mutex<BTreeSet<(Owner,[u32;4])>>=Mutex::new(BTreeSet::new());
  let first=SEEN.lock().ok().is_some_and(|mut seen|seen.len()<128&&seen.insert((id,counts.0)));
  if first{struct Guard(bool);impl Drop for Guard{fn drop(&mut self){PROBE.with(|p|p.set(self.0));}}let _guard=Guard(PROBE.with(|p|p.replace(true)));r.events_refresh_stats(game).ok().and_then(|_|r.events_ring_stat_values(game).ok())}else{None}
 }else{None};
 match r.events_crystal_stats(game,counts){Ok(loan)=>{STATS.with(|v|v.borrow_mut().push(game));let _scope=Scope{r,loan};native();},Err(e)=>{r.events_warn(&format!("Crystal inputs rejected: {e}"));native();}}
 #[cfg(feature="test-hooks")]if let Some(base)=base{
  let actual=r.events_ring_stat_values(game);let before=r.events_test_stat_input(game);let refresh=r.events_refresh_stats(game);let repeat=r.events_ring_stat_values(game);
  r.events_info(&format!("crystal native stats: slot={} owner={} counts={:?} base={base:?} actual={actual:?} stable={} restored={} refresh={refresh:?}",id.0,id.1,counts.0,actual==repeat,before==r.events_test_stat_input(game)));
 }
}
#[cfg(feature="test-hooks")]pub fn fixture_enabled()->bool{std::env::var("ASX_TEST_CRYSTALS").as_deref()==Ok("1")}
#[cfg(not(feature="test-hooks"))]pub fn fixture_enabled()->bool{false}
