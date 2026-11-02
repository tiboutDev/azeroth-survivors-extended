//! Boss-loot compensation policy. Native collection and transport stay in the adapter.
use crate::Result;
use std::collections::BTreeMap;

/// Completed encounters, including repeated endless encounters; never chest count.
pub fn base_gold(encounter: u32) -> Result<u32> {
    if encounter == 0 {
        return Err("Missing boss encounter".into());
    }
    let value = (20.0 * 1.5f64.powf(f64::from(encounter - 1))).round();
    if !value.is_finite() || value > f64::from(u32::MAX) {
        return Err("Boss compensation exceeds native gold capacity".into());
    }
    Ok(value as u32)
}
pub fn award(base:u32,multiplier:f32)->Result<u32>{ crate::gold::multiply(base,multiplier) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entitlement {
    pub encounter: u32,
    pub player: (u32, u64),
    pub items: Vec<u32>,
}
/// Only native-issued loot can be claimed. Opaque ticket identity is scoped to a run.
#[derive(Default)]
pub struct Ledger {
    tickets: BTreeMap<u64, Entitlement>,
}
impl Ledger {
    pub fn issue(&mut self, ticket: u64, entitlement: Entitlement) -> Result<()> {
        if ticket == 0
            || entitlement.encounter == 0
            || entitlement.player.1 == 0
            || entitlement.items.is_empty()
            || entitlement.items.len() > 64
            || self.tickets.len() >= 256
            || self.tickets.contains_key(&ticket)
        {
            return Err("Invalid duplicate-loot entitlement".into());
        }
        self.tickets.insert(ticket, entitlement);
        Ok(())
    }
    /// Reserve before native IO: replay cannot pay twice, including after an IO failure.
    /// A report only states local pre-existing collection membership, never an amount.
    pub fn claim(
        &mut self,
        ticket: u64,
        player: (u32, u64),
        report: &[(u32, bool)],
    ) -> Result<(u32, Vec<u32>)> {
        let entitlement = self
            .tickets
            .get(&ticket)
            .ok_or("Unknown or settled loot ticket")?;
        let mut allowed=BTreeMap::<u32,usize>::new();
        for &id in &entitlement.items{*allowed.entry(id).or_default()+=1;}
        if player != entitlement.player || report.len()>64 || report.iter().any(|(id,_)|{
            let n=allowed.entry(*id).or_default();if *n==0{true}else{*n-=1;false}
        }) {return Err("Loot report does not match native entitlement".into());}
        let entitlement = self.tickets.remove(&ticket).unwrap();
        let duplicates=report.iter().filter_map(|&(item,known)|known.then_some(item)).collect();
        Ok((entitlement.encounter, duplicates))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn entitlement(encounter: u32, player: (u32, u64)) -> Entitlement {
        Entitlement {
            encounter,
            player,
            items: [101, 102].into(),
        }
    }
    #[test]
    fn curve_and_bonus_round_once() {
        assert_eq!(
            (1..=8).map(|n| base_gold(n).unwrap()).collect::<Vec<_>>(),
            [20, 30, 45, 68, 101, 152, 228, 342]
        );
        assert_eq!(award(45, 1.1).unwrap(), 50);
        assert!(base_gold(0).is_err());
        assert!(base_gold(u32::MAX).is_err());
        assert!(award(20, f32::NAN).is_err());
        assert!(award(20, -1.0).is_err());
    }
    #[test]
    fn authentic_entitlement_and_preexisting_collection_only() {
        let mut l = Ledger::default();
        l.issue(7, entitlement(1, (1, 99))).unwrap();
        assert!(l.claim(7, (2, 99), &[(101, true)]).is_err());
        assert!(l.claim(7, (1, 99), &[(103, true)]).is_err());
        assert!(l.claim(7, (1, 99), &[(101, true), (101, true)]).is_err());
        assert_eq!(
            l.claim(7, (1, 99), &[(101, true), (102, false)]).unwrap(),
            (1, vec![101])
        );
        assert!(l.claim(7, (1, 99), &[(101, true)]).is_err());
        l.issue(8, entitlement(1, (1, 99))).unwrap();
        // A distinct native-issued reward can legitimately contain the same item.
        assert_eq!(l.claim(8, (1, 99), &[(101, true)]).unwrap().1,[101]);
        l.issue(9, entitlement(2, (1, 99))).unwrap();
        assert_eq!(l.claim(9, (1, 99), &[(101, true)]).unwrap().1, [101]);
        l.issue(10, entitlement(2, (0, 88))).unwrap();
        assert_eq!(l.claim(10, (0, 88), &[(101, true)]).unwrap().1, [101]);
    }
    #[test]fn legitimate_native_multiplicity_is_bounded_and_replay_safe(){
      let mut l=Ledger::default();l.issue(1,Entitlement{encounter:1,player:(0,77),items:vec![101,101]}).unwrap();
      assert!(l.claim(1,(0,77),&[(101,true),(101,true),(101,true)]).is_err());
      assert_eq!(l.claim(1,(0,77),&[(101,true),(101,true)]).unwrap().1,[101,101]);
      assert!(l.claim(1,(0,77),&[(101,true)]).is_err());
      l.issue(2,Entitlement{encounter:2,player:(0,77),items:vec![101,101]}).unwrap();
      assert_eq!(l.claim(2,(0,77),&[(101,false),(101,true)]).unwrap().1,[101]);
    }

}
