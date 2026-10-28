//! Ranked, run-local ring enchantments. No native offsets or equipment mutations.
use std::collections::BTreeMap;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum Enchantment { Might, Alacrity, Fortitude, Precision, Ferocity, Warding, Renewal, Reach }
impl Enchantment {
 pub const ALL:[Self;8]=[Self::Might,Self::Alacrity,Self::Fortitude,Self::Precision,Self::Ferocity,Self::Warding,Self::Renewal,Self::Reach];
 pub fn name(self)->&'static str {match self{Self::Might=>"Might",Self::Alacrity=>"Alacrity",Self::Fortitude=>"Fortitude",Self::Precision=>"Precision",Self::Ferocity=>"Ferocity",Self::Warding=>"Warding",Self::Renewal=>"Renewal",Self::Reach=>"Reach"}}

 pub fn at_ring_rank(self,ring_rank:u32)->Option<RankedEnchantment>{Some(RankedEnchantment{kind:self,rank:Rank::from_ring_rank(ring_rank)?})}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]#[repr(u8)]
pub enum Rank{Minor,Lesser,Regular,Greater,Superior}
impl Rank{
 pub const ALL:[Self;5]=[Self::Minor,Self::Lesser,Self::Regular,Self::Greater,Self::Superior];
 pub fn from_ring_rank(n:u32)->Option<Self>{match n{0..=1=>None,2..=3=>Some(Self::Minor),4..=6=>Some(Self::Lesser),7..=9=>Some(Self::Regular),10..=13=>Some(Self::Greater),_=>Some(Self::Superior)}}
 pub fn name(self)->&'static str{["Minor","Lesser","","Greater","Superior"][self as usize]}
 pub fn icon(self)->&'static str{["INV_Enchant_DustStrange","INV_Enchant_EssenceMagicSmall","INV_Enchant_ShardGlimmeringSmall","INV_Enchant_ShardGlimmeringLarge","INV_Enchant_ShardNexusLarge"][self as usize]}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct RankedEnchantment{pub kind:Enchantment,pub rank:Rank}
impl RankedEnchantment{
 pub fn name(self)->String{if self.rank==Rank::Regular{self.kind.name().to_owned()}else{format!("{} {}",self.rank.name(),self.kind.name())}}
 pub fn icon(self)->&'static str{self.rank.icon()}
 pub fn value(self)->f32{let values=match self.kind{
  Enchantment::Might|Enchantment::Reach=>[0.05,0.09,0.15,0.22,0.30],Enchantment::Alacrity=>[0.02,0.04,0.07,0.10,0.14],Enchantment::Fortitude=>[0.05,0.10,0.17,0.25,0.35],Enchantment::Precision=>[0.01,0.02,0.04,0.07,0.10],Enchantment::Ferocity=>[0.10,0.20,0.32,0.48,0.65],Enchantment::Warding=>[0.01,0.02,0.035,0.05,0.07],Enchantment::Renewal=>[0.3,0.6,1.2,2.0,3.0]};values[self.rank as usize]}
 pub fn text(self)->String{
  let (value,suffix)=if self.kind==Enchantment::Renewal{(self.value()," health per second")}else{(self.value()*100.,match self.kind{Enchantment::Might=>"% damage",Enchantment::Alacrity=>"% cooldown recovery",Enchantment::Fortitude=>"% maximum health",Enchantment::Precision=>"% critical strike chance",Enchantment::Ferocity=>"% critical strike damage",Enchantment::Warding=>"% damage reduction",Enchantment::Reach=>"% area of effect",_=>unreachable!()})};
  let number=format!("{value:.1}");format!("+{}{suffix}",number.trim_end_matches(".0"))
 }
 pub fn note(self,previous:Option<Self>)->String{
  if previous.is_some_and(|old|old==self){return format!("Active enchantment: {}",self.name())}
  let improving=previous.is_some_and(|old|(old.rank as u8)<(self.rank as u8));
  let text=if improving{format!("{} → {}",previous.unwrap().text(),self.text())}else{self.text()};
  format!("Enchantment: {}\n{text}{}",self.name(),if previous.is_none(){"\nApplies enchantment"}else if improving{"\nImproves enchantment"}else{""})
 }

}
#[derive(Default)]pub struct Enchantments {locked:BTreeMap<(u32,u64,u8),Enchantment>}
impl Enchantments {
 pub fn get(&self,slot:u32,guid:u64,ring:u8)->Option<Enchantment>{self.locked.get(&(slot,guid,ring)).copied()}
 pub fn accept(&mut self,slot:u32,guid:u64,ring:u8,before:u32,after:u32,e:Enchantment)->bool{
  if slot>=5||guid==0||ring>=9||before==0||before.checked_add(1)!=Some(after){return false}
  match self.locked.entry((slot,guid,ring)){std::collections::btree_map::Entry::Vacant(v)=>{v.insert(e);true},_=>false}
 }
 pub fn retain(&mut self,ids:&[(u32,u64)]){self.locked.retain(|(s,g,_),_|ids.contains(&(*s,*g)));}
 pub fn bonuses(&self,slot:u32,guid:u64)->Vec<(u8,Enchantment)>{self.locked.iter().filter_map(|((s,g,r),e)|((*s,*g)==(slot,guid)).then_some((*r,*e))).collect()}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn thresholds_cap_and_preview_status(){
  for (rank, first, last) in [(Rank::Minor,2,3),(Rank::Lesser,4,6),(Rank::Regular,7,9),(Rank::Greater,10,13),(Rank::Superior,14,20)] {for n in first..=last {assert_eq!(Rank::from_ring_rank(n),Some(rank));}}
  assert_eq!(Rank::from_ring_rank(0),None);assert_eq!(Rank::from_ring_rank(1),None);
  assert_eq!(Rank::from_ring_rank(u32::MAX),Some(Rank::Superior));
  assert_eq!(Enchantment::Might.at_ring_rank(7).unwrap().name(),"Might");
  let arathi8=Enchantment::Might.at_ring_rank(8).unwrap();let arathi10=Enchantment::Might.at_ring_rank(10).unwrap();
  assert_eq!(arathi8.value(),0.15);assert_eq!(arathi10.value(),0.22);assert_eq!(arathi10.name(),"Greater Might");assert!(arathi10.note(Some(arathi8)).contains("+15% damage → +22% damage"));
  let minor=Enchantment::Might.at_ring_rank(2).unwrap();let lesser=Enchantment::Might.at_ring_rank(4).unwrap();
  assert!(minor.note(None).ends_with("Applies enchantment"));assert!(!minor.note(Some(minor)).contains("Applies"));assert!(lesser.note(Some(minor)).ends_with("Improves enchantment"));assert!(lesser.note(Some(minor)).contains("+5% damage → +9% damage"));assert!(!lesser.note(Some(lesser)).contains("Improves"));
 }
 #[test]fn lock_remains_immutable_and_two_rings_progress_independently(){
  let mut s=Enchantments::default();assert!(!s.accept(0,10,0,0,1,Enchantment::Might));assert!(s.accept(0,10,0,1,2,Enchantment::Might));assert!(!s.accept(0,10,0,3,4,Enchantment::Warding));assert_eq!(s.get(0,10,0),Some(Enchantment::Might));
  assert!(s.accept(0,10,1,1,2,Enchantment::Might));assert_eq!(s.bonuses(0,10).len(),2);
  let total=s.bonuses(0,10).iter().map(|(ring,e)|e.at_ring_rank(if *ring==0{14}else{2}).unwrap().value()).sum::<f32>();assert!((total-0.35).abs()<1e-6);
  s.retain(&[(0,20)]);assert!(s.bonuses(0,10).is_empty());
 }
 #[test]fn all_values_and_rank_icons(){
  let wanted=[[0.05,0.09,0.15,0.22,0.30],[0.02,0.04,0.07,0.10,0.14],[0.05,0.10,0.17,0.25,0.35],[0.01,0.02,0.04,0.07,0.10],[0.10,0.20,0.32,0.48,0.65],[0.01,0.02,0.035,0.05,0.07],[0.3,0.6,1.2,2.,3.],[0.05,0.09,0.15,0.22,0.30]];
  for(e,values)in Enchantment::ALL.into_iter().zip(wanted){for(rank,value)in Rank::ALL.into_iter().zip(values){let e=RankedEnchantment{kind:e,rank};assert_eq!(e.value(),value);assert!(e.icon().contains("Enchant"));assert!(e.name().starts_with(rank.name()));}}
 }
}
