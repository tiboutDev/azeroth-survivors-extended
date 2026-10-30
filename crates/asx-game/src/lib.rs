//! Selected native integration for official 0.130 and typed readers over a
//! memory view. Pure: no Windows API and no process access, so everything here
//! is testable offline against the executables on disk and synthetic memory.

pub mod enemies;
pub mod memory;
pub mod pe;

pub type Result<T> = std::result::Result<T, String>;

pub mod official;

pub mod official_native;

// Inherited translation data for the selected current image, not old selectors.
#[cfg(test)]
#[allow(dead_code)]
#[path = "bindings/inherited/official117.rs"]
pub(crate) mod official117;
#[cfg(test)]
#[allow(dead_code)]
#[path = "bindings/inherited/official120.rs"]
pub(crate) mod official120;
#[cfg(test)]
#[allow(dead_code)]
#[path = "bindings/inherited/official1201.rs"]
pub(crate) mod official1201;
#[cfg(test)]
#[allow(dead_code)]
#[path = "bindings/inherited/official121.rs"]
pub(crate) mod official121;
#[cfg(test)]
#[allow(dead_code)]
#[path = "bindings/official125.rs"]
pub(crate) mod official125;

// All active native consumers use this boundary. Versioned descriptors retain
// their exact identities; switching the selected build never relabels old data.
pub use official130 as current;

// Previous identity remains a test-only reference; it is never selected.
#[cfg(test)]
#[allow(dead_code)]
#[path = "bindings/official129.rs"]
pub(crate) mod official129;

// Exact current descriptor; runtime validation remains a separate gate.
#[path = "bindings/official130.rs"]
pub mod official130;
