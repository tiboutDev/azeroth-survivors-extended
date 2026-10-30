//! Solo-first Darkmoon trial over a native scene object and the existing Fortune effects.
use super::event_services::Services;
use super::RUNTIME;
use asx_game::official::{RitualScene};
use asx_gameplay::{Phase,rituals::{Ritual,Outcome},blessings::Blessing};
use std::sync::{Mutex,atomic::{AtomicBool,Ordering}};
static ENABLED:AtomicBool=AtomicBool::new(false);
static CONTACTS_VERIFIED:AtomicBool=AtomicBool::new(false);
struct State{game:u64,owner:u64,clock:f32,encounters:u32,next:Option<f32>,attempts:u8,object:Option<RitualScene>,ritual:Option<Ritual>,previous:Option<(u32,Blessing)>,rng:u64,fixture:u8}
static STATE:Mutex<Option<State>>=Mutex::new(None);
impl State {fn draw(&mut self,n:u32)->u32{loop{self.rng=self.rng.wrapping_add(0x9e3779b97f4a7c15);let mut z=self.rng;z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);z^=z>>31;let max=u64::MAX-u64::MAX%u64::from(n);if z<max{return(z%u64::from(n))as u32}}}}
/// The field serializer is shared with the wallet hook. Verify exact original
/// bytes before any ASX detour is installed, never mistake our own hook for drift.
pub fn verify_before_hooks(r:&dyn Services){
 if std::env::var("ASX_DARKMOON_TRIALS").as_deref()==Ok("0"){return}
 if let Err(e)=r.events_verify_ritual(){r.events_warn(&format!("Sayge's Trial unavailable: {e}; boss Fortunes retained."));return}
 CONTACTS_VERIFIED.store(true,Ordering::Release);
}
pub fn install(r:&dyn Services){
 if !CONTACTS_VERIFIED.load(Ordering::Acquire){return}
 ENABLED.store(true,Ordering::Release);r.events_info("Sayge's Trial: solo and cooperative group-presence integration enabled.");
}

pub fn enabled()->bool{ENABLED.load(Ordering::Acquire)}
pub fn replaces_boss_fortunes(game:u64)->bool{enabled()&&(super::world_events::coop::owns(game)||RUNTIME.get().is_some_and(|r|r.events_solo_run(game).ok().flatten().is_some()))}
pub fn reset(){let old=STATE.lock().ok().and_then(|mut s|s.take());if let(Some(r),Some(s))=(RUNTIME.get(),old){cleanup(r,&s);}}
fn cleanup(r:&dyn Services,s:&State){if let Some(o)=s.object.as_ref(){if r.events_solo_run(s.game).ok().flatten().is_some_and(|p|p.actors.first().is_some_and(|a|a.guid==s.owner)){let _=unsafe{r.events_remove_ritual_scene(s.game, o)};}}}
fn initialize(r:&dyn Services,game:u64)->Result<Option<asx_game::official::PartyObservation>,String>{
 let Some(p)=r.events_solo_run(game)?else{return Ok(None)};let Some(a)=p.actors.first()else{return Ok(None)};
 let old={let mut state=STATE.lock().map_err(|_|"Ritual state unavailable")?;if state.as_ref().is_none_or(|s|s.game!=game||s.owner!=a.guid||p.clock<s.clock){let old=state.take();*state=Some(State{game,owner:a.guid,clock:p.clock,encounters:0,next:None,attempts:0,object:None,ritual:None,previous:None,rng:game^a.guid^std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()as u64,fixture:0});old}else{None}};
 if let Some(old)=old{cleanup(r,&old);}Ok(Some(p))
}
pub fn encounter(r:&dyn Services,game:u64,count:u64){
 if !replaces_boss_fortunes(game){return}
 if r.events_solo_run(game).ok().flatten().is_none(){return}
 let Ok(Some(p))=initialize(r,game)else{return};let endless=r.events_endless(game).unwrap_or(false);
 let Ok(mut state)=STATE.lock()else{return};let Some(s)=state.as_mut().filter(|s|s.game==game)else{return};
 s.encounters=s.encounters.saturating_add(1);
 if let Some(ritual)=s.ritual.as_mut(){
  ritual.promote(s.encounters);s.next=None;
  r.events_info(&format!("ritual encounter: count={} retained={} stage={} position={:?}",s.encounters,ritual.id,ritual.encounter,ritual.position));
 }else{
  s.attempts=0;s.next=if u64::from(s.encounters)<count||endless{Some(p.clock+20.+s.draw(21)as f32)}else{None};
  #[cfg(feature="test-hooks")]if super::scenario_steps::enabled()&&s.next.is_some(){s.next=Some(p.clock);}
  r.events_info(&format!("ritual encounter: count={} scheduled={}",s.encounters,s.next.is_some()));
 }
}
pub fn view(game:u64,owner:u64)->Option<(u32,u32,bool)>{STATE.lock().ok().and_then(|s|{let s=s.as_ref().filter(|s|s.game==game&&s.owner==owner)?;let r=s.ritual.as_ref()?;Some((r.id,(r.held*100.).round()as u32,r.inside()))})}
pub fn position(game:u64)->Option<[f32;3]>{STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==game)?.ritual.as_ref().map(|v|v.position))}
pub fn marker(game:u64)->Option<super::world_events::Marker>{STATE.lock().ok().and_then(|s|{let s=s.as_ref().filter(|s|s.game==game)?;let v=s.ritual.as_ref()?;Some(super::world_events::Marker{id:u64::from(v.id),kind:1,position:v.position,remaining:0.})})}
fn notice(r:&dyn Services,game:u64,text:&str){if let Err(e)=unsafe{r.events_notice(game, "Sayge", text)}{r.events_warn(&format!("Ritual announcement rejected: {e}"));}}
pub fn finish(r:&dyn Services,game:u64){if !ENABLED.load(Ordering::Acquire){return}if let Err(e)=tick(r,game){r.events_warn(&format!("Sayge's Trial stopped: {e}; boss Fortune fallback restored for future encounters."));ENABLED.store(false,Ordering::Release);reset();}}
fn tick(r:&dyn Services,game:u64)->Result<(),String>{
 let Some(p)=initialize(r,game)?else{return Ok(())};let Some(a)=p.actors.first()else{return Ok(())};let now=p.clock;
 let (remove,outcome,reward,spawn)={let mut state=STATE.lock().map_err(|_|"Ritual state unavailable")?;let s=state.as_mut().ok_or("Ritual state missing")?;s.clock=now;
  let ended=a.phase==Phase::Inactive||a.hp<=0.;let outcome=if ended{Outcome::Expired}else{s.ritual.as_mut().map_or(Outcome::Pending,|v|v.advance(now,a.position,a.phase==Phase::Playing))};
  let mut reward=None;
  if outcome==Outcome::Success {let stage=s.ritual.as_ref().ok_or("Ritual identity missing")?.encounter;let category=s.draw(asx_gameplay::blessings::CATEGORY_COUNT);let power=s.draw(100000);let next=asx_gameplay::rituals::roll(stage,s.previous,category,power);s.previous=Some(next);reward=Some(next.1);}
  let remove=if outcome!=Outcome::Pending{s.ritual=None;s.object.take()}else{None};
  let spawn=!ended&&a.phase==Phase::Playing&&s.ritual.is_none()&&s.next.is_some_and(|due|now>=due);
  if spawn{s.next=None;s.attempts=s.attempts.saturating_add(1);}
  if ended{s.next=None;}
  (remove,outcome,reward,spawn)
 };
 if let Some(o)=remove.as_ref(){unsafe{r.events_remove_ritual_scene(game, o)}?;r.events_info(&format!("ritual scene removed: entities={}",o.len()));}
 if let Some(b)=reward {r.events_solo_fortune(game,a.guid,b)?;notice(r,game,"Sayge's trial is complete. Your fortune has changed!");r.events_info(&format!("ritual completed: stage={} category={} amount={}",b.boss,b.category,b.amount));}
 else if outcome==Outcome::Expired&&remove.is_some(){notice(r,game,"Sayge's trial has faded.");r.events_info("ritual expired without reward");}
 if spawn {
  let draws={let mut state=STATE.lock().map_err(|_|"Ritual state unavailable")?;let s=state.as_mut().ok_or("Ritual state missing")?;(0..32).map(|_|(s.draw(u32::MAX),s.draw(u32::MAX))).collect::<Vec<_>>()};
  let area=r.events_area(game)?;
  let point=if fixture_enabled()&&!polish_fixture()&&!persistence_fixture(){
   // Local diagnostic scene centers only. The UI scenario's second ritual
   // stays outside the circle so the real native presence projection can hide it.
   let far=ui_fixture()&&STATE.lock().ok().is_some_and(|s|s.as_ref().is_some_and(|s|s.encounters>=2));
   let distance=if far{14.}else{3.};let mut point=None;
   for n in 0..8{let angle=n as f32*std::f32::consts::TAU/8.;let p=[a.position[0]+distance*angle.cos(),a.position[1]+distance*angle.sin()];if let Some(site)=unsafe{r.events_ground_sample(game, p, 0.3)}?{point=Some(site.position(0.));break;}}
   if ui_fixture(){r.events_info(&format!("ritual visual fixture: outside={far} position={point:?}"));}point
  }else{super::world_events::place(r,game,area,8.5,&draws)?};
  if let Some(point)=point {
   let enemies=r.events_enemy_count(game)?;let object=unsafe{r.events_ritual_scene(game, point)}?;let guid=object.anchor();let entities=object.len();
   r.events_info(&format!("ritual scene created: entities={entities} sayge_display=491 enemy_rows_unchanged={}",r.events_enemy_count(game)?==enemies));
   let mut owned=Some(object);
   {let mut state=STATE.lock().map_err(|_|"Ritual state unavailable")?;if let Some(s)=state.as_mut().filter(|s|s.game==game&&s.owner==a.guid&&s.clock==now&&s.ritual.is_none()){s.ritual=Ritual::new(s.encounters,now,point);s.object=owned.take();}}
   if let Some(stale)=owned{unsafe{r.events_remove_ritual_scene(game, &stale)}?;return Ok(())}
   notice(r,game,"Sayge's trial awaits. Hold the circle to claim your fortune!");r.events_info(&format!("ritual spawned: guid={} position={point:?} clock={now}",guid));
  }else{r.events_info("ritual placement: no walkable footprint in bounded sample; retry in 10 simulation seconds");let mut state=STATE.lock().map_err(|_|"Ritual state unavailable")?;if let Some(s)=state.as_mut(){s.next=Some(now+10.);}}
 }
 #[cfg(feature="test-hooks")]if persistence_fixture(){
  static OBSERVED:AtomicBool=AtomicBool::new(false);
  let observed=STATE.lock().ok().and_then(|s|{let s=s.as_ref()?;let v=s.ritual.as_ref()?;(now-v.created>65.&&v.encounter>=2).then(||(v.id,v.encounter,v.position,s.object.as_ref().map(|o|o.anchor())))});
  if let Some((id,stage,point,anchor))=observed{if !OBSERVED.swap(true,Ordering::Relaxed){r.events_info(&format!("ritual persistence observed: id={id} stage={stage} position={point:?} anchor={anchor:?} older_than_60=true"));}}
 }
 #[cfg(feature="test-hooks")]if fixture_enabled(){fixture(r,game,&p)?;}
 Ok(())
}
fn ui_fixture()->bool{cfg!(feature="test-hooks")&&std::env::var("ASX_TEST_RITUAL_UI").as_deref()==Ok("1")}
fn persistence_fixture()->bool{cfg!(feature="test-hooks")&&std::env::var("ASX_TEST_RITUAL_PERSISTENCE").as_deref()==Ok("1")}
fn polish_fixture()->bool{cfg!(feature="test-hooks")&&std::env::var("ASX_TEST_EVENT_POLISH").as_deref()==Ok("1")}
#[cfg(feature="test-hooks")]fn fixture_enabled()->bool{std::env::var("ASX_TEST_DARKMOON_TRIAL").as_deref()==Ok("1")}
#[cfg(not(feature="test-hooks"))]fn fixture_enabled()->bool{false}
#[cfg(feature="test-hooks")]fn fixture(r:&dyn Services,game:u64,p:&asx_game::official::PartyObservation)->Result<(),String>{
 if p.actors.iter().any(|a|a.phase!=Phase::Playing){return Ok(())}
 let observed_targets=if super::scenario_steps::enabled(){r.events_blessing_fixture_targets(game)?}else{vec![]};
 let action={let mut s=STATE.lock().map_err(|_|"Ritual fixture state")?;let s=s.as_mut().ok_or("Ritual fixture missing")?;if polish_fixture()&&s.fixture>=2{return Ok(())}let due=match s.fixture{0=>8.,1=>10.,2=>85.,3=>87.,_=>return Ok(())};let ready=if super::scenario_steps::enabled(){s.fixture==0||(s.fixture==1&&!observed_targets.is_empty())}else{p.clock>=due};if !ready{return Ok(())}let action=s.fixture;s.fixture+=1;action};
 
 if super::scenario_steps::enabled(){r.events_info(&format!("scenario step: ritual action={action} clock={}",p.clock));}
 if action%2==0 {r.events_verify_fixture_boss()?;let def=r.events_blessing_fixture_definition(game, u32::from(action/2))?;unsafe{r.events_fixture_spawn_encounter(game, def)};r.events_info(&format!("ritual fixture native boss spawned: index={}",action/2));}
 else {let targets=r.events_blessing_fixture_targets(game)?;if targets.is_empty(){return Err("Ritual fixture boss target absent".into())}for target in targets{r.events_damage_fixture_target(game,target,1e8)?;}r.events_info("ritual fixture native boss defeated");}
 Ok(())
}
