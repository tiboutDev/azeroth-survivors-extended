//! Selected terrain and minimap translation for neutral world-event presentation.
use super::Adapter;
use crate::current as v;
use crate::{
    memory::Mem,
    official::{Entry, MutableMemory},
    Result,
};
use azsx_gameplay::world_events::{Area, GroundSite};
fn q(m: &dyn Mem, at: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(
        m.read(at, 8)?
            .try_into()
            .map_err(|_| "Truncated event pointer")?,
    ))
}
fn f(m: &dyn Mem, at: u64) -> Result<f32> {
    let x = f32::from_le_bytes(
        m.read(at, 4)?
            .try_into()
            .map_err(|_| "Truncated event coordinate")?,
    );
    if !x.is_finite() {
        return Err("Invalid event coordinate".into());
    }
    Ok(x)
}
/// Native Rust Option<f32> uses AL for presence and XMM0 for the value.
#[cfg(target_arch = "x86_64")]
unsafe fn height(entry: usize, terrain: u64, x: f32, y: f32) -> Option<f32> {
    let tag: u64;
    let z: f32;
    core::arch::asm!("sub rsp, 32","call r11","add rsp, 32",in("r11")entry,in("rcx")terrain,in("xmm1")x,in("xmm2")y,lateout("rax")tag,lateout("xmm0")z,clobber_abi("win64"));
    (tag & 1 != 0 && z.is_finite()).then_some(z)
}
#[derive(Clone, Copy, Debug)]
pub struct EventMinimap {
    pub center: [f32; 2],
    pub radii: [f32; 6],
    pub ping: [f32; 2],
}
impl Adapter {
    pub fn ui_context_entry(self) -> Entry {
        let (rva, bytes) = v::UI_CONTEXT_LOOKUP;
        Entry { rva, bytes }
    }
    /// The getter's Rc is owned and released by its original caller. Observe only;
    /// never increment/decrement it or retain an additional native borrow.
    pub fn ui_context_result(self, root: u64) -> Option<u64> {
        (root >= 0x10000).then(|| root.checked_add(0x18)).flatten()
    }
    pub fn event_placement_entries(self) -> Vec<Entry> {
        [v::EVENT_GROUND_HEIGHT]
            .into_iter()
            .map(|(rva, bytes)| Entry { rva, bytes })
            .collect()
    }
    pub fn event_area(self, m: &dyn Mem, base: u64, game: u64) -> Result<Area> {
        let (map, _) = self.blessing_map(m, game, base)?;
        let row = base + v::GOBLIN_MAP_TABLE + map * 240;
        let area = Area {
            center: [f(m, row + 0xc8)?, f(m, row + 0xcc)?],
            radius: f(m, row + 0xd8)?,
        };
        if !(10. ..=5000.).contains(&area.radius) {
            return Err("Invalid event map bounds".into());
        }
        Ok(area)
    }
    /// The reviewed height getter overlays this half-yard liquid grid on terrain.
    /// Reject any finite liquid sample conservatively, including water beneath
    /// elevated ground. Never confuse a navigable water surface with dry land.
    fn event_dry(self, m: &dyn Mem, terrain: u64, p: [f32; 2]) -> Result<bool> {
        let grid = terrain + 0xa8;
        let cap = q(m, grid)?;
        if cap == u64::MAX { return Ok(true); }
        let ptr = q(m, grid+8)?;
        let len = q(m, grid+16)?;
        let dim = q(m, grid+24)?;
        if dim == 0 && len == 0 { return Ok(true); }
        if dim == 0 || dim > 16384 || len > cap || len < dim*dim
            || cap > 256*1024*1024 || ptr < 0x10000 {
            return Err("Invalid native event liquid grid".into());
        }
        let x = ((p[0]-f(m,grid+32)?)*2.).floor();
        let y = ((p[1]-f(m,grid+36)?)*2.).floor();
        if x < 0. || y < 0. || x >= dim as f32 || y >= dim as f32 { return Ok(true); }
        let raw = m.read(ptr+(y as u64*dim+x as u64)*4,4)?;
        let level = f32::from_le_bytes(raw.try_into().map_err(|_|"Truncated event liquid sample")?);
        Ok(level.is_nan())
    }
    fn event_walkable(self, m: &dyn Mem, terrain: u64, p: [f32; 2]) -> Result<bool> {
        if !self.event_dry(m,terrain,p)? { return Ok(false); }
        let grid = terrain + 0x40;
        let cap = q(m, grid)?;
        if cap == u64::MAX {
            return Ok(false);
        }
        let ptr = q(m, grid + 8)?;
        let len = q(m, grid + 16)?;
        let dim = q(m, grid + 24)?;
        if dim == 0
            || dim > 8192
            || len > cap
            || len < dim * dim
            || cap > 64 * 1024 * 1024
            || ptr < 0x10000
        {
            return Err("Invalid native event navigation grid".into());
        }
        let x = (p[0] - f(m, grid + 32)?).floor();
        let y = (p[1] - f(m, grid + 36)?).floor();
        if x < 0. || y < 0. || x >= dim as f32 || y >= dim as f32 {
            return Ok(false);
        }
        Ok(m.read(ptr + y as u64 * dim + x as u64, 1)?[0] & 1 == 0)
    }
    pub unsafe fn event_ground(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        point: [f32; 2],
        footprint: f32,
        max_slope: f32,
    ) -> Result<Option<[f32; 3]>> {
        if !max_slope.is_finite() || max_slope < 0. {
            return Err("Invalid event slope budget".into());
        }
        Ok(self
            .event_ground_sample(m, base, game, point, footprint)?
            .filter(|s| s.high - s.low <= max_slope)
            .map(|s| s.position(max_slope)))
    }
    pub unsafe fn event_ground_sample(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        point: [f32; 2],
        footprint: f32,
    ) -> Result<Option<GroundSite>> {
        if point.iter().any(|v| !v.is_finite())
            || !footprint.is_finite()
            || !(0.1..=10.).contains(&footprint)
        {
            return Err("Invalid event footprint".into());
        }
        let area = self.event_area(m, base, game)?;
        if !area.contains(point, footprint + 0.25) {
            return Ok(None);
        }
        let terrain = q(m, game + crate::current::EVENT_TERRAIN_OFFSET)?;
        if terrain < 0x10000 {
            return Err("Missing event terrain".into());
        }
        let Some(center) = height(
            (base + v::EVENT_GROUND_HEIGHT.0) as usize,
            terrain + 0x10,
            point[0],
            point[1],
        ) else {
            return Ok(None);
        };
        let (mut low, mut high) = (center, center);
        for i in 0..=8 {
            let p = if i == 8 {
                point
            } else {
                let angle = i as f32 * std::f32::consts::TAU / 8.;
                [
                    point[0] + footprint * angle.cos(),
                    point[1] + footprint * angle.sin(),
                ]
            };
            if !self.event_walkable(m, terrain, p)? {
                return Ok(None);
            }
            let Some(z) = height(
                (base + v::EVENT_GROUND_HEIGHT.0) as usize,
                terrain + 0x10,
                p[0],
                p[1],
            ) else {
                return Ok(None);
            };
            low = low.min(z);
            high = high.max(z);
        }
        // A clear rim does not prove a clear interior: large decorative tree
        // footprints can occupy cells between the center and eight rim probes.
        for sample in azsx_gameplay::world_events::interior_clearance_points(point, footprint) {
            if !self.event_walkable(m, terrain, sample)? { return Ok(None); }
        }
        Ok(Some(GroundSite {
            center: [point[0], point[1], center],
            low,
            high,
        }))
    }
    /// The same UI state read by native GetPingPosition, relative to its borrowed data.
    pub fn event_minimap(
        self,
        m: &dyn Mem,
        script: u64,
        owner: u64,
    ) -> Result<Option<EventMinimap>> {
        if script < 0x10000 || self.native_ui_participants(m, script)?.first() != Some(&owner) {
            return Ok(None);
        }
        let [visible, indoor, center_x, center_y, ping_x, ping_y] = v::MINIMAP_STATE_FIELDS;
        if m.read(script + visible, 1)?[0] & 1 == 0 {
            return Ok(None);
        }
        let indoor = match m.read(script + indoor, 1)?.as_slice() {
            [0] => false,
            [1] => true,
            _ => return Err("Invalid native minimap mode".into()),
        };
        let radii = if indoor {
            [150., 120., 90., 60., 40., 25.]
        } else {
            [14., 12., 10., 8., 6., 4.].map(|v| v * 16.666666)
        };
        Ok(Some(EventMinimap {
            center: [f(m, script + center_x)?, f(m, script + center_y)?],
            radii,
            ping: [f(m, script + ping_x)?, f(m, script + ping_y)?],
        }))
    }
    /// A personal supply restores native HP directly without casting a healing spell.
    pub fn drink_supply_potion(self,m:&dyn MutableMemory,game:u64,id:(u32,u64),potion:azsx_gameplay::supplies::Consumable)->Result<bool>{
        let Some(party)=self.event_party(m,game)? else{return Ok(false)};
        let Some(actor)=party.actors.iter().find(|a|(a.slot,a.guid)==id && a.phase==azsx_gameplay::Phase::Playing) else{return Ok(false)};
        let Some(hp)=azsx_gameplay::supplies::potion_health(actor.hp,actor.maximum,potion) else{return Ok(false)};
        let row=self.support_row(m,game,id.0)?;
        m.write(row+self.row_field(row==game,0x4f8),&hp.to_le_bytes())?;
        Ok(true)
    }
    pub fn clamp_event_health(
        self,
        m: &dyn MutableMemory,
        game: u64,
        owner: u64,
        before: f32,
    ) -> Result<()> {
        if !before.is_finite() || before < 0. {
            return Err("Invalid temporary health budget".into());
        }
        let p = self
            .event_party(m, game)?
            .ok_or("Temporary health run ended")?;
        let a = p
            .actors
            .iter()
            .find(|a| a.guid == owner)
            .ok_or("Temporary health owner changed")?;
        if !a.hp.is_finite() || !a.maximum.is_finite() || a.maximum <= 0. {
            return Err("Invalid temporary health state".into());
        }
        let hp = a.hp.min(before).min(a.maximum).max(0.);
        if hp != a.hp {
            let row = self.support_row(m, game, a.slot)?;
            m.write(row + self.row_field(row == game, 0x4f8), &hp.to_le_bytes())?;
        }
        Ok(())
    }
}
#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    #[unsafe(naked)]
    unsafe extern "system" fn sample() {
        core::arch::naked_asm!("mov eax, 1", "movaps xmm0, xmm1", "addss xmm0, xmm2", "ret");
    }
    #[unsafe(naked)]
    unsafe extern "system" fn absent() {
        core::arch::naked_asm!("xor eax, eax", "ret");
    }
    #[test]
    fn mixed_native_height_return_and_win64_arguments() {
        unsafe {
            assert_eq!(
                height(sample as *const () as usize, 0x123, [2., 3.][0], 3.),
                Some(5.)
            );
            assert_eq!(height(absent as *const () as usize, 0x123, 2., 3.), None);
        }
    }
}

#[cfg(test)]
mod dry_site_tests {
    use super::*;
    struct Bytes(Vec<u8>);
    impl Mem for Bytes {
        fn read(&self, at:u64, len:usize)->Result<Vec<u8>> {
            self.0.get(at as usize..at as usize+len).map(|v|v.to_vec()).ok_or_else(||"Out of fixture".into())
        }
    }
    fn grid(level:f32)->Bytes {
        let mut m=Bytes(vec![0;0x12000]);let g=0x1000+0xa8;
        for (at,value) in [(g,4u64),(g+8,0x11000),(g+16,4),(g+24,2)] {m.0[at..at+8].copy_from_slice(&value.to_le_bytes());}
        for n in 0..4 {m.0[0x11000+n*4..0x11004+n*4].copy_from_slice(&level.to_le_bytes());}m
    }
    #[test]fn liquids_are_rejected_even_when_navigation_would_allow_them(){
        let a=Adapter::select(v::SHA256).unwrap();
        for level in [-10.,0.,12.,f32::INFINITY] {assert!(!a.event_dry(&grid(level),0x1000,[0.25,0.25]).unwrap());}
        assert!(a.event_dry(&grid(f32::NAN),0x1000,[0.25,0.25]).unwrap());
        let mut absent=grid(0.);absent.0[0x10a8..0x10b0].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(a.event_dry(&absent,0x1000,[0.25,0.25]).unwrap());
        let mut malformed=grid(0.);malformed.0[0x10c0..0x10c8].copy_from_slice(&0u64.to_le_bytes());
        assert!(a.event_dry(&malformed,0x1000,[0.25,0.25]).is_err());
    }
}
