//! Bounded read-only selected-native death/authority checks.
use super::super::{Runtime, RUNTIME};
pub(in crate::runtime) trait Services {
    fn continuation(&self, kind: usize, registers: [u64; 6]) -> Result<Option<usize>, String>;
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
}
pub(super) fn active() -> Option<&'static dyn Services> {
    RUNTIME.get().filter(|runtime| runtime.rules).map(|runtime| runtime as &dyn Services)
}
impl Services for Runtime {
    fn continuation(&self, kind: usize, registers: [u64; 6]) -> Result<Option<usize>, String> {
        self.adapter.downed_xp_continuation(&self.local, kind, registers)
            .map(|at| at.map(|at| self.local.base + at as usize))
    }
    fn info(&self, message: &str) { self.log.info(message); }
    fn warn(&self, message: &str) { self.log.warn(message); }
}
