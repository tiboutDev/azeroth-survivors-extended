//! Exact-build Banished Demon contacts. Native combat and Banish remain authoritative.
use super::{Adapter, treasure_goblins::{GoblinDefinition, GoblinHandle}, world_events::EventObject};
use crate::{current as v, memory::Mem, official::{Entry, MutableMemory}, Result};

impl Adapter {
    pub fn demon_entries(self) -> Vec<Entry> {
        [v::DEMON_BANISH_READ, v::DEMON_BANISH_CONSUME,
         v::DEMON_BANISH_SERIALIZE, v::DEMON_BANISH_ACTION,
         v::GOBLIN_SPAWN, v::GOBLIN_DEFINITION]
            .into_iter().map(|(rva, bytes)| Entry { rva, bytes }).collect()
    }

    /// Copy one coherent 80-byte native kit. The private event entry has no
    /// ordinary XP, gold, guardian or boss completion rewards.
    pub fn demon_definition(self, m: &dyn Mem, base: u64, variant: usize) -> Result<GoblinDefinition> {
        let (&rva, &entry, &display) = (
            v::DEMON_DEFINITIONS.get(variant).ok_or("Unknown demon kit")?,
            v::DEMON_ENTRIES.get(variant).ok_or("Unknown demon entry")?,
            v::DEMON_DISPLAYS.get(variant).ok_or("Unknown demon display")?,
        );
        let mut row: [u8; 80] = m.read(base + rva, 80)?
            .try_into().map_err(|_| "Truncated demon definition")?;
        if u32::from_le_bytes(row[32..36].try_into().unwrap()) != entry
            || u32::from_le_bytes(row[36..40].try_into().unwrap()) != display {
            return Err("Native demon definition identity changed".into());
        }
        let pointer = u64::from_le_bytes(row[16..24].try_into().unwrap());
        let length = u64::from_le_bytes(row[24..32].try_into().unwrap());
        if !(1..=64).contains(&length) || pointer < base
            || pointer.checked_add(length).is_none_or(|end| end > base + 0x8000000) {
            return Err("Invalid native demon name".into());
        }
        for at in [40, 48, 52, 56, 60] {
            let value = f32::from_le_bytes(row[at..at+4].try_into().unwrap());
            if !value.is_finite() || value <= 0. { return Err("Invalid native demon combat kit".into()) }
        }
        row[32..36].copy_from_slice(&(v::DEMON_EVENT_ENTRY + variant as u32).to_le_bytes());
        row[64..].fill(0);
        Ok(GoblinDefinition::event_native(row))
    }

    pub fn demon_banish_count(self, m: &dyn Mem, game: u64) -> Result<u32> {
        Ok(u32::from_le_bytes(m.read(game + v::DEMON_BANISH_OFFSET, 4)?
            .try_into().map_err(|_| "Truncated native Banish count")?))
    }

    /// One shared native charge for the completed encounter. Reserve the
    /// policy receipt before entering; never retry an uncertain native write.
    pub fn demon_grant_banish(self, m: &dyn MutableMemory, game: u64, host: u64) -> Result<Option<u32>> {
        let Some(party) = self.event_party(m, game)? else { return Ok(None) };
        if !party.actors.iter().any(|a| a.slot == 0 && a.guid == host)
            || self.run_source(m, game)? != Some((0, host)) {
            return Ok(None);
        }
        let next = self.demon_banish_count(m, game)?.checked_add(1)
            .ok_or("Native Banish count exhausted")?;
        m.write(game + v::DEMON_BANISH_OFFSET, &next.to_le_bytes())?;
        Ok(Some(next))
    }

    pub unsafe fn demon_query(self, m: &dyn MutableMemory, base: u64, game: u64,
        variant: usize) -> Result<()> {
        let definition = self.demon_definition(m, base, variant)?;
        let bytes = definition.event_bytes();
        let pointer = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let length = u64::from_le_bytes(bytes[24..32].try_into().unwrap()) as usize;
        let name = m.read(pointer, length)?;
        type Query = unsafe extern "system" fn(*mut u64, u32, *const u8, usize,
            *const u8, usize, u32, u32, u32);
        let call: Query = std::mem::transmute((base + v::GOBLIN_QUERY_BUILD.0) as usize);
        let title = b"Banished Demon";
        let mut packet = [0u64; 4];
        call(packet.as_mut_ptr(), v::DEMON_EVENT_ENTRY + variant as u32,
            name.as_ptr(), name.len(), title.as_ptr(), title.len(),
            3, 1, v::DEMON_DISPLAYS[variant]);
        self.send_owned_scene_packet(m, base, game, 0x61, packet)
    }

    pub unsafe fn demon_spawn(self, m: &dyn Mem, base: u64, game: u64,
        point: [f32; 3], variant: usize) -> Result<Option<GoblinHandle>> {
        if variant >= v::DEMON_DEFINITIONS.len() || point.iter().any(|v| !v.is_finite())
            || self.event_party(m, game)?.is_none()
            || self.run_source(m, game)?.is_none_or(|(slot, _)| slot != 0) {
            return Err("Invalid demon spawn context".into());
        }
        let (_, before) = self.goblin_array(m, game)?;
        let mut position = [0u8; 16];
        position[..4].copy_from_slice(&1u32.to_le_bytes());
        for (i, value) in point.into_iter().enumerate() {
            position[4+i*4..8+i*4].copy_from_slice(&value.to_le_bytes());
        }
        type Spawn = unsafe extern "system" fn(u64, u64, *const u8);
        let call: Spawn = std::mem::transmute((base + v::GOBLIN_SPAWN.0) as usize);
        call(game, 0, position.as_ptr());
        let (rows, count) = self.goblin_array(m, game)?;
        for index in before..count {
            let bytes = m.read(rows + index as u64 * v::ENEMY_STRIDE as u64 + v::ENEMY_GUID as u64, 8)?;
            let guid = u64::from_le_bytes(bytes.try_into().map_err(|_| "Truncated demon GUID")?);
            if ((guid >> 24) & 0xffffff) == u64::from(v::DEMON_EVENT_ENTRY + variant as u32) {
                return Ok(Some(GoblinHandle::event_spawn(guid, index)));
            }
        }
        Ok(None)
    }

    pub unsafe fn demon_scene(self, m: &dyn MutableMemory, base: u64, game: u64,
        point: [f32; 3], variant: usize) -> Result<Vec<EventObject>> {
        let definition = self.demon_definition(m, base, variant)?;
        let bytes = definition.event_bytes();
        let pointer = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let length = u64::from_le_bytes(bytes[24..32].try_into().unwrap()) as usize;
        let name = String::from_utf8(m.read(pointer, length)?)
            .map_err(|_| "Invalid demon name")?;
        let scale = f32::from_le_bytes(bytes[40..44].try_into().unwrap());
        let mut objects = Vec::new();
        let result = (|| -> Result<()> {
            let warlock = self.event_ground_sample(m, base, game,
                [point[0] + 3., point[1]], 0.5)?
                .ok_or("No walkable warlock site")?.position(0.);
            objects.push(self.event_object(m, base, game, 1, point)?);
            objects.push(self.event_scene_unit(m, base, game, point,
                v::DEMON_EVENT_ENTRY + 4 + variant as u32,
                v::DEMON_DISPLAYS[variant], scale, &name, "Bound Demon")?);
            objects.push(self.event_scene_unit(m, base, game, warlock,
                v::DEMON_EVENT_ENTRY + 7, 2011, 1.,
                "Council Warlock", "Break the binding")?);
            Ok(())
        })();
        if let Err(error) = result {
            for object in objects { let _ = self.remove_event_object(m, base, game, object); }
            return Err(error);
        }
        Ok(objects)
    }
}
