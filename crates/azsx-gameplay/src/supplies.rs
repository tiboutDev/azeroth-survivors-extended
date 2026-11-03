//! Personal run-local supplies: deterministic catalog, claims, tiers and bounded stacking.
use std::collections::{BTreeMap,BTreeSet};
pub const MAX_BUFF_SECONDS:f32=120.;
pub const SUPPLY_SECONDS:f32=90.;
pub const PICKUP_RADIUS:f32=2.;
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]#[repr(u8)]
pub enum Family{Recovery,Critical,Defense,Vitality,Ferocity,Reach,Swiftness,Healing}
struct Item {name:&'static str,icon:&'static str,value:f32,seconds:Option<u16>}
const fn timed(name:&'static str,icon:&'static str,value:f32,seconds:u16)->Item{
 Item{name,icon,value,seconds:Some(seconds)}
}
const fn potion(name:&'static str,icon:&'static str,value:f32)->Item{
 Item{name,icon,value,seconds:None}
}
impl Family{
 pub const ALL:[Self;8]=[Self::Recovery,Self::Critical,Self::Defense,Self::Vitality,Self::Ferocity,Self::Reach,Self::Swiftness,Self::Healing];
 pub fn stat(self)->Option<u32>{[Some(1),Some(3),Some(5),Some(2),Some(4),Some(8),None,None][self as usize]}
 fn catalog(self)->&'static [Item]{
  match self{
   Self::Recovery=>const { &[timed("Elixir of Wisdom","INV_Potion_06",0.08,60),timed("Elixir of Greater Intellect","INV_Potion_10",0.12,75),timed("Flask of Distilled Wisdom","INV_Potion_97",0.16,120)] },
   Self::Critical=>const { &[timed("Elixir of Lesser Agility","INV_Potion_92",0.04,60),timed("Elixir of Greater Agility","INV_Potion_94",0.07,75),timed("Elixir of the Mongoose","INV_Potion_32",0.10,90)] },
   Self::Defense=>const { &[timed("Elixir of Defense","INV_Potion_64",0.03,60),timed("Elixir of Greater Defense","INV_Potion_65",0.05,75),timed("Elixir of Superior Defense","INV_Potion_66",0.07,90)] },
   Self::Vitality=>const { &[timed("Elixir of Fortitude","INV_Potion_43",0.20,60),timed("Elixir of Brute Force","INV_Potion_40",0.30,75),timed("Flask of the Titans","INV_Potion_62",0.40,120)] },
   Self::Ferocity=>const { &[timed("Minor Wizard Oil","INV_Poison_MindNumbing",0.20,60),timed("Wizard Oil","INV_Potion_104",0.35,75),timed("Brilliant Wizard Oil","INV_Potion_105",0.50,90)] },
   Self::Reach=>const { &[timed("Elixir of Giant Growth","INV_Potion_10",0.25,60)] },
   Self::Swiftness=>const { &[timed("Swiftness Potion","INV_Potion_95",0.20,15)] },
   Self::Healing=>const { &[potion("Healing Potion","INV_Potion_51",0.20),potion("Superior Healing Potion","INV_Potion_53",0.30),potion("Major Healing Potion","INV_Potion_54",0.40)] },
  }
 }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Consumable{pub family:Family,pub tier:u8}
impl Consumable{
 pub fn new(family:Family,tier:u8)->Option<Self>{
  (1..=3).contains(&tier).then(||Self{family,tier:tier.min(family.catalog().len()as u8)})
 }
 fn item(self)->&'static Item{&self.family.catalog()[self.tier as usize-1]}
 pub fn value(self)->f32{self.item().value}
 pub fn name(self)->&'static str{self.item().name}
 pub fn icon(self)->&'static str{self.item().icon}
 pub fn seconds(self)->Option<u16>{self.item().seconds}
 pub fn wire_seconds(self,remaining:f32)->u32{if self.seconds().is_some(){remaining.ceil()as u32}else{0}}
 pub fn text(self)->String{
  let percent=(self.value()*100.).round()as u32;
  if self.family==Family::Healing{return format!("Automatically restores {percent}% of maximum health at or below 35% health. One charge.")}
  format!("+{percent}% {}",["cooldown recovery","critical strike chance","damage reduction","maximum health","critical strike damage","area of effect","movement speed"][self.family as usize])
 }
}
pub fn potion_health(hp:f32,maximum:f32,potion:Consumable)->Option<f32>{
 if potion.family!=Family::Healing || !hp.is_finite() || !maximum.is_finite() || maximum<=0. || hp<=0. || hp>maximum*0.35{return None}
 Some((hp+maximum*potion.value()).min(maximum))
}
pub fn tier(bosses:u32)->u8{match bosses{0..=1=>1,2..=3=>2,_=>3}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Reward{Buff(Consumable),Material{rarity:u8,variant:u8}}
impl Reward{
 /// 90% consumables; remaining 10% materials with conditional 60/30/9/1 rarities.
 pub fn roll(bosses:u32,kind:u32,rarity:u32,variant:u32)->Self{if kind%100<90{Self::Buff(Consumable::new(Family::ALL[(variant%Family::ALL.len()as u32)as usize],tier(bosses)).unwrap())}else{Self::Material{rarity:match rarity%100{0..=59=>0,60..=89=>1,90..=98=>2,_=>3},variant:(variant%3)as u8}}}
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
 /// Classic reference item quality, independent of AZSX reward tier/power.
 pub fn quality(self)->u8{match self{Self::Buff(_)=>1,Self::Material{rarity:3,..}=>2,Self::Material{..}=>1}}
 pub fn rarity(self)->u8{match self{Self::Buff(c)=>c.tier-1,Self::Material{rarity,..}=>rarity}}
 pub fn text(self)->String{match self{Self::Buff(c)=>match c.seconds(){Some(seconds)=>format!("{} for {seconds} sec",c.text()),None=>c.text()},_=>"Automatically sold for gold".into()}}
 pub fn nominal_gold(self,bosses:u32)->u32{match self{Self::Buff(_)=>0,Self::Material{rarity,..}=>{
  // Normalized rarity mean=1.64. Faster generation keeps the original expected gold/minute.
  let n=10*(1+bosses.min(5))*200*[1,2,4,8][rarity as usize];(n+246)/492
 }}}
}
#[derive(Clone,Copy,Debug,PartialEq)]pub struct ActiveBuff{pub consumable:Consumable,pub expires:f32}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Costume { Skeleton, Small, Pirate, Ninja }

impl Costume {
 pub const SECONDS:f32=60.;
 pub fn roll(mut draw:impl FnMut(u32)->u32)->Option<Self>{
  if draw(100)>=8 {return None}
  Some(match draw(4){0=>Self::Skeleton,1=>Self::Small,2=>Self::Pirate,_=>Self::Ninja})
 }
 pub fn item(self)->&'static str{match self{Self::Skeleton|Self::Small=>"Noggenfogger Elixir",Self::Pirate|Self::Ninja=>"Savory Deviate Delight"}}
 pub fn label(self)->&'static str{match self{Self::Skeleton=>"Skeleton",Self::Small=>"Miniature",Self::Pirate=>"Pirate",Self::Ninja=>"Ninja"}}
}
#[derive(Clone,Copy,Debug)]pub struct Discovery{pub event:u64,pub reward:Reward,pub gold:u32,pub bonus:Option<Costume>}
impl Discovery{
 pub fn text(self)->String{let text=self.reward.text();match self.bonus{Some(b)=>format!("{text} + {} ({} for 60 sec)",b.item(),b.label()),None=>text}}
}
#[derive(Clone,Debug)]pub struct Supply{pub event:u64,pub position:[f32;3],pub expires:f32,pub reward:Reward,pub gold:u32,eligible:BTreeSet<(u32,u64)>,claimed:BTreeSet<(u32,u64)>,bonuses:BTreeMap<(u32,u64),Costume>,personal:BTreeMap<(u32,u64),(Reward,u32)>}
impl Supply{
 pub fn new(event:u64,now:f32,position:[f32;3],actors:&[(u32,u64)],reward:Reward,bosses:u32)->Option<Self>{
  if event==0||!now.is_finite()||now<0.||position.iter().any(|v|!v.is_finite())||actors.is_empty()||actors.len()>5||actors.iter().any(|(s,g)|*s>=5||*g==0){return None}
  Some(Self{event,position,expires:now+SUPPLY_SECONDS,reward,gold:reward.nominal_gold(bosses),eligible:actors.iter().copied().collect(),claimed:BTreeSet::new(),personal:BTreeMap::new(),bonuses:BTreeMap::new()})
 }
 pub fn roll_bonuses(&mut self,mut roll:impl FnMut()->Option<Costume>){self.bonuses=self.eligible.iter().filter_map(|id|Some((*id,roll()?))).collect();}
 pub fn personal_rewards(&mut self,rewards:&[((u32,u64),Reward)],bosses:u32)->bool{
  if rewards.len()!=self.eligible.len()||rewards.iter().any(|(id,_)|!self.eligible.contains(id)){return false}
  let map=rewards.iter().map(|(id,r)|(*id,(*r,r.nominal_gold(bosses)))).collect::<BTreeMap<_,_>>();
  if map.len()!=self.eligible.len(){return false}self.personal=map;true
 }
 pub fn available(&self,actor:(u32,u64))->bool{self.eligible.contains(&actor)&&!self.claimed.contains(&actor)}
 pub fn retain(&mut self,actors:&[(u32,u64)]){self.eligible.retain(|id|actors.contains(id));self.personal.retain(|id,_|self.eligible.contains(id));}
 pub fn claim(&mut self,now:f32,actor:(u32,u64),position:[f32;3],alive:bool)->Option<Discovery>{
  if !now.is_finite()||now>=self.expires||!alive||!self.eligible.contains(&actor)||self.claimed.contains(&actor)||position.iter().any(|v|!v.is_finite())||(position[0]-self.position[0]).hypot(position[1]-self.position[1])>PICKUP_RADIUS{return None}
  self.claimed.insert(actor);let(reward,gold)=self.personal.get(&actor).copied().unwrap_or((self.reward,self.gold));Some(Discovery{event:self.event,reward,gold,bonus:self.bonuses.get(&actor).copied()})
 }
 pub fn complete(&self)->bool{self.eligible.is_subset(&self.claimed)}
}
#[derive(Default,Clone)]pub struct Buffs(BTreeMap<Family,ActiveBuff>);
impl Buffs{
 pub fn accept(&mut self,now:f32,reward:Reward)->bool{let Reward::Buff(mut c)=reward else{return false};if !now.is_finite()||now<0.{return false}
  if let Some(old)=self.0.get(&c.family).filter(|b|b.expires>now){if old.consumable.tier>c.tier{c=old.consumable;}}
  self.0.insert(c.family,ActiveBuff{consumable:c,expires:c.seconds().map_or(f32::INFINITY,|seconds|now+f32::from(seconds))});true
 }
 pub fn take_potion(&mut self,hp:f32,maximum:f32)->Option<Consumable>{
  let c=self.0.get(&Family::Healing)?.consumable;potion_health(hp,maximum,c)?;self.0.remove(&Family::Healing);Some(c)
 }
 pub fn expire(&mut self,now:f32)->bool{let len=self.0.len();if now.is_finite(){self.0.retain(|_,v|now<v.expires);}self.0.len()!=len}
 pub fn discard_potion(&mut self){self.0.remove(&Family::Healing);}
 pub fn clear(&mut self)->bool{let changed=!self.0.is_empty();self.0.clear();changed}
 pub fn values(&self)->Vec<ActiveBuff>{self.0.values().copied().collect()}
 pub fn is_empty(&self)->bool{self.0.is_empty()}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn balance_tiers_refresh_and_expiry_keep_exact_bonus_budget(){
  for (family,values) in [(Family::Recovery,[0.08,0.12,0.16]),(Family::Vitality,[0.20,0.30,0.40]),(Family::Defense,[0.03,0.05,0.07])]{
   let mut buffs=Buffs::default();
   for tier in 1..=3 {let c=Consumable::new(family,tier).unwrap();buffs.accept(tier as f32,Reward::Buff(c));assert_eq!(buffs.values().len(),1);assert_eq!(buffs.values()[0].consumable.value(),values[tier as usize-1]);}
   buffs.accept(10.,Reward::Buff(Consumable::new(family,1).unwrap()));
   assert_eq!(buffs.values()[0].consumable.value(),values[2]);let end=10.+f32::from(buffs.values()[0].consumable.seconds().unwrap());assert_eq!(buffs.values()[0].expires,end);
   assert!(!buffs.expire(end-1.));assert!(buffs.expire(end));assert!(buffs.is_empty());
  }
 }

 fn buff(f:Family,t:u8)->Reward{Reward::Buff(Consumable::new(f,t).unwrap())}
 #[test]fn speed_and_cosmetic_quality_are_independent_of_gameplay_tiers(){
  for tier in 1..=3{let c=Consumable::new(Family::Swiftness,tier).unwrap();assert_eq!(c.name(),"Swiftness Potion");assert_eq!((c.tier,c.value(),c.seconds()),(1,0.20,Some(15)));assert_eq!(Reward::Buff(c).quality(),1);assert_eq!(c.family.stat(),None);}
  let mut families=BTreeSet::new();for draw in 0..700{if let Reward::Buff(c)=Reward::roll(0,0,0,draw){families.insert(c.family);}}assert_eq!(families.len(),8);
  for family in Family::ALL{for tier in 1..=3{assert_eq!(Reward::Buff(Consumable::new(family,tier).unwrap()).quality(),1);}}
  for rarity in 0..4{for variant in 0..3{assert_eq!(Reward::Material{rarity,variant}.quality(),if rarity==3{2}else{1});}}
 }
 #[test]fn bonus_roll_preserves_the_primary_reward_and_each_claim(){
  let ids=[(0,11),(1,22)];
  let reward=buff(Family::Recovery,1);
  let mut pile=Supply::new(1,0.,[0.;3],&ids,reward,0).unwrap();
  let mut results=[Some(Costume::Skeleton),None].into_iter();
  pile.roll_bonuses(||results.next().unwrap());
  let guest=pile.claim(2.,ids[1],[0.;3],true).unwrap();
  assert_eq!((guest.reward,guest.bonus),(reward,None));
  let host=pile.claim(2.,ids[0],[0.;3],true).unwrap();
  assert_eq!((host.reward,host.bonus),(reward,Some(Costume::Skeleton)));
  assert!(pile.claim(3.,ids[0],[0.;3],true).is_none());
  let mut counts=[0;4];
  for ticket in 0..100 {for variant in 0..4{
   if let Some(effect)=Costume::roll(|n|if n==100{ticket}else{variant}){counts[effect as usize]+=1;}
  }}
  assert_eq!(counts,[8;4]);
 }
 #[test]fn one_personal_charge_retains_the_strongest_and_fires_once_at_threshold(){
  let mut b=Buffs::default();
  for t in [2,1]{assert!(b.accept(0.,buff(Family::Healing,t)));}
  assert!(!b.expire(1_000.));assert!(b.take_potion(35.01,100.).is_none());
  assert!(b.take_potion(0.,100.).is_none());
  let c=b.take_potion(35.,100.).unwrap();assert_eq!(c.tier,2);
  assert_eq!(potion_health(35.,100.,c),Some(65.));assert!(b.take_potion(20.,100.).is_none());
  for (t,amount) in [(1,20.),(2,30.),(3,40.)]{let c=Consumable::new(Family::Healing,t).unwrap();assert_eq!(potion_health(10.,100.,c),Some(10.+amount));}
  b.accept(0.,buff(Family::Healing,1));b.discard_potion();assert!(b.is_empty());
 }
 #[test]fn catalog_only_assigns_distinct_real_upgrades(){
  for family in Family::ALL {
   let items=family.catalog();
   for boss in [0,2,4,10] {
    let c=Consumable::new(family,tier(boss)).unwrap();
    assert!(usize::from(c.tier)<=items.len());
    assert_eq!(items.iter().filter(|i|i.name==c.name()).count(),1);
   }
  }
  assert_eq!(Consumable::new(Family::Reach,3).unwrap().tier,1);
 }
 #[test]fn personal_rolls_cannot_be_farmed_by_rejoining_or_reveal_a_shared_reward(){let mut s=Supply::new(1,0.,[0.;3],&[(0,10),(1,20)],buff(Family::Critical,1),0).unwrap();assert!(s.personal_rewards(&[((0,10),buff(Family::Critical,1)),((1,20),buff(Family::Defense,2))],0));assert_eq!(s.claim(1.,(0,10),[0.;3],true).unwrap().reward,buff(Family::Critical,1));assert!(!s.available((0,10)));assert!(s.available((1,20)));s.retain(&[(0,10)]);assert!(s.complete());assert!(s.claim(2.,(1,21),[0.;3],true).is_none());}
 #[test]fn claims_are_fixed_at_spawn_and_bound_to_identity(){let mut s=Supply::new(1,0.,[0.;3],&[(0,10),(1,20)],buff(Family::Critical,1),0).unwrap();assert!(s.claim(1.,(0,10),[0.;3],true).is_some());assert!(s.claim(2.,(0,10),[0.;3],true).is_none());assert!(s.claim(2.,(1,21),[0.;3],true).is_none());assert!(s.claim(2.,(1,20),[0.;3],true).is_some());assert!(s.complete());}
 #[test]fn ninety_second_deadline_death_and_distance_do_not_consume_claim(){let mut s=Supply::new(1,0.,[0.;3],&[(0,10)],buff(Family::Recovery,1),0).unwrap();assert!(s.claim(1.,(0,10),[3.,0.,0.],true).is_none());assert!(s.claim(1.,(0,10),[0.;3],false).is_none());assert!(s.claim(90.,(0,10),[0.;3],true).is_none());assert!(s.claim(89.9,(0,10),[0.;3],true).is_some());}
 #[test]fn families_coexist_without_repeated_stacking_or_downgrades(){let mut b=Buffs::default();b.accept(0.,buff(Family::Critical,3));b.accept(1.,buff(Family::Defense,2));b.accept(2.,buff(Family::Critical,1));assert_eq!(b.values().len(),2);assert_eq!(b.0[&Family::Critical].consumable.tier,3);assert_eq!(b.0[&Family::Critical].expires,92.);assert!(!b.expire(2.));assert!(b.expire(76.));assert_eq!(b.values().len(),1);assert!(b.expire(92.));assert!(b.is_empty());}
 #[test]fn tier_boundaries_and_endless_cap(){assert_eq!((tier(0),tier(1),tier(2),tier(3),tier(4),tier(999)),(1,1,2,2,3,3));}
 #[test]fn faster_supplies_preserve_expected_material_gold_budget(){for boss in 0..=5{let sum:f64=(0..100).map(|r|Reward::roll(boss,99,r,0).nominal_gold(boss)as f64).sum();let new_per_min=0.75*0.10*sum/100.;let old_per_min=0.2*0.25*(10*(1+boss))as f64;assert!((new_per_min-old_per_min).abs()<0.04);}}
}
