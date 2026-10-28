//! Portable memory views shared by active native adapters.

/// A bounded view of process memory. Implementations own the policy for
/// reading; adapter and parser code only receive copied bytes.
pub trait Mem {
    fn read(&self, address: u64, len: usize) -> crate::Result<Vec<u8>>;
}
