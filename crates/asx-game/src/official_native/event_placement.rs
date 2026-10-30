//! Selected terrain and minimap translation for neutral world-event presentation.
use super::Adapter;
use crate::current as v;
use crate::{
    memory::Mem,
    official::{Entry, MutableMemory},
    Result,
};
use asx_gameplay::world_events::{Area, GroundSite};
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
    pub radius: f32,
    pub zoom: u32,
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
    fn event_walkable(self, m: &dyn Mem, terrain: u64, p: [f32; 2]) -> Result<bool> {
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
        if m.read(script + 0x6648, 1)?[0] & 1 == 0 {
            return Ok(None);
        }
        let indoor = match m.read(script + 0x665c, 1)?.as_slice() {
            [0] => false,
            [1] => true,
            _ => return Err("Invalid native minimap mode".into()),
        };
        let zoom = u32::from_le_bytes(
            m.read(script + if indoor { 0x6658 } else { 0x6654 }, 4)?
                .try_into()
                .map_err(|_| "Truncated minimap zoom")?,
        )
        .min(5) as usize;
        let radii = if indoor {
            [150., 120., 90., 60., 40., 25.]
        } else {
            [14., 12., 10., 8., 6., 4.].map(|v| v * 16.666666)
        };
        Ok(Some(EventMinimap {
            center: [f(m, script + 0x664c)?, f(m, script + 0x6650)?],
            radius: radii[zoom],
            zoom: zoom as u32,
            radii,
            ping: [f(m, script + 0x6640)?, f(m, script + 0x6644)?],
        }))
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
