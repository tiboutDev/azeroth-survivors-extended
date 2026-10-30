//! Bounded native and feature operations used by session callback policy.
use std::sync::atomic::{AtomicU64, Ordering};
use super::super::{
    blessings, combat, duplicate_loot, elites, enchantments, experience, fatigue, gems, glyphs, io,
    menu_pause, party_ui, pause, support, wallet, Runtime, RUNTIME,
};
use asx_game::official::Observation;
use asx_gameplay::official::{PausePresentation, PauseReason};
use serde_json::Value;

static HUD_PHASES: [AtomicU64; 5] = [const { AtomicU64::new(0) }; 5];

pub(super) trait WorldControl {
    fn rules_enabled(&self) -> bool;
    fn observe_world(&self, game: u64) -> Result<Option<PauseReason>, String>;
    fn capture_world(&self, game: u64);
    fn finish_world(&self, game: u64);
    fn log_group_pause(&self, reason: &Option<PauseReason>);
    fn warn(&self, message: &str);
}

pub(super) trait WorldServices: WorldControl {
    fn duplicate_enabled(&self) -> bool;
    fn observe_role(&self, server: u64) -> Result<Observation, String>;
    fn begin_duplicates(&self, server: u64);
    fn refresh_host(&self, server: u64);
    fn finish_experience(&self, server: u64);
    #[cfg(feature = "test-hooks")]
    fn fixture_before_update(&self, server: u64);
    #[cfg(feature = "test-hooks")]
    fn fixture_after_update(&self, server: u64);
    fn finish_duplicates(&self, server: u64);
    fn emit_diagnostics(&self, server: u64);
}

pub(super) trait HudServices {
    fn rules_enabled(&self) -> bool;
    fn solo_active(&self) -> bool;
    fn hud_snapshot(&self, game: u64, output: u64) -> Result<Option<(u32, Value)>, String>;
    fn capture_party(&self, actor: u32, value: &Value);
    fn effect_owner(&self, game: u64) -> Option<(u32, u64)>;
    fn hud_phase(&self, value: &Value) -> Result<asx_gameplay::Phase, String>;
    fn update_pause_party(
        &self,
        game: u64,
        owner: Option<(u32, u64)>,
        phase: Option<asx_gameplay::Phase>,
    ) -> (Option<PauseReason>, Vec<(u32, u64)>);
    fn decorate_pause(&self, value: &mut Value, view: &PausePresentation);
    fn replace_hud(&self, output: u64, value: &Value) -> Result<(), String>;
    fn first_hud_phase(&self, actor: u32, value: &Value) -> bool;
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
}

pub(super) trait CommandServices {
    fn rules_enabled(&self) -> bool;
    fn duplicate_enabled(&self) -> bool;
    fn menu_command(&self, context: &mut crate::stub::Context) -> bool;
    fn duplicate_command(&self, context: &mut crate::stub::Context) -> bool;
    fn movement_opcode(&self, opcode: u16) -> bool;
    fn active_role(&self, receiver: u64) -> bool;
}

pub(super) fn world() -> Option<&'static dyn WorldServices> {
    RUNTIME.get().map(|runtime| runtime as &dyn WorldServices)
}
pub(super) fn world_control() -> Option<&'static dyn WorldControl> {
    RUNTIME.get().map(|runtime| runtime as &dyn WorldControl)
}
pub(super) fn hud() -> Option<&'static dyn HudServices> {
    RUNTIME.get().map(|runtime| runtime as &dyn HudServices)
}
pub(super) fn command() -> Option<&'static dyn CommandServices> {
    RUNTIME.get().map(|runtime| runtime as &dyn CommandServices)
}

impl WorldControl for Runtime {
    fn rules_enabled(&self) -> bool { self.rules }
    fn observe_world(&self, game: u64) -> Result<Option<PauseReason>, String> {
        let party = self.adapter.world_party(&self.local, game)?;
        fatigue::observe(game, party.as_ref());
        blessings::observe(game, party.as_ref());
        glyphs::observe(game, party.as_ref());
        enchantments::observe(game, party.as_ref());
        gems::observe(game, party.as_ref());
        if let Err(error) = wallet::boundary(self, game, party.as_ref()) {
            self.log.warn(&format!("Official wallet boundary rejected: {error}"));
        }
        if let Ok(mut cached) = self.party.lock() {
            *cached = party.clone();
        }
        Ok(party.and_then(|value| menu_pause::reason(self, game, &value)))
    }
    fn capture_world(&self, game: u64) {
        support::capture(self, game);
        combat::capture(self, game);
    }
    fn finish_world(&self, game: u64) {
        combat::finish(self, game);
        support::finish(self, game);
        elites::finish(self, game);
        blessings::finish(self, game);
        super::super::treasure_goblins::finish(self, game);
        super::super::world_events::coop::finish(self,game);
    }
    fn log_group_pause(&self, reason: &Option<PauseReason>) {
        self.log.info(&format!("official group pause: {reason:?}"));
    }
    fn warn(&self, message: &str) { self.log.warn(message); }
}

impl WorldServices for Runtime {
    fn duplicate_enabled(&self) -> bool { self.adapter.duplicate_loot_enabled() }
    fn observe_role(&self, server: u64) -> Result<Observation, String> {
        self.adapter.observe(&self.local, server)
    }
    fn begin_duplicates(&self, server: u64) { duplicate_loot::begin(self, server); }
    fn refresh_host(&self, server: u64) {
        let Ok(game) = self.adapter.game_address(server) else { return };
        let snapshot = self.adapter.party(&self.local, server).ok().flatten();
        fatigue::observe(game, snapshot.as_ref());
        blessings::observe(game, snapshot.as_ref());
        glyphs::observe(game, snapshot.as_ref());
        enchantments::observe(game, snapshot.as_ref());
        gems::observe(game, snapshot.as_ref());
        if let Err(error) = wallet::boundary(self, game, snapshot.as_ref()) {
            self.log.warn(&format!("Official wallet boundary rejected: {error}"));
        }
        let opened = snapshot.as_ref().is_some_and(|party| wallet::present(self, game, party));
        let snapshot = if opened { self.adapter.party(&self.local, server).ok().flatten() } else { snapshot };
        super::super::raid_markers::observe(self,server,snapshot.as_ref());
        let reason = snapshot.as_ref().and_then(|party| menu_pause::reason(self, game, party));
        let active = reason.is_some();
        if pause::set_active(active) != active {
            self.log.info(&format!("official group pause: {reason:?}"));
        }
        if let Ok(mut cached) = self.party.lock() { *cached = snapshot; }
    }
    fn finish_experience(&self, server: u64) {
        if let Ok(game) = self.adapter.game_address(server) {
            experience::finish(self, game);
        }
    }
    #[cfg(feature = "test-hooks")]
    fn fixture_before_update(&self, server: u64) { menu_pause::fixture(self, server); }
    #[cfg(feature = "test-hooks")]
    fn fixture_after_update(&self, server: u64) {
        if !super::super::performance::enabled() { super::super::fixture::tick(self, server); }
    }
    fn finish_duplicates(&self, server: u64) { duplicate_loot::finish(self, server); }
    fn emit_diagnostics(&self, server: u64) {
        let Ok(mut sampled) = self.sampled.try_lock() else { return };
        if sampled.elapsed().as_secs() < 1 { return; }
        *sampled = std::time::Instant::now();
        #[cfg(feature = "test-hooks")]
        super::super::performance::report(self.log);
        #[cfg(feature = "test-hooks")]
        let _timing = super::super::performance::start(6);
        if self.rules {
            self.log.info(&format!("official pause UI: {}", crate::presentation::pause::evidence()));
        }
        match self.adapter.observe(&self.local, server) {
            Ok(state) => {
                self.log.info(&format!("official observation: role={:?} guest_game_stream={:?} remote_connections={} updates={}", state.role, state.guest_game_stream, state.remote_connections, super::CALLS.load(Ordering::Relaxed)));
                match self.adapter.party(&self.local, server) {
                    Ok(Some(party)) => self.log.info(&format!("official party: clock={} actors={:?}", party.clock, party.actors)),
                    Ok(None) => {},
                    Err(reason) => self.log.warn(&format!("official party unavailable: {reason}")),
                }
            }
            Err(reason) => self.log.warn(&format!("official observation unavailable: {reason}")),
        }
    }
}

impl HudServices for Runtime {
    fn rules_enabled(&self) -> bool { self.rules }
    fn solo_active(&self) -> bool { self.solo.lock().ok().is_some_and(|value| value.is_some()) }
    fn hud_snapshot(&self, game: u64, output: u64) -> Result<Option<(u32, Value)>, String> {
        let Some((actor, _)) = self.adapter.run_source(&self.local, game)? else { return Ok(None) };
        Ok(Some((actor, self.adapter.hud_value(&self.local, output)?)))
    }
    fn capture_party(&self, actor: u32, value: &Value) { party_ui::capture(self, actor, value); }
    fn effect_owner(&self, game: u64) -> Option<(u32, u64)> {
        self.adapter.effect_source(&self.local, game).ok().flatten()
    }
    fn hud_phase(&self, value: &Value) -> Result<asx_gameplay::Phase, String> {
        self.adapter.hud_phase(value)
    }
    fn update_pause_party(&self, game: u64, owner: Option<(u32, u64)>, phase: Option<asx_gameplay::Phase>) -> (Option<PauseReason>, Vec<(u32, u64)>) {
        let Some(mut party) = self.party.lock().ok() else { return (None, Vec::new()) };
        let Some(observation) = party.as_mut() else { return (None, Vec::new()) };
        if let (Some((slot, guid)), Some(phase)) = (owner, phase) {
            if let Some(actor) = observation.actors.iter_mut().find(|actor| actor.slot == slot && actor.guid == guid) {
                actor.phase = phase;
            }
        }
        let reason = menu_pause::reason(self, game, observation);
        let identities = if reason.is_some() {
            observation.actors.iter().map(|actor| (actor.slot + 1, actor.guid)).collect()
        } else { Vec::new() };
        (reason, identities)
    }
    fn decorate_pause(&self, value: &mut Value, view: &PausePresentation) {
        self.adapter.decorate_pause(value, view);
    }
    fn replace_hud(&self, output: u64, value: &Value) -> Result<(), String> {
        self.adapter.replace_hud(&io::Access { local: &self.local, adapter: self.adapter }, output, value)
    }
    fn first_hud_phase(&self, actor: u32, value: &Value) -> bool {
        let phase = match value["phase"].as_str().unwrap_or("unknown") {
            "playing" => 1, "choosing" => 2, "paused" => 4, "dead" => 8, "victory" => 16, _ => 32,
        };
        HUD_PHASES[actor as usize].fetch_or(phase, Ordering::Relaxed) & phase == 0
    }
    fn info(&self, message: &str) { self.log.info(message); }
    fn warn(&self, message: &str) { self.log.warn(message); }
}

impl CommandServices for Runtime {
    fn rules_enabled(&self) -> bool { self.rules }
    fn duplicate_enabled(&self) -> bool { self.adapter.duplicate_loot_enabled() }
    fn menu_command(&self, context: &mut crate::stub::Context) -> bool { menu_pause::command(self, context) }
    fn duplicate_command(&self, context: &mut crate::stub::Context) -> bool { duplicate_loot::command(self, context) }
    fn movement_opcode(&self, opcode: u16) -> bool { self.adapter.command_is_movement(opcode) }
    fn active_role(&self, receiver: u64) -> bool {
        self.adapter.observe(&self.local, receiver).is_ok_and(|state| matches!(state.role, asx_game::official::Role::Host | asx_game::official::Role::Guest))
    }
}
