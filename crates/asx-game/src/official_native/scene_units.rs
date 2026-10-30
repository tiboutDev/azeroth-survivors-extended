//! Native visual units. Never inserts a row into the game's combat simulation.
use super::world_events::EventObject;
use super::Adapter;
use crate::current as v;
use crate::{
    memory::Mem,
    official::{Entry, MutableMemory},
    Result,
};
const SAYGE_ENTRY: u32 = 0xa50010;
type SetField = unsafe extern "system" fn(*mut u64, u16, u32);
type DropFields = unsafe extern "system" fn(*mut u64);
type Create =
    unsafe extern "system" fn(*mut u64, u64, u8, u8, *const f32, f32, *const f32, *const u64);
type Pack = unsafe extern "system" fn(*mut u64, *mut u64);
/// Shared boundary also exercised by Windows ABI stand-ins.
unsafe fn unit_block(
    create: Create,
    out: *mut u64,
    guid: u64,
    point: [f32; 3],
    fields: *const u64,
) {
    let speeds = [0.; 6];
    create(out, guid, 3, 0, point.as_ptr(), 0., speeds.as_ptr(), fields);
}
fn sayge_fields(guid: u64) -> Vec<(u16, u32)> {
    vec![
        (0, guid as u32),
        (1, (guid >> 32) as u32),
        (2, 9),
        (3, SAYGE_ENTRY),
        (4, 1.0f32.to_bits()),
        (22, 1),
        (28, 1),
        (34, 55),
        (35, 35),
        (36, 0x100),
        // Friendly faction, non-attackable and non-selectable. No combat target or NPC gossip.
        (46, 0x02000002),
        (129, 0.3f32.to_bits()),
        (130, 0),
        (131, 491),
        (132, 491),
        (143, 0),
        (145, 1.0f32.to_bits()),
        (147, 0),
        (148, 0),
    ]
}
impl Adapter {
    pub fn ritual_scene_entries(self) -> Vec<Entry> {
        [
            v::EVENT_UNIT_FIELDS_SET,
            v::EVENT_UNIT_FIELDS_DROP,
            v::EVENT_UNIT_CREATE,
            v::EVENT_UPDATE_PACKET,
            v::GOBLIN_QUERY_BUILD,
            v::GOBLIN_CLIP,
        ]
        .into_iter()
        .map(|(rva, bytes)| Entry { rva, bytes })
        .collect()
    }
    pub fn event_enemy_count(self, m: &dyn Mem, game: u64) -> Result<u64> {
        let b = m.read(game + 0x5c0, 8)?;
        let n = u64::from_le_bytes(b.try_into().map_err(|_| "Truncated native enemy count")?);
        if n > 8192 {
            return Err("Invalid native enemy count".into());
        }
        Ok(n)
    }
    pub unsafe fn event_sayge(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        position: [f32; 3],
    ) -> Result<EventObject> {
        if self.event_party(m, game)?.is_none() || position.iter().any(|v| !v.is_finite()) {
            return Err("Visual unit requires a valid authoritative run".into());
        }
        let counter = game + crate::current::EVENT_SCENE_COUNTER_OFFSET;
        let next = u32::from_le_bytes(
            m.read(counter, 4)?
                .try_into()
                .map_err(|_| "Truncated scene GUID counter")?,
        )
        .checked_add(1)
        .filter(|v| *v <= 0xffffff)
        .ok_or("Scene GUID counter exhausted")?;
        // Reserve before any callback-capable serialization or delivery.
        m.write(counter, &next.to_le_bytes())?;
        let guid = 0xf130000000000000 | (u64::from(SAYGE_ENTRY) << 24) | u64::from(next);
        type Query = unsafe extern "system" fn(
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
        let query: Query = std::mem::transmute((base + v::GOBLIN_QUERY_BUILD.0) as usize);
        let mut packet = [0u64; 4];
        let name = b"Sayge";
        let title = b"Darkmoon Faire";
        query(
            packet.as_mut_ptr(),
            SAYGE_ENTRY,
            name.as_ptr(),
            name.len(),
            title.as_ptr(),
            title.len(),
            7,
            0,
            491,
        );
        self.send_owned_scene_packet(m, base, game, 0x61, packet)?;
        let set: SetField = std::mem::transmute((base + v::EVENT_UNIT_FIELDS_SET.0) as usize);
        let drop_fields: DropFields =
            std::mem::transmute((base + v::EVENT_UNIT_FIELDS_DROP.0) as usize);
        let create: Create = std::mem::transmute((base + v::EVENT_UNIT_CREATE.0) as usize);
        let pack: Pack = std::mem::transmute((base + v::EVENT_UPDATE_PACKET.0) as usize);
        let mut fields = [0u64; 3];
        for (index, value) in sayge_fields(guid) {
            set(fields.as_mut_ptr(), index, value);
        }
        let mut block = [0, 1, 0, 0];
        unit_block(create, block.as_mut_ptr(), guid, position, fields.as_ptr());
        // The packet builder consumes/frees the block allocation; the field map stays borrowed.
        let mut packet = [0u64; 4];
        pack(packet.as_mut_ptr(), block.as_mut_ptr());
        drop_fields(fields.as_mut_ptr());
        let object = EventObject { guid, position };
        if let Err(e) = self.send_owned_scene_packet(m, base, game, 0xa9, packet) {
            let _ = self.remove_event_object(m, base, game, object);
            return Err(e);
        }
        Ok(object)
    }
    unsafe fn send_owned_scene_packet(
        self,
        m: &dyn MutableMemory,
        base: u64,
        game: u64,
        opcode: u16,
        packet: [u64; 4],
    ) -> Result<()> {
        if packet[0] < packet[2] || packet[2] > 4096 || packet[0] > 1048576 || packet[1] < 0x10000 {
            return Err("Invalid native scene packet".into());
        }
        let sent = m
            .read(packet[1], packet[2] as usize)
            .and_then(|body| self.event_broadcast(m, base, game, opcode, &body));
        m.free_native(packet[1], packet[0] as usize)?;
        sent
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decorative_fields_exclude_interaction_and_combat_ownership() {
        let f = sayge_fields(0xf130a50010000001);
        let value = |key| f.iter().find(|(k, _)| *k == key).map(|(_, v)| *v);
        assert_eq!(value(35), Some(35));
        assert_eq!(value(46), Some(0x02000002));
        assert_eq!(value(131), Some(491));
        assert_eq!(value(147), Some(0));
        assert!(value(16).is_none());
        assert!(f.windows(2).all(|w| w[0].0 < w[1].0));
    }
    unsafe extern "system" fn create(
        out: *mut u64,
        guid: u64,
        kind: u8,
        flags: u8,
        position: *const f32,
        facing: f32,
        speeds: *const f32,
        fields: *const u64,
    ) {
        assert_eq!(guid, 77);
        assert_eq!((kind, flags), (3, 0));
        assert_eq!(std::slice::from_raw_parts(position, 3), [1., 2., 3.]);
        assert_eq!(facing, 0.);
        assert_eq!(std::slice::from_raw_parts(speeds, 6), [0.; 6]);
        assert_eq!(*fields, 99);
        *out = 123;
    }
    #[test]
    fn unit_create_abi_preserves_all_eight_arguments() {
        let mut out = [0u64; 4];
        unsafe {
            unit_block(
                create,
                out.as_mut_ptr(),
                77,
                [1., 2., 3.],
                [99u64; 3].as_ptr(),
            )
        };
        assert_eq!(out[0], 123);
    }
}
