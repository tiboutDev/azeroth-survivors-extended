//! Retain native pill-shaped plates; transform presentation inputs before drawing.
use super::*;
static SCRIPT:AtomicU64=AtomicU64::new(0);
pub(super)fn participants(r:&Runtime)->Vec<u64>{r.adapter.native_ui_participants(&r.local,SCRIPT.load(Ordering::Acquire)).unwrap_or_default()}
pub(super)fn reset(){SCRIPT.store(0,Ordering::Release);}
static UPDATE:AtomicUsize=AtomicUsize::new(0);
static STYLE:AtomicUsize=AtomicUsize::new(0);
unsafe extern "system" fn update(out:u64,script:u64,index:u32,generation:u32,style:u64,rows:u64,len:u64,extra:u64)->u64{
 let original:unsafe extern "system" fn(u64,u64,u32,u32,u64,u64,u64,u64)->u64=std::mem::transmute(UPDATE.load(Ordering::Acquire));
 let copy=RUNTIME.get().filter(|r|r.rules).and_then(|r|{
  SCRIPT.store(script,Ordering::Release);
  let result=r.adapter.nameplate_rows(&r.local,r.local.base,script,rows,len);
  #[cfg(feature="test-hooks")]if std::env::var("ASX_TEST_UI").as_deref()==Ok("1"){static N:AtomicUsize=AtomicUsize::new(0);if N.fetch_add(1,Ordering::Relaxed)<4{r.log.info(&format!("Native plate input: len={len} rows={rows:#x} changed={:?}",result.as_ref().map(|v|v.is_some())));}}
  result.ok().flatten()
 });
 original(out,script,index,generation,style,copy.as_ref().map_or(rows,|r|r.as_ptr()as u64),len,extra)
}
crate::context_stub_rw!(style_entry,0,style,STYLE);
unsafe extern "system" fn style(c:&mut crate::stub::Context){
 let Some(r)=RUNTIME.get().filter(|r|r.rules)else{return};
 let Some(caller)=c.return_address().checked_sub(r.local.base as u64)else{return};
 let selected=r.adapter.nameplate_style(&r.local,caller,c.rbp,c.rcx,c.rdx,c.r8);
 #[cfg(feature="test-hooks")]if std::env::var("ASX_TEST_UI").as_deref()==Ok("1"){static N:AtomicUsize=AtomicUsize::new(0);if N.fetch_add(1,Ordering::Relaxed)<4{r.log.info(&format!("Native plate style callback: caller={caller:#x} result={:?}",selected.as_ref().map(|v|v.is_some())));}}
 let Ok(Some((scaled,cache,original)))=selected else{return};
 let call:unsafe extern "system" fn(u64,u64,*const f32)->u64=std::mem::transmute(STYLE.load(Ordering::Acquire));
 let result=call(c.rcx,c.rdx,scaled.as_ptr());
 // Native cache comparisons remain against the unmodified native settings. The
 // actual layout is already resized; unchanged frames must not be restyled.
 let _=r.adapter.restore_nameplate_style_cache(&io::Access{local:&r.local,adapter:r.adapter},cache,&original);
 c.skip=1;c.result=result;
 #[cfg(feature="test-hooks")]if std::env::var("ASX_TEST_UI").as_deref()==Ok("1"){static N:AtomicUsize=AtomicUsize::new(0);if N.fetch_add(1,Ordering::Relaxed)<8{r.log.info(&format!("Native plate layout: index={} original={original:?} scaled={scaled:?}",c.rdx));}}
}
pub(super)fn hooks(a:Adapter)->[crate::install::HookSpec;2]{let e=a.nameplate_entries();[
 crate::install::HookSpec{name:"native-plate-input",target:local_base()+e[0].rva as usize,detour:update as *const() as usize,trampoline:&UPDATE},
 crate::install::HookSpec{name:"native-plate-style",target:local_base()+e[1].rva as usize,detour:style_entry as *const() as usize,trampoline:&STYLE}]
}

#[cfg(test)]mod tests{
 use super::*;
 unsafe extern "system" fn native(out:u64,script:u64,index:u32,generation:u32,style:u64,rows:u64,len:u64,extra:u64)->u64{assert_eq!((out,script,index,generation,style,rows,len,extra),(1,2,3,4,5,6,7,8));0x123}
 #[test]fn native_plate_wrapper_preserves_all_eight_arguments_and_output(){UPDATE.store(native as *const()as usize,Ordering::Release);assert_eq!(unsafe{update(1,2,3,4,5,6,7,8)},0x123);}
}
