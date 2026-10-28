//! Shared native reward annotation widgets; versioned pointers stay here.
use super::*;
use crate::official::{Entry,MutableMemory};
use crate::official117 as v;
fn q(m:&dyn Mem,a:u64)->Result<u64>{Ok(u64::from_le_bytes(m.read(a,8)?.try_into().map_err(|_|"Truncated UI word")?))}
#[derive(Clone,Copy)]pub struct BonusSeparator {node:[u8;crate::official125::NODE_SIZE],color:[u8;20]}
#[derive(Clone,Copy)]pub struct BonusHeaderColumn {node:[u8;crate::official125::NODE_SIZE],meta:u64}
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
  for(offset,value)in[(v::GLYPH_UI_HEADER_WIDTH,208f32),(v::GLYPH_UI_HEADER_HEIGHT,40f32)]{
   m.write(bundle+v::GLYPH_UI_HEADER_NODE+offset,&[1u32.to_le_bytes(),value.to_le_bytes()].concat())?;
  }
  // This header alone is a row; keep the icon/title spacing native and centered.
  m.write(bundle+v::GLYPH_UI_HEADER_NODE+crate::official125::NODE_COLUMN_GAP,&[1u32.to_le_bytes(),6f32.to_le_bytes()].concat())?;
  // The separator supplies the separation; no extra top margin on ASX rows.
  m.write(bundle+v::GLYPH_UI_HEADER_NODE+v::GLYPH_UI_HEADER_MARGIN_TOP,&[1u32.to_le_bytes(),0f32.to_le_bytes()].concat())?;
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
  self.draw_annotation(base,parent,font,ui,&g.name(),g.icon())
 }
 pub unsafe fn bonus_header_draw(self,base:usize,parent:u64,font:u64,ui:BonusUi,label:&str,name:&str)->Result<()>{
  self.draw_annotation(base,parent,font,ui,label,name)
 }
 unsafe fn draw_annotation(self,base:usize,parent:u64,font:u64,ui:BonusUi,label:&str,name:&str)->Result<()>{
  if !self.supports_extensions() {return Err("Card annotation UI unavailable".into())}
  type Icon=unsafe extern "system" fn(*mut u8,u64,*const u8,usize)->u64;
  type Card=unsafe extern "system" fn(*mut usize,usize,*const u8,*const f32,f32,*const u8,usize,u8,u8,*const u8,*const u8)->usize;
  let icon:Icon=std::mem::transmute(base+self.contact117(v::GLYPH_ICON_RVA) as usize);
  let card:Card=std::mem::transmute(base+self.rva(0x100a20) as usize);
  let mut image=[0u8;24];
  icon(image.as_mut_ptr(),ui.assets,name.as_ptr(),name.len());
  let mut action=[0u8;80];action[1]=1;
  card(parent as *mut usize,font as usize,image.as_ptr(),[0f32,0.,64.,64.].as_ptr(),32.,label.as_ptr(),label.len(),0,0,ui.color.as_ptr(),action.as_ptr());
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

impl Adapter {
 pub fn bonus_separator_entries(self)->[Entry;2]{
  [crate::official125::BONUS_SEPARATOR_BUILD,crate::official125::BONUS_SEPARATOR_SPAWN].map(|(rva,bytes)|Entry{rva,bytes})
 }
 pub fn bonus_separator_capture(self,m:&dyn Mem,caller:u64,node:u64,color:u64)->Result<Option<BonusSeparator>>{
  if caller!=crate::official125::BONUS_SEPARATOR_CALLER{return Ok(None)}
  if node<0x10000||color<0x10000{return Err("Invalid native separator inputs".into())}
  let node:[u8;crate::official125::NODE_SIZE]=m.read(node,crate::official125::NODE_SIZE)?.try_into().map_err(|_|"Truncated separator Node")?;
  // The constructor moves four grid Vecs at offsets 0,24,48,72. This card uses
  // empty defaults; reject owned allocations rather than alias their ownership.
  if node[..96].chunks_exact(24).any(|v|v[..8]!=[0;8]||v[16..24]!=[0;8]){return Err("Owned separator grid data cannot be copied".into())}
  Ok(Some(BonusSeparator{node,color:m.read(color,20)?.try_into().map_err(|_|"Truncated separator color")?}))
 }
 pub unsafe fn bonus_separator_draw(self,m:&dyn Mem,base:usize,parent:u64,style:BonusSeparator)->Result<()>{
  type Build=unsafe extern "system" fn(*mut u8,*const u8,*const u8)->u64;
  type Spawn=unsafe extern "system" fn(*mut u8,u64,*mut u8)->u64;
  let build:Build=std::mem::transmute(base+crate::official125::BONUS_SEPARATOR_BUILD.0 as usize);
  let spawn:Spawn=std::mem::transmute(base+crate::official125::BONUS_SEPARATOR_SPAWN.0 as usize);
  // Native constructor owns its fresh gradient handles. Spawn consumes them once;
  // only the plain Node/color inputs are copied from this card's native separator.
  let mut bundle=[0u64;0x1a0/8];let mut output=[0u64;0x30/8];
  // The queued bundle is (ChildOf, Node, BackgroundGradient): the native
  // constructor returns the last two fields, not the leading parent entity.
  bundle[0]=q(m,parent)?;
  build(bundle.as_mut_ptr().add(1).cast(),style.node.as_ptr(),style.color.as_ptr());
  // ChildSpawner stores its parent entity first, then EntityCommands. The
  // native spawn monomorphization borrows EntityCommands, not ChildSpawner.
  spawn(output.as_mut_ptr().cast(),parent+8,bundle.as_mut_ptr().cast());
  Ok(())
 }
}

impl Adapter {
 pub fn bonus_column_entries(self)->[Entry;2]{[crate::official125::BONUS_TITLE_SPAWN,crate::official125::BONUS_COLUMN_SPAWN].map(|(rva,bytes)|Entry{rva,bytes})}
 /// Capture only the shared native header's empty-grid Node and bundle metadata.
 pub fn bonus_column_capture(self,m:&dyn Mem,caller:u64,bundle:u64,meta:u64)->Result<Option<BonusHeaderColumn>>{
  if caller!=crate::official125::BONUS_HEADER_CALLER{return Ok(None)}
  if meta<0x10000{return Err("Invalid bonus column metadata".into())}
  let mut node:[u8;crate::official125::NODE_SIZE]=m.read(bundle+8,crate::official125::NODE_SIZE)?.try_into().map_err(|_|"Truncated bonus header Node")?;
  if node[..96].chunks_exact(24).any(|v|v[..8]!=[0;8]||v[16..24]!=[0;8]){return Err("Owned bonus header grids cannot be copied".into())}
  // A vertical text column inside the native horizontal icon row. Auto dimensions
  // follow its two text children; the outer native header remains centered.
  for at in [v::GLYPH_UI_HEADER_WIDTH,v::GLYPH_UI_HEADER_HEIGHT]{node[at as usize..at as usize+8].fill(0);}
  for at in [0xb8usize,0xc0,0xc8,0xd0]{node[at..at+4].copy_from_slice(&1u32.to_le_bytes());node[at+4..at+8].fill(0);}
  node[v::GLYPH_UI_HEADER_DIRECTION as usize]=1;
  node[crate::official125::BONUS_NODE_ALIGN_ITEMS]=crate::official125::BONUS_ALIGN_START;
  let gap=crate::official125::BONUS_NODE_ROW_GAP;
  node[gap..gap+4].copy_from_slice(&1u32.to_le_bytes());node[gap+4..gap+8].fill(0);
  Ok(Some(BonusHeaderColumn{node,meta}))
 }
 /// Borrow the current title's EntityCommands; consume one fresh native Node.
 /// Return the native EntityCommands/ChildSpawner view for the status child.
 pub unsafe fn bonus_column_draw(self,m:&dyn MutableMemory,base:usize,commands:u64,title:u64,style:BonusHeaderColumn)->Result<[u64;6]>{
  // Validate all borrowed destinations before queueing native work.
  let parent=q(m,title)?;m.read(title+v::GLYPH_STATUS_FONT_SIZE,4)?;
  type Spawn=unsafe extern "system" fn(*mut u64,u64,*mut u64,u64)->u64;
  let spawn:Spawn=std::mem::transmute(base+crate::official125::BONUS_COLUMN_SPAWN.0 as usize);
  let mut bundle=[0u64;0x188/8];bundle[0]=parent;
  std::ptr::copy_nonoverlapping(style.node.as_ptr(),bundle.as_mut_ptr().add(1).cast(),style.node.len());
  let mut output=[0u64;6];spawn(output.as_mut_ptr(),commands,bundle.as_mut_ptr(),style.meta);
  m.write(title,&output[0].to_le_bytes())?;
  m.write(title+v::GLYPH_STATUS_FONT_SIZE,&13f32.to_le_bytes())?;
  Ok(output)
 }
}

impl Adapter{
 /// Reborrow the live card's command queue for a caption delivered as a separate
 /// native line. Only its column entity survives from header construction.
 pub fn bonus_column_spawner(self,m:&dyn Mem,live_parent:u64,column:u64)->Result<[u64;6]>{
  let bytes=m.read(live_parent,48)?;
  let mut view=std::array::from_fn(|i|u64::from_le_bytes(bytes[i*8..i*8+8].try_into().unwrap()));view[0]=column;Ok(view)
 }
}
