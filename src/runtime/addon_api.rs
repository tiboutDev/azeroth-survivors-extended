//! Extend only the native virtual WorldStateFrame.lua read after native patches.
//! This is a selected game entry, not an OS import or an addon-file interceptor.
use super::{io, local_base, RUNTIME};
use std::sync::atomic::{AtomicUsize, Ordering};
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
const PATH:&[u8]=b"Interface\\FrameXML\\WorldStateFrame.lua";
type Reader = unsafe extern "system" fn(*mut u8,usize,usize,usize) -> *mut u8;
fn api_path(path:&[u8])->bool{
 path.len()==PATH.len() && path.iter().zip(PATH).all(|(a,b)|a.to_ascii_lowercase()==b.to_ascii_lowercase()||(*a==b'/'&&*b==b'\\'))
}
unsafe extern "system" fn read(output:*mut u8,assets:usize,path:usize,length:usize)->*mut u8{
 let original:Reader=std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
 let result=original(output,assets,path,length);
 if length==PATH.len(){
  if let Some(r)=RUNTIME.get().filter(|r|r.rules){
   if r.local.read(path,length).is_ok_and(|b|api_path(&b)){
    let extension=include_str!("../../native-ui/official-party.lua").replacen("local azsxNativeActive = 0","local azsxNativeActive = 1",1);
    match r.adapter.extend_addon_api(&io::Access{local:&r.local,adapter:r.adapter},output as u64,extension.as_bytes()){
     Ok(())=>r.log.info("AZSX addon API embedded in native initialization"),
     Err(e)=>r.log.warn(&format!("AZSX addon API initialization failed: {e}")),
    }
   }
  }
 }
 result
}
pub fn install()->Result<(),String>{
 let r=RUNTIME.get().ok_or("Missing addon API runtime")?;let e=r.adapter.addon_api_reader_entry();
 if r.local.read(local_base()+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Native addon API reader changed".into());}
 crate::install::install(&[crate::install::HookSpec{name:"official-addon-api",target:local_base()+e.rva as usize,detour:read as *const()as usize,trampoline:&ORIGINAL}])
}
#[cfg(test)]mod tests{
 use super::*;
 unsafe extern "system" fn original(out:*mut u8,a:usize,p:usize,n:usize)->*mut u8{assert_eq!((out as usize,a,p,n),(0x12345678,0x2222,0x3333,38));0x76543210usize as *mut u8}
 #[test]fn native_reader_forwards_all_arguments_and_result(){ORIGINAL.store(original as *const()as usize,Ordering::Release);assert_eq!(unsafe{read(0x12345678usize as *mut u8,0x2222,0x3333,38)}as usize,0x76543210);}
 #[test]fn only_exact_native_script_path_matches(){assert!(api_path(PATH));assert!(api_path(b"interface/framexml/worldstateframe.lua"));assert!(!api_path(b"Interface\\AddOns\\ASX\\Presentation.lua"));assert!(!api_path(b"Interface\\FrameXML\\WorldStateFrame.xml"));}
}
