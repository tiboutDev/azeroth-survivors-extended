//! Selected-client cosmetic projection, with gear/Look ownership preserved.
use super::*;
use azsx_game::official::{piece_look, WORN_NATIVE};

const PROFILE: u64 = 0x100000;
const VISIBLE: u64 = 0x110000;
const LEARNED: u64 = 0x120000;
const LOOKS: u64 = PROFILE + 0x68;
const VISUAL: u32 = 0x6a5a_0000;

fn fixture(class: u8, tree: usize, tier: u32) -> Memory {
    let m = memory();
    for (address, size) in [(PROFILE, 0x300), (VISIBLE, 0x100), (LEARNED, 0x20)] {
        m.data.borrow_mut().insert(address, vec![0; size]);
    }
    // Independently reviewed saved-profile layout for the selected client.
    m.write(PROFILE + 0x1bc, &[class]).unwrap();
    let class_index = match class {
        1 => 0,
        2 => 1,
        7 => 2,
        _ => unreachable!(),
    };
    let definition = azsx_game::current::WEAPON_TALENT_TREES[class_index][tree].0;
    m.write(LEARNED, &definition.to_le_bytes()).unwrap();
    m.write(LEARNED + 8, &[5]).unwrap();
    m.write(
        PROFILE + 0x18,
        &[1u64, LEARNED, 1].map(u64::to_le_bytes).concat(),
    )
    .unwrap();
    let main = 100800 + u32::from(class) * 1000 + tier * 10;
    let off = if class == 7 { main + 100 } else { 0 };
    m.write(PROFILE + 0x1a4, &main.to_le_bytes()).unwrap();
    m.write(VISIBLE + 60, &[main, off].map(u32::to_le_bytes).concat())
        .unwrap();
    m
}

fn expected(displays: [u32; 2]) -> [u32; 2] {
    displays.map(|display| if display == 0 { 0 } else { VISUAL + display })
}

#[test]
fn all_five_new_set_variants_and_lawbringer_project_without_mutating_equipment() {
    // Display identities are pinned to the Classic item references, not derived
    // from the production tables under test.
    for (class, tree, tier, displays) in [
        (1, 1, 7, [24756, 26494]), // Heroism Fury: Silent Fang / Mirah's Song.
        (1, 2, 7, [25036, 23825]), // Heroism Protection.
        (2, 0, 7, [6798, 25133]),  // Soulforge Holy.
        (2, 1, 7, [23836, 25133]), // Soulforge Protection.
        (7, 1, 7, [32782, 0]),     // The Five Thunders Enhancement.
        (2, 1, 2, [29171, 34110]), // Lawbringer: Empyrean Demolisher / Drillborer.
        (2, 1, 1, [23618, 23419]), // Lightforge retains Ironfoe.
        (7, 1, 4, [29698, 0]),     // Earthshatter retains Sulfuras.
    ] {
        let m = fixture(class, tree, tier);
        let before = m.read(PROFILE, 0x300).unwrap();
        assert_eq!(
            adapter()
                .weapon_projection(&m, 0, PROFILE, VISIBLE)
                .unwrap(),
            Some(expected(displays))
        );
        assert_eq!(m.read(PROFILE, 0x300).unwrap(), before);
        assert_eq!(
            m.read(VISIBLE + 60, 8).unwrap(),
            expected(displays).map(u32::to_le_bytes).concat()
        );
    }
}

#[test]
fn selected_new_set_look_adapts_independently_of_the_worn_set() {
    for (class, tree, displays) in [
        (1, 1, [24756, 26494]),
        (2, 0, [6798, 25133]),
        (7, 1, [32782, 0]),
    ] {
        let m = fixture(class, tree, 1);
        let selected = 100870 + u32::from(class) * 1000;
        let choice = piece_look(class, 15, selected).unwrap();
        m.write(LOOKS + 15 * 8, &[1, choice].map(u32::to_le_bytes).concat())
            .unwrap();
        m.write(VISIBLE + 60, &selected.to_le_bytes()).unwrap();
        let before = m.read(PROFILE, 0x300).unwrap();
        assert_eq!(
            adapter()
                .weapon_projection(&m, 0, PROFILE, VISIBLE)
                .unwrap(),
            Some(expected(displays))
        );
        assert_eq!(m.read(PROFILE, 0x300).unwrap(), before);
    }
}

#[test]
fn native_look_precedence_unknown_sets_and_empty_builds_stay_native() {
    let m = fixture(7, 1, 7);
    // An explicit native off-hand prevents replacing the complete pair by a 2H.
    m.write(
        LOOKS + 16 * 8,
        &[1, WORN_NATIVE].map(u32::to_le_bytes).concat(),
    )
    .unwrap();
    let before = m.read(VISIBLE, 0x100).unwrap();
    assert_eq!(
        adapter()
            .weapon_projection(&m, 0, PROFILE, VISIBLE)
            .unwrap(),
        None
    );
    assert_eq!(m.read(VISIBLE, 0x100).unwrap(), before);
    for (tree, tier, empty) in [(0, 7, false), (1, 8, false), (1, 7, true)] {
        let m = fixture(1, tree, tier);
        if empty {
            m.write(PROFILE + 0x28, &0u64.to_le_bytes()).unwrap();
        }
        let before = m.read(VISIBLE, 0x100).unwrap();
        assert_eq!(
            adapter()
                .weapon_projection(&m, 0, PROFILE, VISIBLE)
                .unwrap(),
            None
        );
        assert_eq!(m.read(VISIBLE, 0x100).unwrap(), before);
    }
}

#[test]
fn new_metadata_changes_only_the_visual_item_fields_and_preserves_grip_rules() {
    for (display, item_class, subclass, inventory, sheath) in [
        (24756, 2, 7, 21, 3),
        (26494, 2, 7, 13, 3),
        (25036, 2, 7, 13, 1),
        (23825, 4, 6, 14, 4),
        (6798, 2, 4, 21, 3),
        (25133, 4, 6, 14, 4),
        (23836, 2, 7, 13, 3),
        (32782, 2, 5, 17, 1),
        (29171, 2, 4, 21, 3),
    ] {
        let m = memory();
        m.data.borrow_mut().insert(VISIBLE, vec![0x55; 128]);
        let entry = VISUAL + display;
        adapter().weapon_metadata(&m, VISIBLE, entry).unwrap();
        assert_eq!(m.read(VISIBLE, 0x50).unwrap(), vec![0x55; 0x50]);
        assert_eq!(
            m.read(VISIBLE + 0x50, 24).unwrap(),
            [entry, display, item_class, subclass, inventory, sheath]
                .map(u32::to_le_bytes)
                .concat()
        );
        assert_eq!(m.read(VISIBLE + 0x68, 24).unwrap(), vec![0x55; 24]);
        assert_eq!(
            adapter().weapon_two_handed(entry),
            if item_class == 2 {
                Some(inventory == 17)
            } else {
                None
            }
        );
    }
}
