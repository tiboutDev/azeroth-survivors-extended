//! Exact-build glyph IO; gameplay policy never sees native row layouts.
use super::*;
use crate::official::{Entry,MutableMemory,SupportCalls};
use crate::official117 as v;
use asx_gameplay::glyphs::{Glyph,Modifiers};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct GlyphOwner {pub game:u64,pub slot:u32,pub guid:u64,pub spell:u32}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct GlyphChoice {pub owner:GlyphOwner,pub index:u64,pub rank:u32,pub rarity:u64,pub upgrades:Vec<u8>}
fn q(m:&dyn Mem,a:u64)->Result<u64>{Ok(u64::from_le_bytes(m.read(a,8)?.try_into().map_err(|_|"Truncated glyph pointer")?))}
fn u(m:&dyn Mem,a:u64)->Result<u32>{Ok(u32::from_le_bytes(m.read(a,4)?.try_into().map_err(|_|"Truncated glyph integer")?))}
impl Adapter {
 pub fn glyph_entries(self)->Vec<Entry>{if !self.supports_extensions() {return vec![]}vec![Entry{rva:self.contact117(v::GLYPH_CARD_RVA),bytes:self.extension_bytes117(0x5c007f0,v::GLYPH_CARD_BYTES)},Entry{rva:self.contact117(v::GLYPH_ACCEPT_RVA),bytes:self.extension_bytes117(0x5c05be0,v::GLYPH_ACCEPT_BYTES)},Entry{rva:self.contact117(v::GLYPH_MODIFIERS_RVA),bytes:self.extension_bytes117(0x5ab3360,v::GLYPH_MODIFIERS_BYTES)},Entry{rva:self.contact117(v::GLYPH_ON_HIT_RVA),bytes:self.extension_bytes117(0x5bbf7b0,v::GLYPH_ON_HIT_BYTES)},Entry{rva:self.contact117(v::GLYPH_SHEEP_GATE_RVA),bytes:self.extension_bytes117(0x5bf0520,v::GLYPH_SHEEP_GATE_BYTES)}]}
 fn glyph_spell(self,m:&dyn Mem,base:u64,definition:u64)->Result<Option<u32>>{
  let Some((_,id))=v::GLYPH_DEFINITIONS.iter().find(|(rva,_)|base.checked_add(self.contact117(*rva))==Some(definition)) else{return Ok(None)};
  Ok((u(m,definition+0x40)?==*id).then_some(*id))
 }
 pub fn glyph_owner(self,m:&dyn Mem,base:u64,perks:u64,definition:u64)->Result<Option<GlyphOwner>>{
  if !self.supports_extensions() {return Ok(None)}
  let Some(spell)=self.glyph_spell(m,base,definition)? else{return Ok(None)};
  let Some(game)=perks.checked_sub(v::GLYPH_PERKS).filter(|g|*g>=0x10000) else{return Ok(None)};
  let Some((slot,guid))=self.run_source(m,game)? else{return Ok(None)};
  Ok(Some(GlyphOwner{game,slot,guid,spell}))
 }
 fn glyph_ability(self,m:&dyn Mem,game:u64,index:u64)->Result<u64>{
  let count=q(m,game+v::GLYPH_ABILITIES[1])?;
  if count>16 || index>=count {return Err("Glyph ability extent".into())}
  let ptr=q(m,game+v::GLYPH_ABILITIES[0])?;
  if ptr<0x10000{return Err("Glyph ability pointer".into())}
  ptr.checked_add(index*v::GLYPH_ABILITY_STRIDE).ok_or("Glyph ability overflow".into())
 }
 pub fn glyph_choice(self,m:&dyn Mem,base:u64,game:u64,payload:u64,rarity:u64)->Result<Option<GlyphChoice>>{
  if !self.supports_extensions() || rarity>5 {return Ok(None)}
  let capacity=q(m,payload)?;
  // Rust enum's negative niches describe spells/items/currency, not upgrades.
  if (capacity as i64)<0 {return Ok(None)}
  let len=q(m,payload+16)?;if !(1..=16).contains(&len)||len>capacity||capacity>256{return Ok(None)}
  let index=q(m,payload+24)?;let ability=self.glyph_ability(m,game,index)?;
  let definition=q(m,ability)?;
  let Some(owner)=self.glyph_owner(m,base,game+v::GLYPH_PERKS,definition)?else{return Ok(None)};
  let rank=u(m,ability+8)?;if !(1..=254).contains(&rank)||rank+len as u32>255{return Ok(None)}
  let ptr=q(m,payload+8)?;if ptr<0x10000{return Ok(None)}
  let upgrades=m.read(ptr,len as usize)?;if upgrades.len()!=len as usize||upgrades.iter().any(|k|*k>4){return Ok(None)}
  Ok(Some(GlyphChoice{owner,index,rank,rarity,upgrades}))
 }
 pub fn glyph_card(self,m:&dyn Mem,base:u64,game:u64,output:u64)->Result<Option<GlyphChoice>>{
  self.glyph_choice(m,base,game,output+v::GLYPH_CHOICE_PAYLOAD,q(m,output+v::GLYPH_CHOICE_RARITY)?)
 }
 pub fn glyph_selected(self,m:&dyn Mem,base:u64,game:u64,index:u64)->Result<Option<GlyphChoice>>{
  if !self.supports_extensions() || m.read(game+self.field117(v::GLYPH_CHOOSING),1)?!=[2] || m.read(game+self.field117(v::GLYPH_CHOICE_KIND),1)?==[255]{return Ok(None)}
  let count=q(m,game+v::GLYPH_CHOICES[1])?;if count>16||index>=count{return Ok(None)}
  let ptr=q(m,game+v::GLYPH_CHOICES[0])?;if ptr<0x10000{return Ok(None)}
  self.glyph_card(m,base,game,ptr+index*v::GLYPH_CHOICE_STRIDE)
 }
 pub fn glyph_rank(self,m:&dyn Mem,base:u64,c:&GlyphChoice)->Result<Option<u32>>{
  let a=self.glyph_ability(m,c.owner.game,c.index)?;
  if self.glyph_owner(m,base,c.owner.game+v::GLYPH_PERKS,q(m,a)?)?!=Some(c.owner){return Ok(None)}
  Ok(Some(u(m,a+8)?))
 }
 pub fn refresh_glyph(self,m:&dyn SupportCalls,base:u64,c:&GlyphChoice)->Result<()>{
  if self.glyph_rank(m,base,c)?.is_none(){return Err("Glyph owner changed".into())}
  let ability=self.glyph_ability(m,c.owner.game,c.index)?;
  m.resolve_glyph_modifiers(ability+v::GLYPH_DERIVED,c.owner.game+v::GLYPH_PERKS,q(m,ability)?)
 }
 pub fn apply_glyph(self,m:&dyn MutableMemory,output:u64,g:Glyph)->Result<()>{
  let mut bytes=m.read(output,28)?;if bytes.len()!=28{return Err("Truncated glyph modifiers".into())}
  let f=|at|f32::from_le_bytes(bytes[at..at+4].try_into().unwrap());
  let old=Modifiers{damage:f(0),cooldown:f(4),targets:u32::from_le_bytes(bytes[8..12].try_into().unwrap()),radius:f(12),duration:f(16),health_damage:f(20),effect:f(24),healing:1.,proc_rate:1.};
  if [old.damage,old.cooldown,old.radius,old.duration,old.health_damage,old.effect].iter().any(|f|!f.is_finite()||*f<0.)||old.targets>1000{return Err("Invalid glyph modifiers".into())}
  let mut new=g.apply(old);
  // Slow duration consumes the native duration word; root/stun duration uses effect.
  if matches!(g.spell(),116|120|11113|8056){new.duration*=g.apply(Modifiers::default()).effect;new.effect=old.effect;}
  // Mark vulnerability consumes damage too; scale its explosion only at damage delivery.
  // Seal Fate divides its interval by this word AFTER the native 2-second floor.
  if matches!(g.spell(),14186|118|6353){new.effect*=new.proc_rate;}
  if matches!(g.spell(),1130|118|770){new.damage=old.damage;new.health_damage=old.health_damage;}
  // Holy Light natively derives healing from sqrt(learned damage * derived damage).
  // Map semantic healing here; descriptors and tooltip calculations then agree.
  if matches!(g.spell(),635|139|331|774|8936) {
   // Rejuvenation stores a total heal budget distributed over the final duration.
   // Preserve the promised per-tick factor by scaling that budget by duration too.
   let healing=new.healing*if g.spell()==774{g.apply(Modifiers::default()).duration}else{1.};
   new.damage*=healing*healing;
  }
  for (at,value) in [(0,new.damage),(4,new.cooldown),(12,new.radius),(16,new.duration),(20,new.health_damage),(24,new.effect)]{bytes[at..at+4].copy_from_slice(&value.to_le_bytes());}
  bytes[8..12].copy_from_slice(&new.targets.to_le_bytes());m.write(output,&bytes)
 }
 /// Holy Shock's coefficient is raw in native damage/heal paths; scale it at
 /// the existing heal/delivery boundary. Holy Light is already mapped above.
 pub fn glyph_extra_healing(self,g:Glyph)->f32{if self.supports_extensions()&&matches!(g.spell(),25912|15237){g.healing_factor()}else{1.}}
 /// Native tooltip borrows these arrays only for its synchronous formatting call.
 /// No global definition or learned-ability pointer is ever changed.
 pub unsafe fn glyph_special_presentation(self,m:&dyn Mem,original:usize,args:[u64;5],factor:f32)->Result<u64>{
  if !self.supports_extensions()||!factor.is_finite()||!(0.1..=2.).contains(&factor){return Err("Invalid glyph healing presentation".into())}
  let [out,ability,stats,kind,perks]=args;
  #[repr(C,align(8))]struct AbilityCopy([u8;88]);
  #[repr(C,align(8))]struct DefinitionCopy([u8;120]);
  if kind!=0{return Err("Not a learned spell presentation".into())}
  let mut row=AbilityCopy(m.read(ability,88)?.try_into().map_err(|_|"Truncated glyph ability")?);
  let definition=u64::from_le_bytes(row.0[..8].try_into().unwrap());
  let mut def=DefinitionCopy(m.read(definition,120)?.try_into().map_err(|_|"Truncated glyph definition")?);
  let spell=u32::from_le_bytes(def.0[0x40..0x44].try_into().unwrap());
  if matches!(spell,1130|118){
   for (bytes,at) in [(&mut def.0[..],0x44),(&mut row.0[..],0x4c)]{let value=f32::from_le_bytes(bytes[at..at+4].try_into().unwrap())*factor;if !value.is_finite()||value<0.{return Err("Invalid mark presentation damage".into())}bytes[at..at+4].copy_from_slice(&value.to_le_bytes());}
  }else if matches!(spell,5782|8122){
   let duration=f32::from_le_bytes(def.0[0x6c..0x70].try_into().unwrap())*factor;if !duration.is_finite()||!(0.0..=120.).contains(&duration){return Err("Invalid glyph fear presentation duration".into())}def.0[0x6c..0x70].copy_from_slice(&duration.to_le_bytes());
  }else if matches!(spell,25912|15237){
  let coefficient=f32::from_le_bytes(def.0[0x6c..0x70].try_into().unwrap())*factor;
  if !coefficient.is_finite()||!(0.0..=1.).contains(&coefficient){return Err("Invalid glyph healing coefficient".into())}
  def.0[0x6c..0x70].copy_from_slice(&coefficient.to_le_bytes());
  }else{return Err("Unsupported glyph special presentation".into())}
  row.0[..8].copy_from_slice(&(def.0.as_ptr() as u64).to_le_bytes());
  let call:unsafe extern "system" fn(u64,u64,u64,u8,u64)->u64=std::mem::transmute(original);
  Ok(call(out,row.0.as_ptr() as u64,stats,kind as u8,perks))
 }

}

impl Adapter {
 fn glyph_fixture_scored(self,m:&dyn Mem,base:u64,game:u64,seen:&[String],required:&[u32],class:u32)->Result<Option<(u64,u32)>>{
  let count=q(m,game+v::GLYPH_CHOICES[1])?;if count>16||m.read(game+self.field117(v::GLYPH_CHOOSING),1)?!=[2]{return Ok(None)}
  let ptr=q(m,game+v::GLYPH_CHOICES[0])?;let mut best=None;
  for index in 0..count {let row=ptr+index*v::GLYPH_CHOICE_STRIDE;
   let len=q(m,row+0x58)?;if len>64 {continue}let name=m.read(q(m,row+0x50)?,len as usize)?;
   let allowed=if class==2{[b"Exorcism".as_slice(),b"Holy Shock",b"Holy Light",b"Holy Shield",b"Divine Shield",b"Consecration"]}else if class==3{[b"Auto Shot".as_slice(),b"Serpent Sting",b"Volley",b"Concussive Shot",b"Frost Trap",b"Hunter's Mark"]}else if class==4{[b"Sinister Strike".as_slice(),b"Rupture",b"Ghostly Strike",b"Adrenaline Rush",b"Slice and Dice",b"Seal Fate"]}else if class==5{[b"Smite".as_slice(),b"Holy Nova",b"Power Word: Shield",b"Renew",b"Mind Flay",b"Lightwell"]}else if class==7{[b"Lightning Bolt".as_slice(),b"Lightning Shield",b"Magma Totem",b"Searing Totem",b"Healing Wave",b"Call of Flame"]}else if class==8{[b"Fireball".as_slice(),b"Arcane Missiles",b"Frost Nova",b"Blizzard",b"Ice Barrier",b"Polymorph"]}else if class==9{[b"Shadow Bolt".as_slice(),b"Fear",b"Death Coil",b"Drain Life",b"Soulfire",b"Demonic Empowerment"]}else if class==11{[b"Wrath".as_slice(),b"Rejuvenation",b"Regrowth",b"Thorns",b"Entangling Roots",b"Faerie Fire"]}else{[b"Cleave".as_slice(),b"Whirlwind",b"Rend",b"Battle Shout",b"Demoralizing Shout",b"Shield Slam"]};
   if !allowed.contains(&name.as_slice()){continue}
   let c=self.glyph_card(m,base,game,row)?;
   let removal=if seen.len()>=6&&!seen.iter().any(|s|s=="Unglyphed")&&c.as_ref().is_some_and(|c|!matches!(c.owner.spell,770|1130|14186|724|16038|118|11426|6353|18708)){
    let n=q(m,row+0x70)?;let lines=q(m,row+0x68)?;
    if (1..=16).contains(&n){let header=lines+(n-1)*24;let len=q(m,header+16)?;len<=8192&&String::from_utf8_lossy(&m.read(q(m,header+8)?,len as usize)?).contains("Remove glyph:")}else{false}
   }else{false};
   let wanted=if class==11{[(774,b"Rejuvenation".as_slice()),(467,b"Thorns"),(770,b"Faerie Fire")]}else if class==9{[(18708,b"Demonic Empowerment".as_slice()),(6353,b"Soulfire"),(5782,b"Fear")]}else if class==8{[(11426,b"Ice Barrier".as_slice()),(118,b"Polymorph"),(7269,b"Arcane Missiles")]}else if class==7{[(324,b"Lightning Shield".as_slice()),(331,b"Healing Wave"),(16038,b"Call of Flame")]}else if class==5{[(17,b"Power Word: Shield".as_slice()),(139,b"Renew"),(15237,b"Holy Nova")]}else{[(14186,b"Seal Fate".as_slice()),(13750,b"Adrenaline Rush"),(5171,b"Slice and Dice")]};
   let wanted_new=matches!(class,4|5|7|8|9|11)&&wanted.iter().any(|(id,n)|required.contains(id)&&*n==name.as_slice());
   let score=if removal{7}else{match c {None=>if wanted_new{8}else{4},Some(c)if matches!(class,4|5|7|8|9|11)&&required.contains(&c.owner.spell)=>if asx_gameplay::glyphs::offer(c.owner.spell,c.rank,c.rarity,&c.upgrades).is_some(){6}else{5},Some(c)=>match asx_gameplay::glyphs::offer(c.owner.spell,c.rank,c.rarity,&c.upgrades){Some(_)if required.contains(&c.owner.spell)=>6,Some(g)if !seen.iter().any(|s|s==g.name())=>3,Some(_)=>2,None=>1}}};
   if best.is_none_or(|(_,s)|score>s){best=Some((index,score));}
  }Ok(best)
 }
 pub fn glyph_fixture_ability(self,m:&dyn Mem,base:u64,game:u64,ability:u64)->Result<Option<(GlyphOwner,u32,Vec<f32>)>>{
  let Some(owner)=self.glyph_owner(m,base,game+v::GLYPH_PERKS,q(m,ability)?)?else{return Ok(None)};
  let bytes=m.read(ability+v::GLYPH_DERIVED,28)?;
  let values=(0..7).map(|n|if n==2 {u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as f32}else{f32::from_le_bytes(bytes[n*4..n*4+4].try_into().unwrap())}).collect();
  Ok(Some((owner,u(m,ability+8)?,values)))
 }
 /// Diagnostic-only caller: resolves into owned scratch bytes, never into a learned spell.
 pub fn glyph_fixture_resolve(self,m:&dyn SupportCalls,base:u64,owner:GlyphOwner)->Result<Vec<f32>>{
  if !self.supports_extensions()||self.run_source(m,owner.game)?!=Some((owner.slot,owner.guid)){return Err("Glyph probe owner changed".into())}
  let rva=v::GLYPH_DEFINITIONS.iter().find(|(_,id)|*id==owner.spell).map(|(rva,_)|self.contact117(*rva)).ok_or("Unknown glyph probe spell")?;
  if self.glyph_spell(m,base,base+rva)?!=Some(owner.spell){return Err("Glyph probe definition changed".into())}
  let mut bytes=[0u8;28];m.resolve_glyph_modifiers(bytes.as_mut_ptr() as u64,owner.game+v::GLYPH_PERKS,base+rva)?;
  Ok((0..7).map(|n|if n==2{u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as f32}else{f32::from_le_bytes(bytes[n*4..n*4+4].try_into().unwrap())}).collect())
 }
 /// Controlled test inputs only: use the owned learned ability / current native mark.
 pub fn glyph_fixture_cast_choice(self,m:&dyn SupportCalls,base:u64,c:&GlyphChoice)->Result<bool>{
  if self.glyph_rank(m,base,c)?.is_none()||self.run_source(m,c.owner.game)?!=Some((c.owner.slot,c.owner.guid)){return Err("Glyph cast fixture owner changed".into())}
  m.cast_actor(c.owner.game,self.glyph_ability(m,c.owner.game,c.index)?)
 }
 pub fn glyph_fixture_mark_target(self,m:&dyn Mem,game:u64,slot:u32)->Result<Option<u64>>{
  if !self.supports_extensions(){return Ok(None)}
  let row=self.support_row(m,game,slot)?;let target=q(m,row+self.row_field(row==game,0x8f0))?;
  if target==0{return Ok(None)}
  Ok(self.combat_hp(m,game,target)?.filter(|hp|*hp>0.).map(|_|target))
 }
 pub fn glyph_fixture_refresh_ability(self,m:&dyn SupportCalls,base:u64,game:u64,ability:u64)->Result<()>{
  let definition=q(m,ability)?;if self.glyph_owner(m,base,game+v::GLYPH_PERKS,definition)?.is_none(){return Err("Unknown glyph fixture ability".into())}
  m.resolve_glyph_modifiers(ability+v::GLYPH_DERIVED,game+v::GLYPH_PERKS,definition)
 }
 pub fn glyph_fixture_current(self,m:&dyn Mem,base:u64,c:&GlyphChoice)->Result<Vec<f32>>{
  if self.glyph_rank(m,base,c)?.is_none(){return Err("Glyph fixture owner changed".into())}
  self.glyph_fixture_modifiers(m,self.glyph_ability(m,c.owner.game,c.index)?+v::GLYPH_DERIVED)
 }
 pub fn glyph_fixture_modifiers(self,m:&dyn Mem,output:u64)->Result<Vec<f32>>{
  let bytes=m.read(output,28)?;if bytes.len()!=28{return Err("Truncated glyph probe".into())}
  Ok((0..7).map(|n|if n==2 {u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as f32}else{f32::from_le_bytes(bytes[n*4..n*4+4].try_into().unwrap())}).collect())
 }
}

impl Adapter {
 pub fn glyph_fixture_card_probe(self,m:&dyn Mem,base:u64,game:u64,output:u64)->Result<String>{
  let cap=q(m,output+0x78)?;if (cap as i64)<0{return Ok(String::new())}
  let owner=self.run_source(m,game);let index=q(m,output+0x90)?;
  let ability=self.glyph_ability(m,game,index);let def=ability.as_ref().ok().and_then(|a|q(m,*a).ok());
  let spell=def.and_then(|d|self.glyph_spell(m,base,d).ok().flatten());
  Ok(format!("owner={owner:?} cap={cap} payload={:?} index={index} ability={ability:?} def={def:?} spell={spell:?} decode={:?}",m.read(output+0x78,40),self.glyph_card(m,base,game,output)))
 }
}


impl Adapter {
 pub fn glyph_fixture_pick(self,m:&dyn Mem,base:u64,game:u64,seen:&[String],required:&[u32],class:u32)->Result<Option<(u64,bool)>>{
  let Some((index,score))=self.glyph_fixture_scored(m,base,game,seen,required,class)?else{return Ok(Some((0,true)))};
  let count=q(m,game+v::GLYPH_ABILITIES[1])?;if count>16{return Err("Glyph fixture ability extent".into())}
  let mut learned=std::collections::BTreeSet::new();
  for i in 0..count{if let Some(spell)=self.glyph_spell(m,base,q(m,self.glyph_ability(m,game,i)?)?)?{learned.insert(spell);}}
  let reroll=if learned.len()<3 {score!=4&&score!=8}else {score!=6&&score!=3&&score!=7&&score!=8&&!(matches!(class,4|5|7|8|9|11)&&score==5)&&(!required.is_empty()||seen.len()<7)};
  Ok(Some((index,reroll)))
 }
 pub unsafe fn glyph_fixture_reroll(self,m:&dyn MutableMemory,base:usize,game:u64)->Result<()>{
  if !self.supports_extensions()||m.read(base as u64+self.contact117(v::GLYPH_REROLL_RVA),self.extension_bytes117(0x5c09350,v::GLYPH_REROLL_BYTES).len())?!=self.extension_bytes117(0x5c09350,v::GLYPH_REROLL_BYTES){return Err("Glyph fixture reroll binding".into())}
  if m.read(game+self.field117(v::GLYPH_CHOOSING),1)?!=[2]{return Err("Glyph fixture reroll phase".into())}
  if u(m,game+self.field117(v::GLYPH_REROLLS))?==0 {m.write(game+self.field117(v::GLYPH_REROLLS),&1u32.to_le_bytes())?;}
  let call:unsafe extern "system" fn(u64)=std::mem::transmute(base+self.contact117(v::GLYPH_REROLL_RVA) as usize);call(game);Ok(())
 }
}

impl Adapter {
 pub fn glyph_sheep_count(self,m:&dyn Mem,game:u64)->Result<u64>{if !self.supports_extensions(){return Err("Unsupported sheep observation".into())}let n=q(m,game+v::GLYPH_SHEEP_COUNT)?;if n>16{return Err("Invalid native sheep count".into())}Ok(n)}
 pub fn glyph_sheep_flags(self,count:u64,limit:u32,flags:u64)->u64{if self.supports_extensions()&&limit==5&&(3..5).contains(&count){flags|0x40}else{flags}}
 pub fn glyph_polymorph_description(self,text:&str,factor:f32,limit:u32)->Result<String>{
  if !self.supports_extensions()||text.len()>8192||!(0.1..=2.).contains(&factor)||!matches!(limit,3|5){return Err("Invalid Polymorph presentation".into())}
  fn number(span:&str,value:f32)->Result<String>{
   let (prefix,body)=if span.starts_with("|c")&&span.len()>=10{span.split_at(10)}else{("",span)};
   let (body,suffix)=body.strip_suffix("|r").map(|b|(b,"|r")).unwrap_or((body,""));
   if body.parse::<f32>().is_err(){return Err("Unrecognized native Polymorph number".into())}
   let n=format!("{value:.2}");Ok(format!("{prefix}{}{suffix}",n.trim_end_matches('0').trim_end_matches('.')))
  }
  let mut result=text.to_string();let end=result.find(" at once.").ok_or("Missing native sheep limit")?;let start=result[..end].rfind(" and ").ok_or("Missing native sheep conjunction")?+5;
  let value=number(&result[start..end],limit as f32)?;result.replace_range(start..end,&value);
  let start=result.find("burst of ").ok_or("Missing native sheep burst")?+9;let end=start+result[start..].find("% of its maximum health").ok_or("Missing native sheep percentage")?;
  let span=&result[start..end];let plain=if span.starts_with("|c")&&span.len()>=10{&span[10..]}else{span};let raw=plain.trim_end_matches("|r").parse::<f32>().map_err(|_|"Invalid native sheep percentage")?;
  let value=number(span,raw*factor)?;result.replace_range(start..end,&value);Ok(result)
 }
 pub fn glyph_polymorph_text(self,m:&dyn MutableMemory,header:u64,factor:f32,limit:u32)->Result<()>{let cap=q(m,header)?;let ptr=q(m,header+8)?;let len=q(m,header+16)?;if len>8192||len>cap||ptr<0x10000{return Err("Invalid Polymorph text extent".into())}let bytes=m.read(ptr,len as usize)?;let text=std::str::from_utf8(&bytes).map_err(|_|"Invalid Polymorph text")?;let value=self.glyph_polymorph_description(text,factor,limit)?;self.replace_owned_text(m,header,value.as_bytes())}
}

impl Adapter {
 /// Synchronous diagnostic use of the existing reviewed five-argument formatter.
 pub fn glyph_fixture_presentation(self,m:&dyn MutableMemory,base:u64,owner:GlyphOwner)->Result<Vec<(u32,String)>>{
  if !self.supports_extensions(){return Err("Unsupported glyph presentation probe".into())}
  let count=q(m,owner.game+v::GLYPH_ABILITIES[1])?;if count>16{return Err("Invalid glyph ability extent".into())}
  let mut ability=None;for index in 0..count{let a=self.glyph_ability(m,owner.game,index)?;if self.glyph_spell(m,base,q(m,a)?)?==Some(owner.spell){ability=Some(a);break}}
  let ability=ability.ok_or("Missing learned glyph ability")?;
  #[repr(C,align(8))]struct Output([u8;0xb0]);let mut out=Output([0;0xb0]);
  let call:unsafe extern "system" fn(u64,u64,u64,u8,u64)->u64=unsafe{std::mem::transmute((base+self.spell_presentation_entry().rva)as usize)};
  // The fifth formatter argument is an annotation container, not the modifier/perk block.
  // The probe owns a valid empty annotation; normal callers keep their native value.
  let annotation=[0u64,8,0];
  unsafe{call(out.0.as_mut_ptr()as u64,ability,owner.game+self.game_field(0x490),0,annotation.as_ptr()as u64)};
  let result=self.spell_presentation_strings(m,out.0.as_ptr()as u64);
  for at in [0,24,48,72,96,120,144]{let cap=u64::from_le_bytes(out.0[at..at+8].try_into().unwrap());let ptr=u64::from_le_bytes(out.0[at+8..at+16].try_into().unwrap());if cap>0&&cap<=1048576&&ptr>=0x10000{m.free_native(ptr,cap as usize)?;}}
  result
 }
}

impl Adapter {
 /// Native Fear copies its raw duration instead of consuming the derived word.
 /// Borrow only tag-4 payloads; immunity and all other native effects stay native.
 pub fn glyph_fear_effect(self,m:&dyn Mem,effect:u64,factor:f32)->Result<Option<([u8;12],f32,f32)>>{
  if !self.supports_extensions(){return Ok(None)}if !factor.is_finite()||!(0.1..=2.).contains(&factor){return Err("Invalid fear glyph factor".into())}
  let mut b:[u8;12]=m.read(effect,12)?.try_into().map_err(|_|"Truncated fear effect")?;if u32::from_le_bytes(b[..4].try_into().unwrap())!=4{return Ok(None)}let before=f32::from_le_bytes(b[4..8].try_into().unwrap());let after=before*factor;if !before.is_finite()||!after.is_finite()||before<0.||after>120.{return Err("Invalid native fear duration".into())}b[4..8].copy_from_slice(&after.to_le_bytes());Ok(Some((b,before,after)))
 }
}

impl Adapter {
 pub fn glyph_fixture_leech_fraction(self,m:&dyn Mem,effect:u64)->Result<Option<f32>>{if !self.supports_extensions(){return Ok(None)}let b=m.read(effect,12)?;if b.len()!=12{return Err("Truncated leech effect".into())}if u32::from_le_bytes(b[..4].try_into().unwrap())!=5{return Ok(None)}let f=f32::from_le_bytes(b[4..8].try_into().unwrap());if !f.is_finite()||!(0.0..=1.).contains(&f){return Err("Invalid native leech fraction".into())}Ok(Some(f))}
}

impl Adapter {
 /// Faerie Fire derives its capped percentage with a square root. Its glyph
 /// scales the completed arc, so adjust both displayed terms after formatting.
 pub fn glyph_faerie_description(self,text:&str,factor:f32)->Result<String>{
  if !self.supports_extensions()||text.len()>8192||!factor.is_finite()||!(0.1..=2.).contains(&factor){return Err("Invalid Faerie Fire presentation".into())}
  let mut result=text.to_string();
  for (prefix,suffix) in [(" arc ","% of their damage"),("up to ",", as Nature damage")]{
   let start=result.find(prefix).ok_or("Missing native Faerie Fire term")?+prefix.len();
   let end=start+result[start..].find(suffix).ok_or("Missing native Faerie Fire delimiter")?;
   let span=&result[start..end];let (color,body)=if span.starts_with("|c")&&span.len()>=10{span.split_at(10)}else{("",span)};
   let (body,reset)=body.strip_suffix("|r").map(|b|(b,"|r")).unwrap_or((body,""));
   let before=body.parse::<f32>().map_err(|_|"Invalid native Faerie Fire number")?;
   let value=before*factor;if !value.is_finite()||value<0.{return Err("Invalid Faerie Fire value".into())}
   let value=format!("{value:.2}");let value=format!("{color}{}{reset}",value.trim_end_matches('0').trim_end_matches('.'));result.replace_range(start..end,&value);
  }Ok(result)
 }
 pub fn glyph_faerie_text(self,m:&dyn MutableMemory,header:u64,factor:f32)->Result<()>{
  let cap=q(m,header)?;let ptr=q(m,header+8)?;let len=q(m,header+16)?;
  if len>8192||len>cap||ptr<0x10000{return Err("Invalid Faerie Fire text extent".into())}
  let bytes=m.read(ptr,len as usize)?;let text=std::str::from_utf8(&bytes).map_err(|_|"Invalid Faerie Fire text")?;
  let value=self.glyph_faerie_description(text,factor)?;self.replace_owned_text(m,header,value.as_bytes())
 }
}

impl Adapter{pub fn glyph_retention_pick(self,m:&dyn Mem,base:u64,game:u64)->Result<Option<u64>>{
 let count=q(m,game+v::GLYPH_CHOICES[1])?;let ptr=q(m,game+v::GLYPH_CHOICES[0])?;if count>16{return Err("Glyph retention extent".into())}let mut best=None;
 for index in 0..count{let row=ptr+index*v::GLYPH_CHOICE_STRIDE;let len=q(m,row+0x58)?;if len>64{continue}let name=m.read(q(m,row+0x50)?,len as usize)?;
  let c=self.glyph_selected(m,base,game,index)?;let score=if c.as_ref().is_some_and(|c|c.owner.spell==845){100}else if name==b"Cleave"{200}else{0};if score>0&&best.is_none_or(|(_,s)|score>s){best=Some((index,score));}}
 Ok(best.map(|(i,_)|i))
}}

impl Adapter{
 pub unsafe fn glyph_ui_draw(self,base:usize,parent:u64,font:u64,ui:BonusUi,g:Option<Glyph>)->Result<()>{
  self.bonus_header_draw(base,parent,font,ui,g.map(Glyph::name).unwrap_or("Unglyphed"),"INV_Scroll_02")
 }


}
