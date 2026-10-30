//! Optional creature integration for the reviewed native build. Native creation,
//! terrain clipping and ordinary object transport retain their original owners.
use super::Adapter;
use crate::current as v;
use crate::{
    memory::Mem,
    official::{Entry, MutableMemory},
    Result,
};
fn q(m: &dyn Mem, at: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(
        m.read(at, 8)?
            .try_into()
            .map_err(|_| "Truncated creature pointer")?,
    ))
}
fn number(b: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
#[repr(C, align(8))]
pub struct GoblinDefinition([u8; 80]);
impl GoblinDefinition {
    pub fn base_health(&self) -> f32 {
        number(&self.0, 48)
    }
    pub fn pointer(&self) -> u64 {
        self.0.as_ptr() as u64
    }
    fn from_native(mut b: [u8; 80]) -> Self {
        const NAME: &str = "Venture Co. Treasurer";
        b[..16].fill(0); // No native special attack.
        b[16..24].copy_from_slice(&(NAME.as_ptr() as u64).to_le_bytes());
        b[24..32].copy_from_slice(&(NAME.len() as u64).to_le_bytes());
        b[32..36].copy_from_slice(&v::GOBLIN_ENTRY.to_le_bytes());
        b[36..40].copy_from_slice(&v::GOBLIN_DISPLAY.to_le_bytes());
        b[40..44].copy_from_slice(&2.0f32.to_le_bytes());
        // Native rank reconstruction, time/Heroic/party coefficients still run.
        let hp = number(&b, 48).max(1.);
        b[48..52].copy_from_slice(&hp.to_le_bytes());
        b[52..56].copy_from_slice(&0f32.to_le_bytes()); // No outgoing damage.
        b[56..60].copy_from_slice(&5f32.to_le_bytes()); // Native attack interval, inert for this event.
        b[60..64].copy_from_slice(&5.2f32.to_le_bytes()); // Native movement speed.
        b[64..].fill(0); // No ordinary XP, coin or elite/guardian reward.
        Self(b)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct GoblinHandle {
    guid: u64,
    index: usize,
}
impl GoblinHandle {
    pub fn guid(self) -> u64 {
        self.guid
    }
}
#[derive(Clone, Copy, Debug)]
pub struct GoblinSnapshot {
    pub alive: bool,
    pub hp: f32,
    pub elite: bool,
    pub position: [f32; 3],
}
impl Adapter {
    /// Solo uses its native actor; cooperative events require restored host authority.
    /// The map's native center/radius are also used by native guardian leashing
    /// and ordinary spawn placement (reviewed 0x5c4642d and 0x5c50bf2..c30).
    pub fn goblin_arena(
        self,
        m: &dyn Mem,
        game: u64,
        base: u64,
    ) -> Result<asx_gameplay::treasure_goblins::Arena> {
        let (map, _) = self.blessing_map(m, game, base)?;
        let row = base + v::GOBLIN_MAP_TABLE + map * 240;
        let center = m.read(row + 0xc8, 8)?;
        let radius = number(&m.read(row + 0xd8, 4)?, 0);
        let arena = asx_gameplay::treasure_goblins::Arena {
            center: [number(&center, 0), number(&center, 4)],
            radius,
        };
        if arena.center.iter().any(|v| !v.is_finite())
            || !radius.is_finite()
            || !(4. ..=5000.).contains(&radius)
        {
            return Err("Invalid native event arena".into());
        }
        Ok(arena)
    }
    pub fn goblin_final_motion_entry(self) -> Entry {
        Entry {
            rva: v::GOBLIN_FINAL_MOTION.0,
            bytes: v::GOBLIN_FINAL_MOTION.1,
        }
    }
    pub fn goblin_final_motion(
        self,
        m: &dyn Mem,
        frame: u64,
        saved: u64,
        game: u64,
        guid: u64,
    ) -> Result<Option<([f32; 3], [f32; 3])>> {
        let basic = self.basic_frame(m, frame)?;
        if basic.game != game || basic.enemy != guid {
            return Ok(None);
        }
        let xy = m.read(saved, 8)?;
        let z = m.read(saved + 16, 4)?;
        let candidate = [number(&xy, 0), number(&xy, 4), number(&z, 0)];
        Ok(Some((basic.position(), candidate)))
    }
    pub fn goblin_hold_motion(
        self,
        m: &dyn MutableMemory,
        saved: u64,
        origin: [f32; 3],
    ) -> Result<()> {
        if origin.iter().any(|v| !v.is_finite()) {
            return Err("Invalid held event position".into());
        }
        let xy = origin[..2]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        m.write(saved, &xy)?;
        m.write(saved + 16, &origin[2].to_le_bytes())
    }
    pub fn goblin_run(self, m: &dyn Mem, game: u64) -> Result<Option<super::PartyObservation>> {
        if let Some(solo) = self.solo_run(m, game)? {
            return Ok(Some(solo));
        }
        self.world_party(m, game)
    }
    pub fn goblin_entries(self) -> Vec<Entry> {
        [
            v::GOBLIN_SPAWN,
            v::GOBLIN_DEFINITION,
            v::GOBLIN_PACKET_SEND,
            v::GOBLIN_QUERY_BUILD,
            v::GOBLIN_CLIP,
            v::GOBLIN_POSITION,
            v::GOBLIN_EVENT,
        ]
        .map(|(rva, bytes)| Entry { rva, bytes })
        .to_vec()
    }
    pub fn goblin_skin_entry(self) -> Entry {
        Entry {
            rva: v::GOBLIN_SKIN.0,
            bytes: v::GOBLIN_SKIN.1,
        }
    }
    /// The chosen legacy display's baked atlas is absent. Use its existing native
    /// Goblin skin atlas, through the engine's ordinary texture loader. No files
    /// are extracted or installed; unrelated texture names retain native behavior.
    pub fn goblin_skin_probe(
        self,
        m: &dyn Mem,
        name: u64,
        length: usize,
        frame: u64,
        caller: u64,
    ) -> Result<Option<String>> {
        let hash = b"3ef7f3eb90fa64d85635bc771b647bee.blp";
        if length != hash.len() || name < 0x10000 || m.read(name, length)? != hash {
            return Ok(None);
        }
        let details = if caller == v::GOBLIN_OUTFIT_SKIN_CALLER {
            let header = m.read(frame + v::GOBLIN_OUTFIT_ATLAS, 24)?;
            let key = m.read(frame + v::GOBLIN_OUTFIT_ARMOR_KEY, 64)?;
            let models = q(m, frame + v::GOBLIN_OUTFIT_MODELS)?;
            format!("header={header:02x?} key={key:02x?} models={models:x}")
        } else {
            String::new()
        };
        Ok(Some(format!("caller={caller:x} {details}")))
    }
    pub fn goblin_skin_path(
        self,
        m: &dyn MutableMemory,
        output: u64,
        name: u64,
        length: usize,
    ) -> Result<bool> {
        const MISSING: &[u8] = b"3ef7f3eb90fa64d85635bc771b647bee.blp";
        const SKIN: &[u8] = b"mpq://character/goblin/male/goblinmaleskin00_100.blp";
        if length != MISSING.len() || name < 0x10000 || m.read(name, length)? != MISSING {
            return Ok(false);
        }
        let pointer = m.allocate_native(SKIN)?;
        let result = [SKIN.len() as u64, pointer, SKIN.len() as u64]
            .into_iter()
            .flat_map(u64::to_le_bytes)
            .collect::<Vec<_>>();
        if let Err(e) = m.write(output, &result) {
            m.free_native(pointer, SKIN.len())?;
            return Err(e);
        }
        Ok(true)
    }
    pub fn goblin_outfit_entries(self) -> [Entry; 2] {
        [v::GOBLIN_OUTFIT_METADATA, v::GOBLIN_OUTFIT_KEY].map(|(rva, bytes)| Entry { rva, bytes })
    }
    /// The native metadata helper returns an owned baked atlas string. Convert
    /// just this NPC to its valid None(face=0) representation, retaining its
    /// native race/gender/skin metadata. Ownership is committed before freeing.
    pub fn goblin_outfit_metadata(self, m: &dyn MutableMemory, frame: u64) -> Result<Option<u64>> {
        let out = frame + v::GOBLIN_OUTFIT_ATLAS;
        let cap = q(m, out)?;
        if cap == u64::MAX || cap == u64::MAX - 1 {
            return Ok(None);
        }
        let ptr = q(m, out + 8)?;
        let len = q(m, out + 16)?;
        const HASH: &[u8] = b"3ef7f3eb90fa64d85635bc771b647bee.blp";
        if len != HASH.len() as u64 || ptr < 0x10000 || m.read(ptr, len as usize)? != HASH {
            return Ok(None);
        }
        if cap < len || cap > 4096 {
            return Err("Invalid owned Goblin atlas".into());
        }
        if m.read(out + 24, 6)? != [9, 0, 0, 0, 0, 0] {
            return Err("Unexpected native Goblin appearance metadata".into());
        }
        let entity = q(m, frame + v::GOBLIN_OUTFIT_ENTITY)?;
        let header = [u64::MAX, 0, 0]
            .into_iter()
            .flat_map(u64::to_le_bytes)
            .collect::<Vec<_>>();
        m.write(out, &header)?;
        m.free_native(ptr, cap as usize)?;
        Ok(Some(entity))
    }
    /// Populate the selected native armor block. Some compositor revisions
    /// keep the tag/appearance inline; others carry them in live registers.
    /// These display IDs reference reviewed native Nightslayer texture parts.
    pub fn goblin_outfit_key(self, m: &dyn MutableMemory, frame: u64, entity: u64) -> Result<()> {
        if q(m, frame + v::GOBLIN_OUTFIT_ENTITY)? != entity
            || m.read(frame + v::GOBLIN_OUTFIT_APPEARANCE, 6)? != [9, 0, 0, 0, 0, 0]
        {
            return Err("Goblin outfit owner changed".into());
        }
        let mut key = vec![0u8; v::GOBLIN_OUTFIT_KEY_SIZE];
        for (slot, display) in [(0x18, 31105u32), (0x20, 31340), (0x24, 31109)] {
            let at = slot + v::GOBLIN_OUTFIT_KEY_PREFIX;
            key[at..at + 4].copy_from_slice(&display.to_le_bytes());
        }
        if let Some(at) = v::GOBLIN_OUTFIT_KEY_RACE_OFFSET {
            key[at] = 9;
        }
        m.write(frame + v::GOBLIN_OUTFIT_ARMOR_KEY, &key)
    }
    pub fn goblin_recycle_entry(self) -> Entry {
        Entry {
            rva: v::GOBLIN_RECYCLE.0,
            bytes: v::GOBLIN_RECYCLE.1,
        }
    }
    /// Preserve the native list and order for every ordinary enemy. Only this
    /// run's event GUID is excluded from native far-distance recycling/teleports.
    pub fn goblin_filter_recycle(
        self,
        m: &dyn MutableMemory,
        iterator: u64,
        output: u64,
        game: u64,
        guid: u64,
    ) -> Result<bool> {
        if q(m, iterator + 16)? != game {
            return Ok(false);
        }
        let cap = q(m, output)?;
        let ptr = q(m, output + 8)?;
        let len = q(m, output + 16)?;
        if len > 4 || cap < len || cap > 65536 || (len > 0 && ptr < 0x10000) {
            return Err("Invalid native recycle list".into());
        }
        if len == 0 {
            return Ok(false);
        }
        let bytes = m.read(ptr, len as usize * 8)?;
        let kept = bytes
            .chunks_exact(8)
            .filter(|b| u64::from_le_bytes((*b).try_into().unwrap()) != guid)
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        if kept.len() == bytes.len() {
            return Ok(false);
        }
        if !kept.is_empty() {
            m.write(ptr, &kept)?;
        }
        m.write(output + 16, &((kept.len() / 8) as u64).to_le_bytes())?;
        Ok(true)
    }
    pub fn goblin_definition_entry(self) -> Entry {
        Entry {
            rva: v::GOBLIN_DEFINITION.0,
            bytes: v::GOBLIN_DEFINITION.1,
        }
    }
    pub fn goblin_template(
        self,
        m: &dyn Mem,
        game: u64,
        base: u64,
    ) -> Result<(usize, GoblinDefinition)> {
        let (map, _) = self.blessing_map(m, game, base)?;
        if map >= 12 {
            return Err("Unsupported event map".into());
        }
        let row = base + v::GOBLIN_MAP_TABLE + map * 240;
        let p = q(m, row + 0x78)?;
        let n = q(m, row + 0x80)?;
        if !(1..=64).contains(&n)
            || p < base
            || p.checked_add(n * 80)
                .is_none_or(|end| end > base + 0x8000000)
        {
            return Err("Invalid native horde definitions".into());
        }
        let rows = m.read(p, n as usize * 80)?;
        let guardian = rows
            .chunks_exact(80)
            .find(|b| b[76] == 1)
            .ok_or("Missing native map guardian")?;
        let data: [u8; 80] = guardian
            .try_into()
            .map_err(|_| "Truncated guardian template")?;
        if !number(&data, 48).is_finite() || !(1. ..=1e9).contains(&number(&data, 48)) {
            return Err("Invalid native guardian base health".into());
        }
        Ok((map as usize, GoblinDefinition::from_native(data)))
    }
    fn goblin_array(self, m: &dyn Mem, game: u64) -> Result<(u64, usize)> {
        let cap = q(m, game + 0x5b0)?;
        let p = q(m, game + 0x5b8)?;
        let n = q(m, game + 0x5c0)?;
        if n > cap
            || cap > 65536
            || n > 8192
            || (n > 0 && (p < 0x10000 || p.checked_add(n * 504).is_none()))
        {
            return Err("Invalid event enemy array".into());
        }
        Ok((p, n as usize))
    }
    /// Input registers match the reviewed Rust callee. Its optional tuple return
    /// is intentionally ignored; the authoritative array is rebound after callbacks.
    pub unsafe fn spawn_goblin(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
    ) -> Result<Option<GoblinHandle>> {
        let (_, before) = self.goblin_array(m, game)?;
        type Spawn = unsafe extern "system" fn(u64, u64, *const u8);
        let call: Spawn = std::mem::transmute((base + v::GOBLIN_SPAWN.0) as usize);
        type Position = unsafe extern "system" fn(*mut f32, u64, f32, f32, f32);
        let place: Position = std::mem::transmute((base + v::GOBLIN_POSITION.0) as usize);
        let mut point = [0f32; 3];
        place(point.as_mut_ptr(), game, 6., 9., 0.5);
        let party = self
            .goblin_run(m, game)?
            .ok_or("Missing event spawn authority")?;
        if !self.goblin_arena(m, game, base)?.contains(point)
            || point.iter().any(|v| !v.is_finite())
            || party
                .actors
                .iter()
                .filter(|a| a.hp > 0.)
                .any(|a| (a.position[0] - point[0]).hypot(a.position[1] - point[1]) < 4.)
        {
            return Ok(None);
        }
        let mut position = [0u8; 16];
        position[..4].copy_from_slice(&1u32.to_le_bytes());
        for (i, v) in point.into_iter().enumerate() {
            position[4 + i * 4..8 + i * 4].copy_from_slice(&v.to_le_bytes());
        }
        call(game, 0, position.as_ptr());
        let (p, n) = self.goblin_array(m, game)?;
        for i in before..n {
            let guid = q(m, p + i as u64 * 504 + v::ENEMY_GUID as u64)?;
            if ((guid >> 24) & 0xffffff) == u64::from(v::GOBLIN_ENTRY) {
                // The existing native raid-target decoder accepts a trusted local
                // command in solo and host runs, then broadcasts its normal update.
                // Reuse its reviewed table/command; never overwrite a manual mark.
                let server=game.checked_sub(16).ok_or("Invalid goblin server")?;
                let raw=m.read(server+v::SERVER_RAID_MARKS,64)?;
                if raw.len()!=64{return Err("Truncated goblin marker table".into())}
                let occupied=std::array::from_fn(|n|{let id=u64::from_le_bytes(raw[n*8..n*8+8].try_into().unwrap());(id!=0).then_some(id)});
                if skull_available(&occupied,guid){
                    let mut body=[0u8;9];body[0]=7;body[1..].copy_from_slice(&guid.to_le_bytes());
                    type Mark=unsafe extern "system" fn(u64,u64,u64,u16,*const u8,usize);
                    let mark:Mark=std::mem::transmute((base+self.command().rva)as usize);
                    mark(server,0,0,v::RAID_MARKER_COMMAND,body.as_ptr(),body.len());
                }
                return Ok(Some(GoblinHandle { guid, index: i }));
            }
        }
        Ok(None)
    }
    fn goblin_row(self, m: &dyn Mem, game: u64, h: &mut GoblinHandle) -> Result<Option<u64>> {
        let (p, n) = self.goblin_array(m, game)?;
        if h.index < n && q(m, p + h.index as u64 * 504 + v::ENEMY_GUID as u64)? == h.guid {
            return Ok(Some(p + h.index as u64 * 504));
        }
        for i in 0..n {
            if q(m, p + i as u64 * 504 + v::ENEMY_GUID as u64)? == h.guid {
                h.index = i;
                return Ok(Some(p + i as u64 * 504));
            }
        }
        Ok(None)
    }
    /// Target priority is an instance flag. Guardian drops are controlled separately
    /// by the immutable definition's +0x4c, which deliberately remains clear.
    pub fn goblin_elite(
        self,
        m: &dyn MutableMemory,
        game: u64,
        h: &mut GoblinHandle,
    ) -> Result<()> {
        let p = self
            .goblin_row(m, game, h)?
            .ok_or("Missing event instance for elite priority")?;
        m.write(p + 0x1f4, &[1])
    }
    pub fn goblin_snapshot(
        self,
        m: &dyn Mem,
        game: u64,
        h: &mut GoblinHandle,
    ) -> Result<Option<GoblinSnapshot>> {
        let Some(p) = self.goblin_row(m, game, h)? else {
            return Ok(None);
        };
        let b = m.read(p + 0x178, 60)?;
        let now = number(&m.read(game + crate::current::EVENT_CLOCK_OFFSET, 4)?, 0);
        let began = number(&b, 52);
        let duration = number(&b, 56);
        let t = if duration > 0. {
            ((now - began) / duration).clamp(0., 1.)
        } else {
            1.
        };
        let position = std::array::from_fn(|i| {
            let a = number(&b, 28 + i * 4);
            a + (number(&b, 40 + i * 4) - a) * t
        });
        let hp = number(&b, 0);
        if !hp.is_finite() || !now.is_finite() || position.iter().any(|v| !v.is_finite()) {
            return Err("Invalid goblin motion".into());
        }
        Ok(Some(GoblinSnapshot {
            alive: hp > 0. && m.read(p + 0xb0, 4)? != 1u32.to_le_bytes(),
            hp,
            elite: m.read(p + 0x1f4, 1)? == [1],
            position,
        }))
    }
    pub unsafe fn goblin_clip(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        origin: [f32; 3],
        goal: [f32; 3],
    ) -> Result<[f32; 3]> {
        self.event_surface(m, base, game, origin, goal)
    }

    pub unsafe fn goblin_name(self, m: &dyn MutableMemory, base: u64, game: u64) -> Result<()> {
        type Build = unsafe extern "system" fn(
            *mut u64,
            u32,
            *const u8,
            usize,
            *const u8,
            usize,
            u32,
            u32,
            u32,
        );
        let call: Build = std::mem::transmute((base + v::GOBLIN_QUERY_BUILD.0) as usize);
        let mut packet = [0u64; 4];
        let name = b"Venture Co. Treasurer";
        let sub = b"Catch it before it escapes!";
        call(
            packet.as_mut_ptr(),
            v::GOBLIN_ENTRY,
            name.as_ptr(),
            name.len(),
            sub.as_ptr(),
            sub.len(),
            7,
            1, // Elite presentation; combat priority is set on the native instance.
            v::GOBLIN_DISPLAY,
        );
        if packet[2] > 4096 || packet[0] < packet[2] || packet[0] > 1048576 || packet[1] < 0x10000 {
            return Err("Invalid native creature query output".into());
        }
        let result = m
            .read(packet[1], packet[2] as usize)
            .and_then(|body| self.event_broadcast(m, base, game, 0x61, &body));
        m.free_native(packet[1], packet[0] as usize)?;
        result
    }
    pub unsafe fn goblin_notice(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        text: &str,
    ) -> Result<()> {
        self.event_notice(m, base, game, "Venture Co.", text)
    }
    pub unsafe fn despawn_goblin(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        h: &mut GoblinHandle,
    ) -> Result<()> {
        if let Some(p) = self.goblin_row(m, game, h)? {
            m.write(p + 0xb0, &1u32.to_le_bytes())?;
            m.write(p + 0x178, &0f32.to_le_bytes())?;
        }
        self.event_broadcast(m, base, game, 0xaa, &h.guid.to_le_bytes())
    }
}
#[cfg(test)]
pub(super) mod tests {
    use super::*;

    use std::{cell::RefCell, collections::BTreeMap};
    #[derive(Default)]
    pub(in crate::official_native) struct Memory(RefCell<BTreeMap<u64, u8>>);
    impl Mem for Memory {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>> {
            (0..n)
                .map(|i| {
                    self.0
                        .borrow()
                        .get(&(a + i as u64))
                        .copied()
                        .ok_or("Unmapped event read".into())
                })
                .collect()
        }
    }
    impl MutableMemory for Memory {
        fn write(&self, a: u64, b: &[u8]) -> Result<()> {
            for (i, v) in b.iter().enumerate() {
                self.0.borrow_mut().insert(a + i as u64, *v);
            }
            Ok(())
        }
        fn allocate_native(&self, b: &[u8]) -> Result<u64> {
            self.write(0x90000, b)?;
            Ok(0x90000)
        }
        fn free_native(&self, _: u64, _: usize) -> Result<()> {
            Ok(())
        }
    }
    #[test]
    fn only_the_selected_goblin_skin_uses_native_body_atlas() {
        let a = Adapter::select(v::SHA256).unwrap();
        let m = Memory::default();
        let name = b"3ef7f3eb90fa64d85635bc771b647bee.blp";
        m.write(0x10000, name).unwrap();
        assert!(a
            .goblin_skin_path(&m, 0x20000, 0x10000, name.len())
            .unwrap());
        let cap = q(&m, 0x20000).unwrap();
        let ptr = q(&m, 0x20008).unwrap();
        let len = q(&m, 0x20010).unwrap();
        assert_eq!(cap, len);
        assert_eq!(
            m.read(ptr, len as usize).unwrap(),
            b"mpq://character/goblin/male/goblinmaleskin00_100.blp"
        );
        m.write(0x10000, b"x").unwrap();
        assert!(!a
            .goblin_skin_path(&m, 0x30000, 0x10000, name.len())
            .unwrap());
        assert!(m.read(0x30000, 1).is_err());
    }
    #[test]
    fn movement_probe_decodes_packed_guid_and_rejects_truncation() {
        let a = Adapter::select(v::SHA256).unwrap();
        let m = Memory::default();
        let guid = 0xf130000000000000u64 | (u64::from(v::GOBLIN_ENTRY) << 24) | 42;
        let mut body = vec![0u8];
        for (i, b) in guid.to_le_bytes().into_iter().enumerate() {
            if b != 0 {
                body[0] |= 1 << i;
                body.push(b);
            }
        }
        for f in [1f32, 2., 3.] {
            body.extend_from_slice(&f.to_le_bytes());
        }
        m.write(0x10000, &0x8000000000000006u64.to_le_bytes())
            .unwrap();
        m.write(0x10010, &0x20000u64.to_le_bytes()).unwrap();
        m.write(0x10018, &(body.len() as u64).to_le_bytes())
            .unwrap();
        m.write(0x10020, &0xddu16.to_le_bytes()).unwrap();
        m.write(0x20000, &body).unwrap();
        assert_eq!(
            a.goblin_packet_probe(&m, 0x10000).unwrap(),
            Some((0xdd, guid, Some([1., 2., 3.])))
        );
        m.write(0x10018, &1u64.to_le_bytes()).unwrap();
        assert_eq!(a.goblin_packet_probe(&m, 0x10000).unwrap(), None);
    }
    #[test]
    fn elite_target_priority_does_not_enable_guardian_drop_definition() {
        let a = Adapter::select(v::SHA256).unwrap();
        let m = Memory::default();
        let game = 0x10000;
        let row = 0x20000;
        let guid = 0xf130f5a001000001u64;
        for (offset, value) in [(0x5b0, 1u64), (0x5b8, row), (0x5c0, 1)] {
            m.write(game + offset, &value.to_le_bytes()).unwrap();
        }
        m.write(row + v::ENEMY_GUID as u64, &guid.to_le_bytes())
            .unwrap();
        m.write(row + 0x1f4, &[0]).unwrap();
        let mut handle = GoblinHandle { guid, index: 0 };
        a.goblin_elite(&m, game, &mut handle).unwrap();
        assert_eq!(m.read(row + 0x1f4, 1).unwrap(), [1]);
        let template = GoblinDefinition::from_native([0u8; 80]);
        assert_eq!(template.0[76], 0);
        assert_eq!(number(&template.0, 40), 2.);
    }
    #[test]
    fn recycle_filter_preserves_other_enemies_order_and_native_vector_ownership() {
        let a = Adapter::select(v::SHA256).unwrap();
        let m = Memory::default();
        m.write(0x10010, &7u64.to_le_bytes()).unwrap();
        for (offset, value) in [(0, 4u64), (8, 0x30000), (16, 4)] {
            m.write(0x20000 + offset, &value.to_le_bytes()).unwrap();
        }
        for (i, guid) in [10u64, 99, 11, 12].into_iter().enumerate() {
            m.write(0x30000 + i as u64 * 8, &guid.to_le_bytes())
                .unwrap();
        }
        assert!(!a
            .goblin_filter_recycle(&m, 0x10000, 0x20000, 8, 99)
            .unwrap());
        assert!(a
            .goblin_filter_recycle(&m, 0x10000, 0x20000, 7, 99)
            .unwrap());
        assert_eq!(q(&m, 0x20000).unwrap(), 4);
        assert_eq!(q(&m, 0x20008).unwrap(), 0x30000);
        assert_eq!(q(&m, 0x20010).unwrap(), 3);
        assert_eq!(
            [
                q(&m, 0x30000).unwrap(),
                q(&m, 0x30008).unwrap(),
                q(&m, 0x30010).unwrap()
            ],
            [10, 11, 12]
        );
        m.write(0x20010, &0u64.to_le_bytes()).unwrap();
        m.write(0x20008, &8u64.to_le_bytes()).unwrap();
        assert!(!a
            .goblin_filter_recycle(&m, 0x10000, 0x20000, 7, 99)
            .unwrap());
    }
    #[test]
    fn npc_outfit_uses_native_ownership_and_scopes_its_key_to_the_entity() {
        let a = Adapter::select(v::SHA256).unwrap();
        let m = Memory::default();
        let frame = 0x10000;
        let hash = b"3ef7f3eb90fa64d85635bc771b647bee.blp";
        let cap = hash.len() as u64;
        m.write(frame + 0xcb8, &cap.to_le_bytes()).unwrap();
        m.write(frame + 0xcc0, &0x30000u64.to_le_bytes()).unwrap();
        m.write(frame + 0xcc8, &cap.to_le_bytes()).unwrap();
        m.write(0x30000, hash).unwrap();
        m.write(frame + 0xcd0, &[9, 0, 0, 0, 0, 0]).unwrap();
        m.write(frame + 0xdf0, &71u64.to_le_bytes()).unwrap();
        assert_eq!(a.goblin_outfit_metadata(&m, frame).unwrap(), Some(71));
        assert_eq!(q(&m, frame + 0xcb8).unwrap(), u64::MAX);
        assert_eq!(a.goblin_outfit_metadata(&m, frame).unwrap(), None);
        assert!(a.goblin_outfit_key(&m, frame, 72).is_err());
        m.write(frame + 0x954, &[0xa5; 12]).unwrap();
        a.goblin_outfit_key(&m, frame, 71).unwrap();
        assert_eq!(m.read(frame + 0x954, 12).unwrap(), [0xa5; 12]);
        let key = m.read(frame + 0x920, 52).unwrap();
        assert_eq!(m.read(frame + 0xcd0, 6).unwrap(), [9, 0, 0, 0, 0, 0]);
        assert_eq!(
            u32::from_le_bytes(key[0x18..0x1c].try_into().unwrap()),
            31105
        );
    }
    #[test]
    fn final_navigation_scopes_the_creature_and_preserves_unrelated_register_lanes() {
        let a = Adapter::select(v::SHA256).unwrap();
        let m = Memory::default();
        let frame = 0x10000;
        let row = 0x20000u64;
        let saved = 0x30000;
        m.write(frame + v::BASIC_GAME, &7u64.to_le_bytes()).unwrap();
        m.write(frame + v::BASIC_ENEMY, &row.to_le_bytes()).unwrap();
        m.write(row + v::ENEMY_GUID as u64, &9u64.to_le_bytes())
            .unwrap();
        let origin = [1f32, 2., 3.];
        let goal = [4f32, 5., 6.];
        m.write(
            frame + v::BASIC_POSITION,
            &origin
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        m.write(saved, &[0xa5; 32]).unwrap();
        m.write(
            saved,
            &goal[..2]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        m.write(saved + 16, &goal[2].to_le_bytes()).unwrap();
        assert_eq!(
            a.goblin_final_motion(&m, frame, saved, 7, 10).unwrap(),
            None
        );
        assert_eq!(
            a.goblin_final_motion(&m, frame, saved, 7, 9).unwrap(),
            Some((origin, goal))
        );
        a.goblin_hold_motion(&m, saved, origin).unwrap();
        assert_eq!(
            a.goblin_final_motion(&m, frame, saved, 7, 9).unwrap(),
            Some((origin, origin))
        );
        assert_eq!(m.read(saved + 8, 8).unwrap(), [0xa5; 8]);
        assert_eq!(m.read(saved + 20, 12).unwrap(), [0xa5; 12]);
    }
    #[test]
    fn event_template_retains_native_shape_but_no_offense_or_ordinary_rewards() {
        let mut b = [0u8; 80];
        b[48..52].copy_from_slice(&600f32.to_le_bytes());
        let d = GoblinDefinition::from_native(b);
        assert_eq!(number(&d.0, 48), 600.);
        assert_eq!(number(&d.0, 52), 0.);
        assert!(d.0[..16].iter().all(|v| *v == 0));
        assert!(d.0[64..].iter().all(|v| *v == 0));
        assert_eq!(
            u32::from_le_bytes(d.0[32..36].try_into().unwrap()),
            v::GOBLIN_ENTRY
        );
    }
}

impl Adapter {
    pub fn goblin_packet_probe_entry(self) -> Entry {
        Entry {
            rva: v::GOBLIN_PACKET_PROBE.0,
            bytes: v::GOBLIN_PACKET_PROBE.1,
        }
    }
    /// Read-only diagnostic decoding at the native host-message boundary.
    pub fn goblin_packet_probe(
        self,
        m: &dyn Mem,
        message: u64,
    ) -> Result<Option<(u16, u64, Option<[f32; 3]>)>> {
        if q(m, message)? != 0x8000000000000006 {
            return Ok(None);
        }
        let op = u16::from_le_bytes(
            m.read(message + 32, 2)?
                .try_into()
                .map_err(|_| "Packet opcode")?,
        );
        if !matches!(op, 0xa9 | 0xaa | 0xdd | 0x61) {
            return Ok(None);
        }
        let p = q(m, message + 16)?;
        let n = q(m, message + 24)?;
        if p < 0x10000 || n > 65533 {
            return Err("Invalid diagnostic event packet".into());
        }
        let b = m.read(p, n as usize)?;
        let mut pose_at = 0;
        let guid = match op {
            0xa9 if b.len() >= 7 && b[5] == 3 => {
                let mask = b[6];
                let mut at = 7;
                let mut guid = 0u64;
                for i in 0..8 {
                    if mask & (1 << i) != 0 {
                        let Some(v) = b.get(at) else { return Ok(None) };
                        guid |= u64::from(*v) << (i * 8);
                        at += 1;
                    }
                }
                guid
            }
            0xdd if !b.is_empty() => {
                let mask = b[0];
                let mut at = 1;
                let mut guid = 0u64;
                for i in 0..8 {
                    if mask & (1 << i) != 0 {
                        let Some(v) = b.get(at) else { return Ok(None) };
                        guid |= u64::from(*v) << (i * 8);
                        at += 1;
                    }
                }
                pose_at = at;
                guid
            }
            0xaa if b.len() >= 8 => u64::from_le_bytes(b[..8].try_into().unwrap()),
            0x61 if b.len() >= 4
                && u32::from_le_bytes(b[..4].try_into().unwrap()) == v::GOBLIN_ENTRY =>
            {
                return Ok(Some((op, 0, None)))
            }
            _ => return Ok(None),
        };
        if ((guid >> 24) & 0xffffff) != u64::from(v::GOBLIN_ENTRY) {
            return Ok(None);
        }
        let position = if op == 0xdd && b.len() >= pose_at + 12 {
            Some(std::array::from_fn(|i| number(&b, pose_at + i * 4)))
        } else {
            None
        };
        Ok(Some((op, guid, position)))
    }
}

fn skull_available(occupied:&[Option<u64>;8],guid:u64)->bool{
    guid!=0&&!occupied.contains(&Some(guid))&&occupied[7].is_none_or(|id|((id>>24)&0xffffff)==u64::from(v::GOBLIN_ENTRY))
}
#[cfg(test)]mod skull_tests{
    use super::*;
    #[test]fn skull_reuses_previous_goblin_but_preserves_manual_player_marks(){
        let mut marks=[None;8];assert!(skull_available(&marks,42));marks[7]=Some(123);assert!(!skull_available(&marks,42));
        marks[7]=Some((u64::from(v::GOBLIN_ENTRY)<<24)|1);assert!(skull_available(&marks,42));marks[0]=Some(42);assert!(!skull_available(&marks,42));
    }
}
