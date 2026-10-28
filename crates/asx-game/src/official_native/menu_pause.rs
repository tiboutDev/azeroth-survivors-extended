//! Native pause commands and reliable coop transport for the exact selected build.
use super::*;
use crate::official::Entry;
impl Adapter {
    pub fn menu_key_entry(self) -> Option<Entry> {
        Some(Entry {
            rva: crate::official125::MENU_KEY.0,
            bytes: crate::official125::MENU_KEY.1,
        })
    }
    pub fn menu_key_target(self) -> Option<u64> {
        Some(crate::official125::MENU_KEY_NATIVE_PAUSE)
    }
    pub fn menu_entry(self) -> Option<Entry> {
        Some(Entry {
            rva: crate::official125::MENU_INPUT.0,
            bytes: crate::official125::MENU_INPUT.1,
        })
    }
    pub fn pause_notice(self, active: bool) -> Option<&'static str> {
        Some(if active {
            "{\"type\":\"paused\"}"
        } else {
            "{\"type\":\"unpaused\"}"
        })
    }
    pub fn menu_request(self, m: &dyn Mem, p: u64) -> Result<Option<bool>> {
        if p < 0x10000 {
            return Ok(None);
        }
        if m.read(p, 4)? != 7u32.to_le_bytes() {
            return Ok(None);
        }
        match m.read(p + 4, 1)?.as_slice() {
            [0] => Ok(Some(false)),
            [1] => Ok(Some(true)),
            _ => Err("Invalid native pause request".into()),
        }
    }
    pub fn menu_peer(self, m: &dyn Mem, server: u64, peer: u64) -> Result<u32> {
        if self.observe(m, server)?.role != Role::Host {
            return Err("Missing menu authority".into());
        }
        let q = |a| -> Result<u64> {
            Ok(u64::from_le_bytes(
                m.read(a, 8)?
                    .try_into()
                    .map_err(|_| "Truncated menu peer")?,
            ))
        };
        if peer >= q(server + crate::official125::SERVER_PEERS)? {
            return Err("Menu peer departed".into());
        }
        let row = q(server + crate::official125::SERVER_PEER_PTR)?
            + peer * crate::official125::SERVER_PEER_STRIDE;
        if m.read(row, 4)? != 1u32.to_le_bytes() {
            return Err("Inactive menu peer".into());
        }
        let slot = q(row + 8)?;
        if !(1..5).contains(&slot) {
            return Err("Invalid menu peer slot".into());
        }
        Ok(slot as u32)
    }
    pub unsafe fn menu_send(self, m: &dyn Mem, base: u64, server: u64, bytes: &[u8]) -> Result<()> {
        if bytes.len() != 9
            || self.observe(m, server)?.role != Role::Guest
        {
            return Err("Invalid menu transport owner".into());
        }
        let (at, expected) = crate::official125::MENU_SEND;
        if m.read(base + at, expected.len())? != expected {
            return Err("Native menu transport changed".into());
        }
        type Send = unsafe extern "system" fn(u64, u64, u16, *const u8, usize);
        let call: Send = std::mem::transmute((base + at) as usize);
        call(
            server,
            0,
            crate::official125::MENU_OPCODE,
            bytes.as_ptr(),
            bytes.len(),
        );
        Ok(())
    }
    pub fn menu_wire(self, m: &dyn Mem, stack: u64) -> Result<(u64, bool)> {
                let args = m.read(stack + 40, 16)?;
        let ptr = u64::from_le_bytes(args[..8].try_into().unwrap());
        let len = u64::from_le_bytes(args[8..].try_into().unwrap());
        if ptr < 0x10000 || len != 9 {
            return Err("Invalid menu request extent".into());
        }
        let raw = m.read(ptr, 9)?;
        let epoch = u64::from_le_bytes(raw[..8].try_into().unwrap());
        if epoch == 0 || raw[8] > 1 {
            return Err("Invalid menu request payload".into());
        }
        Ok((epoch, raw[8] != 0))
    }
    pub fn menu_opcode(self) -> Option<u16> {
        Some(crate::official125::MENU_OPCODE)
    }
}
