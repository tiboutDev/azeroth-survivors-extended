//! Source-owned support over the official native effect queue.
use super::*;
use azsx_gameplay::support::{self as policy, Active, Member, Spell, UnitKind};
#[derive(Clone, Copy)]
struct Selection {
    game: u64,
    caster: u32,
    target: u32,
    spell: u32,
    suppressed: bool,
}
thread_local! {static CAST:std::cell::Cell<Option<Selection>>=const{std::cell::Cell::new(None)};}
use azsx_game::official::{OwnedEffect, SupportReceipt, SupportTalents};
static ACTIVE: std::sync::Mutex<Vec<(u64, Active)>> = std::sync::Mutex::new(vec![]);
static TALENTS: std::sync::Mutex<Vec<(u64, u32, SupportTalents)>> = std::sync::Mutex::new(vec![]);
static RECEIPTS: std::sync::Mutex<Vec<SupportReceipt>> = std::sync::Mutex::new(vec![]);
static BREAKS: std::sync::Mutex<Vec<(OwnedEffect, [f32; 3], f32)>> = std::sync::Mutex::new(vec![]);
static CONDITION_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static QUERY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static BREAK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
thread_local! {static ORIGIN:std::cell::Cell<Option<[f32;3]>>=const{std::cell::Cell::new(None)};static QUERY_POSITION:std::cell::UnsafeCell<[f32;3]>=const{std::cell::UnsafeCell::new([0.0;3])};}
thread_local! {static PROC_RECIPIENT:std::cell::Cell<Option<(u64,u64)>>=const{std::cell::Cell::new(None)};}
static CAST_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static HURT_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RETALIATION: std::sync::Mutex<Vec<(azsx_game::official::RetaliationTick, u64)>> =
    std::sync::Mutex::new(vec![]);
static HEAL_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PERIODIC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static METER_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
thread_local! {static FOREIGN_HEAL:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
static OWNERS: std::sync::Mutex<Vec<azsx_game::official::OwnedEffect>> =
    std::sync::Mutex::new(vec![]);
static APPLY_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static CREDITS: std::sync::Mutex<Vec<(u32, u32, f32)>> = std::sync::Mutex::new(vec![]);
type Cast = unsafe extern "system" fn(u64, u64) -> bool;
pub(super) unsafe extern "system" fn cast(game: u64, ability: u64) -> bool {
    NEXT_NECKLACE_CAST.with(|v| v.set(None));
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
        if party.actors.len() > azsx_game::official::NATIVE_PLAYERS {
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
        } else if spell == 11426 {
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
                    .and_then(|_| azsx_game::official::SupportCalls::refresh_support(&io, game))
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
                        azsx_game::official::SupportCalls::emit_support_effect(
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
                if let Err(e) = azsx_game::official::SupportCalls::refresh_support(&io, game) {
                    runtime
                        .log
                        .warn(&format!("Official source aura refresh rejected: {e}"));
                }
            }
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
        #[cfg(feature="test-hooks")]
        if std::env::var("AZSX_TEST_SPELL_BLOCK").as_deref()==Ok("1") && matches!(spell,324|467) {
            runtime.log.info(&format!("spell block retaliation queue: spell={spell} caster={} target={target:?} before={before} after={:?}",caster.slot,runtime.adapter.support_queue_count(&runtime.local,game)));
        }
        if let Some(target) = target {
            let guid = runtime.party.lock().ok().and_then(|p| {
                p.as_ref()
                    .and_then(|p| p.actors.iter().find(|a| a.slot == target).map(|a| a.guid))
            });
            if let Some(guid) = guid {
                if target != caster.slot && matches!(spell, 635|8936|139|774) {
                    if let Ok(clock)=runtime.adapter.set_clock(&runtime.local,game) {
                        NEXT_NECKLACE_CAST.with(|v| v.set(Some(NecklaceCast {game,spell,caster:(caster.slot,caster.guid),target:(target,guid),clock})));
                    }
                }
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
            let factor=fatigue::healing_multiplier(runtime,c.rcx);
            let scaled = amount * factor;
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
            && s.target != s.caster
    }) else {
        return;
    };
    let Some(runtime) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
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
            let shield_before = match runtime.adapter.support_shield_pool(&runtime.local, game, delivery.spell) {
                Ok(value) => value,
                Err(error) => {
                    runtime.log.warn(&format!("Official shield observation rejected: {error}"));
                    return;
                }
            };
            // The native consumer copies this POD descriptor; its original Vec is also
            // destroyed in the same world pass. No descriptor pointer escapes the call.
            #[cfg(feature="test-hooks")]
            let raw=delivery.amount;
            let factor=fatigue::for_source(game,delivery.caster,delivery.guid);
            let Ok(scaled)=runtime.adapter.scale_direct_heal_delivery(&mut delivery,factor) else {return;};
            fatigue::with_source(game, delivery.caster, delivery.guid, || {
                #[cfg(feature="test-hooks")]
                if scaled {fatigue::fixture_trace(runtime,game,1,raw,delivery.amount);}
                fatigue::with_scaled_heal(game,scaled,||original(game, delivery.row.as_ptr() as u64));
            });
            let Ok(after) = runtime.adapter.support_caster(&runtime.local, game) else {
                return;
            };
            // A reentrant callback may have rebound the native actor. The
            // completed native commit is never retried against another identity.
            if (after.slot, after.guid) != (before.slot, before.guid) { return; }
            let shield_after = match runtime.adapter.support_shield_pool(&runtime.local, game, delivery.spell) {
                Ok(value) => value,
                Err(error) => {
                    runtime.log.warn(&format!("Official completed shield observation rejected: {error}"));
                    return;
                }
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
                            if let Err(error) = owner.record(&mut owners, shield_before.zip(shield_after)) {
                                runtime.log.warn(&format!("Official shield ownership commit rejected: {error}"));
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
    let hurt_detour = match adapter.hurt_source_layout() {
        0 => hurt as *const () as usize,
        1 => hurt_borrowed as *const () as usize,
        _ => panic!("Unreviewed native hurt hook ABI"),
    };
    let mut hooks = vec![
        crate::install::HookSpec{name:"official-necklace-leech",target:local_base()+adapter.necklace_leech_entry().rva as usize,detour:necklace_leech as *const () as usize,trampoline:&LEECH_ORIGINAL},
        crate::install::HookSpec{name:"official-support-talent-condition",target:local_base()+adapter.talent_condition_entry().rva as usize,detour:talent_condition as *const () as usize,trampoline:&CONDITION_ORIGINAL},
 crate::install::HookSpec{name:"official-support-hit-effect",target:local_base()+adapter.native_hit_effect_entry().rva as usize,detour:effect_hit as *const ()as usize,trampoline:&EFFECT_HIT},
        crate::install::HookSpec {
            name: "official-support-retaliation",
            target: local_base() + adapter.hurt_entry().rva as usize,
            detour: hurt_detour,
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
    NEXT_NECKLACE_CAST.with(|v|v.set(None));
    if let Ok(mut windows) = LEECH_WINDOWS.lock() { windows.clear(); }
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
        let heal: azsx_game::official::NativeHealCall = std::mem::transmute(HEAL_ORIGINAL.load(Ordering::Acquire));
        let metadata = [(-2i64) as u64, 0, 0, 0, 0, 0];
        let previous = FOREIGN_HEAL.with(|v| v.replace(true));
        let raw = tick.raw * fatigue::for_source(game, tick.owner.caster, tick.owner.caster_guid);
        #[cfg(feature="test-hooks")]
        fatigue::with_source(game,tick.owner.caster,tick.owner.caster_guid,||fatigue::fixture_trace(runtime,game,if tick.owner.caster_guid==tick.owner.target_guid {3}else{2},tick.raw,raw));
        combat_text::with_periodic_text_source(game,Some((tick.owner.caster_guid,tick.owner.spell)),||azsx_game::official::call_native_heal(heal, game, raw, 0, metadata.as_ptr()));
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
    target: &azsx_game::official::SupportCaster,
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
    let absorbed = match runtime.adapter.support_overheal(&io, game, talents, raw, effective) {
        Ok(amount) => amount,
        Err(e) => {
            runtime.log.warn(&format!("Official support overheal rejected: {e}"));
            0.0
        }
    };
    if talents.nature_coefficient <= 0.0 {
        return;
    }
    if let Ok(mut pending) = RECEIPTS.lock() {
        if let Some(p) = pending
            .iter_mut()
            .find(|p| p.caster_guid == guid && p.target_guid == target.guid && p.spell == spell)
        {
            p.raw = (p.raw + raw).min(1e12);
            p.effective = (p.effective + effective).min(1e12);
            p.absorbed = (p.absorbed + absorbed).min(1e12);
            p.origin = target.position;
        } else if pending.len() < 64 {
            pending.push(SupportReceipt {
                caster,
                caster_guid: guid,
                target_guid: target.guid,
                spell,
                raw,
                effective,
                absorbed,
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
    if let Some(r)=RUNTIME.get().filter(|r|r.rules){
        let radius=f32::from_bits(c.xmm[3] as u32);
        match r.adapter.talent_query_radius(&r.local,c.rdx,c.return_address().wrapping_sub(r.local.base as u64),radius){
            Ok(adjusted)=>{
                #[cfg(feature="test-hooks")]
                if ORIGIN.with(|v|v.get()).is_some() && std::env::var("AZSX_TEST_SET_BLOCK").as_deref()==Ok("1"){
                    r.log.info(&format!("set shield radius: base={radius} effective={adjusted}"));
                }
                c.xmm[3]=(c.xmm[3]&!(u32::MAX as u128))|u128::from(adjusted.to_bits());
            },
            Err(e)=>r.log.warn(&format!("Official talent radius rejected: {e}")),
        }
    }
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
    // No native calls occur while these bounded queues are locked. Retire the
    // owner only after accepting the break; overflow must retain attribution.
    let Ok(mut pending)=BREAKS.lock()else{return};
    if pending.len()>=64{return}
    let Ok(mut owners)=OWNERS.lock()else{return};
    let owner = owners.iter().position(|o|o.shield && o.target_guid==actor.guid && o.spell==frame.spell && o.caster!=actor.slot)
        .map(|i|owners.remove(i))
        .unwrap_or(OwnedEffect{caster:actor.slot,caster_guid:actor.guid,target_guid:actor.guid,spell:frame.spell,expires:0.,shield:true});
    pending.push((owner,actor.position,frame.amount));
    drop(owners);
    drop(pending);
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

type Hurt = azsx_game::official::NativeHurtCall;
unsafe extern "system" fn hurt(game: u64, amount: f32, enemy: u64, melee: bool, kind: u32, spell: u32) {
    let native: Hurt = std::mem::transmute(HURT_ORIGINAL.load(Ordering::Acquire));
    dispatch_hurt(game, amount, enemy, melee, u64::from(kind), spell,
                  |game, amount, enemy, melee| native(game, amount, enemy, melee, kind, spell));
}
unsafe extern "system" fn hurt_borrowed(game: u64, amount: f32, enemy: u64, melee: bool, source: *const u8) {
    let native: azsx_game::official::NativeHurtBorrowedCall =
        std::mem::transmute(HURT_ORIGINAL.load(Ordering::Acquire));
    dispatch_hurt(game, amount, enemy, melee, source as u64, 0,
                  |game, amount, enemy, melee| native(game, amount, enemy, melee, source));
}
unsafe fn dispatch_hurt(game: u64, amount: f32, enemy: u64, melee: bool, source_word: u64,
                        source_spell: u32, mut forward: impl FnMut(u64, f32, u64, bool)) {
    #[cfg(feature="test-hooks")]
    if std::env::var("AZCOOP_TEST_BLESSINGS").as_deref()==Ok("1"){return;}
    let mut original=|game,amount,enemy,melee|{forward(game,amount,enemy,melee);if let Some(r)=RUNTIME.get().filter(|r|r.rules){combat::observe_immunity(r,game);}};
    if let Some(r) = RUNTIME.get().filter(|r| r.rules) {
        // No native coop context exists in a solo run. Keep its original hit
        // path; foreign aura isolation only applies to a captured coop party.
        let coop=r.party.lock().map(|party|party.is_some()).unwrap_or(false);
        if !coop {return original(game,amount,enemy,melee);}
        if melee && enemy != 0 {
            if let Ok(source) = r.adapter.capture_hurt_source(&r.local, source_word, source_spell) {
                if combat::redirect_melee(r, game, amount, enemy, melee, source) { return; }
            }
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

/// Native hybrid self-heal payload with resurrection fatigue.
pub(super) unsafe fn scale_native_hit_heal(original:unsafe extern "system" fn(u64,u64,u64,f32,u32),game:u64,enemy:u64,effect:u64,damage:f32,spell:u32)->bool{
 if matches!(spell,25912|15237) {
  if let Some(r)=RUNTIME.get().filter(|r|r.rules){
   let factor=fatigue::healing_multiplier(r,game);
   if factor!=1. {if let Ok(Some((bytes,_before,_after)))=r.adapter.scaled_heal_effect(&r.local,effect,factor){
    original(game,enemy,bytes.as_ptr() as u64,damage,spell);
    return true;
   }}
  }
 }
 false
}

static EFFECT_HIT:AtomicUsize=AtomicUsize::new(0);
unsafe extern "system" fn effect_hit(game:u64,enemy:u64,effect:u64,damage:f32,spell:u32){
 let original:unsafe extern "system" fn(u64,u64,u64,f32,u32)=std::mem::transmute(EFFECT_HIT.load(Ordering::Acquire));
 if scale_native_hit_heal(original,game,enemy,effect,damage,spell){return}
 original(game,enemy,effect,damage,spell);
 if spell==13237 { if let Some(r)=RUNTIME.get().filter(|r|r.rules){super::solo_gadgets::impact(r,game,enemy,effect,spell);} }
}

#[cfg(test)]mod effect_abi_tests {
 use super::*;
 static ARGS:std::sync::Mutex<Option<(u64,u64,u64,f32,u32)>>=std::sync::Mutex::new(None);
 unsafe extern "system" fn native(g:u64,e:u64,p:u64,d:f32,s:u32){*ARGS.lock().unwrap()=Some((g,e,p,d,s));}
 #[test]fn borrowed_effect_wrapper_preserves_five_arguments_and_xmm3_damage(){
 EFFECT_HIT.store(native as *const () as usize,Ordering::SeqCst);let a=0x1122334455667788;let b=0x8877665544332211;
 for spell in [15237,25912,635,845]{unsafe{effect_hit(a,b,0x123456789abcdef0,1.25,spell)};assert_eq!(*ARGS.lock().unwrap(),Some((a,b,0x123456789abcdef0,1.25,spell)));}
 }
}

#[cfg(test)]mod hurt_forwarding_abi_tests{
 use super::*;
 static ARGS:std::sync::Mutex<Option<(u64,f32,u64,bool,u32,u32)>>=std::sync::Mutex::new(None);
 unsafe extern "system" fn native(game:u64,amount:f32,enemy:u64,melee:bool,kind:u32,spell:u32){*ARGS.lock().unwrap()=Some((game,amount,enemy,melee,kind,spell));}
 #[test]fn inactive_hurt_forwarding_preserves_both_native_source_stack_words(){
  let previous=HURT_ORIGINAL.swap(native as *const ()as usize,Ordering::SeqCst);
  struct Restore(usize);impl Drop for Restore{fn drop(&mut self){HURT_ORIGINAL.store(self.0,Ordering::SeqCst);}}let _restore=Restore(previous);
  for melee in [false,true]{let args=(0x123456789abcdef0,1.25,0xf130000012345678,melee,0x11223344,0x55667788);unsafe{hurt(args.0,args.1,args.2,args.3,args.4,args.5)}assert_eq!(*ARGS.lock().unwrap(),Some(args));}
 }
}


#[cfg(test)]
mod borrowed_hurt_forwarding_tests {
    use super::*;
    static ARGS: std::sync::Mutex<Option<(u64, f32, u64, bool, u64)>> = std::sync::Mutex::new(None);
    unsafe extern "system" fn native(game: u64, amount: f32, enemy: u64, melee: bool, source: *const u8) {
        *ARGS.lock().unwrap() = Some((game, amount, enemy, melee, source as u64));
    }
    #[test]
    fn inactive_forwarding_retains_the_full_fifth_argument_pointer() {
        let previous = HURT_ORIGINAL.swap(native as *const () as usize, Ordering::SeqCst);
        struct Restore(usize);
        impl Drop for Restore { fn drop(&mut self) { HURT_ORIGINAL.store(self.0, Ordering::SeqCst); } }
        let _restore = Restore(previous);
        for melee in [false, true] {
            let args = (0x123456789abcdef0, 1.25, 0xf130000012345678, melee, 0x1234567887654321);
            unsafe { hurt_borrowed(args.0, args.1, args.2, args.3, args.4 as *const u8); }
            assert_eq!(*ARGS.lock().unwrap(), Some(args));
        }
    }
}

thread_local! {static SHIELD_CONTEXT:std::cell::Cell<Option<(u64,u32,u64)>>=const{std::cell::Cell::new(None)};}
pub(super) fn with_shield<T>(game:u64,spell:u32,recipient:u64,f:impl FnOnce()->T)->T{
    let previous=SHIELD_CONTEXT.with(|v|v.replace(Some((game,spell,recipient))));
    let result=f();SHIELD_CONTEXT.with(|v|v.set(previous));result
}
#[derive(Clone, Copy)]
struct NecklaceCast {game:u64,spell:u32,caster:(u32,u64),target:(u32,u64),clock:f32}
thread_local! {
    static NEXT_NECKLACE_CAST:std::cell::Cell<Option<NecklaceCast>>=const{std::cell::Cell::new(None)};
    static NECKLACE_EVENT:std::cell::Cell<Option<NecklaceCast>>=const{std::cell::Cell::new(None)};
}
// The native ability driver emits event 3 immediately after a successful cast.
pub(in crate::runtime) fn with_necklace_event(game:u64,event:u64,run:impl FnOnce()){
    let cast=NEXT_NECKLACE_CAST.with(|v|v.get()).filter(|c|{
        let Some(r)=RUNTIME.get() else{return false};
        c.game==game && r.adapter.necklace_cast_event(&r.local,event,c.spell).unwrap_or(false)
            && r.adapter.set_clock(&r.local,game)==Ok(c.clock)
            && r.adapter.effect_source(&r.local,game)==Ok(Some(c.caster))
    });
    if cast.is_some(){NEXT_NECKLACE_CAST.with(|v|v.set(None));}
    let previous=NECKLACE_EVENT.with(|v|v.replace(cast));
    run();
    NECKLACE_EVENT.with(|v|v.set(previous));
}
static LEECH_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Copy)]
struct LeechWindow {
    game: u64, caster: (u32, u64), target: (u32, u64), spell: u32, end: f32,
}
static LEECH_WINDOWS: std::sync::Mutex<Vec<LeechWindow>> = std::sync::Mutex::new(Vec::new());
unsafe extern "system" fn necklace_leech(game: u64) -> f32 {
    let original: unsafe extern "system" fn(u64) -> f32 = std::mem::transmute(LEECH_ORIGINAL.load(Ordering::Acquire));
    let base = original(game);
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else { return base };
    if !LEECH_WINDOWS.lock().is_ok_and(|w| !w.is_empty()) { return base; }
    let Ok(Some(target)) = r.adapter.effect_source(&r.local, game) else { return base };
    let Ok(now) = r.adapter.set_clock(&r.local, game) else { return base };
    let Some(party) = r.party.lock().ok().and_then(|p| p.clone()) else { return base };
    let present = |id: (u32,u64)| party.actors.iter().any(|a| (a.slot,a.guid)==id);
    let Ok(mut windows) = LEECH_WINDOWS.lock() else { return base };
    windows.retain(|w| w.game == game && w.end > now && present(w.caster) && present(w.target));
    base + 0.05 * windows.iter().filter(|w| w.target == target).count() as f32
}
type TalentCondition=unsafe extern "system" fn(u64,u64,u64)->bool;
unsafe extern "system" fn talent_condition(game:u64,condition:u64,event:u64)->bool{
    let original:TalentCondition=std::mem::transmute(CONDITION_ORIGINAL.load(Ordering::Acquire));
    let native=original(game,condition,event);
    let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return native};
    let players=r.party.lock().ok().and_then(|p|p.clone()).map(|p|p.actors.iter().filter(|a|a.phase==azsx_gameplay::Phase::Playing).map(|a|(a.slot,a.guid)).collect::<Vec<_>>()).unwrap_or_default();
    if let Some(s)=NECKLACE_EVENT.with(|v|v.get()).filter(|s| native && s.game==game && players.contains(&s.caster) && players.contains(&s.target)) {
        if let Ok(Some((now,end)))=r.adapter.necklace_leech_window(&r.local,game,s.spell) {
            if let Ok(mut windows)=LEECH_WINDOWS.lock() {
                windows.retain(|w| w.game==game && w.end>now && !(w.caster==s.caster && w.target==s.target && w.spell==s.spell));
                if windows.len()<50 {
                    windows.push(LeechWindow{game,caster:s.caster,target:s.target,spell:s.spell,end});
                    return false;
                }
            }
        }
        let io=io::Access{local:&r.local,adapter:r.adapter};
        match r.adapter.route_necklace_absorb(&io,game,s.spell,s.caster,s.target){
            Ok(true)=>return false,Ok(false)=>{},
            Err(e)=>r.log.warn(&format!("Necklace absorb routing rejected: {e}")),
        }
    }
    let shield=SHIELD_CONTEXT.with(|v|v.get()).filter(|(g,_,_)|*g==game).map(|(_,s,g)|(s,g));
    match r.adapter.talent_condition_override(&r.local,game,condition,event,shield,&players){
        Ok(Some(allowed))=>allowed,
        Ok(None)=>native,
        Err(e)=>{r.log.warn(&format!("Official talent condition rejected: {e}"));native}
    }
}

#[cfg(test)]
mod talent_abi_tests{
 use super::*;
 unsafe extern "system" fn native(game:u64,condition:u64,event:u64)->bool{assert_eq!((game,condition,event),(11,22,33));true}
 #[test]fn inactive_talent_condition_forwards_native_abi(){
  let old=CONDITION_ORIGINAL.swap(native as *const ()as usize,Ordering::SeqCst);
  let result=unsafe{talent_condition(11,22,33)};
  CONDITION_ORIGINAL.store(old,Ordering::SeqCst);assert!(result);
 }
 #[test]fn nested_shield_context_restores_the_outer_producer(){
  assert_eq!(SHIELD_CONTEXT.with(|v|v.get()),None);
  with_shield(1,17,100,||{assert_eq!(SHIELD_CONTEXT.with(|v|v.get()),Some((1,17,100)));with_shield(1,0,200,||assert_eq!(SHIELD_CONTEXT.with(|v|v.get()),Some((1,0,200))));assert_eq!(SHIELD_CONTEXT.with(|v|v.get()),Some((1,17,100)));});
  assert_eq!(SHIELD_CONTEXT.with(|v|v.get()),None);
 }
}
