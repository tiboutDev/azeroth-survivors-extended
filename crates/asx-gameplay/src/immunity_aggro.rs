//! Invulnerability drops existing threat once; later deliberate taunts still work.
use crate::{PlayerId,taunts::Taunts};
use std::collections::BTreeMap;
#[derive(Clone,Default)]pub struct ImmunityAggro {active:BTreeMap<PlayerId,(u64,bool)>}
impl ImmunityAggro {
 pub fn observe(&mut self,id:PlayerId,life:u64,immune:bool,taunts:&mut Taunts)->bool{
  let entering=immune&&self.active.get(&id)!=Some(&(life,true));
  self.active.insert(id,(life,immune));
  if entering{taunts.bindings.retain(|_,b|b.player!=id||b.life!=life);}
  entering
 }
 pub fn retain(&mut self,ids:&[(PlayerId,u64)]){self.active.retain(|id,(life,_)|ids.contains(&(*id,*life)));}
 pub fn immune(&self,id:PlayerId)->bool{self.active.get(&id).is_some_and(|(_,v)|*v)}
}
#[derive(Clone,Copy,Debug)]pub struct Candidate {pub slot:u32,pub position:[f32;3],pub available:bool}
/// Preserve a valid ordinary target. When excluded, choose the nearest living
/// non-immune candidate. A taunt acquired after immunity began takes precedence.
pub fn replacement(current:Option<u32>,position:[f32;3],candidates:&[Candidate],blocked:&[u32],taunt:Option<u32>)->Option<u32>{
 let valid=|slot|candidates.iter().any(|c|c.slot==slot&&c.available);
 if let Some(slot)=taunt.filter(|s|valid(*s)){return Some(slot)}
 if let Some(slot)=current.filter(|s|valid(*s)&&!blocked.contains(s)){return Some(slot)}
 candidates.iter().filter(|c|c.available&&!blocked.contains(&c.slot)).min_by(|a,b|{
  let dist=|c:&Candidate|(c.position[0]-position[0]).powi(2)+(c.position[1]-position[1]).powi(2);
  dist(a).total_cmp(&dist(b)).then(a.slot.cmp(&b.slot))
 }).map(|c|c.slot)
}
