//! Exact-build verification and ordered installation of official-engine hooks.
use super::{
    addon_activation, nameplates, combat_text, blessings, bonus_tooltip, bonus_ui, combat, duplicate_loot,
    elites, experience, fatigue, glyphs, local_base, menu_pause, menu_ui, party_ui,
    pause, reward_cards, run_stats, session, support, wallet, xp_hooks, Adapter, Runtime, RUNTIME,
};
#[cfg(feature = "test-hooks")]
use super::presentation;
use crate::{log::Log, proc::Local};

pub(crate) fn install(log: &'static Log, sha: &str) -> Result<(), String> {
    let adapter = Adapter::select(sha)?;
    log.info(&format!("ASX capabilities: glyphs={} duplicate-gold={} native-marks={}",adapter.glyphs_enabled(),adapter.duplicate_loot_enabled(),adapter.version()=="0.125.0"));
    // Until gameplay capabilities are connected, activation is an explicit
    // observation trial. Never present an unmodified native run as mod parity.
    let asx = std::env::var("ASX_ACTIVE").as_deref() == Ok("1");
    if !asx && std::env::var("AZCOOP_OFFICIAL_OBSERVE").as_deref() != Ok("1") {
        return Err(
            "Official gameplay migration is not playable yet; explicit observation trial required"
                .into(),
        );
    }
    let local = Local::game()?;
    adapter.verify(&local, local.base as u64)?;
    let rules = asx || std::env::var("AZCOOP_OFFICIAL_GAMEPLAY").as_deref() == Ok("1");
    RUNTIME
        .set(Runtime {
            adapter,
            local,
            log,
            rules,
            sampled: std::sync::Mutex::new(std::time::Instant::now()),
            party: std::sync::Mutex::new(None),
            solo: std::sync::Mutex::new(None),
        })
        .map_err(|_| "Official runtime already installed")?;
    // The column builder is also the menu group's constructor. Verify its native
    // bytes before menu_ui installs that shared hook, not against our detour.
    let runtime=RUNTIME.get().ok_or("Official runtime unavailable")?;
    for entry in adapter.bonus_column_entries(){if runtime.local.read(local_base()+entry.rva as usize,entry.bytes.len())?!=entry.bytes{return Err(format!("Native bonus column boundary changed at {:#x}",entry.rva))}}
    if rules {
        crate::presentation::pause::install_selected(
            local_base(),
            adapter.pause_layout(),
        )?;
    }
    if rules {
        let entry=adapter.party_addon_entry();
        let r=RUNTIME.get().ok_or("Official runtime unavailable")?;
        if r.local.read(local_base()+entry.rva as usize,entry.bytes.len())?!=entry.bytes {return Err("Native addon-line constructor changed".into())}
        let spell=adapter.spell_presentation_entry();
        if r.local.read(local_base()+spell.rva as usize,spell.bytes.len())?!=spell.bytes {return Err("Native owner spell formatter changed".into())}
        for entry in adapter.menu_entries().into_iter().chain(adapter.menu_cache_entries()){if r.local.read(local_base()+entry.rva as usize,entry.bytes.len())?!=entry.bytes{return Err(format!("Native menu class-decoration entry changed at {:#x}",entry.rva))}}
        crate::install::install(&menu_ui::hooks(adapter))?;
        crate::install::install(&[party_ui::hook(adapter),party_ui::spell_hook(adapter)])?;
    }
    crate::install::install(&wallet::hooks(adapter))?;
    if let Some(entry)=adapter.fatigue_aura_entry() {
        let r=RUNTIME.get().ok_or("Missing official runtime")?;
        if r.local.read(local_base()+entry.rva as usize,entry.bytes.len())?!=entry.bytes {return Err("Native fatigue aura constructor changed".into());}
    }
    crate::install::install(&fatigue::hooks(adapter))?;
    let r=RUNTIME.get().ok_or("Missing official runtime")?;
    for e in adapter.blessing_entries(){
        if r.local.read(local_base()+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Native blessing boundary changed".into());}
    }
    crate::install::install(&blessings::hooks(adapter))?;
    crate::install::install(&[run_stats::hook(adapter)])?;
    for e in adapter.loot_entries().into_iter().chain(duplicate_loot::fixture_entry(adapter)){if r.local.read(local_base()+e.rva as usize,e.bytes.len())?!=e.bytes{return Err(format!("Native duplicate-loot boundary changed at {:#x}",e.rva));}}
    crate::install::install(&duplicate_loot::hooks(adapter))?;

    for e in adapter.reward_entries().into_iter().chain([adapter.native_hit_effect_entry()]).chain(if adapter.glyphs_enabled(){adapter.glyph_entries()}else{vec![]}){
        if r.local.read(local_base()+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Native reward/effect boundary changed".into());}
    }
    crate::install::install(&reward_cards::hooks(adapter))?;
    if adapter.glyphs_enabled(){crate::install::install(&glyphs::hooks(adapter))?;}
    for e in adapter.bonus_tooltip_entries(){if r.local.read(local_base()+e.rva as usize,e.bytes.len())?!=e.bytes{return Err(format!("Native glyph hover boundary changed at {:#x}",e.rva));}}
    #[cfg(feature="test-hooks")]if bonus_tooltip::fixture_enabled(){if let Some(e)=adapter.kept_tooltip_fixture_focus(){if r.local.read(local_base()+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Native tooltip fixture focus contract changed".into())}}}
    crate::install::install(&bonus_tooltip::hooks(adapter))?;
    for e in adapter.bonus_ui_entries().into_iter().chain(adapter.bonus_separator_entries()){if r.local.read(local_base()+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Native glyph UI boundary changed".into());}}
    crate::install::install(&bonus_ui::ui_hooks(adapter))?;
    if rules {
        for entry in adapter.combat_text_entries().into_iter().chain(adapter.nameplate_entries()).chain([adapter.ui_unit_hash_entry()]){if r.local.read(local_base()+entry.rva as usize,entry.bytes.len())?!=entry.bytes{return Err(format!("Native combat-text boundary changed at {:#x}",entry.rva))}}
        crate::install::install(&combat_text::hooks(adapter))?;
        crate::install::install(&nameplates::hooks(adapter))?;
    }
    crate::install::install(&support::hooks(adapter))?;
    crate::install::install(&combat::hooks(adapter))?;
    #[cfg(feature="test-hooks")]
    if std::env::var("AZCOOP_TEST_OFFICIAL_PARTY_UI").as_deref()==Ok("1") {
        let entry=adapter.ui_mailbox_writer().ok_or("Native CVar diagnostic channel is unavailable for this build; use the read-only visual audit")?;
        if RUNTIME.get().ok_or("Missing runtime")?.local.read(local_base()+entry.rva as usize,entry.bytes.len())?!=entry.bytes{return Err("Native diagnostic writer changed".into())}
        crate::install::install(&[presentation::hook(adapter)])?;
    }
    crate::install::install(&[experience::hook(adapter)])?;
    if let Some(runtime) = RUNTIME.get().filter(|r| r.rules) {
        combat::install_descriptions(runtime)?;
    }
    if adapter.needs_guardian_item_supplement(){crate::install::install(&[elites::hook(adapter)])?;}
    let [shared_xp, award_xp] = xp_hooks::hooks(adapter, local_base());
    let [ui_hud, ui_event] = pause::hooks(adapter, local_base());
    let [server, hud, world, command] = session::hooks(adapter, local_base());
    crate::install::install(&[
        shared_xp,
        award_xp,
        ui_hud,
        ui_event,
        server,
        hud,
        world,
        command,
    ])?;

    if rules {if let Some(e)=adapter.menu_entry(){let r=RUNTIME.get().ok_or("Missing runtime")?;for entry in [Some(e),adapter.menu_key_entry()].into_iter().flatten(){if r.local.read(local_base()+entry.rva as usize,entry.bytes.len())?!=entry.bytes{return Err("Native menu input changed".into())}}crate::install::install(&menu_pause::hooks(adapter))?;}}
    adapter.verify_native_difficulty(&RUNTIME.get().ok_or("Missing runtime")?.local, local_base() as u64)?;
    log.info(&format!("Official difficulty: native HP, damage and density; {} legacy sites verified intact.",adapter.native_difficulty_entries().len()));
    let ui_active=asx;
    #[cfg(feature="test-hooks")]
    let ui_active=ui_active||std::env::var("AZCOOP_TEST_OFFICIAL_PARTY_UI").as_deref()==Ok("1");
    if ui_active {addon_activation::install(&RUNTIME.get().ok_or("Missing ASX runtime")?.local,log)?;}
    if asx {
        log.info("ASX active: official multiplayer, shared progression rules and native interface extensions enabled.");
        if let Some(path)=std::env::var_os("ASX_READY_FILE") {
            std::fs::write(path,serde_json::json!({"product":"asx","game_sha256":sha}).to_string()).map_err(|e|format!("ASX startup acknowledgement failed: {e}"))?;
        }
    }else {
        if rules {log.info("Official gameplay trial: group barrier, retained pause presentation and shared XP adaptation enabled; remaining gameplay parity pending.");}
        log.info("Official engine selected: native simulation and transport retained; custom engine inactive. Gameplay parity pending; isolated partial integration trial.");
    }
    Ok(())
}
