//! Selected strongest-PW:S pool observations and bounded source loans. Native
//! delivery still computes the shield; these copies never enter its dispatcher.
use super::{Adapter, Mem, MutableMemory, Result};

const SPELL: u32 = 17;
fn id(row: &[u8; 8]) -> u32 { u32::from_le_bytes(row[..4].try_into().unwrap()) }
fn amount(row: &[u8; 8]) -> Result<f32> {
    let value = f32::from_le_bytes(row[4..].try_into().unwrap());
    if !value.is_finite() || !(0.0..=1e12).contains(&value) { return Err("Invalid native shield amount".into()); }
    Ok(value)
}
#[derive(Clone, Debug)]
pub(super) struct ShieldState {
    guid: u64,
    main: [u8; 8],
    extra: Vec<[u8; 8]>,
    amount: f32,
}
struct Pools {
    guid: u64,
    main: [u8; 8],
    extra: Vec<[u8; 8]>,
    capacity: u64,
    pointer: u64,
    total: f32,
}
struct Plan { main: [u8; 8], extra: Vec<[u8; 8]>, total: f32 }
impl Pools {
    fn snapshot(&self) -> Result<ShieldState> {
        let extra = self.extra.iter().filter(|r| id(r) == SPELL).copied().collect::<Vec<_>>();
        let mut value = if id(&self.main) == SPELL { amount(&self.main)? } else { 0. };
        for row in &extra { value += amount(row)?; }
        if !value.is_finite() || value > 1e12 { return Err("Native PW:S pool overflow".into()); }
        Ok(ShieldState { guid: self.guid, main: self.main, extra, amount: value })
    }
    fn loan(&self, saved: &ShieldState, restore: bool) -> Result<Plan> {
        if saved.guid == 0 || saved.guid != self.guid { return Err("Shield loan identity changed".into()); }
        let current = self.snapshot()?;
        let mut main = if id(&self.main) == SPELL { [0; 8] } else { self.main };
        let mut extra = self.extra.iter().filter(|r| id(r) != SPELL).copied().collect::<Vec<_>>();
        let mut total = (self.total - current.amount).max(0.);
        if restore {
            extra.extend_from_slice(&saved.extra);
            if id(&saved.main) == SPELL {
                if id(&main) == 0 { main = saved.main; } else { extra.push(saved.main); }
            }
            total += saved.amount;
        }
        if !total.is_finite() || total > 1e12 { return Err("Shield loan total overflow".into()); }
        if extra.len() as u64 > self.capacity { return Err("Shield rollback cannot grow native storage".into()); }
        Ok(Plan { main, extra, total })
    }
}
impl Adapter {
    fn shield_pools(self, mem: &dyn Mem, game: u64) -> Result<Pools> {
        let read = |at: u64, n| mem.read(game.checked_add(at).ok_or("Shield field overflow")?, n);
        let guid = u64::from_le_bytes(read(self.game_field(0x478), 8)?.try_into().map_err(|_| "Truncated shield identity")?);
        let main = read(self.game_field(0x480), 8)?.try_into().map_err(|_| "Truncated main shield")?;
        let total = f32::from_le_bytes(read(self.game_field(0x4fc), 4)?.try_into().map_err(|_| "Truncated total absorption")?);
        if !total.is_finite() || !(0.0..=1e12).contains(&total) { return Err("Invalid total absorption".into()); }
        let header = read(crate::current::STRONGEST_SHIELD_POOLS, 24)?;
        if header.len() != 24 { return Err("Truncated auxiliary shield header".into()); }
        let word = |at| u64::from_le_bytes(header[at..at + 8].try_into().unwrap());
        let (capacity, pointer, count) = (word(0), word(8), word(16));
        if count > capacity || capacity > 512 || (capacity > 0 && (pointer < 65536 || pointer.checked_add(capacity * 8).is_none())) {
            return Err("Invalid auxiliary shield extent".into());
        }
        let bytes = if count == 0 { vec![] } else { mem.read(pointer, count as usize * 8)? };
        if bytes.len() != count as usize * 8 { return Err("Truncated auxiliary shields".into()); }
        let extra = bytes.chunks_exact(8).map(|r| r.try_into().unwrap()).collect::<Vec<_>>();
        amount(&main)?;
        for row in &extra { amount(row)?; }
        Ok(Pools { guid, main, extra, capacity, pointer, total })
    }
    fn commit_shield_loan(self, mem: &dyn MutableMemory, game: u64, pools: &Pools, plan: Plan) -> Result<()> {
        // All reads, identity/extent checks and planning precede mutation. No
        // native callback or retained native container borrow crosses this scope.
        if !plan.extra.is_empty() {
            let bytes = plan.extra.iter().flatten().copied().collect::<Vec<_>>();
            mem.write(pools.pointer, &bytes)?;
        }
        mem.write(game + crate::current::STRONGEST_SHIELD_POOLS + 16, &(plan.extra.len() as u64).to_le_bytes())?;
        mem.write(game + self.game_field(0x480), &plan.main)?;
        mem.write(game + self.game_field(0x4fc), &plan.total.to_le_bytes())
    }
    pub(super) fn shield_snapshot(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<Option<ShieldState>> {
        if crate::current::STRONGEST_SHIELD_POOLS == 0 || spell != SPELL { return Ok(None); }
        self.shield_pools(mem, game)?.snapshot().map(Some)
    }
    pub(super) fn isolate_shield(self, mem: &dyn MutableMemory, game: u64, saved: &ShieldState) -> Result<()> {
        let pools = self.shield_pools(mem, game)?;
        let plan = pools.loan(saved, false)?;
        self.commit_shield_loan(mem, game, &pools, plan)
    }
    pub(super) fn restore_shield(self, mem: &dyn MutableMemory, game: u64, saved: &ShieldState) -> Result<()> {
        let pools = self.shield_pools(mem, game)?;
        let plan = pools.loan(saved, true)?;
        self.commit_shield_loan(mem, game, &pools, plan)
    }
    /// None retains the older additive producer rule. A selected PW:S pool
    /// observation separates it from Ice Barrier and unrelated absorption.
    pub fn support_shield_pool(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<Option<f32>> {
        Ok(self.shield_snapshot(mem, game, spell)?.map(|s| s.amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(spell: u32, value: f32) -> [u8; 8] {
        let mut b = [0; 8]; b[..4].copy_from_slice(&spell.to_le_bytes()); b[4..].copy_from_slice(&value.to_le_bytes()); b
    }
    fn pools(main: [u8; 8], extra: Vec<[u8; 8]>, total: f32) -> Pools {
        Pools { guid: 111, main, extra, total, capacity: 8, pointer: 0x20000 }
    }
    #[test]
    fn main_and_auxiliary_power_word_shield_loans_restore_without_a_successful_cast() {
        for before in [pools(row(17, 50.), vec![row(999, 30.)], 80.),
                       pools(row(11426, 40.), vec![row(17, 50.)], 90.)] {
            let saved = before.snapshot().unwrap();
            assert_eq!(saved.amount, 50.);
            let isolated = before.loan(&saved, false).unwrap();
            assert!(isolated.extra.iter().all(|r| id(r) != 17));
            assert_ne!(id(&isolated.main), 17);
            let after = pools(isolated.main, isolated.extra, isolated.total);
            let restored = after.loan(&saved, true).unwrap();
            assert_eq!(restored.main, before.main);
            assert_eq!(restored.extra, before.extra);
            assert_eq!(restored.total, before.total);
        }
    }
    #[test]
    fn restoring_the_loan_preserves_other_absorption_gained_during_the_native_call() {
        let before = pools(row(11426, 40.), vec![row(17, 50.)], 100.);
        let saved = before.snapshot().unwrap();
        let after = pools(row(11426, 40.), vec![row(17, 44.8), row(999, 20.)], 114.8);
        let restored = after.loan(&saved, true).unwrap();
        assert_eq!(restored.main, row(11426, 40.));
        assert_eq!(restored.extra, vec![row(999, 20.), row(17, 50.)]);
        assert!((restored.total - 120.).abs() < 0.0001);
        // A different shield can occupy the now-vacant main pool during a call.
        let saved = pools(row(17, 50.), vec![], 50.).snapshot().unwrap();
        let after = pools(row(11426, 40.), vec![row(17, 44.8)], 84.8);
        let restored = after.loan(&saved, true).unwrap();
        assert_eq!(restored.main, row(11426, 40.));
        assert_eq!(restored.extra, vec![row(17, 50.)]);
        assert_eq!(restored.total, 90.);
    }
    #[test]
    fn rollback_rejects_reused_identity_and_insufficient_capacity_before_writes() {
        let saved = pools(row(17, 50.), vec![], 50.).snapshot().unwrap();
        let mut after = pools(row(11426, 40.), vec![], 40.);
        after.capacity = 0;
        assert!(after.loan(&saved, true).is_err());
        after.capacity = 8; after.guid = 222;
        assert!(after.loan(&saved, true).is_err());
    }
}
