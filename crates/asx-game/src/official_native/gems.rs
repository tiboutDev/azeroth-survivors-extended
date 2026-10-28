//! Exact 0.120.1 amulet identities and scoped gem stat input.
use super::*;
use crate::official::MutableMemory;
use crate::official1201 as v;
use asx_gameplay::gems::Gem;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct AmuletChoice{pub game:u64,pub slot:u32,pub guid:u64,pub amulet:u64,pub previous:Option<u64>,pub rank:u32,pub fingerprint:u64}
fn q(m:&dyn Mem,a:u64)->Result<u64>{Ok(u64::from_le_bytes(m.read(a,8)?.try_into().map_err(|_|"Truncated amulet word")?))}
impl Adapter{
 fn amulet_key(self,m:&dyn Mem,pointer:u64,len:u64)->Result<u64>{
  if pointer<0x10000||len==0||len>64{return Err("Invalid native amulet key".into())}
  let bytes=m.read(pointer,len as usize)?;
  if let Some(index)=v::AMULET_KEYS.iter().position(|key|key.as_bytes()==bytes){return Ok(index as u64)}
  // The native generator uses this key namespace for spell-specific legendaries.
  if bytes.starts_with(b"legendary:")&&bytes.len()>10&&bytes[10..].iter().all(|b|b.is_ascii_alphanumeric()||matches!(*b,b'_'|b':'|b'-')){return Ok(bytes.iter().fold(0xcbf29ce484222325u64,|h,b|(h^u64::from(*b)).wrapping_mul(0x100000001b3))|1<<63)}
  Err("Unreviewed amulet identity".into())
 }
 pub fn amulet_owned(self,m:&dyn Mem,game:u64)->Result<Option<u64>>{
  if !self.supports_enchantments(){return Ok(None)}
  let at=game+self.field1201(v::AMULET_OWNED);if q(m,at)?==u64::MAX{return Ok(None)}
  self.amulet_key(m,q(m,at+8)?,q(m,at+16)?).map(Some)
 }
 pub fn amulet_card(self,m:&dyn Mem,base:u64,game:u64,output:u64)->Result<Option<AmuletChoice>>{
  if !self.supports_enchantments()||q(m,output+0x78)?!=v::AMULET_CHOICE_TAG{return Ok(None)}
  let Some((slot,guid))=self.run_source(m,game)?else{return Ok(None)};
  let p=q(m,output+0x80)?;if p<0x10000{return Err("Invalid amulet card pointer".into())}
  let amulet=self.amulet_key(m,q(m,p+8)?,q(m,p+16)?)?;
  if amulet<16{let def=base+self.amulet_table()+amulet*v::AMULET_STRIDE;
  if self.amulet_key(m,q(m,def)?,q(m,def+8)?)?!=amulet{return Err("Native amulet definition changed".into())}}
  let previous=self.amulet_owned(m,game)?;
  let rank=self.amulet_rank(m,game)?;
  // Stable while the same native offer is retained, including reentrant redraws.
  let fingerprint=m.read(p,0x70)?.iter().fold(0xcbf29ce484222325u64,|h,b|(h^u64::from(*b)).wrapping_mul(0x100000001b3));
  Ok(Some(AmuletChoice{game,slot,guid,amulet,previous,rank,fingerprint}))
 }
 pub fn amulet_rank(self,m:&dyn Mem,game:u64)->Result<u32>{if self.amulet_owned(m,game)?.is_none(){return Ok(0)}let rank=u32::from_le_bytes(m.read(game+self.field1201(v::AMULET_RANK),4)?.try_into().map_err(|_|"Truncated amulet rank")?);if rank==0||rank>100000{return Err("Invalid owned amulet rank".into())}Ok(rank)}
 pub fn amulet_selected(self,m:&dyn Mem,base:u64,game:u64,index:u64)->Result<Option<AmuletChoice>>{
  if !self.supports_enchantments(){return Ok(None)}let count=q(m,game+self.field1201(0x680))?;let ptr=q(m,game+self.field1201(0x678))?;
  if count>16||index>=count{return Ok(None)}if ptr<0x10000{return Err("Invalid amulet choice storage".into())}
  self.amulet_card(m,base,game,ptr+index*0xa0)
 }
 pub fn gem_stats(self,m:&dyn MutableMemory,game:u64,gem:Gem)->Result<Option<enchantments::RingStatLoan>>{self.jewelry_stat_input(m,game,&[(v::RING_STAT_KINDS[gem as usize],gem.value())])}
 pub fn amulet_fixture_pick(self,m:&dyn Mem,base:u64,game:u64)->Result<Option<u64>>{
  let count=q(m,game+self.field1201(0x680))?;let ptr=q(m,game+self.field1201(0x678))?;if count>16||count>0&&ptr<0x10000{return Err("Invalid amulet fixture extent".into())}
  let mut upgrade=None;for index in 0..count{if let Some(c)=self.amulet_card(m,base,game,ptr+index*0xa0)?{if c.previous!=Some(c.amulet){return Ok(Some(index))}upgrade=Some(index);}}Ok(upgrade)
 }
}

impl Adapter{
 pub unsafe fn gem_ui_draw(self,base:usize,parent:u64,font:u64,ui:BonusUi,gem:asx_gameplay::gems::Gem)->Result<()>{self.bonus_header_draw(base,parent,font,ui,gem.name(),gem.icon())}
}
