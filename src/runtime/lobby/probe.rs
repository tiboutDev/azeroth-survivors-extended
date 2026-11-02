//! Execute the real native host request constructor with a private capture sink.
//! No request enters the native queue or reaches a lobby service.
use super::Submit;
use crate::runtime::{io, local_base, Runtime, RUNTIME};
use std::sync::atomic::{AtomicUsize, Ordering};
use azsx_game::official::MutableMemory;
const WRITER:usize=0x415a5358;

static HOSTS:AtomicUsize=AtomicUsize::new(0);

pub(super) unsafe fn capture(c:&mut crate::stub::Context)->bool{
 if c.rcx as usize!=WRITER{return false}
 let event=c.rdx as *mut u8;
 let r=RUNTIME.get().unwrap();let io=io::Access{local:&r.local,adapter:r.adapter};
 let kind=std::ptr::read_unaligned(event.cast::<u32>());
 if kind==0x17{
  let ptr=std::ptr::read_unaligned(event.add(16).cast::<usize>());
  let len=std::ptr::read_unaligned(event.add(24).cast::<usize>());
  let name=std::str::from_utf8(std::slice::from_raw_parts(ptr as *const u8,len)).unwrap();
  if name=="[AZSX] Migration trial"&&*event.add(80)==0&&*event.add(81)==5{
   HOSTS.fetch_add(1,Ordering::Relaxed);r.log.info("AZSX lobby native request verified: prefixed name, private visibility, five-player capacity; network suppressed");
  }
 }
 let offsets:&[usize]=match kind{0xc=>&[8,32],0x17=>&[8,32,56],_=>&[]};
 for offset in offsets{
  let cap=std::ptr::read_unaligned(event.add(*offset).cast::<usize>());
  let ptr=std::ptr::read_unaligned(event.add(*offset+8).cast::<u64>());
  let _=io.free_native(ptr,cap);
 }
 c.skip=1;c.result=0;true
}
fn string(io:&io::Access<'_>,bytes:&mut[u8],at:usize,text:&str)->Result<(),String>{
 let ptr=io.allocate_native(text.as_bytes())?;
 bytes[at..at+8].copy_from_slice(&(text.len()as u64).to_le_bytes());
 bytes[at+8..at+16].copy_from_slice(&ptr.to_le_bytes());bytes[at+16..at+24].copy_from_slice(&(text.len()as u64).to_le_bytes());Ok(())
}
pub(super)fn run(r:&Runtime)->Result<(),String>{
 let base=local_base();let e=r.adapter.menu_queue_entry();let target=base+e.rva as usize;
 let actual=r.local.read(target,e.bytes.len())?;
 if !crate::install::owns_entry_guard(target,&actual,e.bytes){return Err("Native menu queue is not owned for probe".into());}
 let io=io::Access{local:&r.local,adapter:r.adapter};let mut resource=vec![0u8;0x200];let mut hero=vec![0u8;0x40];
 for(at,text)in [(0xd0,"Migration trial"),(0xe8,"private-test"),(0x100,"fixture")]{string(&io,&mut resource,at,text)?;}
 for(at,text)in [(0,"Test hero"),(24,"Test class")]{string(&io,&mut hero,at,text)?;}
 resource[0x1f8]=1;resource[0x1fd]=5;
 let native:Submit=unsafe{std::mem::transmute(base+r.adapter.lobby_submit_entry().rva as usize)};
 unsafe{native(resource.as_mut_ptr()as usize,WRITER,hero.as_mut_ptr()as usize,1);native(resource.as_mut_ptr()as usize,WRITER,hero.as_mut_ptr()as usize,1);}
 for (bytes,offsets)in [(&resource[..],&[0xd0usize,0xe8,0x100][..]),(&hero[..],&[0usize,24][..])]{
  for at in offsets{let cap=u64::from_le_bytes(bytes[*at..*at+8].try_into().unwrap());let ptr=u64::from_le_bytes(bytes[*at+8..*at+16].try_into().unwrap());io.free_native(ptr,cap as usize)?;}
 }
 if HOSTS.load(Ordering::Relaxed)!=2{return Err("Native lobby prefix probe failed".into());}Ok(())
}
