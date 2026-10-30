//! Source-owned support over the official native effect queue.
use super::*;
use asx_gameplay::support::{self as policy, Active, Member, Spell, UnitKind};
#[derive(Clone, Copy)]
struct Selection {
    game: u64,
    caster: u32,
    target: u32,
    spell: u32,
    suppressed: bool,
}
thread_local! {static CAST:std::cell::Cell<Option<Selection>>=const{std::cell::Cell::new(None)};}
use asx_game::official::{OwnedEffect, SupportReceipt, SupportTalents};
static ACTIVE: std::sync::Mutex<Vec<(u64, Active)>> = std::sync::Mutex::new(vec![]);
static TALENTS: std::sync::Mutex<Vec<(u64, u32, SupportTalents)>> = std::sync::Mutex::new(vec![]);
static RECEIPTS: std::sync::Mutex<Vec<SupportReceipt>> = std::sync::Mutex::new(vec![]);
static BREAKS: std::sync::Mutex<Vec<(OwnedEffect, [f32; 3], f32)>> = std::sync::Mutex::new(vec![]);
static QUERY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static BREAK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
thread_local! {static ORIGIN:std::cell::Cell<Option<[f32;3]>>=const{std::cell::Cell::new(None)};static QUERY_POSITION:std::cell::UnsafeCell<[f32;3]>=const{std::cell::UnsafeCell::new([0.0;3])};}
thread_local! {static PROC_RECIPIENT:std::cell::Cell<Option<(u64,u64)>>=const{std::cell::Cell::new(None)};}
static CAST_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static HURT_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RETALIATION: std::sync::Mutex<Vec<(asx_game::official::RetaliationTick, u64)>> =
    std::sync::Mutex::new(vec![]);
static HEAL_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PERIODIC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static METER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
thread_local! {static FOREIGN_HEAL:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
static OWNERS: std::sync::Mutex<Vec<asx_game::official::OwnedEffect>> =
    std::sync::Mutex::new(vec![]);
static APPLY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static CREDITS: std::sync::Mutex<Vec<(u32, u32, f32)>> = std::sync::Mutex::new(vec![]);
type Cast = unsafe extern "system" fn(u64, u64) -> bool;
pub(super) unsafe extern "system" fn cast(game: u64, ability: u64) -> bool {
    let original: Cast = std::mem::transmute(CAST_ORIGINAL.load(Ordering::Acquire));
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return original(game, ability);
    };
    #[cfg(feature = "test-hooks")]
    fixture::observe_support_cast(runtime, game, ability);
    let spell = runtime
        .adapter
        .support_spell(&runtime.local, runtime.local.base as u64, ability)
        .ok()
        .flatten();
    let before = spell.and_then(|_| {
        runtime
            .adapter
            .support_queue_count(&runtime.local, game)
            .ok()
    });
    let mut caster = spell.and_then(|_| runtime.adapter.support_caster(&runtime.local, game).ok());
    #[cfg(feature = "test-hooks")]
    if let Some(caster) = caster.as_mut() {
        if let Some(t) = FIXTURE_TALENTS.with(|v| v.get()) {
            caster.talents = t;
            caster.talents.maximum = caster.maximum;
        }
    }
    let selection = spell.zip(caster.as_ref()).and_then(|(spell, caster)| {
        let party = runtime.party.lock().ok().and_then(|p| p.clone())?;
        if party.actors.len() > asx_game::official::NATIVE_PLAYERS {
            return None;
        }
        let active = ACTIVE.lock().map(|a| a.clone()).unwrap_or_default();
        let current_active = runtime.adapter.support_active(&runtime.local, game).ok()?;
        let rejuvenation_slots=if spell==774{runtime.adapter.rejuvenation_source_slots(&runtime.local,game,caster.guid).ok()?}else{None};
        let mut members = party
            .actors
            .iter()
            .map(|a| {
                let (hp, maximum, position) = if a.slot == caster.slot {
                    (caster.hp, caster.maximum, caster.position)
                } else {
                    (a.hp, a.maximum, a.position)
                };
                Member {
                    id: a.slot + 1,
                    kind: UnitKind::Player,
                    hp: hp as f64,
                    max: maximum as f64,
                    position: position.map(f64::from),
                    phase: a.phase,
                    seen: party.clock as f64,
                    life: 0,
                    active: if a.slot == caster.slot {
                        current_active
                    } else {
                        active
                            .iter()
                            .find(|(g, _)| *g == a.guid)
                            .map(|(_, v)| *v)
                            .unwrap_or_default()
                    },
                }
            })
            .collect::<Vec<_>>();
        if let Some(slots)=rejuvenation_slots{for member in &mut members{member.active.rejuvenation=slots.contains(&(member.id-1));}}
        let enemies = runtime.adapter.support_enemies(&runtime.local, game).ok()?;
        let enemies = enemies
            .into_iter()
            .map(|e| policy::TargetEnemy {
                id: e.guid,
                alive: e.hp > 0.0,
                position: e.position,
            })
            .collect::<Vec<_>>();
        let pressure = policy::pressure(&members, &enemies);
        let target = if let Some((_, guid)) = PROC_RECIPIENT
            .with(|v| v.get())
            .filter(|(g, _)| *g == game && spell == 17)
        {
            let actor = party.actors.iter().find(|a| a.guid == guid)?;
            let member = members.iter().find(|m| m.id == actor.slot + 1)?;
            if !member.eligible(
                caster.position.map(f64::from),
                policy::RANGE,
                party.clock as f64,
                false,
            ) {
                return None;
            }
            actor.slot + 1
        } else if matches!(spell, 15237 | 25912 | 11426) {
            caster.slot + 1
        } else {
            policy::select(
                &members,
                &pressure,
                caster.slot + 1,
                match spell {
                    635 => Spell::HolyLight,
                    331 => Spell::HealingWave,
                    139 => Spell::Renew,
                    774 => Spell::Rejuvenation,
                    8936 => Spell::Regrowth,
                    17 => Spell::PowerWordShield,
                    324 => Spell::LightningShield,
                    467 => Spell::Thorns,
                    _ => return None,
                },
                party.clock as f64,
            ).or_else(|| {
                // Explicit diagnostic targets may bypass cached eligibility while
                // a coefficient probe temporarily clears a native support aura.
                #[cfg(feature="test-hooks")]
                { FIXTURE_TARGET.with(|v|v.get()).map(|slot|slot+1) }
                #[cfg(not(feature="test-hooks"))]
                { None }
            })?
        };
        #[cfg(feature = "test-hooks")]
        let target = FIXTURE_TARGET
            .with(|v| v.get())
            .map(|t| t + 1)
            .unwrap_or(target);
        Some(Selection {
            game,
            caster: caster.slot,
            target: target - 1,
            spell,
            suppressed: false,
        })
    });
    if let (Some(spell), Some(caster)) = (spell, caster.as_ref()) {
        if let Ok(mut talents) = TALENTS.lock() {
            talents.retain(|(guid, id, _)| *guid != caster.guid || *id != spell);
            if talents.len() < 64 {
                talents.push((caster.guid, spell, caster.talents));
            }
        }
    }
    if spell.is_some() && caster.is_some() && selection.is_none() {
        // No eligible unbuffed/wounded player. Keep the ability available instead
        // of refreshing the caster and broadcasting a native fallback effect.
        return false;
    }
    let saved = selection
        .filter(|s| s.target != s.caster && matches!(s.spell, 139 | 774 | 8936 | 17 | 324 | 467))
        .and_then(|s| {
            runtime
                .adapter
                .capture_aura(&runtime.local, game, s.spell)
                .ok()
        });
    // A missing rollback snapshot must retain native behavior instead of removing
    // a personal effect without being able to restore its previous state.
    let selection = selection.filter(|s| {
        !matches!(s.spell, 139 | 774 | 8936 | 17 | 324 | 467)
            || s.target == s.caster
            || saved.is_some()
    });
    let prepared = selection.is_some_and(|s| s.spell == 17 && s.target != s.caster);
    if prepared {
        let io = io::Access {
            local: &runtime.local,
            adapter: runtime.adapter,
        };
        if let Some(saved) = saved.as_ref() {
            if let Err(e) = runtime.adapter.prepare_foreign_shield(&io, game, saved) {
                let _ = runtime.adapter.restore_aura(&io, game, saved);
                runtime
                    .log
                    .warn(&format!("Official shield eligibility rejected: {e}"));
                return false;
            }
        }
    }
    let previous = CAST.with(|v| v.replace(selection));
    let success = original(game, ability);
    #[cfg(feature="test-hooks")]
    glyphs::fixture_cast(runtime,game,ability,success);
    let completed = CAST.with(|v| v.replace(previous));
    if let (Some(spell), Some(before), Some(caster)) = (spell, before, caster) {
        let target = completed
            .filter(|s| success && (s.target == s.caster || s.suppressed || saved.is_some()))
            .map(|s| s.target);
        let io = io::Access {
            local: &runtime.local,
            adapter: runtime.adapter,
        };
        if prepared {
            if let Some(saved) = saved.as_ref() {
                if let Err(e) = runtime
                    .adapter
                    .restore_aura(&io, game, saved)
                    .and_then(|_| asx_game::official::SupportCalls::refresh_support(&io, game))
                {
                    runtime
                        .log
                        .warn(&format!("Official source shield restore rejected: {e}"));
                    return success;
                }
            }
        }
        if target.is_some_and(|t| t != caster.slot) {
            if matches!(spell, 324 | 467) {
                match runtime
                    .adapter
                    .native_effect(&runtime.local, game, spell)
                    .and_then(|effect| {
                        asx_game::official::SupportCalls::emit_support_effect(
                            &io,
                            game,
                            spell,
                            &effect.data,
                            effect.duration,
                        )
                    }) {
                    Ok(()) => {}
                    Err(e) => {
                        runtime
                            .log
                            .warn(&format!("Official retaliation emit rejected: {e}"));
                        return success;
                    }
                }
            }
            if let Some(saved) = saved.as_ref().filter(|_| !prepared) {
                if let Err(error) = runtime.adapter.restore_aura(&io, game, saved) {
                    runtime
                        .log
                        .warn(&format!("Official support rollback rejected: {error}"));
                    return success;
                }
                if let Err(e) = asx_game::official::SupportCalls::refresh_support(&io, game) {
                    runtime
                        .log
                        .warn(&format!("Official source aura refresh rejected: {e}"));
                }
            }
        }
        if matches!(spell, 15237 | 25912) {
            // Keep every offensive target and the native self-heal. Add one
            // per-unit heal for each eligible allied player after a real heal.
            if (spell == 25912 || completed.is_some_and(|s| s.suppressed)) && success {
                if let Err(e) = runtime.adapter.emit_hybrid(
                    &io,
                    game,
                    runtime.local.base as u64,
                    spell,
                    before,
                    caster.maximum,
                ) {
                    runtime
                        .log
                        .warn(&format!("Official hybrid emit rejected: {e}"));
                }
            }
            return success;
        }
        if let Err(error) =
            runtime
                .adapter
                .route_support(&io, game, before, spell, target, caster.maximum)
        {
            runtime
                .log
                .warn(&format!("Official support route rejected: {error}"));
        }
        if let Some(target) = target {
            let guid = runtime.party.lock().ok().and_then(|p| {
                p.as_ref()
                    .and_then(|p| p.actors.iter().find(|a| a.slot == target).map(|a| a.guid))
            });
            if let Some(guid) = guid {
                if let Ok(mut active) = ACTIVE.lock() {
                    if let Some((_, a)) = active.iter_mut().find(|(g, _)| *g == guid) {
                        match spell {
                            139 => a.renew = true,
                            774 => a.rejuvenation = true,
                            17 => a.shield = true,
                            324 => a.lightning = true,
                            467 => a.thorns = true,
                            _ => {}
                        }
                    }
                }
            }
            runtime.log.info(&format!(
                "official support cast: caster={} target={target} spell={spell} success={success}",
                caster.slot
            ));
        }
    }
    success
}
crate::context_stub_rw!(heal_entry, 0, heal, HEAL_ORIGINAL);
unsafe extern "system" fn heal(c: &mut crate::stub::Context) {
    select_heal(c);
    if c.skip == 0 {
        if let Some(runtime) = RUNTIME.get().filter(|r| r.rules) {
            let amount = f32::from_bits(c.xmm[1] as u32);
            let spell=c.r8 as u32;
            let solo=runtime.solo.lock().ok().is_some_and(|p|p.is_some());
            let glyph=if spell==15237 || solo && spell==25912 {glyphs::healing_multiplier(runtime,c.rcx,spell)}else{1.};
            let factor=fatigue::healing_multiplier(runtime,c.rcx)*glyph;
            let scaled = amount * factor;
            #[cfg(feature="test-hooks")]
            if c.r8 as u32==15237&&glyph!=1.{glyphs::fixture_nova_heal(runtime,c.rcx,amount,scaled,factor);}
            #[cfg(feature="test-hooks")]
            fatigue::fixture_trace(runtime,c.rcx,1,amount,scaled);
            c.xmm[1] = (c.xmm[1] & !0xffff_ffffu128) | scaled.to_bits() as u128;
        }
    }
}
unsafe fn select_heal(c: &mut crate::stub::Context) {
    let Some(scope) = CAST.with(|v| v.get()).filter(|s| {
        s.game == c.rcx
            && s.spell == c.r8 as u32
            && (s.target != s.caster || matches!(s.spell, 15237 | 25912))
    }) else {
        return;
    };
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    if matches!(scope.spell, 15237 | 25912) {
        CAST.with(|v| {
            v.set(Some(Selection {
                suppressed: true,
                ..scope
            }))
        });
        return;
    }
    // The reviewed direct cast sites pass the non-owning native metadata sentinel.
    // Never skip a call that transfers an owned label/string to the native meter.
    if runtime
        .local
        .read(c.r9 as usize, 8)
        .is_ok_and(|v| v == (-2i64).to_le_bytes())
    {
        c.skip = 1;
        c.result = 0;
        CAST.with(|v| {
            v.set(Some(Selection {
                suppressed: true,
                ..scope
            }))
        });
    }
}
type Apply = unsafe extern "system" fn(u64, u64);
unsafe extern "system" fn apply(game: u64, descriptor: u64) {
    let original: Apply = std::mem::transmute(APPLY_ORIGINAL.load(Ordering::Acquire));
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return original(game, descriptor);
    };
    match runtime
        .adapter
        .support_delivery(&runtime.local, game, descriptor)
    {
        Ok(Some(mut delivery)) => {
            if !delivery.eligible {
                return;
            }
            let valid_source = runtime.party.lock().ok().is_some_and(|p| {
                p.as_ref().is_some_and(|p| {
                    p.actors
                        .iter()
                        .any(|a| a.slot == delivery.caster && a.guid == delivery.guid)
                })
            });
            if !valid_source {
                runtime
                    .log
                    .warn("Official support delivery rejected: source identity changed");
                return;
            }
            let Ok(before) = runtime.adapter.support_caster(&runtime.local, game) else {
                return;
            };
            if before.slot != delivery.target || before.hp <= 0.0 {
                return;
            }
            // The native consumer copies this POD descriptor; its original Vec is also
            // destroyed in the same world pass. No descriptor pointer escapes the call.
            #[cfg(feature="test-hooks")]
            let raw=delivery.amount;
            let glyph_factor=glyphs::healing_for_source(runtime,game,delivery.caster,delivery.guid,delivery.spell);
            let factor=fatigue::for_source(game,delivery.caster,delivery.guid)*glyph_factor;
            let Ok(scaled)=runtime.adapter.scale_direct_heal_delivery(&mut delivery,factor) else {return;};
            fatigue::with_source(game, delivery.caster, delivery.guid, || {
                #[cfg(feature="test-hooks")]
                if scaled {fatigue::fixture_trace(runtime,game,1,raw,delivery.amount);glyphs::fixture_heal(runtime,game,delivery.spell,raw,delivery.amount,factor,true);}
                fatigue::with_scaled_heal(game,scaled,||original(game, delivery.row.as_ptr() as u64));
            });
            let Ok(after) = runtime.adapter.support_caster(&runtime.local, game) else {
                return;
            };
            if delivery.duration > 0.0 || delivery.shield {
                match runtime.adapter.effect_owner(
                    &runtime.local,
                    game,
                    delivery.caster,
                    delivery.guid,
                    delivery.spell,
                    delivery.shield,
                ) {
                    Ok(owner) => {
                        if let Ok(mut owners) = OWNERS.lock() {
                            owners.retain(|o| {
                                o.target_guid != owner.target_guid || o.spell != owner.spell
                            });
                            if owners.len() < 64 {
                                owners.push(owner)
                            }
                        }
                    }
                    Err(error) => runtime
                        .log
                        .warn(&format!("Official support ownership rejected: {error}")),
                }
            }
            let effective = (after.hp - before.hp).max(0.0);
            if !delivery.shield && delivery.duration == 0.0 {
                receipt(
                    runtime,
                    game,
                    delivery.caster,
                    delivery.guid,
                    delivery.spell,
                    delivery.amount,
                    effective,
                    &after,
                );
            }
            if effective.is_finite() && effective > 0.0 {
                if let Ok(mut credits) = CREDITS.lock() {
                    if credits.len() < 128 {
                        credits.push((delivery.caster, delivery.spell, effective));
                    } else {
                        runtime.log.warn("Official support credit bound exceeded");
                    }
                }
            }
            runtime.log.info(&format!("official support delivered: caster={} target={} spell={} raw={} effective={effective}",delivery.caster,delivery.target,delivery.spell,delivery.amount));
        }
        Ok(None) => original(game, descriptor),
        Err(error) => runtime
            .log
            .warn(&format!("Official support delivery rejected: {error}")),
    }
}
pub fn finish(runtime: &Runtime, game: u64) {
    let pending = CREDITS
        .lock()
        .ok()
        .map(|mut c| std::mem::take(&mut *c))
        .unwrap_or_default();
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    let retaliation = RETALIATION
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    for (tick, enemy) in retaliation {
        match runtime.adapter.retaliate(&io,game,&tick,enemy){
            Ok((amount,effective))=>runtime.log.info(&format!("official retaliation hit: caster={} target_guid={} spell={} enemy={enemy} amount={amount} effective={effective}",tick.owner.caster,tick.owner.target_guid,tick.owner.spell)),
            Err(e)=>runtime.log.warn(&format!("Official retaliation damage rejected: {e}")),
        }
    }
    let breaks = BREAKS
        .lock()
        .map(|mut b| std::mem::take(&mut *b))
        .unwrap_or_default();
    for (owner, origin, amount) in breaks {
        match runtime
            .adapter
            .support_shield_break(&io, game, &owner, origin, amount)
        {
            Ok(()) => runtime.log.info(&format!(
                "official shield talent: caster={} target_guid={} amount={amount}",
                owner.caster, owner.target_guid
            )),
            Err(e) => runtime
                .log
                .warn(&format!("Official shield talent rejected: {e}")),
        }
    }
    let receipts = RECEIPTS
        .lock()
        .map(|mut b| std::mem::take(&mut *b))
        .unwrap_or_default();
    for receipt in receipts {
        match runtime.adapter.support_receipt(&io, game, &receipt) {
            Ok(None) => {
                if let Ok(mut pending) = RECEIPTS.lock() {
                    if pending.len() < 64 {
                        pending.push(receipt);
                    }
                }
            }
            Ok(Some(hits)) => {
                if receipt.talents.nature_coefficient > 0.0 {
                    runtime.log.info(&format!(
                        "official nature talent: caster={} target_guid={} raw={} hits={hits}",
                        receipt.caster, receipt.target_guid, receipt.raw
                    ));
                }
            }
            Err(e) => runtime
                .log
                .warn(&format!("Official support talent rejected: {e}")),
        }
    }
    if let Some(party) = runtime.party.lock().ok().and_then(|p| p.clone()) {
        if let Ok(mut owners) = OWNERS.lock() {
            owners.retain(|o| {
                o.expires > party.clock
                    && party.actors.iter().any(|a| a.guid == o.target_guid)
                    && party
                        .actors
                        .iter()
                        .any(|a| a.slot == o.caster && a.guid == o.caster_guid)
            });
        }
    }
    for (caster, spell, amount) in pending {
        match runtime.adapter.credit_support(&io,game,caster,spell,amount) {
   Ok((before,after))=> {
       #[cfg(feature="test-hooks")]
       if METER_TRACE.fetch_update(Ordering::AcqRel,Ordering::Acquire,|n|(n<64).then_some(n+1)).is_ok() {
           runtime.log.info(&format!("official support meter: caster={caster} spell={spell} amount={amount} before={before} after={after}"));
       }
       #[cfg(not(feature="test-hooks"))]
       let _=(before,after);
   },
   Err(error)=>runtime.log.warn(&format!("Official support meter rejected: {error}")),
  }
    }
}
pub fn hooks(adapter: Adapter) -> Vec<crate::install::HookSpec> {
    let mut hooks = vec![
 crate::install::HookSpec{name:"official-support-hit-effect",target:local_base()+adapter.native_hit_effect_entry().rva as usize,detour:effect_hit as *const ()as usize,trampoline:&EFFECT_HIT},
        crate::install::HookSpec {
            name: "official-support-retaliation",
            target: local_base() + adapter.hurt_entry().rva as usize,
            detour: hurt as *const () as usize,
            trampoline: &HURT_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-support-query-origin",
            target: local_base() + adapter.support_proc_entries()[2].rva as usize,
            detour: query_entry as *const () as usize,
            trampoline: &QUERY_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-support-shield-break",
            target: local_base() + adapter.support_proc_entries()[3].rva as usize,
            detour: break_entry as *const () as usize,
            trampoline: &BREAK_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-support-periodic",
            target: local_base() + adapter.periodic_entry().rva as usize,
            detour: periodic as *const () as usize,
            trampoline: &PERIODIC_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-support-meter",
            target: local_base() + adapter.support_entries()[3].rva as usize,
            detour: meter_entry as *const () as usize,
            trampoline: &METER_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-support-cast",
            target: local_base() + adapter.support_entries()[0].rva as usize,
            detour: cast as *const () as usize,
            trampoline: &CAST_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-support-self-heal",
            target: local_base() + adapter.support_entries()[1].rva as usize,
            detour: heal_entry as *const () as usize,
            trampoline: &HEAL_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-support-delivery",
            target: local_base() + adapter.support_entries()[2].rva as usize,
            detour: apply as *const () as usize,
            trampoline: &APPLY_ORIGINAL,
        },
    ];
    hooks
}

#[cfg(feature="test-hooks")]
static METER_TRACE:AtomicUsize=AtomicUsize::new(0);
pub fn reset() {
    #[cfg(feature="test-hooks")]
    METER_TRACE.store(0,Ordering::Release);
    if let Ok(mut r) = RETALIATION.lock() {
        r.clear();
    }
    if let Ok(mut a) = ACTIVE.lock() {
        a.clear();
    }
    if let Ok(mut v) = TALENTS.lock() {
        v.clear();
    }
    if let Ok(mut v) = RECEIPTS.lock() {
        v.clear();
    }
    if let Ok(mut v) = BREAKS.lock() {
        v.clear();
    }
    if let Ok(mut owners) = OWNERS.lock() {
        owners.clear();
    }
    if let Ok(mut credits) = CREDITS.lock() {
        credits.clear();
    }
}

crate::context_stub_rw!(meter_entry, 0, meter, METER_ORIGINAL);
unsafe extern "system" fn meter(c: &mut crate::stub::Context) {
    if FOREIGN_HEAL.with(|v| v.get()) {
        // Only our synthetic spell-0 call, with a borrowed sentinel, is suppressed.
        c.skip = 1;
        c.result = 0;
    }
}
type Periodic = unsafe extern "system" fn(u64, f32);
unsafe extern "system" fn periodic(game: u64, dt: f32) {
    let original: Periodic = std::mem::transmute(PERIODIC_ORIGINAL.load(Ordering::Acquire));
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return combat_text::with_periodic_text_source(game,None,||original(game, dt));
    };
    let owners = OWNERS.lock().map(|o| o.clone()).unwrap_or_default();
    let personal=fatigue::multiplier(runtime,game)<1.0;
    if owners.is_empty() && !personal {
        return combat_text::with_periodic_text_source(game,None,||original(game, dt));
    }
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    let ticks = match if personal {
        runtime.adapter.hide_fatigued_periodic(&io,game,dt,&owners)
    } else {
        runtime.adapter.hide_foreign_periodic(&io,game,dt,&owners)
    }
    {
        Ok(t) => t,
        Err(e) => {
            runtime
                .log
                .warn(&format!("Official periodic isolation rejected: {e}"));
            return combat_text::with_periodic_text_source(game,None,||original(game, dt));
        }
    };
    combat_text::with_periodic_text_source(game,None,||original(game, dt));
    for tick in ticks {
        match runtime.adapter.restore_periodic(&io, game, &tick) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(e) => {
                runtime
                    .log
                    .warn(&format!("Official periodic restore rejected: {e}"));
                continue;
            }
        }
        if tick.raw <= 0.0 {
            continue;
        }
        let valid = runtime.party.lock().ok().is_some_and(|p| {
            p.as_ref().is_some_and(|p| {
                p.actors
                    .iter()
                    .any(|a| a.slot == tick.owner.caster && a.guid == tick.owner.caster_guid)
            })
        });
        if !valid {
            continue;
        }
        let Ok(before) = runtime.adapter.support_caster(&runtime.local, game) else {
            continue;
        };
        type Heal = unsafe extern "system" fn(u64, f32, u32, *const u64);
        let heal: Heal = std::mem::transmute(HEAL_ORIGINAL.load(Ordering::Acquire));
        let metadata = [(-2i64) as u64, 0, 0, 0, 0, 0];
        let previous = FOREIGN_HEAL.with(|v| v.replace(true));
        let raw = tick.raw * fatigue::for_source(game, tick.owner.caster, tick.owner.caster_guid);
        #[cfg(feature="test-hooks")]
        fatigue::with_source(game,tick.owner.caster,tick.owner.caster_guid,||fatigue::fixture_trace(runtime,game,if tick.owner.caster_guid==tick.owner.target_guid {3}else{2},tick.raw,raw));
        combat_text::with_periodic_text_source(game,Some((tick.owner.caster_guid,tick.owner.spell)),||heal(game, raw, 0, metadata.as_ptr()));
        FOREIGN_HEAL.with(|v| v.set(previous));
        let Ok(after) = runtime.adapter.support_caster(&runtime.local, game) else {
            continue;
        };
        let effective = (after.hp - before.hp).max(0.0);
        receipt(
            runtime,
            game,
            tick.owner.caster,
            tick.owner.caster_guid,
            tick.owner.spell,
            raw,
            effective,
            &after,
        );
        if effective > 0.0 && effective.is_finite() {
            if let Ok(mut credits) = CREDITS.lock() {
                if credits.len() < 128 {
                    credits.push((tick.owner.caster, tick.owner.spell, effective));
                }
            }
        }
        // Bounded observation: no per-frame diagnostic spam in ordinary play.
        #[cfg(feature = "test-hooks")]
        if effective > 0.0 && tick.owner.caster_guid!=tick.owner.target_guid {
            runtime.log.info(&format!("official periodic delivered: caster={} target={} spell={} raw={} effective={effective}",tick.owner.caster,after.slot,tick.owner.spell,tick.raw));
        }
    }
}

fn receipt(
    runtime: &Runtime,
    game: u64,
    caster: u32,
    guid: u64,
    spell: u32,
    raw: f32,
    effective: f32,
    target: &asx_game::official::SupportCaster,
) {
    let talents = TALENTS
        .lock()
        .ok()
        .and_then(|t| {
            t.iter()
                .find(|(g, s, _)| *g == guid && *s == spell)
                .map(|(_, _, t)| *t)
        })
        .unwrap_or_default();
    let io = io::Access {
        local: &runtime.local,
        adapter: runtime.adapter,
    };
    if let Err(e) = runtime
        .adapter
        .support_overheal(&io, game, talents, raw, effective)
    {
        runtime
            .log
            .warn(&format!("Official support overheal rejected: {e}"));
    }
    if talents.nature_coefficient <= 0.0 {
        return;
    }
    if let Ok(mut pending) = RECEIPTS.lock() {
        if let Some(p) = pending
            .iter_mut()
            .find(|p| p.caster_guid == guid && p.target_guid == target.guid && p.spell == spell)
        {
            p.raw = (p.raw + raw).min(1e12);
            p.origin = target.position;
        } else if pending.len() < 64 {
            pending.push(SupportReceipt {
                caster,
                caster_guid: guid,
                target_guid: target.guid,
                spell,
                raw,
                effective,
                origin: target.position,
                talents,
            });
        }
    }
}
pub(super) fn with_origin<T>(origin: [f32; 3], f: impl FnOnce() -> T) -> T {
    let previous = ORIGIN.with(|o| o.replace(Some(origin)));
    let result = f();
    ORIGIN.with(|o| o.set(previous));
    result
}
crate::context_stub_rw!(query_entry, 0, query_origin, QUERY_ORIGINAL);
unsafe extern "system" fn query_origin(c: &mut crate::stub::Context) {
    // The query reads synchronously. The owned TLS copy lives until native proc returns.
    if let Some(origin) = ORIGIN.with(|o| o.get()) {
        if let Some(r) = RUNTIME.get() {
            let io = io::Access {
                local: &r.local,
                adapter: r.adapter,
            };
            match r.adapter.support_query_origin(
                &io,
                c.r8,
                c.rsp_entry,
                c.return_address().wrapping_sub(r.local.base as u64),
                origin,
            ) {
                Ok(true) => r
                    .log
                    .info(&format!("official shield proc origin: {origin:?}")),
                Err(e) => r.log.warn(&format!("Official proc origin rejected: {e}")),
                _ => {}
            }
        }
        QUERY_POSITION.with(|p| {
            *p.get() = origin;
            c.r8 = p.get() as u64;
        });
    }
}
crate::context_stub_rw!(break_entry, 0, shield_break, BREAK_ORIGINAL);
unsafe extern "system" fn shield_break(c: &mut crate::stub::Context) {
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let registers = [
        c.rax, c.rcx, c.rdx, c.r8, c.r9, c.r10, c.r11, c.rbx, c.rbp, c.rsi, c.rdi, c.r12, c.r13,
        c.r14, c.r15,
    ];
    let Ok(frame) = runtime.adapter.support_break_frame(
        registers,
        [
            f32::from_bits(c.xmm[0] as u32),
            f32::from_bits(c.xmm[1] as u32),
        ],
    ) else {
        return;
    };
    let Ok(actor) = runtime.adapter.support_caster(&runtime.local, frame.game) else {
        return;
    };
    let owner = OWNERS.lock().ok().and_then(|mut owners| {
        let i = owners.iter().position(|o| {
            o.shield
                && o.target_guid == actor.guid
                && o.spell == frame.spell
                && o.caster != actor.slot
        })?;
        Some(owners.remove(i))
    });
    let Some(owner) = owner else { return };
    if let Ok(mut pending) = BREAKS.lock() {
        if pending.len() < 64 {
            pending.push((owner, actor.position, frame.amount));
        }
    }
    // Continue the native hurt function: zero only its outgoing proc amount.
    c.xmm[0] &= !(u32::MAX as u128);
    c.xmm[1] &= !(u32::MAX as u128);
}

#[cfg(feature = "test-hooks")]
thread_local! {static FIXTURE_TALENTS:std::cell::Cell<Option<SupportTalents>>=const{std::cell::Cell::new(None)};}
#[cfg(feature = "test-hooks")]
pub(super) fn fixture_talents<T>(enabled: bool, f: impl FnOnce() -> T) -> T {
    let talents = enabled.then_some(SupportTalents {
        overheal_cap: 0.1,
        nature_coefficient: 0.5,
        nature_radius: 10.0,
        maximum: 0.0,
    });
    let previous = FIXTURE_TALENTS.with(|v| v.replace(talents));
    let result = f();
    FIXTURE_TALENTS.with(|v| v.set(previous));
    result
}

pub fn capture(runtime: &Runtime, game: u64) {
    let Some(party) = runtime.party.lock().ok().and_then(|p| p.clone()) else {
        return;
    };
    let slots = [139, 774, 17, 324, 467].map(|spell| {
        runtime
            .adapter
            .support_buff_slots(&runtime.local, game, spell)
    });
    if slots.iter().any(|s| s.is_err()) {
        return;
    }
    let slots = slots.map(|s| s.unwrap());
    let values = party
        .actors
        .iter()
        .map(|a| {
            (
                a.guid,
                Active {
                    renew: slots[0].contains(&a.slot),
                    rejuvenation: slots[1].contains(&a.slot),
                    shield: slots[2].contains(&a.slot),
                    lightning: slots[3].contains(&a.slot),
                    thorns: slots[4].contains(&a.slot),
                    ..Active::default()
                },
            )
        })
        .collect();
    if let Ok(mut a) = ACTIVE.lock() {
        *a = values;
    }
}
#[cfg(feature = "test-hooks")]
thread_local! {static FIXTURE_TARGET:std::cell::Cell<Option<u32>>=const{std::cell::Cell::new(None)};}
#[cfg(feature = "test-hooks")]
pub(super) fn fixture_target<T>(target: Option<u32>, f: impl FnOnce() -> T) -> T {
    let previous = FIXTURE_TARGET.with(|v| v.replace(target));
    let result = f();
    FIXTURE_TARGET.with(|v| v.set(previous));
    result
}

#[cfg(feature = "test-hooks")]
pub(super) fn observe_combat_damage(game: u64, enemy: u64, amount: f32, spell: u32) {
    let Some(origin) = ORIGIN.with(|o| o.get()) else {
        return;
    };
    let Some(r) = RUNTIME.get() else { return };
    if let Ok(source) = r.adapter.support_caster(&r.local, game) {
        r.log.info(&format!("official shield talent hit: caster={} enemy={enemy} amount={amount} spell={spell} origin={origin:?}",source.slot));
    }
}

type Hurt = unsafe extern "system" fn(u64, f32, u64, bool);
unsafe extern "system" fn hurt(game: u64, amount: f32, enemy: u64, melee: bool) {
    #[cfg(feature="test-hooks")]
    if std::env::var("AZCOOP_TEST_BLESSINGS").as_deref()==Ok("1"){return;}
    let original: Hurt = std::mem::transmute(HURT_ORIGINAL.load(Ordering::Acquire));
    let original=|game,amount,enemy,melee|{original(game,amount,enemy,melee);if let Some(r)=RUNTIME.get().filter(|r|r.rules){combat::observe_immunity(r,game);}};
    if let Some(r) = RUNTIME.get().filter(|r| r.rules) {
        // No native coop context exists in a solo run. Keep its original hit
        // path; foreign aura isolation only applies to a captured coop party.
        let coop=r.party.lock().map(|party|party.is_some()).unwrap_or(false);
        if !coop {return original(game,amount,enemy,melee);}
        if combat::redirect_melee(r, game, amount, enemy, melee) {
            return;
        }
    }
    let Some(r) = RUNTIME.get().filter(|r| r.rules && melee && enemy != 0) else {
        return original(game, amount, enemy, melee);
    };
    let owners = OWNERS.lock().map(|o| o.clone()).unwrap_or_default();
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    let ticks = match r.adapter.hide_retaliation(&io, game, &owners) {
        Ok(t) => t,
        Err(e) => {
            r.log
                .warn(&format!("Official retaliation isolation rejected: {e}"));
            return original(game, amount, enemy, melee);
        }
    };
    original(game, amount, enemy, melee);
    for tick in ticks {
        match r.adapter.restore_retaliation(&io, game, &tick) {
            Ok(true) => {
                if let Ok(mut q) = RETALIATION.lock() {
                    if q.len() < 128 {
                        q.push((tick, enemy));
                    }
                }
            }
            Ok(false) => {}
            Err(e) => r
                .log
                .warn(&format!("Official retaliation restore rejected: {e}")),
        }
    }
}

pub(super) fn with_recipient<T>(game: u64, guid: u64, f: impl FnOnce() -> T) -> T {
    let old = PROC_RECIPIENT.with(|v| v.replace(Some((game, guid))));
    let result = f();
    PROC_RECIPIENT.with(|v| v.set(old));
    result
}

/// Native hybrid self-heal payload; independent of optional glyph interception.
pub(super) unsafe fn scale_native_hit_heal(original:unsafe extern "system" fn(u64,u64,u64,f32,u32),game:u64,enemy:u64,effect:u64,damage:f32,spell:u32)->bool{
 if matches!(spell,25912|15237) {
  if let Some(r)=RUNTIME.get().filter(|r|r.rules){
   let factor=glyphs::healing_multiplier(r,game,spell)*fatigue::healing_multiplier(r,game);
   if factor!=1. {if let Ok(Some((bytes,_before,_after)))=r.adapter.scaled_heal_effect(&r.local,effect,factor){
    #[cfg(feature="test-hooks")]
    {glyphs::fixture_heal(r,game,spell,_before,_after,factor,false);}
    #[cfg(feature="test-hooks")]
    let observed=if glyphs::fixture_enabled(){static SAMPLES:AtomicUsize=AtomicUsize::new(0);if SAMPLES.fetch_add(1,Ordering::Relaxed)<32{r.adapter.support_caster(&r.local,game).ok()}else{None}}else{None};
    original(game,enemy,bytes.as_ptr() as u64,damage,spell);
    #[cfg(feature="test-hooks")]
    if let Some(before)=observed{if let Ok(after)=r.adapter.support_caster(&r.local,game){r.log.info(&format!("glyph fixture self heal applied: spell={spell} before_hp={} after_hp={} maximum={} fraction={_after}",before.hp,after.hp,before.maximum));}}
    return true;
   }}
  }
 }
 false
}

static EFFECT_HIT:AtomicUsize=AtomicUsize::new(0);
unsafe extern "system" fn effect_hit(game:u64,enemy:u64,effect:u64,damage:f32,spell:u32){
 let original:unsafe extern "system" fn(u64,u64,u64,f32,u32)=std::mem::transmute(EFFECT_HIT.load(Ordering::Acquire));
 if glyphs::intercept_hit(original,game,enemy,effect,damage,spell){return}
 if scale_native_hit_heal(original,game,enemy,effect,damage,spell){return}
 original(game,enemy,effect,damage,spell)
}

#[cfg(test)]mod effect_abi_tests {
 use super::*;
 static ARGS:std::sync::Mutex<Option<(u64,u64,u64,f32,u32)>>=std::sync::Mutex::new(None);
 unsafe extern "system" fn native(g:u64,e:u64,p:u64,d:f32,s:u32){*ARGS.lock().unwrap()=Some((g,e,p,d,s));}
 #[test]fn borrowed_effect_wrapper_preserves_five_arguments_and_xmm3_damage(){
 EFFECT_HIT.store(native as *const () as usize,Ordering::SeqCst);let a=0x1122334455667788;let b=0x8877665544332211;
 for spell in [25912,635,845]{unsafe{effect_hit(a,b,0x123456789abcdef0,1.25,spell)};assert_eq!(*ARGS.lock().unwrap(),Some((a,b,0x123456789abcdef0,1.25,spell)));}
 }
}
