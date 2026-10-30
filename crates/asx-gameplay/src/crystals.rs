//! Run-local world crystals. No native layouts or network authority.
use std::collections::{BTreeMap,BTreeSet};
pub type Owner=(u32,u64);
pub const CLAIM_SECONDS:f32=60.;
pub const PICKUP_RADIUS:f32=1.5;
pub const MAX_PRESENT:usize=2;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]#[repr(u8)]
pub enum Color{Red,Blue,Green,Yellow}
impl Color{
 pub const ALL:[Self;4]=[Self::Red,Self::Blue,Self::Green,Self::Yellow];
 pub fn roll(draw:u32)->Self{Self::ALL[(draw%4)as usize]}
 pub fn percent(self)->u32{[10,5,15,10][self as usize]}
 pub fn stat(self)->u32{[0,1,2,8][self as usize]}
 pub fn name(self)->&'static str{["Red Crystal","Blue Crystal","Green Crystal","Yellow Crystal"][self as usize]}
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Totals(pub [u32;4]);
impl Totals{
 pub fn add(&mut self,color:Color)->bool{let n=&mut self.0[color as usize];if *n>=1_000_000{return false}*n+=1;true}
 pub fn count(self)->u32{self.0.iter().sum()}
 pub fn inputs(self)->Vec<(u32,f32)>{Color::ALL.into_iter().filter_map(|c|{let n=self.0[c as usize];(n>0).then_some((c.stat(),n as f32*c.percent()as f32/100.))}).collect()}
}
#[derive(Clone,Debug)]pub struct Crystal{pub id:u64,pub color:Color,pub position:[f32;3],pub deadline:Option<f32>,eligible:BTreeSet<Owner>,claimed:BTreeSet<Owner>}
impl Crystal{
 pub fn new(id:u64,color:Color,position:[f32;3],owners:&[Owner])->Option<Self>{
  if id==0||position.iter().any(|x|!x.is_finite())||owners.is_empty()||owners.len()>5||owners.iter().any(|(slot,guid)|*slot>=5||*guid==0){return None}
  let eligible=owners.iter().copied().collect::<BTreeSet<_>>();if eligible.len()!=owners.len(){return None}
  Some(Self{id,color,position,deadline:None,eligible,claimed:BTreeSet::new()})
 }
 pub fn claim(&mut self,now:f32,owner:Owner,point:[f32;3],playing:bool)->bool{
  if !now.is_finite()||now<0.||!playing||point.iter().any(|x|!x.is_finite())||!self.eligible.contains(&owner)||self.claimed.contains(&owner)||self.deadline.is_some_and(|v|now>=v){return false}
  if (point[0]-self.position[0]).hypot(point[1]-self.position[1])>PICKUP_RADIUS||(point[2]-self.position[2]).abs()>3.{return false}
  self.claimed.insert(owner);if self.deadline.is_none(){self.deadline=Some(now+CLAIM_SECONDS)}true
 }
 pub fn retain(&mut self,owners:&[Owner]){self.eligible.retain(|id|owners.contains(id));}
 pub fn done(&self,now:f32)->bool{self.eligible.iter().all(|id|self.claimed.contains(id))||self.deadline.is_some_and(|at|now>=at)}
}
#[derive(Default)]pub struct Collection{pub people:BTreeMap<Owner,Totals>}
impl Collection{pub fn retain(&mut self,ids:&[Owner]){self.people.retain(|id,_|ids.contains(id));}pub fn grant(&mut self,id:Owner,color:Color)->bool{self.people.entry(id).or_default().add(color)}}
#[derive(Clone,Debug)]pub struct Director{times:Vec<f32>,cursor:usize,next_endless:f32,rng:u64}
impl Director{
 pub fn new(seed:u64)->Self{
  let mut s=Self{times:vec![],cursor:0,next_endless:0.,rng:seed};let count=4+s.draw(3);
  // Prototype window: independent uniform opportunities during a 15-minute run.
  for _ in 0..count{let time=30.+s.draw(841)as f32;s.times.push(time)}s.times.sort_by(f32::total_cmp);s
 }
 pub fn draw(&mut self,n:u32)->u32{assert!(n>0);loop{self.rng=self.rng.wrapping_add(0x9e3779b97f4a7c15);let mut z=self.rng;z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);z^=z>>31;let max=u64::MAX-u64::MAX%u64::from(n);if z<max{return(z%u64::from(n))as u32}}}
 pub fn due(&mut self,now:f32,endless:bool,present:usize)->bool{
  if !now.is_finite()||now<0.||present>=MAX_PRESENT{return false}
  if endless{if self.next_endless==0.{self.next_endless=now+90.+self.draw(151)as f32;}if now<self.next_endless{return false}self.next_endless=now+90.+self.draw(151)as f32;return true}
  if now>900.{return false}if self.times.get(self.cursor).is_some_and(|at|now>=*at){self.cursor+=1;true}else{false}
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn personal_claims_start_one_shared_deadline_and_keep_model_until_complete(){let ids=[(0,11),(1,22)];let mut c=Crystal::new(1,Color::Red,[0.;3],&ids).unwrap();assert!(!c.done(1000.));assert!(c.claim(1000.,ids[0],[0.;3],true));assert_eq!(c.deadline,Some(1060.));assert!(!c.done(1001.));assert!(!c.claim(1001.,ids[0],[0.;3],true));assert!(c.claim(1059.,ids[1],[0.;3],true));assert_eq!(c.deadline,Some(1060.));assert!(c.done(1059.));}
 #[test]fn deadline_range_pause_death_departure_and_reused_slots(){let mut c=Crystal::new(1,Color::Blue,[0.;3],&[(0,11),(1,22)]).unwrap();assert!(!c.claim(1.,(0,11),[2.,0.,0.],true));assert!(!c.claim(1.,(0,11),[0.;3],false));assert!(c.claim(5.,(0,11),[0.;3],true));assert!(!c.claim(6.,(1,33),[0.;3],true));for _ in 0..100{assert!(!c.done(5.));}assert!(!c.claim(65.,(1,22),[0.;3],true));assert!(c.done(65.));c.retain(&[(0,11),(1,33)]);assert!(c.done(6.));}
 #[test]fn additive_totals_are_individual_and_retire_departed_identity(){let mut c=Collection::default();c.grant((0,11),Color::Red);c.grant((0,11),Color::Red);c.grant((1,22),Color::Green);assert_eq!(c.people[&(0,11)].inputs(),vec![(0,0.2)]);assert_eq!(c.people[&(1,22)].inputs(),vec![(2,0.15)]);c.retain(&[(0,33),(1,22)]);assert!(!c.people.contains_key(&(0,11)));}
 #[test]fn free_draws_and_nonperiodic_budget_allow_repetitions(){let mut counts=[0;4];let mut repeated=false;for seed in 0..1000{let mut d=Director::new(seed);assert!((4..=6).contains(&d.times.len()));assert!(d.times.iter().all(|v|(30.0..=870.0).contains(v)));let a=d.draw(4);let b=d.draw(4);repeated|=a==b;counts[a as usize]+=1;let old=d.cursor;assert!(!d.due(899.,false,2));assert_eq!(d.cursor,old);}assert!(repeated);assert!(counts.iter().all(|n|*n>150));}
 #[test]fn solo_completes_immediately_and_no_unbounded_appearance_backlog(){let mut c=Crystal::new(1,Color::Yellow,[0.;3],&[(0,1)]).unwrap();assert!(c.claim(2.,(0,1),[0.;3],true));assert!(c.done(2.));let mut d=Director::new(1);assert!(!d.due(901.,false,0));assert!(!d.due(901.,true,0));assert!(!d.due(901.,true,0));assert!(d.due(1200.,true,0));}
}
