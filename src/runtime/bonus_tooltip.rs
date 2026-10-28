//! Reuse the native item-replacement tooltip hierarchy for retained ASX bonuses.
use super::*;
use asx_gameplay::kept_bonus::KeptBonus;
#[derive(Clone,Copy)]struct Row{font:u64,ui:asx_game::official::BonusUi,bonus:KeptBonus}
thread_local!{static ROW:std::cell::Cell<Option<Row>>=const{std::cell::Cell::new(None)};static ATTACHED:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
struct Scope(Option<Row>,bool);
impl Drop for Scope{fn drop(&mut self){ROW.with(|v|v.set(self.0));ATTACHED.with(|v|v.set(self.1));}}
pub fn with_row<T>(font:u64,ui:asx_game::official::BonusUi,bonus:KeptBonus,draw:impl FnOnce()->T)->(T,bool){
 let _scope=Scope(ROW.with(|v|v.replace(Some(Row{font,ui,bonus}))),ATTACHED.with(|v|v.replace(false)));
 let result=draw();(result,ATTACHED.with(|v|v.get()))
}
static ROW_ORIGINAL:AtomicUsize=AtomicUsize::new(0);
static BODY_ORIGINAL:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_rw!(row_entry,0,attach,ROW_ORIGINAL);
unsafe extern "system" fn attach(c:&mut crate::stub::Context){
 let Some(row)=ROW.with(|v|v.get())else{return};let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};
 let io=io::Access{local:&r.local,adapter:r.adapter};
 match r.adapter.kept_tooltip_attach(&io,r.local.base,c.rbp,row.font,row.ui,row.bonus){
  Ok(_entity)=>{
   #[cfg(feature="test-hooks")]if fixture_enabled(){fixture_target(_entity,row.bonus);}
   ATTACHED.with(|v|v.set(true));#[cfg(feature="test-hooks")]if fixture_enabled(){r.log.info(&format!("native kept tooltip attached: category={} name={}",row.bonus.category(),row.bonus.name()));}},
  Err(e)=>r.log.warn(&format!("Native kept tooltip rejected: {e}")),
 }
}
unsafe extern "system" fn body(parent:u64,closure:u64){
 if let Some(r)=RUNTIME.get().filter(|r|r.rules){
  match r.adapter.kept_tooltip_body(&r.local,closure){
   Ok(Some((font,bonus)))=>{bonus_ui::draw_kept_tooltip(r,parent,font,bonus);return},
   Err(e)=>{r.log.warn(&format!("Native kept tooltip body rejected: {e}"));return},
   _=>{},
  }
 }
 let original:unsafe extern "system" fn(u64,u64)=std::mem::transmute(BODY_ORIGINAL.load(Ordering::Acquire));original(parent,closure)
}
pub fn hooks(adapter:Adapter)->Vec<crate::install::HookSpec>{let e=adapter.bonus_tooltip_entries();if e.len()!=5{return vec![]}
 let mut hooks=vec![crate::install::HookSpec{name:"official-kept-native-tooltip-row",target:local_base()+e[0].rva as usize,detour:row_entry as *const ()as usize,trampoline:&ROW_ORIGINAL},crate::install::HookSpec{name:"official-kept-native-tooltip-body",target:local_base()+e[1].rva as usize,detour:body as *const ()as usize,trampoline:&BODY_ORIGINAL}];
 #[cfg(feature="test-hooks")]if fixture_enabled(){if let Some(e)=adapter.kept_tooltip_fixture_focus(){hooks.push(crate::install::HookSpec{name:"fixture-kept-native-hit",target:local_base()+e.rva as usize,detour:focus_entry as *const ()as usize,trampoline:&FOCUS_ORIGINAL});}}
 hooks
}
#[cfg(test)]mod tests{use super::*;#[test]fn nested_native_rows_restore_the_outer_context_without_holding_a_borrow(){let g=KeptBonus::Glyph(asx_gameplay::glyphs::Glyph::SweepingCleave);let gem=KeptBonus::Gem(asx_gameplay::gems::Gem::Ruby);struct Mem;impl asx_game::memory::Mem for Mem{fn read(&self,a:u64,n:usize)->Result<Vec<u8>,String>{match(a,n){(0x10060,8)=>Ok(0x20000u64.to_le_bytes().to_vec()),(0x10058,8)=>Ok(0x30000u64.to_le_bytes().to_vec()),(0x30000,20)=>Ok(vec![0;20]),_=>Err("Unexpected test read".into())}}}
 let ui=asx_game::official::SelectedAdapter::select(asx_game::official122::SHA256).unwrap().bonus_ui_context(&Mem,0x10000).unwrap();let(value,attached)=with_row(10,ui,g,||{assert_eq!(ROW.with(|v|v.get().unwrap().bonus),g);ATTACHED.with(|v|v.set(true));let(_,inner)=with_row(30,ui,gem,||{assert_eq!(ROW.with(|v|v.get().unwrap().bonus),gem);});assert!(!inner);assert_eq!(ROW.with(|v|v.get().unwrap().bonus),g);7});assert_eq!(value,7);assert!(attached);assert!(ROW.with(|v|v.get()).is_none());assert!(!ATTACHED.with(|v|v.get()));}}

#[cfg(feature="test-hooks")]struct Target{entity:u64,bonus:KeptBonus,at:std::time::Instant,entered:bool,left:bool}
#[cfg(feature="test-hooks")]static TARGETS:std::sync::Mutex<Vec<Target>>=std::sync::Mutex::new(Vec::new());
#[cfg(feature="test-hooks")]fn fixture_target(entity:u64,bonus:KeptBonus){if let Ok(mut t)=TARGETS.lock(){t.retain(|t|t.at.elapsed().as_secs()<60);if t.len()<12&&!t.iter().any(|t|t.entity==entity){t.push(Target{entity,bonus,at:std::time::Instant::now(),entered:false,left:false});}}}
#[cfg(feature="test-hooks")]static FOCUS_ORIGINAL:AtomicUsize=AtomicUsize::new(0);
#[cfg(feature="test-hooks")]crate::context_stub_rw!(focus_entry,0,focus,FOCUS_ORIGINAL);
#[cfg(feature="test-hooks")]unsafe extern "system" fn focus(c:&mut crate::stub::Context){
 if !fixture_enabled(){return}
 let result=TARGETS.lock().ok().and_then(|mut list|{let t=list.iter_mut().find(|t|t.entity==c.r12)?;let hover=(1.0..19.0).contains(&t.at.elapsed().as_secs_f32());let changed=if hover&&!t.entered{t.entered=true;Some(true)}else if !hover&&t.entered&&!t.left{t.left=true;Some(false)}else{None};Some((hover,changed,t.bonus))});
 if let Some((hover,changed,bonus))=result{if hover{c.rax=(c.rax&!255)|1;}if let Some(entered)=changed{if let Some(r)=RUNTIME.get(){r.log.info(&format!("native kept tooltip fixture hit: entity={} category={} entered={entered}",c.r12,bonus.category()));}}}
}

#[cfg(feature="test-hooks")]
pub fn fixture_enabled()->bool {
 std::env::var("AZCOOP_TEST_BONUS_TOOLTIP").as_deref()==Ok("1") || std::env::var("AZCOOP_TEST_GLYPH_RETENTION").as_deref()==Ok("1")
}
