//! Native pause commands and reliable coop transport for the exact selected build.
use super::*;
use crate::official::Entry;
impl Adapter {
    pub fn menu_key_entry(self) -> Option<Entry> {
        Some(Entry {
            rva: crate::current::MENU_KEY.0,
            bytes: crate::current::MENU_KEY.1,
        })
    }
    pub fn menu_key_target(self) -> Option<u64> {
        Some(crate::current::MENU_KEY_NATIVE_PAUSE)
    }
    /// UI input prefilter: guests consume Pause before native apply dispatch.
    /// The hook must preserve its bool result as well as both borrowed inputs.
    pub fn menu_queue_entry(self) -> Entry {
        Entry { rva: crate::current::MENU_UI_QUEUE.0, bytes: crate::current::MENU_UI_QUEUE.1 }
    }
    pub fn menu_escape_queue_return(self) -> u64 { crate::current::MENU_ESCAPE_QUEUE_RETURN }
    pub fn menu_entry(self) -> Option<Entry> {
        Some(Entry {
            rva: crate::current::MENU_INPUT.0,
            bytes: crate::current::MENU_INPUT.1,
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
        let server_state = self.observe(m, server)?;
        if server_state.role != Role::Host {
            return Err("Missing menu authority".into());
        }
        if peer >= u64::from(server_state.remote_connections) {
            return Err("Menu peer departed".into());
        }
        let pointer_field = server.checked_add(crate::current::SERVER_PEER_PTR)
            .ok_or("Menu peer table overflow")?;
        let pointer = u64::from_le_bytes(m.read(pointer_field, 8)?.try_into()
            .map_err(|_| "Truncated menu peer table")?);
        // peer is the command's vector index. Only the native row supplies its slot.
        let row = pointer.checked_add(peer * crate::current::SERVER_PEER_STRIDE)
            .filter(|p| pointer >= 0x10000 && p.checked_add(16).is_some())
            .ok_or("Invalid menu peer table")?;
        let header: [u8; 16] = m.read(row, 16)?.try_into()
            .map_err(|_| "Truncated menu peer")?;
        if header[..4] != 1u32.to_le_bytes() {
            return Err("Inactive menu peer".into());
        }
        let slot = u64::from_le_bytes(header[8..].try_into().unwrap());
        if !(1..NATIVE_PLAYERS as u64).contains(&slot) {
            return Err("Invalid menu peer slot".into());
        }
        Ok(slot as u32)
    }
    pub unsafe fn menu_send(self, m: &dyn Mem, base: u64, server: u64, bytes: &[u8]) -> Result<()> {
        if bytes.len() != 9 || self.observe(m, server)?.role != Role::Guest {
            return Err("Invalid menu transport owner".into());
        }
        let (at, expected) = crate::current::MENU_SEND;
        if m.read(base + at, expected.len())? != expected {
            return Err("Native menu transport changed".into());
        }
        type Send = unsafe extern "system" fn(u64, u64, u16, *const u8, usize);
        let call: Send = std::mem::transmute((base + at) as usize);
        call(
            server,
            0,
            crate::current::MENU_OPCODE,
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
        Some(crate::current::MENU_OPCODE)
    }
}

#[cfg(feature = "test-hooks")]
impl Adapter {
    pub unsafe fn fixture_menu_apply(
        self,
        m: &dyn Mem,
        base: u64,
        server: u64,
        action: u64,
    ) -> Result<()> {
        let (rva, bytes) = crate::current::MENU_INPUT_APPLY;
        if m.read(base + rva, bytes.len())? != bytes {
            return Err("Native menu apply changed".into());
        }
        let call: unsafe extern "system" fn(u64, u64) = std::mem::transmute((base + rva) as usize);
        call(server, action);
        Ok(())
    }
}
