//! Read-only native difficulty integrity checks and an isolated diagnostic probe.
use super::Adapter;
use crate::{
    official::{Entry, SupportCalls},
    Result,
};
impl Adapter {
    pub fn scaling_probe_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x5983510),
                bytes: self.expected(0x5983510, &[
                0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53,
            ]),
        }
    }
    pub fn fixture_scaling_probe(self, io: &dyn SupportCalls, game: u64) -> Result<[f32; 2]> {
        if self.support_caster(io, game)?.slot != 0 {
            return Err("Scaling probe requires restored host".into());
        }
        // Probe only the real native solo/cooperative paths. The retired HP
        // option had no effect and must not become a guessed field dependency.
        let flag = io.read(game + self.game_field(0xc41), 1)?;
        let result = (|| {
            let mut out = [0.; 2];
            for (i, coop) in [0u8, 1]
                .into_iter()
                .enumerate()
            {
                io.write(game + self.game_field(0xc41), &[coop])?;
                out[i] = io.native_boss_health(game, 0)?;
                if !out[i].is_finite() || out[i] <= 0. {
                    return Err("Invalid native scaling probe value".into());
                }
            }
            Ok(out)
        })();
        let b = io.write(game + self.game_field(0xc41), &flag);
        b?;
        result
    }

    pub fn native_difficulty_entries(self) -> Vec<Entry> {
        vec![]
    }
}
