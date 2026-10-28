//! Fixed, run-local amulet gems. No client layouts or permanent progression.
use std::collections::BTreeMap;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum Gem { Ruby,Sapphire,Bloodstone,Topaz,Amethyst,Diamond,Emerald,Opal }
impl Gem {
 pub const ALL:[Self;8]=[Self::Ruby,Self::Sapphire,Self::Bloodstone,Self::Topaz,Self::Amethyst,Self::Diamond,Self::Emerald,Self::Opal];
 pub fn roll(draw:u32)->Self{Self::ALL[(draw%8)as usize]}
 pub fn name(self)->&'static str{["Bold Ruby","Quick Sapphire","Solid Bloodstone","Deadly Topaz","Potent Amethyst","Enduring Diamond","Renewing Emerald","Expansive Opal"][self as usize]}
 pub fn icon(self)->&'static str{["INV_Misc_Gem_Ruby_03","INV_Misc_Gem_Sapphire_03","INV_Misc_Gem_Bloodstone_03","INV_Misc_Gem_Topaz_03","INV_Misc_Gem_Amethyst_03","INV_Misc_Gem_Diamond_03","INV_Misc_Gem_Emerald_03","INV_Misc_Gem_Opal_03"][self as usize]}
 pub fn value(self)->f32{[0.12,0.06,0.20,0.05,0.35,0.03,1.0,0.20][self as usize]}
 pub fn text(self)->&'static str{["+12% damage","+6% cooldown recovery","+20% maximum health","+5% critical strike chance","+35% critical strike damage","+3% damage reduction","+1 health per second","+20% area of effect"][self as usize]}
 pub fn note(self,first:bool)->String{if !first{return format!("Active gem: {}",self.name())}format!("Gem: {}\n{}\n{}",self.name(),self.text(),if first{"Sets your gem for this run"}else{"Gem retained"})}
}
#[derive(Default)]pub struct Gems {locked:BTreeMap<(u32,u64),Gem>}
impl Gems {
 pub fn get(&self,slot:u32,guid:u64)->Option<Gem>{self.locked.get(&(slot,guid)).copied()}
 pub fn accept(&mut self,slot:u32,guid:u64,changed:bool,gem:Gem)->bool{
  if slot>=5||guid==0||!changed{return false}
  match self.locked.entry((slot,guid)){std::collections::btree_map::Entry::Vacant(v)=>{v.insert(gem);true},_=>false}
 }
 pub fn retain(&mut self,ids:&[(u32,u64)]){self.locked.retain(|id,_|ids.contains(id));}
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn catalogue_fixed_values_and_equal_draws(){let mut counts=[0;8];for n in 0..800{counts[Gem::roll(n)as usize]+=1;}assert_eq!(counts,[100;8]);for(g,v)in Gem::ALL.into_iter().zip([0.12,0.06,0.20,0.05,0.35,0.03,1.,0.20]){assert_eq!(g.value(),v);assert!(g.icon().starts_with("INV_Misc_Gem_"));assert!(g.note(true).ends_with("Sets your gem for this run"));assert_eq!(g.note(false),format!("Active gem: {}",g.name()));}}
 #[test]fn only_successful_first_selection_locks_and_owner_reuse_resets(){let mut s=Gems::default();assert!(!s.accept(0,11,false,Gem::Ruby));assert_eq!(s.get(0,11),None);assert!(s.accept(0,11,true,Gem::Ruby));assert!(!s.accept(0,11,true,Gem::Emerald));assert_eq!(s.get(0,11),Some(Gem::Ruby));assert!(s.accept(1,22,true,Gem::Diamond));assert_eq!(s.get(1,22),Some(Gem::Diamond));assert!(!s.accept(5,33,true,Gem::Ruby));assert!(!s.accept(0,0,true,Gem::Ruby));s.retain(&[(0,44),(1,22)]);assert_eq!(s.get(0,11),None);assert_eq!(s.get(1,22),Some(Gem::Diamond));assert!(s.accept(0,44,true,Gem::Opal));}
}
