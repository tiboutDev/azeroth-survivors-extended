//! One periodic-healing batch. No native frames, registers or addresses.
use crate::Result;
#[derive(Clone, Debug, Default)]
pub struct Batch {
    foreign_raw: f32,
    foreign_rows: u32,
    personal_rows: u32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub effective: f32,
    pub overheal: f32,
    pub raw_total: f32,
    pub foreign_raw: f32,
    pub personal_raw: f32,
    pub foreign_rows: u32,
    pub personal_rows: u32,
}
impl Batch {
    pub fn observe(&mut self, raw: f32, foreign: bool) -> Result<()> {
        if !raw.is_finite() || !(0. ..=1e12).contains(&raw) || self.foreign_rows + self.personal_rows >= 512 {
            return Err("Invalid periodic healing contribution or row bound".into());
        }
        let next = if foreign { self.foreign_raw + raw } else { self.foreign_raw };
        if !next.is_finite() { return Err("Periodic healing sum overflow".into()); }
        self.foreign_raw = next;
        if foreign { self.foreign_rows += 1; } else { self.personal_rows += 1; }
        Ok(())
    }
    pub fn plan(&self, effective: f32, overheal: f32) -> Result<Plan> {
        // Native HP rounding can produce a slightly negative overheal amount.
        // Keep the established f32 operation order; never recompute totals from
        // per-spell credit fractions or subtract from a stored talent budget.
        let raw_total = effective + overheal;
        if !effective.is_finite() || effective < 0. || !overheal.is_finite() || !raw_total.is_finite() {
            return Err("Invalid periodic healing totals".into());
        }
        Ok(Plan { effective, overheal, raw_total, foreign_raw: self.foreign_raw,
            personal_raw: (raw_total - self.foreign_raw).max(0.),
            foreign_rows: self.foreign_rows, personal_rows: self.personal_rows })
    }
}
