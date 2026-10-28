//! Separate official-engine activation. This entry must never install Bridge,
//! custom peer networking, old lobby hooks or legacy gameplay suppression.
mod bootstrap;
pub(crate) use bootstrap::install;
mod menu_pause;
mod pause;
mod session;
mod lifecycle;
mod combat;
mod elites;
mod experience;
mod xp_hooks;
#[cfg(feature = "test-hooks")]
mod fixture;
#[cfg(feature = "test-hooks")]
mod performance;
mod io;
mod party_ui;
mod fatigue;
mod blessings;
mod glyphs;
mod bonus_tooltip;
mod bonus_ui;
mod bonus_catalog;
mod reward_cards;
mod reward_services;
mod run_stats;
mod enchantments;
mod gems;
mod menu_ui;
#[cfg(feature="test-hooks")]
mod presentation;
mod addon_activation;
mod support;
mod wallet;
mod duplicate_loot;
use crate::{log::Log, proc::Local};
use asx_game::official::SelectedAdapter as Adapter;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    OnceLock,
};
struct Runtime {
    adapter: Adapter,
    local: Local,
    log: &'static Log,
    rules: bool,
    sampled: std::sync::Mutex<std::time::Instant>,
    party: std::sync::Mutex<Option<asx_game::official::PartyObservation>>,
    solo: std::sync::Mutex<Option<asx_game::official::PartyObservation>>,
}
impl Runtime {
    fn extension_party(&self)->Option<asx_game::official::PartyObservation>{
        self.party.lock().ok().and_then(|p|p.clone()).or_else(||self.solo.lock().ok().and_then(|p|p.clone()))
    }
}
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn local_base() -> usize {
    RUNTIME
        .get()
        .expect("Official runtime installed before hooks")
        .local
        .base
}

pub(crate) fn restore_addon_activation(){addon_activation::restore();}

mod raid_markers;

mod combat_text;

mod nameplates;
