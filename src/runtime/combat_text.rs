//! Connect native healing logs to the existing world-number renderer.
use super::*;
static DIRECT:AtomicUsize=AtomicUsize::new(0);
static PERIODIC:AtomicUsize=AtomicUsize::new(0);
crate::context_stub_rw!(direct_entry,0,heal,DIRECT);
crate::context_stub_rw!(periodic_entry,1,heal,PERIODIC);
unsafe extern "system" fn heal(c:&mut crate::stub::Context){
 let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};
 // Copy the already retained native run identities; never borrow state over calls.
 let players=nameplates::participants(r);
 #[cfg(feature="test-hooks")]if std::env::var("ASX_TEST_UI").as_deref()==Ok("1"){static N:AtomicUsize=AtomicUsize::new(0);if N.fetch_add(1,Ordering::Relaxed)<8{r.log.info(&format!("Native healing-log callback: periodic={} players={players:?}",c.id==1));}}
 if players.is_empty()||players.len()>asx_game::official::NATIVE_PLAYERS{return}
 let periodic=c.id==1;
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
 vec![crate::install::HookSpec{name:"native-damage-text-size",target:local_base()+entries[0].rva as usize,detour:asx_game::official_native::combat_text::damage_entry as *const() as usize,trampoline:&asx_game::official_native::combat_text::DAMAGE_NEXT},
 crate::install::HookSpec{name:"native-direct-heal-text",target:local_base()+entries[1].rva as usize,detour:direct_entry as *const() as usize,trampoline:&DIRECT},
 crate::install::HookSpec{name:"native-periodic-heal-text",target:local_base()+entries[2].rva as usize,detour:periodic_entry as *const() as usize,trampoline:&PERIODIC}]
}
