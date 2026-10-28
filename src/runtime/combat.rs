//! Shared marks and semantic taunt ownership over the official simulation.
use super::*;
use asx_gameplay::{
    marks::Marks,
    taunts::{Hit, Taunts},
    Mark, Player, Talents, TauntSpell,
};
use std::{cell::Cell, collections::BTreeMap, sync::Mutex};
#[derive(Clone)]
struct Pending {
    game: u64,
    enemy: u64,
    target: u32,
    guid: u64,
    fallback: u32,
    fallback_guid: u64,
    amount: f32,
}
#[derive(Default)]
struct State {
    game: u64,
    clock: f32,
    players: Vec<Player>,
    marks: Marks,
    taunts: Taunts,
    immunity:asx_gameplay::immunity_aggro::ImmunityAggro,
    suppressed:std::collections::BTreeSet<u64>,
    pending: Vec<Pending>,
    observed: BTreeMap<(u64, u32), u32>,
    #[cfg(feature = "test-hooks")]
    fixture_marks: BTreeMap<u32, Mark>,
    #[cfg(feature = "test-hooks")]
    fixture_talents: BTreeMap<u32, Talents>,
}
static STATE: std::sync::LazyLock<Mutex<State>> =
    std::sync::LazyLock::new(|| Mutex::new(State::default()));
static TICK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static DAMAGE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static NPC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static LOOP_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
static BASIC_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static BASIC_DISPATCH: AtomicUsize = AtomicUsize::new(0);
thread_local! {static ORIGIN:Cell<Option<(u64,[f32;3])>>=const{Cell::new(None)};static CHANGED:Cell<bool>=const{Cell::new(false)};}
pub fn reset() {
    if let Ok(mut s) = STATE.lock() {
        *s = State::default();
    }
}
pub fn capture(r: &Runtime, game: u64) {
    let Some(p) = r.party.lock().ok().and_then(|p| p.clone()) else {
        return;
    };
    let Ok(enemies) = r.adapter.combat_enemies(&r.local, game) else {
        return;
    };
    let immune=p.actors.iter().filter_map(|a|r.adapter.combat_immune(&r.local,game,a.slot,a.guid).ok().map(|immune|(a.slot+1,a.guid,immune))).collect::<Vec<_>>();
    if let Ok(mut s) = STATE.lock() {
        if s.game != game || p.clock + 0.01 < s.clock {
            *s = State::default();
        }
        for a in &p.actors {
            if s.players
                .iter()
                .any(|old| old.id == a.slot + 1 && old.life != a.guid)
            {
                s.marks.players.remove(&(a.slot + 1));
            }
        }
        s.game = game;
        s.clock = p.clock;
        s.players = p
            .actors
            .iter()
            .map(|a| Player {
                id: a.slot + 1,
                phase: a.phase,
                life: a.guid,
                position: Some(a.position),
            })
            .collect();
        let ids=s.players.iter().map(|p|(p.id,p.life)).collect::<Vec<_>>();s.immunity.retain(&ids);
        for(id,life,immune)in immune{let State{immunity,taunts,..}=&mut *s;immunity.observe(id,life,immune,taunts);}
        s.suppressed.retain(|id|enemies.iter().any(|e|e.id==*id&&e.alive));
        let players = s.players.clone();
        s.taunts.settle(&players, &enemies, []);
        s.marks
            .players
            .retain(|id, _| players.iter().any(|p| p.id == *id));
    }
}
fn reconcile(r: &Runtime, game: u64) {
    let Ok(enemies) = r.adapter.combat_enemies(&r.local, game) else {
        return;
    };
    let (writes, visuals) = if let Ok(mut s) = STATE.lock() {
        let players = s.players.clone();
        s.taunts.settle(&players, &enemies, []);
        let writes = s.marks.reconcile(&players, &enemies);
        let mut visuals = vec![];
        for _ in 0..32 {
            if let Some(v) = s.marks.next_visual() {
                visuals.push(v)
            } else {
                break;
            }
        }
        (writes, visuals)
    } else {
        return;
    };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    for w in writes {
        if let Err(e) = r.adapter.write_combat_mark(&io, game, w.enemy, w.strength) {
            r.log
                .warn(&format!("Official mark projection rejected: {e}"));
        }
    }
    for v in visuals {
        type Aura = unsafe extern "system" fn(u64, u64, u32, bool);
        let call: Aura = unsafe {
            std::mem::transmute(r.local.base + r.adapter.combat_entries()[4].rva as usize)
        };
        unsafe { call(game, v.enemy, 1130, v.active) };
        r.log.info(&format!(
            "official mark aura: enemy={} active={}",
            v.enemy, v.active
        ));
    }
}
type Tick = unsafe extern "system" fn(u64, f32);
unsafe extern "system" fn tick(game: u64, dt: f32) {
    let original: Tick = std::mem::transmute(TICK_ORIGINAL.load(Ordering::Acquire));
    original(game, dt);
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    let Ok(actor) = r.adapter.combat_actor(&r.local, game, r.local.base as u64) else {
        return;
    };
    if let Ok(mut s) = STATE.lock() {
        if let Some(p) = s
            .players
            .iter_mut()
            .find(|p| p.id == actor.slot + 1 && p.life == actor.guid)
        {
            p.position = Some(actor.position);
        }
        let mark = actor.mark;
        #[cfg(feature = "test-hooks")]
        let mark = s
            .fixture_marks
            .get(&(actor.slot + 1))
            .copied()
            .unwrap_or(mark);
        s.marks.players.insert(actor.slot + 1, mark);
    }
    observe_immunity(r,game);
    reconcile(r, game);
}
type Damage = unsafe extern "system" fn(u64, u64, f32, u32, u32, bool, bool);
unsafe extern "system" fn damage(
    game: u64,
    enemy: u64,
    amount: f32,
    spell: u32,
    school: u32,
    flag: bool,
    extra: bool,
) {
    let original: Damage = std::mem::transmute(DAMAGE_ORIGINAL.load(Ordering::Acquire));
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return original(game, enemy, amount, spell, school, flag, extra);
    };
    let explosion=glyphs::explosion_multiplier(r,game,spell);
    #[cfg(feature="test-hooks")]
    {glyphs::fixture_warlock_hit(r,game,spell,amount);glyphs::fixture_shaman_hit(r,game,spell,amount);glyphs::fixture_explosion(r,game,spell,amount,amount*explosion,explosion);if spell==14186{glyphs::fixture_seal_hit(r,game,enemy,amount);}if spell==724{glyphs::fixture_lightwell_hit(r,game,amount);}}
    let scaled = amount * explosion * fatigue::multiplier(r, game);
    #[cfg(feature="test-hooks")]
    fatigue::fixture_trace(r,game,0,amount,scaled);
    let amount = scaled;
    #[cfg(feature = "test-hooks")]
    support::observe_combat_damage(game, enemy, amount, spell);
    let strength = STATE
        .lock()
        .ok()
        .and_then(|s| s.marks.effective(&s.players).get(&enemy).copied());
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    if let Some(strength) = strength {
        if let Err(e) = r.adapter.write_combat_mark(&io, game, enemy, strength) {
            r.log.warn(&format!("Official mark damage rejected: {e}"));
        }
    }
    observe_immunity(r,game);
    let kind = match spell {
        23922 => Some(TauntSpell::ShieldSlam),
        853 => Some(TauntSpell::HammerOfJustice),
        5221 => Some(TauntSpell::Shred),
        _ => None,
    };
    let actor = kind.and_then(|kind| {
        r.adapter
            .combat_actor(&r.local, game, r.local.base as u64)
            .ok()
            .and_then(|a| {
                let talents = a.talents;
                #[cfg(feature = "test-hooks")]
                let talents = STATE
                    .lock()
                    .ok()
                    .and_then(|s| s.fixture_talents.get(&(a.slot + 1)).copied())
                    .unwrap_or(talents);
                Taunts::eligible(talents, kind).then_some((a, kind))
            })
    });
    let before = if actor.is_some() || strength.is_some() {
        r.adapter.combat_hp(&r.local, game, enemy).ok().flatten()
    } else {
        None
    };
    original(game, enemy, amount, spell, school, flag, extra);
    let after = before.and_then(|_| r.adapter.combat_hp(&r.local, game, enemy).ok().flatten());
    if let (Some(strength), Some(before), Some(after)) = (strength, before, after) {
        if before > after {
            r.log.info(&format!("official marked damage: enemy={enemy} strength={strength} raw={amount} effective={}",before-after));
        }
    }
    if let (Some((actor, kind)), Some(before), Some(after)) = (actor, before, after) {
        if before > after && after > 0.0 {
            if let Ok(enemies) = r.adapter.combat_enemies(&r.local, game) {
                if let Ok(mut s) = STATE.lock() {
                    let players = s.players.clone();
                    let changes = s.taunts.settle(
                        &players,
                        &enemies,
                        [Hit {
                            enemy,
                            player: actor.slot + 1,
                            spell: kind,
                            life: actor.guid,
                        }],
                    );
                    if !changes.is_empty() {
                        r.log.info(&format!(
                            "official taunt applied: enemy={enemy} caster={} spell={spell}",
                            actor.slot
                        ));
                    }
                }
            }
        }
    }
}
unsafe extern "system" fn npc(game: u64, dt: f32) {
    let original: Tick = std::mem::transmute(NPC_ORIGINAL.load(Ordering::Acquire));
    let origin = RUNTIME
        .get()
        .filter(|r| r.rules)
        .and_then(|r| r.adapter.support_caster(&r.local, game).ok())
        .map(|a| (game, a.position));
    let _immunity=RUNTIME.get().filter(|r|r.rules).and_then(|r|prepare_immunity(r,game));
    let old = ORIGIN.with(|o| o.replace(origin));
    let changed = CHANGED.with(|v| v.replace(false));
    original(game, dt);
    ORIGIN.with(|o| o.set(old));
    CHANGED.with(|v| v.set(changed));
}
crate::context_stub_rw!(loop_entry, 0, loop_start, LOOP_ORIGINAL);
unsafe extern "system" fn loop_start(c: &mut crate::stub::Context) {
    if !CHANGED.with(|v| v.replace(false)) {
        return;
    }
    let Some((game, pose)) = ORIGIN.with(|o| o.get()) else {
        return;
    };
    let Some(r) = RUNTIME.get() else { return };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    if let Err(e) = r.adapter.restore_basic_origin(&io, c.rbp, game, pose) {
        r.log.warn(&format!("Official basic origin rejected: {e}"));
    }
}
crate::context_stub_rw!(basic_entry, 0, basic, BASIC_DISPATCH);
#[cfg(test)]
#[unsafe(naked)]
unsafe extern "C" fn basic_resume() {
    core::arch::naked_asm!("test rax, rax","jz 2f","movss xmm14, xmm5","2:","jmp qword ptr [rip+{original}]",original=sym BASIC_ORIGINAL);
}
unsafe extern "system" fn basic(c: &mut crate::stub::Context) {
    c.rax = 0;
    let Some(r) = RUNTIME.get().filter(|r| r.rules) else {
        return;
    };
    if STATE.lock().is_ok_and(|s| s.taunts.bindings.is_empty()) {
        return;
    }
    let Ok(frame) = r.adapter.basic_frame(&r.local, c.rbp) else {
        return;
    };
    if !ORIGIN.with(|o| o.get().is_some_and(|(g, _)| g == frame.game)) {
        return;
    }
    let target = STATE.lock().ok().and_then(|s| {
        let id = s.taunts.target(frame.enemy, &s.players)?;
        let p = s.players.iter().find(|p| p.id == id)?;
        Some((id, p.position?))
    });
    let Some((id, pose)) = target else { return };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    match r.adapter.prepare_basic_target(&io, &frame, pose) {
        Ok(distance) => {
            CHANGED.with(|v| v.set(true));
            c.xmm[5] = (c.xmm[5] & !(u32::MAX as u128)) | u128::from(distance.to_bits());
            c.rax = 1;
            if let Ok(mut s) = STATE.lock() {
                let uses = s.observed.entry((frame.enemy, id)).or_default();
                *uses = uses.saturating_add(1);
                if *uses == 1 {
                    r.log.info(&format!(
                        "official taunt pursuit: enemy={} caster={} position={pose:?}",
                        frame.enemy,
                        id - 1
                    ));
                }
            }
        }
        Err(e) => r.log.warn(&format!("Official basic target rejected: {e}")),
    }
}
pub fn redirect_melee(r: &Runtime, game: u64, amount: f32, enemy: u64, melee: bool) -> bool {
    if !melee || enemy == 0 {
        return false;
    }
    let binding = STATE.lock().ok().and_then(|s| {
        let id = s.taunts.target(enemy, &s.players)?;
        let p = s.players.iter().find(|p| p.id == id)?;
        Some((id - 1, p.life))
    });
    let Some((target, guid)) = binding else {
        return false;
    };
    let Ok(actor) = r.adapter.support_caster(&r.local, game) else {
        return false;
    };
    if actor.slot == target {
        return false;
    }
    if let Ok(mut s) = STATE.lock() {
        if s.pending.len() < 128 {
            s.pending.push(Pending {
                game,
                enemy,
                target,
                guid,
                fallback: actor.slot,
                fallback_guid: actor.guid,
                amount,
            });
            return true;
        }
    }
    false
}
pub fn finish(r: &Runtime, game: u64) {
    let pending = STATE
        .lock()
        .map(|mut s| std::mem::take(&mut s.pending))
        .unwrap_or_default();
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    for p in pending {
        if p.game != game {
            continue;
        }
        match r
            .adapter
            .hurt_actor(&io, game, p.target, p.guid, p.amount, p.enemy, true)
        {
            Ok(Some(effective)) => r.log.info(&format!(
                "official taunt melee: enemy={} original={} recipient={} effective={effective}",
                p.enemy, p.fallback, p.target
            )),
            Ok(None) => {
                if let Ok(mut s) = STATE.lock() {
                    s.taunts.bindings.remove(&p.enemy);
                }
                let _ = r.adapter.hurt_actor(
                    &io,
                    game,
                    p.fallback,
                    p.fallback_guid,
                    p.amount,
                    p.enemy,
                    true,
                );
            }
            Err(e) => r.log.warn(&format!("Official taunt hurt rejected: {e}")),
        }
    }
    reconcile(r, game);
}
pub fn install_descriptions(r: &Runtime) -> Result<(), String> {
    for (address, text) in r
        .adapter
        .combat_descriptions(&r.local, r.local.base as u64)?
    {
        let pointer = r.local.alloc(text.len())?;
        r.local.write(pointer, text.as_bytes())?;
        let mut header = (pointer as u64).to_le_bytes().to_vec();
        header.extend((text.len() as u64).to_le_bytes());
        r.local.patch_at(address as usize, &header)?;
    }
    r.log
        .info("official taunt descriptions: three English native talent annotations installed");
    Ok(())
}
pub fn hooks(adapter: Adapter) -> Vec<crate::install::HookSpec> {
    let (resume,original)=adapter.basic_resume_entry();
    BASIC_DISPATCH.store(resume, Ordering::Release);
    vec![
        crate::install::HookSpec {
            name: "official-combat-player",
            target: local_base() + adapter.combat_entries()[0].rva as usize,
            detour: tick as *const () as usize,
            trampoline: &TICK_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-combat-damage",
            target: local_base() + adapter.support_proc_entries()[0].rva as usize,
            detour: damage as *const () as usize,
            trampoline: &DAMAGE_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-combat-npc",
            target: local_base() + adapter.combat_entries()[1].rva as usize,
            detour: npc as *const () as usize,
            trampoline: &NPC_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-combat-loop",
            target: local_base() + adapter.combat_entries()[2].rva as usize,
            detour: loop_entry as *const () as usize,
            trampoline: &LOOP_ORIGINAL,
        },
        crate::install::HookSpec {
            name: "official-combat-basic",
            target: local_base() + adapter.combat_entries()[3].rva as usize,
            detour: basic_entry as *const () as usize,
            trampoline: original,
        },
    ]
}
#[cfg(feature = "test-hooks")]
pub fn fixture_mark(slot: u32, mark: Mark) {
    if let Ok(mut s) = STATE.lock() {
        s.fixture_marks.insert(slot + 1, mark);
        s.marks.players.insert(slot + 1, mark);
    }
}
#[cfg(feature = "test-hooks")]
pub fn fixture_talents(slot: u32, t: Talents) {
    if let Ok(mut s) = STATE.lock() {
        s.fixture_talents.insert(slot + 1, t);
    }
}
#[cfg(feature = "test-hooks")]
pub fn fixture_owner(enemy: u64) -> Option<u32> {
    STATE
        .lock()
        .ok()
        .and_then(|s| s.taunts.target(enemy, &s.players).map(|p| p - 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[unsafe(naked)]
    unsafe extern "C" fn continuation() {
        core::arch::naked_asm!("ret");
    }
    #[unsafe(naked)]
    unsafe extern "system" fn gate_probe(_flag: u64, _old: f32, _new: f32) -> f32 {
        core::arch::naked_asm!("sub rsp, 88","movaps [rsp+32], xmm14","movaps xmm14, xmm1","movaps xmm5, xmm2","mov rax, rcx","call {gate}","movss xmm0, xmm14","movaps xmm14, [rsp+32]","add rsp, 88","ret",gate=sym basic_resume);
    }
    #[test]
    fn basic_resume_preserves_native_distance_or_selects_only_the_tank_distance() {
        let old = BASIC_ORIGINAL.swap(continuation as *const () as usize, Ordering::SeqCst);
        let (native, tank) = unsafe { (gate_probe(0, 3.0, 7.0), gate_probe(1, 3.0, 7.0)) };
        BASIC_ORIGINAL.store(old, Ordering::SeqCst);
        assert_eq!(native, 3.0);
        assert_eq!(tank, 7.0);
    }
}

/// Observe actual native protection, including immunity activated by an incoming
/// lethal hit. Only its rising edge discards old taunts; new hits remain eligible.
pub fn observe_immunity(r:&Runtime,game:u64){
 if !r.adapter.supports_immunity_aggro(){return}
 let Ok(actor)=r.adapter.support_caster(&r.local,game)else{return};
 let Ok(immune)=r.adapter.combat_immune(&r.local,game,actor.slot,actor.guid)else{return};
 if let Ok(mut s)=STATE.lock(){if s.game!=game{return}let State{immunity,taunts,..}=&mut *s;
  if immunity.observe(actor.slot+1,actor.guid,immune,taunts){r.log.info(&format!("official immunity aggro cleared: slot={} guid={}",actor.slot,actor.guid));}
 }
}
struct ImmunityScope<'a>{r:&'a Runtime,loan:Option<asx_game::official::AggroLoan>}
impl Drop for ImmunityScope<'_>{fn drop(&mut self){if let Some(loan)=self.loan.take(){let io=io::Access{local:&self.r.local,adapter:self.r.adapter};if let Err(e)=self.r.adapter.aggro_restore(&io,loan){self.r.log.warn(&format!("Official immunity pool restore rejected: {e}"));}}}}
fn prepare_immunity(r:&Runtime,game:u64)->Option<ImmunityScope<'_>>{
 if !r.adapter.supports_immunity_aggro(){return None}observe_immunity(r,game);
 let blocked={let s=STATE.lock().ok()?;if s.game!=game{return None}let b=s.players.iter().filter(|p|s.immunity.immune(p.id)).map(|p|p.id-1).collect::<Vec<_>>();if b.is_empty()&&s.suppressed.is_empty(){return None}b};
 let result=(||->Result<_,String>{
  let (candidates,enemies)=r.adapter.aggro_snapshot(&r.local,game)?;
  let changes={let mut s=STATE.lock().map_err(|_|"Immunity state unavailable")?;let mut changes=vec![];
   for e in enemies{
    if e.target.is_some_and(|t|blocked.contains(&t))||s.suppressed.contains(&e.id){
     let taunt=s.taunts.target(e.id,&s.players).map(|p|p-1);
     let next=asx_gameplay::immunity_aggro::replacement(e.target,e.position,&candidates,&blocked,taunt);
     if next!=e.target{changes.push((e.id,next));}
     if next.is_none(){s.suppressed.insert(e.id);}else{s.suppressed.remove(&e.id);}
    }
   }changes
  };
  let io=io::Access{local:&r.local,adapter:r.adapter};r.adapter.aggro_prepare(&io,game,&blocked,&changes)
 })();match result{Ok(loan)=>Some(ImmunityScope{r,loan:Some(loan)}),Err(e)=>{r.log.warn(&format!("Official immunity targeting rejected: {e}"));None}}
}

#[cfg(feature="test-hooks")]
pub fn immunity_fixture_enabled()->bool{std::env::var("AZCOOP_TEST_IMMUNITY_AGGRO").as_deref()==Ok("1")}
#[cfg(feature="test-hooks")]
pub fn immunity_fixture_tick(r:&Runtime,server:u64){
 static STAGE:std::sync::Mutex<(usize,u8,f32,u64)>=std::sync::Mutex::new((0,0,0.,0));
 let Ok(Some(p))=r.adapter.party(&r.local,server)else{return};
 if p.actors.len()!=2||p.clock<2.||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 let (case,stage,began,mut enemy)=*STAGE.lock().unwrap();if case>=4{return}let slot=(case%2)as u32;let spell=[642,20580,7744,1856][case];let game=match r.adapter.game_address(server){Ok(g)=>g,Err(_)=>return};
 let io=io::Access{local:&r.local,adapter:r.adapter};
 let result=(||->Result<Option<(usize,u8,f32,u64)>,String>{
  if stage==0{
   // An ordinary native immunity may already be active (e.g. a racial proc).
   // Establish a real false -> true transition before asserting threat removal;
   // a taunt deliberately made while already immune is required to survive.
   if r.adapter.combat_immune(&r.local,game,slot,p.actors[slot as usize].guid)?{return Ok(None)}
   enemy=r.adapter.fixture_combat_enemy(&io,game,5)?.ok_or("Missing fixture enemy")?;
   fixture_talents(slot,Talents{fortified:true,..Default::default()});r.adapter.attack_actor(&io,game,slot,enemy,23922,1.)?;
   let old=fixture_owner(enemy);r.adapter.fixture_immunity(&io,game,slot,spell,5.)?;
   // Refresh the actual protected actor from the safe restored-host boundary.
   let immune=r.adapter.combat_immune(&r.local,game,slot,p.actors[slot as usize].guid)?;
   {let mut s=STATE.lock().unwrap();let State{immunity,taunts,..}=&mut *s;immunity.observe(slot+1,p.actors[slot as usize].guid,immune,taunts);}
   let loan=r.adapter.aggro_prepare(&io,game,&[],&[(enemy,Some(slot))])?;r.adapter.aggro_restore(&io,loan)?;
   r.log.info(&format!("immunity fixture begin: case={case} slot={slot} spell={spell} active={immune} old={old:?} cleared={:?}",fixture_owner(enemy)));
   return Ok(Some((case,1,p.clock,enemy)))
  }
  if stage==1&&p.clock>began+0.8{
   let(c,enemies)=r.adapter.aggro_snapshot(&r.local,game)?;let target=enemies.iter().find(|e|e.id==enemy).and_then(|e|e.target);
   let restored=c.iter().all(|c|c.available);
   let effective=r.adapter.hurt_actor(&io,game,slot,p.actors[slot as usize].guid,1.,enemy,false)?;
   r.adapter.attack_actor(&io,game,slot,enemy,23922,1.)?;
   r.log.info(&format!("immunity fixture retarget: case={case} target={target:?} expected={} restored={restored} effective={effective:?} retaunt={:?}",1-slot,fixture_owner(enemy)));
   return Ok(Some((case,2,began,enemy)))
  }
  if stage==2&&p.clock>began+1.8{
   r.log.info(&format!("immunity fixture held: case={case} owner={:?} expected={slot}",fixture_owner(enemy)));return Ok(Some((case,3,began,enemy)))
  }
  if stage==3&&p.clock>began+5.5{
   let immune=r.adapter.combat_immune(&r.local,game,slot,p.actors[slot as usize].guid)?;
   r.log.info(&format!("immunity fixture expired: case={case} active={immune} owner={:?} expected={slot}",fixture_owner(enemy)));return Ok(Some((case+1,0,p.clock,0)))
  }Ok(None)
 })();match result{Ok(Some(next))=>*STAGE.lock().unwrap()=next,Ok(None)=>{},Err(e)=>{r.log.warn(&format!("Immunity fixture rejected: {e}"));*STAGE.lock().unwrap()=(4,0,0.,0);}}
}
