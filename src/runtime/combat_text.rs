//! Connect native healing logs to the existing world-number renderer.
use super::*;
static DIRECT:AtomicUsize=AtomicUsize::new(0);
static PERIODIC:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_rw!(direct_entry,0,heal,DIRECT);
crate::context_stub_rw!(periodic_entry,1,heal,PERIODIC);
unsafe extern "system" fn heal(c:&mut crate::stub::Context){
 let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};
 // Copy the already retained native run identities; never borrow state over calls.
 let players=super::ui_context::participants(r);
 #[cfg(feature="test-hooks")]if std::env::var("ASX_TEST_UI").as_deref()==Ok("1"){static N:AtomicUsize=AtomicUsize::new(0);if N.fetch_add(1,Ordering::Relaxed)<8{r.log.info(&format!("Native healing-log callback: periodic={} players={players:?}",c.id==1));}}
 if players.is_empty()||players.len()>asx_game::official::NATIVE_PLAYERS{return}
 let periodic=c.id==1;
 #[cfg(feature="test-hooks")]
 if std::env::var("ASX_TEST_UI_REMAINING").as_deref()==Ok("1"){
  static DIRECT_HEADERS:AtomicUsize=AtomicUsize::new(0);static PERIODIC_HEADERS:AtomicUsize=AtomicUsize::new(0);
  let counter=if periodic{&PERIODIC_HEADERS}else{&DIRECT_HEADERS};
  if counter.fetch_add(1,Ordering::Relaxed)<8{r.log.info(&format!("remaining UI heal packet: periodic={periodic} bytes={:?}",r.adapter.fixture_heal_packet(&r.local,c.rbp,periodic)));}
 }

 if let Ok(rows)=r.adapter.native_heal_texts(&r.local,c.rbp,periodic,&players){
  for row in rows{
   let result=r.adapter.draw_native_heal_text(&io::Access{local:&r.local,adapter:r.adapter},r.local.base,c.rbp,periodic,row);
   #[cfg(feature="test-hooks")]
   if std::env::var("ASX_TEST_UI").as_deref()==Ok("1"){
    static COUNT:AtomicUsize=AtomicUsize::new(0);
    if COUNT.fetch_add(1,Ordering::Relaxed)<24{r.log.info(&format!("native heal text: periodic={periodic} event={row:?} result={result:?}"));}
   }
   #[cfg(not(feature="test-hooks"))]let _=result;
  }
 }
}
pub(super)fn hooks(adapter:Adapter)->Vec<crate::install::HookSpec>{
 let entries=adapter.combat_text_entries();
 vec![crate::install::HookSpec{name:"native-inline-periodic-text",target:local_base()+entries[7].rva as usize,detour:inline_periodic_entry as *const()as usize,trampoline:&INLINE_PERIODIC},
 crate::install::HookSpec{name:"native-fractional-heal-text",target:local_base()+entries[4].rva as usize,detour:fractional_entry as *const()as usize,trampoline:&FRACTIONAL},
 crate::install::HookSpec{name:"native-direct-heal-text",target:local_base()+entries[0].rva as usize,detour:direct_entry as *const() as usize,trampoline:&DIRECT},
 crate::install::HookSpec{name:"native-periodic-heal-text",target:local_base()+entries[1].rva as usize,detour:periodic_entry as *const() as usize,trampoline:&PERIODIC}]
}

static INLINE_PERIODIC:AtomicUsize=AtomicUsize::new(0);
// These seams read RSI/RDI: only the full context stub captures nonvolatiles.
crate::context_stub_rw!(inline_periodic_entry,1,fractional,INLINE_PERIODIC);
static FRACTIONAL:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_rw!(fractional_entry,0,fractional,FRACTIONAL);
type TextSource=Option<(u64,Option<(u64,u32)>)>;
thread_local!{static PERIODIC_TEXT_SOURCE:std::cell::Cell<TextSource>=const{std::cell::Cell::new(None)};}
pub(super) fn with_periodic_text_source<T>(game:u64,owner:Option<(u64,u32)>,f:impl FnOnce()->T)->T{
 struct Restore(TextSource);impl Drop for Restore{fn drop(&mut self){PERIODIC_TEXT_SOURCE.with(|v|v.set(self.0));}}
 if FRACTIONS_ACTIVE.load(Ordering::Relaxed){
  if let Some(r)=RUNTIME.get().filter(|r|r.rules){
   if let Ok(clock)=r.adapter.fractional_heal_clock(&r.local,game){if let Ok(mut state)=FRACTIONS.lock(){if let Some(accumulator)=state.get_mut(&game){accumulator.observe_clock(clock);}}}
  }
 }
 let _restore=Restore(PERIODIC_TEXT_SOURCE.with(|v|v.replace(Some((game,owner)))));f()
}
static FRACTIONS_ACTIVE:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
static FRACTIONS:std::sync::Mutex<std::collections::BTreeMap<u64,asx_gameplay::healing_text::Accumulator>>=std::sync::Mutex::new(std::collections::BTreeMap::new());
pub(super)fn reset(){if let Ok(mut state)=FRACTIONS.lock(){state.clear();FRACTIONS_ACTIVE.store(false,Ordering::Relaxed);}}
unsafe extern "system" fn fractional(c:&crate::stub::Context){
 let Some((game,owner))=PERIODIC_TEXT_SOURCE.with(|v|v.get()).filter(|(game,_)|*game==c.rsi)else{return};
 #[cfg(test)]{*FRACTIONAL_CAPTURE.lock().unwrap()=Some((c.rsi,if c.id==1{c.r8 as u32}else{c.rdi as u32},f32::from_bits(c.xmm[1]as u32)));}
 let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};
 // xmm1 is the final, clamped HP gain, after native talents and ASX penalties.
 let amount=f32::from_bits(c.xmm[1]as u32);let original_spell=if c.id==1{c.r8 as u32}else{c.rdi as u32};
 if !amount.is_finite()||amount<=0.||c.id==0&&original_spell!=0&&amount>=1.{return}
 let Ok((clock,actor))=r.adapter.fractional_heal_context(&r.local,game)else{return};
 let(caster,spell)=owner.unwrap_or((actor.guid,original_spell));
 let key=asx_gameplay::healing_text::Key{caster,target:actor.guid,spell};
 FRACTIONS_ACTIVE.store(true,Ordering::Relaxed);
 let ready={let Ok(mut state)=FRACTIONS.lock()else{return};if state.len()>=4&&!state.contains_key(&game){return}state.entry(game).or_default().credit(key,clock,amount,false)};
 let Some(amount)=ready else{return};
 let message=asx_game::official_native::HealText{caster,target:actor.guid,amount,critical:false};
 let result=r.adapter.emit_fractional_heal_text(&io::Access{local:&r.local,adapter:r.adapter},r.local.base,game,message,spell);
 #[cfg(feature="test-hooks")]
 if std::env::var("ASX_TEST_UI_REMAINING").as_deref()==Ok("1"){r.log.info(&format!("fractional HoT text: event={message:?} spell={spell} result={result:?}"));}
 #[cfg(not(feature="test-hooks"))]let _=result;
}

#[cfg(test)]mod fractional_scope_tests{
 use super::*;
 #[test]fn nested_native_and_foreign_scopes_restore_copied_ownership(){
  assert!(PERIODIC_TEXT_SOURCE.with(|v|v.get()).is_none());
  with_periodic_text_source(7,None,||{
   assert_eq!(PERIODIC_TEXT_SOURCE.with(|v|v.get()),Some((7,None)));
   with_periodic_text_source(7,Some((33,139)),||{
    assert_eq!(PERIODIC_TEXT_SOURCE.with(|v|v.get()),Some((7,Some((33,139)))));
   });
   assert_eq!(PERIODIC_TEXT_SOURCE.with(|v|v.get()),Some((7,None)));
  });
  assert!(PERIODIC_TEXT_SOURCE.with(|v|v.get()).is_none());
 }
}

#[cfg(test)]static FRACTIONAL_CAPTURE:std::sync::Mutex<Option<(u64,u32,f32)>>=std::sync::Mutex::new(None);
#[cfg(test)]mod fractional_native_register_tests{
 use super::*;
 #[unsafe(naked)]unsafe extern "C" fn passthrough(){core::arch::naked_asm!("ret");}
 macro_rules! probe{($name:ident,$entry:path)=>{
  #[unsafe(naked)]unsafe extern "system" fn $name(_game:u64,_spell:u64,_amount:f32)->f32{
   core::arch::naked_asm!("push rsi","push rdi","sub rsp,40","mov rsi,rcx","mov rdi,rdx","mov r8,rdx","movaps xmm1,xmm2","call {entry}","movss xmm0,xmm1","add rsp,40","pop rdi","pop rsi","ret",entry=sym $entry);
  }
 }}
 probe!(shared_probe,fractional_entry);probe!(inline_probe,inline_periodic_entry);
 #[test]fn actual_periodic_seams_capture_nonvolatile_owners_and_preserve_heal_values(){
  let old=FRACTIONAL.swap(passthrough as *const()as usize,Ordering::SeqCst);
  let inline=INLINE_PERIODIC.swap(passthrough as *const()as usize,Ordering::SeqCst);
  for call in [shared_probe as unsafe extern "system" fn(u64,u64,f32)->f32,inline_probe]{
   *FRACTIONAL_CAPTURE.lock().unwrap()=None;
   let result=with_periodic_text_source(0x123450,None,||unsafe{call(0x123450,139,0.25)});
   assert_eq!(result,0.25);assert_eq!(*FRACTIONAL_CAPTURE.lock().unwrap(),Some((0x123450,139,0.25)));
  }
  FRACTIONAL.store(old,Ordering::SeqCst);INLINE_PERIODIC.store(inline,Ordering::SeqCst);
 }
}
