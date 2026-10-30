//! Shared retained UI context, observed at native presentation boundaries only.
use super::Runtime;
use std::sync::atomic::{AtomicU64,Ordering};
static SCRIPT:AtomicU64=AtomicU64::new(0);
pub fn capture(script:u64){SCRIPT.store(script,Ordering::Release)}
pub fn reset(){SCRIPT.store(0,Ordering::Release)}
pub fn participants(r:&Runtime)->Vec<u64>{r.adapter.native_ui_participants(&r.local,SCRIPT.load(Ordering::Acquire)).unwrap_or_default()}
pub fn minimap(r:&Runtime,owner:u64)->Option<asx_game::official::EventMinimap>{
 let value=r.adapter.event_minimap(&r.local,SCRIPT.load(Ordering::Acquire),owner);
 #[cfg(feature="test-hooks")]if std::env::var("ASX_TEST_EVENT_POLISH").as_deref()==Ok("1"){
  static SEEN:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
  if let Ok(Some(map))=&value{if !SEEN.swap(true,Ordering::Relaxed){r.log.info(&format!("events native minimap: center={:?} radius={}",map.center,map.radius));}}
  static ERRORS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
  if let Err(e)=&value{if ERRORS.fetch_add(1,Ordering::Relaxed)<3{r.log.warn(&format!("Event minimap read rejected: {e}"));}}
 }
 value.ok().flatten()
}

static LOOKUP:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
unsafe extern "system" fn lookup(appdata:u64)->u64{
 let original:unsafe extern "system" fn(u64)->u64=std::mem::transmute(LOOKUP.load(Ordering::Acquire));
 let root=original(appdata);
 if let Some(r)=super::RUNTIME.get(){if let Some(script)=r.adapter.ui_context_result(root){capture(script);}}
 root
}
/// Native UI lookup is independent of optional nameplate visibility. Forward the
/// owned result untouched; the observer performs no reads, allocations or locks.
pub fn install(r:&Runtime)->Result<(),String>{
 let e=r.adapter.ui_context_entry();
 if r.local.read(r.local.base+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Native UI context lookup changed".into())}
 crate::install::install(&[crate::install::HookSpec{name:"native-ui-context",target:r.local.base+e.rva as usize,detour:lookup as *const()as usize,trampoline:&LOOKUP}])
}
#[cfg(test)]mod tests{
 use super::*;
 unsafe extern "system" fn native(appdata:u64)->u64{assert_eq!(appdata,0x123456);0x654321}
 #[test]fn lookup_preserves_native_argument_and_owned_result(){LOOKUP.store(native as *const()as usize,Ordering::Release);assert_eq!(unsafe{lookup(0x123456)},0x654321);}
}

#[cfg(feature="test-hooks")]
pub(super) fn diagnostic_identity(r:&Runtime)->Option<u64>{
 r.adapter.native_ui_participants(&r.local,SCRIPT.load(Ordering::Acquire)).ok().and_then(|ids|ids.first().copied())
}
