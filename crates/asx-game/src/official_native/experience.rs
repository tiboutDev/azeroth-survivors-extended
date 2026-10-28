//! Native shared-XP dead guard and restored-context catch-up, exact 0.115 only.
use super::Adapter;
use crate::{
    official::{Entry, SupportCalls},
    memory::Mem,
    Result,
};
impl Adapter {
    pub fn deferred_xp_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x58f9f29),
                bytes: self.expected(0x58f9f29, &[0x85, 0xf6, 0x0f, 0x84, 0xdb, 0, 0, 0]),
        }
    }
    pub fn deferred_xp_actor(self, mem: &dyn Mem, game: u64) -> Result<Option<(u32, u64)>> {
        if mem.read(game + self.game_field(0xc41), 1)? != [1] || mem.read(game + self.game_field(0xc46), 1)? != [4] {
            return Ok(None);
        }
        let slot = u64::from_le_bytes(
            mem.read(game + self.game_field(0xb40), 8)?
                .try_into()
                .map_err(|_| "Truncated deferred slot")?,
        );
        let guid = u64::from_le_bytes(
            mem.read(game + self.game_field(0x478), 8)?
                .try_into()
                .map_err(|_| "Truncated deferred identity")?,
        );
        if slot as usize >= crate::official_native::NATIVE_PLAYERS || guid == 0 {
            return Err("Invalid deferred XP actor".into());
        }
        Ok(Some((slot as u32, guid)))
    }
    pub fn replay_experience(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        guid: u64,
        amount: u32,
    ) -> Result<bool> {
        if amount == 0 {
            return Ok(false);
        }
        if io.read(game + self.game_field(0xc41), 1)? != [1] || io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("XP replay requires restored native coop host".into());
        }
        let row = self.support_row(io, game, slot)?;
        if io.read(row + self.row_field(row == game, 0x478), 8)? != guid.to_le_bytes()
            || io.read(row + self.row_field(slot == 0, if slot == 0 { 0xc46 } else { 0x97c }), 1)? != [1]
        {
            return Ok(false);
        }
        let xp = u32::from_le_bytes(
            io.read(row + self.row_field(row == game, 0x4f4), 4)?
                .try_into()
                .map_err(|_| "Truncated XP")?,
        );
        xp.checked_add(amount).ok_or("XP replay overflow")?;
        if slot != 0 {
            io.swap_actor(game, row)?;
            io.write(game + self.game_field(0xb40), &u64::from(slot).to_le_bytes())?;
        }
        let result = io
            .award_experience(game, amount)
            .and_then(|_| io.flush_actor(game));
        // Rebind after callbacks: award/flush may replace the actor vector.
        if slot != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result.map(|_| true)
    }
}
