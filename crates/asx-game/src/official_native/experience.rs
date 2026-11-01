//! Selected native dead-actor XP guards.
use super::Adapter;
use crate::{
    official::Entry,
    memory::Mem,
    Result,
};
impl Adapter {
    pub fn downed_xp_entries(self) -> Vec<Entry> {
        crate::current::DOWNED_XP_GUARDS.iter().flat_map(|g| [
            Entry { rva: g.0, bytes: g.2 }, Entry { rva: g.1, bytes: g.3 }
        ]).collect()
    }
    /// Native actors are temporarily borrowed into Game, including remote
    /// actors. Never traverse the lent-out player vector at this boundary.
    /// Both shared XP receipts and queued whole-level rewards must be consumed
    /// while dead, before later XP changes their native level-threshold deficit.
    pub fn downed_xp_continuation(self, mem: &dyn Mem, kind: usize, registers: [u64; 6]) -> Result<Option<u64>> {
        let (game, amount) = match kind {
            0 | 2 => (registers[0], registers[2]), // R13 / RSI
            1 => (registers[1], registers[2]),     // R9 / RSI
            3 => (registers[2], registers[3]),     // RSI / RBX
            4 => (registers[2], registers[4]),     // RSI / RDI
            5 => (registers[0], 1), // Host queued whole-level producer
            6 => (registers[2], 1), // Borrowed remote whole-level producer
            _ => return Err("Unknown native XP guard".into()),
        };
        if amount as u32 == 0 || game < 0x10000 { return Ok(None); }
        let server = game.checked_sub(0x10).ok_or("Missing XP authority")?;
        if self.observe(mem, server)?.role != super::Role::Host { return Ok(None); }
        let read = |offset, size| mem.read(game.checked_add(self.game_field(offset)).ok_or("XP field overflow")?, size);
        if read(0xc41, 1)? != [1] || read(0xc46, 1)? != [4] { return Ok(None); }
        let slot = u64::from_le_bytes(read(0xb40, 8)?.try_into().map_err(|_| "Truncated XP slot")?);
        let guid = u64::from_le_bytes(read(0x478, 8)?.try_into().map_err(|_| "Truncated XP identity")?);
        if slot >= crate::official_native::NATIVE_PLAYERS as u64 || guid == 0 { return Err("Invalid downed XP actor".into()); }
        if (2..5).contains(&kind) {
            let at=game.checked_add(crate::current::DOWNED_XP_PENDING_FIELD).ok_or("Pending level overflow")?;
            let pending=u32::from_le_bytes(mem.read(at,4)?.try_into().map_err(|_| "Truncated pending levels")?);
            if pending == 0 { return Ok(None); }
        }
        if kind >= 5 {
            let at=if kind==5 {
                registers[5].checked_add(crate::current::DOWNED_XP_HOST_REWARD_SLOT).ok_or("Host reward frame overflow")?
            } else {
                let field=registers[4].checked_add(8).ok_or("Remote reward owner overflow")?;
                u64::from_le_bytes(mem.read(field,8)?.try_into().map_err(|_| "Truncated reward owner")?)
            };
            if at<0x10000 { return Err("Invalid native whole-level receipt".into()); }
            let amount=u32::from_le_bytes(mem.read(at,4)?.try_into().map_err(|_| "Truncated whole-level receipt")?);
            if amount==0 { return Ok(None); }
        }
        Ok(Some(crate::current::DOWNED_XP_GUARDS[kind].1))
    }

}
