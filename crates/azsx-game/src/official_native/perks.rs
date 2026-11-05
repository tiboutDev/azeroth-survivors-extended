//! Selected perk observations. Normalized copies never go back to native
//! storage: the dispatcher keeps its own enum, padding, cooldowns and row order.
use crate::Result;

pub(super) fn observe(layout: u8, native: &[u8]) -> Result<Option<[u8; 64]>> {
    let raw: [u8; 64] = native.try_into().map_err(|_| "Truncated native perk")?;
    if layout == 0 {
        return Ok(Some(raw));
    }
    if layout != 1 {
        return Err("Unreviewed perk layout".into());
    }
    let mut row = [0; 64];
    match u32::from_le_bytes(raw[..4].try_into().unwrap()) {
        12 => {
            // Conditional cast: condition, spell key, chance, cooldown.
            row[0] = 0;
            row[4..12].copy_from_slice(&raw[56..64]);
            row[16..32].copy_from_slice(&raw[40..56]);
            row[32..64].copy_from_slice(&raw[8..40]);
        }
        15 => {
            // Shield burst: condition, coefficient, radius, spell, health base.
            let flag = u32::from_le_bytes(raw[52..56].try_into().unwrap());
            if flag > 1 { return Ok(None); }
            row[0] = 3;
            row[1] = flag as u8;
            row[4..16].copy_from_slice(&raw[40..52]);
            row[16..48].copy_from_slice(&raw[8..40]);
        }
        23 => {
            let flag = u32::from_le_bytes(raw[48..52].try_into().unwrap());
            if flag > 1 { return Ok(None); }
            row[0] = 11;
            row[1] = flag as u8;
            row[4..16].copy_from_slice(&raw[32..44]);
            row[16..20].copy_from_slice(&raw[44..48]);
            row[24..48].copy_from_slice(&raw[8..32]);
        }
        26 => {
            row[0] = 14;
            row[4..8].copy_from_slice(&raw[32..36]);
            row[8..32].copy_from_slice(&raw[8..32]);
        }
        27 => {
            // Temporary stat: condition, value, parameter, duration, stat kind.
            let stat = u32::from_le_bytes(raw[52..56].try_into().unwrap());
            let Ok(stat) = u8::try_from(stat) else { return Ok(None) };
            row[0] = 15;
            row[1] = stat;
            row[4..16].copy_from_slice(&raw[40..52]);
            row[16..48].copy_from_slice(&raw[8..40]);
        }
        30 => {
            row[0] = 18;
            row[4..12].copy_from_slice(&raw[40..48]);
            row[16..48].copy_from_slice(&raw[8..40]);
        }
        32 | 33 => {
            row[0] = raw[0] - 12; // Tree of Life / Tranquility scalars are unchanged.
            row[4..12].copy_from_slice(&raw[4..12]);
        }
        38 => {
            // Active shapeshift (including Leader of the Pack bear form).
            row[0] = 26;
            row[4..8].copy_from_slice(&raw[4..8]);
        }
        39 => {
            row[0] = 27;
            row[4..8].copy_from_slice(&raw[24..28]);
            row[8..24].copy_from_slice(&raw[8..24]);
        }
        // New set variants and unrelated perks remain entirely native. This is
        // an observation filter, never an enum-wide subtraction or a rewrite.
        _ => return Ok(None),
    }
    Ok(Some(row))
}

/// Read the native stat-only variant without passing it to effect dispatch.
pub(super) fn stat_modifier(layout: u8, native: &[u8]) -> Result<Option<(u32, f32)>> {
    if native.len() != 64 { return Err("Truncated native stat perk".into()); }
    let kind = match layout {
        0 if native[0] == 47 => u32::from(native[8]),
        1 if u32::from_le_bytes(native[..4].try_into().unwrap()) == 72 =>
            u32::from_le_bytes(native[8..12].try_into().unwrap()),
        0 | 1 => return Ok(None),
        _ => return Err("Unreviewed native stat perk layout".into()),
    };
    let value = f32::from_le_bytes(native[4..8].try_into().unwrap());
    if !value.is_finite() { return Err("Non-finite native stat perk".into()); }
    Ok(Some((kind, value)))
}

/// Diagnostic native definition filtering: stat rows are rebuilt by native
/// stats, while effect rows go to its own dispatcher unchanged.
pub(super) fn dispatch_effect(layout: u8, native: &[u8]) -> Result<bool> {
    if native.len() != 64 { return Err("Truncated native fixture perk".into()); }
    match layout {
        0 => Ok(native[0] < 47),
        1 => Ok(u32::from_le_bytes(native[..4].try_into().unwrap()) < 72),
        _ => Err("Unreviewed fixture perk layout".into()),
    }
}

/// Disable only this native clone's discriminant, keeping its payload intact.
pub(super) fn disable(layout: u8, native: &mut [u8]) -> Result<()> {
    if native.len() != 64 { return Err("Truncated native perk clone".into()); }
    match layout {
        0 => native[0] = 13,
        1 => native[..4].copy_from_slice(&25u32.to_le_bytes()),
        _ => return Err("Unreviewed perk layout".into()),
    }
    Ok(())
}

/// Temporary dispatch barrier while a private clone is being filtered.
pub(super) fn terminator(layout: u8) -> Result<&'static [u8]> {
    match layout {
        0 => Ok(&[0xff]),
        1 => Ok(&[0xff; 4]),
        _ => Err("Unreviewed perk layout".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_modifiers_use_the_native_enum_and_never_become_dispatch_effects() {
        let mut row = [0; 64];
        row[..4].copy_from_slice(&72u32.to_le_bytes());
        row[4..8].copy_from_slice(&0.6f32.to_le_bytes());
        row[8..12].copy_from_slice(&18u32.to_le_bytes());
        assert_eq!(stat_modifier(1, &row).unwrap(), Some((18, 0.6)));
        assert!(!dispatch_effect(1, &row).unwrap());
        assert_eq!(stat_modifier(0, &row).unwrap(), None);
        row[..4].copy_from_slice(&47u32.to_le_bytes());
        assert_eq!(stat_modifier(1, &row).unwrap(), None);
        assert_eq!(stat_modifier(0, &row).unwrap(), Some((18, 0.6)));
        assert!(stat_modifier(2, &row).is_err());
        row[4..8].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(stat_modifier(0, &row).is_err());
    }

    fn word(row: &mut [u8; 64], at: usize, value: u32) {
        row[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn tier_zero_point_five_effects_are_not_dropped_as_legacy_stat_rows() {
        for tag in [60, 64, 65, 66, 67, 69, 70, 71] {
            let mut native = [0; 64];
            word(&mut native, 0, tag);
            assert!(dispatch_effect(1, &native).unwrap());
        }
        for (layout, tags) in [(0, [47, 48]), (1, [72, 73])] {
            for tag in tags {
                let mut native = [0; 64];
                word(&mut native, 0, tag);
                assert!(!dispatch_effect(layout, &native).unwrap());
            }
        }
        assert!(dispatch_effect(1, &[0; 63]).is_err());
        assert!(dispatch_effect(2, &[0; 64]).is_err());
    }

    #[test]
    fn clone_disabling_and_barriers_respect_native_tag_width() {
        let raw = [0xa7; 64];
        for (layout, width, noop) in [(0, 1, 13), (1, 4, 25)] {
            let mut row = raw;
            disable(layout, &mut row).unwrap();
            assert_eq!(row[0], noop);
            assert!(row[1..width].iter().all(|v| *v == 0));
            assert_eq!(row[width..], raw[width..]);
            assert_eq!(terminator(layout).unwrap(), vec![0xff; width]);
        }
        let mut row = raw;
        assert!(disable(2, &mut row).is_err());
        assert_eq!(row, raw);
        assert!(disable(1, &mut row[..63]).is_err());
        assert!(terminator(2).is_err());
    }

    #[test]
    fn corpse_burst_keeps_death_selector_spell_and_key_distinct() {
        let mut native = [0; 64];
        word(&mut native, 0, 23);
        word(&mut native, 8, 1);
        native[16..24].copy_from_slice(&0x123456789abcdef0u64.to_le_bytes());
        native[24..32].copy_from_slice(&7u64.to_le_bytes());
        word(&mut native, 32, 0.2f32.to_bits());
        word(&mut native, 36, 7f32.to_bits());
        word(&mut native, 40, 1943);
        word(&mut native, 44, 2);
        word(&mut native, 48, 1);
        let row = observe(1, &native).unwrap().unwrap();
        assert_eq!(row[..2], [11, 1]);
        assert_eq!(&row[4..16], &native[32..44]);
        assert_eq!(u32::from_le_bytes(row[16..20].try_into().unwrap()), 2);
        assert_eq!(&row[24..48], &native[8..32]);
    }

    #[test]
    fn historical_observation_preserves_all_bytes_and_unknown_native_perks() {
        let raw = std::array::from_fn(|n| (n as u8).wrapping_mul(29));
        assert_eq!(observe(0, &raw).unwrap(), Some(raw));
        for tag in [0, 7, 25, 54, 60, 64, 65, 66, 67, 69, 70, 71, 72, 73, u32::MAX] {
            let mut native = raw;
            word(&mut native, 0, tag);
            assert_eq!(observe(1, &native).unwrap(), None);
            assert_eq!(native[4..], raw[4..]);
        }
        assert!(observe(1, &raw[..63]).is_err());
        assert!(observe(2, &raw).is_err());
    }

    #[test]
    fn active_form_preserves_bear_and_cat_without_accepting_adjacent_variant() {
        for spell in [768u32, 9634] {
            let mut native = [0; 64];
            word(&mut native, 0, 38);
            word(&mut native, 4, spell);
            let row = observe(1, &native).unwrap().unwrap();
            assert_eq!(row[0], 26);
            assert_eq!(u32::from_le_bytes(row[4..8].try_into().unwrap()), spell);
            word(&mut native, 0, 37);
            assert_eq!(observe(1, &native).unwrap(), None);
        }
    }

    #[test]
    fn conditional_cast_preserves_swiftmend_threshold_and_spell_reference() {
        let mut native = [0; 64];
        word(&mut native, 0, 12);
        word(&mut native, 8, 9);
        word(&mut native, 12, 0.4f32.to_bits());
        native[40..48].copy_from_slice(&0x123456789abcdef0u64.to_le_bytes());
        native[48..56].copy_from_slice(&8u64.to_le_bytes());
        word(&mut native, 56, 1f32.to_bits());
        word(&mut native, 60, 8f32.to_bits());
        let row = observe(1, &native).unwrap().unwrap();
        assert_eq!(&row[16..32], &native[40..56]);
        assert_eq!(&row[32..40], &native[8..16]);
        assert_eq!(&row[4..12], &native[56..64]);
    }

    #[test]
    fn shield_burst_keeps_spell_separate_from_condition_and_health_flag() {
        let mut native = [0; 64];
        word(&mut native, 0, 15);
        word(&mut native, 8, 10);
        word(&mut native, 40, 0.1f32.to_bits());
        word(&mut native, 44, 10f32.to_bits());
        word(&mut native, 48, 15237);
        word(&mut native, 52, 1);
        let row = observe(1, &native).unwrap().unwrap();
        assert_eq!(row[..2], [3, 1]);
        assert_eq!(u32::from_le_bytes(row[12..16].try_into().unwrap()), 15237);
        assert_eq!(u32::from_le_bytes(row[16..20].try_into().unwrap()), 10);
        assert_eq!(&row[4..12], &native[40..48]);
        word(&mut native, 52, 256);
        assert_eq!(observe(1, &native).unwrap(), None);
    }

    #[test]
    fn necklace_leech_retains_stat_kind_duration_and_conditional_key() {
        let mut native = [0; 64];
        word(&mut native, 0, 27);
        word(&mut native, 8, 7);
        word(&mut native, 16, 1);
        native[24..32].copy_from_slice(&0x123456789abcdef0u64.to_le_bytes());
        native[32..40].copy_from_slice(&5u64.to_le_bytes());
        word(&mut native, 40, 0.05f32.to_bits());
        word(&mut native, 44, 1);
        word(&mut native, 48, 10f32.to_bits());
        word(&mut native, 52, 14);
        let row = observe(1, &native).unwrap().unwrap();
        assert_eq!(row[..2], [15, 14]);
        assert_eq!(&row[32..48], &native[24..40]);
        assert_eq!(&row[4..16], &native[40..52]);
        assert_eq!(row[16], 7);
        assert_eq!(row[24], 1);
        word(&mut native, 52, 270);
        assert_eq!(observe(1, &native).unwrap(), None);
    }
}
