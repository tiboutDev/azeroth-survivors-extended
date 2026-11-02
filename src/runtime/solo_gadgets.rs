//! One solo engineering reward slot; native effects own combat and movement.
use super::{event_services::Services, io, Runtime, RUNTIME};
use azsx_game::official::SoloMortarLaunch;
use azsx_gameplay::{
    solo_events::{sapper_backlash_budget, Equipment, Gadget},
    Phase,
};
use std::{collections::BTreeMap, sync::Mutex};
static STATE: Mutex<Option<State>> = Mutex::new(None);
struct Receipt {
    launch: Option<SoloMortarLaunch>,
    early_impact: Option<u64>,
    expires: f32,
}
struct State {
    game: u64,
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
    r.adapter
        .solo_run(&r.local, s.game)
        .ok()
        .flatten()
        .is_some_and(|p| p.clock >= s.clock && p.actors.first().is_some_and(|a| a.guid == s.owner))
}
fn refresh(r: &Runtime, game: u64, owner: u64) -> Result<(), String> {
    r.events_refresh_blessings(game, &[(0, owner)])
}
pub fn reset() {
    let old = STATE.lock().ok().and_then(|mut s| s.take());
    if let (Some(r), Some(s)) = (RUNTIME.get(), old) {
        if valid(r, &s) {
            if s.scaled {
                let _ = unsafe {
                    r.adapter.solo_player_scale(
                        &r.local,
                        r.local.base as u64,
                        s.game,
                        s.owner,
                        s.scale_base,
                    )
                };
            }
            if s.speed {
                let _ = refresh(r, s.game, s.owner);
            }
        }
    }
}
/// Called after the event's once-only reward reservation, not from a heartbeat.
pub fn award(r: &Runtime, game: u64, owner: u64, kind: Option<Gadget>) -> Result<Gadget, String> {
    let p = r
        .adapter
        .solo_run(&r.local, game)?
        .ok_or("Engineering reward is solo only")?;
    let a = p
        .actors
        .first()
        .filter(|a| a.guid == owner && a.hp > 0. && a.phase == Phase::Playing)
        .ok_or("Engineering reward owner changed")?;
    let stale = STATE
        .lock()
        .map_err(|_| "Engineering state unavailable")?
        .as_ref()
        .is_some_and(|s| s.game != game || s.owner != owner || p.clock < s.clock);
    if stale {
        reset();
    }
    let (old_speed, old_scale, base, chosen, revision) = {
        let mut state = STATE.lock().map_err(|_| "Engineering state unavailable")?;
        let s = state.get_or_insert_with(|| State {
            game,
            owner,
            clock: p.clock,
            rng: game
                ^ owner
                ^ std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            equipment: None,
            revision: 0,
            serial: 0,
            speed: false,
            scaled: false,
            scale_base: r.adapter.solo_player_scale_base(),
            next_check: p.clock,
            mortars: BTreeMap::new(),
        });
        s.clock = p.clock;
        let chosen = kind.unwrap_or_else(|| Gadget::ALL[s.draw() as usize % 6]);
        let draw = s.draw();
        if let Some(e) = s.equipment.as_mut() {
            if !e.replace(chosen, p.clock, draw) {
                return Err("Engineering reward clock rolled back".into());
            }
        } else {
            s.equipment = Equipment::new(chosen, p.clock, draw);
        }
        s.revision = s
            .revision
            .checked_add(1)
            .ok_or("Engineering reward identity exhausted")?;
        let old = (s.speed, s.scaled, s.scale_base, chosen, s.revision);
        s.speed = false;
        s.scaled = false;
        s.next_check = p.clock;
        old
    };
    if old_scale {
        unsafe {
            r.adapter
                .solo_player_scale(&r.local, r.local.base as u64, game, owner, base)
        }?;
    }
    if old_speed {
        refresh(r, game, a.guid)?;
    }
    r.log.info(&format!(
        "solo gadget equipped: revision={revision} kind={chosen:?} owner={owner}"
    ));
    Ok(chosen)
}
pub fn apply_speed(r: &Runtime, game: u64) {
    let value = STATE.lock().ok().and_then(|state| {
        let s = state.as_ref().filter(|s| s.game == game && s.speed)?;
        s.equipment
            .as_ref()
            .filter(|e| e.kind == Gadget::RocketBoots && s.clock < e.effect_until())?;
        Some((s.owner, 0.35))
    });
    let Some((owner, value)) = value else { return };
    let access = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    if let Err(e) = r.adapter.solo_gadget_speed(&access, game, owner, value) {
        r.log
            .warn(&format!("Rocket Boots reconstruction rejected: {e}"));
    }
}
pub fn tick(r: &Runtime, game: u64) -> Result<(), String> {
    let Some(p) = r.adapter.solo_run(&r.local, game)? else {
        reset();
        return Ok(());
    };
    let Some(a) = p.actors.first() else {
        return Ok(());
    };
    let stale = STATE
        .lock()
        .map_err(|_| "Engineering state unavailable")?
        .as_ref()
        .is_some_and(|s| s.game != game || s.owner != a.guid || p.clock < s.clock);
    if stale {
        reset();
        return Ok(());
    }
    if a.hp <= 0. {
        reset();
        return Ok(());
    }
    let expires = {
        let mut state = STATE.lock().map_err(|_| "Engineering state unavailable")?;
        let Some(s) = state.as_mut() else {
            return Ok(());
        };
        s.clock = p.clock;
        s.mortars.retain(|_, v| v.expires >= p.clock);
        let ended = s
            .equipment
            .as_ref()
            .is_some_and(|e| p.clock >= e.effect_until());
        let speed = ended && s.speed;
        let scaled = ended && s.scaled;
        if speed {
            s.speed = false
        }
        if scaled {
            s.scaled = false
        }
        (speed, scaled, s.scale_base)
    };
    if expires.1 {
        unsafe {
            r.adapter
                .solo_player_scale(&r.local, r.local.base as u64, game, a.guid, expires.2)
        }?;
        r.log.info("solo Shrink Ray restored native scale");
    }
    if expires.0 {
        refresh(r, game, a.guid)?;
        r.log
            .info("solo Rocket Boots expired through native stat reconstruction");
    }
    if a.phase != Phase::Playing {
        return Ok(());
    }
    let check_target = STATE.lock().ok().is_some_and(|state| {
        state.as_ref().is_some_and(|s| {
            p.clock >= s.next_check
                && s.equipment
                    .as_ref()
                    .is_some_and(|e| e.kind == Gadget::Sapper && e.due(p.clock))
        })
    });
    let sapper_target =
        !check_target || r.adapter.solo_sapper_has_target(&r.local, game, a.guid)?;
    let activation = {
        let mut state = STATE.lock().map_err(|_| "Engineering state unavailable")?;
        let s = state.as_mut().ok_or("Engineering state retired")?;
        if p.clock < s.next_check {
            return Ok(());
        }
        s.next_check = p.clock + 0.2;
        let Some(e) = s.equipment.as_ref() else {
            return Ok(());
        };
        if !e.due(p.clock) {
            return Ok(());
        }
        if e.kind == Gadget::Sapper
            && sapper_backlash_budget(a.hp, a.maximum).is_none_or(|v| v <= 0.)
        {
            s.next_check = p.clock + 1.;
            r.log
                .info("solo Sapper deferred: protected 75% health budget unavailable");
            return Ok(());
        }
        if e.kind == Gadget::Sapper && !sapper_target {
            s.next_check = p.clock + 1.;
            return Ok(());
        }
        let schedule = s.draw();
        let effect_draw = s.draw();
        let e = s
            .equipment
            .as_mut()
            .ok_or("Engineering equipment missing")?;
        if !e.trigger(p.clock, true, true, schedule) {
            return Ok(());
        }
        let kind = e.kind;
        if kind == Gadget::RocketBoots {
            s.speed = true
        }
        if kind == Gadget::ShrinkRay {
            s.scaled = true
        }
        Some((kind, effect_draw, e.effect_until()))
    };
    if let Some((kind, draw, until)) = activation {
        activate(r, game, a.guid, kind, draw, until)?;
    }
    Ok(())
}
fn activate(
    r: &Runtime,
    game: u64,
    owner: u64,
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
        Gadget::RocketBoots => refresh(r, game, owner)?,
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
                let p = r
                    .adapter
                    .solo_run(&r.local, game)?
                    .ok_or("Sapper run retired")?;
                let point = p
                    .actors
                    .first()
                    .filter(|a| a.guid == owner)
                    .ok_or("Sapper owner changed")?
                    .position;
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
                r.log.info(&format!("solo Sapper native explosion: targets={hits} before_hp={before} after_hp={after} maximum_hp={} protected_floor=75%",p.actors[0].maximum));
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
                    .as_mut()
                    .filter(|s| s.game == game && s.owner == owner)
                    .ok_or("Mortar owner retired")?;
                if s.mortars.len() >= 8 {
                    return Err("Mortar receipt safety bound reached".into());
                }
                s.serial = s.serial.checked_add(1).ok_or("Mortar identity exhausted")?;
                let token = r
                    .adapter
                    .solo_new_mortar_token(s.serial)
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
            let deferred = {
                let mut state = STATE.lock().map_err(|_| "Mortar state unavailable")?;
                let s = state
                    .as_mut()
                    .filter(|s| s.game == game && s.owner == owner)
                    .ok_or("Mortar owner changed during launch")?;
                if let Some(launch) = launch {
                    let receipt = s
                        .mortars
                        .get_mut(&token)
                        .ok_or("Mortar receipt disappeared")?;
                    receipt.launch = Some(launch);
                    receipt.expires = launch.arrives + 5.;
                    let early = receipt.early_impact;
                    r.log.info(&format!("solo Mortar native launched: token={token} target={} point={:?} arrives={}",launch.target,launch.point,launch.arrives));
                    if early.is_some() {
                        let receipt = s.mortars.remove(&token).unwrap();
                        Some((receipt, early.unwrap()))
                    } else {
                        None
                    }
                } else {
                    s.mortars.remove(&token);
                    None
                }
            };
            if let Some((receipt, enemy)) = deferred {
                deliver_mortar(r, game, owner, receipt, enemy)?;
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
    let result = (|| -> Result<(), String> {
        let Some(token) = r.adapter.solo_mortar_token(&r.local, spell, effect)? else {
            return Ok(());
        };
        let receipt = {
            let mut state = STATE.lock().map_err(|_| "Mortar state unavailable")?;
            let Some(s) = state.as_mut().filter(|s| s.game == game) else {
                return Ok(());
            };
            let Some(receipt) = s.mortars.get_mut(&token) else {
                return Ok(());
            };
            if receipt.launch.is_none() {
                receipt.early_impact = Some(enemy);
                return Ok(());
            }
            let owner = s.owner;
            Some((
                owner,
                s.mortars.remove(&token).ok_or("Mortar receipt missing")?,
            ))
        };
        if let Some((owner, receipt)) = receipt {
            deliver_mortar(r, game, owner, receipt, enemy)?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        r.log.warn(&format!(
            "Engineering impact stopped: {e}; no uncertain impact is retried"
        ));
        reset();
    }
}
pub fn wire(game: u64, owner: u64) -> Option<String> {
    let state = STATE.lock().ok()?;
    let s = state
        .as_ref()
        .filter(|s| s.game == game && s.owner == owner)?;
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
