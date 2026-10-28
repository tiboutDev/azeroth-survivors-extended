//! Native cosmetic raid-target command; no extra transport or Lua roster.
use super::*;
impl Adapter {
 pub fn raid_markers(self,m:&dyn Mem,server:u64)->Result<[Option<u64>;8]>{
  if self.observe(m,server)?.role!=Role::Host{return Err("Marker initialization requires host authority".into())}
  let raw=m.read(server+crate::official125::SERVER_RAID_MARKS,64)?;
  if raw.len()!=64{return Err("Truncated native markers".into())}
  Ok(std::array::from_fn(|i|{let id=u64::from_le_bytes(raw[i*8..i*8+8].try_into().unwrap());(id!=0).then_some(id)}))
 }
 pub unsafe fn set_raid_marker(self,m:&dyn Mem,base:u64,server:u64,slot:u32,guid:u64,icon:u8)->Result<bool>{
  if !(1..=8).contains(&icon){return Err("Invalid native marker".into())}
  let party=self.party(m,server)?.ok_or("Missing cooperative marker run")?;
  if !party.actors.iter().any(|a|a.slot==slot&&a.guid==guid&&a.phase!=asx_gameplay::Phase::Inactive){return Err("Marker participant changed".into())}
  // A manual assignment may arrive after planning, including during a previous
  // native marker callback. Never displace its icon or replace this player's mark.
  if !asx_gameplay::raid_markers::can_assign(&self.raid_markers(m,server)?,guid,icon){return Ok(false)}
  // Native wire is zero-based icon followed by the full little-endian GUID.
  let mut body=[0;9];body[0]=icon-1;body[1..].copy_from_slice(&guid.to_le_bytes());
  type Command=unsafe extern "system" fn(u64,u64,u64,u16,*const u8,usize);
  // This guarded selected entry is already installed by bootstrap. Its normal
  // ASX forwarding stub admits marker commands even during reward/menu pauses.
  let call:Command=std::mem::transmute((base+self.command().rva)as usize);
  call(server,0,0,crate::official125::RAID_MARKER_COMMAND,body.as_ptr(),body.len());
  Ok(true)
 }
}
