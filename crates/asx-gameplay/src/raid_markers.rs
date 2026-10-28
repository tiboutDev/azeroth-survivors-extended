//! One-time cosmetic marker initialization; caller supplies current identities and RNG.
use std::collections::{BTreeMap,BTreeSet};
#[derive(Default)]
pub struct Initialization {seen:BTreeMap<u32,u64>}
impl Initialization {
 pub fn needs(&self,actors:impl Iterator<Item=(u32,u64)>)->bool{actors.into_iter().any(|(slot,id)|self.seen.get(&slot)!=Some(&id))}
 pub fn plan(&mut self,actors:&[(u32,u64)],occupied:&[Option<u64>;8],mut pick:impl FnMut(usize)->usize)->Result<Vec<(u32,u64,u8)>,String>{
  let mut slots=BTreeSet::new();let mut ids=BTreeSet::new();
  if actors.iter().any(|(slot,id)|*id==0||!slots.insert(*slot)||!ids.insert(*id)){return Err("Ambiguous marker identity".into())}
  self.seen.retain(|slot,id|actors.contains(&(*slot,*id)));
  let mut free=(0..8).filter(|i|occupied[*i].is_none()).collect::<Vec<_>>();let mut result=vec![];
  for &(slot,id) in actors {
   if self.seen.get(&slot)==Some(&id){continue}
   // Commit initialization before presentation; an uncertain native action must
   // never be retried later over a player's manual marker change.
   self.seen.insert(slot,id);
   if occupied.contains(&Some(id))||free.is_empty(){continue}
   let choice=pick(free.len());if choice>=free.len(){return Err("Invalid marker draw".into())}
   let icon=free.swap_remove(choice);result.push((slot,id,icon as u8+1));
  }
  Ok(result)
 }
}
/// Recheck against the live native table immediately before applying a plan.
pub fn can_assign(occupied:&[Option<u64>;8],guid:u64,icon:u8)->bool{guid!=0&&(1..=8).contains(&icon)&&occupied[usize::from(icon-1)].is_none()&&!occupied.contains(&Some(guid))}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn unique_random_markers_cover_the_whole_native_pool(){let ids=[(0,11),(1,22),(2,33),(3,44),(4,55)];let mut s=Initialization::default();let plan=s.plan(&ids,&[None;8],|n|n-1).unwrap();assert_eq!(plan.iter().map(|p|p.2).collect::<Vec<_>>(),vec![8,7,6,5,4]);}
 #[test]fn refresh_removal_and_resurrection_do_not_reinitialize(){let ids=[(0,11),(1,22)];let mut s=Initialization::default();assert_eq!(s.plan(&ids,&[None;8],|_|0).unwrap().len(),2);assert!(s.plan(&ids,&[None;8],|_|panic!("redraw")).unwrap().is_empty());}
 #[test]fn late_join_and_reused_slot_keep_current_manual_assignments(){let mut s=Initialization::default();s.plan(&[(0,11)],&[None;8],|_|0).unwrap();let occupied=[Some(11),Some(99),None,None,None,None,None,None];assert_eq!(s.plan(&[(0,11),(1,22)],&occupied,|_|0).unwrap(),vec![(1,22,3)]);assert!(s.plan(&[(0,11),(1,22)],&[None;8],|_|0).unwrap().is_empty());assert_eq!(s.plan(&[(0,11),(1,33)],&occupied,|_|0).unwrap(),vec![(1,33,3)]);}
 #[test]fn a_new_run_preserves_existing_participant_and_unrelated_markers(){let mut s=Initialization::default();let occupied=[Some(11),Some(99),None,None,None,None,None,None];assert!(s.plan(&[(0,11)],&occupied,|_|panic!("existing mark must be preserved")).unwrap().is_empty());assert!(!s.needs([(0,11)].into_iter()));}
 #[test]fn a_marked_late_joiner_keeps_their_mark_and_another_arrival_uses_a_free_icon(){let mut s=Initialization::default();s.plan(&[(0,11)],&[None;8],|_|0).unwrap();let marks=[Some(11),Some(22),Some(99),None,None,None,None,None];assert_eq!(s.plan(&[(0,11),(1,22),(2,33)],&marks,|_|0).unwrap(),vec![(2,33,4)]);}
 #[test]fn a_live_manual_change_invalidates_a_pending_automatic_assignment(){let empty=[None;8];assert!(can_assign(&empty,11,2));let mut marks=empty;marks[1]=Some(22);assert!(!can_assign(&marks,11,2));marks[1]=None;marks[6]=Some(11);assert!(!can_assign(&marks,11,2));assert!(!can_assign(&empty,11,0));assert!(!can_assign(&empty,11,9));assert!(!can_assign(&empty,0,2));}
 #[test]fn invalid_identities_and_draws_are_rejected(){let mut s=Initialization::default();assert!(s.plan(&[(0,11),(1,11)],&[None;8],|_|0).is_err());assert!(s.plan(&[(0,0)],&[None;8],|_|0).is_err());assert!(s.plan(&[(0,11)],&[None;8],|n|n).is_err());}
}
