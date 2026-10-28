//! Native replacement-tooltip construction; no Lua or native layout in gameplay rules.
use super::*;
use crate::official::{Entry,MutableMemory};
use asx_gameplay::kept_bonus::KeptBonus;
fn q(m:&dyn Mem,a:u64)->Result<u64>{Ok(u64::from_le_bytes(m.read(a,8)?.try_into().map_err(|_|"Truncated native tooltip word")?))}
impl Adapter{
 pub fn supports_bonus_tooltips(self)->bool{true}
 pub fn bonus_tooltip_entries(self)->Vec<Entry>{self.tooltip_bindings().iter().map(|(rva,bytes)|Entry{rva:*rva,bytes}).collect()}
 pub fn kept_tooltip_fixture_focus(self)->Option<Entry>{let(rva,bytes)=crate::official122::KEPT_TOOLTIP_FOCUS;Some(Entry{rva,bytes})}
 pub fn kept_tooltip_row(self,frame:u64)->Option<u64>{frame.checked_add_signed(crate::official122::KEPT_TOOLTIP_ROW)}
 /// Synthetic borrowed item descriptor is consumed synchronously by our body hook.
 /// The native background helper only reads rarity, then borrows this descriptor.
 pub fn kept_tooltip_descriptor(self,bonus:KeptBonus)->Option<[u64;11]>{
  Some([0,1,0,0,1,0,0,1,0,crate::official1201::KEPT_TOOLTIP_MARKER,u64::from(bonus.code())])
 }
 pub fn kept_tooltip_body(self,m:&dyn Mem,closure:u64)->Result<Option<(u64,KeptBonus)>>{
  if !self.supports_bonus_tooltips()||closure<0x10000{return Ok(None)}
  let item=q(m,closure+16)?;if item<0x10000||q(m,item+72)?!=crate::official1201::KEPT_TOOLTIP_MARKER{return Ok(None)}
  let code=q(m,item+80)?;let bonus=u16::try_from(code).ok().and_then(KeptBonus::decode).ok_or("Invalid ASX native tooltip identity")?;
  for at in [0,24,48]{if [q(m,item+at)?,q(m,item+at+8)?,q(m,item+at+16)?]!=[0,1,0]{return Err("Invalid ASX borrowed tooltip descriptor".into())}}
  let font=q(m,closure)?;if font<0x10000{return Err("Invalid ASX native tooltip font".into())}Ok(Some((font,bonus)))
 }
 /// Same owned command as native replacement links: three marker/interaction bytes.
 /// Queue and row belong to the current UI callback; no pointer is retained.
 pub unsafe fn kept_tooltip_target(self,m:&dyn MutableMemory,base:usize,row:u64)->Result<()> {
  if !self.supports_bonus_tooltips()||row<0x10000{return Err("Native tooltip target unavailable".into())}
  let entity=q(m,row)?;let primary=q(m,row+24)?;let queue=if primary!=0{primary}else{q(m,row+32)?};
  if queue<0x10000{return Err("Native tooltip command queue unavailable".into())}
  let mut capacity=q(m,queue)?;let mut len=q(m,queue+16)?;
  if len>capacity||capacity>268435456{return Err("Invalid native tooltip queue extent".into())}
  if capacity-len<24{
   type Reserve=unsafe extern "system" fn(u64,u64,u64,u64,u64);
   let reserve:Reserve=std::mem::transmute(base+self.tooltip_bindings()[4].0 as usize);
   reserve(queue,len,24,1,1);capacity=q(m,queue)?;len=q(m,queue+16)?;
   if len>capacity||capacity>268435456||capacity-len<24{return Err("Native tooltip reserve failed".into())}
  }
  let buffer=q(m,queue+8)?;if buffer<0x10000{return Err("Invalid native tooltip queue storage".into())}
  let mut command=[0u8;24];command[..8].copy_from_slice(&(base as u64+self.tooltip_bindings()[3].0).to_le_bytes());command[8..16].copy_from_slice(&entity.to_le_bytes());command[16..19].copy_from_slice(&[0,2,1]);
  m.write(buffer.checked_add(len).ok_or("Native tooltip queue overflow")?,&command)?;
  m.write(queue+16,&(len+24).to_le_bytes())
 }
 pub unsafe fn kept_tooltip_attach(self,m:&dyn MutableMemory,base:usize,frame:u64,font:u64,ui:super::BonusUi,bonus:KeptBonus)->Result<u64> {
  let assets=ui.assets;
  let row=self.kept_tooltip_row(frame).ok_or("Native tooltip row unavailable")?;
  if font<0x10000||assets<0x10000{return Err("Invalid native tooltip resources".into())}
  self.kept_tooltip_target(m,base,row)?;
  let descriptor=self.kept_tooltip_descriptor(bonus).ok_or("Native tooltip descriptor unavailable")?;
  type Draw=unsafe extern "system" fn(u64,u64,u64,*const u64);
  let draw:Draw=std::mem::transmute(base+self.tooltip_bindings()[2].0 as usize);
  draw(row,font,assets,descriptor.as_ptr());q(m,row)
 }
 pub fn kept_tooltip_title(self,m:&dyn MutableMemory,bundle:u64)->Result<()> {
  if !self.supports_bonus_tooltips(){return Ok(())}
  if m.read(bundle+crate::official117::GLYPH_STATUS_COLOR,4)?!=0u32.to_le_bytes(){return Err("Unexpected native tooltip title color".into())}
  m.write(bundle+crate::official117::GLYPH_STATUS_COLOR+4,&[1f32,0.82,0.,1.].into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>())
 }
 pub fn bonus_compact_size(self,m:&dyn crate::official::MutableMemory,caller:u64,bundle:u64)->Result<()>{
  if !self.supports_bonus_tooltips()||caller!=self.contact117(crate::official117::GLYPH_UI_HEADER_CALLER){return Ok(())}
  let v=crate::official117::GLYPH_UI_HEADER_NODE;
  m.write(bundle+v+crate::official117::GLYPH_UI_HEADER_HEIGHT,&[1u32.to_le_bytes(),20f32.to_le_bytes()].concat())?;
  m.write(bundle+v+crate::official117::GLYPH_UI_HEADER_MARGIN_TOP,&[1u32.to_le_bytes(),8f32.to_le_bytes()].concat())
 }
 pub fn bonus_compact_style(self,m:&dyn crate::official::MutableMemory,bundle:u64)->Result<()>{
  if !self.supports_bonus_tooltips(){return Ok(())}
  let at=bundle+crate::official117::GLYPH_STATUS_FONT_SIZE;let raw=m.read(at,4)?;if raw!=12f32.to_le_bytes()&&raw!=15f32.to_le_bytes(){return Ok(())}
  if m.read(bundle+crate::official117::GLYPH_STATUS_COLOR,4)?!=0u32.to_le_bytes(){return Ok(())}
  m.write(at,&12f32.to_le_bytes())?;m.write(bundle+crate::official117::GLYPH_STATUS_COLOR+4,&[0.65f32,0.65,0.65,1.].into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>())
 }
}

impl Adapter{fn tooltip_bindings(self)-> &'static[(u64,&'static[u8]);5]{&crate::official122::KEPT_TOOLTIP_BINDINGS}}
