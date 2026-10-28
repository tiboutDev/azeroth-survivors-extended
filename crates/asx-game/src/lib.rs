//! Selected native integration for official 0.122 and typed readers over a
//! memory view. Pure: no Windows API and no process access, so everything here
//! is testable offline against the executables on disk and synthetic memory.

pub mod pe;
pub mod memory;
pub mod enemies;

pub type Result<T> = std::result::Result<T, String>;

pub mod official;

pub mod official_native;

// Inherited translation data for the selected current image, not old selectors.
#[path = "bindings/inherited/official117.rs"]
pub mod official117;
#[path = "bindings/inherited/official120.rs"]
pub mod official120;
#[path = "bindings/inherited/official1201.rs"]
pub mod official1201;
#[path = "bindings/inherited/official121.rs"]
pub mod official121;
#[path = "bindings/official122.rs"]
pub mod official122;
