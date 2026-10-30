//! Safe hook installation and removal via MinHook.
//!
//! MinHook suspends every other thread in the process while it writes a jump,
//! and if a suspended thread's instruction pointer sits inside the bytes being
//! overwritten it moves that IP onto the equivalent spot in the relocated
//! trampoline. So no game thread ever runs half-patched code. We enable all
//! queued hooks in one `apply_queued`, i.e. one suspend/resume of the process.
//!
//! `uninstall` disables and removes every hook (another single suspend) and is
//! also the kill switch: after it returns, the game runs the original code.

use minhook::MinHook;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};

/// One installed hook: target address and the trampoline slot to fill.
pub struct HookSpec {
    pub name: &'static str,
    pub target: usize,
    pub detour: usize,
    pub trampoline: &'static AtomicUsize,
}

static INSTALLED: AtomicBool = AtomicBool::new(false);
static TARGETS: std::sync::Mutex<Vec<usize>> = std::sync::Mutex::new(Vec::new());

// Only a patch installed here may substitute for a native entry guard. Record
// the exact overwritten prefix; unchanged guard tails still have to match.
#[derive(Clone)]
struct OwnedPatch {target:usize,before:[u8;5],after:[u8;5]}
impl OwnedPatch {
 fn accepts(&self,target:usize,actual:&[u8],expected:&[u8])->bool{
  self.target==target && expected.len()>=5 && actual.len()==expected.len()
   && expected[..5]==self.before && actual[..5]==self.after && actual[5..]==expected[5..]
 }
}
static PATCHES:std::sync::Mutex<Vec<OwnedPatch>>=std::sync::Mutex::new(Vec::new());
unsafe fn prefix(target:usize)->[u8;5]{std::ptr::read_unaligned(target as *const [u8;5])}
pub fn owns_entry_guard(target:usize,actual:&[u8],expected:&[u8])->bool{
 PATCHES.lock().unwrap_or_else(|e|e.into_inner()).iter().any(|p|p.accepts(target,actual,expected))
}

/// Install one atomic batch. Earlier batches retain their ownership and teardown.
pub fn install(specs:&[HookSpec])->Result<(),String>{
 if specs.is_empty(){return Ok(())}
 let existing=TARGETS.lock().unwrap_or_else(|e|e.into_inner()).clone();
 let mut batch=std::collections::BTreeSet::new();
 for spec in specs{
  if spec.target==0 || spec.detour==0 || existing.contains(&spec.target) || !batch.insert(spec.target){
   return Err(format!("Native hook requires one owner: {}",spec.name));
  }
 }
 let mut created=Vec::new();
 let mut records=Vec::new();
 let result=(||->Result<(),String>{
  for spec in specs{
   let before=unsafe{prefix(spec.target)};
   let original=unsafe{MinHook::create_hook(spec.target as *mut c_void,spec.detour as *mut c_void)}.map_err(|e|format!("create {} hook: {e:?}",spec.name))?;
   created.push(spec.target);
   spec.trampoline.store(original as usize,Relaxed);
   unsafe{MinHook::queue_enable_hook(spec.target as *mut c_void)}.map_err(|e|format!("queue {} hook: {e:?}",spec.name))?;
   records.push((spec.target,before));
  }
  unsafe{MinHook::apply_queued()}.map_err(|e|format!("apply hooks: {e:?}"))?;
  Ok(())
 })();
 if let Err(error)=result{
  for target in &created{let _=unsafe{MinHook::queue_disable_hook(*target as *mut c_void)};}
  let _=unsafe{MinHook::apply_queued()};
  for target in &created{let _=unsafe{MinHook::remove_hook(*target as *mut c_void)};}
  for spec in specs{if created.contains(&spec.target){spec.trampoline.store(0,Relaxed);}}
  return Err(error);
 }
 let patches=records.into_iter().map(|(target,before)|OwnedPatch{target,before,after:unsafe{prefix(target)}}).collect::<Vec<_>>();
 PATCHES.lock().unwrap_or_else(|e|e.into_inner()).extend(patches);
 TARGETS.lock().unwrap_or_else(|e|e.into_inner()).extend(created);
 INSTALLED.store(true,Relaxed);Ok(())
}
/// Disable every owned batch. Native entries and ownership guards retire together.
pub fn uninstall(){
 crate::runtime::restore_addon_activation();
 if !INSTALLED.swap(false,Relaxed){return}
 let targets=std::mem::take(&mut *TARGETS.lock().unwrap_or_else(|e|e.into_inner()));
 PATCHES.lock().unwrap_or_else(|e|e.into_inner()).clear();
 for target in &targets{let _=unsafe{MinHook::queue_disable_hook(*target as *mut c_void)};}
 let _=unsafe{MinHook::apply_queued()};
 for target in &targets{let _=unsafe{MinHook::remove_hook(*target as *mut c_void)};}
}
/// Optional failures roll back their own batch and preserve already installed hooks.
pub fn install_optional(spec:HookSpec)->Result<(),String>{install(&[spec])}

#[cfg(test)]
mod owned_guard_tests {
 use super::*;
 #[test]
 fn permits_only_the_registered_patch_over_the_reviewed_native_prefix(){
  let expected=[0x55,0x41,0x57,0x41,0x56,0x41,0x55];
  let patch=OwnedPatch{target:0x1000,before:expected[..5].try_into().unwrap(),after:[0xe9,1,2,3,4]};
  let mut actual=expected;actual[..5].copy_from_slice(&patch.after);
  assert!(patch.accepts(0x1000,&actual,&expected));
  assert!(!patch.accepts(0x2000,&actual,&expected));
  let mut foreign=actual;foreign[1]^=1;assert!(!patch.accepts(0x1000,&foreign,&expected));
  let mut tail=actual;tail[6]^=1;assert!(!patch.accepts(0x1000,&tail,&expected));
  let mut wrong_original=expected;wrong_original[1]^=1;assert!(!patch.accepts(0x1000,&actual,&wrong_original));
  assert!(!patch.accepts(0x1000,&actual[..4],&expected[..4]));
 }
}

#[cfg(test)]
mod batch_tests{
 use super::*;
 static ORIGINAL_A:AtomicUsize=AtomicUsize::new(0);
 static ORIGINAL_B:AtomicUsize=AtomicUsize::new(0);
 #[unsafe(naked)]unsafe extern "system" fn a()->u32{core::arch::naked_asm!("mov eax,11","ret")}
 #[unsafe(naked)]unsafe extern "system" fn b()->u32{core::arch::naked_asm!("mov eax,17","ret")}
 unsafe extern "system" fn detour_a()->u32{let original:unsafe extern "system" fn()->u32=std::mem::transmute(ORIGINAL_A.load(Relaxed));original()+1}
 unsafe extern "system" fn detour_b()->u32{let original:unsafe extern "system" fn()->u32=std::mem::transmute(ORIGINAL_B.load(Relaxed));original()+2}
 #[test]fn successive_batches_preserve_forwarding_ownership_and_full_teardown(){
  let _lock=crate::test_support::TEST_LOCK.lock().unwrap_or_else(|e|e.into_inner());
  let spec_a=||HookSpec{name:"unit-a",target:a as *const()as usize,detour:detour_a as *const()as usize,trampoline:&ORIGINAL_A};
  let spec_b=||HookSpec{name:"unit-b",target:b as *const()as usize,detour:detour_b as *const()as usize,trampoline:&ORIGINAL_B};
  install(&[spec_a()]).unwrap();install_optional(spec_b()).unwrap();
  assert_eq!((unsafe{a()},unsafe{b()}),(12,19));
  assert!(install(&[spec_a()]).is_err());
  assert_eq!((unsafe{a()},unsafe{b()}),(12,19));
  uninstall();assert_eq!((unsafe{a()},unsafe{b()}),(11,17));
  assert!(TARGETS.lock().unwrap().is_empty());assert!(PATCHES.lock().unwrap().is_empty());
 }
}
