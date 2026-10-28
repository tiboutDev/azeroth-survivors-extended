//! Borrow the native world-text message writer only during its native log callback.
use super::*;
use crate::official::MutableMemory;
use std::sync::atomic::AtomicUsize;
pub static DAMAGE_NEXT:AtomicUsize=AtomicUsize::new(0);
#[cfg(all(target_os="windows",target_arch="x86_64"))]
static THIRD:f32=1.0/3.0;
/// The selected renderer keeps the current entry in r14 and its freshly calculated
/// size in xmm7. No stored size changes; preserve every other register and flags.
#[cfg(all(target_os="windows",target_arch="x86_64"))]
#[unsafe(naked)]
pub unsafe extern "C" fn damage_entry(){core::arch::naked_asm!(
 "pushfq",
 "cmp byte ptr [r14 + 0x38], 0", "je 2f",
 "cmp byte ptr [r14 + 0x38], 2", "jne 3f",
 "2:", "cmp dword ptr [r14 + 0x34], {green}", "je 3f",
 "mulss xmm7, dword ptr [rip + {third}]",
 "3:", "popfq", "jmp qword ptr [rip + {next}]",
 green=const crate::official125::HEAL_TEXT_COLOR,third=sym THIRD,next=sym DAMAGE_NEXT);}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct HealText{pub caster:u64,pub target:u64,pub amount:u32,pub critical:bool}
impl Adapter{
 pub fn combat_text_entries(self)->[super::super::official::Entry;5]{
  use crate::official125 as v;
  [v::DAMAGE_TEXT_SIZE,v::DIRECT_HEAL_TEXT,v::PERIODIC_HEAL_TEXT,v::TEXT_ENTITY,v::TEXT_RESERVE].map(|(rva,bytes)|super::super::official::Entry{rva,bytes})
 }
 pub fn native_heal_texts(self,m:&dyn Mem,bp:u64,periodic:bool,players:&[u64])->Result<Vec<HealText>>{
  let u64_at=|at|->Result<u64>{Ok(u64::from_le_bytes(m.read(at,8)?.try_into().map_err(|_|"Truncated text identity")?))};
  let payload=bp+if periodic{0x4b0}else{0xd0};
  let(target,caster)=if periodic{(u64_at(payload+0x18)?,u64_at(payload+0x20)?)}else{(u64_at(payload)?,u64_at(payload+8)?)};
  if target==0||!players.contains(&target)||!players.contains(&caster){return Ok(vec![])}
  let mut result=vec![];
  if periodic{
   let len=u64_at(payload+0x10)?;let cap=u64_at(payload)?;let rows=u64_at(payload+8)?;
   if len>32||cap<len||cap>4096{return Err("Invalid periodic text rows".into())}
   for row in m.read(rows,(len*20)as usize)?.chunks_exact(20){
    // Native periodic variant 1 is healing; variant 0 already emits damage text.
    if u32::from_le_bytes(row[..4].try_into().unwrap())==1{
     let amount=u32::from_le_bytes(row[4..8].try_into().unwrap());
     if amount>0{result.push(HealText{caster,target,amount,critical:false});}
    }
   }
  }else{
   let raw=m.read(payload+0x14,5)?;let amount=u32::from_le_bytes(raw[..4].try_into().unwrap());
   if amount>0{result.push(HealText{caster,target,amount,critical:raw[4]==1});}
  }
  Ok(result)
 }
 /// Append exactly one message per actual native healing log. No simulation,
 /// healing, meters or network calls. Native renderer owns lifetime and animation.
 pub unsafe fn draw_native_heal_text(self,m:&dyn MutableMemory,base:usize,bp:u64,periodic:bool,heal:HealText)->Result<bool>{
  let word=|at|->Result<u64>{Ok(u64::from_le_bytes(m.read(at,8)?.try_into().map_err(|_|"Truncated native text writer")?))};
  let objects=word(bp+if periodic{0x238}else{0x228})?;
  let query=m.read(bp+if periodic{0x378}else{0x368},24)?;
  type Entity=unsafe extern "system" fn(u64,u64,u64,u64,*const u8)->u64;
  let resolve:Entity=std::mem::transmute(base+crate::official125::TEXT_ENTITY.0 as usize);
  let entity=resolve(heal.target,objects,0,0,query.as_ptr());
  if entity as u32==0{return Ok(false)}
  let params=bp+if periodic{0x358}else{0x348};
  let writer=word(params+0x70)?;let tick_ptr=word(params+0x80)?;let tick=m.read(params+0x8c,4)?;
  let len=word(writer+0x30)?;let cap=word(writer+0x20)?;
  if len>=4096||cap<len||cap>1<<20{return Err("Native text capacity rejected".into())}
  let sequence=word(writer+0x40)?;let next_sequence=sequence.checked_add(1).ok_or("Native text sequence overflow")?;
  // Validate the changed-tick destination before allocating any owned value.
  m.read(tick_ptr,4)?;
  type Reserve=unsafe extern "system" fn(u64);
  if len==cap{let reserve:Reserve=std::mem::transmute(base+crate::official125::TEXT_RESERVE.0 as usize);reserve(writer+0x20);}
  let buffer=word(writer+0x28)?;
  if word(writer+0x20)?<=len{return Err("Native text reserve did not grow".into())}
  let cursor=buffer.checked_add(len.checked_mul(0x38).ok_or("Text offset overflow")?).ok_or("Text pointer overflow")?;
  m.read(cursor,0x38)?;
  let text=format!("+{}",heal.amount);let size=text.len();
  let owned=m.allocate_native(text.as_bytes())?;
  // The native message destructor owns this String after the single commit.
  let mut row=[0u8;0x38];
  row[..8].copy_from_slice(&(size as u64).to_le_bytes());row[8..16].copy_from_slice(&owned.to_le_bytes());row[16..24].copy_from_slice(&(size as u64).to_le_bytes());
  row[24..28].copy_from_slice(&1u32.to_le_bytes());row[28..32].copy_from_slice(&crate::official125::HEAL_TEXT_COLOR.to_le_bytes());row[32..40].copy_from_slice(&entity.to_le_bytes());row[40]=if heal.critical{2}else{0};row[48..].copy_from_slice(&sequence.to_le_bytes());
  // Destinations were validated above; no native callback holds this borrowed row.
  if let Err(error)=m.write(cursor,&row){m.free_native(owned,size)?;return Err(error)}
  m.write(writer+0x30,&(len+1).to_le_bytes())?;m.write(writer+0x40,&next_sequence.to_le_bytes())?;m.write(tick_ptr,&tick)?;
  Ok(true)
 }
}

#[cfg(test)]mod tests{
 use super::*;
 struct Bytes{data:Vec<u8>}
 impl Mem for Bytes{fn read(&self,at:u64,n:usize)->Result<Vec<u8>>{self.data.get(at as usize..at as usize+n).map(|b|b.to_vec()).ok_or("Outside fixture".into())}}
 fn put(m:&mut Bytes,at:usize,n:u64){m.data[at..at+8].copy_from_slice(&n.to_le_bytes());}
 fn adapter()->Adapter{Adapter::select(crate::official125::SHA256).unwrap()}
 #[test]fn direct_native_amount_critical_and_both_identities_are_required(){
  let mut m=Bytes{data:vec![0;0x1000]};put(&mut m,0xd0,11);put(&mut m,0xd8,22);m.data[0xe4..0xe8].copy_from_slice(&37u32.to_le_bytes());m.data[0xe8]=1;
  assert_eq!(adapter().native_heal_texts(&m,0,false,&[11,22]).unwrap(),vec![HealText{target:11,caster:22,amount:37,critical:true}]);
  assert!(adapter().native_heal_texts(&m,0,false,&[11]).unwrap().is_empty());
  assert!(adapter().native_heal_texts(&m,0,false,&[22]).unwrap().is_empty());
 }
 #[test]fn periodic_heals_preserve_packet_order_and_exclude_native_damage(){
  let mut m=Bytes{data:vec![0;0x1000]};put(&mut m,0x4b0,3);put(&mut m,0x4b8,0x700);put(&mut m,0x4c0,3);put(&mut m,0x4c8,11);put(&mut m,0x4d0,22);
  for(i,(kind,amount))in [(0u32,77u32),(1,13),(1,17)].into_iter().enumerate(){m.data[0x700+i*20..0x708+i*20].copy_from_slice(&[kind.to_le_bytes(),amount.to_le_bytes()].concat());}
  let rows=adapter().native_heal_texts(&m,0,true,&[11,22]).unwrap();assert_eq!(rows.iter().map(|r|r.amount).collect::<Vec<_>>(),vec![13,17]);assert!(rows.iter().all(|r|r.target==11&&r.caster==22&&!r.critical));
  put(&mut m,0x4c0,33);assert!(adapter().native_heal_texts(&m,0,true,&[11,22]).is_err());
 }
 #[cfg(all(target_os="windows",target_arch="x86_64"))]
 #[test]fn damage_size_uses_fresh_baseline_preserves_other_categories_and_heals(){
  #[unsafe(naked)]unsafe extern "C" fn end(){core::arch::naked_asm!("pushfq","pop rax","mov [r14 + 0x20], rax","movaps xmm0, xmm7","ret");}
  #[unsafe(naked)]unsafe extern "system" fn invoke(_row:*const u8,_size:f32)->f32{core::arch::naked_asm!("push r14","sub rsp, 48","movaps [rsp + 32], xmm7","mov r14, rcx","movaps xmm7, xmm1","cmp rsp, rsp","call {entry}","movaps xmm7, [rsp + 32]","add rsp, 48","pop r14","ret",entry=sym damage_entry);}
  DAMAGE_NEXT.store(end as *const() as usize,std::sync::atomic::Ordering::Release);
  for cat in 0..6{
   let mut row=[0u8;64];row[0x38]=cat;
   for _ in 0..3{let got=unsafe{invoke(row.as_ptr(),0.0275)};let expected=if cat==0||cat==2{0.0275/3.0}else{0.0275};assert!((got-expected).abs()<1e-8);assert_eq!(u64::from_le_bytes(row[0x20..0x28].try_into().unwrap())&0x41,0x40);}
   row[0x34..0x38].copy_from_slice(&crate::official125::HEAL_TEXT_COLOR.to_le_bytes());assert_eq!(unsafe{invoke(row.as_ptr(),0.0275)},0.0275);
  }
 }
}
