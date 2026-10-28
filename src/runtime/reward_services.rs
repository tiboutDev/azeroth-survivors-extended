//! Bounded native operations for gem and enchantment coordination.
//!
//! No runtime, memory accessor or adapter escapes this interface. Each operation
//! completes before returning; callers retain only copied observations or an
//! owned stat loan, never a runtime lock across a reentrant native rebuild.
use super::{io, Runtime};
use asx_game::official::{AmuletChoice, RingChoice, RingStatLoan, SupportCalls};
use asx_gameplay::{enchantments::RankedEnchantment, gems::Gem};

pub(super) trait Services {
    fn amulet_card(&self, game: u64, output: u64) -> Result<Option<AmuletChoice>, String>;
    fn amulet_selected(&self, game: u64, index: u64) -> Result<Option<AmuletChoice>, String>;
    fn amulet_owned(&self, game: u64) -> Result<Option<u64>, String>;
    fn amulet_rank(&self, game: u64) -> Result<u32, String>;
    fn ring_card(&self, game: u64, output: u64) -> Result<Option<RingChoice>, String>;
    fn ring_selected(&self, game: u64, index: u64) -> Result<Option<RingChoice>, String>;
    fn ring_owned(&self, game: u64, ring: u8) -> Result<Option<(u32, u32)>, String>;
    fn run_source(&self, game: u64) -> Result<Option<(u32, u64)>, String>;
    fn reward_card_note(&self, output: u64, note: &str) -> Result<(), String>;
    fn refresh_stats(&self, game: u64) -> Result<(), String>;
    fn gem_stats(&self, game: u64, gem: Gem) -> Result<Option<RingStatLoan>, String>;
    fn ring_stats(&self, game: u64, bonuses: &[RankedEnchantment]) -> Result<Option<RingStatLoan>, String>;
    fn restore_stats(&self, loan: RingStatLoan) -> Result<(), String>;
    fn warn(&self, message: &str);
    fn info(&self, message: &str);
}

impl Services for Runtime {
    fn amulet_card(&self, game: u64, output: u64) -> Result<Option<AmuletChoice>, String> {
        self.adapter.amulet_card(&self.local, self.local.base as u64, game, output)
    }
    fn amulet_selected(&self, game: u64, index: u64) -> Result<Option<AmuletChoice>, String> {
        self.adapter.amulet_selected(&self.local, self.local.base as u64, game, index)
    }
    fn amulet_owned(&self, game: u64) -> Result<Option<u64>, String> {
        self.adapter.amulet_owned(&self.local, game)
    }
    fn amulet_rank(&self, game: u64) -> Result<u32, String> {
        self.adapter.amulet_rank(&self.local, game)
    }
    fn ring_card(&self, game: u64, output: u64) -> Result<Option<RingChoice>, String> {
        self.adapter.ring_card(&self.local, self.local.base as u64, game, output)
    }
    fn ring_selected(&self, game: u64, index: u64) -> Result<Option<RingChoice>, String> {
        self.adapter.ring_selected(&self.local, self.local.base as u64, game, index)
    }
    fn ring_owned(&self, game: u64, ring: u8) -> Result<Option<(u32, u32)>, String> {
        self.adapter.ring_owned(&self.local, game, ring, self.local.base as u64)
    }
    fn run_source(&self, game: u64) -> Result<Option<(u32, u64)>, String> {
        self.adapter.run_source(&self.local, game)
    }
    fn reward_card_note(&self, output: u64, note: &str) -> Result<(), String> {
        self.adapter.reward_card_note(&io::Access { local: &self.local, adapter: self.adapter }, output, note)
    }
    fn refresh_stats(&self, game: u64) -> Result<(), String> {
        SupportCalls::refresh_support(&io::Access { local: &self.local, adapter: self.adapter }, game)
    }
    fn gem_stats(&self, game: u64, gem: Gem) -> Result<Option<RingStatLoan>, String> {
        self.adapter.gem_stats(&io::Access { local: &self.local, adapter: self.adapter }, game, gem)
    }
    fn ring_stats(&self, game: u64, bonuses: &[RankedEnchantment]) -> Result<Option<RingStatLoan>, String> {
        self.adapter.ring_stats(&io::Access { local: &self.local, adapter: self.adapter }, game, bonuses)
    }
    fn restore_stats(&self, loan: RingStatLoan) -> Result<(), String> {
        self.adapter.restore_ring_stats(&io::Access { local: &self.local, adapter: self.adapter }, loan)
    }
    fn warn(&self, message: &str) { self.log.warn(message); }
    fn info(&self, message: &str) { self.log.info(message); }
}

#[cfg(test)]
mod tests;
