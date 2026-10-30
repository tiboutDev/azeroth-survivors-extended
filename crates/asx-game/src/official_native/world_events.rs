//! Shared native world-event presentation. Optional feature policy stays in gameplay.
use super::{Adapter, PartyObservation};
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
            .map_err(|_| "Truncated scene pointer")?,
    ))
}
#[derive(Clone, Copy, Debug)]
pub struct EventObject {
    pub(super) guid: u64,
    pub position: [f32; 3],
}
impl EventObject {
    pub fn guid(self) -> u64 {
        self.guid
    }
}
impl Adapter {
    pub fn event_party(self, m: &dyn Mem, game: u64) -> Result<Option<PartyObservation>> {
        if let Some(p) = self.solo_run(m, game)? {
            return Ok(Some(p));
        }
        self.world_party(m, game)
    }
    /// The native sender borrows this packet; it owns any clones it queues.
    pub unsafe fn event_broadcast(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        opcode: u16,
        body: &[u8],
    ) -> Result<()> {
        if body.len() > 4096 || !matches!(opcode, 0x61 | 0xaa | 0x96 | 0x5f | 0xa9) {
            return Err("Invalid event presentation packet".into());
        }
        let party = self
            .event_party(m, game)?
            .ok_or("Missing event authority")?;
        if self.run_source(m, game)?.is_none_or(|(slot, _)| slot != 0) {
            return Err("Event requires restored authority".into());
        }
        let mut packet = [0u64; 4];
        packet[0] = body.len() as u64;
        packet[1] = body.as_ptr() as u64;
        packet[2] = body.len() as u64;
        packet[3] = u64::from(opcode);
        type Send = unsafe extern "system" fn(u64, u64, *const u64);
        let call: Send = std::mem::transmute((base + v::GOBLIN_PACKET_SEND.0) as usize);
        for actor in party.actors {
            call(game - 16, u64::from(actor.slot), packet.as_ptr());
        }
        Ok(())
    }
    pub unsafe fn event_notice(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        sender: &str,
        text: &str,
    ) -> Result<()> {
        if text.len() > 240
            || text.as_bytes().contains(&0)
            || sender.len() > 80
            || sender.as_bytes().contains(&0)
        {
            return Err("Invalid event notice".into());
        }
        // Same native CHAT_MSG 0x5a builder as the horde-wave announcements.
        type Build = unsafe extern "system" fn(*mut u64, *const u8, usize, *const u8, usize);
        let call: Build = std::mem::transmute((base + v::GOBLIN_EVENT.0) as usize);
        let mut packet = [0u64; 4];
        let name = sender.as_bytes();
        call(
            packet.as_mut_ptr(),
            name.as_ptr(),
            name.len(),
            text.as_ptr(),
            text.len(),
        );
        if packet[2] > 4096 || packet[0] < packet[2] || packet[0] > 1048576 || packet[1] < 0x10000 {
            return Err("Invalid native event announcement".into());
        }
        let result = m
            .read(packet[1], packet[2] as usize)
            .and_then(|body| self.event_broadcast(m, base, game, 0x96, &body));
        m.free_native(packet[1], packet[0] as usize)?;
        result
    }

    pub fn world_event_entries(self) -> Vec<Entry> {
        [
            v::EVENT_OBJECT_CREATE,
            v::EVENT_OBJECT_INFO,
            v::GOBLIN_PACKET_SEND,
            v::GOBLIN_EVENT,
            v::GOBLIN_POSITION,
        ]
        .into_iter()
        .map(|(rva, bytes)| Entry { rva, bytes })
        .collect()
    }
    /// Native GUID allocation and GameObject serialization; no purchased-cache bank or AI.
    pub unsafe fn event_object(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        visual: u8,
        position: [f32; 3],
    ) -> Result<EventObject> {
        if self.event_party(m, game)?.is_none() || position.iter().any(|x| !x.is_finite()) {
            return Err("World-event prototype requires a valid authoritative run".into());
        }
        let (entry, display, scale, name) = match visual {
            0 => (0xa50001u32, 36u32, 1.0f32, "Lost Supplies"),
            1 => (0xa50002, 465, 3.2940652, "Sayge's Trial"),
            2 => (0xa50003, 259, 1.1, "Sealed Reliquary"),
            3 => (0xa50004, 4152, 1.6, "Ritual Candle"),
            4 => (0xa50005, 2973, 0.75, "Red Crystal"),
            5 => (0xa50006, 2971, 0.75, "Blue Crystal"),
            6 => (0xa50007, 2972, 0.75, "Green Crystal"),
            7 => (0xa50008, 2974, 0.75, "Yellow Crystal"),
            _ => return Err("Invalid owned event visual".into()),
        };
        type Info = unsafe extern "system" fn(*mut u64, u32, u32, u32, *const u8, usize);
        let info: Info = std::mem::transmute((base + v::EVENT_OBJECT_INFO.0) as usize);
        let mut packet = [0u64; 4];
        info(
            packet.as_mut_ptr(),
            entry,
            5,
            display,
            name.as_ptr(),
            name.len(),
        );
        if packet[0] < packet[2] || packet[2] > 4096 || packet[0] > 1048576 || packet[1] < 0x10000 {
            return Err("Invalid owned object metadata".into());
        }
        let sent = m
            .read(packet[1], packet[2] as usize)
            .and_then(|body| self.event_broadcast(m, base, game, 0x5f, &body));
        m.free_native(packet[1], packet[0] as usize)?;
        sent?;
        type Create = unsafe extern "system" fn(u64, *const u32, *const f32, u32, u32) -> u64;
        let create: Create = std::mem::transmute((base + v::EVENT_OBJECT_CREATE.0) as usize);
        let descriptor = [entry, display, scale.to_bits()];
        // The native constructor appends to the current actor's packet queue.
        // Take that exact new packet before callbacks and deliver through the
        // native participant sender, so decorative objects exist for everyone.
        let queued = q(m, game + 0x680)?;
        let guid = create(game, descriptor.as_ptr(), position.as_ptr(), 5, 0);
        if ((guid >> 24) & 0xffffff) != u64::from(entry) {
            return Err("Native event GUID identity rejected".into());
        }
        let packet = self.take_created_event_packet(m, game, queued)?;
        self.event_broadcast(m, base, game, 0xa9, &packet)?;
        Ok(EventObject { guid, position })
    }
    fn take_created_event_packet(
        self,
        m: &dyn MutableMemory,
        game: u64,
        before: u64,
    ) -> Result<Vec<u8>> {
        let cap = q(m, game + 0x670)?;
        let ptr = q(m, game + 0x678)?;
        let len = q(m, game + 0x680)?;
        if before >= 4096 || len != before + 1 || len > cap || cap > 4096 || ptr < 0x10000 {
            return Err("Native event queue changed".into());
        }
        let row = ptr + before * 32;
        let allocation = q(m, row)?;
        let bytes = q(m, row + 8)?;
        let size = q(m, row + 16)?;
        let opcode = u16::from_le_bytes(
            m.read(row + 24, 2)?
                .try_into()
                .map_err(|_| "Truncated event opcode")?,
        );
        if opcode != 0xa9
            || allocation < size
            || allocation > 1048576
            || size > 4096
            || bytes < 0x10000
        {
            return Err("Invalid created event packet".into());
        }
        let body = m.read(bytes, size as usize)?;
        m.write(game + 0x680, &before.to_le_bytes())?;
        m.free_native(bytes, allocation as usize)?;
        Ok(body)
    }
    /// Personal disappearance uses the same native destroy packet, sent once to
    /// the actor whose claim was already committed by the host.
    pub unsafe fn hide_event_object(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        slot: u32,
        guid: u64,
        object: EventObject,
    ) -> Result<()> {
        let party = self.event_party(m, game)?.ok_or("Missing event party")?;
        if self.run_source(m, game)?.is_none_or(|(s, _)| s != 0)
            || !party
                .actors
                .iter()
                .any(|a| a.slot == slot && a.guid == guid)
        {
            return Err("Personal event recipient changed".into());
        }
        let body = object.guid.to_le_bytes();
        let packet = [8u64, body.as_ptr() as u64, 8, 0xaa];
        let call: unsafe extern "system" fn(u64, u64, *const u64) =
            std::mem::transmute((base + v::GOBLIN_PACKET_SEND.0) as usize);
        call(game - 16, u64::from(slot), packet.as_ptr());
        Ok(())
    }
    pub unsafe fn remove_event_object(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        object: EventObject,
    ) -> Result<()> {
        self.event_broadcast(m, base, game, 0xaa, &object.guid.to_le_bytes())
    }
}

impl Adapter {
    pub fn project_consumable_auras(
        self,
        m: &dyn MutableMemory,
        f: &super::FatigueAura,
        buffs: &[(asx_gameplay::supplies::Family, f32)],
    ) -> Result<Vec<(asx_gameplay::supplies::Family, u32)>> {
        if buffs.len() > asx_gameplay::supplies::Family::ALL.len()
            || buffs
                .iter()
                .any(|(_, v)| !v.is_finite() || *v <= 0. || *v > asx_gameplay::supplies::BUFF_SECONDS+0.01)
        {
            return Err("Invalid consumable aura budget".into());
        }
        let bytes = m.read(f.rows, 32 * 16)?;
        if bytes.len() != 512 {
            return Err("Truncated consumable aura slots".into());
        }
        let mut free = vec![];
        let mut existing = std::collections::BTreeMap::new();
        for (index, row) in bytes.chunks_exact(16).enumerate() {
            let tag = u32::from_le_bytes(row[..4].try_into().unwrap());
            if tag > 1 {
                return Err("Invalid event buff slot".into());
            }
            if tag == 0 {
                free.push(index)
            } else {
                let spell = u32::from_le_bytes(row[4..8].try_into().unwrap());
                if (2330..2337).contains(&spell) {
                    existing.insert(spell, index);
                }
            }
        }
        let requested = buffs
            .iter()
            .map(|(family, _)| 2330 + *family as u32)
            .collect::<std::collections::BTreeSet<_>>();
        if requested.len() != buffs.len() {
            return Err("Duplicate consumable aura family".into());
        }
        // These six recipe-only IDs are ASX presentation slots, never native class
        // effects. Retire only our inactive rows; preserve every unrelated native row.
        for (spell, index) in &existing {
            if !requested.contains(spell) {
                m.write(f.rows + *index as u64 * 16, &[0; 16])?;
                free.push(*index);
            }
        }
        let mut result = vec![];
        let mut free = free.into_iter();
        for (family, remaining) in buffs {
            let spell = 2330 + *family as u32;
            let Some(index) = existing.get(&spell).copied().or_else(|| free.next()) else {
                continue;
            };
            let mut row = [0u8; 16];
            row[..4].copy_from_slice(&1u32.to_le_bytes());
            row[4..8].copy_from_slice(&spell.to_le_bytes());
            row[8..12].copy_from_slice(&(f.clock + remaining).to_le_bytes());
            row[12..].copy_from_slice(&1u32.to_le_bytes());
            m.write(f.rows + index as u64 * 16, &row)?;
            result.push((*family, index as u32));
        }
        Ok(result)
    }
    pub fn event_aura_readback(
        self,
        m: &dyn Mem,
        f: &super::FatigueAura,
    ) -> Result<Vec<(u32, u32)>> {
        Ok(m.read(f.rows, 512)?
            .chunks_exact(16)
            .enumerate()
            .filter(|(_, row)| row[..4] == 1u32.to_le_bytes())
            .map(|(i, row)| (i as u32, u32::from_le_bytes(row[4..8].try_into().unwrap())))
            .collect())
    }
    pub fn event_test_stat_input(self, m: &dyn Mem, game: u64) -> Result<Vec<u8>> {
        m.read(
            game + self.field1201(crate::current::RING_STAT_INPUT),
            24,
        )
    }
}

impl Adapter {
    pub unsafe fn event_position(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        near: f32,
        far: f32,
    ) -> Result<Option<[f32; 3]>> {
        if !near.is_finite()
            || !far.is_finite()
            || near < 0.5
            || far < near
            || far > 30.
            || self.event_party(m, game)?.is_none()
        {
            return Err("Invalid solo event placement".into());
        }
        type Place = unsafe extern "system" fn(*mut f32, u64, f32, f32, f32);
        let place: Place = std::mem::transmute((base + v::GOBLIN_POSITION.0) as usize);
        let mut point = [0f32; 3];
        place(point.as_mut_ptr(), game, near, far, 0.5);
        let (map, _) = self.blessing_map(m, game, base)?;
        let row = base + v::GOBLIN_MAP_TABLE + map * 240;
        let center = m.read(row + 0xc8, 8)?;
        let x = f32::from_le_bytes(center[..4].try_into().unwrap());
        let y = f32::from_le_bytes(center[4..].try_into().unwrap());
        let radius = f32::from_le_bytes(
            m.read(row + 0xd8, 4)?
                .try_into()
                .map_err(|_| "Event arena extent")?,
        );
        if point.iter().any(|v| !v.is_finite())
            || !radius.is_finite()
            || radius <= 2.
            || (point[0] - x).hypot(point[1] - y) > radius - 2.
        {
            return Ok(None);
        }
        Ok(Some(point))
    }
}

impl Adapter {
    pub fn event_endless(self, m: &dyn Mem, game: u64) -> Result<bool> {
        match m.read(game + crate::current::EVENT_ENDLESS_OFFSET, 1)?.as_slice() {
            [0] => Ok(false),
            [1] => Ok(true),
            _ => Err("Invalid native endless state".into()),
        }
    }
}

impl Adapter {
    pub unsafe fn event_surface(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        origin: [f32; 3],
        goal: [f32; 3],
    ) -> Result<[f32; 3]> {
        if origin.iter().chain(goal.iter()).any(|v| !v.is_finite()) {
            return Err("Invalid scene surface destination".into());
        }
        let terrain = q(m, game + crate::current::EVENT_TERRAIN_OFFSET)?;
        if terrain < 0x10000 {
            return Err("Missing event terrain".into());
        }
        if q(m, terrain + 0x40)? == u64::MAX {
            return Err("Missing native navigation grid".into());
        }
        type Clip = unsafe extern "system" fn(*mut f32, u64, *const f32, *const f32);
        let call: Clip = std::mem::transmute((base + v::GOBLIN_CLIP.0) as usize);
        let mut result = [0f32; 3];
        call(
            result.as_mut_ptr(),
            terrain + 0x40,
            origin.as_ptr(),
            goal.as_ptr(),
        );
        if result.iter().any(|v| !v.is_finite()) {
            return Err("Invalid projected scene destination".into());
        }
        Ok(result)
    }
}

/// Owned presentation group. Its units never enter native enemy targeting arrays.
pub struct RitualScene {
    objects: Vec<EventObject>,
}
impl RitualScene {
    pub fn anchor(&self) -> u64 {
        self.objects[0].guid
    }
    pub fn len(&self) -> usize {
        self.objects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}
impl Adapter {
    pub unsafe fn event_ritual_scene(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        point: [f32; 3],
    ) -> Result<RitualScene> {
        let mut scene = RitualScene { objects: vec![] };
        let result = (|| -> Result<()> {
            scene
                .objects
                .push(self.event_object(m, base, game, 1, point)?);
            let actor = self
                .event_ground_sample(m, base, game, [point[0], point[1]], 0.1)?
                .ok_or("Invalid ritual center")?
                .position(0.);

            scene.objects.push(self.event_sayge(m, base, game, actor)?);
            for group in 0..3 {
                let angle = group as f32 * std::f32::consts::TAU / 3.;
                for offset in [-0.3f32, 0.3] {
                    let goal = [
                        point[0] + 6. * angle.cos() - offset * angle.sin(),
                        point[1] + 6. * angle.sin() + offset * angle.cos(),
                        point[2],
                    ];
                    let Some(sample) =
                        self.event_ground_sample(m, base, game, [goal[0], goal[1]], 0.3)?
                    else {
                        continue;
                    };
                    let surface = sample.position(0.);
                    // Never accept a clipped candle stacked at the center or through a wall.
                    if (surface[0] - goal[0]).hypot(surface[1] - goal[1]) > 0.5 {
                        continue;
                    }
                    scene
                        .objects
                        .push(self.event_object(m, base, game, 3, surface)?);
                }
            }
            Ok(())
        })();
        if let Err(e) = result {
            let _ = self.remove_ritual_scene(m, base, game, &scene);
            return Err(e);
        }
        Ok(scene)
    }
    pub unsafe fn remove_ritual_scene(
        self,
        m: &dyn Mem,
        base: u64,
        game: u64,
        scene: &RitualScene,
    ) -> Result<()> {
        let mut error = None;
        for object in &scene.objects {
            if let Err(e) = self.remove_event_object(m, base, game, *object) {
                error = Some(e)
            }
        }
        error.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod consumable_aura_tests {
    use super::*;
    use std::cell::RefCell;
    struct Memory(RefCell<Vec<u8>>);
    impl Mem for Memory {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>> {
            let offset = a.checked_sub(0x10000).ok_or("Bad aura address")? as usize;
            self.0
                .borrow()
                .get(offset..offset + n)
                .map(|v| v.to_vec())
                .ok_or("Bad aura extent".into())
        }
    }
    impl MutableMemory for Memory {
        fn write(&self, a: u64, b: &[u8]) -> Result<()> {
            let offset = (a - 0x10000) as usize;
            self.0.borrow_mut()[offset..offset + b.len()].copy_from_slice(b);
            Ok(())
        }
        fn allocate_native(&self, _: &[u8]) -> Result<u64> {
            Err("Unexpected aura allocation".into())
        }
        fn free_native(&self, _: u64, _: usize) -> Result<()> {
            Err("Unexpected aura free".into())
        }
    }
    #[test]
    fn crystal_projection_is_permanent_and_never_displaces_native_or_supply_rows(){
        let m=Memory(RefCell::new(vec![0;512]));let a=Adapter::select(crate::current::SHA256).unwrap();
        let f=super::super::FatigueAura{game:1,slot:0,guid:2,rows:0x10000,clock:5.};
        m.write(0x10000,&[1,0,0,0,44,0,0,0]).unwrap();
        a.project_consumable_auras(&m,&f,&[(asx_gameplay::supplies::Family::Recovery,60.)]).unwrap();
        let before=m.read(0x10000,32).unwrap();
        assert_eq!(a.project_crystal_aura(&m,&f,3).unwrap(),Some(2));
        assert_eq!(m.read(0x10020+8,4).unwrap(),0f32.to_le_bytes());
        assert_eq!(m.read(0x10020+12,4).unwrap(),3u32.to_le_bytes());
        assert_eq!(a.project_crystal_aura(&m,&f,7).unwrap(),Some(2));
        assert_eq!(m.read(0x10000,32).unwrap(),before);
        assert_eq!(a.project_crystal_aura(&m,&f,0).unwrap(),None);
        assert_eq!(m.read(0x10020,16).unwrap(),vec![0;16]);
        for index in 0..32{m.write(0x10000+index*16,&[1,0,0,0,44,0,0,0]).unwrap();}
        let full=m.read(0x10000,512).unwrap();assert_eq!(a.project_crystal_aura(&m,&f,1).unwrap(),None);assert_eq!(m.read(0x10000,512).unwrap(),full);
    }
    #[test]
    fn distinct_families_reuse_own_slots_without_overwriting_native_buffs() {
        use asx_gameplay::supplies::Family;
        let m = Memory(RefCell::new(vec![0; 512]));
        m.write(0x10000, &[1, 0, 0, 0, 0x44, 0, 0, 0]).unwrap();
        let frame = super::super::FatigueAura {
            game: 1,
            slot: 0,
            guid: 2,
            rows: 0x10000,
            clock: 4.,
        };
        let a = Adapter::select(crate::current::SHA256).unwrap();
        let buffs = [(Family::Recovery, 60.), (Family::Critical, 59.)];
        assert!(a.project_consumable_auras(&m,&frame,&[(Family::Critical,60.1)]).is_err());
        let first = a.project_consumable_auras(&m, &frame, &buffs).unwrap();
        assert_eq!(first, vec![(Family::Recovery, 1), (Family::Critical, 2)]);
        let second = a.project_consumable_auras(&m, &frame, &buffs).unwrap();
        assert_eq!(first, second);
        assert_eq!(&m.0.borrow()[..8], &[1, 0, 0, 0, 0x44, 0, 0, 0]);
        let remaining = a.project_consumable_auras(&m, &frame, &buffs[1..]).unwrap();
        assert_eq!(remaining, vec![(Family::Critical, 2)]);
        assert_eq!(&m.0.borrow()[16..32], &[0; 16]);
        assert!(a
            .project_consumable_auras(&m, &frame, &[])
            .unwrap()
            .is_empty());
        assert_eq!(&m.0.borrow()[32..48], &[0; 16]);
        assert_eq!(&m.0.borrow()[..8], &[1, 0, 0, 0, 0x44, 0, 0, 0]);
    }
}

impl Adapter {
    pub fn fixture_event_pose(
        self,
        m: &dyn MutableMemory,
        game: u64,
        slot: u32,
        guid: u64,
        point: [f32; 3],
    ) -> Result<()> {
        if self.run_source(m, game)?.is_none_or(|(s, _)| s != 0)
            || point.iter().any(|v| !v.is_finite())
        {
            return Err("Invalid fixture pose authority".into());
        }
        let row = self.support_row(m, game, slot)?;
        if q(m, row + self.row_field(row == game, 0x478))? != guid {
            return Err("Fixture pose owner changed".into());
        }
        m.write(
            row + self.row_field(row == game, 0x4e0),
            &point
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>(),
        )
    }
}

#[cfg(test)]
mod event_packet_tests {
    use super::*;
    use std::cell::RefCell;
    struct Memory {
        bytes: RefCell<Vec<u8>>,
        freed: RefCell<Vec<(u64, usize)>>,
    }
    impl Mem for Memory {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>> {
            let i = a.checked_sub(0x10000).ok_or("Bad event address")? as usize;
            self.bytes
                .borrow()
                .get(i..i + n)
                .map(|v| v.to_vec())
                .ok_or("Bad event extent".into())
        }
    }
    impl MutableMemory for Memory {
        fn write(&self, a: u64, b: &[u8]) -> Result<()> {
            let i = (a - 0x10000) as usize;
            self.bytes.borrow_mut()[i..i + b.len()].copy_from_slice(b);
            Ok(())
        }
        fn allocate_native(&self, _: &[u8]) -> Result<u64> {
            Err("Unexpected event allocation".into())
        }
        fn free_native(&self, p: u64, n: usize) -> Result<()> {
            self.freed.borrow_mut().push((p, n));
            Ok(())
        }
    }
    fn queue() -> Memory {
        let m = Memory {
            bytes: RefCell::new(vec![0; 0x3000]),
            freed: RefCell::new(vec![]),
        };
        for (at, v) in [
            (0x10670, 4u64),
            (0x10678, 0x11000),
            (0x10680, 2),
            (0x11020, 4),
            (0x11028, 0x12000),
            (0x11030, 4),
            (0x11038, 0xdeadbeef000000a9),
        ] {
            m.write(at, &v.to_le_bytes()).unwrap();
        }
        m.write(0x12000, b"body").unwrap();
        m
    }
    #[test]
    fn created_packet_is_taken_once_without_borrowing_native_queue_across_delivery() {
        let m = queue();
        let a = Adapter::select(crate::current::SHA256).unwrap();
        assert_eq!(
            a.take_created_event_packet(&m, 0x10000, 1).unwrap(),
            b"body"
        );
        assert_eq!(q(&m, 0x10680).unwrap(), 1);
        assert_eq!(*m.freed.borrow(), vec![(0x12000, 4)]);
        assert!(a.take_created_event_packet(&m, 0x10000, 1).is_err());
        assert_eq!(m.freed.borrow().len(), 1);
    }
    #[test]
    fn changed_queue_is_never_truncated_or_freed() {
        let m = queue();
        m.write(0x10680, &3u64.to_le_bytes()).unwrap();
        let a = Adapter::select(crate::current::SHA256).unwrap();
        assert!(a.take_created_event_packet(&m, 0x10000, 1).is_err());
        assert_eq!(q(&m, 0x10680).unwrap(), 3);
        assert!(m.freed.borrow().is_empty());
    }
}

impl Adapter {
    pub fn fixture_event_damage(
        self,
        io: &dyn crate::official::SupportCalls,
        game: u64,
        slot: u32,
        guid: u64,
        target: u64,
    ) -> Result<()> {
        if self.run_source(io, game)?.is_none_or(|(s, _)| s != 0) || slot == 0 {
            return Err("Invalid guest fixture source".into());
        }
        let row = self.support_row(io, game, slot)?;
        if q(io, row + self.row_field(false, 0x478))? != guid {
            return Err("Guest fixture identity changed".into());
        }
        io.swap_actor(game, row)?;
        let result = (|| {
            io.write(
                game + self.game_field(0xb40),
                &u64::from(slot).to_le_bytes(),
            )?;
            io.support_damage(game, target, 1e8)
        })();
        let host = self.support_row(io, game, 0)?;
        io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
        io.swap_actor(game, host)?;
        result
    }
}

impl Adapter {
 /// Invoke the selected native gold route, retaining installed ASX wallet hooks.
 /// Caller must supply the current owned Game context on its native thread.
 pub unsafe fn event_credit_gold(self,base:u64,game:u64,amount:u32)->usize {
  type Gold=unsafe extern "system" fn(u64,u32)->usize;
  let native:Gold=unsafe{std::mem::transmute(base as usize+self.gold_entry().rva as usize)};
  unsafe{native(game,amount)}
 }
}
