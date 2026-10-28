//! Shared presentation of already-active run bonuses; no native UI layouts.
use crate::{glyphs::Glyph,enchantments::{Enchantment,Rank,RankedEnchantment},gems::Gem};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum KeptBonus{Glyph(Glyph),Enchantment(RankedEnchantment),Gem(Gem)}
impl KeptBonus{
 pub fn code(self)->u16{match self{Self::Glyph(g)=>g as u16,Self::Enchantment(e)=>800+e.kind as u16*5+e.rank as u16,Self::Gem(g)=>900+g as u16}}
 pub fn decode(n:u16)->Option<Self>{if n<204{Some(Self::Glyph(*Glyph::ALL.get(n as usize)?))}else if (800..840).contains(&n){Some(Self::Enchantment(RankedEnchantment{kind:*Enchantment::ALL.get((n as usize-800)/5)?,rank:*Rank::ALL.get((n as usize-800)%5)?}))}else if (900..908).contains(&n){Some(Self::Gem(*Gem::ALL.get(n as usize-900)?))}else{None}}
 pub fn name(self)->String{match self{Self::Glyph(g)=>g.name().into(),Self::Enchantment(e)=>e.name(),Self::Gem(g)=>g.name().into()}}
 pub fn icon(self)->&'static str{match self{Self::Glyph(_)=>"INV_Scroll_02",Self::Enchantment(e)=>e.icon(),Self::Gem(g)=>g.icon()}}
 pub fn lines(self)->(String,String){match self{Self::Glyph(g)=>(g.lines()[0].into(),g.lines()[1].into()),Self::Enchantment(e)=>(e.text(),String::new()),Self::Gem(g)=>(g.text().into(),String::new())}}
 pub fn category(self)->u8{match self{Self::Glyph(_)=>0,Self::Enchantment(_)=>1,Self::Gem(_)=>2}}
}
#[cfg(test)]mod tests{use super::*;#[test]fn all_active_bonuses_round_trip_and_keep_their_exact_values(){let all=Glyph::ALL.into_iter().map(KeptBonus::Glyph).chain(Enchantment::ALL.into_iter().flat_map(|kind|Rank::ALL.into_iter().map(move|rank|KeptBonus::Enchantment(RankedEnchantment{kind,rank})))).chain(Gem::ALL.into_iter().map(KeptBonus::Gem));let mut ids=std::collections::BTreeSet::new();for b in all{assert!(ids.insert(b.code()));assert_eq!(KeptBonus::decode(b.code()),Some(b));assert!(!b.lines().0.is_empty());assert!(!b.icon().is_empty());}assert_eq!(ids.len(),252);assert_eq!(KeptBonus::decode(65535),None);}}
