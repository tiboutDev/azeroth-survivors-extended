use asx_gameplay::{immunity_aggro::*,taunts::{Taunts,Binding},TauntSpell};
fn binding(life:u64)->Binding{Binding{player:1,life,spell:TauntSpell::ShieldSlam}}
#[test]fn immunity_drops_old_taunts_once_and_allows_new_taunts(){
 let mut a=ImmunityAggro::default();let mut t=Taunts::default();t.bindings.insert(10,binding(111));
 assert!(!a.observe(1,111,false,&mut t));assert!(a.observe(1,111,true,&mut t));assert!(t.bindings.is_empty());
 t.bindings.insert(10,binding(111));assert!(!a.observe(1,111,true,&mut t));assert!(t.bindings.contains_key(&10));
 // Refresh/overlapping immunity does not continuously erase a fresh taunt.
 assert!(!a.observe(1,111,true,&mut t));assert!(t.bindings.contains_key(&10));
 a.observe(1,111,false,&mut t);assert!(a.observe(1,111,true,&mut t));assert!(t.bindings.is_empty());
 a.retain(&[(1,222)]);assert!(!a.immune(1));
}
fn candidates()->[Candidate;3]{[Candidate{slot:0,position:[0.,0.,0.],available:true},Candidate{slot:1,position:[10.,0.,0.],available:true},Candidate{slot:2,position:[-10.,0.,0.],available:true}]}
#[test]fn ordinary_target_ignores_immunity_but_explicit_taunt_can_override_it(){
 let c=candidates();assert_eq!(replacement(Some(0),[0.;3],&c,&[0],None),Some(1));
 assert_eq!(replacement(Some(0),[0.;3],&c,&[0],Some(0)),Some(0));
 assert_eq!(replacement(Some(1),[0.;3],&c,&[0],None),Some(1));
 assert_eq!(replacement(Some(0),[0.;3],&c,&[0,1,2],None),None);
 assert_eq!(replacement(None,[0.;3],&c,&[],None),Some(0));
 let mut dead=c;dead[0].available=false;assert_eq!(replacement(Some(0),[0.;3],&dead,&[],Some(0)),Some(1));
}
