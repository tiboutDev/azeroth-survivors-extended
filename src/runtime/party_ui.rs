//! Cosmetic builds over native addon lines. No simulation, socket or CVar state.
use super::*;
use asx_game::official::PartyBuild;
use std::collections::BTreeMap;
#[derive(Default)]
struct State {
    epoch: u64,
    clock: f32,
    ids: Vec<(u32, u64)>,
    names: BTreeMap<u64, String>,
    builds: BTreeMap<u64, PartyBuild>,
    encoded: BTreeMap<u64, (String, String)>,
    enchantments: BTreeMap<(u64,u32),String>,
    drops: Vec<NativeDrop>,
    drop_sequence: u64,
}
#[derive(Clone)]
struct NativeDrop { owner:u64, sequence:u64, name:String, icon:String, rarity:u8, is_new:bool }
impl NativeDrop {
 fn parse(value:&serde_json::Value,owner:u64,sequence:u64)->Option<Self>{
  if value["type"]!="drop"||owner==0{return None}
  let name=value["name"].as_str().filter(|v|!v.is_empty()&&v.len()<=128&&!v.chars().any(char::is_control))?;
  let icon=value["icon"].as_str().filter(|v|v.starts_with("INV_")&&v.len()<=96&&v.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'))?;
  let rarity=value["rarity"].as_u64().filter(|v|*v<=5)? as u8;
  Some(Self{owner,sequence,name:name.into(),icon:icon.into(),rarity,is_new:value["new"].as_bool()?})
 }
}
fn drop_wire(state:&State,own:u64)->String {
 let mut out=String::new();
 for d in state.drops.iter().filter(|d|d.owner==own){out.push_str(&format!("Q|{own}|{}|{}|{}|{}|{}\n",d.sequence,d.rarity,u8::from(d.is_new),hex(d.name.as_bytes()),hex(d.icon.as_bytes())));}
 out
}
/// Observe the already-local native UI receipt, with no gameplay calls or replay.
pub fn native_drop(value:&serde_json::Value){
 if value["type"]!="drop"{return}
 let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};
 let Some(own)=super::ui_context::participants(r).first().copied()else{return};
 if let Ok(mut state)=STATE.lock(){if let Some(s)=state.as_mut(){
  if !s.ids.iter().any(|(_,id)|*id==own){return}
  let Some(sequence)=s.drop_sequence.checked_add(1).filter(|n|*n<=1_000_000_000)else{return};
  if let Some(drop)=NativeDrop::parse(value,own,sequence){s.drop_sequence=sequence;if s.drops.len()>=16{s.drops.remove(0);}s.drops.push(drop);}
 }}

}
static STATE: std::sync::Mutex<Option<State>> = std::sync::Mutex::new(None);
static HEARTBEAT: OnceLock<std::time::Instant> = OnceLock::new();
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 15) as usize] as char);
    }
    out
}
// Reuse already captured native item text; never recapture a local spell build.
fn item_text(icon:&asx_game::official::PartyIcon,own:u64,notes:&BTreeMap<(u64,u32),String>)->String{
    match notes.get(&(own,icon.id)){Some(note)=>format!("{}\n\n{note}",icon.text),None=>icon.text.clone()}
}
#[cfg(test)]
fn local_equipment(build: &PartyBuild, own: u64) -> String {local_equipment_notes(build,own,&BTreeMap::new())}
fn local_equipment_notes(build: &PartyBuild, own: u64, notes:&BTreeMap<(u64,u32),String>) -> String {
    if build.items.len() > 5 { return String::new(); }
    let mut text = format!("L|{own}|{}\n", build.items.len());
    for (index, icon) in build.items.iter().enumerate() {
        text.push_str(&format!("E|{own}|{}|{}|{}|{}|{}{}\n", index + 1, icon.rank,
            hex(icon.name.as_bytes()), hex(icon.icon.as_bytes()), hex(item_text(icon,own,notes).as_bytes()),
            icon.category.map(|c| format!("|{c}")).unwrap_or_default()));
    }
    text
}
fn wire(state: &mut State, own: u64, heartbeat: u64) -> Option<String> {wire_extra(state,own,heartbeat,"")}
fn wire_extra(state: &mut State, own: u64, heartbeat: u64, extra:&str) -> Option<String> {
    if !state.ids.iter().any(|(_, id)| *id == own) {
        return None;
    }
    let mut text = format!("1|{}|{own}|{heartbeat}\n", state.epoch);
    for (slot, id) in &state.ids {
        let build=state.builds.get(id);
        // The exact native recipient is known independently of cosmetic roster
        // delivery. Its internal label is never used to bind a remote unit.
        let name=state.names.get(id).map(String::as_str).or_else(||(*id==own).then_some("Player"));
        let Some(name)=name else{continue};
        if *id!=own&&build.is_none(){continue}
        text.push_str(&format!("M|{id}|{slot}|{}\n", hex(name.as_bytes())));
        if let Some(b)=blessings::view(*slot,*id){
            let best=blessings::strongest(b.category);
            text.push_str(&format!("B|{id}|{}|{}|{}|{}|{}\n",b.boss,b.category,b.rarity,b.amount,(best*10000.0).round() as u32));
            if let Some(receipt)=blessings::receipt(*slot,*id){text.pop();text.push_str(&format!("|{receipt}\n"));}
        }
        if let Some(row)=super::crystals::member_wire((*slot,*id)){text.push_str(&row);}
        let fatigue = fatigue::view(*slot, *id);
        text.push_str(&format!("F|{id}|{}|{}\n", fatigue.stacks, fatigue.remaining.ceil() as u32));
        // Only the five local items are needed by ASXUI; local spells remain omitted.
        if *id == own { if let Some(build)=build{text.push_str(&local_equipment_notes(build, own,&state.enchantments));}text.push_str(&drop_wire(state,own)); continue; }
        let Some(build)=build else{continue};
        for (kind, icons) in [("S", &build.spells), ("I", &build.items)] {
            for icon in icons {
                text.push_str(&format!(
                    "{kind}|{id}|{}|{}|{}|{}|{}",
                    icon.id,
                    icon.rank,
                    hex(icon.name.as_bytes()),
                    hex(icon.icon.as_bytes()),
                    hex(if kind=="I"{item_text(icon,*id,&state.enchantments)}else{icon.text.clone()}.as_bytes())
                ));
                if kind == "S" {
                    let tip = icon.spell_tip.clone().unwrap_or_default();
                    for value in [&tip.rank, &tip.range, &tip.cast, &tip.cooldown] {
                        text.push('|');
                        text.push_str(&hex(value.as_bytes()));
                    }
                }
                text.push('\n');
            }
        }
    }
    text.push_str(extra);
    if let Some((old, packed)) = state.encoded.get(&own) {
        if *old == text {
            return Some(packed.clone());
        }
    }
    let packed = hex(&asx_gameplay::party_text::pack(text.as_bytes())?);
    state.encoded.insert(own, (text, packed.clone()));
    Some(packed)
}
pub fn roster(value: &serde_json::Value) {
    if value["type"] != "party" {
        return;
    }
    let mut names = BTreeMap::new();
    let valid = value["members"]
        .as_array()
        .filter(|a| a.len() <= asx_game::official::NATIVE_PLAYERS)
        .is_some_and(|rows| {
            rows.iter().all(|row| {
                let id = row["id"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .filter(|id| *id > 0);
                let name = row["name"].as_str().filter(|s| {
                    !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
                });
                match (id, name) {
                    (Some(id), Some(name)) => {
                        names.insert(id, name.into()).is_none()
                    }
                    _ => false,
                }
            })
        });
    #[cfg(feature="test-hooks")]
    if std::env::var("ASX_TEST_HUD_OWNERS").as_deref()==Ok("1"){if let Some(r)=RUNTIME.get(){static SAMPLES:AtomicUsize=AtomicUsize::new(0);if SAMPLES.fetch_add(1,Ordering::Relaxed)<4{r.log.info(&format!("party roster diagnostic: valid={valid} identities={:?}",names.keys().collect::<Vec<_>>()));}}}
    if let Ok(mut state) = STATE.lock() {
        let s = state.get_or_insert_with(State::default);
        s.names = if valid { names } else { BTreeMap::new() };
    }
}
/// Resolve pause participants by the current native actor GUID, never by row
/// order, class or a cached build. No native calls while borrowing this cache.
pub fn pause_view(
    reason: Option<asx_gameplay::official::PauseReason>,
    phase: asx_gameplay::Phase,
    identities: &[(u32, u64)],
) -> asx_gameplay::official::PausePresentation {
    let state = if reason.is_some() { STATE.lock().ok() } else { None };
    asx_gameplay::official::pause_presentation(reason, phase, |slot| {
        let guid = identities.iter().find(|(id, _)| *id == slot)?.1;
        state.as_ref()?.as_ref()?.names.get(&guid).cloned()
    })
}
pub fn reset() {
    if let Ok(mut state) = STATE.lock() {
        if let Some(s) = state.as_mut() {
            s.epoch = s.epoch.wrapping_add(1);
            s.ids.clear();
            s.builds.clear();s.enchantments.clear();
            s.encoded.clear();
            s.drops.clear();
        }
    }
}
pub fn capture(r: &Runtime, actor: u32, value: &serde_json::Value) {
    let Some(party) = r.extension_party() else {
        reset();
        return;
    };
    let Some(owner) = party.actors.iter().find(|a| a.slot == actor) else {
        return;
    };
    let parsed=r.adapter.party_build(value);
    #[cfg(feature="test-hooks")]
    if let Err(error)=&parsed{static SAMPLES:AtomicUsize=AtomicUsize::new(0);if std::env::var("ASX_TEST_HUD_OWNERS").as_deref()==Ok("1")&&SAMPLES.fetch_add(1,Ordering::Relaxed)<4{r.log.warn(&format!("party build diagnostic rejected: actor={actor} reason={error} value={value}"));}}
    let Ok(build) = parsed else {
        if let Ok(mut state) = STATE.lock() {
            if let Some(state) = state.as_mut() {
                state.builds.remove(&owner.guid);
            }
        }
        return;
    };
    let mut ids = party
        .actors
        .iter()
        .map(|a| (a.slot, a.guid))
        .collect::<Vec<_>>();
    ids.sort_unstable();
    if !party.clock.is_finite() || party.clock < 0.0 || ids.iter().any(|(_, id)| *id == 0) {
        return;
    }
    if let Ok(mut state) = STATE.lock() {
        let s = state.get_or_insert_with(State::default);
        if s.ids != ids || party.clock + 0.5 < s.clock {
            s.epoch = s.epoch.wrapping_add(1);
            s.ids = ids;
            s.builds.clear();s.enchantments.clear();
            s.encoded.clear();
            s.drops.clear();
        }
        let changed = s.builds.get(&owner.guid) != Some(&build);
        s.clock = party.clock;
        s.builds.insert(owner.guid, build);
        #[cfg(feature = "test-hooks")]
        if changed && std::env::var("ASX_TEST_HUD_OWNERS").as_deref() == Ok("1") && {static SAMPLES:AtomicUsize=AtomicUsize::new(0);SAMPLES.fetch_add(1,Ordering::Relaxed)<16} {
            r.log.info(&format!(
                "official party build: actor={} guid={} abilities={} jewelry={}",
                actor, owner.guid, value["abilities"], value["jewelry"]
            ));
        }
        #[cfg(not(feature = "test-hooks"))]
        let _ = changed;
    }
}
type Addons = unsafe extern "system" fn(*mut u8, *mut u8, usize) -> *mut u8;
unsafe extern "system" fn addons(output: *mut u8, game: *mut u8, native_context: usize) -> *mut u8 {
    let original: Addons = std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
    // Both native modes construct addon lines. Solo does not enter the coop
    // server/HUD serializer. This restored presentation boundary owns its cache.
    if let Some(r)=RUNTIME.get().filter(|r|r.rules){
        lifecycle::refresh_solo(r,game as u64);
        if r.solo.lock().ok().is_some_and(|p|p.is_some()){blessings::finish(r,game as u64);super::supplies::finish(r,game as u64);super::rituals::finish(r,game as u64);super::crystals::finish(r,game as u64);}
    }
    let result = original(output, game, native_context);
    #[cfg(feature="test-hooks")]
    if let Some(r)=RUNTIME.get().filter(|r|r.rules){
        if r.solo.lock().ok().is_some_and(|p|p.is_some()) && (std::env::var("AZCOOP_TEST_EXTENSIONS").as_deref()==Ok("1") || enchantments::fixture_enabled() || gems::fixture_enabled()){fixture::tick(r,(game as u64)-0x10);}
    }
    #[cfg(feature = "test-hooks")]
    let _timing = performance::start(1);
    if let Some(r) = RUNTIME.get().filter(|r| r.rules) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Ok(Some((slot,_))) = r.adapter.run_source(&r.local, game as u64) else {
                return;
            };
            #[cfg(feature="test-hooks")]
            if std::env::var("ASX_TEST_HUD_OWNERS").as_deref()==Ok("1"){static SAMPLE:AtomicUsize=AtomicUsize::new(0);if {let n=SAMPLE.fetch_add(1,Ordering::Relaxed);n<800&&n%100==0}{if let Ok(s)=STATE.lock(){if let Some(s)=s.as_ref(){r.log.info(&format!("party cache diagnostic: ids={:?} names={:?} builds={:?}",s.ids,s.names,s.builds.keys().collect::<Vec<_>>()));}}}}
            refresh_enchantments(r,game as u64);
            let solo=r.solo.lock().ok().and_then(|p|p.clone());
            let payload = if let Some(p)=solo {
                p.actors.first().map(|actor|{
                    let heartbeat=HEARTBEAT.get_or_init(std::time::Instant::now).elapsed().as_secs();
                    let epoch=STATE.lock().ok().map(|mut state|{
                        let s=state.get_or_insert_with(State::default);
                        if s.ids!=[(0,actor.guid)] || p.clock+0.5<s.clock{s.epoch=s.epoch.wrapping_add(1);s.ids=vec![(0,actor.guid)];s.builds.clear();s.enchantments.clear();s.encoded.clear();s.drops.clear();}
                        s.clock=p.clock;s.epoch
                    }).unwrap_or(0);
                    let mut text=format!("1|{epoch}|{}|{heartbeat}\nM|{}|0|{}\n",actor.guid,actor.guid,hex(b"Player"));
                    if let Some(items)=STATE.lock().ok().and_then(|state|state.as_ref().and_then(|s|s.builds.get(&actor.guid).map(|b|local_equipment_notes(b,actor.guid,&s.enchantments)))){text.push_str(&items);}
                    if let Some(drops)=STATE.lock().ok().and_then(|state|state.as_ref().map(|s|drop_wire(s,actor.guid))){text.push_str(&drops);}
                    if let Some(row)=super::crystals::wire(game as u64,(0,actor.guid)){text.push_str(&row);}
                    if let Some(b)=blessings::view(0,actor.guid){text.push_str(&format!("B|{}|{}|{}|{}|{}|{}\n",actor.guid,b.boss,b.category,b.rarity,b.amount,b.amount));if let Some(receipt)=blessings::receipt(0,actor.guid){text.pop();text.push_str(&format!("|{receipt}\n"));}}
                    if let Some((event,held,inside))=super::rituals::view(game as u64,actor.guid){text.push_str(&format!("R|{}|{event}|{held}|{}|{}\n",actor.guid,u8::from(inside),u8::from(inside)));}
                    for(c,remaining,index)in super::supplies::buffs(game as u64,actor.guid){if index<32&&remaining>0.{text.push_str(&format!("C|{}|{}|{}|{}|{index}|{}|{}|{}\n",actor.guid,c.family as u8,c.tier,remaining.ceil()as u32,hex(c.name().as_bytes()),hex(c.icon().as_bytes()),hex(c.text().as_bytes())));}}
                    if let Some(d)=super::supplies::discovery(game as u64,actor.guid){text.push_str(&format!("D|{}|{}|{}|{}|{}|{}|{}|{}\n",actor.guid,d.event,d.reward.rarity(),d.gold,hex(d.reward.name().as_bytes()),hex(d.reward.icon().as_bytes()),hex(d.reward.text().as_bytes()),d.reward.quality()));}
                    if let Some(map)=super::ui_context::minimap(r,actor.guid){
                        text.push_str(&format!("A|{}|{}|{}",actor.guid,(map.center[0]*100.).round()as i32,(map.center[1]*100.).round()as i32));
                        for radius in map.radii{text.push_str(&format!("|{}",(f64::from(radius)*1_000_000.).round()as u64));}
                        text.push_str(&format!("|{}|{}\n",(map.ping[0]*100.).round()as i32,(map.ping[1]*100.).round()as i32));
                        for marker in super::world_events::markers(game as u64){text.push_str(&format!("P|{}|{}|{}|{}|{}|{}\n",actor.guid,marker.id,marker.kind,(marker.position[0]*100.).round()as i32,(marker.position[1]*100.).round()as i32,marker.remaining.ceil()as u32));}
                    }
                    hex(&asx_gameplay::party_text::pack(text.as_bytes()).unwrap_or_default())
                })
            }else{
              let event_text=event_text(r,game as u64,slot);
              STATE.lock().ok().and_then(|mut state| {
                let s = state.as_mut()?;
                let own = s
                    .ids
                    .iter()
                    .find(|(index, _)| *index == slot)
                    .map(|(_, id)| *id)?;
                wire_extra(
                    s,
                    own,
                    HEARTBEAT
                        .get_or_init(std::time::Instant::now)
                        .elapsed()
                        .as_secs(),
                    &event_text,
                )
            })};
            if let Some(payload) = payload {
                let io = io::Access {
                    local: &r.local,
                    adapter: r.adapter,
                };
                if let Err(error) = r.adapter.decorate_party_addon(&io, output as u64, &payload) {
                    r.log
                        .warn(&format!("Official party addon rejected: {error}"));
                }
            }
        }));
    }
    result
}
fn refresh_enchantments(r:&Runtime,game:u64){
 let Some(party)=r.extension_party()else{return};
 let builds=STATE.lock().ok().and_then(|s|s.as_ref().map(|s|s.builds.clone())).unwrap_or_default();let mut notes=BTreeMap::new();
 for actor in party.actors{if let Some(build)=builds.get(&actor.guid){for icon in &build.items{
  if icon.category!=Some("ring"){continue}
  if let Ok(Some(ring))=r.adapter.presented_ring(&r.local,game,actor.slot,actor.guid,&icon.icon,icon.rank,r.local.base as u64){
   if let Some(e)=enchantments::retained(actor.slot,actor.guid,ring,icon.rank,game){let note=format!("Enchantment: {}\n{}",e.name(),e.text());if icon.text.len()+note.len()+2<=2048{notes.insert((actor.guid,icon.id),note);}}
  }
 }}}
 if let Ok(mut state)=STATE.lock(){if let Some(s)=state.as_mut(){s.enchantments=notes;}}
}
// Keep each independently known value. An explicit unavailable cell prevents Lua
// from silently displaying the recipient's personal stat as a shared multiplier.
fn economy_wire(owner:u64,xp:Option<f32>,gold:Option<f32>)->String{
 fn cell(value:Option<f32>)->String{value.filter(|v|v.is_finite()&&(0.0..=10000.0).contains(v)).map_or_else(||"-".into(),|v|((f64::from(v)*1_000_000.).round()as u64).to_string())}
 format!("O|{owner}|{}|{}\n",cell(xp),cell(gold))
}
#[cfg(test)]mod economy_projection_tests{
 use super::*;
 #[test]fn missing_or_invalid_gold_never_discards_current_shared_xp(){
  assert_eq!(economy_wire(11,Some(1.35),None),"O|11|1350000|-\n");
  assert_eq!(economy_wire(11,None,Some(1.6)),"O|11|-|1600000\n");
  assert_eq!(economy_wire(11,Some(f32::NAN),Some(-1.)),"O|11|-|-\n");
  assert_eq!(economy_wire(11,Some(0.),Some(1.)),"O|11|0|1000000\n");
 }
}
fn event_text(r:&Runtime,game:u64,slot:u32)->String{
 let Some(party)=r.extension_party()else{return String::new()};
 let Some(actor)=party.actors.iter().find(|a|a.slot==slot)else{return String::new()};let owner=actor.guid;let mut text=String::new();
 #[cfg(feature="test-hooks")]
 if std::env::var("ASX_TEST_COOP_UI").as_deref()==Ok("1") {static SAMPLES:AtomicUsize=AtomicUsize::new(0);let n=SAMPLES.fetch_add(1,Ordering::Relaxed);if n<3000&&n%100==0{r.log.info(&format!("coop economy diagnostic: slot={slot} owner={owner} xp={:?} gold={:?} actors={:?}",super::xp_hooks::services::effective_multiplier(r),super::wallet::effective_multiplier(r,game,actor),party.actors.iter().map(|a|(a.slot,a.xp_multiplier,a.gold_multiplier)).collect::<Vec<_>>()));}}
 text.push_str(&economy_wire(owner,super::xp_hooks::services::effective_multiplier(r),super::wallet::effective_multiplier(r,game,actor)));
 if let Some((event,held,inside))=super::world_events::coop::view(game,owner){text.push_str(&format!("R|{owner}|{event}|{held}|{}|{}\n",u8::from(inside),u8::from(super::world_events::coop::all_inside(game))));}
 for(c,remaining,index)in super::supplies::buffs(game,owner){if index<32&&remaining>0.{text.push_str(&format!("C|{owner}|{}|{}|{}|{index}|{}|{}|{}\n",c.family as u8,c.tier,remaining.ceil()as u32,hex(c.name().as_bytes()),hex(c.icon().as_bytes()),hex(c.text().as_bytes())));}}
 if let Some(d)=super::supplies::discovery(game,owner){text.push_str(&format!("D|{owner}|{}|{}|{}|{}|{}|{}|{}\n",d.event,d.reward.rarity(),d.gold,hex(d.reward.name().as_bytes()),hex(d.reward.icon().as_bytes()),hex(d.reward.text().as_bytes()),d.reward.quality()));}
 if let Some(mut map)=party.actors.iter().find_map(|a|super::ui_context::minimap(r,a.guid)){
  if slot!=0{map.center=[actor.position[0],actor.position[1]];map.ping=[0.,0.];}
  text.push_str(&format!("A|{owner}|{}|{}",(map.center[0]*100.).round()as i32,(map.center[1]*100.).round()as i32));
  for radius in map.radii{text.push_str(&format!("|{}",(f64::from(radius)*1_000_000.).round()as u64));}text.push_str(&format!("|{}|{}\n",(map.ping[0]*100.).round()as i32,(map.ping[1]*100.).round()as i32));
  for marker in super::world_events::coop::markers(game,owner){text.push_str(&format!("P|{owner}|{}|{}|{}|{}|{}\n",marker.id,marker.kind,(marker.position[0]*100.).round()as i32,(marker.position[1]*100.).round()as i32,marker.remaining.ceil()as u32));}
 }text
}
pub fn hook(adapter: Adapter) -> crate::install::HookSpec {
    crate::install::HookSpec {
        name: "official-party-addon",
        target: local_base() + adapter.party_addon_entry().rva as usize,
        detour: addons as *const () as usize,
        trampoline: &ORIGINAL,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_drop_metadata_is_bounded_and_only_sent_to_its_local_owner(){
        let value=serde_json::json!({"type":"drop","name":"Recruit's Helm","icon":"INV_Helmet_02","rarity":0,"new":true});
        let drop=NativeDrop::parse(&value,11,1).unwrap();
        let state=State{drops:vec![drop],..Default::default()};
        assert!(drop_wire(&state,11).starts_with("Q|11|1|0|1|"));assert!(drop_wire(&state,22).is_empty());
        assert!(NativeDrop::parse(&value,0,1).is_none());
        for (key,bad)in [("name",serde_json::json!("bad\nname")),("icon",serde_json::json!("../foreign")),("rarity",serde_json::json!(6)),("new",serde_json::json!(null))]{let mut v=value.clone();v[key]=bad;assert!(NativeDrop::parse(&v,11,1).is_none());}
    }
    unsafe extern "system" fn native(
        out: *mut u8,
        game: *mut u8,
        native_context: usize,
    ) -> *mut u8 {
        assert_eq!(out as usize, 0x10000);
        assert_eq!(game as usize, 0x20000);
        assert_eq!(native_context, 0x3456);
        out
    }
    #[test]
    fn typed_addon_wrapper_forwards_native_output_context_and_native_context() {
        ORIGINAL.store(native as *const () as usize, Ordering::Release);
        assert_eq!(
            unsafe { addons(0x10000 as *mut u8, 0x20000 as *mut u8, 0x3456) } as usize,
            0x10000
        );
    }
    unsafe extern "system" fn native_spell(
        out: *mut u8,
        ability: u64,
        stats: u64,
        kind: u8,
        perks: u64,
    ) -> *mut u8 {
        assert_eq!(
            (out as usize, ability, stats, kind, perks),
            (0x10000, 0x20000, 0x30490, 3, 0x40000)
        );
        out
    }
    #[test]
    fn typed_spell_wrapper_preserves_all_five_native_arguments() {
        SPELL_ORIGINAL.store(native_spell as *const () as usize, Ordering::Release);
        assert_eq!(
            unsafe { spell_presentation(0x10000 as *mut u8, 0x20000, 0x30490, 3, 0x40000) }
                as usize,
            0x10000
        );
    }
    #[test]
    fn recipients_get_remote_builds_local_metadata_and_fresh_heartbeats() {
        use asx_game::official::PartyIcon;
        let mut state = State { epoch: 7, ids: vec![(0, 11), (1, 22)], ..Default::default() };
        for id in [11, 22] {
            state.names.insert(id, format!("Owner {id}"));
            let icon = PartyIcon { category:None, id: 42, rank: 1, name: "Spell".into(), icon: "Spell_Test".into(), text: "Owner tooltip".into(), spell_tip: None };
            state.builds.insert(id, PartyBuild { spells: vec![icon.clone()], items: vec![icon] });
        }
        for own in [11, 22] {
            let other = if own == 11 { 22 } else { 11 };
            let first = wire(&mut state, own, 1).unwrap();
            let text = &state.encoded[&own].0;
            assert!(text.starts_with(&format!("1|7|{own}|1\n")));
            assert!(text.contains(&format!("M|{own}|")));
            assert!(text.contains(&format!("F|{own}|")));
            for kind in ["S", "I"] {
                assert!(!text.contains(&format!("{kind}|{own}|")));
                assert!(text.contains(&format!("{kind}|{other}|42|1|")));
            }
            assert_eq!(first, wire(&mut state, own, 1).unwrap());
            // No gameplay clock advance: a long common pause still refreshes transport.
            let later = wire(&mut state, own, 600).unwrap();
            assert_ne!(first, later);
            assert!(state.encoded[&own].0.starts_with(&format!("1|7|{own}|600\n")));
        }
        assert_eq!(state.builds.len(), 2);
    }
    #[test]
    fn local_feature_envelope_does_not_require_cosmetic_build_or_roster_delivery(){
        let _guard=crate::test_support::TEST_LOCK.lock().unwrap();
        let mut state=State{epoch:1,ids:vec![(0,11),(1,22)],..Default::default()};
        assert!(wire_extra(&mut state,22,1,"O|22|1350000|1600000\n").is_some());
        let text=&state.encoded[&22].0;assert!(text.contains("M|22|1|"));assert!(text.contains("O|22|1350000|1600000"));
        assert!(!text.contains("M|11|")&&!text.contains("E|22|")&&!text.contains("L|22|"));
    }
    #[test]
    fn item_annotations_preserve_owner_text_and_do_not_cross_item_or_actor_identity(){
        let icon=asx_game::official::PartyIcon{category:Some("ring"),id:3,rank:4,name:"Owner ring".into(),icon:"INV_Ring".into(),text:"Exact native owner effect".into(),spell_tip:None};
        let notes=BTreeMap::from([((11,3),"Enchantment: Lesser Might\n+9% damage".into())]);
        assert_eq!(item_text(&icon,11,&notes),"Exact native owner effect\n\nEnchantment: Lesser Might\n+9% damage");
        assert_eq!(item_text(&icon,22,&notes),icon.text);let other=asx_game::official::PartyIcon{id:4,..icon.clone()};assert_eq!(item_text(&other,11,&notes),icon.text);
    }
    #[test]
    fn local_equipment_is_bounded_native_text_and_explicitly_empty() {
        use asx_game::official::PartyIcon;
        let icon=PartyIcon { category:None, id:42, rank:2, name:"Native|Ring".into(), icon:"INV_Test".into(), text:"Native text\n+10 damage".into(), spell_tip:None };
        let mut build=PartyBuild { spells:vec![icon.clone()], items:vec![icon.clone()] };
        let text=local_equipment(&build,11);
        assert!(text.starts_with("L|11|1\nE|11|1|2|"));
        assert!(text.contains(&hex(icon.text.as_bytes())));
        assert!(!text.contains("S|"));
        build.items.clear();assert_eq!(local_equipment(&build,11),"L|11|0\n");
        build.items=vec![icon;6];assert!(local_equipment(&build,11).is_empty());
    }
    #[test]
    fn wire_uses_actual_owner_and_hex_escapes_all_external_text() {
        let mut s = State {
            epoch: 3,
            clock: 2.9,
            ids: vec![(0, 11), (1, 22)],
            ..Default::default()
        };
        s.names.insert(11, "Same|name".into());
        s.names.insert(22, "Same|name".into());
        s.builds.insert(11, PartyBuild::default());
        s.builds.insert(22, PartyBuild::default());
        let payload = wire(&mut s, 22, 2).unwrap();
        assert!(payload.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(payload.starts_with("02"));
        assert!(wire(&mut s, 33, 2).is_none());
    }
}

#[cfg(feature = "test-hooks")]
static TIP_SAMPLES: std::sync::Mutex<BTreeMap<(u64, u32, u32), Vec<(u32, String)>>> =
    std::sync::Mutex::new(BTreeMap::new());
static SPELL_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
type SpellPresentation = unsafe extern "system" fn(*mut u8, u64, u64, u8, u64) -> *mut u8;
unsafe extern "system" fn spell_presentation(
    output: *mut u8,
    ability: u64,
    stats: u64,
    kind: u8,
    perks: u64,
) -> *mut u8 {
    let original: SpellPresentation = std::mem::transmute(SPELL_ORIGINAL.load(Ordering::Acquire));
    let result = original(output, ability, stats, kind, perks);
    #[cfg(feature = "test-hooks")]
    let _timing = performance::start(2);
    if let Some(r) = RUNTIME.get().filter(|r| r.rules) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Ok(Some((slot, id, rank))) = r
                .adapter
                .spell_presentation_owner(&r.local, ability, stats, kind)
            else {
                return;
            };
            let Some(guid) = r.extension_party().and_then(|p|p.actors.iter().find(|a|a.slot==slot).map(|a|a.guid)) else {
                return;
            };
            let Ok(strings) = r
                .adapter
                .spell_presentation_strings(&r.local, output as u64)
            else {
                return;
            };
            #[cfg(feature = "test-hooks")]
            if std::env::var("ASX_TEST_HUD_OWNERS").as_deref() == Ok("1") || std::env::var("AZCOOP_TEST_NATIVE_TIPS121").as_deref()==Ok("1") {
                let changed = TIP_SAMPLES.lock().ok().is_some_and(|mut cache| {
                    if cache.len() > 128 {
                        cache.clear();
                    }
                    let key = (guid, id, rank);
                    let changed = cache.get(&key) != Some(&strings);
                    cache.insert(key, strings.clone());
                    changed
                });
                if changed {
                    r.log.info(&format!("official native spell presentation: guid={guid} id={id} rank={rank} strings={}",serde_json::json!(strings)));
                }
            }
            let get = |offset| {
                strings
                    .iter()
                    .find(|(at, _)| *at == offset)
                    .map(|(_, s)| s.clone())
                    .unwrap_or_default()
            };
            let tip = asx_game::official::SpellTip {
                title: get(0),
                body: get(24),
                rank: get(72),
                range: get(96),
                cast: get(120),
                cooldown: get(144),
            };
            if !tip.body.is_empty() && tip.body.len() <= 8192 {
                if let Ok(mut state) = STATE.lock() {
                    if let Some(s) = state.as_mut() {
                        if let Some(build) = s.builds.get_mut(&guid) {
                            if let Some(icon) = build
                                .spells
                                .iter_mut()
                                .find(|i| i.id == id && i.rank == rank && i.name == tip.title)
                            {
                                icon.text = tip.body.clone();
                                icon.spell_tip = Some(tip);
                            }
                        }
                    }
                }
            }
        }));
    }
    result
}
pub fn spell_hook(adapter: Adapter) -> crate::install::HookSpec {
    crate::install::HookSpec {
        name: "official-owner-spell-presentation",
        target: local_base() + adapter.spell_presentation_entry().rva as usize,
        detour: spell_presentation as *const () as usize,
        trampoline: &SPELL_ORIGINAL,
    }
}

#[cfg(feature="test-hooks")]
pub(super) fn fixture_economy(r:&Runtime,server:u64){
 if std::env::var("ASX_TEST_COOP_UI").as_deref()!=Ok("1"){return}
 if r.adapter.observe(&r.local,server).ok().is_none_or(|s|s.role!=asx_game::official::Role::Host){return}
 let Some(p)=r.extension_party()else{return};
 if p.actors.len()!=2||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 static STAGE:AtomicUsize=AtomicUsize::new(0);let stage=STAGE.load(Ordering::Acquire);
 if stage>=4{return}
 if stage==3{
  if p.clock<12.||STAGE.compare_exchange(3,4,Ordering::AcqRel,Ordering::Acquire).is_err(){return}
  let game=r.adapter.game_address(server).unwrap();
  let gold:unsafe extern "system" fn(u64,u32)->usize=unsafe{std::mem::transmute(r.local.base+r.adapter.gold_entry().rva as usize)};
  let chest:unsafe extern "system" fn(u64)->bool=unsafe{std::mem::transmute(r.local.base+r.adapter.chest_entry().rva as usize)};
  unsafe{gold(game,1000);let opened=chest(game);r.log.info(&format!("coop economy equipment draw: opened={opened}"));}
  return
 }
 if p.clock<[2.,6.,10.][stage]{return}
 if STAGE.compare_exchange(stage,stage+1,Ordering::AcqRel,Ordering::Acquire).is_err(){return}
 let xp=if stage==2{2500}else{1000};let gold=if stage==0{2000}else{3000};
 let rewards=p.actors.iter().map(|a|((a.slot,a.guid),asx_gameplay::blessings::Blessing{boss:1,category:if a.slot==0{1}else{2},rarity:0,amount:if a.slot==0{xp}else{gold}})).collect::<Vec<_>>();
 match blessings::ritual_rewards(r,r.adapter.game_address(server).unwrap(),&rewards){
  Ok(())=>r.log.info(&format!("coop economy fixture: stage={} xp={xp} gold={gold}",stage+1)),
  Err(e)=>r.log.warn(&format!("Coop economy fixture rejected: {e}")),
 }
}
