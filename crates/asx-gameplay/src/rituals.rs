//! Solo ritual policy. Shared presence can supply `inside` in a later coop integration.
use crate::blessings::Blessing;
pub const RADIUS:f32=8.;
pub const HOLD_SECONDS:f32=20.;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Outcome {Pending,Success,Expired}
#[derive(Clone,Debug)]pub struct Ritual {pub encounter:u32,pub id:u32,pub position:[f32;3],pub created:f32,pub held:f32,last:f32,was_inside:bool,resolved:bool}
impl Ritual {
 pub fn new(encounter:u32,now:f32,position:[f32;3])->Option<Self>{if encounter==0||!now.is_finite()||now<0.||position.iter().any(|v|!v.is_finite()){return None}Some(Self{encounter,id:encounter,position,created:now,held:0.,last:now,was_inside:false,resolved:false})}
 /// Later bosses improve this one opportunity without replacing its scene or
 /// resetting earned presence. The first encounter remains its stable identity.
 pub fn inside(&self)->bool{self.was_inside&&!self.resolved}
 pub fn promote(&mut self,encounter:u32){if !self.resolved{self.encounter=self.encounter.max(encounter);}}
 pub fn advance(&mut self,now:f32,position:[f32;3],playing:bool)->Outcome{
  if self.resolved||!now.is_finite()||now<self.last{return Outcome::Pending}
  let inside=playing&&position.iter().all(|v|v.is_finite())&&(position[0]-self.position[0]).hypot(position[1]-self.position[1])<=RADIUS;
  self.advance_presence(now,inside)
 }
 pub fn advance_presence(&mut self,now:f32,inside:bool)->Outcome{
  if self.resolved||!now.is_finite()||now<self.last{return Outcome::Pending}
  let dt=(now-self.last).min(0.5);self.last=now;
  if inside&&self.was_inside{self.held=(self.held+dt).min(HOLD_SECONDS);}
  self.was_inside=inside;
  if self.held>=HOLD_SECONDS{self.resolved=true;Outcome::Success}else{Outcome::Pending}
 }
}
/// A stage gives a range, not a fixed award. Progress is independent of missed rituals.
pub fn roll(encounter:u32,previous:Option<(u32,Blessing)>,category_draw:u32,power_draw:u32)->(u32,Blessing){
 let stage=encounter.clamp(1,5);let(low,high)=[(10,18),(14,24),(18,30),(22,36),(26,40)][stage as usize-1];
 let category=(category_draw%crate::blessings::CATEGORY_COUNT)as u8;let mut low=low;
 if let Some((power,old))=previous {
  low=low.max((power+1).min(high));
  while low<high&&old.category==category&&amount(category,low)<=old.amount{low+=1;}
 }
 let power=low+power_draw%(high-low+1);let rarity=match power{0..=14=>0,15..=21=>1,22..=28=>2,29..=35=>3,_=>4};
 (power,Blessing{category,rarity,amount:amount(category,power),boss:encounter})
}
fn amount(category:u8,power:u32)->u32{(([1500,1200,2500][category as usize]*power/10+50)/100)*100}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn group_presence_requires_every_live_identity_and_keeps_partial_progress(){let mut r=Ritual::new(1,0.,[0.;3]).unwrap();r.advance_presence(0.,true);r.advance_presence(0.5,true);let before=r.held;r.advance_presence(1.,false);r.advance_presence(1.5,true);assert_eq!(r.held,before);r.advance_presence(2.,true);assert_eq!(r.held,before+0.5);}
 #[test]fn partial_presence_pause_and_one_success_only(){
  let mut r=Ritual::new(1,0.,[0.;3]).unwrap();for i in 0..=20{assert_eq!(r.advance(i as f32/2.,[0.;3],true),Outcome::Pending);}
  let held=r.held;for _ in 0..10{r.advance(10.,[0.;3],true);}assert_eq!(r.held,held);
  r.advance(10.5,[20.,0.,0.],true);assert_eq!(r.held,held);assert!(!r.inside());r.advance(11.,[0.;3],true);assert_eq!(r.held,held);assert!(r.inside());
  let mut successes=0;for i in 23..=80{if r.advance(i as f32/2.,[0.;3],true)==Outcome::Success{successes+=1;}}
  assert_eq!(successes,1);
 }
 #[test]fn no_deadline_and_long_unobserved_gaps_do_not_grant(){let mut r=Ritual::new(1,0.,[0.;3]).unwrap();r.advance(1.,[0.;3],true);r.advance(20.,[0.;3],true);assert!(r.held<=0.5);assert_eq!(r.advance(3600.,[20.,0.,0.],true),Outcome::Pending);assert!(r.held<=0.5);}
 #[test]fn later_bosses_keep_identity_position_and_partial_progress(){
  let mut r=Ritual::new(1,10.,[1.,2.,3.]).unwrap();r.advance(10.,[1.,2.,3.],true);r.advance(10.5,[1.,2.,3.],true);let held=r.held;
  r.promote(4);r.promote(2);assert_eq!((r.id,r.encounter,r.position,r.created,r.held),(1,4,[1.,2.,3.],10.,held));
  assert_eq!(r.advance(500.,[100.;3],true),Outcome::Pending);
  let mut successes=0;for i in 0..=45{if r.advance(500.+i as f32/2.,[1.,2.,3.],true)==Outcome::Success{successes+=1;}}
  assert_eq!(successes,1);assert_eq!(roll(r.encounter,None,0,0).0,22);
 }
 #[test]fn ranges_keep_surprise_and_replace_monotonically_until_cap(){let mut previous=None;for stage in 1..=100 {let(p,b)=roll(stage,previous,stage%crate::blessings::CATEGORY_COUNT,0);if let Some((old,_))=previous{assert!(p>old||p==40);}assert_eq!(b.amount%100,0);assert!(p<=40);previous=Some((p,b));}let(a,_)=roll(1,None,0,0);let(b,_)=roll(1,None,0,8);assert_eq!((a,b),(10,18));}
 #[test]fn skipping_rituals_keeps_map_stage_and_percentage_rounding_never_regresses(){let(p,b)=roll(4,None,2,0);assert_eq!(p,22);let(_,new)=roll(5,Some((p,b)),2,0);assert!(new.amount>b.amount);assert_eq!(roll(1,None,1,0).1.amount,1200);}
}
