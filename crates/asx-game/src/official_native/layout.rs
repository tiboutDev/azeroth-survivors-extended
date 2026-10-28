//! Native field translation for the sole supported official client, 0.122.0.
use super::Adapter;
use crate::{memory::Mem, Result};

impl Adapter {
    pub(super) fn game_field(self, old: u64) -> u64 {
        crate::official122::game_field(old)
    }

    pub(super) fn row_field(self, host: bool, old: u64) -> u64 {
        if host {
            self.game_field(old)
        } else {
            crate::official122::actor_field(old)
        }
    }

    pub(super) fn context_size(self) -> u64 {
        crate::official122::CONTEXT_SIZE
    }

    pub(super) fn read_game(self, mem: &dyn Mem, game: u64) -> Result<Vec<u8>> {
        self.read_context(mem, game, true)
    }

    pub(super) fn read_context(self, mem: &dyn Mem, address: u64, host: bool) -> Result<Vec<u8>> {
        let size = if host { 0xc48 } else { 0x980 };
        let native_size = if host {
            self.game_field((size - 1) as u64) as usize + 1
        } else {
            crate::official122::CONTEXT_SIZE as usize
        };
        let raw = mem.read(address, native_size)?;
        if raw.len() != native_size {
            return Err("Truncated selected native context".into());
        }
        let mut normalized = vec![0; size];
        let spans = if host {
            crate::official122::GAME_COPY
        } else {
            crate::official122::ACTOR_COPY
        };
        for &(start, end, native) in spans {
            let input = raw
                .get(native..native + end - start)
                .ok_or("Selected native field outside context")?;
            normalized[start..end].copy_from_slice(input);
        }
        Ok(normalized)
    }
}
