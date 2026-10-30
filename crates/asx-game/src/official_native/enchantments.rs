//! Selected native ring choices and scoped stat inputs; no saved item fields.
use super::*;
use crate::official::MutableMemory;
use crate::current as v;
use asx_gameplay::enchantments::RankedEnchantment;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RingChoice {pub game:u64,pub slot:u32,pub guid:u64,pub ring:u8,pub rank:u32,pub rarity:u64,pub value:u32}
fn q(m:&dyn Mem,a:u64)->Result<u64>{Ok(u64::from_le_bytes(m.read(a,8)?.try_into().map_err(|_|"Truncated ring word")?))}
fn u(m:&dyn Mem,a:u64)->Result<u32>{Ok(u32::from_le_bytes(m.read(a,4)?.try_into().map_err(|_|"Truncated ring integer")?))}
impl Adapter {
 pub fn supports_enchantments(self)->bool{true}
 fn ring_definition(self,m:&dyn Mem,base:u64,definition:u64)->Result<Option<u8>>{
  if !self.supports_enchantments(){return Ok(None)}
  let Some(offset)=definition.checked_sub(base+self.ring_table())else{return Ok(None)};
  if offset>=9*v::RING_STRIDE||offset%v::RING_STRIDE!=0{return Ok(None)}
  let ring=(offset/v::RING_STRIDE)as u8;let key=v::RING_KEYS[ring as usize].as_bytes();
  if q(m,definition+8)?!=key.len()as u64||m.read(q(m,definition)?,key.len())?!=key||m.read(definition+0x45,1)?!=[0]{return Err("Native ring definition changed".into())}
  Ok(Some(ring))
 }
 pub fn ring_owned(self,m:&dyn Mem,game:u64,ring:u8,base:u64)->Result<Option<(u32,u32)>>{
  if !self.supports_enchantments()||ring>=9{return Ok(None)}
  let count=q(m,game+self.field1201(v::RING_INVENTORY[1]))?;let ptr=q(m,game+self.field1201(v::RING_INVENTORY[0]))?;
  if count>16||(count>0&&(ptr<0x10000||ptr.checked_add(count*24).is_none())){return Err("Invalid native jewelry extent".into())}
  for n in 0..count {let row=ptr+n*24;if q(m,row)?==base+self.ring_table()+u64::from(ring)*v::RING_STRIDE{let rank=u(m,row+20)?;let value=u(m,row+16)?;if rank==0||rank>100000||!f32::from_bits(value).is_finite(){return Err("Invalid native ring rank/value".into())}return Ok(Some((rank,value)))}}
  Ok(None)
 }
 pub fn ring_actor_owned(self,m:&dyn Mem,game:u64,slot:u32,guid:u64,ring:u8,base:u64)->Result<Option<(u32,u32)>>{let row=self.support_row(m,game,slot)?;if q(m,row+self.row_field(row==game,0x478))?!=guid{return Err("Ring actor identity changed".into())}self.ring_owned(m,row,ring,base)}
 pub fn ring_card(self,m:&dyn Mem,base:u64,game:u64,output:u64)->Result<Option<RingChoice>>{
  if !self.supports_enchantments(){return Ok(None)}
  if q(m,output+0x78)?!=0x8000000000000002{return Ok(None)}
  let Some(ring)=self.ring_definition(m,base,q(m,output+0x80)?)?else{return Ok(None)};
  let Some((rank,value))=self.ring_owned(m,game,ring,base)?else{return Ok(None)};
  let rarity=q(m,output+0x98)?;if rarity>5{return Err("Invalid ring rarity".into())}
  let Some((slot,guid))=self.run_source(m,game)?else{return Ok(None)};
  Ok(Some(RingChoice{game,slot,guid,ring,rank,rarity,value}))
 }
 pub fn reward_choice_source(self,m:&dyn Mem,game:u64,index:u64)->Result<Option<(u32,u64)>>{
  if m.read(game+self.field1201(0xca2),1)?!=[2]||m.read(game+self.field1201(0xca0),1)?==[255]{return Ok(None)}
  let count=q(m,game+self.field1201(0x680))?;let ptr=q(m,game+self.field1201(0x678))?;
  if count>16||index>=count{return Ok(None)}
  if ptr<0x10000{return Err("Invalid reward choice storage".into())}
  self.run_source(m,game)
 }
 pub fn ring_selected(self,m:&dyn Mem,base:u64,game:u64,index:u64)->Result<Option<RingChoice>>{
  if !self.supports_enchantments()||m.read(game+self.field1201(0xca2),1)?!=[2]||m.read(game+self.field1201(0xca0),1)?==[255]{return Ok(None)}
  let count=q(m,game+self.field1201(0x680))?;let ptr=q(m,game+self.field1201(0x678))?;
  if count>16||index>=count{return Ok(None)}if ptr<0x10000{return Err("Invalid ring choice storage".into())}
  self.ring_card(m,base,game,ptr+index*0xa0)
 }
 pub fn ring_stat_values(self,m:&dyn Mem,game:u64)->Result<[f32;8]>{if !self.supports_enchantments(){return Err("Unreviewed ring stat snapshot".into())}let mut out=[0.;8];for(i,at)in v::RING_STAT_OUTPUTS.into_iter().enumerate(){out[i]=f32::from_bits(u(m,game+self.field1201(at))?);if !out[i].is_finite(){return Err("Invalid ring stat value".into())}}Ok(out)}
 pub fn ring_input_header(self,m:&dyn Mem,game:u64)->Result<Vec<u8>>{if !self.supports_enchantments(){return Err("Unreviewed ring stat input".into())}m.read(game+self.field1201(v::RING_STAT_INPUT),24)}
 pub fn ring_fixture_pick(self,m:&dyn Mem,base:u64,game:u64)->Result<Option<u64>>{
  if !self.supports_enchantments()||m.read(game+self.field1201(0xca2),1)?!=[2]{return Ok(None)}let n=q(m,game+self.field1201(0x680))?;let ptr=q(m,game+self.field1201(0x678))?;if n>16{return Err("Invalid fixture choices".into())}let mut best=None;
  for index in 0..n{let c=ptr+index*0xa0;if q(m,c+0x78)?!=0x8000000000000002{continue}let Some(ring)=self.ring_definition(m,base,q(m,c+0x80)?)?else{continue};if matches!(ring,5|7|8){continue}let owned=self.ring_owned(m,game,ring,base)?;let score=owned.map_or(1,|(rank,_)|if rank<14{100}else{0});if best.is_none_or(|(_,s)|score>s){best=Some((index,score));}}
  Ok(best.map(|(index,_)|index))
 }
 pub fn ring_stats(self,m:&dyn MutableMemory,game:u64,bonuses:&[RankedEnchantment])->Result<Option<RingStatLoan>>{
  if bonuses.len()>2{return Err("Invalid ring stat scope".into())}
  let entries=bonuses.iter().map(|e|(v::RING_STAT_KINDS[e.kind as usize],e.value())).collect::<Vec<_>>();
  self.temporary_stat_input(m,game,&entries)
 }


}

impl Adapter{
 pub unsafe fn enchantment_ui_draw(self,base:usize,parent:u64,font:u64,ui:BonusUi,e:asx_gameplay::enchantments::RankedEnchantment)->Result<()>{self.bonus_header_draw(base,parent,font,ui,&e.name(),e.icon())}
}

pub use super::stat_inputs::StatInputLoan as RingStatLoan;
impl Adapter {pub fn restore_ring_stats(self,m:&dyn MutableMemory,loan:RingStatLoan)->Result<()>{self.restore_stat_input(m,loan)}}
