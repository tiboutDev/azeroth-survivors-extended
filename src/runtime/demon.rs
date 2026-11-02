//! Host-owned native Banished Demon encounter; no parallel enemy simulation.
use super::{event_services::Services, io, Runtime, RUNTIME};
use azsx_game::official::{EventObject, GoblinDefinition, GoblinHandle};
use azsx_gameplay::{Phase, solo_events::{Identity, Kind, Observation, Proofs, Scene, Stage}};
use std::sync::{Mutex, OnceLock, atomic::{AtomicBool, AtomicU64, Ordering}};

static ENABLED: AtomicBool = AtomicBool::new(false);
static TICKING: AtomicBool = AtomicBool::new(false);
static ACTIVE_GAME: AtomicU64 = AtomicU64::new(0);
static ACTIVE_GUID: AtomicU64 = AtomicU64::new(0);
static DEFINITIONS: [OnceLock<GoblinDefinition>; 3] = [const { OnceLock::new() }; 3];
static STATE: Mutex<Option<State>> = Mutex::new(None);
struct State {
    game: u64, host: u64, clock: f32, next: f32, rng: u64, serial: u64,
    scene: Option<Scene>, objects: Vec<EventObject>, handle: Option<GoblinHandle>,
    point: [f32; 3], variant: usize, created: f32, ended: Option<f32>,
    killed: bool, charge: u32, inside: bool, participants: Vec<(u32,u64)>,
    #[cfg(feature="test-hooks")] preview_present: Option<bool>,
}
impl State {
    fn draw(&mut self, n: u32) -> u32 {
        self.rng=self.rng.wrapping_add(0x9e3779b97f4a7c15);
        let mut z=self.rng;
        z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);
        (z^(z>>31)) as u32 % n
    }
}
pub fn install(r: &Runtime) {
    if std::env::var("AZSX_BANISHED_DEMON").as_deref()==Ok("0") { return }
    let result=(|| -> Result<(), String> {
        for entry in r.adapter.demon_entries().into_iter()
            .chain(r.adapter.ritual_scene_entries())
            .chain(r.adapter.world_event_entries())
            .chain(r.adapter.event_placement_entries()) {
            if r.local.read(r.local.base+entry.rva as usize,entry.bytes.len())?!=entry.bytes {
                return Err(format!("Banished Demon native boundary changed at {:#x}",entry.rva));
            }
        }
        super::treasure_goblins::ensure_definition_hook(r)?;
        for variant in 0..3 {
            let definition=r.adapter.demon_definition(&r.local,r.local.base as u64,variant)?;
            let _=DEFINITIONS[variant].set(definition);
        }
        Ok(())
    })();
    match result {
        Ok(())=>{ENABLED.store(true,Ordering::Release);r.log.info("Banished Demon native event enabled.");},
        Err(error)=>r.log.warn(&format!("Banished Demon unavailable: {error}")),
    }
}
fn cleanup(r:&Runtime,mut state:State) {
    ACTIVE_GAME.store(0,Ordering::Release);ACTIVE_GUID.store(0,Ordering::Release);
    let same=r.adapter.event_party(&r.local,state.game).ok().flatten()
        .is_some_and(|p|p.clock>=state.clock&&p.actors.iter().any(|a|a.slot==0&&a.guid==state.host));
    if !same{return}
    let access=io::Access{local:&r.local,adapter:r.adapter};
    for object in state.objects {
        let _=unsafe{r.adapter.remove_event_object(&access,r.local.base as u64,state.game,object)};
    }
    if let Some(mut handle)=state.handle.take(){
        let _=unsafe{r.adapter.despawn_goblin(&access,r.local.base as u64,state.game,&mut handle)};
    }
}
pub fn reset() {
    let old=STATE.lock().ok().and_then(|mut s|s.take());
    if let (Some(r),Some(old))=(RUNTIME.get(),old){cleanup(r,old)}
    ACTIVE_GAME.store(0,Ordering::Release);ACTIVE_GUID.store(0,Ordering::Release);
}
fn notice(r:&Runtime,game:u64,text:&str){
    let access=io::Access{local:&r.local,adapter:r.adapter};
    let _=unsafe{r.adapter.event_notice(&access,r.local.base as u64,game,"AZSX Events",text)};
}
pub fn wire(game:u64,owner:u64)->Option<String>{
    let lock=STATE.lock().ok()?;
    let s=lock.as_ref().filter(|s|s.game==game&&s.participants.iter().any(|(_,id)|*id==owner))?;
    let scene=s.scene.as_ref()?;
    let stage=match scene.stage(){Stage::Waiting=>0,Stage::Preparing=>1,Stage::Engaged=>2,
        Stage::Succeeded=>3,Stage::Failed|Stage::Retired=>4};
    Some(format!("V|{owner}|{}|0|{stage}|{}|200|{}|{}|0|0|0|{:.3}|{:.3}\n",
        scene.id.event,(scene.held()*100.).round().min(200.) as u32,
        u8::from(s.inside),s.charge,s.point[0],s.point[1]))
}
pub fn active(game:u64)->bool{
    STATE.lock().ok().is_some_and(|s|s.as_ref().is_some_and(|s|s.game==game&&s.scene.as_ref().is_some_and(|scene|matches!(scene.stage(),Stage::Waiting|Stage::Preparing|Stage::Engaged))))
}
pub fn hit_complete(r:&Runtime,game:u64,enemy:u64){
    if ACTIVE_GAME.load(Ordering::Acquire)!=game||ACTIVE_GUID.load(Ordering::Acquire)!=enemy{return}
    if r.adapter.combat_hp(&r.local,game,enemy).ok().flatten().is_some_and(|hp|hp<=0.) {
        if let Ok(mut lock)=STATE.lock(){
            if let Some(s)=lock.as_mut().filter(|s|s.game==game&&s.handle.is_some_and(|h|h.guid()==enemy)){
                s.killed=true;
            }
        }
    }
}
pub fn finish(r:&Runtime,game:u64){
    if !ENABLED.load(Ordering::Acquire)||TICKING.swap(true,Ordering::AcqRel){return}
    struct Guard;impl Drop for Guard{fn drop(&mut self){TICKING.store(false,Ordering::Release)}}
    let _guard=Guard;
    if let Err(error)=tick(r,game){
        r.log.warn(&format!("Banished Demon stopped: {error}"));
        ENABLED.store(false,Ordering::Release);
        reset();
    }
}
fn tick(r:&Runtime,game:u64)->Result<(),String>{
    let Some(party)=r.adapter.event_party(&r.local,game)? else {reset();return Ok(())};
    let Some(host)=party.actors.iter().find(|a|a.slot==0).cloned()else{return Ok(())};
    let now=party.clock;
    #[cfg(feature="test-hooks")]
    let preview=std::env::var("AZSX_TEST_DEMON_PREVIEW").as_deref()==Ok("1");
    #[cfg(not(feature="test-hooks"))]
    let preview=false;
    let participants=party.actors.iter().filter(|a|a.phase!=Phase::Inactive)
        .map(|a|(a.slot,a.guid)).collect::<Vec<_>>();
    if participants.is_empty()||participants.iter().any(|(_,guid)|*guid==0){return Ok(())}
    let old={
        let mut lock=STATE.lock().map_err(|_|"Demon state unavailable")?;
        if lock.as_ref().is_none_or(|s|s.game!=game||s.host!=host.guid||now<s.clock){
            let old=lock.take();
            let seed=game^host.guid^std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
            let mut fresh=State{game,host:host.guid,clock:now,next:now,rng:seed,serial:0,
                scene:None,objects:Vec::new(),handle:None,point:[0.;3],variant:0,created:now,
                ended:None,killed:false,charge:0,inside:false,participants:participants.clone(),
                #[cfg(feature="test-hooks")] preview_present:None};
            fresh.next=now+90.+fresh.draw(61) as f32;
            #[cfg(feature="test-hooks")]
            if preview{fresh.next=now+25.}
            *lock=Some(fresh);old
        }else{None}
    };
    if let Some(old)=old{cleanup(r,old)}
    let escort_active=super::solo_escort::active(game);
    let work={
        let mut lock=STATE.lock().map_err(|_|"Demon state unavailable")?;
        let s=lock.as_mut().ok_or("Demon state missing")?;
        s.clock=now;s.participants=participants.clone();
        if s.ended.is_some_and(|at|now-at>=5.){
            let old=(std::mem::take(&mut s.objects),s.handle.take());
            s.scene=None;s.ended=None;s.killed=false;s.charge=0;
            Some((None,old))
        }else if s.scene.is_none()&&!escort_active&&now>=s.next
            && party.actors.iter().any(|a|a.hp>0.&&a.phase==Phase::Playing) {
            s.next=now+180.+s.draw(121) as f32;
            s.serial=s.serial.checked_add(1).ok_or("Demon event identity exhausted")?;
            let variant=if preview{2}else{s.draw(3) as usize};
            let draws=(0..12).map(|_|(s.draw(u32::MAX),s.draw(u32::MAX))).collect::<Vec<_>>();
            Some((Some((s.serial,variant,draws)),(Vec::new(),None)))
        }else{None}
    };
    if let Some((create,(objects,handle)))=work{
        if !objects.is_empty()||handle.is_some(){
            let expired=State{game,host:host.guid,clock:now,next:0.,rng:0,serial:0,scene:None,
                objects,handle,point:[0.;3],variant:0,created:now,ended:None,killed:false,
                charge:0,inside:false,participants:Vec::new(),
                #[cfg(feature="test-hooks")] preview_present:None};
            cleanup(r,expired);
        }
        if let Some((serial,variant,draws))=create{
            let area=r.events_area(game)?;
            let mut selected=None;
            #[cfg(feature="test-hooks")]
            if preview{
                for offset in [[5.,0.],[0.,5.],[-5.,0.],[0.,-5.]] {
                    let xy=[host.position[0]+offset[0],host.position[1]+offset[1]];
                    if let Some(site)=unsafe{r.adapter.event_ground_sample(
                        &r.local,r.local.base as u64,game,xy,4.
                    )}?{selected=Some(site.center);break}
                }
            }
            for group in draws.chunks(4){
                if selected.is_some(){break}
                selected=super::world_events::place(r,game,area,4.,group)?;
            }
            if let Some(point)=selected{
                let access=io::Access{local:&r.local,adapter:r.adapter};
                let objects=unsafe{r.adapter.demon_scene(
                    &access,r.local.base as u64,game,point,variant
                )}?;
                let proofs=Proofs{assets:true,cleanup:true,hostile_combat:true,
                    banish:true,attribution:true,..Proofs::default()};
                let scene=Scene::new(Identity{run:game,owner:host.guid,event:serial},
                    Kind::BanishedDemon,now,proofs,party.actors.len()==1)
                    .ok_or("Demon scene policy refused verified context")?;
                let mut lock=STATE.lock().map_err(|_|"Demon state unavailable")?;
                let s=lock.as_mut().filter(|s|s.game==game&&s.host==host.guid&&s.scene.is_none())
                    .ok_or("Demon scene authority changed")?;
                s.objects=objects;s.scene=Some(scene);s.point=point;s.variant=variant;
                s.created=now;s.ended=None;s.charge=0;s.inside=false;
                #[cfg(feature="test-hooks")] { s.preview_present=None; }
                drop(lock);
                notice(r,game,"A bound demon awaits. Break the binding and defeat it to gain Banish.");
                r.log.info(&format!("Banished Demon created: event={serial} variant={variant}"));
            }
        }
    }
    // Test-hook rehearsal: stage all owned actors near the binding, withdraw
    // one briefly, then return. Production never changes participant positions.
    #[cfg(feature="test-hooks")]
    if preview {
        let scene=STATE.lock().ok().and_then(|lock|lock.as_ref()
            .filter(|s|s.game==game&&s.scene.is_some())
            .map(|s|(s.point,now-s.created)));
        if let Some((focus,age))=scene{
            for actor in party.actors.iter().filter(|a|a.phase==Phase::Playing){
                let withdrawing=if party.actors.len()==1{actor.slot==0}else{actor.slot==1};
                let offset=if withdrawing&&(0.5 ..2.5).contains(&age){7.}else{0.5};
                r.adapter.fixture_event_pose(&io::Access{local:&r.local,adapter:r.adapter},
                    game,actor.slot,actor.guid,[focus[0]+offset,focus[1],focus[2]])?;
            }
        }
    }
    #[cfg(feature="test-hooks")]
    let party=if preview{r.adapter.event_party(&r.local,game)?
        .ok_or("Demon preview party retired")?}else{party};
    let (release,finish,retire,point,variant,objects,handle)={
        let mut lock=STATE.lock().map_err(|_|"Demon state unavailable")?;
        let s=lock.as_mut().ok_or("Demon state missing")?;
        let Some(scene)=s.scene.as_mut()else{return Ok(())};
        let present=azsx_gameplay::solo_events::event_party_present(
            party.actors.iter().filter(|a|a.phase!=Phase::Inactive)
                .map(|a|(a.hp>0.&&a.phase==Phase::Playing,[a.position[0],a.position[1]])),
            [s.point[0],s.point[1]],4.);
        #[cfg(feature="test-hooks")]
        if preview&&s.preview_present!=Some(present) {
            r.log.info(&format!("demon preview group: inside={present} held={:.2}",scene.held()));
            s.preview_present=Some(present);
        }
        s.inside=present;
        let before=scene.stage();
        let alive=party.actors.iter().any(|a|a.hp>0.);
        let playing=party.actors.iter().filter(|a|a.hp>0.&&a.phase!=Phase::Inactive)
            .all(|a|a.phase==Phase::Playing);
        scene.observe(Observation{id:scene.id,now,playing,solo:party.actors.len()==1,
            player_alive:alive,present,unit_alive:true,arrived:false,enemy_defeated:s.killed});
        if now-s.created>240.&&!matches!(scene.stage(),Stage::Succeeded|Stage::Failed|Stage::Retired){
            scene.retire();
        }
        let release=before!=Stage::Engaged&&scene.stage()==Stage::Engaged;
        let finish=scene.take_reward().is_some();
        let retire=matches!(scene.stage(),Stage::Failed|Stage::Retired)
            || finish;
        if retire&&s.ended.is_none(){s.ended=Some(now)}
        let objects=if release||retire{std::mem::take(&mut s.objects)}else{Vec::new()};
        let handle=if retire{s.handle.take()}else{None};
        (release,finish,retire,s.point,s.variant,objects,handle)
    };
    let access=io::Access{local:&r.local,adapter:r.adapter};
    for object in objects{
        unsafe{r.adapter.remove_event_object(&access,r.local.base as u64,game,object)}?;
    }
    if release{
        let definition=DEFINITIONS[variant].get().ok_or("Demon definition absent")?;
        unsafe{r.adapter.demon_query(&access,r.local.base as u64,game,variant)}?;
        let spawned=super::treasure_goblins::with_definition(game,definition,||unsafe{
            r.adapter.demon_spawn(&r.local,r.local.base as u64,game,point,variant)
        })?;
        let handle=spawned.ok_or("Native demon spawn rejected")?;
        ACTIVE_GAME.store(game,Ordering::Release);
        ACTIVE_GUID.store(handle.guid(),Ordering::Release);
        let mut lock=STATE.lock().map_err(|_|"Demon state unavailable")?;
        let s=lock.as_mut().filter(|s|s.game==game&&s.host==host.guid)
            .ok_or("Demon scene authority changed after spawn")?;
        s.handle=Some(handle);
        notice(r,game,"The binding is broken! Defeat the demon.");
        r.log.info(&format!("Banished Demon engaged: guid={} variant={variant}",handle.guid()));
    }
    if finish{
        let count=r.adapter.demon_grant_banish(&access,game,host.guid)?
            .ok_or("Demon Banish recipient changed")?;
        if let Ok(mut lock)=STATE.lock(){
            if let Some(s)=lock.as_mut().filter(|s|s.game==game&&s.host==host.guid){s.charge=count}
        }
        notice(r,game,"The demon is defeated. Banish gained.");
        r.log.info(&format!("Banished Demon Banish granted once: count={count}"));
    }
    if retire{
        ACTIVE_GAME.store(0,Ordering::Release);ACTIVE_GUID.store(0,Ordering::Release);
        if let Some(mut handle)=handle{
            unsafe{r.adapter.despawn_goblin(&access,r.local.base as u64,game,&mut handle)}?;
        }
    }
    Ok(())
}
