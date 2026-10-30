//! Current native support reads must keep identity, life and motion independent.
use asx_game::{
    current,
    enemies::{parse_enemies_layout, EnemyLayout},
    memory::Mem,
    official::SelectedAdapter,
};

const GAME: u64 = 0x10000;
const RECORDS: u64 = 0x20000;
fn layout() -> EnemyLayout {
    // Independent native125 fixture values, not derived from the descriptor under test.
    EnemyLayout {
        stride: 0x1f8,
        guid_offset: 0xf0,
        guid_gap: 60,
        hp_gap: 128,
    }
}

struct Memory {
    game: Vec<u8>,
    records: Vec<u8>,
}
impl Mem for Memory {
    fn read(&self, address: u64, len: usize) -> asx_game::Result<Vec<u8>> {
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
    let mut game = vec![0; 0xd90];
    // Reviewed current Game enemy vector and 0.130 native clock.
    for (at, value) in [(0x5b0, 2u64), (0x5b8, RECORDS), (0x5c0, 2)] {
        game[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    game[0xce0..0xce4].copy_from_slice(&11f32.to_le_bytes());
    let stride = 0x1f8;
    let mut records = vec![0; stride * 2];
    records[0xf0..0xf0 + 8].copy_from_slice(&123456u64.to_le_bytes());
    records[0x178 - 4..0x178].copy_from_slice(&f32::NAN.to_le_bytes());
    records[0x178..0x178 + 4].copy_from_slice(&42f32.to_le_bytes());
    let motion = 0x178 + 4 + 24;
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
    records[stride + 0xb0..stride + 0xb0 + 4].copy_from_slice(&1u32.to_le_bytes());
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
    m.game[0x5c0..0x5c8].copy_from_slice(&3u64.to_le_bytes());
    assert!(adapter.support_enemies(&m, GAME).is_err());
    let mut m = memory();
    m.records.pop();
    assert!(adapter.support_enemies(&m, GAME).is_err());
    let mut m = memory();
    m.records[0x178..0x178 + 4].copy_from_slice(&f32::NAN.to_le_bytes());
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
    assert!(parse_enemies_layout(&vec![0; 0x1f8 - 1], 0., layout()).is_err());
}

#[test]
fn shared_mark_preserves_other_native_amplifiers_and_enemy_state() {
    use asx_game::official::MutableMemory;
    struct Writable(std::cell::RefCell<Memory>);
    impl Mem for Writable {
        fn read(&self, address: u64, len: usize) -> asx_game::Result<Vec<u8>> {
            self.0.borrow().read(address, len)
        }
    }
    impl MutableMemory for Writable {
        fn write(&self, address: u64, bytes: &[u8]) -> asx_game::Result<()> {
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
        fn allocate_native(&self, _: &[u8]) -> asx_game::Result<u64> {
            Err("unexpected allocation".into())
        }
        fn free_native(&self, _: u64, _: usize) -> asx_game::Result<()> {
            Err("unexpected free".into())
        }
    }
    let mut fixture = memory();
    // Native damage consumes these timed terms separately from Hunter's Mark.
    for (index, value) in [0.15f32, 100., 0.20, 200., 0.25, 300.]
        .into_iter()
        .enumerate()
    {
        let at = 0x120 + index * 4;
        fixture.records[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    let original_game = fixture.game.clone();
    let mut expected_records = fixture.records.clone();
    expected_records[0x1c4..0x1c8].copy_from_slice(&0.3f32.to_le_bytes());
    let writable = Writable(std::cell::RefCell::new(fixture));
    let adapter = SelectedAdapter::select(current::SHA256).unwrap();
    adapter
        .write_combat_mark(&writable, GAME, 123456, 0.3)
        .unwrap();
    assert_eq!(writable.0.borrow().records, expected_records);
    assert_eq!(writable.0.borrow().game, original_game);
    adapter
        .write_combat_mark(&writable, GAME, 123456, 0.)
        .unwrap();
    expected_records[0x1c4..0x1c8].fill(0);
    assert_eq!(writable.0.borrow().records, expected_records);
}
