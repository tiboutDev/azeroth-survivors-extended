//! A delayed native autopick for one disposable client. No pointer survives a
//! callback: rebind the exact UI owner at the server boundary before replay.
use super::*;
use std::time::{Duration, Instant};
#[derive(Default)]
struct State {
    pending: Option<(Vec<u8>, Instant)>,
    done: bool,
}
static STATE: std::sync::Mutex<State> = std::sync::Mutex::new(State {
    pending: None,
    done: false,
});
fn delay() -> Option<Duration> {
    std::env::var("AZCOOP_TEST_OFFICIAL_CHOICE_DELAY")
        .ok()?
        .parse::<u64>()
        .ok()
        .filter(|n| (2..=if glyphs::retention_fixture(){40}else{20}).contains(n))
        .map(Duration::from_secs)
}
pub fn event(runtime: &Runtime, c: &crate::stub::Context, value: &serde_json::Value) {
    if compat121()&&!value["choices"].as_array().is_some_and(|a|a.iter().any(|c|c["lines"].as_str().is_some_and(|s|s.contains("Gem:")||s.contains("Enchantment:")))){return;}
    if pause_enabled(){pause_event(runtime,c,value);return;}
    if glyphs::retention_fixture()&&!value["choices"].as_array().is_some_and(|a|a.iter().any(|c|c["lines"].as_str().is_some_and(|s|s.contains("Keeps glyph:")))){return;}
    if !compat121() && gems::fixture_enabled() && !value["choices"].as_array().is_some_and(|a|a.iter().any(|c|c["lines"].as_str().is_some_and(|s|s.contains("Gem:")))){return;}
    if !compat121() && enchantments::fixture_enabled() && !value["choices"].as_array().is_some_and(|a|a.iter().any(|c|c["lines"].as_str().is_some_and(|s|s.contains("Enchantment:")))){return;}
    if std::env::var("ASX_TEST_COVERAGE").as_deref()==Ok("1") && value["type"]=="choices" {
        runtime.log.info(&format!("coverage native choices: {value}"));
    }
    economy_event(runtime, c, value);
    let Some(delay) = delay() else { return };
    if !glyphs::retention_fixture() && std::env::var("AZCOOP_TEST_GLYPHS").as_deref()==Ok("1") && !value["choices"].as_array().is_some_and(|a|a.iter().any(|c|c["lines"].as_str().is_some_and(|s|s.contains("Glyph:")))) {return;}
    if value["type"] != "choices" {
        return;
    }
    let Ok(mut state) = STATE.lock() else { return };
    if state.done || state.pending.is_some() {
        return;
    }
    let Ok(bytes) = runtime.local.read(c.rdx as usize, c.r8 as usize) else {
        return;
    };
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    if runtime.adapter.set_ui_autopick(&io, c.rcx, false).is_ok() {
        state.pending = Some((bytes, Instant::now() + delay));
        runtime.log.info("official fixture: native choice delayed");
    }
}
pub fn tick(runtime: &Runtime, server: u64) {
    if std::env::var("ASX_TEST_UI_REMAINING").as_deref()==Ok("1"){remaining_ui_tick(runtime,server);return;}
    if completion121_enabled(){completion121_tick(runtime,server);if pause_enabled(){pause_tick(runtime,server);}return;}
    if compat121(){marks121_tick(runtime,server);release_pending(runtime,server);if let Ok(game)=runtime.adapter.game_address(server){blessing_tick(runtime,server);gems::fixture_tick(runtime,game);enchantments::fixture_tick(runtime,game);}return;}
    if combat::immunity_fixture_enabled(){combat::immunity_fixture_tick(runtime,server);return;}
    if wallet::fixture_enabled(){wallet::fixture_tick(runtime,server);return;}
    if glyphs::retention_fixture(){release_pending(runtime,server);glyphs::retention_tick(runtime,server);return;}
    if gems::fixture_enabled(){release_pending(runtime,server);if let Ok(game)=runtime.adapter.game_address(server){gems::fixture_tick(runtime,game);}return;}
    if enchantments::fixture_enabled(){release_pending(runtime,server);if let Ok(game)=runtime.adapter.game_address(server){enchantments::fixture_tick(runtime,game);}return;}

    if std::env::var("AZCOOP_TEST_EXTENSIONS").as_deref()==Ok("1") {
        glyphs::fixture_tick(runtime,server);blessing_tick(runtime,server);return;
    }
    if xp_death_enabled(){xp_death_tick(runtime,server);return;}
    if pause_enabled(){pause_tick(runtime,server);return;}
    armory_gold_tick(runtime, server);
    regression_tick(runtime, server);
    fatigue_tick(runtime, server);
    blessing_tick(runtime, server);
    glyphs::fixture_tick(runtime, server);
    economy_tick(runtime, server);
    support_tick(runtime, server);
    combat_tick(runtime, server);
    scaling_tick(runtime, server);
    party_ui_tick(runtime, server);
    release_pending(runtime,server);
}
fn release_pending(runtime:&Runtime,server:u64){
    let pending = {
        let Ok(mut state) = STATE.lock() else { return };
        if state
            .pending
            .as_ref()
            .is_none_or(|(_, until)| Instant::now() < *until)
        {
            return;
        }
        state.done = true;
        state.pending.take()
    };
    let Some((bytes, _)) = pending else { return };
    let Ok(receiver) =
        runtime
            .adapter
            .ui_receiver(&runtime.local, server, runtime.local.base as u64)
    else {
        runtime.log.warn("official fixture: UI owner unavailable");
        return;
    };
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    if runtime
        .adapter
        .set_ui_autopick(&io, receiver, true)
        .is_err()
    {
        return;
    }
    // Same callback and JSON as the native AUTOPICK path, using its original
    // trampoline. Consume fixture state before invoking potentially reentrant code.
    unsafe { pause::replay_event(receiver, &bytes) };
    runtime
        .log
        .info("official fixture: delayed native choice released");
}

static ECONOMY_STAGE: AtomicUsize = AtomicUsize::new(0);
static ECONOMY_CHOICE: std::sync::Mutex<Option<Vec<u8>>> = std::sync::Mutex::new(None);
static HUD_SECOND: AtomicU64 = AtomicU64::new(0);
fn economy_enabled() -> bool {
    std::env::var("AZCOOP_TEST_OFFICIAL_ECONOMY").as_deref() == Ok("1")
}
pub fn economy_hud(runtime: &Runtime, value: &serde_json::Value) {
    if std::env::var("AZCOOP_TEST_XP_HUD").as_deref()==Ok("1") {
        static SAMPLE:std::sync::Mutex<Option<Instant>>=std::sync::Mutex::new(None);
        let sample=if let Ok(mut at)=SAMPLE.lock(){if at.is_none_or(|t|t.elapsed()>=Duration::from_secs(2)){*at=Some(Instant::now());true}else{false}}else{false};
        if sample {runtime.log.info(&format!("XP received HUD: {value}"));}
    }
    if !economy_enabled() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if HUD_SECOND.swap(now, Ordering::Relaxed) != now {
        runtime.log.info(&format!(
            "official fixture HUD economy: phase={} gold={}",
            value["phase"], value["gold"]
        ));
    }
}
fn economy_event(runtime: &Runtime, c: &crate::stub::Context, value: &serde_json::Value) {
    if !economy_enabled() || value["type"] != "choices" {
        return;
    }
    runtime
        .log
        .info(&format!("official fixture choices: {value}"));
    if std::env::var("AZCOOP_TEST_OFFICIAL_ECONOMY_HOST").as_deref() != Ok("1")
        || ECONOMY_STAGE.load(Ordering::Acquire) != 1
        || value["why"] != "cache"
    {
        return;
    }
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    if runtime.adapter.set_ui_autopick(&io, c.rcx, false).is_ok() {
        if let Ok(mut pending) = ECONOMY_CHOICE.lock() {
            *pending = serde_json::to_vec(value).ok();
        }
        ECONOMY_STAGE.store(2, Ordering::Release);
    }
}
fn economy_tick(runtime: &Runtime, server: u64) {
    if !economy_enabled()
        || std::env::var("AZCOOP_TEST_OFFICIAL_ECONOMY_HOST").as_deref() != Ok("1")
    {
        return;
    }
    let Ok(Some(party)) = runtime.adapter.party(&runtime.local, server) else {
        return;
    };
    let Some(host) = party.actors.first() else {
        return;
    };
    let Ok(game) = runtime.adapter.game_address(server) else {
        return;
    };
    let stage = ECONOMY_STAGE.load(Ordering::Acquire);
    if stage == 0
        && party.clock > 3.0
        && host.phase == asx_gameplay::Phase::Playing
        && party.actors.len() == 3
    {
        ECONOMY_STAGE.store(1, Ordering::Release);
        type Grant = unsafe extern "system" fn(u64, u32) -> usize;
        type Chest = unsafe extern "system" fn(u64) -> bool;
        let grant: Grant = unsafe {
            std::mem::transmute(runtime.local.base + runtime.adapter.gold_entry().rva as usize)
        };
        let chest: Chest = unsafe {
            std::mem::transmute(runtime.local.base + runtime.adapter.chest_entry().rva as usize)
        };
        unsafe { grant(game, 101) };
        let success = unsafe { chest(game) };
        runtime.log.info(&format!(
            "official fixture group chest requested: success={success}"
        ));
        if !success {
            ECONOMY_STAGE.store(4, Ordering::Release);
        }
    } else if stage == 2 {
        ECONOMY_STAGE.store(3, Ordering::Release);
        let io = io::Access {
            local: &runtime.local,
            adapter: runtime.adapter,
        };
        let old = runtime
            .adapter
            .fixture_conversion_flag(&io, game, true)
            .ok();
        let ready = runtime
            .adapter
            .conversion_ready(&runtime.local, game)
            .unwrap_or(false);
        if ready {
            type Convert = unsafe extern "system" fn(u64);
            let convert: Convert = unsafe {
                std::mem::transmute(
                    runtime.local.base + runtime.adapter.convert_entry().rva as usize,
                )
            };
            unsafe { convert(game) };
        }
        if let Some(old) = old {
            let _ = runtime.adapter.fixture_conversion_flag(&io, game, old);
        }
        runtime.log.info(&format!(
            "official fixture reward converted: ready={ready} disposable_unlock={}",
            old == Some(false)
        ));
        let Ok(receiver) =
            runtime
                .adapter
                .ui_receiver(&runtime.local, server, runtime.local.base as u64)
        else {
            return;
        };
        let io = io::Access {
            local: &runtime.local,
            adapter: runtime.adapter,
        };
        let _ = runtime.adapter.set_ui_autopick(&io, receiver, true);
        let bytes = ECONOMY_CHOICE.lock().ok().and_then(|mut p| p.take());
        if !ready {
            if let Some(bytes) = bytes {
                unsafe { pause::replay_event(receiver, &bytes) };
            }
        }
    }
}

static SUPPORT_STAGE: AtomicUsize = AtomicUsize::new(0);
static SUPPORT_PROTOTYPE: std::sync::Mutex<[Option<Vec<u8>>; 3]> =
    std::sync::Mutex::new([None, None, None]);
static SUPPORT_CLOCK: std::sync::Mutex<f32> = std::sync::Mutex::new(0.0);
fn support_enabled() -> bool {
    std::env::var("AZCOOP_TEST_OFFICIAL_SUPPORT_HOST").as_deref() == Ok("1")
}
pub fn observe_support_cast(runtime: &Runtime, game: u64, ability: u64) {
    if (!support_enabled() && !party_ui_enabled() && !fatigue_enabled() && !completion121_enabled()) || SUPPORT_STAGE.load(Ordering::Relaxed) != 0 {
        return;
    }
    let Ok(actor) = runtime.adapter.support_caster(&runtime.local, game) else {
        return;
    };
    if actor.slot > 2 {
        return;
    }
    let Ok(bytes) = runtime.local.read(ability as usize, 0x68) else {
        return;
    };
    if let Ok(mut prototype) = SUPPORT_PROTOTYPE.lock() {
        if prototype[actor.slot as usize].is_none() {
            prototype[actor.slot as usize] = Some(bytes);
        }
    }
}
fn support_tick(runtime: &Runtime, server: u64) {
    if !support_enabled() {
        return;
    }
    let Ok(Some(party)) = runtime.adapter.party(&runtime.local, server) else {
        return;
    };
    if party.actors.len() != 3
        || party
            .actors
            .iter()
            .any(|a| a.phase != asx_gameplay::Phase::Playing)
    {
        return;
    }
    let prototype = SUPPORT_PROTOTYPE.lock().ok().and_then(|p| p[0].clone());
    let Some(mut prototype) = prototype else {
        return;
    };
    let Ok(game) = runtime.adapter.game_address(server) else {
        return;
    };
    let stage = SUPPORT_STAGE.load(Ordering::Acquire);
    if stage >= 6 {
        effects_tick(runtime, game, party.clock);
        return;
    }
    let clock = SUPPORT_CLOCK.lock().map(|v| *v).unwrap_or(0.0);
    if stage == 0 && ECONOMY_STAGE.load(Ordering::Acquire) >= 3 && party.clock > 8.0 {
        let io = io::Access {
            local: &runtime.local,
            adapter: runtime.adapter,
        };
        if let Err(error) = runtime.adapter.fixture_support_wounds(&io, game, 0, 2) {
            runtime
                .log
                .warn(&format!("Official support fixture rejected: {error}"));
            return;
        }
        if let Ok(mut clock) = SUPPORT_CLOCK.lock() {
            *clock = party.clock;
        }
        SUPPORT_STAGE.store(1, Ordering::Release);
        runtime
            .log
            .info("official support fixture: three wounded players");
    } else if stage == 1 && party.clock > clock {
        SUPPORT_STAGE.store(2, Ordering::Release);
        prototype[..8].copy_from_slice(
            &(runtime.local.base as u64
                + runtime
                    .adapter
                    .fixture_support_definition(635)
                    .expect("Reviewed fixture spell"))
            .to_le_bytes(),
        );
        let result = unsafe { support::cast(game, prototype.as_ptr() as u64) };
        runtime.log.info(&format!(
            "official support fixture direct cast: success={result}"
        ));
    } else if stage == 2 && party.clock > clock + 1.0 {
        SUPPORT_STAGE.store(3, Ordering::Release);
        prototype[..8].copy_from_slice(
            &(runtime.local.base as u64
                + runtime
                    .adapter
                    .fixture_support_definition(6673)
                    .expect("Reviewed fixture spell"))
            .to_le_bytes(),
        );
        let result = unsafe { support::cast(game, prototype.as_ptr() as u64) };
        runtime.log.info(&format!(
            "official support fixture shout cast: success={result}"
        ));
    } else if stage == 3 && party.clock > clock + 2.0 {
        SUPPORT_STAGE.store(4, Ordering::Release);
        let slots = runtime
            .adapter
            .support_buff_slots(&runtime.local, game, 6673);
        runtime.log.info(&format!(
            "official support fixture shout recipients: {slots:?}"
        ));
    } else if stage == 4 && party.clock > clock + 3.0 {
        let io = io::Access {
            local: &runtime.local,
            adapter: runtime.adapter,
        };
        if runtime
            .adapter
            .fixture_support_wounds(&io, game, 1, 0)
            .is_ok()
        {
            SUPPORT_STAGE.store(5, Ordering::Release);
            if let Ok(mut c) = SUPPORT_CLOCK.lock() {
                *c = party.clock;
            }
        }
    } else if stage == 5 && party.clock > clock {
        let guest = SUPPORT_PROTOTYPE.lock().ok().and_then(|p| p[1].clone());
        let Some(mut guest) = guest else {
            runtime
                .log
                .warn("Official support fixture rejected: missing guest cast prototype");
            SUPPORT_STAGE.store(6, Ordering::Release);
            return;
        };
        guest[..8].copy_from_slice(
            &(runtime.local.base as u64
                + runtime
                    .adapter
                    .fixture_support_definition(331)
                    .expect("Reviewed fixture spell"))
            .to_le_bytes(),
        );
        SUPPORT_STAGE.store(6, Ordering::Release);
        let io = io::Access {
            local: &runtime.local,
            adapter: runtime.adapter,
        };
        let result = runtime
            .adapter
            .fixture_cast_actor(&io, game, 1, guest.as_ptr() as u64);
        runtime.log.info(&format!(
            "official support fixture guest direct cast: {result:?}"
        ));
    }
}

// Small extension of the existing three-client support fixture. Each case uses
// a real native cast with an owned copy of that actor's observed parameters.
static EFFECT_STAGE: AtomicUsize = AtomicUsize::new(0);
static EFFECT_CLOCK: std::sync::Mutex<f32> = std::sync::Mutex::new(0.0);
fn effects_tick(runtime: &Runtime, game: u64, clock: f32) {
    let stage = EFFECT_STAGE.load(Ordering::Acquire);
    const CASES: [(u32, u32, u32); 8] = [
        (139, 0, 2),
        (774, 1, 0),
        (8936, 0, 2),
        (17, 1, 0),
        (467, 0, 2),
        (324, 1, 0),
        (15237, 0, 2),
        (25912, 1, 0),
    ];
    rapture_tick(runtime, game, clock);
    let index = stage / 3;
    if index >= CASES.len() {
        if stage == CASES.len() * 3 && clock > previous_guard_clock() + 2.0 && clock > 22.0 {
            let io = io::Access {
                local: &runtime.local,
                adapter: runtime.adapter,
            };
            let result = runtime.adapter.fixture_guardian(&io, game);
            runtime
                .log
                .info(&format!("official guardian fixture: {result:?}"));
            EFFECT_STAGE.store(stage + 1, Ordering::Release);
        }
        return;
    }
    let (spell, caster, target) = CASES[index];
    let previous = EFFECT_CLOCK.lock().map(|c| *c).unwrap_or(clock);
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    match stage % 3 {
        0 => {
            if stage > 0 && clock < previous + 2.0 {
                return;
            }
            if runtime
                .adapter
                .fixture_support_wounds(&io, game, caster, target)
                .is_err()
            {
                return;
            }
            if let Ok(mut c) = EFFECT_CLOCK.lock() {
                *c = clock;
            }
            EFFECT_STAGE.store(stage + 1, Ordering::Release);
        }
        1 if clock > previous => {
            let prototype = SUPPORT_PROTOTYPE
                .lock()
                .ok()
                .and_then(|p| p[caster as usize].clone());
            let Some(mut ability) = prototype else { return };
            let definition = runtime.adapter.fixture_support_definition(spell).unwrap();
            ability[..8].copy_from_slice(&(runtime.local.base as u64 + definition).to_le_bytes());
            let result = support::fixture_talents(matches!(spell, 139 | 774), || {
                support::fixture_target(matches!(spell, 17 | 324 | 467).then_some(target), || {
                    runtime
                        .adapter
                        .fixture_cast_actor(&io, game, caster, ability.as_ptr() as u64)
                })
            });
            runtime.log.info(&format!("official effect fixture cast: spell={spell} caster={caster} expected={target} result={result:?}"));
            EFFECT_STAGE.store(stage + 1, Ordering::Release);
        }
        2 if clock > previous + 1.0 => {
            let slots = runtime
                .adapter
                .support_buff_slots(&runtime.local, game, spell);
            if spell == 17 {
                let result = runtime.adapter.fixture_break_shield(&io, game, target);
                runtime
                    .log
                    .info(&format!("official shield break fixture: {result:?}"));
                if result == Ok(true) {
                    RAPTURE_STAGE.store(1, Ordering::Release);
                    if let Ok(mut t) = RAPTURE_CLOCK.lock() {
                        *t = clock;
                    }
                }
            }
            if matches!(spell, 324 | 467) {
                let hit = runtime.adapter.fixture_retaliation_hit(&io, game, target);
                runtime.log.info(&format!(
                    "official retaliation fixture: spell={spell} target={target} hit={hit:?}"
                ));
            }
            runtime.log.info(&format!("official effect fixture recipients: spell={spell} expected={target} slots={slots:?}"));
            EFFECT_STAGE.store(stage + 1, Ordering::Release);
        }
        _ => {}
    }
}

fn previous_guard_clock() -> f32 {
    EFFECT_CLOCK.lock().map(|c| *c).unwrap_or(f32::MAX)
}

static RAPTURE_STAGE: AtomicUsize = AtomicUsize::new(0);
static RAPTURE_CLOCK: std::sync::Mutex<f32> = std::sync::Mutex::new(0.0);
fn rapture_tick(r: &Runtime, game: u64, clock: f32) {
    if std::env::var("AZCOOP_TEST_OFFICIAL_RAPTURE").as_deref() != Ok("1") {
        return;
    }
    let stage = RAPTURE_STAGE.load(Ordering::Acquire);
    let began = RAPTURE_CLOCK.lock().map(|c| *c).unwrap_or(clock);
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    match stage {
        1 if clock > began + 1.0 => {
            let slots = r.adapter.support_buff_slots(&r.local, game, 17);
            r.log
                .info(&format!("official rapture first reapply: {slots:?}"));
            let hit = r.adapter.fixture_break_shield(&io, game, 0);
            r.log
                .info(&format!("official rapture second break: {hit:?}"));
            RAPTURE_STAGE.store(2, Ordering::Release);
        }
        2 if clock > began + 2.0 => {
            let slots = r.adapter.support_buff_slots(&r.local, game, 17);
            r.log.info(&format!("official rapture cooldown: {slots:?}"));
            RAPTURE_STAGE.store(3, Ordering::Release);
        }
        3 if clock > began + 16.0 => {
            let p = SUPPORT_PROTOTYPE.lock().ok().and_then(|p| p[1].clone());
            let Some(mut ability) = p else { return };
            let def = r.local.base as u64 + r.adapter.fixture_support_definition(17).unwrap();
            ability[..8].copy_from_slice(&def.to_le_bytes());
            let result = support::fixture_target(Some(0), || {
                r.adapter
                    .fixture_cast_actor(&io, game, 1, ability.as_ptr() as u64)
            });
            r.log.info(&format!("official rapture rearm: {result:?}"));
            if let Ok(mut t) = RAPTURE_CLOCK.lock() {
                *t = clock;
            }
            RAPTURE_STAGE.store(4, Ordering::Release);
        }
        4 if clock > began + 1.0 => {
            let hit = r.adapter.fixture_break_shield(&io, game, 0);
            r.log
                .info(&format!("official rapture after cooldown break: {hit:?}"));
            RAPTURE_STAGE.store(5, Ordering::Release);
        }
        5 if clock > began + 2.0 => {
            let slots = r.adapter.support_buff_slots(&r.local, game, 17);
            r.log.info(&format!(
                "official rapture after cooldown reapply: {slots:?}"
            ));
            RAPTURE_STAGE.store(6, Ordering::Release);
        }
        6 => {
            let cleared = r.adapter.fixture_break_shield(&io, game, 0);
            r.log
                .info(&format!("official shield coexistence cleared: {cleared:?}"));
            if let Some(mut ability) = SUPPORT_PROTOTYPE.lock().ok().and_then(|p| p[1].clone()) {
                ability[..8].copy_from_slice(
                    &(r.local.base as u64 + r.adapter.fixture_support_definition(17).unwrap())
                        .to_le_bytes(),
                );
                let result = support::fixture_target(Some(1), || {
                    r.adapter
                        .fixture_cast_actor(&io, game, 1, ability.as_ptr() as u64)
                });
                r.log
                    .info(&format!("official shield coexistence self: {result:?}"));
            }
            RAPTURE_STAGE.store(7, Ordering::Release);
            if let Ok(mut t) = RAPTURE_CLOCK.lock() {
                *t = clock;
            }
        }
        7 if clock > began + 0.2 => {
            if let Some(mut ability) = SUPPORT_PROTOTYPE.lock().ok().and_then(|p| p[1].clone()) {
                ability[..8].copy_from_slice(
                    &(r.local.base as u64 + r.adapter.fixture_support_definition(17).unwrap())
                        .to_le_bytes(),
                );
                let result = support::fixture_target(Some(2), || {
                    r.adapter
                        .fixture_cast_actor(&io, game, 1, ability.as_ptr() as u64)
                });
                r.log
                    .info(&format!("official shield coexistence ally: {result:?}"));
            }
            RAPTURE_STAGE.store(8, Ordering::Release);
        }
        8 if clock > began + 1.0 => {
            let slots = r.adapter.support_buff_slots(&r.local, game, 17);
            r.log.info(&format!(
                "official shield coexistence recipients: {slots:?}"
            ));
            RAPTURE_STAGE.store(9, Ordering::Release);
        }
        _ => {}
    }
}

static COMBAT_STAGE: AtomicUsize = AtomicUsize::new(0);
static COMBAT_CLOCK: std::sync::Mutex<f32> = std::sync::Mutex::new(0.0);
static COMBAT_ENEMY: AtomicU64 = AtomicU64::new(0);
static DEATH_ATTEMPTS: [AtomicUsize; 3] = [const { AtomicUsize::new(0) }; 3];
static DEATH_LAST: std::sync::Mutex<[f32; 3]> = std::sync::Mutex::new([-100.0; 3]);
fn request_death(r: &Runtime, game: u64, slot: u32, guid: u64, clock: f32) {
    let i = slot as usize;
    if DEATH_ATTEMPTS[i].load(Ordering::Acquire) >= if fatigue_enabled() || xp_death_enabled() {20} else {8} {
        return;
    }
    let ready = if let Ok(mut last) = DEATH_LAST.lock() {
        if clock - last[i] < if fatigue_enabled() || xp_death_enabled() {1.0} else {0.4} {
            false
        } else {
            last[i] = clock;
            true
        }
    } else {
        false
    };
    if !ready {
        return;
    }
    DEATH_ATTEMPTS[i].fetch_add(1, Ordering::AcqRel);
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    let result = r.adapter.hurt_actor(&io, game, slot, guid, 1e7, 0, false);
    let label = if slot == 0 {
        "official XP host death fixture"
    } else {
        "official taunt death fixture"
    };
    r.log.info(&format!("{label}: {result:?}"));
}
fn combat_tick(r: &Runtime, server: u64) {
    if std::env::var("AZCOOP_TEST_OFFICIAL_COMBAT").as_deref() != Ok("1")
        || RAPTURE_STAGE.load(Ordering::Acquire) < 9
    {
        return;
    }
    let Ok(Some(party)) = r.adapter.party(&r.local, server) else {
        return;
    };
    if party.actors.len() != 3 {
        return;
    }
    let Ok(game) = r.adapter.game_address(server) else {
        return;
    };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    let stage = COMBAT_STAGE.load(Ordering::Acquire);
    let began = COMBAT_CLOCK.lock().map(|t| *t).unwrap_or(party.clock);
    let enemy = COMBAT_ENEMY.load(Ordering::Acquire);
    let mark =
        |strength| asx_gameplay::Mark::new((strength > 0.0).then_some(enemy), strength).unwrap();
    match stage {
        0 if party
            .actors
            .iter()
            .all(|a| a.phase == asx_gameplay::Phase::Playing) =>
        {
            let enemy = match r.adapter.fixture_combat_enemy(&io, game, 2) {
                Ok(Some(enemy)) => enemy,
                Ok(None) => return,
                Err(e) => {
                    static REPORTED: std::sync::atomic::AtomicBool =
                        std::sync::atomic::AtomicBool::new(false);
                    if !REPORTED.swap(true, Ordering::AcqRel) {
                        r.log.warn(&format!("Official combat fixture: {e}"));
                    }
                    return;
                }
            };
            COMBAT_ENEMY.store(enemy, Ordering::Release);
            r.log
                .info(&format!("official combat fixture enemy: {enemy}"));
            if let Ok(mut c) = COMBAT_CLOCK.lock() {
                *c = party.clock;
            }
            COMBAT_STAGE.store(1, Ordering::Release);
        }
        1 if party.clock > began + 0.2 => {
            let before = r.adapter.combat_hp(&r.local, game, enemy).ok().flatten();
            let result = r.adapter.attack_actor(&io, game, 0, enemy, 133, 10.0);
            let after = r.adapter.combat_hp(&r.local, game, enemy).ok().flatten();
            r.log.info(&format!(
                "official mark baseline: before={before:?} after={after:?} result={result:?}"
            ));
            combat::fixture_mark(0, mark(0.2));
            combat::fixture_mark(1, mark(0.35));
            COMBAT_STAGE.store(2, Ordering::Release);
        }
        2 if party.clock > began + 1.2 => {
            let read = r
                .adapter
                .combat_enemies(&r.local, game)
                .ok()
                .and_then(|v| v.into_iter().find(|e| e.id == enemy));
            r.log.info(&format!("official mark strongest: {read:?}"));
            let _ = r.adapter.attack_actor(&io, game, 0, enemy, 133, 10.0);
            combat::fixture_mark(1, asx_gameplay::Mark::default());
            let _ = r.adapter.write_combat_mark(&io, game, enemy, 0.0);
            type Aura = unsafe extern "system" fn(u64, u64, u32, bool);
            let aura: Aura = unsafe {
                std::mem::transmute(r.local.base + r.adapter.combat_entries()[4].rva as usize)
            };
            unsafe { aura(game, enemy, 1130, false) };
            COMBAT_STAGE.store(3, Ordering::Release);
        }
        3 if party.clock > began + 2.2 => {
            let read = r
                .adapter
                .combat_enemies(&r.local, game)
                .ok()
                .and_then(|v| v.into_iter().find(|e| e.id == enemy));
            r.log.info(&format!("official mark remaining: {read:?}"));
            combat::fixture_mark(0, asx_gameplay::Mark::default());
            COMBAT_STAGE.store(4, Ordering::Release);
        }
        4 if party.clock > began + 3.2 => {
            let read = r
                .adapter
                .combat_enemies(&r.local, game)
                .ok()
                .and_then(|v| v.into_iter().find(|e| e.id == enemy));
            r.log.info(&format!("official mark cleared: {read:?}"));
            let _ = r.adapter.attack_actor(&io, game, 0, enemy, 23922, 1.0);
            r.log.info(&format!(
                "official taunt without talent: {:?}",
                combat::fixture_owner(enemy)
            ));
            combat::fixture_talents(
                0,
                asx_gameplay::Talents {
                    fortified: true,
                    ..Default::default()
                },
            );
            let _ = r.adapter.attack_actor(&io, game, 0, enemy, 23922, 1.0);
            COMBAT_STAGE.store(5, Ordering::Release);
        }
        5 if party.clock > began + 4.2 => {
            r.log.info(&format!(
                "official taunt warrior owner: {:?}",
                combat::fixture_owner(enemy)
            ));
            combat::fixture_talents(
                1,
                asx_gameplay::Talents {
                    ardent_defender: true,
                    ..Default::default()
                },
            );
            let _ = r.adapter.attack_actor(&io, game, 1, enemy, 853, 1.0);
            COMBAT_STAGE.store(6, Ordering::Release);
        }
        6 if party.clock > began + 5.2 => {
            r.log.info(&format!(
                "official taunt paladin owner: {:?}",
                combat::fixture_owner(enemy)
            ));
            combat::fixture_talents(
                2,
                asx_gameplay::Talents {
                    dire_bear_form: true,
                    ..Default::default()
                },
            );
            let _ = r.adapter.attack_actor(&io, game, 2, enemy, 5221, 1.0);
            COMBAT_STAGE.store(7, Ordering::Release);
        }
        7 if party.clock > began + 6.2 => {
            r.log.info(&format!(
                "official taunt druid owner: {:?}",
                combat::fixture_owner(enemy)
            ));
            let actor = &party.actors[0];
            let result = r
                .adapter
                .hurt_actor(&io, game, 0, actor.guid, 10.0, enemy, true);
            r.log
                .info(&format!("official taunt melee fixture: {result:?}"));
            COMBAT_STAGE.store(8, Ordering::Release);
        }
        8 if party.clock > began + 7.2 => {
            let actor = &party.actors[0];
            let result = r
                .adapter
                .hurt_actor(&io, game, 0, actor.guid, 10.0, enemy, false);
            r.log.info(&format!(
                "official taunt special fixture: recipient=0 result={result:?}"
            ));
            COMBAT_STAGE.store(9, Ordering::Release);
        }
        9 if party.clock > began + 8.2 => {
            let actor = &party.actors[2];
            if actor.phase == asx_gameplay::Phase::Downed {
                COMBAT_STAGE.store(10, Ordering::Release);
            } else {
                request_death(r, game, 2, actor.guid, party.clock);
            }
        }
        10 if party.clock > began + 8.5 => {
            r.log.info(&format!(
                "official taunt after death: owner={:?} phase={:?}",
                combat::fixture_owner(enemy),
                party.actors[2].phase
            ));
            COMBAT_STAGE.store(11, Ordering::Release);
        }
        11 if party.clock > began + 18.5
            && party
                .actors
                .iter()
                .all(|a| a.phase == asx_gameplay::Phase::Playing) =>
        {
            let progression = party
                .actors
                .iter()
                .map(|a| (a.level, a.xp))
                .collect::<Vec<_>>();
            r.log
                .info(&format!("official XP guest recovery: {progression:?}"));
            request_death(r, game, 0, party.actors[0].guid, party.clock);
            COMBAT_STAGE.store(12, Ordering::Release);
        }
        12 if party.actors[0].phase == asx_gameplay::Phase::Downed => {
            r.log.info("official XP host downed observed");
            COMBAT_STAGE.store(13, Ordering::Release);
        }
        12 => {
            request_death(r, game, 0, party.actors[0].guid, party.clock);
        }
        13 if party
            .actors
            .iter()
            .all(|a| a.phase == asx_gameplay::Phase::Playing) =>
        {
            let progression = party
                .actors
                .iter()
                .map(|a| (a.level, a.xp))
                .collect::<Vec<_>>();
            r.log
                .info(&format!("official XP host recovery: {progression:?}"));
            COMBAT_STAGE.store(14, Ordering::Release);
        }
        _ => {}
    }
}

fn scaling_tick(r: &Runtime, server: u64) {
    if std::env::var("AZCOOP_TEST_OFFICIAL_SCALING").as_deref() != Ok("1") {
        return;
    }
    let Ok(Some(p)) = r.adapter.party(&r.local, server) else {
        return;
    };
    if p.actors.len() != 3
        || p.clock < 2.
        || !p
            .actors
            .iter()
            .all(|a| a.phase == asx_gameplay::Phase::Playing)
    {
        return;
    }
    static DONE: AtomicBool = AtomicBool::new(false);
    if DONE.swap(true, Ordering::AcqRel) {
        return;
    }
    let Ok(game) = r.adapter.game_address(server) else {
        return;
    };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    r.log.info(&format!(
        "official difficulty probe: {:?}",
        r.adapter.fixture_scaling_probe(&io, game)
    ));
}

// Small UI-only fixture: use a captured native ability descriptor and the real
// support dispatcher. No economy/death/boss battery is enabled for this scenario.
fn party_ui_enabled() -> bool {
    std::env::var("AZCOOP_TEST_OFFICIAL_PARTY_BUFF").as_deref() == Ok("1")
}
static PARTY_ITEMS_DONE:AtomicBool=AtomicBool::new(false);
static PARTY_BUFF_CLOCK: std::sync::Mutex<f32> = std::sync::Mutex::new(-10.0);
fn party_ui_tick(runtime: &Runtime, server: u64) {
    if !party_ui_enabled() { return; }
    let Ok(Some(party)) = runtime.adapter.party(&runtime.local, server) else { return; };
    if party.actors.len()!=3 || party.clock < 2.0 || party.actors.iter().any(|a| a.phase != asx_gameplay::Phase::Playing) { return; }
    let Some(mut prototype) = SUPPORT_PROTOTYPE.lock().ok().and_then(|p| p[0].clone()) else { return; };
    {
        let Ok(mut clock) = PARTY_BUFF_CLOCK.lock() else { return; };
        if party.clock - *clock < 8.0 { return; }
        *clock = party.clock;
    }
    let Ok(game) = runtime.adapter.game_address(server) else { return; };
    if !PARTY_ITEMS_DONE.swap(true,Ordering::AcqRel) {
        let io=io::Access{local:&runtime.local,adapter:runtime.adapter};
        for actor in &party.actors {
            let result=runtime.adapter.deliver_elite_item(&io,game,actor.slot,actor.guid);
            runtime.log.info(&format!("official party item fixture: slot={} result={result:?}",actor.slot));
        }
    }
    let Ok(definition) = runtime.adapter.fixture_support_definition(6673) else { return; };
    prototype[..8].copy_from_slice(&(runtime.local.base as u64 + definition).to_le_bytes());
    let success = unsafe { support::cast(game, prototype.as_ptr() as u64) };
    runtime.log.info(&format!("official party UI fixture: shout={success}"));
}

// One bounded fatigue scenario. Native lethal damage and native coop revival;
// no time acceleration, production stat writes or unrelated fixture campaigns.
fn fatigue_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("AZCOOP_TEST_OFFICIAL_FATIGUE").as_deref()==Ok("1"))
}
static FATIGUE_STAGE: AtomicUsize = AtomicUsize::new(0);
static FATIGUE_CLOCK: std::sync::Mutex<f32> = std::sync::Mutex::new(0.0);
static FATIGUE_SAMPLE: std::sync::Mutex<i32> = std::sync::Mutex::new(-1);
fn fatigue_tick(r: &Runtime, server: u64) {
    if !fatigue_enabled() { return; }
    let Ok(Some(party))=r.adapter.party(&r.local,server) else {return};
    if party.actors.len()!=2 {return;}
    let Ok(game)=r.adapter.game_address(server) else {return};
    let guest=&party.actors[1];
    let view=fatigue::view(guest.slot,guest.guid);
    if let Ok(mut sample)=FATIGUE_SAMPLE.lock() {
        let second=party.clock.floor() as i32;
        if second != *sample {
            *sample=second;
            r.log.info(&format!("fatigue fixture state: clock={} phase={:?} stacks={} remaining={} multiplier={}",party.clock,guest.phase,view.stacks,view.remaining,view.multiplier()));
        }
    }
    let io=io::Access{local:&r.local,adapter:r.adapter};
    let stage=FATIGUE_STAGE.load(Ordering::Acquire);
    let began=FATIGUE_CLOCK.lock().map(|v|*v).unwrap_or(party.clock);
    match stage {
        0 if party.clock>=10.0 && guest.phase==asx_gameplay::Phase::Playing => {
            request_death(r,game,1,guest.guid,party.clock);
            FATIGUE_STAGE.store(1,Ordering::Release);
        }
        // Redemption can revive between two safe fixture observations. A new
        // fatigue stack establishes that the native down/revive cycle occurred.
        1 if guest.phase==asx_gameplay::Phase::Playing && view.stacks>=1 => {FATIGUE_STAGE.store(2,Ordering::Release);}
        5 if guest.phase==asx_gameplay::Phase::Playing && view.stacks>=2 => {FATIGUE_STAGE.store(6,Ordering::Release);}
        1 | 5 if guest.phase==asx_gameplay::Phase::Downed => { FATIGUE_STAGE.store(stage+1,Ordering::Release); }
        1 | 5 => request_death(r,game,1,guest.guid,party.clock),
        2 | 6 if guest.phase==asx_gameplay::Phase::Playing && view.stacks>0 => {
            FATIGUE_STAGE.store(stage+1,Ordering::Release);
            if let Ok(mut c)=FATIGUE_CLOCK.lock(){*c=party.clock;}
            r.log.info(&format!("fatigue fixture native revival: stacks={}",view.stacks));
        }
        3 if party.clock>began+1.0 => {
            // Observe real hook arguments, bounded to this feature's test.
            fatigue::fixture_arm();
            if let Ok(enemies)=r.adapter.support_enemies(&r.local,game) {
                if let Some(enemy)=enemies.iter().find(|e|e.hp>0.0) {
                    for slot in [0,1] {
                        let hit=r.adapter.attack_actor(&io,game,slot,enemy.guid,845,10.0);
                        r.log.info(&format!("fatigue fixture attack: slot={slot} result={hit:?}"));
                    }
                }
            }
            FATIGUE_STAGE.store(20,Ordering::Release);
        }
        20 | 22 | 24 | 26 => {
            let (caster,target,_)= [(1,0,331),(0,1,635),(1,0,139),(1,1,139)][(stage-20)/2];
            if r.adapter.fixture_support_wounds(&io,game,if caster==target{1-caster}else{caster},target).is_ok() {
                if let Ok(mut c)=FATIGUE_CLOCK.lock(){*c=party.clock;}
                FATIGUE_STAGE.store(stage+1,Ordering::Release);
            }
        }
        21 | 23 | 25 | 27 if party.clock>began+0.1 => {
            let (caster,target,spell)=[(1,0,331),(0,1,635),(1,0,139),(1,1,139)][(stage-21)/2];
            let prototype=SUPPORT_PROTOTYPE.lock().ok().and_then(|p|p[caster as usize].clone());
            if let Some(mut ability)=prototype {
                let def=r.adapter.fixture_support_definition(spell).unwrap();
                ability[..8].copy_from_slice(&(r.local.base as u64+def).to_le_bytes());
                let result=support::fixture_target(Some(target),||r.adapter.fixture_cast_resolved(&io,game,caster,ability.as_ptr()as u64));
                r.log.info(&format!("fatigue fixture cast: caster={caster} target={target} spell={spell} result={result:?}"));
                if let Ok(mut c)=FATIGUE_CLOCK.lock(){*c=party.clock;}
                FATIGUE_STAGE.store(if stage==27 {4}else{stage+1},Ordering::Release);
            }
        }
        4 if party.clock>began+5.0 => {
            request_death(r,game,1,guest.guid,party.clock);
            FATIGUE_STAGE.store(5,Ordering::Release);
        }
        7 if view.stacks==1 => {r.log.info("fatigue fixture recovery: one stack removed after 60 alive seconds");FATIGUE_STAGE.store(8,Ordering::Release);}
        _=>{}
    }
}

// Five bounded native encounters: four distinct definitions, then a repeat.
// No time jump, broad entity hook, CPU restriction or production override.
fn blessing_tick(r:&Runtime,server:u64){
 if std::env::var("AZCOOP_TEST_BLESSINGS").as_deref()!=Ok("1"){return;}
 static STAGE:AtomicUsize=AtomicUsize::new(0);
 let stage=STAGE.load(Ordering::Acquire);if stage>=if std::env::var("AZCOOP_TEST_EXTENSIONS").as_deref()==Ok("1"){6}else if std::env::var("AZCOOP_TEST_DUPLICATE_LOOT").as_deref()==Ok("1"){10}else{12}{return;}
 let Some(p)=r.extension_party()else{return;};
 if !(1..=2).contains(&p.actors.len()){return;}
 if stage==10 {
  let Ok(game)=r.adapter.game_address(server)else{return;};
  if r.adapter.blessing_fixture_victory(&r.local,game)==Ok(true){
   let entry=r.adapter.blessing_endless_entry();
   if r.local.read(r.local.base+entry.rva as usize,entry.bytes.len()).ok().as_deref()!=Some(entry.bytes){return;}
   let call:unsafe extern "system" fn(u64)=unsafe{std::mem::transmute(r.local.base+entry.rva as usize)};
   unsafe{call(game)};
   r.log.info("blessing fixture native Keep fighting action");
   return;
  }
 }
 if p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return;}
 let wave=stage/2;let due=8.0+wave as f32*7.0+if stage%2==1{1.5}else{0.0};
 if p.clock<due{return;}
 let Ok(game)=r.adapter.game_address(server)else{return;};
 let io=io::Access{local:&r.local,adapter:r.adapter};
 if stage%2==0 {
  let entry=r.adapter.blessing_spawn_entry();
  if r.local.read(r.local.base+entry.rva as usize,entry.bytes.len()).ok().as_deref()!=Some(entry.bytes){return;}
  let encounter_index=if wave==5{if std::env::var("AZCOOP_TEST_DUPLICATE_LOOT").as_deref()==Ok("1"){1}else{0}}else{wave as u32};
  let def=match r.adapter.blessing_fixture_definition(&r.local,game,r.local.base as u64,encounter_index){Ok(d)=>d,Err(e)=>{static REJECTED:AtomicBool=AtomicBool::new(false);if !REJECTED.swap(true,Ordering::Relaxed){r.log.warn(&format!("Native encounter fixture definition rejected: {e}"));}return}};
  STAGE.store(stage+1,Ordering::Release);
  unsafe{r.adapter.fixture_spawn_encounter(r.local.base,game,def)};
  r.log.info(&format!("blessing fixture encounter spawned: wave={wave} index={}",if wave==5{0}else{wave}));
 }else{
  use asx_game::official::SupportCalls;
  let Ok(targets)=r.adapter.blessing_fixture_targets(&r.local,game)else{return;};
  if targets.is_empty(){return;}
  STAGE.store(stage+1,Ordering::Release);
  r.log.info(&format!("blessing fixture encounter targets: wave={wave} count={}",targets.len()));
  for guid in targets {if let Err(e)=io.support_damage(game,guid,1e8){r.log.warn(&format!("Blessing fixture damage rejected: {e}"));}}
 }
}

// One two-player regression scenario: native cache purchase and one guardian kill.
static REGRESSION_STAGE:AtomicUsize=AtomicUsize::new(0);
fn regression_tick(runtime:&Runtime,server:u64){
 if std::env::var("AZCOOP_TEST_OFFICIAL_REGRESSIONS").as_deref()!=Ok("1"){return}
 let Ok(Some(party))=runtime.adapter.party(&runtime.local,server)else{return};
 if party.actors.len()!=2 || !party.actors.iter().all(|a|a.phase==asx_gameplay::Phase::Playing){return}
 let Ok(game)=runtime.adapter.game_address(server)else{return};
 match REGRESSION_STAGE.load(Ordering::Acquire){
  0 if party.clock>4.=>{
   REGRESSION_STAGE.store(1,Ordering::Release);
   type Gold=unsafe extern "system" fn(u64,u32)->usize;type Chest=unsafe extern "system" fn(u64)->bool;
   let gold:Gold=unsafe{std::mem::transmute(runtime.local.base+runtime.adapter.gold_entry().rva as usize)};
   let chest:Chest=unsafe{std::mem::transmute(runtime.local.base+runtime.adapter.chest_entry().rva as usize)};
   unsafe{gold(game,101);let ok=chest(game);runtime.log.info(&format!("regression native cache purchase: {ok}"));}
  },
  1 if party.clock>8.=>{
   let io=io::Access{local:&runtime.local,adapter:runtime.adapter};
   let result=runtime.adapter.fixture_guardian(&io,game);
   if result.as_ref().is_ok_and(|guid|guid.is_some()){REGRESSION_STAGE.store(2,Ordering::Release);runtime.log.info(&format!("regression one native guardian: {result:?}"));}
  },_=>{}
 }
}

static ARMORY_GOLD_DONE:AtomicBool=AtomicBool::new(false);
fn armory_gold_tick(runtime:&Runtime,server:u64){
 if std::env::var("AZCOOP_TEST_ARMORY_GOLD").as_deref()!=Ok("1")||ARMORY_GOLD_DONE.load(Ordering::Acquire){return}
 let Ok(Some(party))=runtime.adapter.party(&runtime.local,server)else{return};
 if party.clock<4.||party.actors.len()!=2||!party.actors.iter().all(|a|a.phase==asx_gameplay::Phase::Playing){return}
 ARMORY_GOLD_DONE.store(true,Ordering::Release);
 let Ok(game)=runtime.adapter.game_address(server)else{return};let io=io::Access{local:&runtime.local,adapter:runtime.adapter};
 let result=(||->Result<(),String>{
  for (actor,bonus)in party.actors.iter().zip([0.1,0.3]){runtime.adapter.fixture_armory_gold(&io,game,actor.slot,actor.guid,bonus)?;}
  runtime.adapter.refresh_blessings(&io,game,&party.actors.iter().map(|a|(a.slot,a.guid)).collect::<Vec<_>>())?;
  wallet::invalidate_fixture_armory(game);
  wallet::boundary(runtime,game,Some(&party))?;
  let (_,collector,_)=runtime.adapter.current_gold(&runtime.local,game)?;
  runtime.log.info(&format!("armory gold fixture native collector: {collector}"));
  type Gold=unsafe extern "system" fn(u64,u32)->usize;
  let gold:Gold=unsafe{std::mem::transmute(runtime.local.base+runtime.adapter.gold_entry().rva as usize)};
  unsafe{gold(game,100)};Ok(())
 })();runtime.log.info(&format!("armory gold fixture complete: {result:?}"));
}

// Two bounded asymmetric choice windows; no OS input or position writes.
pub(super) fn pause_enabled()->bool{std::env::var("AZCOOP_TEST_OFFICIAL_PAUSE").as_deref()==Ok("1")}
struct PauseTrial {pending:Option<(Vec<u8>,Instant)>,held:Vec<(&'static str,Instant,u64)>,sample:Option<Instant>,cache:bool}
static PAUSE_TRIAL:std::sync::Mutex<PauseTrial>=std::sync::Mutex::new(PauseTrial{pending:None,held:Vec::new(),sample:None,cache:false});
fn pause_event(r:&Runtime,c:&crate::stub::Context,v:&serde_json::Value){
 if matches!(v["type"].as_str(),Some("menu"|"resume"|"paused"|"pause")){r.log.info(&format!("pause native UI event: {v}"));}
 let kind=match (v["type"].as_str(),v["why"].as_str()){(Some("choices"),Some("levelup"))=>"level",(Some("choices"),Some("cache"))=>"cache",_=>return};
 let host=std::env::var("AZCOOP_TEST_OFFICIAL_PAUSE_HOST").as_deref()==Ok("1");
 let seconds=if (kind=="level")==host{3}else{20};
 {let Ok(s)=PAUSE_TRIAL.lock()else{return};if s.held.iter().any(|(k,_,_)|*k==kind){return}}
 let io=io::Access{local:&r.local,adapter:r.adapter};
 if r.adapter.set_ui_autopick(&io,c.rcx,false).is_err(){return}
 let now=Instant::now();if let Ok(mut s)=PAUSE_TRIAL.lock(){s.held.push((kind,now,seconds));s.pending=Some((serde_json::to_vec(v).unwrap(),now+Duration::from_secs(seconds)));}
 r.log.info(&format!("pause fixture held: kind={kind} delay={seconds}"));
}
fn pause_tick(r:&Runtime,server:u64){
 let release={let Ok(mut s)=PAUSE_TRIAL.lock()else{return};if s.pending.as_ref().is_some_and(|(_,due)|Instant::now()>=*due){s.pending.take()}else{None}};
 if let Some((bytes,_))=release{
  let io=io::Access{local:&r.local,adapter:r.adapter};
  if let Ok(receiver)=r.adapter.ui_receiver(&r.local,server,r.local.base as u64){
   if r.adapter.set_ui_autopick(&io,receiver,true).is_ok(){unsafe{pause::replay_event(receiver,&bytes)};r.log.info("pause fixture choice released");}
  }
 }
 let (sample,windows)={let Ok(mut s)=PAUSE_TRIAL.lock()else{return};let sample=s.sample.is_none_or(|at|at.elapsed()>=Duration::from_secs(2));if sample{s.sample=Some(Instant::now());}(sample,s.held.iter().map(|(k,at,d)|(*k,at.elapsed().as_secs_f32(),*d)).collect::<Vec<_>>())};
 if !sample{return}
 // Exercise the real production command filter, without invoking an unreviewed
 // native packet decoder. Native clock/positions are observed separately.
 let mut movement:crate::stub::Context=unsafe{std::mem::zeroed()};movement.rcx=server;movement.r9=0xb5;unsafe{session::command(&mut movement)};
 let mut choice:crate::stub::Context=unsafe{std::mem::zeroed()};choice.rcx=server;choice.r9=0x100;unsafe{session::command(&mut choice)};
 r.log.info(&format!("pause fixture gate: paused={} blocked={} nonmovement={} windows={windows:?}",pause::active(),movement.skip,choice.skip));
 r.log.info(&format!("pause fixture local: paused={} position={:?}",pause::active(),r.adapter.fixture_pause_position(&r.local,server)));

 let Ok(Some(party))=r.adapter.party(&r.local,server)else{return};
 r.log.info(&format!("pause fixture snapshot: clock={} actors={:?}",party.clock,party.actors.iter().map(|a|(a.slot,a.phase,a.position)).collect::<Vec<_>>()));
 if party.actors.len()!=2||party.clock<8.||party.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 let open={let Ok(mut s)=PAUSE_TRIAL.lock()else{return};if s.cache{false}else{s.cache=true;true}};
 if open{if let Ok(game)=r.adapter.game_address(server){type Gold=unsafe extern "system" fn(u64,u32)->usize;type Chest=unsafe extern "system" fn(u64)->bool;let gold:Gold=unsafe{std::mem::transmute(r.local.base+r.adapter.gold_entry().rva as usize)};let chest:Chest=unsafe{std::mem::transmute(r.local.base+r.adapter.chest_entry().rva as usize)};let result=unsafe{gold(game,101);chest(game)};r.log.info(&format!("pause fixture native cache: {result}"));}}
}

fn xp_death_enabled()->bool{std::env::var("AZCOOP_TEST_XP_DEATH").as_deref()==Ok("1")}
struct XpDeath {stage:u32,began:f32,sampled:Option<Instant>}
static XP_DEATH:std::sync::Mutex<XpDeath>=std::sync::Mutex::new(XpDeath{stage:0,began:0.,sampled:None});
fn xp_death_tick(r:&Runtime,server:u64){
 {let Ok(mut s)=XP_DEATH.lock()else{return};if s.sampled.is_some_and(|t|t.elapsed()<Duration::from_secs(2)){return}s.sampled=Some(Instant::now());}
 let Ok(Some(party))=r.adapter.party(&r.local,server)else{return};if party.actors.len()!=2{return}
 let Ok(game)=r.adapter.game_address(server)else{return};
 let (stage,began)={let Ok(s)=XP_DEATH.lock()else{return};(s.stage,s.began)};
 r.log.info(&format!("XP death state: stage={stage} clock={} actors={:?}",party.clock,party.actors.iter().map(|a|(a.slot,a.guid,a.phase,a.level,a.xp)).collect::<Vec<_>>()));
 let target=if stage<3{0}else{1};let victim=&party.actors[target];
 let advance=|next|{if let Ok(mut s)=XP_DEATH.lock(){s.stage=next;s.began=party.clock;}};
 match stage {
  0 if party.clock>=10. && party.actors.iter().all(|a|a.phase==asx_gameplay::Phase::Playing)=>{request_death(r,game,0,victim.guid,party.clock);advance(1);},
  1|4 if victim.phase==asx_gameplay::Phase::Downed=>{r.log.info(&format!("XP death confirmed: slot={target} guid={}",victim.guid));advance(stage+1);},
  1|4 if victim.phase==asx_gameplay::Phase::Playing=>request_death(r,game,target as u32,victim.guid,party.clock),
  2|5 if victim.phase==asx_gameplay::Phase::Downed=>{
   let alive=&party.actors[1-target];if alive.phase!=asx_gameplay::Phase::Playing{return}
   let io=io::Access{local:&r.local,adapter:r.adapter};
   if stage==2 {
       static GUARDIAN:AtomicBool=AtomicBool::new(false);
       if !GUARDIAN.load(Ordering::Acquire){
           let target=r.adapter.fixture_guardian_target(&io,game);
           r.log.info(&format!("XP death guardian target: {target:?}"));
           if let Ok(Some((guid,hp)))=target {
               let result=r.adapter.attack_actor(&io,game,alive.slot,guid,75,hp*100.+1e6);
               r.log.info(&format!("XP death guardian hit during host death: guid={guid} result={result:?}"));
               if result.is_ok(){GUARDIAN.store(true,Ordering::Release);}
           }
       }
   }
   if let Ok(enemies)=r.adapter.support_enemies(&r.local,game){if let Some(e)=enemies.iter().find(|e|e.hp>0. && e.position.iter().zip(alive.position).map(|(x,y)|(*x-f64::from(y)).powi(2)).sum::<f64>()<49.){let result=r.adapter.attack_actor(&io,game,alive.slot,e.guid,845,1e6);r.log.info(&format!("XP death native kill: owner={} result={result:?}",alive.slot));}}
  },
  2|5 if victim.phase==asx_gameplay::Phase::Playing=>{r.log.info(&format!("XP death revived: slot={target} guid={}",victim.guid));advance(stage+1);},
  3 if party.clock>began+5. && party.actors.iter().all(|a|a.phase==asx_gameplay::Phase::Playing)=>{request_death(r,game,1,victim.guid,party.clock);advance(4);},
  _=>{}
 }
}

pub fn compat121()->bool{std::env::var("AZCOOP_TEST_COMPAT121").as_deref()==Ok("1")}
pub fn compat121_pick(r:&Runtime,game:u64)->Option<u64>{
 let(slot,guid)=r.adapter.run_source(&r.local,game).ok().flatten()?;
 static PICKS:std::sync::Mutex<std::collections::BTreeMap<(u32,u64),u32>>=std::sync::Mutex::new(std::collections::BTreeMap::new());
 let n={let mut p=PICKS.lock().ok()?;let n=p.entry((slot,guid)).or_default();*n+=1;*n};
 let gem=r.adapter.amulet_fixture_pick(&r.local,r.local.base as u64,game).ok().flatten();
 let ring=r.adapter.ring_fixture_pick(&r.local,r.local.base as u64,game).ok().flatten();
 if !gems::fixture_has_gem(slot,guid)||n%3==0{gem.or(ring)}else{ring.or(gem)}
}

fn completion121_enabled()->bool{std::env::var("AZCOOP_TEST_SUPPORT121").as_deref()==Ok("1")}
// Diagnostic only: owned actor parameter copies, actual selected native casts,
// two participants and no changes to production coefficients or cooldowns.
fn completion121_tick(r:&Runtime,server:u64){
 static STATE:std::sync::Mutex<(usize,u8,f32,f32)>=std::sync::Mutex::new((0,0,0.,0.));
 const CASES:[(u32,u32,u32);12]=[(635,0,1),(331,1,0),(139,0,1),(774,1,0),(8936,1,0),(17,0,1),(467,1,0),(324,0,1),(15237,0,1),(25912,1,0),(11426,0,1),(774,0,1)];
 let shield_only=std::env::var("ASX_TEST_SUPPORT_SHIELD_ONLY").as_deref()==Ok("1");
 static INITIALIZED:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
 if !INITIALIZED.swap(true,Ordering::AcqRel)&&shield_only{*STATE.lock().unwrap()=(5,0,0.,0.);}
 let Ok(Some(p))=r.adapter.party(&r.local,server)else{return};if p.actors.len()!=2||p.clock<12.||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 let (index,stage,began,before)=*STATE.lock().unwrap();if index>=if shield_only{6}else{CASES.len()}{
  if std::env::var("AZCOOP_TEST_SUPPORT121_FULL").as_deref()==Ok("1"){
   fatigue_tick(r,server);
   if FATIGUE_STAGE.load(Ordering::Acquire)>=8 && FATIGUE_STAGE.load(Ordering::Acquire)<20{
    if std::env::var("ASX_TEST_COVERAGE").as_deref()==Ok("1") {
     static GUARDIAN_DONE:AtomicBool=AtomicBool::new(false);
     if !GUARDIAN_DONE.load(Ordering::Acquire){
      if let Ok(game)=r.adapter.game_address(server){
       let io=io::Access{local:&r.local,adapter:r.adapter};
       let result=r.adapter.fixture_guardian(&io,game);
       if result.as_ref().is_ok_and(|guid|guid.is_some()){
        GUARDIAN_DONE.store(true,Ordering::Release);
        r.log.info(&format!("coverage native guardian killed: {result:?}"));
       }
      }
      return;
     }
    }
    combat::immunity_fixture_tick(r,server);
   }
  }
  return
 }
 let(spell,caster,target)=CASES[index];let Ok(game)=r.adapter.game_address(server)else{return};let io=io::Access{local:&r.local,adapter:r.adapter};
 if stage==0 {
  if p.clock<began+1. {return}
  let prototype=SUPPORT_PROTOTYPE.lock().ok().and_then(|p|p[caster as usize].clone());let Some(mut ability)=prototype else{return};
  let result=(||->Result<bool,String>{r.adapter.fixture_support_wounds(&io,game,caster,target)?;let before=r.adapter.party(&r.local,server)?.ok_or("Missing actors")?.actors[target as usize].hp;
   let definition=r.adapter.fixture_support_definition(spell)?;ability[..8].copy_from_slice(&(r.local.base as u64+definition).to_le_bytes());
   let cast=support::fixture_talents(matches!(spell,139|774),||support::fixture_target((spell!=11426).then_some(target),||r.adapter.fixture_cast_resolved(&io,game,caster,ability.as_ptr()as u64)));
   r.log.info(&format!("support121 cast: index={index} spell={spell} caster={caster} target={target} before={before} result={cast:?}"));
   *STATE.lock().unwrap()=(index,u8::from(cast==Ok(true)),p.clock,before);cast
  })();if let Err(e)=result{r.log.warn(&format!("Support121 fixture rejected: {e}"));*STATE.lock().unwrap()=(CASES.len(),0,0.,0.);}
 }else if p.clock>began+2.5{
  // Native Shadowmeld can legitimately reject the diagnostic hit. Observe the
  // real immunity state and retain this case until it expires; never remove it.
  if spell==17 && r.adapter.combat_immune(&r.local,game,target,p.actors[target as usize].guid).unwrap_or(true){return;}
  let slots=r.adapter.support_buff_slots(&r.local,game,spell);let after=p.actors[target as usize].hp;
  r.log.info(&format!("support121 observed: index={index} spell={spell} caster={caster} target={target} before={before} after={after} slots={slots:?}"));
  if matches!(spell,467|324){let hit=r.adapter.fixture_retaliation_hit(&io,game,target);r.log.info(&format!("support121 retaliation: spell={spell} hit={hit:?}"));}
  if spell==17{let broken=r.adapter.fixture_break_shield(&io,game,target);r.log.info(&format!("support121 shield break: {broken:?}"));}
  *STATE.lock().unwrap()=(index+1,0,p.clock,0.);
 }
}

fn marks121_tick(r:&Runtime,server:u64){
 if std::env::var("AZCOOP_TEST_MARKS121").as_deref()!=Ok("1"){return}
 static STATE:std::sync::Mutex<(u8,f32,u64)>=std::sync::Mutex::new((0,0.,0));
 let Ok(Some(p))=r.adapter.party(&r.local,server)else{return};if p.actors.len()!=2||p.clock<4.||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 let(stage,began,mut enemy)=*STATE.lock().unwrap();if stage>=7||p.clock<began+1.{return}let Ok(game)=r.adapter.game_address(server)else{return};let io=io::Access{local:&r.local,adapter:r.adapter};
 let result=(||->Result<(),String>{
  if stage==0{let Some(id)=r.adapter.fixture_combat_enemy(&io,game,5)?else{return Ok(())};enemy=id;combat::fixture_mark(0,asx_gameplay::Mark::new(Some(enemy),0.2).unwrap());combat::fixture_mark(1,asx_gameplay::Mark::new(Some(enemy),0.35).unwrap());}
  if (1..=3).contains(&stage){let e=r.adapter.combat_enemies(&r.local,game)?.into_iter().find(|e|e.id==enemy).ok_or("Missing marked enemy")?;r.log.info(&format!("marks121 native: stage={stage} marked={} vulnerability={}",e.marked,e.vulnerability));
   if stage==1{combat::fixture_mark(1,asx_gameplay::Mark::default());}else if stage==2{combat::fixture_mark(0,asx_gameplay::Mark::default());}
  }
  if stage==3{combat::fixture_talents(0,asx_gameplay::Talents{fortified:true,..Default::default()});r.adapter.attack_actor(&io,game,0,enemy,23922,1.)?;}
  if stage==4{r.log.info(&format!("marks121 taunt: spell=23922 owner={:?}",combat::fixture_owner(enemy)));combat::fixture_talents(1,asx_gameplay::Talents{ardent_defender:true,..Default::default()});r.adapter.attack_actor(&io,game,1,enemy,853,1.)?;}
  if stage==5{r.log.info(&format!("marks121 taunt: spell=853 owner={:?}",combat::fixture_owner(enemy)));combat::fixture_talents(0,asx_gameplay::Talents{dire_bear_form:true,..Default::default()});r.adapter.attack_actor(&io,game,0,enemy,5221,1.)?;}
  if stage==6{r.log.info(&format!("marks121 taunt: spell=5221 owner={:?}",combat::fixture_owner(enemy)));}
  *STATE.lock().unwrap()=(stage+1,p.clock,enemy);Ok(())
 })();if let Err(e)=result{r.log.warn(&format!("Marks121 fixture rejected: {e}"));*STATE.lock().unwrap()=(7,0.,0);}
}

// One combined native naming/periodic-text trial; self HoTs exercise native ticking.
fn remaining_ui_tick(r:&Runtime,server:u64){
 static WITNESSES:std::sync::Mutex<Vec<(String,u64)>>=std::sync::Mutex::new(Vec::new());
 static STATE:std::sync::Mutex<(u8,f32,u8)>=std::sync::Mutex::new((0,0.,0));
 let Ok(Some(party))=r.adapter.party(&r.local,server)else{return};
 if party.actors.len()!=2||party.clock<6.||party.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 let Ok(game)=r.adapter.game_address(server)else{return};let io=io::Access{local:&r.local,adapter:r.adapter};
 let ids=WITNESSES.lock().map(|v|v.clone()).unwrap_or_default();
 if !ids.is_empty()&&party.clock<40.{let _=r.adapter.fixture_world_ui_hold(&io,game,&ids);}
 let(stage,began,attempt)=*STATE.lock().unwrap();
 if stage==0{
  *STATE.lock().unwrap()=(1,party.clock,0);
  let npcs=unsafe{r.adapter.fixture_world_ui_npcs(&io,r.local.base as u64,game)};
  r.log.info(&format!("remaining UI native NPCs: {npcs:?}"));
  if let Ok(ids)=npcs{*WITNESSES.lock().unwrap()=ids;}
 }else if stage<=2&&attempt<4&&party.clock>=began+3.{
  let caster=u32::from(stage-1);
  let prototype=SUPPORT_PROTOTYPE.lock().ok().and_then(|p|p[caster as usize].clone());
  let Some(mut ability)=prototype else{return};
  *STATE.lock().unwrap()=(stage,party.clock,attempt+1);
  let cast=(||->Result<bool,String>{
   r.adapter.fixture_world_ui_wound(&io,game,caster)?;
   let spell=if caster==0{139}else{774};
   let def=r.adapter.fixture_support_definition(spell)?;
   ability[..8].copy_from_slice(&(r.local.base as u64+def).to_le_bytes());
   support::fixture_target(Some(caster),||r.adapter.fixture_cast_resolved(&io,game,caster,ability.as_ptr()as u64))
  })();
  let spell=if caster==0{139}else{774};
  r.log.info(&format!("remaining UI self HoT: caster={caster} spell={spell} result={cast:?}"));
  if cast==Ok(true){*STATE.lock().unwrap()=(stage+1,party.clock,0);}

 }
}
