//! Shared retained UI context, observed at native presentation boundaries only.
use super::Runtime;
use std::sync::atomic::{AtomicU64,AtomicUsize,Ordering};
static SCRIPT:AtomicU64=AtomicU64::new(0);
pub fn capture(script:u64){SCRIPT.store(script,Ordering::Release)}
pub fn reset(){SCRIPT.store(0,Ordering::Release)}
pub fn participants(r:&Runtime)->Vec<u64>{r.adapter.native_ui_participants(&r.local,SCRIPT.load(Ordering::Acquire)).unwrap_or_default()}
pub fn minimap(r:&Runtime,owner:u64)->Option<azsx_game::official::EventMinimap>{
 let value=r.adapter.event_minimap(&r.local,SCRIPT.load(Ordering::Acquire),owner);
 #[cfg(feature="test-hooks")]if std::env::var("AZSX_TEST_EVENT_POLISH").as_deref()==Ok("1"){
  static SEEN:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
  if let Ok(Some(map))=&value{if !SEEN.swap(true,Ordering::Relaxed){r.log.info(&format!("events native minimap: center={:?} radii={:?}",map.center,map.radii));}}
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
crate::context_stub_rw!(panel_entry, 0, panel, PANEL_ORIGINAL);
static PANEL_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
unsafe extern "system" fn panel(c: &mut crate::stub::Context) {
    let Some(r) = super::RUNTIME.get() else { return };
    let io = super::io::Access { local: &r.local, adapter: r.adapter };
    if let Err(e) = r.adapter.enrich_talent_panel(&io, r.local.base as u64, c.rbp, c.r14) {
        r.log.warn(&format!("Talent panel note rejected: {e}"));
    }
}
#[cfg(feature="test-hooks")]
mod panel_probe {
    use super::*;
    static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
    crate::context_stub_rw!(probe, 0, select, ORIGINAL);
    unsafe extern "system" fn select(c: &mut crate::stub::Context) {
        let Some(r) = super::super::RUNTIME.get() else { return };
        if r.local.read(c.rsp_entry as usize,8).ok().as_deref()
            != Some(&(r.local.base as u64 + 0x40c8dc).to_le_bytes())
            || r.local.read(c.rsp_entry as usize+0x40,1).ok().as_deref()!=Some(&[1]) { return; }
        // The native detail builder only borrows the hovered key at +0x70/+0x78.
        // Supply that key without modifying the panel, the catalog or its text.
        static SELECTION: std::sync::OnceLock<[usize;16]> = std::sync::OnceLock::new();
        let selection=SELECTION.get_or_init(|| {
            let mut state=[0;16];
            state[14]=b"fortified".as_ptr() as usize;
            state[15]=b"fortified".len();
            state
        });
        c.r9=selection.as_ptr() as u64;
    }
    pub(super) fn install(r:&Runtime)->Result<(),String>{
        let target=r.local.base+0x1dbb60;
        if r.local.read(target,5)?!=[0x55,0x41,0x57,0x41,0x56]{return Err("Native talent inspection probe changed".into());}
        crate::install::install(&[crate::install::HookSpec{name:"fixture-inspected-talent",target,detour:probe as *const() as usize,trampoline:&ORIGINAL}])
    }
}
pub fn install(r:&Runtime)->Result<(),String>{
 #[cfg(feature="test-hooks")]
 if std::env::var("AZSX_TEST_TALENT_PANEL").as_deref()==Ok("1"){panel_probe::install(r)?;}
 let p=r.adapter.talent_panel_entry();
 if r.local.read(r.local.base+p.rva as usize,p.bytes.len())?!=p.bytes{return Err("Native talent panel changed".into())}
 let e=r.adapter.ui_context_entry();
 if r.local.read(r.local.base+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Native UI context lookup changed".into())}
 crate::install::install(&[crate::install::HookSpec{name:"native-ui-context",target:r.local.base+e.rva as usize,detour:lookup as *const()as usize,trampoline:&LOOKUP},
 crate::install::HookSpec{name:"native-coop-talent-panel",target:r.local.base+p.rva as usize,detour:panel_entry as *const () as usize,trampoline:&PANEL_ORIGINAL}])
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
