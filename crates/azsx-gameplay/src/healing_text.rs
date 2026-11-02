//! Bounded cosmetic accumulation of actual healing omitted by native integer text.
use std::collections::BTreeMap;
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub struct Key{pub caster:u64,pub target:u64,pub spell:u32}
#[derive(Default)]pub struct Accumulator{pending:BTreeMap<Key,(f32,f32)>,last_clock:Option<f32>}
impl Accumulator{
 pub fn observe_clock(&mut self,clock:f32){
  if !clock.is_finite()||clock<0.{return}
  if self.last_clock.is_some_and(|previous|clock<previous){self.pending.clear();}
  self.last_clock=Some(clock);
 }

 pub fn credit(&mut self,key:Key,clock:f32,effective:f32,native_reported:bool)->Option<u32>{
  if key.caster==0||key.target==0||!clock.is_finite()||clock<0.||!effective.is_finite()||effective<=0.||effective>1e6{return None}
  self.observe_clock(clock);
  // Native already sends nonzero-spell heals >=1; never duplicate those packets.
  if native_reported{return None}
  if !self.pending.contains_key(&key)&&self.pending.len()>=64{return None}
  let state=self.pending.entry(key).or_insert((0.,clock+1.));
  state.0=(state.0+effective).min(1e6);
  if clock<state.1||state.0<1.{return None}
  let whole=state.0.floor()as u32;state.0-=whole as f32;state.1=clock+1.;Some(whole)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn key()->Key{Key{caster:11,target:22,spell:139}}
 #[test]fn fractions_are_combined_without_inventing_or_repeating_credit(){let mut a=Accumulator::default();assert_eq!(a.credit(key(),0.,0.4,false),None);assert_eq!(a.credit(key(),0.5,0.4,false),None);assert_eq!(a.credit(key(),1.,0.4,false),Some(1));assert_eq!(a.credit(key(),2.,0.1,false),None);}
 #[test]fn ordinary_native_packets_and_overheal_add_nothing(){let mut a=Accumulator::default();for n in [0.,-1.,f32::NAN,f32::INFINITY,1.,10.]{assert_eq!(a.credit(key(),0.,n,n>=1.),None);}assert!(a.pending.is_empty());}
 #[test]fn actor_and_spell_sources_do_not_stack_into_each_other(){let mut a=Accumulator::default();a.credit(key(),0.,0.6,false);let other=Key{caster:33,..key()};assert_eq!(a.credit(other,2.,0.6,false),None);assert_eq!(a.credit(key(),2.,0.6,false),Some(1));}
 #[test]fn reset_clock_discards_previous_run_fraction(){let mut a=Accumulator::default();a.credit(key(),100.,0.9,false);assert_eq!(a.credit(key(),0.,0.2,false),None);assert_eq!(a.credit(key(),1.,0.2,false),None);}
 #[test]fn replay_is_observed_even_when_the_next_heal_arrives_later(){let mut a=Accumulator::default();a.credit(key(),10.,0.9,false);a.observe_clock(0.);assert_eq!(a.credit(key(),20.,0.2,false),None);assert_eq!(a.credit(key(),21.,0.2,false),None);}
 #[test]fn native_spell_zero_has_no_original_packet_even_for_whole_amounts(){let mut a=Accumulator::default();let k=Key{spell:0,..key()};assert_eq!(a.credit(k,0.,2.,false),None);assert_eq!(a.credit(k,1.,2.,false),Some(4));}
}
