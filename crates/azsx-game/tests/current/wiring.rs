//! Expected native 0.140.0 field transactions, independent of descriptor offsets.
use azsx_game::{current, memory::Mem, official::{MutableMemory, SelectedAdapter}};
use std::cell::RefCell;

struct Memory(RefCell<Vec<u8>>);
impl Memory {
    fn new() -> Self { Self(RefCell::new(vec![0xa5; 0x40000])) }
    fn put(&self, at: u64, bytes: &[u8]) { self.write(at, bytes).unwrap(); }
    fn u32(&self, at: u64) -> u32 { u32::from_le_bytes(self.read(at, 4).unwrap().try_into().unwrap()) }
}
impl Mem for Memory {
    fn read(&self, at: u64, size: usize) -> azsx_game::Result<Vec<u8>> {
        self.0.borrow().get(at as usize..at as usize + size).map(<[u8]>::to_vec).ok_or("Outside fixture".into())
    }
}
impl MutableMemory for Memory {
    fn write(&self, at: u64, bytes: &[u8]) -> azsx_game::Result<()> {
        self.0.borrow_mut().get_mut(at as usize..at as usize + bytes.len()).ok_or("Outside fixture")?.copy_from_slice(bytes);
        Ok(())
    }
    fn allocate_native(&self, _: &[u8]) -> azsx_game::Result<u64> { Err("Unexpected allocation".into()) }
    fn free_native(&self, _: u64, _: usize) -> azsx_game::Result<()> { Err("Unexpected free".into()) }
}
fn adapter() -> SelectedAdapter { SelectedAdapter::select(current::SHA256).unwrap() }

#[test]
fn shared_chest_metadata_and_pending_rewards_use_distinct_host_and_ally_fields() {
    let m = Memory::new(); let a = adapter(); let (game, ally) = (0x10000, 0x20000);
    for row in [game, ally] { m.put(row, &0u32.to_le_bytes()); }
    m.put(game + 0x10d1, &[1]); m.put(ally + 0xc00, &[1]);
    m.put(game + 0xcd0, &ally.to_le_bytes()); m.put(game + 0xcd8, &1u64.to_le_bytes());
    m.put(game + 0xfc0, &0u64.to_le_bytes());
    for (at, value) in [(game + 0x1028, 2u32), (game + 0x1070, 40), (game + 0xa9c, 3),
                        (ally + 0xba8, 5), (ally + 0xbf0, 50), (ally + 0x944, 7)] { m.put(at, &value.to_le_bytes()); }
    assert_eq!(a.chest_state(&m, game).unwrap(), (0, 40, 3));
    let mut expected = m.0.borrow().clone();
    assert!(a.deliver_chests(&m, game, 0, 2, 60, 4).unwrap());
    assert!(a.deliver_chests(&m, game, 1, 1, 60, 4).unwrap());
    for (at, value) in [(game + 0x1028, 4u32), (game + 0x1070, 60), (game + 0xa9c, 4),
                        (ally + 0xba8, 6), (ally + 0xbf0, 60), (ally + 0x944, 4)] {
        expected[at as usize..at as usize + 4].copy_from_slice(&value.to_le_bytes());
    }
    assert_eq!(*m.0.borrow(), expected);
    assert_eq!(a.chest_state(&m, game).unwrap(), (0, 60, 4));
    // A pending/paused native choice can receive another earned item; an
    // inactive actor cannot have its bank or shared metadata changed.
    m.put(ally + 0xc00, &[2]);
    assert!(a.deliver_chests(&m, game, 1, 1, 60, 4).unwrap());
    assert_eq!(m.u32(ally + 0xba8), 7);
    m.put(ally + 0xc00, &[0]); let before = m.0.borrow().clone();
    assert!(!a.deliver_chests(&m, game, 1, 1, 999, 999).unwrap());
    assert_eq!(*m.0.borrow(), before);
}

#[test]
fn effect_xp_rewrites_only_its_current_owned_native_receipt() {
    let m = Memory::new(); let a = adapter(); let (game, records) = (0x10000, 0x30000);
    m.put(game + 0x10cb, &[1]); m.put(game + 0x5dc, &10u32.to_le_bytes());
    m.put(game + 0xfc0, &1u64.to_le_bytes());
    for (off, value) in [(0xce0, 2u64), (0xce8, records), (0xcf0, 2)] { m.put(game + off, &value.to_le_bytes()); }
    m.put(records + 16, &1u64.to_le_bytes()); m.put(records + 24, &37u32.to_le_bytes());
    let mut expected = m.0.borrow().clone();
    // Exact native effect producer return and register ownership are checked
    // independently against the pinned executable.
    let award = a.normalize_xp(&m, game, 0x5fd3092, [0, 0, 999, 37], 1.5, None).unwrap().unwrap();
    assert_eq!((award.source, award.raw, award.amount), ("effect", 37, 56));
    expected[(records + 24) as usize..(records + 28) as usize].copy_from_slice(&56u32.to_le_bytes());
    assert_eq!(*m.0.borrow(), expected);
    m.put(records + 16, &2u64.to_le_bytes()); let before = m.0.borrow().clone();
    assert!(a.normalize_xp(&m, game, 0x5fd3092, [0, 0, 999, 37], 1.5, None).is_err());
    assert_eq!(*m.0.borrow(), before);
    m.put(game + 0xcf0, &3u64.to_le_bytes());
    assert!(a.normalize_xp(&m, game, 0x5fd3092, [0, 0, 999, 37], 1.5, None).is_err());
}

#[test]
fn native_ui_sink_uses_current_server_storage_and_keeps_the_borrowed_receiver() {
    let m = Memory::new(); let a = adapter(); let (server, context, vtable) = (0x10000, 0x20000u64, 0x30000u64);
    m.put(server + 0x1f60, &context.to_le_bytes());
    m.put(server + 0x1f68, &vtable.to_le_bytes());
    m.put(vtable + 24, &0x123456u64.to_le_bytes());
    let before = m.0.borrow().clone();
    assert_eq!(a.loot_ui_sink(&m, server).unwrap(), (context, 0x123456));
    assert_eq!(*m.0.borrow(), before);
}

#[test]
fn melee_fixture_excludes_current_boss_guardian_and_special_action_instances() {
    let m = Memory::new(); let a = adapter(); let (game, rows, states): (u64, u64, u64) = (0x10000, 0x20000, 0x30000);
    for (at, value) in [(0x6b8, 4u64), (0x6c0, rows), (0x6c8, 4)] { m.put(game + at, &value.to_le_bytes()); }
    m.put(rows, &[0; 4 * 552]);
    for (index, (encounter, guardian, state)) in [
        (0, u64::MAX, 0), (u64::MAX, 2, 0), (u64::MAX, u64::MAX, states), (u64::MAX, u64::MAX, states + 16),
    ].into_iter().enumerate() {
        let row = rows + index as u64 * 552;
        m.put(row + 0x110, &(1001 + index as u64).to_le_bytes());
        m.put(row + 0x30, &encounter.to_le_bytes()); m.put(row + 0x78, &guardian.to_le_bytes());
        m.put(row + 0x120, &state.to_le_bytes());
        m.put(row + 0x198, &100f32.to_le_bytes()); m.put(row + 0x19c, &100f32.to_le_bytes());
        // Retired fields deliberately describe a conflicting classification.
        let old = if index < 2 { u64::MAX } else { 0 };
        m.put(row + 0x18, &old.to_le_bytes()); m.put(row + 0x58, &old.to_le_bytes());
    }
    m.put(states, &[1]); m.put(states + 16, &[0]);
    let mut expected = m.0.borrow().clone();
    let row = rows + 3 * 552;
    for at in [row + 0x198, row + 0x19c] {
        expected[at as usize..at as usize + 4].copy_from_slice(&1e6f32.to_le_bytes());
    }
    assert_eq!(a.fixture_combat_enemy(&m, game, 5).unwrap(), Some(1004));
    assert_eq!(*m.0.borrow(), expected);
}
