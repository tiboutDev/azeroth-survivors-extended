//! Run-local glyph coordination at native choice/rebuild boundaries only.
use super::*;
use asx_game::official::{GlyphChoice,GlyphOwner};
use asx_gameplay::glyphs::{self as rules,Glyph,Glyphs,Offer};
#[derive(Default)] struct State {game:u64,clock:f32,rules:Glyphs,offers:Vec<(GlyphChoice,Offer)>}
static STATE:std::sync::Mutex<Option<State>>=std::sync::Mutex::new(None);
thread_local! {static PENDING:std::cell::RefCell<Vec<(GlyphOwner,Option<Glyph>)>>=const{std::cell::RefCell::new(Vec::new())};}
struct Pending;
impl Pending {fn push(owner:GlyphOwner,glyph:Option<Glyph>)->Self {PENDING.with(|p|p.borrow_mut().push((owner,glyph)));Self}}
impl Drop for Pending {fn drop(&mut self){PENDING.with(|p|{p.borrow_mut().pop();});}}
pub fn enabled()->bool{RUNTIME.get().is_none_or(|r|r.adapter.glyphs_enabled())}
pub fn reset(){if let Ok(mut s)=STATE.lock(){*s=None;}}
pub fn observe(game:u64,p:Option<&asx_game::official::PartyObservation>){
 if !enabled(){return}
 let Some(p)=p else{reset();return};
 if let Ok(mut guard)=STATE.lock(){let s=guard.get_or_insert_with(State::default);
 if s.game!=game||p.clock<s.clock{*s=State{game,..Default::default()};}
 s.clock=p.clock;let ids=p.actors.iter().map(|a|(a.slot,a.guid)).collect::<Vec<_>>();s.rules.retain(&ids);s.offers.retain(|(c,_)|ids.contains(&(c.owner.slot,c.owner.guid)));}
}
fn view(owner:GlyphOwner)->Option<Glyph>{
 if !enabled(){return None}
 if let Some(pending)=PENDING.with(|p|p.borrow().iter().rev().find(|(o,_)|*o==owner).map(|(_,g)|*g)){return pending}
 STATE.lock().ok().and_then(|s|s.as_ref().filter(|s|s.game==owner.game)?.rules.get(owner.slot,owner.guid,owner.spell))
}
fn view_for_ui(slot:u32,guid:u64,spell:u32)->Option<Glyph>{
 if !enabled(){return None}
 if let Some(pending)=PENDING.with(|p|p.borrow().iter().rev().find(|(o,_)|o.slot==slot&&o.guid==guid&&o.spell==spell).map(|(_,g)|*g)){return pending}
 STATE.lock().ok().and_then(|s|s.as_ref()?.rules.get(slot,guid,spell))
}
pub fn healing_for_source(r:&Runtime,game:u64,slot:u32,guid:u64,spell:u32)->f32{
 if !matches!(spell,25912|15237){return 1.}view(GlyphOwner{game,slot,guid,spell}).map(|g|r.adapter.glyph_extra_healing(g)).unwrap_or(1.)
}
pub fn explosion_multiplier(r:&Runtime,game:u64,spell:u32)->f32{
 if !matches!(spell,1130|118|770){return 1.}
 r.adapter.run_source(&r.local,game).ok().flatten().and_then(|(slot,guid)|view(GlyphOwner{game,slot,guid,spell})).map(Glyph::damage_factor).unwrap_or(1.)
}
pub fn healing_multiplier(r:&Runtime,game:u64,spell:u32)->f32{
 if !matches!(spell,25912|15237)||fatigue::heal_pre_scaled(game){return 1.}
 r.adapter.run_source(&r.local,game).ok().flatten().map(|(slot,guid)|healing_for_source(r,game,slot,guid,spell)).unwrap_or(1.)
}
pub unsafe fn special_presentation(r:&Runtime,original:usize,args:[u64;5])->Option<u64>{
 let eligible=PENDING.with(|p|p.borrow().iter().any(|(o,_)|matches!(o.spell,25912|15237|1130|118|5782|8122)))||STATE.lock().ok().is_some_and(|s|s.as_ref().is_some_and(|s|s.rules.has_spell(25912)||s.rules.has_spell(15237)||s.rules.has_spell(1130)||s.rules.has_spell(118)||s.rules.has_spell(5782)||s.rules.has_spell(8122)));
 if !eligible{return None}
 let (slot,spell,_)=r.adapter.spell_presentation_owner(&r.local,args[1],args[2],args[3] as u8).ok().flatten()?;
 if !matches!(spell,25912|15237|1130|118|5782|8122){return None}
 let guid=r.extension_party().and_then(|p|p.actors.iter().find(|a|a.slot==slot).map(|a|a.guid))?;
 let glyph=view_for_ui(slot,guid,spell)?;
 let factor=if matches!(spell,5782|8122){glyph.effect_factor()}else if matches!(spell,1130|118){glyph.damage_factor()}else{r.adapter.glyph_extra_healing(glyph)};if factor==1.{return None}
 r.adapter.glyph_special_presentation(&r.local,original,args,factor).ok()
}
pub fn tooltip(r:&Runtime,output:u64,slot:u32,guid:u64,spell:u32){
 let glyph=view_for_ui(slot,guid,spell);
 if let Some(g)=glyph {let io=io::Access{local:&r.local,adapter:r.adapter};if spell==770{let _=r.adapter.glyph_faerie_text(&io,output+24,g.damage_factor());}if spell==118{let _=r.adapter.glyph_polymorph_text(&io,output+24,g.damage_factor(),g.sheep_limit());}let _=r.adapter.glyph_text(&io,output+24,&format!("Glyph: {}\n{}",g.name(),g.text()));}
}
fn card_note(offer:Offer,previous:Option<Glyph>)->Option<String>{
 match offer {
 Offer::Remove=>previous.map(|g|format!("Remove glyph: {}\nRestores unglyphed behavior\nRemoves {}",g.name(),g.name())),
 Offer::Apply(g) if previous!=Some(g)=>Some(format!("Glyph: {}\n{}\n{}",g.name(),g.text(),previous.map(|old|format!("Replaces {}",old.name())).unwrap_or_else(||"Applies glyph".into()))),
 _=>previous.map(|g|format!("Keeps glyph: {}",g.name()))
 }
}
static MODIFIERS:AtomicUsize=AtomicUsize::new(0);
type Resolve=unsafe extern "system" fn(u64,u64,u64)->u64;
pub fn card(output:u64,game:u64){
 if !enabled(){return}
 if let Some(r)=RUNTIME.get().filter(|r|r.rules){let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
  #[cfg(feature="test-hooks")]
  if fixture_enabled(){static SAMPLES:AtomicUsize=AtomicUsize::new(0);if SAMPLES.fetch_add(1,Ordering::Relaxed)<20{r.log.info(&format!("glyph fixture card probe: {:?}",r.adapter.glyph_fixture_card_probe(&r.local,r.local.base as u64,game,output)));}}
  let Ok(Some(c))=r.adapter.glyph_card(&r.local,r.local.base as u64,game,output)else{return};
  let previous=view(c.owner);
  let offer=rules::choice(c.owner.spell,c.rank,c.rarity,&c.upgrades,previous);
  #[cfg(feature="test-hooks")]
  let offer=if retention_fixture()&&c.owner.spell==845&&c.rank+c.upgrades.len()as u32>=5{
   if previous.is_none()&&!RETENTION_REMOVED.lock().ok().is_some_and(|s|s.contains(&(c.owner.slot,c.owner.guid))){Offer::Apply(Glyph::SweepingCleave)}
   else if previous.is_some()&&RETENTION_KEPT.lock().ok().is_some_and(|s|s.contains(&(c.owner.slot,c.owner.guid))){Offer::Remove}else{Offer::Keep}
  }else{offer};
  // The compatibility fixture must observe Faerie Fire's native arc hook even
  // when the production 20% offer roll never selects it in a bounded run.
  #[cfg(feature="test-hooks")]
  let offer=if fixture_enabled()&&fixture_class()==11&&c.owner.spell==770&&c.rank>=1&&previous.is_none(){Offer::Apply(Glyph::FarReachingSparks)}else{offer};
  let Some(note)=card_note(offer,previous)else{return};
  let io=io::Access{local:&r.local,adapter:r.adapter};
  if r.adapter.reward_card_note(&io,output,&note).is_err(){return}
  if offer!=Offer::Keep {if let Ok(mut guard)=STATE.lock(){let s=guard.get_or_insert_with(||State{game,..Default::default()});if s.game==game {s.offers.retain(|(old,_)|old!=&c);if s.offers.len()>=128{s.offers.remove(0);}s.offers.push((c,offer));}}}
 }));}
}
pub struct Selection {pub index:u64, chosen:Option<(GlyphChoice,Offer)>, _pending:Option<Pending>,
 #[cfg(feature="test-hooks")]kept_choice:Option<GlyphChoice>}
pub unsafe fn selecting(game:u64,index:u64)->Selection {
 if !enabled(){return Selection{index,chosen:None,_pending:None,#[cfg(feature="test-hooks")]kept_choice:None}}
 let r=RUNTIME.get().filter(|r|r.rules);
 #[cfg(feature="test-hooks")]
 let index=if fixture_enabled()&&!retention_fixture(){
  let pick=r.and_then(|r|{let seen=FIXTURE_SEEN.lock().ok()?.clone();let (slot,guid)=r.adapter.run_source(&r.local,game).ok().flatten()?;
   let required=if fixture_paladin(){[25912,635,20925]}else if fixture_class()==3{[1130,13810,1978]}else if fixture_class()==4{[14186,13750,5171]}else if fixture_class()==5{[17,139,15237]}else if fixture_class()==7{[324,331,16038]}else if fixture_class()==8{[11426,118,7269]}else if fixture_class()==9{[18708,6353,5782]}else if fixture_class()==11{[774,467,770]}else{[845,1680,772]}.into_iter().filter(|spell|{let g=view(GlyphOwner{game,slot,guid,spell:*spell});if fixture_class()==8&&*spell==118{g!=Some(Glyph::FlockMaster)}else if fixture_class()==5&&*spell==15237{g!=Some(Glyph::MendingNova)}else{g.is_none()}}).collect::<Vec<_>>();
   r.adapter.glyph_fixture_pick(&r.local,r.local.base as u64,game,&seen,&required,fixture_class()).ok().flatten()});
  if let (Some(r),Some((_,true)))=(r,pick){static REROLLS:AtomicUsize=AtomicUsize::new(0);if !fixture_ui_only() && REROLLS.fetch_add(1,Ordering::Relaxed)<if std::env::var("AZCOOP_TEST_EXTENSIONS").as_deref()==Ok("1"){2}else if matches!(fixture_class(),4|5|7|8|9|11){16}else{64}{
   let io=io::Access{local:&r.local,adapter:r.adapter};if r.adapter.glyph_fixture_reroll(&io,r.local.base,game).is_ok(){r.log.info("glyph fixture native reroll");return Selection{index:u64::MAX,chosen:None,_pending:None,kept_choice:None};}
  }}
  pick.map(|(i,_)|i).unwrap_or(index)
 }else{index};
 #[cfg(feature="test-hooks")]
 let index=if enchantments::fixture_enabled(){r.and_then(|r|r.adapter.ring_fixture_pick(&r.local,r.local.base as u64,game).ok().flatten()).unwrap_or(index)}else{index};
 #[cfg(feature="test-hooks")]
 let index=if gems::fixture_enabled(){r.and_then(|r|r.adapter.amulet_fixture_pick(&r.local,r.local.base as u64,game).ok().flatten()).unwrap_or(index)}else{index};
 #[cfg(feature="test-hooks")]
 let index=if retention_fixture(){r.and_then(|r|r.adapter.glyph_retention_pick(&r.local,r.local.base as u64,game).ok().flatten()).unwrap_or(index)}else{index};
 #[cfg(feature="test-hooks")]
 let kept_choice=if retention_fixture(){r.and_then(|r|r.adapter.glyph_selected(&r.local,r.local.base as u64,game,index).ok().flatten()).filter(|c|c.owner.spell==845&&view(c.owner).is_some())}else{None};
 let chosen=r.and_then(|r|r.adapter.glyph_selected(&r.local,r.local.base as u64,game,index).ok().flatten()).and_then(|c|{
 STATE.lock().ok().and_then(|s|{let s=s.as_ref().filter(|s|s.game==game)?;s.offers.iter().find(|(old,_)|old==&c).map(|(_,g)|(c.clone(),*g))})});
 let pending=chosen.as_ref().map(|(c,g)|Pending::push(c.owner,g.result(view(c.owner))));
 Selection{index,chosen,_pending:pending,#[cfg(feature="test-hooks")]kept_choice}
}
pub fn accepted(game:u64,s:&Selection){
 if !enabled(){return}
 let r=RUNTIME.get().filter(|r|r.rules);let chosen=s.chosen.clone();
 #[cfg(feature="test-hooks")]let kept_choice=s.kept_choice.clone();
 #[cfg(feature="test-hooks")]
 if let(Some(r),Some(c))=(r,kept_choice.as_ref()){
  if r.adapter.glyph_rank(&r.local,r.local.base as u64,c).ok().flatten()==Some(c.rank+c.upgrades.len()as u32){
   let owner=(c.owner.slot,c.owner.guid);if chosen.as_ref().is_some_and(|(_,g)|*g==Offer::Remove){if let Ok(mut v)=RETENTION_REMOVED.lock(){v.insert(owner);}}else{if let Ok(mut v)=RETENTION_KEPT.lock(){v.insert(owner);}r.log.info(&format!("glyph retained native acceptance: slot={} before={} after={}",c.owner.slot,c.rank,c.rank+c.upgrades.len()as u32));}
  }
 }
 if let (Some(r),Some((c,glyph)))=(r,chosen){let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
  let Ok(Some(rank))=r.adapter.glyph_rank(&r.local,r.local.base as u64,&c)else{return};
  let expected=c.rank+c.upgrades.len() as u32;
  let applied=STATE.lock().ok().is_some_and(|mut guard|{let Some(s)=guard.as_mut().filter(|s|s.game==game)else{return false};let accepted=s.rules.accept(c.owner.slot,c.owner.guid,c.owner.spell,c.rank,rank,expected,glyph);s.offers.retain(|(old,_)|old.owner!=c.owner||old.rank>=rank);accepted});
  if applied {
  #[cfg(feature="test-hooks")]
  if fixture_enabled(){if let Ok(mut seen)=FIXTURE_SEEN.lock(){let name=match glyph{Offer::Apply(g)=>g.name(),Offer::Remove=>"Unglyphed",Offer::Keep=>"Keep"};if !seen.iter().any(|s|s==name){seen.push(name.into());}}}
  #[cfg(feature="test-hooks")]
  if fixture_enabled(){if let Ok(mut removed)=FIXTURE_REMOVED.lock(){let key=(c.owner.slot,c.owner.guid,c.owner.spell);if glyph==Offer::Remove{removed.insert(key);}else{removed.remove(&key);}}}
  let io=io::Access{local:&r.local,adapter:r.adapter};if let Err(e)=r.adapter.refresh_glyph(&io,r.local.base as u64,&c){r.log.warn(&format!("Glyph refresh rejected: {e}"));}else{
   #[cfg(feature="test-hooks")]
   if fixture_enabled()&&glyph==Offer::Remove{let baseline=FIXTURE_REMOVAL_BASELINE.lock().ok().and_then(|mut b|b.remove(&(c.owner.slot,c.owner.guid,c.owner.spell)));r.log.info(&format!("glyph fixture removal rebuilt: slot={} spell={} baseline={baseline:?} current={:?}",c.owner.slot,c.owner.spell,r.adapter.glyph_fixture_current(&r.local,r.local.base as u64,&c)));}
   #[cfg(feature="test-hooks")]
   if fixture_enabled()&&matches!(fixture_class(),3|4|5|7|8|9|11)&&glyph==Offer::Remove&&!matches!(c.owner.spell,770|1130|14186|724|16038|118|11426|6353|18708){r.log.info(&format!("glyph fixture controlled unglyphed cast: slot={} spell={} result={:?}",c.owner.slot,c.owner.spell,r.adapter.glyph_fixture_cast_choice(&io,r.local.base as u64,&c)));}
   #[cfg(feature="test-hooks")]
   if fixture_enabled()&&((fixture_class()==8&&c.owner.spell==118)||(fixture_class()==9&&c.owner.spell==6353)||(fixture_class()==11&&matches!(c.owner.spell,770|774|8936|467))){if let Offer::Apply(g)=glyph{static SEEN:std::sync::Mutex<std::collections::BTreeSet<&'static str>>=std::sync::Mutex::new(std::collections::BTreeSet::new());if SEEN.lock().ok().is_some_and(|mut s|s.insert(g.name())){r.log.info(&format!("glyph fixture {} tooltip: glyph={} strings={:?}",if fixture_class()==11{"druid"}else if c.owner.spell==6353{"warlock"}else{"mage"},g.name(),r.adapter.glyph_fixture_presentation(&io,r.local.base as u64,c.owner)));}}}
   r.log.info(&format!("glyph accepted: slot={} guid={} spell={} rank={} glyph={}",c.owner.slot,c.owner.guid,c.owner.spell,rank,match glyph{Offer::Apply(g)=>g.name(),Offer::Remove=>"Unglyphed",Offer::Keep=>"Keep"}));}}
 }));}
}
unsafe extern "system" fn modifiers(output:u64,perks:u64,definition:u64)->u64 {
 let original:Resolve=std::mem::transmute(MODIFIERS.load(Ordering::Acquire));let result=original(output,perks,definition);
 if let Some(r)=RUNTIME.get().filter(|r|r.rules){let _=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
  let Ok(Some(owner))=r.adapter.glyph_owner(&r.local,r.local.base as u64,perks,definition)else{return};
  #[cfg(feature="test-hooks")]
  if fixture_enabled()&&PENDING.with(|p|p.borrow().iter().rev().find(|(o,_)|*o==owner).is_some_and(|(_,g)|g.is_none())){
   if let Ok(values)=r.adapter.glyph_fixture_modifiers(&r.local,output){if let Ok(mut b)=FIXTURE_REMOVAL_BASELINE.lock(){b.insert((owner.slot,owner.guid,owner.spell),values);}}
  }
  if let Some(g)=view(owner){let io=io::Access{local:&r.local,adapter:r.adapter};
   #[cfg(feature="test-hooks")]
   let before=if fixture_enabled(){r.adapter.glyph_fixture_modifiers(&r.local,output).ok()}else{None};
   let applied=r.adapter.apply_glyph(&io,output,g).is_ok();
   #[cfg(not(feature="test-hooks"))]let _=applied;
   #[cfg(feature="test-hooks")]
   if let Some(before)=before.filter(|_|applied){
    let key=(owner.slot,owner.guid,owner.spell,g.name());
    if FIXTURE_MODS.lock().ok().is_some_and(|mut s|s.insert(key)) {r.log.info(&format!("glyph fixture modifiers: slot={} spell={} glyph={} before={before:?} after={:?}",owner.slot,owner.spell,g.name(),r.adapter.glyph_fixture_modifiers(&r.local,output)));}
   }
  }
 }));}result
}

type OnHit=unsafe extern "system" fn(u64,u64,u64,f32,u32);
pub unsafe fn intercept_hit(original:OnHit,game:u64,enemy:u64,effect:u64,damage:f32,spell:u32)->bool{
 if !enabled(){return false}
 if matches!(spell,5782|8122){if let Some(r)=RUNTIME.get().filter(|r|r.rules){let glyph=r.adapter.run_source(&r.local,game).ok().flatten().and_then(|(slot,guid)|view(GlyphOwner{game,slot,guid,spell}));if let Some(g)=glyph{if g.effect_factor()!=1.{if let Ok(Some((bytes,before,after)))=r.adapter.glyph_fear_effect(&r.local,effect,g.effect_factor()){
   #[cfg(feature="test-hooks")]if fixture_enabled(){static N:AtomicUsize=AtomicUsize::new(0);if N.fetch_add(1,Ordering::Relaxed)<16{r.log.info(&format!("glyph fixture fear effect: spell={spell} glyph={} before={before} after={after}",g.name()));}}
   #[cfg(not(feature="test-hooks"))]let _=(before,after);
   original(game,enemy,bytes.as_ptr()as u64,damage,spell);return true
 }}}}}
 #[cfg(feature="test-hooks")]
 if let Some(r)=RUNTIME.get().filter(|r|r.rules){if fixture_warlock_leech(r,original,game,enemy,effect,damage,spell){return true}}
 false
}
pub fn hooks(adapter:Adapter)->Vec<crate::install::HookSpec>{
 let detours=[modifiers as *const () as usize,sheep_gate_entry as *const () as usize];let originals=[&MODIFIERS,&SHEEP_GATE];let names=["official-glyph-modifiers","official-glyph-sheep-limit"];
 adapter.glyph_entries().iter().enumerate().filter(|(i,_)|matches!(i,2|4)).enumerate().map(|(i,(_,e))|crate::install::HookSpec{name:names[i],target:local_base()+e.rva as usize,detour:detours[i],trampoline:originals[i]}).collect()
}
#[cfg(test)]mod tests {
 use super::*;
 static ARGS:std::sync::Mutex<Vec<u64>>=std::sync::Mutex::new(Vec::new());
 unsafe extern "system" fn native_card(o:u64,g:u64,d:u64,r:u64)->u64{*ARGS.lock().unwrap()=vec![o,g,d,r];o^g^d^r}
 unsafe extern "system" fn native_accept(g:u64,i:u64){*ARGS.lock().unwrap()=vec![g,i];}
 unsafe extern "system" fn native_resolve(o:u64,p:u64,d:u64)->u64{*ARGS.lock().unwrap()=vec![o,p,d];o^p^d}
 #[test]fn native_glyph_wrappers_preserve_abi(){let a=0x1122334455667788;let b=0x8877665544332211;reward_cards::CARD.store(native_card as *const () as usize,Ordering::SeqCst);reward_cards::ACCEPT.store(native_accept as *const () as usize,Ordering::SeqCst);MODIFIERS.store(native_resolve as *const () as usize,Ordering::SeqCst);unsafe{assert_eq!(reward_cards::card(a,b,77,4),a^b^77^4);assert_eq!(*ARGS.lock().unwrap(),vec![a,b,77,4]);reward_cards::accept(a,b);assert_eq!(*ARGS.lock().unwrap(),vec![a,b]);assert_eq!(modifiers(a,b,77),a^b^77);assert_eq!(*ARGS.lock().unwrap(),vec![a,b,77]);}}
 #[test]fn pending_is_scoped_and_nested(){reset();let o=GlyphOwner{game:1,slot:0,guid:2,spell:845};assert_eq!(view(o),None);{let _a=Pending::push(o,Some(Glyph::SweepingCleave));assert_eq!(view(o),Some(Glyph::SweepingCleave));{let _b=Pending::push(o,Some(Glyph::HeavyCleave));assert_eq!(view(o),Some(Glyph::HeavyCleave));{let _removed=Pending::push(o,None);assert_eq!(view(o),None);assert_eq!(view_for_ui(o.slot,o.guid,o.spell),None);}}assert_eq!(view(o),Some(Glyph::SweepingCleave));}assert_eq!(view(o),None);}
}

#[cfg(feature="test-hooks")]
pub(super) fn fixture_enabled()->bool{std::env::var("AZCOOP_TEST_GLYPHS").as_deref()==Ok("1")}
#[cfg(feature="test-hooks")]
fn fixture_class()->u32{match std::env::var("AZCOOP_TEST_GLYPHS_CLASS").as_deref(){Ok("paladin")=>2,Ok("hunter")=>3,Ok("rogue")=>4,Ok("priest")=>5,Ok("shaman")=>7,Ok("mage")=>8,Ok("warlock")=>9,Ok("druid")=>11,_=>1}}
#[cfg(feature="test-hooks")]
fn fixture_paladin()->bool{fixture_class()==2}
#[cfg(feature="test-hooks")]
fn fixture_ui_only()->bool{std::env::var("AZCOOP_TEST_GLYPHS_UI_ONLY").as_deref()==Ok("1")}
#[cfg(feature="test-hooks")]
static FIXTURE_REMOVED:std::sync::Mutex<std::collections::BTreeSet<(u32,u64,u32)>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
#[cfg(feature="test-hooks")]
static FIXTURE_REMOVAL_BASELINE:std::sync::Mutex<std::collections::BTreeMap<(u32,u64,u32),Vec<f32>>>=std::sync::Mutex::new(std::collections::BTreeMap::new());
#[cfg(feature="test-hooks")]
static FIXTURE_SEEN:std::sync::Mutex<Vec<String>>=std::sync::Mutex::new(Vec::new());
#[cfg(feature="test-hooks")]
static FIXTURE_MODS:std::sync::Mutex<std::collections::BTreeSet<(u32,u64,u32,&'static str)>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
#[cfg(feature="test-hooks")]
static FIXTURE_CASTS:std::sync::Mutex<std::collections::BTreeSet<(u32,u64,u32,&'static str)>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
#[cfg(feature="test-hooks")]
pub fn fixture_cast(r:&Runtime,game:u64,ability:u64,success:bool){
 if !fixture_enabled()||!success{return}
 fixture_nova_applied(r,game);
 let Ok(Some((owner,rank,values)))=r.adapter.glyph_fixture_ability(&r.local,r.local.base as u64,game,ability)else{return};
 if (fixture_paladin()&&owner.spell==20925)||(fixture_class()==7&&owner.spell==324)||(fixture_class()==11&&owner.spell==467){
  static PROBED:AtomicBool=AtomicBool::new(false);
  if !PROBED.swap(true,Ordering::Relaxed){
   let io=io::Access{local:&r.local,adapter:r.adapter};
   let saved=if matches!(owner.spell,324|467){r.adapter.capture_aura(&r.local,game,owner.spell).ok()}else{None};
   let cast=||->Result<asx_game::official::NativeEffect,String>{
    if matches!(owner.spell,324|467){let _=r.adapter.glyph_fixture_support_clear(&io,game,owner.spell)?;}
    r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability)?;
    if !support::fixture_target(Some(owner.slot),||asx_game::official::SupportCalls::cast_actor(&io,game,ability))?{return Err("Native retaliation probe did not cast".into())}
    r.adapter.native_effect(&r.local,game,owner.spell)
   };
   let baseline={let _p=Pending::push(owner,None);cast()};
   if let Ok(before)=baseline{for g in if owner.spell==467{[Glyph::BarbedThorns,Glyph::LastingThorns]}else if owner.spell==324{[Glyph::OverchargedShield,Glyph::VolatileShield]}else{[Glyph::BarbedAegis,Glyph::LastingAegis]}{
    let after={let _p=Pending::push(owner,Some(g));cast()};
    if let Ok(after)=after{r.log.info(&format!("glyph fixture {} shield aura: glyph={} before_power={} after_power={} before_duration={} after_duration={}",if owner.spell==467{"thorns"}else if owner.spell==324{"lightning"}else{"holy"},g.name(),f32::from_le_bytes(before.data[4..8].try_into().unwrap()),f32::from_le_bytes(after.data[4..8].try_into().unwrap()),before.duration,after.duration));}
   }}
   let _=r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability);
   if let Some(saved)=saved{let _=r.adapter.restore_aura(&io,game,&saved);}else{let _=asx_game::official::SupportCalls::cast_actor(&io,game,ability);}
  }
 }
 if fixture_class()==4&&matches!(owner.spell,13750|5171){
  static PROBED:std::sync::Mutex<std::collections::BTreeSet<u32>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
  if PROBED.lock().ok().is_some_and(|mut p|p.insert(owner.spell)){
   let io=io::Access{local:&r.local,adapter:r.adapter};
   let baseline={let _pending=Pending::push(owner,None);r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability).and_then(|_|asx_game::official::SupportCalls::cast_actor(&io,game,ability)).and_then(|_|r.adapter.glyph_fixture_buff(&r.local,game,owner.spell))};
   let pair=if owner.spell==13750{[Glyph::EnduringRush,Glyph::IntenseRush]}else{[Glyph::LastingEdge,Glyph::KeenEdge]};
   if let Ok(before)=baseline{for g in pair{
    let after={let _pending=Pending::push(owner,Some(g));r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability).and_then(|_|asx_game::official::SupportCalls::cast_actor(&io,game,ability)).and_then(|_|r.adapter.glyph_fixture_buff(&r.local,game,owner.spell))};
    if let Ok(after)=after{r.log.info(&format!("glyph fixture rogue aura: spell={} glyph={} before_power={} after_power={} before_duration={} after_duration={}",owner.spell,g.name(),f32::from_le_bytes(before.data[4..8].try_into().unwrap()),f32::from_le_bytes(after.data[4..8].try_into().unwrap()),before.duration,after.duration));}
   }}
   let _=r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability);let _=asx_game::official::SupportCalls::cast_actor(&io,game,ability);
  }
 }
 if fixture_class()==9&&owner.spell==18708{
  static PROBED:AtomicBool=AtomicBool::new(false);
  if !PROBED.swap(true,Ordering::Relaxed){let io=io::Access{local:&r.local,adapter:r.adapter};if let Ok(saved)=r.adapter.capture_aura(&r.local,game,18708){
   let cast=||->Result<asx_game::official::NativeEffect,String>{let _=r.adapter.glyph_fixture_support_clear(&io,game,18708)?;r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability)?;if !asx_game::official::SupportCalls::cast_actor(&io,game,ability)?{return Err("Demonic Empowerment probe did not cast".into())}r.adapter.glyph_fixture_buff(&r.local,game,18708)};
   let baseline={let _p=Pending::push(owner,None);cast()};if let Ok(before)=baseline{for g in [Glyph::UnboundPact,Glyph::EnduringPact]{let after={let _p=Pending::push(owner,Some(g));cast()};if let Ok(after)=after{r.log.info(&format!("glyph fixture demonic aura: glyph={} before_power={} after_power={} before_duration={} after_duration={}",g.name(),f32::from_le_bytes(before.data[4..8].try_into().unwrap()),f32::from_le_bytes(after.data[4..8].try_into().unwrap()),before.duration,after.duration));}}}
   let _=r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability);let _=r.adapter.restore_aura(&io,game,&saved);
  }}
 }
 if (fixture_class()==7&&owner.spell==331)||(fixture_class()==11&&owner.spell==8936){
  static PROBED:AtomicBool=AtomicBool::new(false);
  if !PROBED.swap(true,Ordering::Relaxed){
   let io=io::Access{local:&r.local,adapter:r.adapter};
   let cast=||->Result<f32,String>{r.adapter.fixture_support_wounds(&io,game,1-owner.slot,owner.slot)?;r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability)?;let before=r.adapter.support_caster(&r.local,game)?.hp;let success=support::fixture_target(Some(owner.slot),||asx_game::official::SupportCalls::cast_actor(&io,game,ability))?;if !success{return Err("Native Healing Wave did not cast".into())}Ok(r.adapter.support_caster(&r.local,game)?.hp-before)};
   let baseline={let _p=Pending::push(owner,None);cast()};
   if let Ok(before)=baseline{for g in if owner.spell==8936{[Glyph::FlourishingRegrowth,Glyph::SwiftRegrowth]}else{[Glyph::DeepWaters,Glyph::SwiftWaters]}{let after={let _p=Pending::push(owner,Some(g));cast()};if let Ok(after)=after{r.log.info(&format!("glyph fixture {} healing: glyph={} before_amount={before} after_amount={after}",if owner.spell==8936{"druid"}else{"shaman"},g.name()));}}}
   let _=r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability);
  }
 }
 if (fixture_class()==5&&matches!(owner.spell,17|139))||(fixture_class()==8&&owner.spell==11426)||(fixture_class()==11&&owner.spell==774){
  static PROBED:std::sync::Mutex<std::collections::BTreeSet<u32>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
  if PROBED.lock().ok().is_some_and(|mut p|p.insert(owner.spell)){
   let io=io::Access{local:&r.local,adapter:r.adapter};
   if let Ok(saved)=r.adapter.capture_aura(&r.local,game,owner.spell){
    let cast=||{let _=r.adapter.glyph_fixture_support_clear(&io,game,owner.spell)?;let _=r.adapter.fixture_support_wounds(&io,game,1-owner.slot,owner.slot);r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability)?;let success=support::fixture_target(Some(owner.slot),||asx_game::official::SupportCalls::cast_actor(&io,game,ability))?;if !success{return Err("Native glyph support probe did not cast".to_string())}r.adapter.glyph_fixture_support_value(&r.local,game,owner.spell)};
    let baseline={let _p=Pending::push(owner,None);cast()};
    let pair=if owner.spell==774{[Glyph::AbundantGrowth,Glyph::PersistentGrowth]}else if owner.spell==11426{[Glyph::DeepBarrier,Glyph::FleetingBarrier]}else if owner.spell==17{[Glyph::DeepShelter,Glyph::FleetingShelter]}else{[Glyph::DeepRenewal,Glyph::SwiftRenewal]};
    if let Ok(before)=baseline{for g in pair{let after={let _p=Pending::push(owner,Some(g));cast()};if let Ok(after)=after{r.log.info(&format!("glyph fixture {} support: spell={} glyph={} before_power={} after_power={} before_duration={} after_duration={}",if owner.spell==774{"druid"}else if owner.spell==11426{"mage"}else{"priest"},owner.spell,g.name(),before.0,after.0,before.1,after.1));}}}
    let _=r.adapter.glyph_fixture_refresh_ability(&io,r.local.base as u64,game,ability);let _=r.adapter.restore_aura(&io,game,&saved);
   }
  }
 }
 let name=if let Some(g)=view(owner){g.name()}else if FIXTURE_REMOVED.lock().ok().is_some_and(|s|s.contains(&(owner.slot,owner.guid,owner.spell))){"Unglyphed"}else{return};
 if FIXTURE_CASTS.lock().ok().is_some_and(|mut s|s.insert((owner.slot,owner.guid,owner.spell,name))){r.log.info(&format!("glyph fixture native cast: slot={} spell={} rank={rank} glyph={} modifiers={values:?}",owner.slot,owner.spell,name));}
}
#[cfg(feature="test-hooks")]
pub fn fixture_tick(r:&Runtime,server:u64){
 if !fixture_enabled(){return}
 if retention_fixture(){retention_tick(r,server);return}
 static NEXT:std::sync::Mutex<(f32,u32)>=std::sync::Mutex::new((10.,0));
 let Some(p)=r.extension_party()else{return};
 if !(1..=2).contains(&p.actors.len())||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 let grant=NEXT.lock().ok().is_some_and(|mut n|{if p.clock<n.0||n.1>=if fixture_ui_only(){4}else{24}{return false}n.0=p.clock+3.;n.1+=1;true});
 if !grant{return}
 let Ok(game)=r.adapter.game_address(server)else{return};let io=io::Access{local:&r.local,adapter:r.adapter};
 static MATRIX:AtomicBool=AtomicBool::new(false);
 if !fixture_ui_only()&&!MATRIX.swap(true,Ordering::Relaxed){
  if let Ok(Some((slot,guid)))=r.adapter.run_source(&r.local,game){
   for glyph in Glyph::ALL.into_iter().filter(|g|match fixture_class(){2=>rules::PALADIN_SPELLS.contains(&g.spell()),3=>rules::HUNTER_SPELLS.contains(&g.spell()),4=>rules::ROGUE_SPELLS.contains(&g.spell()),5=>rules::PRIEST_SPELLS.contains(&g.spell()),7=>rules::SHAMAN_SPELLS.contains(&g.spell()),8=>rules::MAGE_SPELLS.contains(&g.spell()),9=>rules::WARLOCK_SPELLS.contains(&g.spell()),11=>rules::DRUID_SPELLS.contains(&g.spell()),_=>rules::WARRIOR_SPELLS.contains(&g.spell())}){
    let owner=GlyphOwner{game,slot,guid,spell:glyph.spell()};
    let before={let _p=Pending::push(owner,None);r.adapter.glyph_fixture_resolve(&io,r.local.base as u64,owner)};
    let after={let _p=Pending::push(owner,Some(glyph));r.adapter.glyph_fixture_resolve(&io,r.local.base as u64,owner)};
    r.log.info(&format!("glyph fixture {} matrix: slot={slot} spell={} glyph={} before={before:?} after={after:?}",match fixture_class(){2=>"paladin",3=>"hunter",4=>"rogue",5=>"priest",7=>"shaman",8=>"mage",9=>"warlock",11=>"druid",_=>"warrior"},glyph.spell(),glyph.name()));
   }
  }
 }
 if fixture_class()==3&&!MARK_EXPLOSION_OBSERVED.load(Ordering::Relaxed){
  static ATTEMPTS:std::sync::Mutex<std::collections::BTreeMap<u32,u32>>=std::sync::Mutex::new(std::collections::BTreeMap::new());
  for actor in &p.actors{
   if view(GlyphOwner{game,slot:actor.slot,guid:actor.guid,spell:1130}).is_none(){continue}
   let Ok(Some(target))=r.adapter.glyph_fixture_mark_target(&r.local,game,actor.slot)else{continue};
   let allowed=ATTEMPTS.lock().ok().is_some_and(|mut a|{let n=a.entry(actor.slot).or_default();if *n>=3{return false}*n+=1;true});
   if allowed{r.log.info(&format!("glyph fixture controlled marked death: slot={} target={target}",actor.slot));let _=r.adapter.attack_actor(&io,game,actor.slot,target,75,1e8);}
  }
 }
 if p.actors.len()==1 && r.solo.lock().ok().is_some_and(|p|p.is_some()) {
   let result=asx_game::official::SupportCalls::award_experience(&io,game,70);
   r.log.info(&format!("solo glyph fixture XP: {result:?}"));
   return;
 }
 for actor in &p.actors {
  match r.adapter.replay_experience(&io,game,actor.slot,actor.guid,70){Ok(true)=>{},other=>r.log.warn(&format!("Glyph fixture XP delivery: slot={} result={other:?}",actor.slot))}
 }
 if let Ok(Some(after))=r.adapter.world_party(&r.local,game){r.log.info(&format!("glyph fixture XP balanced: {:?}",after.actors.iter().map(|a|(a.slot,a.level,a.xp)).collect::<Vec<_>>()));}
 if fixture_paladin()||matches!(fixture_class(),5|7|9|11){
  static WOUNDS:AtomicUsize=AtomicUsize::new(0);let caster=(WOUNDS.fetch_add(1,Ordering::Relaxed)%2)as u32;
  if let Err(e)=r.adapter.fixture_support_wounds(&io,game,caster,1-caster){r.log.warn(&format!("Glyph healing fixture wounds: {e}"));}
 }
 r.log.info("glyph fixture XP: 70 credited once to each participant");
}


#[cfg(test)]mod removal_tests {
 use super::*;
 #[test]fn notes_place_status_last_and_removal_is_explicit(){
 let old=Glyph::SweepingCleave;let new=Glyph::HeavyCleave;
 assert!(card_note(Offer::Apply(new),Some(old)).unwrap().ends_with("Replaces Sweeping Cleave"));
 assert_eq!(card_note(Offer::Keep,Some(old)).unwrap(),format!("Keeps glyph: {}",old.name()));
 let removed=card_note(Offer::Remove,Some(old)).unwrap();assert_eq!(annotation(&removed),Some((old,true,0)));assert!(removed.ends_with("Removes Sweeping Cleave"));
 assert_eq!(card_note(Offer::Remove,None),None);assert!(!bonus_ui::status_line("Removes glyph bonuses and penalties"));
 }

}

#[cfg(feature="test-hooks")]
pub fn fixture_heal(r:&Runtime,game:u64,spell:u32,before:f32,after:f32,factor:f32,delivery:bool){
 if !fixture_enabled()||!matches!(spell,25912|15237)||factor==1.{return}
 static COUNT:AtomicUsize=AtomicUsize::new(0);if COUNT.fetch_add(1,Ordering::Relaxed)<64{r.log.info(&format!("glyph fixture healing: game={game} spell={spell} delivery={delivery} before={before} after={after} factor={factor} source={:?}",r.adapter.run_source(&r.local,game)));}
}


#[cfg(feature="test-hooks")]
static MARK_EXPLOSION_OBSERVED:AtomicBool=AtomicBool::new(false);
#[cfg(feature="test-hooks")]
pub fn fixture_explosion(r:&Runtime,game:u64,spell:u32,before:f32,after:f32,factor:f32){
 if !fixture_enabled()||!((fixture_class()==3&&spell==1130)||(fixture_class()==8&&spell==118)||(fixture_class()==11&&spell==770)){return}
 if factor!=1.&&before>0. {MARK_EXPLOSION_OBSERVED.store(true,Ordering::Relaxed);}
 if spell==770&&factor==1.{return}
 static SAMPLES:AtomicUsize=AtomicUsize::new(0);if SAMPLES.fetch_add(1,Ordering::Relaxed)<32{r.log.info(&format!("glyph fixture {} explosion: game={game} before={before} after={after} factor={factor} source={:?}",if spell==770{"faerie"}else if spell==118{"sheep"}else{"mark"},r.adapter.run_source(&r.local,game)));}
}

#[cfg(feature="test-hooks")]
pub fn fixture_seal_hit(r:&Runtime,game:u64,enemy:u64,amount:f32){
 if !fixture_enabled()||fixture_class()!=4{return}
 let source=r.adapter.run_source(&r.local,game).ok().flatten();let glyph=source.and_then(|(slot,guid)|view(GlyphOwner{game,slot,guid,spell:14186}));
 if glyph.is_none(){return}
 static COUNT:AtomicUsize=AtomicUsize::new(0);if COUNT.fetch_add(1,Ordering::Relaxed)>=32{return}
 r.log.info(&format!("glyph fixture seal hit: source={source:?} enemy={enemy} amount={amount} glyph={}",glyph.map(Glyph::name).unwrap_or("Unglyphed")));
}

#[cfg(feature="test-hooks")]
pub fn fixture_lightwell_hit(r:&Runtime,game:u64,amount:f32){
 if !fixture_enabled()||fixture_class()!=5{return}
 static COUNT:AtomicUsize=AtomicUsize::new(0);if COUNT.fetch_add(1,Ordering::Relaxed)<8{r.log.info(&format!("glyph fixture lightwell hit: source={:?} amount={amount}",r.adapter.run_source(&r.local,game)));}
}

#[cfg(feature="test-hooks")]
thread_local!{static NOVA_HEAL:std::cell::Cell<Option<(u64,f32,f32,f32)>>=const{std::cell::Cell::new(None)};}
#[cfg(feature="test-hooks")]
pub fn fixture_nova_heal(r:&Runtime,game:u64,before:f32,after:f32,factor:f32){
 if !fixture_enabled()||fixture_class()!=5||factor==1.{return}
 fixture_heal(r,game,15237,before,after,factor,false);
 if let Ok(caster)=r.adapter.support_caster(&r.local,game){NOVA_HEAL.with(|v|v.set(Some((game,caster.hp,caster.maximum,after))));}
}
#[cfg(feature="test-hooks")]
fn fixture_nova_applied(r:&Runtime,game:u64){
 let Some((g,before,maximum,amount))=NOVA_HEAL.with(|v|v.take())else{return};if g!=game{return}
 if let Ok(after)=r.adapter.support_caster(&r.local,game){if (after.maximum-maximum).abs()<0.001{r.log.info(&format!("glyph fixture self heal applied: spell=15237 before_hp={before} after_hp={} maximum={maximum} fraction={}",after.hp,amount/maximum));}}
}

#[cfg(feature="test-hooks")]
pub fn fixture_shaman_hit(r:&Runtime,game:u64,spell:u32,amount:f32){
 if !fixture_enabled()||fixture_class()!=7||!matches!(spell,324|8187|3606|16038)||amount<=0.{return}
 static COUNTS:std::sync::Mutex<std::collections::BTreeMap<u32,usize>>=std::sync::Mutex::new(std::collections::BTreeMap::new());
 let emit=COUNTS.lock().ok().is_some_and(|mut c|{let n=c.entry(spell).or_default();*n+=1;*n<=8});
 if emit{r.log.info(&format!("glyph fixture shaman hit: spell={spell} source={:?} amount={amount}",r.adapter.run_source(&r.local,game)));}
}

static SHEEP_GATE:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_flags!(sheep_gate_entry,sheep_gate,SHEEP_GATE);
unsafe extern "system" fn sheep_gate(c:&mut crate::stub::Context){
 #[cfg(test)]if let Some((count,limit))=SHEEP_TEST.with(|v|v.get()){let a=Adapter::select(asx_game::official122::SHA256).unwrap();c.rflags=a.glyph_sheep_flags(count,limit,c.rflags);return}
 let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};let game=c.rcx;
 let Ok(Some((slot,guid)))=r.adapter.run_source(&r.local,game)else{return};let Some(g)=view(GlyphOwner{game,slot,guid,spell:118})else{return};
 if let Ok(count)=r.adapter.glyph_sheep_count(&r.local,game){let before=c.rflags;c.rflags=r.adapter.glyph_sheep_flags(count,g.sheep_limit(),before);
 #[cfg(feature="test-hooks")]if fixture_enabled()&&fixture_class()==8&&g==Glyph::FlockMaster{static N:AtomicUsize=AtomicUsize::new(0);if N.fetch_add(1,Ordering::Relaxed)<16{r.log.info(&format!("glyph fixture sheep gate: slot={slot} count={count} limit={} glyph={} allowed={}",g.sheep_limit(),g.name(),c.rflags&0x41!=0));}}
 }
}
#[cfg(test)]thread_local!{static SHEEP_TEST:std::cell::Cell<Option<(u64,u32)>>=const{std::cell::Cell::new(None)};}
#[cfg(test)]mod sheep_abi_tests{
 use super::*;
 #[unsafe(naked)]unsafe extern "C" fn continuation(){core::arch::naked_asm!("ret");}
 #[unsafe(naked)]unsafe extern "system" fn probe()->u64{core::arch::naked_asm!("sub rsp,40","mov ecx,3","cmp rcx,2","call {entry}","pushfq","pop rax","add rsp,40","ret",entry=sym sheep_gate_entry);}
 #[test]fn sheep_gate_preserves_native_branch_without_an_owner_and_changes_only_zf(){SHEEP_GATE.store(continuation as *const () as usize,Ordering::SeqCst);let before=unsafe{probe()};assert_eq!(before&0x41,0);SHEEP_TEST.with(|v|v.set(Some((3,5))));let after=unsafe{probe()};SHEEP_TEST.with(|v|v.set(None));assert_eq!(after,before|0x40);assert_eq!(unsafe{probe()},before);}
}

#[cfg(feature="test-hooks")]
pub fn fixture_warlock_hit(r:&Runtime,game:u64,spell:u32,amount:f32){
 if !fixture_enabled()||fixture_class()!=9||!matches!(spell,6353|689|6789)||amount<=0.{return}
 let Ok(Some((slot,guid)))=r.adapter.run_source(&r.local,game)else{return};let Some(g)=view(GlyphOwner{game,slot,guid,spell})else{return};
 static COUNTS:std::sync::Mutex<std::collections::BTreeMap<u32,u32>>=std::sync::Mutex::new(std::collections::BTreeMap::new());if COUNTS.lock().ok().is_some_and(|mut c|{let n=c.entry(spell).or_default();*n+=1;*n<=8}){r.log.info(&format!("glyph fixture warlock hit: slot={slot} spell={spell} glyph={} amount={amount}",g.name()));}
}


#[cfg(feature="test-hooks")]
fn fixture_warlock_leech(r:&Runtime,original:OnHit,game:u64,enemy:u64,effect:u64,damage:f32,spell:u32)->bool{
 if !fixture_enabled()||fixture_class()!=9||!matches!(spell,689|6789)||damage<=0.{return false}
 let Ok(Some(fraction))=r.adapter.glyph_fixture_leech_fraction(&r.local,effect)else{return false};let Ok(before)=r.adapter.support_caster(&r.local,game)else{return false};
 unsafe{original(game,enemy,effect,damage,spell)};
 if let Ok(after)=r.adapter.support_caster(&r.local,game){static N:AtomicUsize=AtomicUsize::new(0);if (before.maximum-after.maximum).abs()<0.001&&N.fetch_add(1,Ordering::Relaxed)<32{r.log.info(&format!("glyph fixture warlock leech HP: spell={spell} before={} after={} maximum={} damage={damage} fraction={fraction}",before.hp,after.hp,after.maximum));}}
 true
}

#[cfg(feature="test-hooks")]pub fn retention_fixture()->bool{std::env::var("AZCOOP_TEST_GLYPH_RETENTION").as_deref()==Ok("1")}
#[cfg(feature="test-hooks")]static RETENTION_KEPT:std::sync::Mutex<std::collections::BTreeSet<(u32,u64)>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
#[cfg(feature="test-hooks")]static RETENTION_REMOVED:std::sync::Mutex<std::collections::BTreeSet<(u32,u64)>>=std::sync::Mutex::new(std::collections::BTreeSet::new());
#[cfg(feature="test-hooks")]pub fn retention_tick(r:&Runtime,server:u64){
 let Some(p)=r.extension_party()else{return};if p.actors.len()!=2||p.actors.iter().any(|a|a.phase!=asx_gameplay::Phase::Playing){return}
 static NEXT:std::sync::Mutex<(f32,u32)>=std::sync::Mutex::new((2.,0));if !NEXT.lock().ok().is_some_and(|mut s|{if p.clock<s.0||s.1>=18{return false}s.0=p.clock+1.;s.1+=1;true}){return}
 let Ok(game)=r.adapter.game_address(server)else{return};let io=io::Access{local:&r.local,adapter:r.adapter};for actor in &p.actors{let _=r.adapter.replay_experience(&io,game,actor.slot,actor.guid,250);}
}

pub fn annotation(content:&str)->Option<(Glyph,bool,usize)>{
 let at=content.find("\n\nGlyph:").or_else(||content.find("\n\nKeeps glyph:")).or_else(||content.find("\n\nRemove glyph:"));
 let (rest,prefix)=if let Some(at)=at{(&content[at+2..],at)}else{(content,0)};
 let name=rest.strip_prefix("Glyph: ").or_else(||rest.strip_prefix("Keeps glyph: ")).or_else(||rest.strip_prefix("Remove glyph: "))?.lines().next()?;
 let g=Glyph::ALL.into_iter().find(|g|g.name()==name)?;
 Some((g,rest.starts_with("Remove glyph:"),prefix))
}
