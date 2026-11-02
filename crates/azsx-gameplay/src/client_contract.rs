//! Version-independent client contract. Migration status is explicit: declaring
//! a domain does not claim its legacy operations have already been migrated.
use crate::{snapshot::GameSnapshot, Capabilities, Result};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApiVersion { pub major: u16, pub minor: u16 }
pub const API: ApiVersion = ApiVersion { major: 1, minor: 0 };
impl ApiVersion {
    pub fn accepts(self, required: Self) -> bool { self.major == required.major && self.minor >= required.minor }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Domain { Session, World, Combat, Progression, Presentation }
impl Domain { pub const ALL: [Self; 5] = [Self::Session, Self::World, Self::Combat, Self::Progression, Self::Presentation]; }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Migration { Legacy, Partial, Contract }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainBinding { pub domain: Domain, pub migration: Migration }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientContract {
    pub api: ApiVersion,
    pub domains: [DomainBinding; 5],
    /// Existing connected semantic operations, not promises for legacy domains.
    pub combat: Capabilities,
    pub allied_support: bool,
    pub shared_snapshot: bool,
}
impl ClientContract {
    pub fn domain(self, domain: Domain) -> DomainBinding { self.domains[domain as usize] }
    pub fn validate(self, required: ApiVersion) -> Result<()> {
        if !self.api.accepts(required) { return Err("Incompatible internal client API".into()); }
        if self.domains.iter().map(|b| b.domain).ne(Domain::ALL) { return Err("Incomplete or reordered client domain contract".into()); }
        Ok(())
    }
}
/// Read-only observation; the generation/freshness belongs to the producer.
/// None means no current observation, never a fabricated empty world.
pub trait ObservationPort {
    fn contract(&self) -> ClientContract;
    fn snapshot(&self) -> Option<&GameSnapshot>;
}
// Existing command ports remain authoritative: crate::CombatPort and
// crate::support::SupportPort. No blanket execute/unsupported stub is introduced
// for Session/Progression/Presentation before those legacy operations migrate.
