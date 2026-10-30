//! Common cosmetic locations and map-wide native placement; no player-relative fallback.
use super::event_services::Services;
pub mod coop;
use asx_gameplay::world_events::{Area,separated};
#[derive(Clone,Copy,Debug)]pub struct Marker{pub id:u64,pub kind:u8,pub position:[f32;3],pub remaining:f32}
pub fn occupied(game:u64)->Vec<([f32;3],f32)>{let mut out=super::supplies::occupied(game);out.extend(coop::occupied(game));if let Some(p)=super::rituals::position(game){out.push((p,8.));}out}
pub fn place(r:&dyn Services,game:u64,area:Area,footprint:f32,draws:&[(u32,u32)])->Result<Option<[f32;3]>,String>{
 let occupied=occupied(game);let mut sites=Vec::with_capacity(32);
 for (radial,angle)in draws.iter().take(32){let Some(p)=area.sample(footprint+0.5,*radial,*angle)else{return Ok(None)};
  if !separated(p,footprint,&occupied){continue}
  if let Some(site)=unsafe{r.events_ground_sample(game, p, footprint)}?{sites.push(site);}
 }
 let preferred=if footprint>4.{0.60}else{0.70};
 let chosen=asx_gameplay::world_events::choose_site(&sites,preferred);
 if let Some(site)=chosen{r.events_info(&format!("events placement: footprint={footprint} height_variation={} preferred_flat={}",site.high-site.low,site.high-site.low<=preferred));}
 Ok(chosen.map(|site|site.position(preferred)))
}
pub fn markers(game:u64)->Vec<Marker>{let mut out=super::supplies::markers(game);if let Some(p)=super::rituals::marker(game){out.push(p)}out}
