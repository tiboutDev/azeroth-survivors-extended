//! Cosmetic models for the native 0.138.0 generated set weapons.
use super::Adapter;
use crate::{
    memory::Mem,
    official::{Entry, MutableMemory},
    Result,
};

#[derive(Clone, Copy)]
struct Model {
    display: u32,
    class: u32,
    subclass: u32,
    inventory: u32,
    sheath: u32,
}
const AURASTONE_HAMMER: Model = Model {
    display: 34109,
    class: 2,
    subclass: 4,
    inventory: 13,
    sheath: 3,
};
const BRUTALITY_BLADE: Model = Model {
    display: 31309,
    class: 2,
    subclass: 7,
    inventory: 13,
    sheath: 3,
};
const CHROMATICALLY_TEMPERED_SWORD: Model = Model {
    display: 31867,
    class: 2,
    subclass: 7,
    inventory: 13,
    sheath: 3,
};
const CORPSEMAKER: Model = Model {
    display: 22217,
    class: 2,
    subclass: 1,
    inventory: 17,
    sheath: 1,
};
const DALRENDS_SACRED_CHARGE: Model = Model {
    display: 25648,
    class: 2,
    subclass: 7,
    inventory: 21,
    sheath: 3,
};
const DALRENDS_TRIBAL_GUARDIAN: Model = Model {
    display: 25647,
    class: 2,
    subclass: 7,
    inventory: 22,
    sheath: 3,
};
const DEATHBRINGER: Model = Model {
    display: 29161,
    class: 2,
    subclass: 0,
    inventory: 13,
    sheath: 3,
};
const DRACONIAN_DEFLECTOR: Model = Model {
    display: 23419,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const DRAKE_TALON_CLEAVER: Model = Model {
    display: 31857,
    class: 2,
    subclass: 1,
    inventory: 17,
    sheath: 1,
};
const DRILLBORER_DISK: Model = Model {
    display: 34110,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const EARTHSHAKER: Model = Model {
    display: 32162,
    class: 2,
    subclass: 5,
    inventory: 17,
    sheath: 1,
};
const ELEMENTIUM_REINFORCED_BULWARK: Model = Model {
    display: 31852,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const GRAND_MARSHALS_AEGIS: Model = Model {
    display: 31733,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const GRAND_MARSHALS_LONGSWORD: Model = Model {
    display: 31966,
    class: 2,
    subclass: 7,
    inventory: 13,
    sheath: 3,
};
const GRAND_MARSHALS_PUNISHER: Model = Model {
    display: 31955,
    class: 2,
    subclass: 4,
    inventory: 13,
    sheath: 3,
};
const GRAND_MARSHALS_WARHAMMER: Model = Model {
    display: 36064,
    class: 2,
    subclass: 4,
    inventory: 13,
    sheath: 3,
};
const HAMMER_OF_THE_TWISTING_NETHER: Model = Model {
    display: 35574,
    class: 2,
    subclass: 4,
    inventory: 13,
    sheath: 3,
};
const HATCHET_OF_SUNDERED_BONE: Model = Model {
    display: 35258,
    class: 2,
    subclass: 0,
    inventory: 13,
    sheath: 3,
};
const HIGH_WARLORDS_BATTLE_AXE: Model = Model {
    display: 31958,
    class: 2,
    subclass: 1,
    inventory: 17,
    sheath: 1,
};
const HIGH_WARLORDS_BLADE: Model = Model {
    display: 31997,
    class: 2,
    subclass: 7,
    inventory: 13,
    sheath: 3,
};
const HIGH_WARLORDS_CLEAVER: Model = Model {
    display: 31957,
    class: 2,
    subclass: 0,
    inventory: 13,
    sheath: 3,
};
const HIGH_WARLORDS_SHIELD_WALL: Model = Model {
    display: 31746,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const IRONFOE: Model = Model {
    display: 23618,
    class: 2,
    subclass: 4,
    inventory: 21,
    sheath: 3,
};
const LARGE_CLUB: Model = Model {
    display: 19601,
    class: 2,
    subclass: 5,
    inventory: 17,
    sheath: 1,
};
const LOKAMIR_IL_ROMATHIS: Model = Model {
    display: 31863,
    class: 2,
    subclass: 4,
    inventory: 21,
    sheath: 3,
};
const MACE: Model = Model {
    display: 8287,
    class: 2,
    subclass: 4,
    inventory: 13,
    sheath: 3,
};
const MALADATH_RUNED_BLADE_OF_THE_BLACK_FLIGHT: Model = Model {
    display: 31866,
    class: 2,
    subclass: 7,
    inventory: 13,
    sheath: 3,
};
const MALISTARS_DEFENDER: Model = Model {
    display: 29702,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const QUELSERRAR: Model = Model {
    display: 30994,
    class: 2,
    subclass: 7,
    inventory: 21,
    sheath: 3,
};
const RED_DRAGONSCALE_PROTECTOR: Model = Model {
    display: 31851,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const SACRED_PROTECTOR: Model = Model {
    display: 28829,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const SHIELD_OF_CONDEMNATION: Model = Model {
    display: 35573,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const SHORTSWORD: Model = Model {
    display: 22075,
    class: 2,
    subclass: 7,
    inventory: 13,
    sheath: 3,
};
const SMALL_SHIELD: Model = Model {
    display: 18480,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const SPINESHATTER: Model = Model {
    display: 31862,
    class: 2,
    subclass: 4,
    inventory: 21,
    sheath: 3,
};
const SULFURAS_HAND_OF_RAGNAROS: Model = Model {
    display: 29698,
    class: 2,
    subclass: 5,
    inventory: 17,
    sheath: 1,
};
const SWORD_OF_ZEAL: Model = Model {
    display: 21554,
    class: 2,
    subclass: 7,
    inventory: 21,
    sheath: 3,
};
const THE_CASTIGATOR: Model = Model {
    display: 35250,
    class: 2,
    subclass: 4,
    inventory: 13,
    sheath: 3,
};
const THE_FACE_OF_DEATH: Model = Model {
    display: 35576,
    class: 4,
    subclass: 6,
    inventory: 14,
    sheath: 4,
};
const THE_HAMMER_OF_GRACE: Model = Model {
    display: 21956,
    class: 2,
    subclass: 4,
    inventory: 21,
    sheath: 3,
};
const THE_HUNGERING_COLD: Model = Model {
    display: 36377,
    class: 2,
    subclass: 7,
    inventory: 13,
    sheath: 3,
};

type Hands = (Option<Model>, Option<Model>);
const FURY: &[Hands] = &[
    (Some(SHORTSWORD), Some(SHORTSWORD)),
    (Some(DALRENDS_SACRED_CHARGE), Some(DALRENDS_TRIBAL_GUARDIAN)),
    (Some(DEATHBRINGER), Some(BRUTALITY_BLADE)),
    (
        Some(CHROMATICALLY_TEMPERED_SWORD),
        Some(MALADATH_RUNED_BLADE_OF_THE_BLACK_FLIGHT),
    ),
    (Some(HATCHET_OF_SUNDERED_BONE), Some(THE_CASTIGATOR)),
    (
        Some(GRAND_MARSHALS_LONGSWORD),
        Some(GRAND_MARSHALS_LONGSWORD),
    ),
    (Some(HIGH_WARLORDS_CLEAVER), Some(HIGH_WARLORDS_CLEAVER)),
];
const WARRIOR_PROTECTION: &[Hands] = &[
    (Some(SHORTSWORD), Some(SMALL_SHIELD)),
    (Some(SWORD_OF_ZEAL), Some(DRACONIAN_DEFLECTOR)),
    (Some(QUELSERRAR), Some(DRILLBORER_DISK)),
    (
        Some(MALADATH_RUNED_BLADE_OF_THE_BLACK_FLIGHT),
        Some(ELEMENTIUM_REINFORCED_BULWARK),
    ),
    (Some(THE_HUNGERING_COLD), Some(THE_FACE_OF_DEATH)),
    (Some(GRAND_MARSHALS_LONGSWORD), Some(GRAND_MARSHALS_AEGIS)),
    (Some(HIGH_WARLORDS_BLADE), Some(HIGH_WARLORDS_SHIELD_WALL)),
];
const HOLY: &[Hands] = &[
    (Some(MACE), Some(SMALL_SHIELD)),
    (Some(THE_HAMMER_OF_GRACE), Some(SACRED_PROTECTOR)),
    (Some(AURASTONE_HAMMER), Some(MALISTARS_DEFENDER)),
    (Some(LOKAMIR_IL_ROMATHIS), Some(RED_DRAGONSCALE_PROTECTOR)),
    (
        Some(HAMMER_OF_THE_TWISTING_NETHER),
        Some(SHIELD_OF_CONDEMNATION),
    ),
    (Some(GRAND_MARSHALS_WARHAMMER), Some(GRAND_MARSHALS_AEGIS)),
];
const PALADIN_PROTECTION: &[Hands] = &[
    (Some(MACE), Some(SMALL_SHIELD)),
    (Some(IRONFOE), Some(DRACONIAN_DEFLECTOR)),
    (Some(IRONFOE), Some(DRILLBORER_DISK)),
    (Some(SPINESHATTER), Some(ELEMENTIUM_REINFORCED_BULWARK)),
    (Some(THE_CASTIGATOR), Some(THE_FACE_OF_DEATH)),
    (Some(GRAND_MARSHALS_PUNISHER), Some(GRAND_MARSHALS_AEGIS)),
];
const ENHANCEMENT: &[Hands] = &[
    (Some(LARGE_CLUB), None),
    (Some(CORPSEMAKER), None),
    (Some(EARTHSHAKER), None),
    (Some(DRAKE_TALON_CLEAVER), None),
    (Some(SULFURAS_HAND_OF_RAGNAROS), None),
    (None, None),
    (Some(HIGH_WARLORDS_BATTLE_AXE), None),
];

// These entries exist only in visible-item fields. The authoritative equipped
// items, their ranks, inventory objects and the player's Look choices stay native.
pub const WORN_NATIVE: u32 = 0x6a59_0001;
const VISUAL_BASE: u32 = 0x6a5a_0000;
fn model(entry: u32) -> Option<Model> {
    let display = entry.checked_sub(VISUAL_BASE)?;
    if display > 65535 {
        return None;
    }
    [
        FURY,
        WARRIOR_PROTECTION,
        HOLY,
        PALADIN_PROTECTION,
        ENHANCEMENT,
    ]
    .into_iter()
    .flatten()
    .flat_map(|&(r, l)| [r, l])
    .flatten()
    .find(|m| m.display == display)
}
fn table(class: u8, totals: [u16; 3]) -> Option<&'static [Hands]> {
    let high = *totals.iter().max()?;
    if high == 0 || totals.iter().filter(|&&v| v == high).count() != 1 {
        return None;
    }
    match (class, totals.iter().position(|&v| v == high)?) {
        (1, 1) => Some(FURY),
        (1, 2) => Some(WARRIOR_PROTECTION),
        (2, 0) => Some(HOLY),
        (2, 1) => Some(PALADIN_PROTECTION),
        (7, 1) => Some(ENHANCEMENT),
        _ => None,
    }
}
fn set_tier(entry: u32, class: u8, index: u32) -> Option<usize> {
    let raw = entry.checked_sub(100000 + u32::from(class) * 1000 + index * 100)?;
    if raw > 60 || raw % 10 != 0 {
        return None;
    }
    let tier = (raw / 10) as usize;
    Some(match (class, tier) {
        (2, 6) => 5,
        (7, 5) => 6,
        _ => tier,
    })
}
// Save a selected specialization look separately from the native item ID.
const PIECE_FLAG: u32 = 0x4000_0000;
pub fn piece_look(class: u8, slot: usize, item: u32) -> Option<u32> {
    if !matches!(class, 1 | 2 | 7) || !matches!(slot, 15 | 16) {
        return None;
    }
    set_tier(item, class, slot as u32 - 7).map(|_| item | PIECE_FLAG)
}
pub fn selected_piece(class: u8, slot: usize, look_id: u32) -> Option<u32> {
    let item = look_id & !PIECE_FLAG;
    (piece_look(class, slot, item) == Some(look_id)).then_some(item)
}
fn word(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
fn id(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
impl Adapter {
    #[cfg(feature = "test-hooks")]
    pub fn weapon_catalog_probe(self) -> (crate::official::Entry, u64) {
        let (rva, bytes) = crate::current::WEAPON_CATALOG_PROBE;
        (crate::official::Entry { rva, bytes }, crate::current::WEAPON_CATALOG_FRAME)
    }

    pub fn weapon_melee_animation(self, two_handed: bool, requested: u16) -> Option<u16> {
        match (requested, two_handed) {
            (17, true) => Some(18),
            (18 | 19, false) => Some(17),
            (57, true) => Some(58),
            (58, false) => Some(57),
            _ => None,
        }
    }
    pub fn weapon_two_handed(self, entry: u32) -> Option<bool> {
        model(entry)
            .filter(|v| v.class == 2)
            .map(|v| v.inventory == 17)
    }
    pub fn weapon_button_entries(self) -> [Entry; 2] {
        [0x296900, 0x5c4d7a0].map(|address| Entry {
            rva: self.rva(address),
            bytes: self.expected(address, &[]),
        })
    }
    pub fn weapon_entries(self) -> [Entry; 10] {
        [
            0x5ceedf2, 0x5bffe00, 0x3c18e0, 0x5c4b7f0, 0x5bff970, 0x2a2e7a, 0x6d3893, 0xaf0050, 0x2828c0, 0x3c1caf,
        ]
        .map(|a| Entry {
            rva: self.rva(a),
            bytes: self.expected(a, &[]),
        })
    }
    /// 5d6ef80 passes profile+18 to the native talent builder. Each learned
    /// rank belongs to exactly one contiguous class-tree definition table.
    fn weapon_trees(self, m: &dyn Mem, base: u64, ranks: u64, class: u8) -> Result<[u16; 3]> {
        let trees = match class {
            1 => &crate::current::WEAPON_TALENT_TREES[0],
            2 => &crate::current::WEAPON_TALENT_TREES[1],
            7 => &crate::current::WEAPON_TALENT_TREES[2],
            _ => return Ok([0; 3]),
        };
        let h = m.read(ranks, 24)?;
        let (ptr, n) = (word(&h, 8), word(&h, 16));
        if n == 0 {
            return Ok([0; 3]);
        }
        if n > 64 || n > word(&h, 0) || ptr < 65536 || ptr.checked_add(n * 16).is_none() {
            return Err("Invalid learned talent vector".into());
        }
        let mut totals = [0; 3];
        for row in m.read(ptr, n as usize * 16)?.chunks_exact(16) {
            let Some(def) = word(row, 0).checked_sub(base) else {
                return Ok([0; 3]);
            };
            let Some(tree) = trees.iter().position(|&(start, len)| {
                def >= start && def < start + len * 176 && (def - start) % 176 == 0
            }) else {
                return Ok([0; 3]);
            };
            if row[8] > 5 {
                return Ok([0; 3]);
            }
            totals[tree] += u16::from(row[8]);
        }
        Ok(totals)
    }
    /// Register pair is [RDI, R15] at the selected native Look result seam.
    pub fn weapon_run_profile(self, registers: [u64; 2]) -> u64 {
        const { assert!(matches!(crate::current::WEAPON_RUN_PROFILE_REGISTER, 7 | 15)) };
        registers[usize::from(crate::current::WEAPON_RUN_PROFILE_REGISTER == 15)]
    }
    pub fn weapon_projection(
        self,
        m: &dyn MutableMemory,
        base: u64,
        profile: u64,
        visible: u64,
    ) -> Result<Option<[u32; 2]>> {
        let class = m.read(profile + 0x1a4, 1)?[0];
        self.project_hands(
            m,
            base,
            class,
            profile + 0x18,
            profile + 0x50,
            profile + 0x18c,
            visible,
        )
    }
    /// The local preview reads the selected class's saved talent record from
    /// the same collection passed to the native Look resolver.
    pub fn preview_weapons(
        self,
        m: &dyn MutableMemory,
        base: u64,
        rdi: u64,
        stack: u64,
        picker: u64,
        looks: u64,
        visible: u64,
    ) -> Result<Option<[u32; 2]>> {
        let class = m.read(picker + 0x30, 1)?[0];
        if !matches!(class, 1 | 2 | 7) {
            return Ok(None);
        }
        let collection = match crate::current::WEAPON_PREVIEW_COLLECTION_SLOT {
            Some(offset) => word(&m.read(stack + offset, 8)?, 0),
            None => rdi,
        };
        let records = m.read(collection + 0x60, 24)?;
        let count = word(&records, 16);
        if count == 0 || count > 16 || count > word(&records, 0) {
            return Ok(None);
        }
        let ptr = word(&records, 8);
        for i in 0..count {
            let row = ptr + i * 0x28;
            if m.read(row + 4, 1)?[0] == class {
                return self.project_hands(
                    m,
                    base,
                    class,
                    row + 8,
                    looks,
                    looks + 0x98 + 60,
                    visible,
                );
            }
        }
        Ok(None)
    }
    fn project_hands(
        self,
        m: &dyn MutableMemory,
        base: u64,
        class: u8,
        ranks: u64,
        look: u64,
        equipped_main: u64,
        visible: u64,
    ) -> Result<Option<[u32; 2]>> {
        let Some(sets) = table(class, self.weapon_trees(m, base, ranks, class)?) else {
            return Ok(None);
        };
        let hands = m.read(visible + 60, 8)?;
        let looks = m.read(look + 15 * 8, 16)?;
        let main = id(&hands, 0);
        let off = id(&hands, 4);
        let selected_main = selected_piece(class, 15, id(&looks, 4));
        let selected_off = selected_piece(class, 16, id(&looks, 12));
        let automatic_main = id(&looks, 0) == 0;
        let automatic_off = id(&looks, 8) == 0;
        let main_tier = if automatic_main {
            set_tier(main, class, 8)
        } else {
            selected_main.and_then(|item| set_tier(item, class, 8))
        };
        let main_tier = if automatic_main
            && main_tier.is_none()
            && main != 0
            && id(&m.read(equipped_main, 4)?, 0) == 0
        {
            Some(0)
        } else {
            main_tier
        };
        let pair = main_tier.and_then(|tier| sets.get(tier));
        let off_pair = if automatic_off {
            pair
        } else {
            selected_off
                .and_then(|item| set_tier(item, class, 9))
                .and_then(|tier| sets.get(tier))
        };
        if pair.is_none() && off_pair.is_none() {
            return Ok(None);
        }
        let left = if let Some(hands) = off_pair {
            hands.1.map_or(0, |model| VISUAL_BASE + model.display)
        } else if !automatic_off && selected_off.is_none() && id(&looks, 12) != WORN_NATIVE {
            id(&looks, 12)
        } else {
            off
        };
        let right = pair.and_then(|hands| hands.0);
        if right.is_some_and(|model| model.inventory == 17) && left != 0 {
            return Ok(None);
        }
        let values = [
            right.map_or(main, |model| VISUAL_BASE + model.display),
            left,
        ];
        m.write(visible + 60, &values.map(u32::to_le_bytes).concat())?;
        Ok(Some(values))
    }
    pub fn weapon_template(self, entry: u32) -> Option<u32> {
        model(entry).map(|m| if m.class == 4 { 107900 } else { 107800 })
    }
    pub fn weapon_metadata(self, m: &dyn MutableMemory, out: u64, entry: u32) -> Result<()> {
        let Some(v) = model(entry) else { return Ok(()) };
        if word(&m.read(out, 8)?, 0) == u64::MAX {
            return Err("Missing cosmetic template".into());
        }
        m.write(
            out + 0x50,
            &[entry, v.display, v.class, v.subclass, v.inventory, v.sheath]
                .map(u32::to_le_bytes)
                .concat(),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_looks_only_roundtrip_real_set_pieces_of_the_same_class_and_hand() {
        for class in [1, 2, 7] {
            for slot in [15, 16] {
                for tier in 0..7 {
                    let item =
                        100000 + u32::from(class) * 1000 + (slot as u32 - 7) * 100 + tier * 10;
                    let choice = piece_look(class, slot, item).unwrap();
                    assert_eq!(selected_piece(class, slot, choice), Some(item));
                    assert_eq!(selected_piece(class, 31 - slot, choice), None);
                    assert_eq!(selected_piece(class, slot, item), None);
                }
            }
        }
        assert_eq!(selected_piece(7, 15, WORN_NATIVE), None);
        assert_eq!(piece_look(1, 15, 107840), None);
    }
    #[test]
    fn explicit_melee_requests_follow_the_rendered_grip() {
        let adapter = Adapter::select(crate::current::SHA256).unwrap();
        for (two, request, expected) in [
            (true, 17, Some(18)),
            (true, 57, Some(58)),
            (false, 18, Some(17)),
            (false, 19, Some(17)),
            (false, 58, Some(57)),
            (true, 18, None),
            (true, 53, None),
            (true, 54, None),
            (false, 87, None),
        ] {
            assert_eq!(adapter.weapon_melee_animation(two, request), expected);
        }
    }
    #[test]
    fn dominant_tree_and_equipped_set_define_the_two_hands() {
        for (class, points) in [(1, [0; 3]), (2, [5, 5, 0]), (1, [8, 5, 1]), (7, [0, 1, 3])] {
            assert!(table(class, points).is_none())
        }
        assert_eq!(
            table(7, [2, 3, 1]).unwrap()[set_tier(107840, 7, 8).unwrap()]
                .0
                .unwrap()
                .display,
            29698
        );
        for (class, points) in [
            (1, [0, 3, 1]),
            (1, [0, 1, 3]),
            (2, [4, 2, 1]),
            (2, [0, 2, 0]),
            (7, [0, 3, 0]),
        ] {
            for tier in 0..7 {
                let t = set_tier(100800 + class as u32 * 1000 + tier * 10, class, 8).unwrap();
                let (r, l) = table(class, points).unwrap()[t];
                assert!(r.is_some());
                assert_eq!(l.is_some(), class != 7);
                for x in [r, l].into_iter().flatten() {
                    assert_eq!(model(VISUAL_BASE + x.display).unwrap().display, x.display);
                }
            }
        }
    }
}
