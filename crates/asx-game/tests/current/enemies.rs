//! Current native support reads must keep identity, life and motion independent.
use asx_game::{enemies::{parse_enemies_layout, EnemyLayout}, memory::Mem,
    official::SelectedAdapter, official122};

const GAME: u64 = 0x10000;
const RECORDS: u64 = 0x20000;
fn layout() -> EnemyLayout {
    EnemyLayout { stride: 0x1e0, guid_offset: 0xf0, guid_gap: 60, hp_gap: 104 }
}

struct Memory { game: Vec<u8>, records: Vec<u8> }
impl Mem for Memory {
    fn read(&self, address: u64, len: usize) -> asx_game::Result<Vec<u8>> {
        let (base, bytes) = if address >= RECORDS { (RECORDS, &self.records) }
            else { (GAME, &self.game) };
        let offset = address.checked_sub(base).ok_or("invalid address")? as usize;
        bytes.get(offset..offset + len).map(|b| b.to_vec()).ok_or("truncated memory".into())
    }
}

fn memory() -> Memory {
    let mut game = vec![0; 0xd58];
    // Reviewed current Game enemy vector and native clock.
    for (at, value) in [(0x5b0, 2u64), (0x5b8, RECORDS), (0x5c0, 2)] {
        game[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    game[0xc8c..0xc90].copy_from_slice(&11f32.to_le_bytes());
    let mut records = vec![0; 0x3c0];
    records[0xf0..0xf8].copy_from_slice(&123456u64.to_le_bytes());
    records[0x158..0x15c].copy_from_slice(&f32::NAN.to_le_bytes());
    records[0x160..0x164].copy_from_slice(&42f32.to_le_bytes());
    for (at, value) in [(0x17c, 2f32), (0x180, 4.), (0x184, 6.), (0x188, 6.),
                        (0x18c, 8.), (0x190, 10.), (0x194, 10.), (0x198, 2.)] {
        records[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    records[0x1e0 + 0xb0..0x1e0 + 0xb4].copy_from_slice(&1u32.to_le_bytes());
    Memory { game, records }
}

#[test]
fn selected_support_reads_current_health_and_ignores_retired_enemies() {
    let adapter = SelectedAdapter::select(official122::SHA256).unwrap();
    let enemies = adapter.support_enemies(&memory(), GAME).unwrap();
    assert_eq!(enemies.len(), 1);
    assert_eq!((enemies[0].guid, enemies[0].hp), (123456, 42.));
    assert_eq!(enemies[0].position, [4., 6., 8.]);
}

#[test]
fn selected_support_rejects_invalid_extents_and_skips_nonfinite_records() {
    let adapter = SelectedAdapter::select(official122::SHA256).unwrap();
    let mut m = memory();
    m.game[0x5c0..0x5c8].copy_from_slice(&3u64.to_le_bytes());
    assert!(adapter.support_enemies(&m, GAME).is_err());
    let mut m = memory(); m.records.pop();
    assert!(adapter.support_enemies(&m, GAME).is_err());
    let mut m = memory();
    m.records[0x160..0x164].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(adapter.support_enemies(&m, GAME).unwrap().is_empty());
}

#[test]
fn parser_rejects_invalid_layouts_and_partial_current_records() {
    for bad in [EnemyLayout { hp_gap: u64::MAX, ..layout() },
                EnemyLayout { guid_offset: 4, ..layout() },
                EnemyLayout { stride: 0, ..layout() }] {
        assert!(parse_enemies_layout(&[], 0., bad).is_err());
    }
    assert!(parse_enemies_layout(&vec![0; 0x1df], 0., layout()).is_err());
}
