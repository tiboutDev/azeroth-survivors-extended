//! Current native support reads must keep identity, life and motion independent.
use azsx_game::{
    current,
    enemies::{parse_enemies_layout, EnemyLayout},
    memory::Mem,
    official::SelectedAdapter,
};

const GAME: u64 = 0x10000;
const RECORDS: u64 = 0x20800;
fn layout() -> EnemyLayout {
    // Independent 0.140.0 fixture values, not derived from the descriptor under test.
    EnemyLayout {
        stride: 0x228,
        guid_offset: 0x110,
        guid_gap: 60,
        hp_gap: 128,
    }
}

struct Memory {
    game: Vec<u8>,
    records: Vec<u8>,
}
impl Mem for Memory {
    fn read(&self, address: u64, len: usize) -> azsx_game::Result<Vec<u8>> {
        let (base, bytes) = if address >= RECORDS {
            (RECORDS, &self.records)
        } else {
            (GAME, &self.game)
        };
        let offset = address.checked_sub(base).ok_or("invalid address")? as usize;
        bytes
            .get(offset..offset + len)
            .map(|b| b.to_vec())
            .ok_or("truncated memory".into())
    }
}

fn memory() -> Memory {
    let mut game = vec![0; 0x1200];
    // Reviewed current Game enemy vector and 0.140.0 native clock.
    for (at, value) in [(0x6b8, 2u64), (0x6c0, RECORDS), (0x6c8, 2)] {
        game[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    game[0x1010..0x1014].copy_from_slice(&11f32.to_le_bytes());
    let stride = 0x228;
    let mut records = vec![0; stride * 2];
    records[0x110..0x110 + 8].copy_from_slice(&123456u64.to_le_bytes());
    records[0x198 - 4..0x198].copy_from_slice(&f32::NAN.to_le_bytes());
    records[0x198..0x198 + 4].copy_from_slice(&42f32.to_le_bytes());
    let motion = 0x198 + 4 + 24;
    for (at, value) in [
        (motion, 2f32),
        (motion + 4, 4.),
        (motion + 8, 6.),
        (motion + 12, 6.),
        (motion + 16, 8.),
        (motion + 20, 10.),
        (motion + 24, 10.),
        (motion + 28, 2.),
    ] {
        records[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    records[stride + 0xd0..stride + 0xd0 + 4].copy_from_slice(&1u32.to_le_bytes());
    Memory { game, records }
}

#[test]
fn selected_support_reads_current_health_and_ignores_retired_enemies() {
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    let enemies = adapter.support_enemies(&memory(), GAME).unwrap();
    assert_eq!(enemies.len(), 1);
    assert_eq!((enemies[0].guid, enemies[0].hp), (123456, 42.));
    assert_eq!(enemies[0].position, [4., 6., 8.]);
}

#[test]
fn selected_support_rejects_invalid_extents_and_skips_nonfinite_records() {
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    let mut m = memory();
    m.game[0x6c8..0x6d0].copy_from_slice(&3u64.to_le_bytes());
    assert!(adapter.support_enemies(&m, GAME).is_err());
    let mut m = memory();
    m.records.pop();
    assert!(adapter.support_enemies(&m, GAME).is_err());
    let mut m = memory();
    m.records[0x198..0x198 + 4].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(adapter.support_enemies(&m, GAME).unwrap().is_empty());
}

#[test]
fn parser_rejects_invalid_layouts_and_partial_current_records() {
    for bad in [
        EnemyLayout {
            hp_gap: u64::MAX,
            ..layout()
        },
        EnemyLayout {
            guid_offset: 4,
            ..layout()
        },
        EnemyLayout {
            stride: 0,
            ..layout()
        },
    ] {
        assert!(parse_enemies_layout(&[], 0., bad).is_err());
    }
    assert!(parse_enemies_layout(&vec![0; 0x228 - 1], 0., layout()).is_err());
}

#[test]
fn shared_mark_preserves_other_native_amplifiers_and_enemy_state() {
    use azsx_game::official::MutableMemory;
    struct Writable(std::cell::RefCell<Memory>);
    impl Mem for Writable {
        fn read(&self, address: u64, len: usize) -> azsx_game::Result<Vec<u8>> {
            self.0.borrow().read(address, len)
        }
    }
    impl MutableMemory for Writable {
        fn write(&self, address: u64, bytes: &[u8]) -> azsx_game::Result<()> {
            let mut memory = self.0.borrow_mut();
            let (base, target) = if address >= RECORDS {
                (RECORDS, &mut memory.records)
            } else {
                (GAME, &mut memory.game)
            };
            let at = address.checked_sub(base).ok_or("invalid write")? as usize;
            target
                .get_mut(at..at + bytes.len())
                .ok_or("write outside fixture")?
                .copy_from_slice(bytes);
            Ok(())
        }
        fn allocate_native(&self, _: &[u8]) -> azsx_game::Result<u64> {
            Err("unexpected allocation".into())
        }
        fn free_native(&self, _: u64, _: usize) -> azsx_game::Result<()> {
            Err("unexpected free".into())
        }
    }
    let mut fixture = memory();
    // Native damage consumes these timed terms separately from Hunter's Mark.
    for (index, value) in [0.15f32, 100., 0.20, 200., 0.25, 300.]
        .into_iter()
        .enumerate()
    {
        let at = 0x140 + index * 4;
        fixture.records[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    // Adjacent native state must never be mistaken for mark strength.
    fixture.records[0x1e4..0x1e8].copy_from_slice(&123.5f32.to_le_bytes());
    let original_game = fixture.game.clone();
    let mut expected_records = fixture.records.clone();
    expected_records[0x1e8..0x1ec].copy_from_slice(&0.3f32.to_le_bytes());
    let writable = Writable(std::cell::RefCell::new(fixture));
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    adapter
        .write_combat_mark(&writable, GAME, 123456, 0.3)
        .unwrap();
    assert_eq!(writable.0.borrow().records, expected_records);
    assert_eq!(writable.0.borrow().game, original_game);
    assert_eq!(adapter.combat_enemies(&writable, GAME).unwrap()[0].vulnerability, 0.3);
    adapter
        .write_combat_mark(&writable, GAME, 123456, 0.)
        .unwrap();
    expected_records[0x1e8..0x1ec].fill(0);
    assert_eq!(writable.0.borrow().records, expected_records);
}

#[test]
fn encounter_fixture_never_kills_ordinary_or_event_creatures() {
    let mut m = memory();
    for at in [0x6b8, 0x6c8] {
        m.game[at..at + 8].copy_from_slice(&4u64.to_le_bytes());
    }
    m.records = vec![0; 4 * 0x228];
    // Native 0.140.0 encounter IDs are at +0x30. The retired +0x18
    // contains unrelated state, including zero on ordinary/event creatures.
    for (index, (guid, encounter, old_word, dead)) in [
        (101u64, u64::MAX, 0u64, false),
        (202, u64::MAX, 1234, false),
        (303, 0, u64::MAX, false),
        (404, 1, 0, true),
    ].into_iter().enumerate() {
        let row = &mut m.records[index * 0x228..(index + 1) * 0x228];
        row[0x110..0x118].copy_from_slice(&guid.to_le_bytes());
        row[0x30..0x38].copy_from_slice(&encounter.to_le_bytes());
        row[0x18..0x20].copy_from_slice(&old_word.to_le_bytes());
        row[0xd0] = u8::from(dead);
    }
    let adapter = azsx_game::official_native::Adapter::select(current::SHA256).unwrap();
    assert_eq!(adapter.blessing_fixture_targets(&m, GAME).unwrap(), [303]);
}

#[test]
fn guardian_fixture_requires_a_live_guardian_and_excludes_bosses_and_events() {
    let mut m = memory();
    for at in [0x6b8, 0x6c8] {
        m.game[at..at + 8].copy_from_slice(&5u64.to_le_bytes());
    }
    m.records = vec![0; 5 * 0x228];
    for (index, (encounter, guardian, old_guardian, dead)) in [
        (u64::MAX, u64::MAX, 2u64, false), // ordinary
        (u64::MAX, u64::MAX, 0, false), // event creature
        (0, u64::MAX, 0, false), // boss
        (u64::MAX, 2, u64::MAX, true), // retired guardian
        (u64::MAX, 3, u64::MAX, false), // eligible guardian
    ].into_iter().enumerate() {
        let row = &mut m.records[index * 0x228..(index + 1) * 0x228];
        row[0x110..0x118].copy_from_slice(&(100 + index as u64).to_le_bytes());
        row[0x30..0x38].copy_from_slice(&encounter.to_le_bytes());
        row[0x78..0x80].copy_from_slice(&guardian.to_le_bytes());
        row[0x58..0x60].copy_from_slice(&old_guardian.to_le_bytes());
        row[0x198..0x19c].copy_from_slice(&50f32.to_le_bytes());
        row[0xd0] = u8::from(dead);
    }
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    assert_eq!(adapter.fixture_guardian_target(&m, GAME).unwrap(), Some((104, 50.)));
}
