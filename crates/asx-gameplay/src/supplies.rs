//! Personal run-local supplies: deterministic catalog, claims, tiers and bounded stacking.
use std::collections::{BTreeMap,BTreeSet};
pub const BUFF_SECONDS:f32=20.;
pub const SUPPLY_SECONDS:f32=60.;
pub const PICKUP_RADIUS:f32=2.;
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]#[repr(u8)]
pub enum Family{Recovery,Critical,Defense,Vitality,Ferocity,Reach}
impl Family{pub const ALL:[Self;6]=[Self::Recovery,Self::Critical,Self::Defense,Self::Vitality,Self::Ferocity,Self::Reach];pub fn stat(self)->u32{[1,3,5,2,4,8][self as usize]}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Consumable{pub family:Family,pub tier:u8}
impl Consumable{
 pub fn new(family:Family,tier:u8)->Option<Self>{(1..=3).contains(&tier).then_some(Self{family,tier})}
 pub fn value(self)->f32{[[0.06,0.10,0.14],[0.04,0.07,0.10],[0.03,0.05,0.07],[0.10,0.20,0.30],[0.20,0.35,0.50],[0.15,0.25,0.35]][self.family as usize][self.tier as usize-1]}
 pub fn name(self)->&'static str{[
  ["Elixir of Wisdom","Elixir of Greater Intellect","Flask of Distilled Wisdom"],
  ["Elixir of Lesser Agility","Elixir of Greater Agility","Elixir of the Mongoose"],
  ["Elixir of Defense","Elixir of Greater Defense","Elixir of Superior Defense"],
  ["Elixir of Fortitude","Elixir of Brute Force","Flask of the Titans"],
  ["Minor Wizard Oil","Wizard Oil","Brilliant Wizard Oil"],
  ["Elixir of Giant Growth","Elixir of Giant Growth","Elixir of Giant Growth"]
 ][self.family as usize][self.tier as usize-1]}
 pub fn icon(self)->&'static str{[
  ["INV_Potion_06","INV_Potion_10","INV_Potion_97"],
  ["INV_Potion_92","INV_Potion_94","INV_Potion_32"],
  ["INV_Potion_64","INV_Potion_65","INV_Potion_66"],
  ["INV_Potion_43","INV_Potion_40","INV_Potion_62"],
  ["INV_Potion_15","INV_Potion_23","INV_Potion_105"],
  ["INV_Potion_10","INV_Potion_10","INV_Potion_10"]
 ][self.family as usize][self.tier as usize-1]}
 pub fn text(self)->String{format!("+{}% {}",(self.value()*100.).round()as u32,["cooldown recovery","critical strike chance","damage reduction","maximum health","critical strike damage","area of effect"][self.family as usize])}
}
pub fn tier(bosses:u32)->u8{match bosses{0..=1=>1,2..=3=>2,_=>3}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Reward{Buff(Consumable),Material{rarity:u8,variant:u8}}
impl Reward{
 /// 90% consumables; remaining 10% materials with conditional 60/30/9/1 rarities.
 pub fn roll(bosses:u32,kind:u32,rarity:u32,variant:u32)->Self{if kind%100<90{Self::Buff(Consumable{family:Family::ALL[(variant%6)as usize],tier:tier(bosses)})}else{Self::Material{rarity:match rarity%100{0..=59=>0,60..=89=>1,90..=98=>2,_=>3},variant:(variant%3)as u8}}}
 pub fn name(self)->&'static str{match self{Self::Buff(c)=>c.name(),Self::Material{rarity,variant}=>[
 ["Bundle of Peacebloom","Copper Ore Shipment","Linen Cloth Bundle"],
 ["Bundle of Goldthorn","Mithril Ore Shipment","Mageweave Cloth Bundle"],
 ["Bundle of Dreamfoil","Thorium Ore Shipment","Runecloth Bundle"],
 ["Black Lotus","Arcane Crystal","Arcanite Bar"]][rarity as usize][variant as usize]}}
 pub fn icon(self)->&'static str{match self{Self::Buff(c)=>c.icon(),Self::Material{rarity,variant}=>[
 ["INV_Misc_Herb_02","INV_Ore_Copper_01","INV_Fabric_Linen_01"],
 ["INV_Misc_Herb_14","INV_Ore_Mithril_02","INV_Fabric_Mageweave_01"],
 ["INV_Misc_Herb_DreamFoil","INV_Ore_Thorium_02","INV_Fabric_PurpleFire_01"],
 ["INV_Misc_Herb_BlackLotus","INV_Misc_Gem_Crystal_02","INV_Ingot_03"]][rarity as usize][variant as usize]}}
 pub fn rarity(self)->u8{match self{Self::Buff(c)=>c.tier-1,Self::Material{rarity,..}=>rarity}}
 pub fn text(self)->String{match self{Self::Buff(c)=>c.text(),_=>"Automatically sold for gold".into()}}
 pub fn nominal_gold(self,bosses:u32)->u32{match self{Self::Buff(_)=>0,Self::Material{rarity,..}=>{
  // Normalized rarity mean=1.64. Faster generation keeps the original expected gold/minute.
  let n=10*(1+bosses.min(5))*200*[1,2,4,8][rarity as usize];(n+246)/492
 }}}
}
#[derive(Clone,Copy,Debug,PartialEq)]pub struct ActiveBuff{pub consumable:Consumable,pub expires:f32}
#[derive(Clone,Copy,Debug)]pub struct Discovery{pub event:u64,pub reward:Reward,pub gold:u32}
#[derive(Clone,Debug)]pub struct Supply{pub event:u64,pub position:[f32;3],pub expires:f32,pub reward:Reward,pub gold:u32,eligible:BTreeSet<(u32,u64)>,claimed:BTreeSet<(u32,u64)>,personal:BTreeMap<(u32,u64),(Reward,u32)>}
impl Supply{
 pub fn new(event:u64,now:f32,position:[f32;3],actors:&[(u32,u64)],reward:Reward,bosses:u32)->Option<Self>{
  if event==0||!now.is_finite()||now<0.||position.iter().any(|v|!v.is_finite())||actors.is_empty()||actors.len()>5||actors.iter().any(|(s,g)|*s>=5||*g==0){return None}
  Some(Self{event,position,expires:now+SUPPLY_SECONDS,reward,gold:reward.nominal_gold(bosses),eligible:actors.iter().copied().collect(),claimed:BTreeSet::new(),personal:BTreeMap::new()})
 }
 pub fn personal_rewards(&mut self,rewards:&[((u32,u64),Reward)],bosses:u32)->bool{
  if rewards.len()!=self.eligible.len()||rewards.iter().any(|(id,_)|!self.eligible.contains(id)){return false}
  let map=rewards.iter().map(|(id,r)|(*id,(*r,r.nominal_gold(bosses)))).collect::<BTreeMap<_,_>>();
  if map.len()!=self.eligible.len(){return false}self.personal=map;true
 }
 pub fn available(&self,actor:(u32,u64))->bool{self.eligible.contains(&actor)&&!self.claimed.contains(&actor)}
 pub fn retain(&mut self,actors:&[(u32,u64)]){self.eligible.retain(|id|actors.contains(id));self.personal.retain(|id,_|self.eligible.contains(id));}
 pub fn claim(&mut self,now:f32,actor:(u32,u64),position:[f32;3],alive:bool)->Option<Discovery>{
  if !now.is_finite()||now>=self.expires||!alive||!self.eligible.contains(&actor)||self.claimed.contains(&actor)||position.iter().any(|v|!v.is_finite())||(position[0]-self.position[0]).hypot(position[1]-self.position[1])>PICKUP_RADIUS{return None}
  self.claimed.insert(actor);let(reward,gold)=self.personal.get(&actor).copied().unwrap_or((self.reward,self.gold));Some(Discovery{event:self.event,reward,gold})
 }
 pub fn complete(&self)->bool{self.eligible.is_subset(&self.claimed)}
}
#[derive(Default,Clone)]pub struct Buffs(BTreeMap<Family,ActiveBuff>);
impl Buffs{
 pub fn accept(&mut self,now:f32,reward:Reward)->bool{let Reward::Buff(mut c)=reward else{return false};if !now.is_finite()||now<0.{return false}
  if let Some(old)=self.0.get(&c.family).filter(|b|b.expires>now){if old.consumable.tier>c.tier{c=old.consumable;}}
  self.0.insert(c.family,ActiveBuff{consumable:c,expires:now+BUFF_SECONDS});true
 }
 pub fn expire(&mut self,now:f32)->bool{let len=self.0.len();if now.is_finite(){self.0.retain(|_,v|now<v.expires);}self.0.len()!=len}
 pub fn clear(&mut self)->bool{let changed=!self.0.is_empty();self.0.clear();changed}
 pub fn values(&self)->Vec<ActiveBuff>{self.0.values().copied().collect()}
 pub fn is_empty(&self)->bool{self.0.is_empty()}
}
#[cfg(test)]mod tests{
 use super::*;
 fn buff(f:Family,t:u8)->Reward{Reward::Buff(Consumable::new(f,t).unwrap())}
 #[test]fn personal_rolls_cannot_be_farmed_by_rejoining_or_reveal_a_shared_reward(){let mut s=Supply::new(1,0.,[0.;3],&[(0,10),(1,20)],buff(Family::Critical,1),0).unwrap();assert!(s.personal_rewards(&[((0,10),buff(Family::Critical,1)),((1,20),buff(Family::Defense,2))],0));assert_eq!(s.claim(1.,(0,10),[0.;3],true).unwrap().reward,buff(Family::Critical,1));assert!(!s.available((0,10)));assert!(s.available((1,20)));s.retain(&[(0,10)]);assert!(s.complete());assert!(s.claim(2.,(1,21),[0.;3],true).is_none());}
 #[test]fn claims_are_fixed_at_spawn_and_bound_to_identity(){let mut s=Supply::new(1,0.,[0.;3],&[(0,10),(1,20)],buff(Family::Critical,1),0).unwrap();assert!(s.claim(1.,(0,10),[0.;3],true).is_some());assert!(s.claim(2.,(0,10),[0.;3],true).is_none());assert!(s.claim(2.,(1,21),[0.;3],true).is_none());assert!(s.claim(2.,(1,20),[0.;3],true).is_some());assert!(s.complete());}
 #[test]fn sixty_second_deadline_death_and_distance_do_not_consume_claim(){let mut s=Supply::new(1,0.,[0.;3],&[(0,10)],buff(Family::Recovery,1),0).unwrap();assert!(s.claim(1.,(0,10),[3.,0.,0.],true).is_none());assert!(s.claim(1.,(0,10),[0.;3],false).is_none());assert!(s.claim(60.,(0,10),[0.;3],true).is_none());assert!(s.claim(59.9,(0,10),[0.;3],true).is_some());}
 #[test]fn families_coexist_without_repeated_stacking_or_downgrades(){let mut b=Buffs::default();b.accept(0.,buff(Family::Critical,3));b.accept(1.,buff(Family::Defense,2));b.accept(2.,buff(Family::Critical,1));assert_eq!(b.values().len(),2);assert_eq!(b.0[&Family::Critical].consumable.tier,3);assert_eq!(b.0[&Family::Critical].expires,22.);assert!(!b.expire(2.));assert!(b.expire(21.));assert_eq!(b.values().len(),1);assert!(b.expire(22.));assert!(b.is_empty());}
 #[test]fn tier_boundaries_and_endless_cap(){assert_eq!((tier(0),tier(1),tier(2),tier(3),tier(4),tier(999)),(1,1,2,2,3,3));}
 #[test]fn faster_supplies_preserve_expected_material_gold_budget(){for boss in 0..=5{let sum:f64=(0..100).map(|r|Reward::roll(boss,99,r,0).nominal_gold(boss)as f64).sum();let new_per_min=0.75*0.10*sum/100.;let old_per_min=0.2*0.25*(10*(1+boss))as f64;assert!((new_per_min-old_per_min).abs()<0.04);}}
}
