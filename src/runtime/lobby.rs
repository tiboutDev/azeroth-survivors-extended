//! Prefix only native host submissions, before the native request is copied.
use super::{io, local_base, RUNTIME};
use std::sync::atomic::{AtomicUsize, Ordering};
static ORIGINAL:AtomicUsize=AtomicUsize::new(0);
type Submit=unsafe extern "system" fn(usize,usize,usize,u8);
unsafe extern "system" fn submit(resource:usize,writer:usize,hero:usize,mode:u8) {
    if mode==1 {
        if let Some(r)=RUNTIME.get().filter(|r|r.rules) {
            if let Err(e)=r.adapter.prefix_lobby_name(&io::Access{local:&r.local,adapter:r.adapter},resource as u64) {
                r.log.info(&format!("AZSX lobby prefix failed: {e}"));
            }
        }
    }
    let original:Submit=std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
    original(resource,writer,hero,mode);
}
pub fn install()->Result<(),String>{
    let r=RUNTIME.get().ok_or("Missing lobby runtime")?;let e=r.adapter.lobby_submit_entry();
    if r.local.read(local_base()+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Native lobby submission changed".into());}
    crate::install::install(&[crate::install::HookSpec{name:"official-lobby-prefix",target:local_base()+e.rva as usize,detour:submit as *const()as usize,trampoline:&ORIGINAL}])?;
    #[cfg(feature="test-hooks")]
    if std::env::var("AZSX_TEST_ADDON_API").as_deref()==Ok("1"){probe::run(r)?;}
    Ok(())
}

#[cfg(test)]mod tests{
 use super::*;
 static CALLS:AtomicUsize=AtomicUsize::new(0);
 unsafe extern "system" fn original(a:usize,b:usize,c:usize,mode:u8){assert_eq!((a,b,c),(0x1111,0x2222,0x3333));assert!(mode<=2);CALLS.fetch_add(1,Ordering::Relaxed);}
 #[test]fn native_submit_preserves_every_argument_and_calls_once_per_mode(){
  ORIGINAL.store(original as *const()as usize,Ordering::Release);
  for mode in 0..=2 {unsafe{submit(0x1111,0x2222,0x3333,mode)}}
  assert_eq!(CALLS.load(Ordering::Relaxed),3);
 }
}

#[cfg(feature="test-hooks")]
mod probe;

#[cfg(feature="test-hooks")]
pub(super) unsafe fn capture_probe(c:&mut crate::stub::Context)->bool{probe::capture(c)}
