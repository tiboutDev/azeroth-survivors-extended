//! Opt-in test-hook witnesses. Native definitions/constructors and transport only.
use super::Adapter;
use crate::{Result,current as v,memory::Mem,official::MutableMemory};
fn word(m:&dyn Mem,p:u64)->Result<u64>{Ok(u64::from_le_bytes(m.read(p,8)?.try_into().map_err(|_|"Truncated fixture word")?))}
impl Adapter {
 pub unsafe fn fixture_world_ui_npcs(self,m:&dyn MutableMemory,base:u64,game:u64)->Result<Vec<(String,u64)>>{
  let party=self.world_party(m,game)?.ok_or("Missing fixture party")?;
  if party.actors.len()!=2||self.run_source(m,game)?.is_none_or(|(slot,_)|slot!=0){return Err("Invalid fixture host".into())}
  let origin=party.actors[0].position;
  for actor in &party.actors{
   let row=self.support_row(m,game,actor.slot)?;
   m.write(row+self.row_field(row==game,0x4f8),&(actor.maximum*0.85).to_le_bytes())?;
  }
  let(map,_)=self.blessing_map(m,game,base)?;
  if map>=12{return Err("Invalid fixture map".into())}
  let table=base+v::GOBLIN_MAP_TABLE+map*240;
  let ptr=word(m,table+0x78)?;let count=word(m,table+0x80)?;
  if !(1..=64).contains(&count)||ptr<base||ptr.checked_add(count*80).is_none_or(|end|end>base+0x8000000){return Err("Invalid fixture horde table".into())}
  let definitions=m.read(ptr,count as usize*80)?;
  let guardian=definitions.chunks_exact(80).position(|r|r[76]==1).ok_or("No guardian definition")?;
  let normal=definitions.chunks_exact(80).position(|r|r[76]==0).ok_or("No ordinary definition")?;
  let mut witnesses=Vec::new();
  for(label,index,dx)in [("elite",normal,-7f32),("guardian",guardian,7f32)]{
   let previous=self.support_enemies(m,game)?.into_iter().map(|e|e.guid).collect::<Vec<_>>();
   let mut point=[0u8;16];point[..4].copy_from_slice(&1u32.to_le_bytes());
   for(i,value)in [origin[0]+dx,origin[1]+8.,origin[2]].into_iter().enumerate(){point[4+i*4..8+i*4].copy_from_slice(&value.to_le_bytes())}
   let spawn:unsafe extern "system" fn(u64,u64,*const u8)=std::mem::transmute((base+v::GOBLIN_SPAWN.0)as usize);
   spawn(game,index as u64,point.as_ptr());
   let new=self.support_enemies(m,game)?.into_iter().find(|e|!previous.contains(&e.guid)).ok_or("Native fixture NPC missing")?;
   // Rebind after the constructor callback; never retain native array addresses across it.
   let header=game+self.game_field(0x570);let cap=word(m,header)?;let ptr=word(m,header+8)?;let n=word(m,header+16)?;
   if n>cap||cap>65536||ptr<0x10000{return Err("Invalid fixture NPC array".into())}
   for i in 0..n{
    let row=ptr+i*v::ENEMY_STRIDE as u64;
    if word(m,row+v::ENEMY_GUID as u64)?==new.guid{
     // Only disposable witnesses: survive the observation window.
     m.write(row+v::ENEMY_HP as u64,&100000f32.to_le_bytes())?;
     if label=="elite"{m.write(row+0x1f4,&[1])?;}
     break;
    }
   }
   witnesses.push((label.into(),new.guid));
  }
  let before=self.blessing_fixture_targets(m,game)?;
  let definition=self.blessing_fixture_definition(m,game,base,0)?;
  self.fixture_spawn_encounter(base as usize,game,definition);
  for guid in self.blessing_fixture_targets(m,game)?{if !before.contains(&guid){witnesses.push(("boss".into(),guid));}}
  Ok(witnesses)
 }
}

impl Adapter {
 pub fn fixture_world_ui_wound(self,m:&dyn MutableMemory,game:u64,slot:u32)->Result<()> {
  let actor=self.world_party(m,game)?.ok_or("Missing fixture party")?.actors.into_iter().find(|a|a.slot==slot).ok_or("Missing fixture actor")?;
  let row=self.support_row(m,game,slot)?;
  m.write(row+self.row_field(row==game,0x4f8),&(actor.maximum*0.5).to_le_bytes())
 }
}

impl Adapter {
 /// Stabilize only explicit diagnostic witnesses so their overhead labels stay
 /// in view. Ordinary enemies and production motion remain untouched.
 pub fn fixture_world_ui_hold(self,m:&dyn MutableMemory,game:u64,ids:&[(String,u64)])->Result<()> {
  let party=self.world_party(m,game)?.ok_or("Missing fixture party")?;
  if self.run_source(m,game)?.is_none_or(|(slot,_)|slot!=0)||ids.len()>8{return Err("Invalid witness authority".into())}
  let origin=party.actors.first().ok_or("Missing fixture actor")?.position;
  let h=game+self.game_field(0x570);let cap=word(m,h)?;let ptr=word(m,h+8)?;let n=word(m,h+16)?;
  if n>cap||cap>65536||n>8192||ptr<0x10000||ptr.checked_add(n*v::ENEMY_STRIDE as u64).is_none(){return Err("Invalid witness array".into())}
  for i in 0..n {
   let row=ptr+i*v::ENEMY_STRIDE as u64;let guid=word(m,row+v::ENEMY_GUID as u64)?;
   if let Some((role,_))=ids.iter().find(|(_,id)|*id==guid){
    let(dx,dy)=match role.as_str(){"elite"=>(-9.,15.),"guardian"=>(9.,15.),"boss"=>(0.,20.),_=>return Err("Unknown witness role".into())};
    let point=[origin[0]+dx,origin[1]+dy,origin[2]];
    let motion=v::ENEMY_HP as u64+28;
    let bytes=point.into_iter().chain(point).chain([party.clock,0.]).flat_map(f32::to_le_bytes).collect::<Vec<_>>();
    m.write(row+motion,&bytes)?;
   }
  }
  Ok(())
 }
}
