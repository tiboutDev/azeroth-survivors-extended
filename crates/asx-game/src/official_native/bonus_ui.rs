//! Shared native reward annotation widgets; versioned pointers stay here.
use super::*;
use crate::official::{Entry,MutableMemory};
use crate::official117 as v;
fn q(m:&dyn Mem,a:u64)->Result<u64>{Ok(u64::from_le_bytes(m.read(a,8)?.try_into().map_err(|_|"Truncated UI word")?))}
#[derive(Clone,Copy)]pub struct BonusUi {pub(super) assets:u64,color:[u8;20]}
impl Adapter {
 pub fn bonus_ui_entries(self)->Vec<Entry>{if !self.supports_extensions(){return vec![]}vec![Entry{rva:self.contact117(v::GLYPH_UI_RVA),bytes:self.extension_bytes117(0x2f6f70,v::GLYPH_UI_BYTES)},Entry{rva:self.contact117(v::GLYPH_LINE_RVA),bytes:self.extension_bytes117(0x1070a0,v::GLYPH_LINE_BYTES)},Entry{rva:self.contact117(v::GLYPH_ICON_RVA),bytes:self.extension_bytes117(0x2365e0,v::GLYPH_ICON_BYTES)},Entry{rva:self.contact117(v::GLYPH_UI_ACTION_RVA),bytes:self.extension_bytes117(0x329159,v::GLYPH_UI_ACTION_BYTES)},Entry{rva:self.contact117(v::GLYPH_TEXT_NODE_RVA),bytes:self.extension_bytes117(0x2be1d0,v::GLYPH_TEXT_NODE_BYTES)}]}
 pub fn bonus_ui_context(self,m:&dyn Mem,closure:u64)->Result<BonusUi>{
  let assets=q(m,closure+v::GLYPH_UI_ASSETS)?;let color=q(m,closure+v::GLYPH_UI_COLOR)?;
  if assets<0x10000||color<0x10000{return Err("Invalid glyph UI resources".into())}
  Ok(BonusUi{assets,color:m.read(color,20)?.try_into().map_err(|_|"Truncated glyph UI color")?})
 }
 pub fn bonus_ui_size(self,m:&dyn MutableMemory,caller:u64,bundle:u64)->Result<()>{
  if !self.supports_extensions()||caller!=self.contact117(v::GLYPH_UI_HEADER_CALLER){return Ok(())}
  for(offset,value)in[(v::GLYPH_UI_HEADER_WIDTH,208f32),(v::GLYPH_UI_HEADER_HEIGHT,28f32)]{
   m.write(bundle+v::GLYPH_UI_HEADER_NODE+offset,&[1u32.to_le_bytes(),value.to_le_bytes()].concat())?;
  }
  // Separate the ASX glyph from native upgrade lines; keep its own header compact.
  m.write(bundle+v::GLYPH_UI_HEADER_NODE+v::GLYPH_UI_HEADER_MARGIN_TOP,&[1u32.to_le_bytes(),12f32.to_le_bytes()].concat())?;
  // The reviewed card constructor sets Column (1); glyph header uses Row (0).
  if m.read(bundle+v::GLYPH_UI_HEADER_NODE+v::GLYPH_UI_HEADER_DIRECTION,1)?==[1]{m.write(bundle+v::GLYPH_UI_HEADER_NODE+v::GLYPH_UI_HEADER_DIRECTION,&[0])?;}
  let justify=bundle+v::GLYPH_UI_HEADER_NODE+v::GLYPH_UI_HEADER_JUSTIFY;
  if m.read(justify,1)?==[0]{m.write(justify,&[v::GLYPH_UI_CENTER])?;}
  Ok(())
 }
 /// Scoped by the glyph status renderer; never called for ordinary spell lines.
 /// Native rich-text spans build a 15px TextFont and a tag-0 sRGBA TextColor.
 pub fn bonus_ui_status(self,m:&dyn MutableMemory,bundle:u64,kept:bool)->Result<()>{
  if !self.supports_extensions(){return Ok(())}
  if m.read(bundle+v::GLYPH_STATUS_FONT_SIZE,4)?!=15f32.to_le_bytes()||m.read(bundle+v::GLYPH_STATUS_COLOR,4)?!=0u32.to_le_bytes(){return Err("Unexpected glyph status text style".into())}
  m.write(bundle+v::GLYPH_STATUS_FONT_SIZE,&12f32.to_le_bytes())?;
  let rgba=if kept{[0.65f32,0.65,0.65,1.]}else{[1f32,0.72,0.3,1.]};
  m.write(bundle+v::GLYPH_STATUS_COLOR+4,&rgba.into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>())
 }
 /// Inside the glyph drawback only: recolor native green numeric spans.
 /// Preserve ordinary text, font size and opacity.
 pub fn bonus_ui_drawback(self,m:&dyn MutableMemory,bundle:u64)->Result<()>{
  if !self.supports_extensions(){return Ok(())}
  if m.read(bundle+v::GLYPH_STATUS_COLOR,4)?!=0u32.to_le_bytes(){return Err("Unexpected glyph drawback color variant".into())}
  let bytes=m.read(bundle+v::GLYPH_STATUS_COLOR+4,16)?;
  let rgba:[f32;4]=std::array::from_fn(|i|f32::from_le_bytes(bytes[i*4..i*4+4].try_into().unwrap()));
  if rgba.iter().all(|v|v.is_finite())&&rgba[1]>rgba[0]&&rgba[1]>rgba[2]{m.write(bundle+v::GLYPH_STATUS_COLOR+4,&[1f32,0.2,0.2].into_iter().flat_map(f32::to_le_bytes).collect::<Vec<_>>())?;}
  Ok(())
 }
 /// Only the reviewed JNE reads ZF here. Do not mutate the disabled boolean:
 /// the subsequent label uses that boolean to choose its foreground color.
 pub fn bonus_ui_no_action(self,flags:u64)->u64{flags & !0x40}
 /// Borrow live ChildSpawner and font only during this native card build.
 /// The cached native icon loader returns one owned Handle transferred to Card.
 pub unsafe fn kept_bonus_ui_draw(self,base:usize,parent:u64,font:u64,ui:BonusUi,g:asx_gameplay::kept_bonus::KeptBonus)->Result<()>{
  if !self.supports_bonus_tooltips(){return Err("Compact glyph hover unavailable".into())}
  self.draw_annotation(base,parent,font,ui,&g.name(),g.icon(),Some(g))
 }
 pub unsafe fn bonus_header_draw(self,base:usize,parent:u64,font:u64,ui:BonusUi,label:&str,name:&str)->Result<()>{
  self.draw_annotation(base,parent,font,ui,label,name,None)
 }
 unsafe fn draw_annotation(self,base:usize,parent:u64,font:u64,ui:BonusUi,label:&str,name:&str,kept:Option<asx_gameplay::kept_bonus::KeptBonus>)->Result<()>{
  if !self.supports_extensions() {return Err("Card annotation UI unavailable".into())}
  type Icon=unsafe extern "system" fn(*mut u8,u64,*const u8,usize)->u64;
  type Card=unsafe extern "system" fn(*mut usize,usize,*const u8,*const f32,f32,*const u8,usize,u8,u8,*const u8,*const u8)->usize;
  let icon:Icon=std::mem::transmute(base+self.contact117(v::GLYPH_ICON_RVA) as usize);
  let card:Card=std::mem::transmute(base+self.rva(0x100a20) as usize);
  let mut image=[0u8;24];
  icon(image.as_mut_ptr(),ui.assets,name.as_ptr(),name.len());
  let mut action=[0u8;80];action[1]=1;
  card(parent as *mut usize,font as usize,image.as_ptr(),[0f32,0.,64.,64.].as_ptr(),if kept.is_some(){16.}else{24.},label.as_ptr(),label.len(),0,0,ui.color.as_ptr(),action.as_ptr());
  Ok(())
 }
}

impl Adapter{
 pub fn append_reward_note(self,m:&dyn MutableMemory,header:u64,note:&str)->Result<()>{
  let capacity=q(m,header)?;let ptr=q(m,header+8)?;let len=q(m,header+16)?;
  if len>8192||len>capacity||capacity>1048576||(len>0&&ptr<0x10000){return Err("Invalid glyph text extent".into())}
  let mut text=if len==0 {vec![]}else{m.read(ptr,len as usize)?};
  if text.len()!=len as usize || note.len()>512{return Err("Invalid glyph text".into())}
  text.extend_from_slice(b"\n\n");text.extend_from_slice(note.as_bytes());self.replace_owned_text(m,header,&text)
 }
 pub fn reward_card_note(self,m:&dyn MutableMemory,output:u64,note:&str)->Result<()>{
  // Native JSON serializer: title, hover text, icon, name, Vec<String> lines.
  // Extend an existing owned line; never treat the Vec header as a String.
  let capacity=q(m,output+0x60)?;let ptr=q(m,output+0x68)?;let len=q(m,output+0x70)?;
  if !(1..=16).contains(&len)||len>capacity||capacity>128||ptr<0x10000{return Err("Invalid glyph card lines".into())}
  self.append_reward_note(m,ptr+(len-1)*24,note)?;
  self.append_reward_note(m,output+0x18,note)
 }
}

impl Adapter{pub fn reward_entries(self)->[Entry;2]{[
 Entry{rva:self.contact117(v::GLYPH_CARD_RVA),bytes:self.extension_bytes117(0x5c007f0,v::GLYPH_CARD_BYTES)},
 Entry{rva:self.contact117(v::GLYPH_ACCEPT_RVA),bytes:self.extension_bytes117(0x5c05be0,v::GLYPH_ACCEPT_BYTES)}]}}
