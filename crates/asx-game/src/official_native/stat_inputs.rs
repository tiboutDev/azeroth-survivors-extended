//! Scoped additive inputs shared by jewelry and temporary effects.
use super::*;
use crate::official::MutableMemory;
use crate::current as v;
/// The native rebuild copies this POD input synchronously. Never retain a loan
/// across a callback return or allow a gameplay-state mutex to span its lifetime.
pub struct StatInputLoan {at:u64,header:Vec<u8>,replacement:Vec<u8>,owned:u64,size:usize}

impl Adapter {
 pub(super) fn temporary_stat_input(self,m:&dyn MutableMemory,game:u64,bonuses:&[(u32,f32)])->Result<Option<StatInputLoan>>{
  if bonuses.is_empty(){return Ok(None)}
  if !self.supports_enchantments()||bonuses.len()>8||self.run_source(m,game)?.is_none(){return Err("Invalid temporary stat scope".into())}
  let at=game+self.field1201(v::RING_STAT_INPUT);let header=m.read(at,24)?;if header.len()!=24{return Err("Truncated ring stat header".into())}
  let cap=u64::from_le_bytes(header[..8].try_into().map_err(|_|"Truncated ring stat header")?);let ptr=u64::from_le_bytes(header[8..16].try_into().unwrap());let len=u64::from_le_bytes(header[16..].try_into().unwrap());
  if len+bonuses.len()as u64>512||len>cap||cap>512||(len>0&&(ptr<0x10000||ptr.checked_add(len*8).is_none())){return Err("Invalid native additive stat input".into())}
  let mut bytes=if len>0{m.read(ptr,len as usize*8)?}else{vec![]};
  for (kind,value) in bonuses{bytes.extend_from_slice(&kind.to_le_bytes());bytes.extend_from_slice(&value.to_le_bytes());}
  let owned=m.allocate_native(&bytes)?;let n=bytes.len()as u64/8;let replacement=[n.to_le_bytes(),owned.to_le_bytes(),n.to_le_bytes()].concat();
  if let Err(e)=m.write(at,&replacement){let _=m.free_native(owned,bytes.len());return Err(e)}
  Ok(Some(StatInputLoan{at,header,replacement,owned,size:bytes.len()}))
 }
 pub fn restore_stat_input(self,m:&dyn MutableMemory,loan:StatInputLoan)->Result<()>{
  if m.read(loan.at,24)?!=loan.replacement{return Err("Native ring stat loan owner changed".into())}
  m.write(loan.at,&loan.header)?;m.free_native(loan.owned,loan.size)
 }
 pub fn consumable_stats(self,m:&dyn MutableMemory,game:u64,buffs:&[asx_gameplay::supplies::Consumable])->Result<Option<StatInputLoan>>{
  let mut seen=std::collections::BTreeSet::new();if buffs.len()>asx_gameplay::supplies::Family::ALL.len()||buffs.iter().any(|c|!(1..=3).contains(&c.tier)||!seen.insert(c.family)){return Err("Invalid consumable families".into())}
  let entries=buffs.iter().filter_map(|c|c.family.stat().map(|kind|(kind,c.value()))).collect::<Vec<_>>();self.temporary_stat_input(m,game,&entries)
 }
 pub fn apply_consumable_speed(self,m:&dyn MutableMemory,game:u64,value:f32)->Result<()>{
  if !value.is_finite()||!(0.0..=0.20).contains(&value)||self.run_source(m,game)?.is_none(){return Err("Invalid consumable speed scope".into())}
  let at=game+self.field117(v::BLESSING_STAT_FIELDS[4]);let bytes=m.read(at,4)?;
  let base=f32::from_le_bytes(bytes.try_into().map_err(|_|"Truncated native movement speed")?);
  if !base.is_finite()||base<0.{return Err("Invalid native movement speed".into())}
  m.write(at,&(base+value).to_le_bytes())
 }

}
