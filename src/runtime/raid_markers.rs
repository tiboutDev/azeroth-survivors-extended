//! Host-owned one-time native markers, scoped to the actual cooperative run.
use super::*;
use asx_gameplay::raid_markers::Initialization;
#[derive(Default)]struct State{key:(u64,u64),clock:f32,seen:Initialization,rng:u64}
static STATE:std::sync::LazyLock<std::sync::Mutex<State>>=std::sync::LazyLock::new(||std::sync::Mutex::new(State::default()));
pub(super) fn reset(){if let Ok(mut s)=STATE.lock(){*s=State::default();}}
pub(super) fn observe(r:&Runtime,server:u64,party:Option<&asx_game::official::PartyObservation>){
 let Some(p)=party else{return};
 let Some(host)=p.actors.iter().find(|a|a.slot==0)else{return};
 let Ok(game)=r.adapter.game_address(server)else{return};
 let key=(game,host.guid);
 {
  let Ok(mut s)=STATE.lock()else{return};
  if s.key!=key||p.clock+0.001<s.clock {
   let seed=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1,|d|d.as_nanos()as u64)^host.guid;
   *s=State{key,rng:seed.max(1),..Default::default()};
  }
  s.clock=p.clock;
  if !s.seen.needs(p.actors.iter().filter(|a|a.phase!=asx_gameplay::Phase::Inactive).map(|a|(a.slot,a.guid))){return}
 }
 // Only an uninitialized identity causes native marker-table IO or allocation.
 let Ok(occupied)=r.adapter.raid_markers(&r.local,server)else{return};
 let actors=p.actors.iter().filter(|a|a.phase!=asx_gameplay::Phase::Inactive).map(|a|(a.slot,a.guid)).collect::<Vec<_>>();
 let pending={
  let Ok(mut s)=STATE.lock()else{return};if s.key!=key{return}
  let State{seen,rng,..}=&mut *s;
  seen.plan(&actors,&occupied,|n|{*rng^=*rng<<13;*rng^=*rng>>7;*rng^=*rng<<17;(*rng%n as u64)as usize})
 };
 match pending {
  Ok(pending)=>for(slot,guid,icon)in pending{let result=unsafe{r.adapter.set_raid_marker(&r.local,local_base()as u64,server,slot,guid,icon)};r.log.info(&format!("native marker initialization: slot={slot} guid={guid} icon={icon} result={result:?}"));},
  Err(e)=>r.log.warn(&format!("Native marker initialization rejected: {e}")),
 }
}
