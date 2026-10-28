//! Selected native/UI access for pause callbacks; policy stays in pause.rs.
use super::super::{lifecycle, Runtime, RUNTIME};
use asx_gameplay::official::PausePresentation;
use serde_json::Value;

pub(super) trait Services {
    fn read_ui(&self, pointer: u64, length: u64) -> Result<Value, String>;
    fn decode_pause(&self, value: &Value) -> Result<Option<PausePresentation>, String>;
    fn pause_notice(&self, active: bool) -> Option<&'static str>;
    fn hud_phase(&self, value: &Value) -> Result<asx_gameplay::Phase, String>;
    fn duplicate_loot_enabled(&self) -> bool;
    fn end_session(&self);
    fn warn(&self, message: &str);
    #[cfg(feature = "test-hooks")]
    fn economy_hud(&self, value: &Value);
    #[cfg(feature = "test-hooks")]
    fn fixture_event(&self, context: &mut crate::stub::Context, value: &Value);
}

pub(super) fn active() -> Option<&'static dyn Services> {
    RUNTIME.get().filter(|runtime| runtime.rules).map(|runtime| runtime as &dyn Services)
}

/// Native notice forwarding retains its historical runtime-present gate even
/// when gameplay rules are disabled. The UI callbacks themselves use active().
pub(super) fn available() -> Option<&'static dyn Services> {
    RUNTIME.get().map(|runtime| runtime as &dyn Services)
}

impl Services for Runtime {
    fn read_ui(&self, pointer: u64, length: u64) -> Result<Value, String> {
        self.adapter.read_ui(&self.local, pointer, length)
    }
    fn decode_pause(&self, value: &Value) -> Result<Option<PausePresentation>, String> {
        self.adapter.decode_pause(value)
    }
    fn pause_notice(&self, active: bool) -> Option<&'static str> {
        self.adapter.pause_notice(active)
    }
    fn hud_phase(&self, value: &Value) -> Result<asx_gameplay::Phase, String> {
        self.adapter.hud_phase(value)
    }
    fn duplicate_loot_enabled(&self) -> bool { self.adapter.duplicate_loot_enabled() }
    fn end_session(&self) { lifecycle::end(self); }
    fn warn(&self, message: &str) { self.log.warn(message); }
    #[cfg(feature = "test-hooks")]
    fn economy_hud(&self, value: &Value) { super::super::fixture::economy_hud(self, value); }
    #[cfg(feature = "test-hooks")]
    fn fixture_event(&self, context: &mut crate::stub::Context, value: &Value) {
        super::super::fixture::event(self, context, value);
    }
}
