//! Host-owned cooperative event lifecycle. Native packets own scene replication;
//! portable supply/ritual rules own claims, duration and progression.
use super::super::event_services::Services;
use super::super::RUNTIME;
use asx_game::official::{EventObject, PartyObservation, RitualScene};
use asx_gameplay::{
    Phase,
    blessings::Blessing,
    rituals::{Outcome, Ritual},
    supplies::{Buffs, Consumable, Discovery, Family, Reward, Supply},
};
use std::{collections::BTreeMap, sync::Mutex};
type Id = (u32, u64);
#[derive(Default)]
struct Person {
    buffs: Buffs,
    auras: BTreeMap<Family, u32>,
    discovery: Option<Discovery>,
    fortune: Option<(u32, Blessing)>,
}
struct Reserve {
    rules: Supply,
    object: EventObject,
}
struct State {
    game: u64,
    owner: u64,
    clock: f32,
    rng: u64,
    stage: u32,
    next_supply: f32,
    next_ritual: Option<f32>,
    sequence: u64,
    reserves: Vec<Reserve>,
    people: BTreeMap<Id, Person>,
    ritual: Option<Ritual>,
    scene: Option<RitualScene>,
    stopped: bool,
    fixture: u8,
    #[cfg(feature="test-hooks")]
    fixture_waited: bool,
    poses: BTreeMap<u64, ([f32; 3], f32, Phase)>,
}
impl State {
    fn draw(&mut self, n: u32) -> u32 {
        loop {
            self.rng = self.rng.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = self.rng;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^= z >> 31;
            let max = u64::MAX - u64::MAX % u64::from(n);
            if z < max {
                return (z % u64::from(n)) as u32;
            }
        }
    }
}
static STATE: Mutex<Option<State>> = Mutex::new(None);
fn party(r: &dyn Services, game: u64) -> Result<Option<PartyObservation>, String> {
    if r.events_run_source(game)?
        .is_none_or(|(slot, _)| slot != 0)
        || r.events_solo_run(game)?.is_some()
    {
        return Ok(None);
    }
    r.events_world_party(game)
}
pub fn reset() {
    let old = STATE.lock().ok().and_then(|mut s| s.take());
    if let (Some(r), Some(s)) = (RUNTIME.get(), old) {
        if party(r, s.game)
            .ok()
            .flatten()
            .is_some_and(|p| p.actors.iter().any(|a| a.slot == 0 && a.guid == s.owner))
        {
            for reserve in s.reserves {
                let _ = unsafe {
                    r.events_remove_event_object(s.game, reserve.object)
                };
            }
            if let Some(scene) = s.scene {
                let _ = unsafe {
                    r.events_remove_ritual_scene(s.game, &scene)
                };
            }
        }
    }
}
pub fn encounter(r: &dyn Services, game: u64, total: u64) {
    let endless = r.events_endless(game).unwrap_or(false);
    if let Ok(mut state) = STATE.lock() {
        if let Some(s) = state.as_mut().filter(|s| s.game == game && !s.stopped) {
            s.stage = s.stage.saturating_add(1);
            if let Some(v) = s.ritual.as_mut() {
                v.promote(s.stage)
            } else if super::super::rituals::enabled() && (u64::from(s.stage) < total || endless) {
                s.next_ritual = Some(
                    s.clock
                        + if fixture_enabled() {
                            3.
                        } else {
                            20. + s.draw(21) as f32
                        },
                );
                #[cfg(feature="test-hooks")]
                if super::super::scenario_steps::enabled(){s.next_ritual=Some(s.clock);}
            }
        }
    }
}
pub fn buffs(game: u64, owner: u64) -> Vec<(Consumable, f32, u32)> {
    STATE
        .lock()
        .ok()
        .and_then(|s| {
            let s = s.as_ref().filter(|s| s.game == game)?;
            let p = s.people.iter().find(|((_, g), _)| *g == owner)?.1;
            Some(
                p.buffs
                    .values()
                    .into_iter()
                    .map(|b| {
                        (
                            b.consumable,
                            (b.expires - s.clock).max(0.),
                            p.auras.get(&b.consumable.family).copied().unwrap_or(32),
                        )
                    })
                    .collect(),
            )
        })
        .unwrap_or_default()
}
pub fn discovery(game: u64, owner: u64) -> Option<Discovery> {
    STATE.lock().ok().and_then(|s| {
        s.as_ref()
            .filter(|s| s.game == game)?
            .people
            .iter()
            .find(|((_, g), _)| *g == owner)?
            .1
            .discovery
    })
}
pub fn aura(game: u64, owner: u64, rows: &[(Family, u32)]) {
    if let Ok(mut s) = STATE.lock() {
        if let Some(p) = s.as_mut().filter(|s| s.game == game).and_then(|s| {
            s.people
                .iter_mut()
                .find(|((_, g), _)| *g == owner)
                .map(|(_, p)| p)
        }) {
            p.auras = rows.iter().copied().collect();
        }
    }
}
pub fn has_aura(game: u64, owner: u64) -> bool {
    STATE.lock().is_ok_and(|s| {
        s.as_ref().is_some_and(|s| {
            s.game == game
                && s.people
                    .iter()
                    .any(|((_, g), p)| *g == owner && !p.auras.is_empty())
        })
    })
}
pub fn view(game: u64, owner: u64) -> Option<(u32, u32, bool)> {
    STATE.lock().ok().and_then(|s| {
        let s = s.as_ref().filter(|s| s.game == game)?;
        let v = s.ritual.as_ref()?;
        let (pos, hp, phase) = s.poses.get(&owner)?;
        Some((
            v.id,
            (v.held * 100.).round() as u32,
            inside(v.position, *pos, *hp, *phase),
        ))
    })
}

fn inside(point: [f32; 3], position: [f32; 3], hp: f32, phase: Phase) -> bool {
    hp > 0.
        && phase == Phase::Playing
        && (position[0] - point[0]).hypot(position[1] - point[1]) <= asx_gameplay::rituals::RADIUS
}
pub fn occupied(game: u64) -> Vec<([f32; 3], f32)> {
    STATE
        .lock()
        .ok()
        .and_then(|s| {
            let s = s.as_ref().filter(|s| s.game == game)?;
            let mut out = s
                .reserves
                .iter()
                .map(|v| (v.rules.position, 2.))
                .collect::<Vec<_>>();
            if let Some(v) = &s.ritual {
                out.push((v.position, 8.));
            }
            Some(out)
        })
        .unwrap_or_default()
}
pub fn markers(game: u64, owner: u64) -> Vec<super::Marker> {
    STATE
        .lock()
        .ok()
        .and_then(|s| {
            let s = s.as_ref().filter(|s| s.game == game)?;
            let id = *s.people.keys().find(|(_, g)| *g == owner)?;
            let mut out = s
                .reserves
                .iter()
                .filter(|v| v.rules.available(id))
                .map(|v| super::Marker {
                    id: v.rules.event,
                    kind: 0,
                    position: v.rules.position,
                    remaining: (v.rules.expires - s.clock).max(0.),
                })
                .collect::<Vec<_>>();
            if let Some(v) = &s.ritual {
                out.push(super::Marker {
                    id: u64::from(v.id),
                    kind: 1,
                    position: v.position,
                    remaining: 0.,
                });
            }
            Some(out)
        })
        .unwrap_or_default()
}
fn notice(r: &dyn Services, game: u64, sender: &str, text: &str) -> Result<(), String> {
    
    unsafe {
        r.events_notice(game, sender, text)
    }
}
pub fn owns(game: u64) -> bool {
    STATE
        .lock()
        .is_ok_and(|s| s.as_ref().is_some_and(|s| s.game == game && !s.stopped))
}
pub fn available(game: u64) -> bool {
    STATE
        .lock()
        .is_ok_and(|s| s.as_ref().is_none_or(|s| s.game != game || !s.stopped))
}
pub fn finish(r: &dyn Services, game: u64) {
    if !available(game) {
        return;
    }
    if let Err(e) = tick(r, game) {
        r.events_warn(&format!(
            "Cooperative events stopped: {e}; core ASX remains active."
        ));
        let old = STATE.lock().ok().and_then(|mut guard| {
            let s = guard.as_mut()?;
            s.stopped = true;
            s.ritual = None;
            s.next_ritual = None;
            let ids = s
                .people
                .iter_mut()
                .filter_map(|(id, p)| p.buffs.clear().then_some(*id))
                .collect::<Vec<_>>();
            Some((
                s.reserves.drain(..).map(|v| v.object).collect::<Vec<_>>(),
                s.scene.take(),
                ids,
            ))
        });
        if let Some((objects, scene, ids)) = old {
            for object in objects {
                let _ = unsafe {
                    r.events_remove_event_object(game, object)
                };
            }
            if let Some(scene) = scene {
                let _ = unsafe {
                    r.events_remove_ritual_scene(game, &scene)
                };
            }
            
            let _ = r.events_refresh_blessings(game, &ids);
        }
    }
}
fn tick(r: &dyn Services, game: u64) -> Result<(), String> {
    let Some(p) = party(r, game)? else {
        return Ok(());
    };
    let Some(host) = p.actors.iter().find(|a| a.slot == 0) else {
        return Ok(());
    };
    let now = p.clock;
    #[cfg(feature = "test-hooks")]
    if fixture_enabled() {
        fixture(r, game, &p)?;
    }
    let Some(p) = party(r, game)? else {
        return Ok(());
    };
    let ids = p
        .actors
        .iter()
        .filter(|a| a.phase != Phase::Inactive)
        .map(|a| (a.slot, a.guid))
        .collect::<Vec<_>>();
    let reset_needed = STATE
        .lock()
        .map_err(|_| "Event state unavailable")?
        .as_ref()
        .is_some_and(|s| s.game != game || s.owner != host.guid || now < s.clock);
    if reset_needed {
        reset();
    }
    let mut hides = vec![];
    let mut discoveries = vec![];
    let mut refresh = vec![];
    let mut removals = vec![];
    let mut rewards = vec![];
    let mut completed_scene = None;
    let (spawn_supply, spawn_ritual, stage) = {
        let mut guard = STATE.lock().map_err(|_| "Event state unavailable")?;
        let s = guard.get_or_insert_with(|| State {
            game,
            owner: host.guid,
            clock: now,
            rng: game
                ^ host.guid
                ^ std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            stage: 0,
            next_supply: now + 30.,
            next_ritual: None,
            sequence: 0,
            reserves: vec![],
            people: BTreeMap::new(),
            ritual: None,
            scene: None,
            stopped: false,
            fixture: 0,
            #[cfg(feature="test-hooks")]
            fixture_waited: false,
            poses: BTreeMap::new(),
        });
        s.clock = now;
        s.poses = p
            .actors
            .iter()
            .map(|a| (a.guid, (a.position, a.hp, a.phase)))
            .collect();
        s.people.retain(|id, _| ids.contains(id));
        for id in &ids {
            s.people.entry(*id).or_default();
        }
        for actor in &p.actors {
            let id = (actor.slot, actor.guid);
            let Some(person) = s.people.get_mut(&id) else {
                continue;
            };
            if person.buffs.expire(now) {
                refresh.push(id)
            }
            if actor.hp > 0. && actor.phase == Phase::Playing {
                for reserve in &mut s.reserves {
                    if let Some(d) = reserve.rules.claim(now, id, actor.position, true) {
                        hides.push((id, reserve.object));
                        if person.buffs.accept(now, d.reward) && !refresh.contains(&id) {
                            refresh.push(id)
                        }
                        discoveries.push((id, d));
                        break;
                    }
                }
            }
        }
        let mut i = 0;
        while i < s.reserves.len() {
            s.reserves[i].rules.retain(&ids);
            if ids.is_empty()
                || now >= s.reserves[i].rules.expires
                || s.reserves[i].rules.complete()
            {
                removals.push(s.reserves.remove(i).object)
            } else {
                i += 1
            }
        }
        let result = if let Some(v) = s.ritual.as_mut() {
            let all = !ids.is_empty()
                && p.actors
                    .iter()
                    .filter(|a| a.phase != Phase::Inactive)
                    .all(|a| inside(v.position, a.position, a.hp, a.phase));
            v.advance_presence(now, all)
        } else {
            Outcome::Pending
        };
        if result == Outcome::Success {
            let stage = s.ritual.take().ok_or("Missing completed ritual")?.encounter;
            completed_scene = s.scene.take();
            for id in &ids {
                let category = s.draw(5);
                let power = s.draw(100000);
                let person = s.people.get_mut(id).ok_or("Missing ritual recipient")?;
                let next = asx_gameplay::rituals::roll(stage, person.fortune, category, power);
                person.fortune = Some(next);
                rewards.push((*id, next.1));
            }
        }
        let supply = super::super::supplies::enabled()
            && p.actors
                .iter()
                .any(|a| a.hp > 0. && a.phase == Phase::Playing)
            && now >= s.next_supply;
        let supply = if supply {
            s.next_supply = now + 30. + s.draw(21) as f32;
            (s.draw(100) < 50 || fixture_enabled()) && s.reserves.len() < 8
        } else {
            false
        };
        let ritual = super::super::rituals::enabled()
            && p.actors
                .iter()
                .any(|a| a.hp > 0. && a.phase == Phase::Playing)
            && s.ritual.is_none()
            && s.next_ritual.is_some_and(|at| now >= at);
        if ritual {
            s.next_ritual = Some(now + 10.);
        }
        (supply, ritual, s.stage)
    };
    
    for (id, object) in hides {
        unsafe {
            r.events_hide_event_object(game, id.0, id.1, object)
        }?;
        r.events_info(&format!(
            "coop supplies claimed: event_guid={} slot={} owner={}",
            object.guid(),
            id.0,
            id.1
        ));
    }
    for object in removals {
        unsafe {
            r.events_remove_event_object(game, object)
        }?;
    }
    for (id, mut d) in discoveries {
        if d.gold > 0 {
            d.gold = r.events_group_sale(game,id,d.gold)?;
        }
        if let Ok(mut guard) = STATE.lock() {
            if let Some(person) = guard
                .as_mut()
                .filter(|s| s.game == game)
                .and_then(|s| s.people.get_mut(&id))
            {
                person.discovery = Some(d);
            }
        }
        r.events_info(&format!(
            "coop supplies discovered: slot={} owner={} event={} reward={:?} gold={}",
            id.0, id.1, d.event, d.reward, d.gold
        ));
    }
    if !refresh.is_empty() {
        r.events_refresh_blessings(game, &refresh)?;
        for id in refresh {
            if let Some(a) = p.actors.iter().find(|a| a.slot == id.0 && a.guid == id.1) {
                r.events_clamp_event_health(game, id.1, a.hp)?;
            }
        }
    }
    if let Some(scene) = completed_scene {
        unsafe {
            r.events_remove_ritual_scene(game, &scene)
        }?;
    }
    if !rewards.is_empty() {
        r.events_group_fortunes(game, &rewards)?;
        notice(
            r,
            game,
            "Sayge",
            "Sayge's trial is complete. Your fortune has changed!",
        )?;
        r.events_info(&format!(
            "coop ritual completed: stage={stage} recipients={}",
            rewards.len()
        ));
    }
    if spawn_supply || spawn_ritual {
        let draws = {
            let mut guard = STATE.lock().map_err(|_| "Event state unavailable")?;
            let s = guard.as_mut().ok_or("Event state reset")?;
            (0..32)
                .map(|_| (s.draw(u32::MAX), s.draw(u32::MAX)))
                .collect::<Vec<_>>()
        };
        let area = r.events_area(game)?;
        let point = if fixture_enabled() {
            unsafe {
                r.events_ground(game, if visual_fixture() && !spawn_ritual {
                        [host.position[0] - 8., host.position[1]]
                    } else {
                        [host.position[0] + 3., host.position[1]]
                    }, 0.3, 2.)
            }?
        } else {
            super::place(r, game, area, if spawn_ritual { 8.5 } else { 2. }, &draws)?
        };
        if let Some(point) = point {
            if spawn_ritual {
                let scene = unsafe {
                    r.events_ritual_scene(game, point)
                }?;
                let accepted = {
                    let mut guard = STATE.lock().map_err(|_| "Event state unavailable")?;
                    if let Some(s) = guard.as_mut().filter(|s| {
                        s.game == game
                            && s.owner == host.guid
                            && s.clock == now
                            && !s.stopped
                            && s.ritual.is_none()
                    }) {
                        s.ritual = Ritual::new(s.stage, now, point);
                        s.next_ritual = None;
                        s.scene = Some(scene);
                        true
                    } else {
                        false
                    }
                };
                if !accepted {
                    return Err("Ritual generation changed".into());
                }
                notice(
                    r,
                    game,
                    "Sayge",
                    "Sayge's trial awaits. Hold the circle to claim your fortune!",
                )?;
                r.events_info(&format!(
                    "coop ritual spawned: position={point:?} stage={stage}"
                ));
            } else {
                let (sequence, rolls) = {
                    let mut guard = STATE.lock().map_err(|_| "Event state unavailable")?;
                    let s = guard.as_mut().ok_or("Event state reset")?;
                    s.sequence += 1;
                    let rolls = ids
                        .iter()
                        .map(|id| {
                            let (a, b, c) = (s.draw(100), s.draw(100), s.draw(u32::MAX));
                            let reward = if fixture_enabled() {
                                if s.sequence == 1 {
                                    Reward::Buff(Consumable {
                                        family: if id.0 == 0 {
                                            Family::Critical
                                        } else {
                                            Family::Defense
                                        },
                                        tier: 2,
                                    })
                                } else {
                                    Reward::Material {
                                        rarity: 1,
                                        variant: 1,
                                    }
                                }
                            } else {
                                Reward::roll(stage, a, b, c)
                            };
                            (*id, reward)
                        })
                        .collect::<Vec<_>>();
                    (s.sequence, rolls)
                };
                let mut rules = Supply::new(
                    sequence,
                    now,
                    point,
                    &ids,
                    Reward::roll(stage, 0, 0, 0),
                    stage,
                )
                .ok_or("Invalid supply roster")?;
                if !rules.personal_rewards(&rolls, stage) {
                    return Err("Invalid personal supply rewards".into());
                }
                let object = unsafe {
                    r.events_object(game, 0, point)
                }?;
                {
                    let mut guard = STATE.lock().map_err(|_| "Event state unavailable")?;
                    let s = guard
                        .as_mut()
                        .filter(|s| {
                            s.game == game && s.owner == host.guid && s.clock == now && !s.stopped
                        })
                        .ok_or("Supply generation changed")?;
                    s.reserves.push(Reserve { rules, object });
                }
                notice(
                    r,
                    game,
                    "Lost Supplies",
                    "Lost supplies have been spotted in the area!",
                )?;
                r.events_info(&format!(
                    "coop supplies spawned: event={sequence} position={point:?} recipients={}",
                    ids.len()
                ));
            }
        }
    }
    Ok(())
}

fn fixture_enabled() -> bool {
    cfg!(feature = "test-hooks") && std::env::var("ASX_TEST_COOP_EVENTS").as_deref() == Ok("1")
}
#[cfg(feature = "test-hooks")]
fn fixture(r: &dyn Services, game: u64, p: &PartyObservation) -> Result<(), String> {
    let observed=super::super::scenario_steps::enabled();
    let targets=if observed{r.events_blessing_fixture_targets(game)?}else{vec![]};
    let action = {
        let mut guard = STATE.lock().map_err(|_| "Event fixture state")?;
        let Some(s) = guard.as_mut() else {
            return Ok(());
        };
        let due = match s.fixture {
            0 => 6.,
            1 => 20.,
            2 => 22.,
            3 => 70.,
            _ => 1e8,
        };
        let ready=if observed {
            p.actors.len()==2&&p.actors.iter().all(|a|a.phase==Phase::Playing)&&match s.fixture{
                0=>s.sequence==0,
                1=>p.actors.iter().all(|a|s.people.get(&(a.slot,a.guid)).is_some_and(|person|person.discovery.is_some_and(|d|d.event==1))),
                2=>!targets.is_empty(),
                3=>s.scene.is_none()&&s.ritual.is_none()&&p.actors.iter().all(|a|s.people.get(&(a.slot,a.guid)).is_some_and(|person|person.fortune.is_some())),
                _=>false,
            }
        }else{p.clock>=due};
        if ready {
            s.fixture += 1;
            Some(s.fixture)
        } else {
            None
        }
    };
    if let Some(action) = action {
        if observed{r.events_info(&format!("scenario step: coop action={action} clock={}",p.clock));}
        match action {
            1 | 4 => {
                if let Ok(mut guard) = STATE.lock() {
                    if let Some(s) = guard.as_mut() {
                        s.next_supply = p.clock;
                    }
                }
            }
            2 => {
                let def = r.events_blessing_fixture_definition(game, 0)?;
                unsafe { r.events_fixture_spawn_encounter(game, def) };
                r.events_info("coop events fixture: native boss spawned");
            }
            3 => {
                
                let guest = p
                    .actors
                    .iter()
                    .find(|a| a.slot == 1)
                    .ok_or("Missing guest kill fixture")?;
                for target in r.events_blessing_fixture_targets(game)? {
                    r.events_fixture_event_damage(game, guest.slot, guest.guid, target)?;
                }
                r.events_info("coop events fixture: native boss defeated by guest");
            }
            _ => {}
        }
    }
    let poses = STATE.lock().ok().and_then(|guard| {
        let s = guard.as_ref()?;
        if let Some(v) = &s.ritual {
            Some((v.position, if observed{!s.fixture_waited}else{p.clock-v.created<8.}))
        } else {
            s.reserves.first().map(|v| {
                (
                    v.rules.position,
                    if observed{v.rules.event==1&&!s.people.values().any(|person|person.discovery.is_some_and(|d|d.event==1))}else{p.clock - (v.rules.expires - 60.) < 8. && v.rules.event == 1},
                )
            })
        }
    });
    if let Some((point, separate)) = poses {
        
        for actor in &p.actors {
            if observed&&actor.phase!=Phase::Playing{continue}
            let mut pos = point;
            if (visual_fixture() && separate && p.clock < 11.) || (actor.slot != 0 && separate) {
                pos[0] += 12.;
            }
            r.events_fixture_event_pose(game, actor.slot, actor.guid, pos)?;
        }
    }
    // No random extra reserves in this focused scenario.
    if let Ok(mut guard) = STATE.lock() {
        if let Some(s) = guard.as_mut() {
            if action != Some(1) && action != Some(4) {
                s.next_supply = p.clock + 10000.;
            }
        }
    }
    static NEXT: Mutex<f32> = Mutex::new(0.);
    let sample = NEXT.lock().is_ok_and(|mut next| {
        if p.clock >= *next {
            *next = p.clock + 2.;
            true
        } else {
            false
        }
    });
    if sample {
        let mut state = STATE.lock().map_err(|_| "Event fixture state")?;
        if let Some(s) = state.as_ref() {
            r.events_info(&format!(
                "coop events fixture: clock={} reserves={} ritual={:?} actors={:?}",
                p.clock,
                s.reserves.len(),
                s.ritual.as_ref().map(|v| (v.id, v.held, v.inside())),
                p.actors
                    .iter()
                    .map(|a| (a.slot, a.guid, a.position, a.hp))
                    .collect::<Vec<_>>()
            ));
        }
        if observed{if let Some(s)=state.as_mut(){if s.ritual.as_ref().is_some_and(|v|v.held==0.&&!v.inside()){s.fixture_waited=true;}}}
    }
    Ok(())
}

fn visual_fixture() -> bool {
    cfg!(feature = "test-hooks")
        && std::env::var("ASX_TEST_COOP_EVENTS_VISUAL").as_deref() == Ok("1")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn downed_paused_missing_health_and_outside_members_cannot_supply_presence() {
        let center = [0.; 3];
        assert!(inside(center, [8., 0., 0.], 1., Phase::Playing));
        assert!(!inside(center, [8.01, 0., 0.], 1., Phase::Playing));
        for phase in [
            Phase::Downed,
            Phase::Choosing,
            Phase::Paused,
            Phase::Inactive,
        ] {
            assert!(!inside(center, center, 1., phase));
        }
        assert!(!inside(center, center, 0., Phase::Playing));
        assert!(!inside(center, [f32::NAN, 0., 0.], 1., Phase::Playing));
    }
}

#[cfg(feature="test-hooks")]
pub(in crate::runtime) fn observed_rewards_complete(game:u64)->bool{
 if !fixture_enabled(){return true}
 STATE.lock().ok().is_some_and(|guard|guard.as_ref().filter(|s|s.game==game).is_some_and(|s|s.sequence>=2&&s.reserves.is_empty()&&s.scene.is_none()&&s.ritual.is_none()&&s.people.len()==2&&s.people.values().all(|p|p.fortune.is_some()&&p.discovery.is_some_and(|d|d.event==2))))
}
