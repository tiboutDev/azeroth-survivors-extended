//! Run-local engineering equipment per native participant.
use super::{event_services::Services, io, Runtime, RUNTIME};
use azsx_game::official::{ActorObservation, SoloMortarLaunch};
use azsx_gameplay::{solo_events::{sapper_backlash_budget, Equipment, Gadget}, Phase};
use std::{collections::BTreeMap, sync::Mutex};

static STATE: Mutex<BTreeMap<u64, State>> = Mutex::new(BTreeMap::new());
struct Receipt {
    launch: Option<SoloMortarLaunch>,
    early_impact: Option<u64>,
    expires: f32,
}
struct State {
    game: u64,
    slot: u32,
    owner: u64,
    clock: f32,
    rng: u64,
    equipment: Option<Equipment>,
    revision: u64,
    serial: u32,
    speed: bool,
    scaled: bool,
    scale_base: f32,
    next_check: f32,
    mortars: BTreeMap<u64, Receipt>,
}
impl State {
    fn draw(&mut self) -> u32 {
        self.rng = self.rng.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        (z ^ (z >> 31)) as u32
    }
}
fn valid(r: &Runtime, s: &State) -> bool {
    r.adapter.event_party(&r.local, s.game).ok().flatten().is_some_and(|p|
        p.clock >= s.clock && p.actors.iter().any(|a| a.slot == s.slot && a.guid == s.owner))
}
fn refresh(r: &Runtime, game: u64, slot: u32, owner: u64) -> Result<(), String> {
    r.events_refresh_blessings(game, &[(slot, owner)])
}
fn retire(r: &Runtime, s: State) {
    if !valid(r, &s) { return }
    if s.scaled {
        let _ = unsafe { r.adapter.solo_player_scale(&r.local, r.local.base as u64,
            s.game, s.owner, s.scale_base) };
    }
    if s.speed { let _ = refresh(r, s.game, s.slot, s.owner); }
}
pub fn reset() {
    let old = STATE.lock().ok().map(|mut guard| std::mem::take(&mut *guard));
    if let (Some(r), Some(old)) = (RUNTIME.get(), old) {
        for (_, state) in old { retire(r, state); }
    }
}
/// Reserve all individual rewards before any native effect runs. A failed
/// recipient preflight grants nobody and can never replay a partial group.
pub fn award_group(r: &Runtime, game: u64, recipients: &[(u32,u64)])
    -> Result<Vec<(u32,u64,Gadget)>, String> {
    let p = r.adapter.event_party(&r.local, game)?.ok_or("Engineering run retired")?;
    if recipients.is_empty() || recipients.len() > azsx_game::official::NATIVE_PLAYERS
        || recipients.len() != p.actors.iter().filter(|a| a.phase != Phase::Inactive).count()
        || recipients.iter().any(|(slot, owner)| *owner == 0 || !p.actors.iter().any(|a|
            a.slot == *slot && a.guid == *owner && a.hp > 0. && a.phase == Phase::Playing))
        || recipients.iter().enumerate().any(|(i, id)| recipients[..i].contains(id)) {
        return Err("Engineering recipient roster changed".into());
    }
    let mut planned = BTreeMap::new();
    let mut grants = Vec::new();
    let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_nanos() as u64;
    for &(slot, owner) in recipients {
        let mut s = State { game, slot, owner, clock:p.clock, rng:game ^ owner ^ seed,
            equipment:None, revision:1, serial:0, speed:false, scaled:false,
            scale_base:r.adapter.solo_player_scale_base(), next_check:p.clock,
            mortars:BTreeMap::new() };
        let kind = Gadget::ALL[s.draw() as usize % Gadget::ALL.len()];
        let draw = s.draw();
        s.equipment = Equipment::new(kind, p.clock, draw);
        if s.equipment.is_none() { return Err("Engineering equipment rejected".into()); }
        grants.push((slot, owner, kind));
        planned.insert(owner, s);
    }
    let mut state = STATE.lock().map_err(|_| "Engineering state unavailable")?;
    if state.values().any(|s| s.game == game && s.equipment.is_some()) {
        return Err("Engineering reward already reserved".into());
    }
    *state = planned;
    for (slot,owner,kind) in &grants {
        r.log.info(&format!("gadget equipped: slot={slot} owner={owner} kind={kind:?}"));
    }
    Ok(grants)
}
fn with_actor<T>(r: &Runtime, game: u64, slot: u32, owner: u64,
    action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let access = io::Access { local: &r.local, adapter: r.adapter };
    r.adapter.engineering_with_actor(&access, game, slot, owner, action)
}
pub fn apply_speed(r: &Runtime, game: u64) {
    let Ok(Some((slot, owner))) = r.adapter.run_source(&r.local, game) else { return };
    let value = STATE.lock().ok().and_then(|state| {
        let s = state.get(&owner).filter(|s| s.game == game && s.slot == slot && s.speed)?;
        s.equipment.as_ref().filter(|e| e.kind == Gadget::RocketBoots
            && s.clock < e.effect_until())?;
        Some(0.35)
    });
    let Some(value) = value else { return };
    let access = io::Access { local: &r.local, adapter: r.adapter };
    if let Err(e) = r.adapter.solo_gadget_speed(&access, game, owner, value) {
        r.log.warn(&format!("Rocket Boots reconstruction rejected: {e}"));
    }
}
pub fn tick(r: &Runtime, game: u64) -> Result<(), String> {
    let Some(p) = r.adapter.event_party(&r.local, game)? else { reset(); return Ok(()) };
    let stale = STATE.lock().map_err(|_| "Engineering state unavailable")?
        .values().any(|s| s.game != game || p.clock < s.clock);
    if stale { reset(); return Ok(()) }
    let departed = {
        let mut state = STATE.lock().map_err(|_| "Engineering state unavailable")?;
        let ids = state.iter().filter(|(_, s)| !p.actors.iter().any(|a|
            a.slot == s.slot && a.guid == s.owner && a.hp > 0. && a.phase != Phase::Inactive))
            .map(|(id, _)| *id).collect::<Vec<_>>();
        ids.into_iter().filter_map(|id| state.remove(&id)).collect::<Vec<_>>()
    };
    for s in departed { retire(r, s); }
    for a in &p.actors { tick_actor(r, game, p.clock, a)?; }
    Ok(())
}
fn tick_actor(r: &Runtime, game: u64, now: f32, a: &ActorObservation) -> Result<(), String> {
    let (speed, scaled, base, pending) = {
        let mut state = STATE.lock().map_err(|_| "Engineering state unavailable")?;
        let Some(s) = state.get_mut(&a.guid).filter(|s| s.game == game && s.slot == a.slot) else {
            return Ok(());
        };
        s.clock = now;
        s.mortars.retain(|_, v| v.expires >= now);
        let ended = s.equipment.as_ref().is_some_and(|e| now >= e.effect_until());
        let speed = ended && std::mem::take(&mut s.speed);
        let scaled = ended && std::mem::take(&mut s.scaled);
        let pending = s.mortars.iter().find_map(|(token, receipt)|
            (receipt.launch.is_some() && receipt.early_impact.is_some()).then_some(*token));
        let pending = pending.and_then(|token| s.mortars.remove(&token))
            .and_then(|receipt| receipt.early_impact.map(|enemy| (receipt, enemy)));
        (speed, scaled, s.scale_base, pending)
    };
    if scaled {
        unsafe { r.adapter.solo_player_scale(&r.local, r.local.base as u64,
            game, a.guid, base) }?;
    }
    if speed { refresh(r, game, a.slot, a.guid)?; }
    if let Some((receipt, enemy)) = pending {
        with_actor(r, game, a.slot, a.guid, || deliver_mortar(r, game, a.guid, receipt, enemy))?;
    }
    if a.phase != Phase::Playing || a.hp <= 0. { return Ok(()) }
    let check_target = STATE.lock().ok().is_some_and(|state| state.get(&a.guid)
        .is_some_and(|s| now >= s.next_check && s.equipment.as_ref()
            .is_some_and(|e| e.kind == Gadget::Sapper && e.due(now))));
    let sapper_target = !check_target || with_actor(r, game, a.slot, a.guid,
        || r.adapter.solo_sapper_has_target(&r.local, game, a.guid))?;
    let activation = {
        let mut state = STATE.lock().map_err(|_| "Engineering state unavailable")?;
        let Some(s) = state.get_mut(&a.guid).filter(|s| s.game == game && s.slot == a.slot) else {
            return Ok(());
        };
        if now < s.next_check { return Ok(()) }
        s.next_check = now + 0.2;
        let Some(e) = s.equipment.as_ref() else { return Ok(()) };
        if !e.due(now) { return Ok(()) }
        if e.kind == Gadget::Sapper &&
            (sapper_backlash_budget(a.hp, a.maximum).is_none_or(|v| v <= 0.) || !sapper_target) {
            s.next_check = now + 1.;
            return Ok(())
        }
        let schedule = s.draw();
        let effect_draw = s.draw();
        let e = s.equipment.as_mut().ok_or("Engineering equipment missing")?;
        if !e.trigger(now, true, true, schedule) { return Ok(()) }
        let kind = e.kind;
        if kind == Gadget::RocketBoots { s.speed = true; }
        if kind == Gadget::ShrinkRay { s.scaled = true; }
        Some((kind, effect_draw, e.effect_until()))
    };
    if let Some((kind, draw, until)) = activation {
        if matches!(kind, Gadget::Net | Gadget::ProtectionBelt | Gadget::Sapper | Gadget::Mortar) {
            with_actor(r, game, a.slot, a.guid,
                || activate(r, game, a.slot, a.guid, a.position, a.maximum, kind, draw, until))?;
        } else {
            activate(r, game, a.slot, a.guid, a.position, a.maximum, kind, draw, until)?;
        }
    }
    Ok(())
}
fn activate(
    r: &Runtime,
    game: u64,
    slot: u32,
    owner: u64,
    position: [f32; 3],
    maximum: f32,
    kind: Gadget,
    draw: u32,
    until: f32,
) -> Result<(), String> {
    r.log.info(&format!(
        "solo gadget activated: kind={kind:?} until={until}"
    ));
    match kind {
        Gadget::Net => {
            let count = unsafe {
                r.adapter
                    .solo_gadget_net(&r.local, r.local.base as u64, game, owner)
            }?;
            r.log
                .info(&format!("solo Net native rooted targets: {count}"));
        }
        Gadget::RocketBoots => refresh(r, game, slot, owner)?,
        Gadget::ProtectionBelt => {
            let (before, after) = unsafe {
                r.adapter
                    .solo_gadget_belt(&r.local, r.local.base as u64, game, owner)
            }?;
            r.log.info(&format!(
                "solo Belt native shield: before={before} after={after}"
            ));
        }
        Gadget::Sapper => {
            if let Some((before, after)) = r.adapter.solo_sapper_backlash(
                &io::Access {
                    local: &r.local,
                    adapter: r.adapter,
                },
                game,
                owner,
            )? {
                let point = position;
                let hits = unsafe {
                    r.adapter.solo_engineering_burst(
                        &r.local,
                        r.local.base as u64,
                        game,
                        owner,
                        point,
                        true,
                    )
                }?;
                r.log.info(&format!("solo Sapper native explosion: targets={hits} before_hp={before} after_hp={after} maximum_hp={} protected_floor=75%",maximum));
            }
        }
        Gadget::ShrinkRay => {
            let scale = match draw % 3 {
                0 => 0.8,
                1 => 1.,
                _ => 1.2,
            };
            unsafe {
                r.adapter
                    .solo_player_scale(&r.local, r.local.base as u64, game, owner, scale)
            }?;
            r.log.info(&format!("solo Shrink Ray native visual: scale={scale} until={until} gameplay_fields_changed=false"));
        }
        Gadget::Mortar => {
            let token = {
                let mut state = STATE.lock().map_err(|_| "Mortar state unavailable")?;
                let s = state
                    .get_mut(&owner)
                    .filter(|s| s.game == game && s.slot == slot)
                    .ok_or("Mortar owner retired")?;
                if s.mortars.len() >= 8 {
                    return Err("Mortar receipt safety bound reached".into());
                }
                s.serial = s.serial.checked_add(1).filter(|n| *n <= 0xffffff)
                    .ok_or("Mortar identity exhausted")?;
                let token = r
                    .adapter
                    .solo_new_mortar_token((slot << 24) | s.serial)
                    .ok_or("Mortar identity is zero")?;
                s.mortars.insert(
                    token,
                    Receipt {
                        launch: None,
                        early_impact: None,
                        expires: s.clock + 12.,
                    },
                );
                token
            };
            let launch = unsafe {
                r.adapter.solo_launch_mortar(
                    &r.local,
                    r.local.base as u64,
                    game,
                    owner,
                    draw,
                    token,
                )
            }?;
            let mut state = STATE.lock().map_err(|_| "Mortar state unavailable")?;
            let s = state.get_mut(&owner).filter(|s| s.game == game && s.slot == slot)
                .ok_or("Mortar owner changed during launch")?;
            if let Some(launch) = launch {
                let receipt = s.mortars.get_mut(&token).ok_or("Mortar receipt disappeared")?;
                receipt.launch = Some(launch);
                receipt.expires = launch.arrives + 5.;
                r.log.info(&format!("Mortar native launched: owner={owner} token={token} target={}", launch.target));
            } else {
                s.mortars.remove(&token);
            }
        }
    }
    Ok(())
}
fn deliver_mortar(
    r: &Runtime,
    game: u64,
    owner: u64,
    receipt: Receipt,
    enemy: u64,
) -> Result<(), String> {
    let launch = receipt
        .launch
        .ok_or("Mortar impact arrived before its launch was bound")?;
    if launch.target != enemy {
        return Err("Mortar impact target identity changed".into());
    }
    let hits = unsafe {
        r.adapter.solo_engineering_burst(
            &r.local,
            r.local.base as u64,
            game,
            owner,
            launch.point,
            false,
        )
    }?;
    r.log.info(&format!(
        "solo Mortar native impact: target={enemy} area_hits={hits} radius=6 original_impact=true"
    ));
    Ok(())
}
pub fn impact(r: &Runtime, game: u64, enemy: u64, effect: u64, spell: u32) {
    let Ok(Some(token)) = r.adapter.solo_mortar_token(&r.local, spell, effect) else { return };
    if let Ok(mut state) = STATE.lock() {
        for s in state.values_mut().filter(|s| s.game == game) {
            if let Some(receipt) = s.mortars.get_mut(&token) {
                if receipt.early_impact.is_none() { receipt.early_impact = Some(enemy); }
                return;
            }
        }
    }
}
pub fn wire(game: u64, owner: u64) -> Option<String> {
    let state = STATE.lock().ok()?;
    let s = state.get(&owner).filter(|s| s.game == game)?;
    let e = s.equipment.as_ref()?;
    let kind = Gadget::ALL.iter().position(|v| *v == e.kind)?;
    let centis = |v: f32| ((v - s.clock).max(0.) * 100.).round().min(60000.) as u32;
    Some(format!(
        "W|{owner}|{}|{kind}|{}|{}\n",
        s.revision,
        centis(e.next_trigger()),
        centis(e.effect_until())
    ))
}
