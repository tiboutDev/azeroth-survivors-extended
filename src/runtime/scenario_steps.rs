//! Diagnostic-only progression gates. Native lifetime/healing/presence clocks stay unchanged.
pub(super) fn enabled()->bool{std::env::var("ASX_TEST_OBSERVED_STEPS").as_deref()==Ok("1")}
pub(super) fn supply_ready(step:u32,reserves:usize,last_claim:Option<u64>)->bool{
 match step{0=>reserves==0,1=>reserves>=1,2..=10=>last_claim==Some(u64::from(step)),_=>false}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn reserve_creation_and_claim_acknowledgements_gate_progress(){assert!(supply_ready(0,0,None));assert!(!supply_ready(1,0,None));assert!(supply_ready(1,1,None));assert!(!supply_ready(2,2,Some(1)));assert!(supply_ready(2,1,Some(2)));assert!(!supply_ready(11,0,Some(11)));}
}

pub(super) fn goblin_action(phase:u32,stage:u8,movement_observed:bool,control_available:bool)->u8{
 if !movement_observed{return 0}
 match(phase,stage){(1,0)if control_available=>1,(2,0)=>2,_=>0}
}
#[cfg(test)]mod goblin_tests{use super::*;
 #[test]fn first_checks_taunt_once_and_second_waits_for_movement_without_repeating_control(){
  assert_eq!(goblin_action(1,0,true,false),0);assert_eq!(goblin_action(1,0,true,true),1);
  assert_eq!(goblin_action(1,1,true,true),0);assert_eq!(goblin_action(2,0,false,true),0);
  assert_eq!(goblin_action(2,0,true,false),2);assert_eq!(goblin_action(2,2,true,true),0);
 }
}
