//! Private diagnostic transport. Reserved payloads never reach CVar storage.
use super::{Runtime,RUNTIME};
use std::sync::{OnceLock,atomic::{AtomicUsize,Ordering}};
const NAME:&[u8]=b"asx_diagnostic_mailbox";
static ORIGINAL:AtomicUsize=AtomicUsize::new(0);
static SESSION:OnceLock<(String,u64)>=OnceLock::new();
static SEQUENCE:AtomicUsize=AtomicUsize::new(0);
type Write=unsafe extern "system" fn(usize,*const u8,usize,*const u8,usize,u8)->u8;
fn validate(line:&str,session:&str,slot:u64)->Result<(usize,serde_json::Value),String>{
 let text=line.strip_prefix("ASX_DIAG ").ok_or("Missing diagnostic prefix")?;
 let value:serde_json::Value=serde_json::from_str(text).map_err(|_|"Invalid diagnostic JSON")?;
 let sequence=value["sequence"].as_u64().filter(|n|(1..=512).contains(n)).ok_or("Invalid diagnostic sequence")? as usize;
 if value["schema"]!=1 || value["session"]!=session || value["slot"]!=slot || !value["data"].is_object(){return Err("Diagnostic identity/schema mismatch".into())}
 let kind=value["kind"].as_str().ok_or("Missing diagnostic kind")?;
 if !["observer.started","observer.stopped","ui.supply_toast","ui.sayge_progress","ui.buffs","ui.tooltip","asx.ritual_model"].contains(&kind){return Err("Unknown diagnostic observation".into())}
 let time=value["clock"].as_f64().filter(|n|n.is_finite()&&*n>=0.).ok_or("Invalid diagnostic time")?;let _=time;
 Ok((sequence,value))
}
unsafe extern "system" fn write(cvars:usize,name:*const u8,len:usize,value:*const u8,size:usize,mode:u8)->u8{
 if len==NAME.len() && !name.is_null() && std::slice::from_raw_parts(name,len)==NAME{
  // Consume even malformed reserved writes: they cannot become persistent settings.
  if !value.is_null() && size<=16384{
   if let(Some((session,slot)),Ok(line))=(SESSION.get(),std::str::from_utf8(std::slice::from_raw_parts(value,size))){
    if let Ok((sequence,mut record))=validate(line,session,*slot){
     if SEQUENCE.compare_exchange(sequence-1,sequence,Ordering::AcqRel,Ordering::Acquire).is_ok(){
      if let Some(r)=RUNTIME.get(){record["observed_own_guid"]=super::ui_context::diagnostic_identity(r).map(serde_json::Value::from).unwrap_or(serde_json::Value::Null);r.log.info(&format!("ASX_DIAG {}",record));}
     }
    }
   }
  }
  return 0;
 }
 let original:Write=std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
 original(cvars,name,len,value,size,mode)
}
pub(super) fn install(r:&Runtime)->Result<(),String>{
 if std::env::var("ASX_LUA_DIAGNOSTICS").as_deref()!=Ok("1"){return Ok(())}
 let session=std::env::var("ASX_DIAGNOSTIC_SESSION").map_err(|_|"Missing diagnostic session")?;
 if session.is_empty()||session.len()>80||!session.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'-'){return Err("Invalid diagnostic session".into())}
 let slot=std::env::var("ASX_DIAGNOSTIC_SLOT").ok().and_then(|s|s.parse::<u64>().ok()).filter(|s|*s<2).ok_or("Invalid diagnostic slot")?;
 SESSION.set((session,slot)).map_err(|_|"Diagnostic sink already initialized")?;
 let e=r.adapter.diagnostic_writer_entry();
 if r.local.read(r.local.base+e.rva as usize,e.bytes.len())?!=e.bytes{return Err("Diagnostic sink native guard changed".into())}
 crate::install::install_optional(crate::install::HookSpec{name:"diagnostic-lua-observer",target:r.local.base+e.rva as usize,detour:write as *const()as usize,trampoline:&ORIGINAL})
}
#[cfg(test)]mod tests{
 use super::*;
 unsafe extern "system" fn native(c:usize,n:*const u8,l:usize,v:*const u8,s:usize,m:u8)->u8{assert_eq!(c,77);assert_eq!(std::slice::from_raw_parts(n,l),b"ordinary");assert_eq!(std::slice::from_raw_parts(v,s),b"yes");assert_eq!(m,1);9}
 #[test]fn ordinary_setter_keeps_all_arguments_and_return(){let _guard=crate::test_support::TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());ORIGINAL.store(native as *const()as usize,Ordering::Release);assert_eq!(unsafe{write(77,b"ordinary".as_ptr(),8,b"yes".as_ptr(),3,1)},9);}
 #[test]fn reserved_malformed_write_is_not_forwarded(){assert_eq!(unsafe{write(77,NAME.as_ptr(),NAME.len(),b"invalid".as_ptr(),7,1)},0);}
 #[test]fn rejects_foreign_sessions_and_malformed_records(){
  let row=r#"ASX_DIAG {"schema":1,"session":"unit","slot":0,"sequence":1,"clock":1,"kind":"ui.buffs","data":{}}"#;
  assert!(validate(row,"unit",0).is_ok());assert!(validate(row,"foreign",0).is_err());assert!(validate(row,"unit",1).is_err());assert!(validate(&row.replace("ui.buffs","unknown"),"unit",0).is_err());assert!(validate(&row.replace("\"sequence\":1","\"sequence\":999"),"unit",0).is_err());
 }
}
