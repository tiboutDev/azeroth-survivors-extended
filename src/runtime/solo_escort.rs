//! One invulnerable native Mechanical Chicken escort in solo play.
use super::{event_services::Services, io, Runtime, RUNTIME};
use azsx_game::official::EventObject;
use azsx_gameplay::{Phase, solo_events::{Identity, Kind, Observation, Proofs, RunOpportunities, Scene, Stage}};
use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};

static ENABLED: AtomicBool = AtomicBool::new(false);
static TICKING: AtomicBool = AtomicBool::new(false);
static STATE: Mutex<Option<State>> = Mutex::new(None);
static OPPORTUNITIES: Mutex<RunOpportunities> = Mutex::new(RunOpportunities::new());
struct State {
    game:u64, owner:u64, clock:f32, next:f32, rng:u64, serial:u64,
    scene:Option<Scene>, objects:Vec<EventObject>, route:Vec<[f32;3]>,
    leg:usize, next_move:f32, created:f32, ended:Option<f32>, arrived:bool, inside:bool,
}
impl State {
    fn draw(&mut self,n:u32)->u32 {
        self.rng=self.rng.wrapping_add(0x9e3779b97f4a7c15);
        let mut z=self.rng;
        z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);
        (z^(z>>31))as u32%n
    }
}
pub fn install(r:&Runtime) {
    let entries=r.adapter.escort_motion_entries().into_iter()
        .chain(r.adapter.ritual_scene_entries())
        .chain(r.adapter.world_event_entries())
        .chain(r.adapter.event_placement_entries())
        .chain(r.adapter.solo_engineering_entries())
        .chain([r.adapter.native_hit_effect_entry()]);
    for e in entries {
        let actual=r.local.read(r.local.base+e.rva as usize,e.bytes.len()).ok();
        if actual.as_deref()!=Some(e.bytes) {
            r.log.warn(&format!("Solo escort unavailable: native boundary changed at {:#x}; observed={:02x?}",e.rva,actual));
            return;
        }
    }
    ENABLED.store(true,Ordering::Release);
}
fn cleanup(r:&Runtime,s:State) {
    let same=r.adapter.solo_run(&r.local,s.game).ok().flatten()
        .is_some_and(|p|p.clock>=s.clock&&p.actors.first().is_some_and(|a|a.guid==s.owner));
    if !same{return}
    let access=io::Access{local:&r.local,adapter:r.adapter};
    for object in s.objects {
        let _=unsafe{r.adapter.remove_event_object(&access,r.local.base as u64,s.game,object)};
    }
}
/// The native completed-encounter callback, not elapsed time, opens the first
/// escort opportunity for this solo run.
pub fn encounter(r:&Runtime,g:u64) {
    if !ENABLED.load(Ordering::Acquire){return}
    let Ok(Some(p))=r.adapter.solo_run(&r.local,g)else{return};
    let Some(a)=p.actors.first()else{return};
    if a.hp<=0.||a.phase==Phase::Inactive{return}
    if let Ok(mut budget)=OPPORTUNITIES.lock(){budget.encounter(g,a.guid,p.clock);}
}
pub fn reset() {
    super::solo_gadgets::reset();
    let old=STATE.lock().ok().and_then(|mut s|s.take());
    if let (Some(r),Some(old))=(RUNTIME.get(),old){cleanup(r,old)}
}
fn notice(r:&Runtime,g:u64,text:&str) {
    let access=io::Access{local:&r.local,adapter:r.adapter};
    let _=unsafe{r.adapter.event_notice(&access,r.local.base as u64,g,"AZSX Events",text)};
}
pub fn wire(g:u64,owner:u64)->Option<String> {
    let lock=STATE.lock().ok()?;
    let s=lock.as_ref().filter(|s|s.game==g&&s.owner==owner)?;
    let scene=s.scene.as_ref()?;
    let stage=match scene.stage(){Stage::Waiting=>0,Stage::Preparing=>1,Stage::Engaged=>2,
        Stage::Succeeded=>3,Stage::Failed|Stage::Retired=>4};
    let mut row=format!("V|{owner}|{}|1|{stage}|{}|200|{}|0|{}|{}|0",
        scene.id.event,(scene.held()*100.).round().min(200.)as u32,
        u8::from(s.inside),s.leg.saturating_sub(1),s.route.len().saturating_sub(1));
    if let Some(object)=s.objects.first() {
        let [x,y,_]=object.position;
        if x.is_finite()&&y.is_finite()&&x.abs()<1e8&&y.abs()<1e8 {
            row.push_str(&format!("|{x:.3}|{y:.3}"));
        }
    }
    row.push('\n');Some(row)
}
pub fn finish(r:&Runtime,g:u64) {
    if !ENABLED.load(Ordering::Acquire)||TICKING.swap(true,Ordering::AcqRel){return}
    struct Guard;
    impl Drop for Guard{fn drop(&mut self){TICKING.store(false,Ordering::Release)}}
    let _guard=Guard;
    if let Err(e)=tick(r,g).and_then(|_|super::solo_gadgets::tick(r,g)){
        r.log.warn(&format!("Solo escort stopped: {e}"));
        ENABLED.store(false,Ordering::Release);
        reset();
    }
}
fn tick(r:&Runtime,g:u64)->Result<(),String> {
    let Some(p)=r.adapter.solo_run(&r.local,g)? else {reset();return Ok(())};
    let Some(a)=p.actors.first() else {return Ok(())};
    let now=p.clock;
    #[cfg(feature="test-hooks")]
    let preview=std::env::var("AZSX_TEST_ESCORT_PREVIEW").as_deref()==Ok("1");
    #[cfg(not(feature="test-hooks"))]
    let preview=false;
    let first_boss_at={
        let mut budget=OPPORTUNITIES.lock().map_err(|_|"Escort budget unavailable")?;
        #[cfg(feature="test-hooks")]
        if preview&&now>=25.&&budget.encounter(g,a.guid,now){
            r.log.info(&format!("Escort preview gate: now={now:.2}"));
        }
        if !budget.observe(g,a.guid,now){return Err("Invalid escort clock".into())}
        budget.first_boss_at(g,a.guid)
    };
    let old={
        let mut lock=STATE.lock().map_err(|_|"Escort state unavailable")?;
        if lock.as_ref().is_none_or(|s|s.game!=g||s.owner!=a.guid||now<s.clock){
            let old=lock.take();
            let s=State{game:g,owner:a.guid,clock:now,next:f32::INFINITY,
                rng:g^a.guid^std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_nanos()as u64,
                serial:0,scene:None,objects:Vec::new(),route:Vec::new(),
                leg:0,next_move:now,created:now,ended:None,arrived:false,inside:false};
            *lock=Some(s);old
        }else{None}
    };
    if let Some(old)=old{cleanup(r,old)}
    if a.hp<=0.||a.phase==Phase::Inactive{reset();return Ok(())}
    if a.phase!=Phase::Playing{
        if let Some(s)=STATE.lock().map_err(|_|"Escort state unavailable")?.as_mut(){
            s.clock=now;
            if let Some(scene)=s.scene.as_mut(){
                scene.observe(Observation{id:scene.id,now,playing:false,solo:true,
                    player_alive:true,present:false,unit_alive:true,arrived:false,enemy_defeated:false});
            }
        }
        return Ok(())
    }
    let work={
        let mut lock=STATE.lock().map_err(|_|"Escort state unavailable")?;
        let s=lock.as_mut().ok_or("Escort state missing")?;
        s.clock=now;
        if s.next.is_infinite(){if let Some(boss_at)=first_boss_at{s.next=if preview{now+2.}else{boss_at+30.+s.draw(61)as f32};}}
        if s.ended.is_some_and(|end|now-end>=5.){s.scene=None;s.ended=None}
        let allowed=OPPORTUNITIES.lock().map_err(|_|"Escort budget unavailable")?.allows(Kind::ChickenEscort);
        if s.scene.is_none()&&allowed&&now>=s.next{
            s.next=if preview{now+3.}else{now+45.+s.draw(31)as f32};
            s.serial=s.serial.checked_add(1).ok_or("Escort identity exhausted")?;
            Some((s.serial,(0..32).map(|_|(s.draw(u32::MAX),s.draw(u32::MAX))).collect::<Vec<_>>()))
        }else{None}
    };
    if let Some((serial,draws))=work{
        let area=r.events_area(g)?;
        let segments=if preview{8}else{azsx_gameplay::solo_events::escort_segments(area.radius)
            .ok_or("Invalid escort map radius")?};
        let mut selected=None;
        if preview {
            // Diagnostic framing only: leave the player outside the wreckage
            // so screenshots show the chicken and its individual parts.
            for offset in [[0.,5.],[5.,0.],[-5.,0.],[0.,-5.],[0.,0.]] {
                let point=[a.position[0]+offset[0],a.position[1]+offset[1]];
                if let Some(site)=unsafe{r.adapter.event_ground_sample(
                    &r.local,r.local.base as u64,g,point,5.
                )}?{
                    for heading in (0..360).step_by(45){
                        if let Some(route)=unsafe{r.adapter.escort_route(
                            &r.local,r.local.base as u64,g,site.center,heading,segments
                        )}?{
                            selected=Some((site.center,route));break
                        }
                    }
                }
                if selected.is_some(){break}
            }
        }
        // Four independent ground sites, with eight headings each. The native
        // route verifier remains authoritative; failed sites consume no quota.
        for group in draws.chunks(8){
            if selected.is_some(){break}
            if let Some(point)=super::world_events::place(r,g,area,5.,group)?{
                for (angle,_) in group{
                    if let Some(route)=unsafe{r.adapter.escort_route(&r.local,r.local.base as u64,g,point,*angle,segments)}?{
                        selected=Some((point,route));break
                    }
                }
            }
            if selected.is_some(){break}
        }
        if preview{r.log.info(&format!("Escort preview placement: now={now:.2} selected={}",selected.is_some()));}
        if let Some((point,route))=selected{
                if !OPPORTUNITIES.lock().map_err(|_|"Escort budget unavailable")?.reserve_escort(g,a.guid){
                    return Ok(())
                }
                let access=io::Access{local:&r.local,adapter:r.adapter};
                let chicken=unsafe{r.adapter.escort_chicken(&access,r.local.base as u64,g,point)}?;
                let portal=match unsafe{r.adapter.event_object(&access,r.local.base as u64,g,10,*route.last().ok_or("Missing extraction")?)}{
                    Ok(portal)=>portal,Err(e)=>{
                        let _=unsafe{r.adapter.remove_event_object(&access,r.local.base as u64,g,chicken)};
                        return Err(e)
                    }
                };
                let mut objects=vec![chicken,portal];
                if let Some(sites)=azsx_gameplay::solo_events::escort_crash_sites(&route) {
                    for (index,xy) in sites.into_iter().enumerate() {
                        let visual=11+index as u8;
                        let ground=unsafe{r.adapter.event_ground_sample(
                            &r.local,r.local.base as u64,g,xy,0.45
                        )};
                        let Ok(Some(site))=ground else{continue};
                        if site.high-site.low>1.5||(site.center[2]-point[2]).abs()>1.5{continue}
                        match unsafe{r.adapter.event_object(
                            &access,r.local.base as u64,g,visual,site.center
                        )}{
                            Ok(object)=>objects.push(object),
                            Err(e)=>r.log.warn(&format!("Escort scenery skipped: {e}")),
                        }
                    }
                }
                let scene=Scene::new(Identity{run:a.guid,owner:a.guid,event:serial},Kind::ChickenEscort,
                    now,Proofs{assets:true,cleanup:true,navigation:true,attribution:true,gadget:true,
                        ..Proofs::default()},true).ok_or("Escort policy rejected")?;
                let mut pending=Some((scene,objects,route));
                {
                    let mut lock=STATE.lock().map_err(|_|"Escort state unavailable")?;
                    if let Some(s)=lock.as_mut().filter(|s|s.game==g&&s.owner==a.guid&&s.serial==serial&&s.scene.is_none()){
                        let (scene,objects,route)=pending.take().ok_or("Escort scene lost")?;
                        s.scene=Some(scene);s.objects=objects;s.route=route;s.leg=1;
                        s.created=now;s.next_move=now;s.arrived=false;
                    }
                }
                if let Some((_,objects,_))=pending{
                    for object in objects{let _=unsafe{r.adapter.remove_event_object(&access,r.local.base as u64,g,object)};}
                    return Ok(())
                }
                notice(r,g,"A damaged Mechanical Chicken awaits repairs. Escort it to extraction for an engineering gadget.");
        }
    }
    let (motion,done,reward,objects)={
        let mut lock=STATE.lock().map_err(|_|"Escort state unavailable")?;
        let s=lock.as_mut().ok_or("Escort state missing")?;
        let Some(scene)=s.scene.as_mut()else{return Ok(())};
        let focus=s.objects.first().map_or([0.;3],|v|v.position);
        s.inside=(a.position[0]-focus[0]).hypot(a.position[1]-focus[1])<=6.;
        let stage=scene.observe(Observation{id:scene.id,now,playing:true,solo:true,
            player_alive:true,present:s.inside,unit_alive:true,arrived:s.arrived,enemy_defeated:false});
        if now-s.created>240.&&!matches!(stage,Stage::Succeeded|Stage::Failed|Stage::Retired){scene.retire()}
        let terminal=matches!(scene.stage(),Stage::Succeeded|Stage::Failed|Stage::Retired);
        let done=terminal&&s.ended.is_none();
        let reward=scene.take_reward();
        let objects=if done{s.ended=Some(now);std::mem::take(&mut s.objects)}else{Vec::new()};
        let motion=if !terminal&&scene.stage()==Stage::Engaged&&s.inside&&now>=s.next_move{
            s.next_move=now+0.5;
            if s.leg>=s.route.len(){s.arrived=true;None}
            else{let object=*s.objects.first().ok_or("Missing chicken")?;
                let to=s.route[s.leg];s.leg+=1;s.objects[0].position=to;Some((object,to))}
        }else{None};
        (motion,done,reward,objects)
    };
    let access=io::Access{local:&r.local,adapter:r.adapter};
    if let Some((object,to))=motion{
        unsafe{r.adapter.escort_move(&access,r.local.base as u64,g,object,to,500)}?;
    }
    if done{for object in objects{
        unsafe{r.adapter.remove_event_object(&access,r.local.base as u64,g,object)}?;
    }}
    if reward.is_some(){
        let gadget=super::solo_gadgets::award(r,g,a.guid,None)?;
        notice(r,g,"Extraction complete. Your engineering gadget is equipped for this run.");
        r.log.info(&format!("solo escort rewarded: kind={gadget:?} once=true"));
    }else if done{notice(r,g,"The Mechanical Chicken escort has ended.")}
    Ok(())
}
