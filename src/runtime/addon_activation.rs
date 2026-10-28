//! Process-local activation of our owned addon, without a game CVar ABI.
//! Patch only the game's NtReadFile import. A constant-time prefix filter runs
//! before checking exact Windows file identity; no path/global file redirection.
use super::*;
use std::{ffi::c_void,fs::File,os::windows::io::AsRawHandle};
use windows_sys::{Win32::{Foundation::{HANDLE,GetLastError,SetLastError},Storage::FileSystem::{GetFileInformationByHandle,BY_HANDLE_FILE_INFORMATION},System::{Memory::{VirtualProtect,PAGE_READWRITE}}}};
const PREFIX:&[u8]=b"local asxNativeActive = 0 -- ASX_LOCAL_UI_GATE";
const MARK:usize=24;
#[repr(C)]
struct IoStatus {status:usize,information:usize}
type Read=unsafe extern "system" fn(HANDLE,HANDLE,*mut c_void,*mut c_void,*mut IoStatus,*mut c_void,u32,*mut i64,*mut u32)->i32;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Identity {volume:u32,high:u32,low:u32}
fn identity(handle:HANDLE)->Option<Identity> {
 let mut info:BY_HANDLE_FILE_INFORMATION=unsafe{std::mem::zeroed()};
 if unsafe{GetFileInformationByHandle(handle,&mut info)}==0{return None}
 Some(Identity{volume:info.dwVolumeSerialNumber,high:info.nFileIndexHigh,low:info.nFileIndexLow})
}
struct State {original:usize,slots:Vec<usize>,file:Identity,log:Option<&'static Log>,seen:AtomicBool}
static STATE:OnceLock<State>=OnceLock::new();
fn activate(expected:Identity,handle:HANDLE,bytes:&mut[u8])->bool {
 if bytes.len()<PREFIX.len()||!bytes.starts_with(PREFIX){return false}
 if identity(handle)!=Some(expected){return false}
 debug_assert_eq!(PREFIX[MARK],b'0');bytes[MARK]=b'1';true
}
unsafe extern "system" fn read(handle:HANDLE,event:HANDLE,apc:*mut c_void,context:*mut c_void,io:*mut IoStatus,buffer:*mut c_void,size:u32,offset:*mut i64,key:*mut u32)->i32 {
 let Some(state)=STATE.get()else{return 0xc000000du32 as i32};
 let original:Read=std::mem::transmute(state.original);
 let result=original(handle,event,apc,context,io,buffer,size,offset,key);let error=GetLastError();
 #[cfg(feature = "test-hooks")]
 let timing = performance::start(0);
 if result==0&&!buffer.is_null()&&!io.is_null() {
  let n=(*io).information.min(size as usize);
  // Do not scan MPQs or arbitrary file contents. Only the first fixed bytes
  // can match; at most one extra file-identity query for our owned script.
  if n>=PREFIX.len()&&std::slice::from_raw_parts(buffer as *const u8,PREFIX.len())==PREFIX {
   if activate(state.file,handle,std::slice::from_raw_parts_mut(buffer as *mut u8,n))&&!state.seen.swap(true,Ordering::Relaxed){if let Some(log)=state.log {log.info("ASX retained UI script activated locally; disk source unchanged");}}
  }
 }
 #[cfg(feature = "test-hooks")]
 drop(timing);
 SetLastError(error);result
}
fn word(local:&Local,at:usize)->Result<u32,String>{Ok(u32::from_le_bytes(local.read(at,4)?.try_into().map_err(|_|"Truncated import field")?))}
fn qword(local:&Local,at:usize)->Result<usize,String>{Ok(u64::from_le_bytes(local.read(at,8)?.try_into().map_err(|_|"Truncated import pointer")?)as usize)}
fn imports(local:&Local)->Result<Vec<(usize,usize)>,String>{
 let base=local.base;let pe=word(local,base+0x3c)? as usize;
 let optional=base+pe+24;
 if local.read(optional,2)?!=0x20bu16.to_le_bytes(){return Err("ASX requires PE32+ imports".into())}
 let table=word(local,optional+112+8)? as usize;
 if table==0||table>=local.image_size{return Err("Missing game import table".into())}
 let mut slots=vec![];
 for index in 0..1024 {
  let at=base+table+index*20;if at+20>base+local.image_size{return Err("Import table escaped image".into())}
  let names=word(local,at)? as usize;let dll=word(local,at+12)? as usize;let addresses=word(local,at+16)? as usize;
  if names==0&&dll==0&&addresses==0{break}
  if names==0||addresses==0{continue}
  let name=local.read(base+dll,64)?;let end=name.iter().position(|b|*b==0).unwrap_or(64);let name=String::from_utf8_lossy(&name[..end]).to_ascii_lowercase();
  if name!="ntdll.dll"{continue}
  for i in 0..4096 {
   let field=names+i*8;let slot=addresses+i*8;
   if field+8>local.image_size||slot+8>local.image_size{return Err("Import thunk escaped image".into())}
   let hint=qword(local,base+field)?;if hint==0{break}if hint&(1usize<<63)!=0{continue}
   if hint+16>local.image_size{return Err("Import name escaped image".into())}
   let name=local.read(base+hint+2,11)?;
   if name==b"NtReadFile\0"{slots.push((base+slot,qword(local,base+slot)?));}
  }
 }
 if slots.is_empty(){return Err("Game does not import synchronous NtReadFile".into())}
 Ok(slots)
}
pub fn install(local:&Local,log:&'static Log)->Result<(),String>{
 let home=std::env::var_os("BENILLA_HOME").map(std::path::PathBuf::from).unwrap_or(std::env::current_exe().map_err(|e|e.to_string())?.parent().ok_or("Missing executable folder")?.join("benilla-config"));
 let path=home.join("AddOns/ASX/Presentation.lua");let file=File::open(&path).map_err(|e|format!("ASX addon activation source unavailable: {e}"))?;
 let id=identity(file.as_raw_handle()).ok_or("Cannot identify owned ASX addon")?;
 let bytes=std::fs::read(&path).map_err(|e|e.to_string())?;
 if !bytes.starts_with(PREFIX){return Err("ASX addon activation header changed".into())}
 let slots=imports(local)?;let original=slots[0].1;
 if original==0||slots.iter().any(|(_,p)|*p!=original){return Err("Conflicting NtReadFile imports; ASX activation refused".into())}
 STATE.set(State{original,slots:slots.iter().map(|(p,_)|*p).collect(),file:id,log:Some(log),seen:AtomicBool::new(false)}).map_err(|_|"ASX addon activation already installed")?;
 for(slot,_)in slots {
  let mut previous=0;
  if unsafe{VirtualProtect(slot as *const c_void,8,PAGE_READWRITE,&mut previous)}==0{return Err("Cannot protect owned game import slot".into())}
  unsafe{(&*(slot as *const AtomicUsize)).store(read as *const ()as usize,Ordering::SeqCst);}
  let mut ignored=0;if unsafe{VirtualProtect(slot as *const c_void,8,previous,&mut ignored)}==0{return Err("Cannot restore game import protection".into())}
 }
 Ok(())
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn game_import_parser_finds_the_standard_file_reader_in_the_test_image() {
  let local=Local::game().unwrap();let slots=imports(&local).unwrap();assert!(!slots.is_empty());
  assert!(slots.iter().all(|(p,old)|*p>=local.base&&*p<local.base+local.image_size&&*old!=0));
 }
 unsafe extern "system" fn native_read(_file:HANDLE,event:HANDLE,apc:*mut c_void,context:*mut c_void,io:*mut IoStatus,buffer:*mut c_void,n:u32,offset:*mut i64,key:*mut u32)->i32 {
  assert_eq!(event as usize,0x11);assert_eq!(apc as usize,0x22);assert_eq!(context as usize,0x33);assert_eq!(*offset,7);assert_eq!(*key,9);
  SetLastError(1234);if n==0{return 0xc000000du32 as i32}
  assert_eq!(n as usize,PREFIX.len());std::ptr::copy_nonoverlapping(PREFIX.as_ptr(),buffer as *mut u8,PREFIX.len());(*io).information=PREFIX.len();0
 }
 #[test]
 fn native_reader_preserves_all_nine_arguments_status_and_last_error() {
  let path=std::env::temp_dir().join(format!("asx-read-abi-{}.lua",std::process::id()));std::fs::write(&path,PREFIX).unwrap();let file=File::open(&path).unwrap();
  let id=identity(file.as_raw_handle()).unwrap();STATE.set(State{original:native_read as *const ()as usize,slots:vec![],file:id,log:None,seen:AtomicBool::new(false)}).ok().unwrap();
  let mut io=IoStatus{status:0,information:0};let(mut offset,mut key)=(7,9);let mut buffer=vec![0u8;PREFIX.len()];
  let result=unsafe{read(file.as_raw_handle(),0x11usize as HANDLE,0x22usize as *mut _,0x33usize as *mut _,&mut io,buffer.as_mut_ptr()as *mut _,buffer.len()as u32,&mut offset,&mut key)};
  assert_eq!(result,0);assert_eq!(unsafe{GetLastError()},1234);assert_eq!(buffer[MARK],b'1');assert_eq!(std::fs::read(&path).unwrap(),PREFIX);
  let failed=unsafe{read(file.as_raw_handle(),0x11usize as HANDLE,0x22usize as *mut _,0x33usize as *mut _,&mut io,buffer.as_mut_ptr()as *mut _,0,&mut offset,&mut key)};
  assert_eq!(failed,0xc000000du32 as i32);assert_eq!(unsafe{GetLastError()},1234);drop(file);std::fs::remove_file(path).unwrap();
 }
 #[test]
 fn only_the_identified_addon_buffer_is_marked_and_disk_bytes_stay_native() {
   let dir=std::env::temp_dir().join(format!("asx-read-{}",std::process::id()));std::fs::create_dir_all(&dir).unwrap();
  let path=dir.join("Presentation.lua");let other=dir.join("other.lua");
  std::fs::write(&path,PREFIX).unwrap();std::fs::write(&other,PREFIX).unwrap();
  let file=File::open(&path).unwrap();let different=File::open(&other).unwrap();let id=identity(file.as_raw_handle()).unwrap();
  let mut b=PREFIX.to_vec();assert!(!activate(id,different.as_raw_handle(),&mut b));assert_eq!(b,PREFIX);
  assert!(!activate(id,file.as_raw_handle(),&mut b[..10]));assert!(activate(id,file.as_raw_handle(),&mut b));assert_eq!(b[MARK],b'1');
  assert_eq!(std::fs::read(&path).unwrap(),PREFIX);drop(file);drop(different);std::fs::remove_file(path).unwrap();std::fs::remove_file(other).unwrap();std::fs::remove_dir(dir).unwrap();
 }
}

/// Restore only slots still owned by this module; never overwrite another hook.
pub fn restore() {
 let Some(state)=STATE.get()else{return};
 for slot in &state.slots {
  let pointer=unsafe{&*(*slot as *const AtomicUsize)};
  if pointer.load(Ordering::Acquire)!=read as *const ()as usize {continue}
  let mut old=0;
  if unsafe{VirtualProtect(*slot as *const c_void,8,PAGE_READWRITE,&mut old)}==0 {continue}
  let _=pointer.compare_exchange(read as *const ()as usize,state.original,Ordering::SeqCst,Ordering::Relaxed);
  let mut ignored=0;unsafe{VirtualProtect(*slot as *const c_void,8,old,&mut ignored);}
 }
}
