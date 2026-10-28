//! Bounded native and cached observations for menu ownership.
use super::super::{Runtime, RUNTIME};
use asx_game::official::{PartyObservation, Role};

pub(in crate::runtime) trait Services {
    fn key_target(&self) -> Option<usize>;
    fn has_menu(&self) -> bool;
    fn role(&self, server: u64) -> Result<Role, String>;
    fn cached_party(&self) -> Option<PartyObservation>;
    fn game_address(&self, server: u64) -> Result<u64, String>;
    fn send(&self, server: u64, bytes: &[u8]) -> Result<(), String>;
    fn request(&self, action: u64) -> Result<Option<bool>, String>;
    fn opcode(&self) -> Option<u16>;
    fn peer(&self, server: u64, peer: u64) -> Result<u32, String>;
    fn wire(&self, stack: u64) -> Result<(u64, bool), String>;
    fn party(&self, server: u64) -> Result<Option<PartyObservation>, String>;
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
}

pub(super) fn active() -> Option<&'static dyn Services> {
    RUNTIME.get().filter(|runtime| runtime.rules).map(|runtime| runtime as &dyn Services)
}

impl Services for Runtime {
    fn key_target(&self) -> Option<usize> {
        self.adapter.menu_key_target().map(|at| self.local.base + at as usize)
    }
    fn has_menu(&self) -> bool { self.adapter.menu_entry().is_some() }
    fn role(&self, server: u64) -> Result<Role, String> {
        self.adapter.observe(&self.local, server).map(|state| state.role)
    }
    fn cached_party(&self) -> Option<PartyObservation> {
        self.party.lock().ok().and_then(|party| party.clone())
    }
    fn game_address(&self, server: u64) -> Result<u64, String> {
        self.adapter.game_address(server)
    }
    fn send(&self, server: u64, bytes: &[u8]) -> Result<(), String> {
        unsafe { self.adapter.menu_send(&self.local, self.local.base as u64, server, bytes) }
    }
    fn request(&self, action: u64) -> Result<Option<bool>, String> {
        self.adapter.menu_request(&self.local, action)
    }
    fn opcode(&self) -> Option<u16> { self.adapter.menu_opcode() }
    fn peer(&self, server: u64, peer: u64) -> Result<u32, String> {
        self.adapter.menu_peer(&self.local, server, peer)
    }
    fn wire(&self, stack: u64) -> Result<(u64, bool), String> {
        self.adapter.menu_wire(&self.local, stack)
    }
    fn party(&self, server: u64) -> Result<Option<PartyObservation>, String> {
        self.adapter.party(&self.local, server)
    }
    fn info(&self, message: &str) { self.log.info(message); }
    fn warn(&self, message: &str) { self.log.warn(message); }
}
