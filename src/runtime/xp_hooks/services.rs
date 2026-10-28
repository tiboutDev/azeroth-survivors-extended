//! Runtime binding for the local XP hook boundary, not a global native-free API.
use super::super::{blessings, experience, io, Runtime, RUNTIME};
use asx_game::official::ExperienceAward;

pub(super) trait Services {
    fn shared_input(&self, request: u64, game: u64) -> Result<u32, String>;
    fn capture_shared(&self, game: u64, sum: u32);
    fn multiplier(&self) -> Option<f32>;
    fn return_site(&self, stack: u64) -> Option<u64>;
    fn normalize(&self, game: u64, site: u64, registers: [u64; 3], multiplier: f32, shared: Option<u32>)
        -> Result<Option<ExperienceAward>, String>;
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
}

pub(super) fn active() -> Option<&'static dyn Services> {
    RUNTIME.get().filter(|runtime| runtime.rules).map(|runtime| runtime as &dyn Services)
}

impl Services for Runtime {
    fn shared_input(&self, request: u64, game: u64) -> Result<u32, String> {
        self.adapter.shared_xp_input(&self.local, request, game)
    }
    fn capture_shared(&self, game: u64, sum: u32) {
        experience::capture(self, game, sum);
    }
    fn multiplier(&self) -> Option<f32> {
        let multiplier = self.party.lock().ok().and_then(|p| {
            p.as_ref().and_then(|party| {
                asx_gameplay::official::shared_xp_multiplier(
                    party.actors.iter().map(|a| (
                        a.xp_multiplier - blessings::personal(a.slot, a.guid, 1),
                        self.adapter.racial_xp_bonus(a.race_id),
                    )),
                ).ok()
            })
        })?;
        Some(multiplier + blessings::strongest(1))
    }
    fn return_site(&self, stack: u64) -> Option<u64> {
        let stack = self.local.read(stack as usize, 8).ok()?;
        let stack: [u8; 8] = stack.try_into().ok()?;
        u64::from_le_bytes(stack).checked_sub(self.local.base as u64)
    }
    fn normalize(&self, game: u64, site: u64, registers: [u64; 3], multiplier: f32, shared: Option<u32>)
        -> Result<Option<ExperienceAward>, String>
    {
        self.adapter.normalize_xp(
            &io::Access { local: &self.local, adapter: self.adapter },
            game, site, registers, multiplier, shared,
        )
    }
    fn info(&self, message: &str) { self.log.info(message); }
    fn warn(&self, message: &str) { self.log.warn(message); }
}
