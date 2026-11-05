//! Native Nature strikes: the selected adapter owns critical rolls and proc scope.
use crate::{official::MutableMemory, Result};

pub type NativeDamageCall = unsafe extern "system" fn(u64, u64, f32, u32, u32, bool, bool);

fn critical(state: &mut [u64; 4], chance: f32, multiplier: f32, amount: f32) -> (f32, bool) {
    // The native per-target xoshiro256++ step. Preserve wrapping and f32 rounding.
    let bits = state[0].wrapping_add(state[3]).rotate_left(23).wrapping_add(state[0]);
    let shifted = state[1] << 17;
    state[2] ^= state[0];
    state[3] ^= state[1];
    state[1] ^= state[2];
    state[0] ^= state[3];
    state[2] ^= shifted;
    state[3] = state[3].rotate_left(45);
    let roll = (bits >> 40) as f32 * (1.0 / 16777216.0);
    let is_critical = chance > roll;
    (if is_critical { amount * multiplier } else { amount }, is_critical)
}

unsafe fn strike(call: NativeDamageCall, mem: &dyn MutableMemory, fields: &[u64], game: u64,
    guid: u64, amount: f32) -> Result<()> {
    if fields.is_empty() {
        call(game, guid, amount, 740, 3, false, false);
        return Ok(());
    }
    let [rng, chance, multiplier, flag, kind] = fields else {
        return Err("Unreviewed Nature critical layout".into());
    };
    let at = |offset: u64| game.checked_add(offset).ok_or_else(|| "Nature field overflow".to_string());
    let read = |offset, length| -> Result<Vec<u8>> {
        let data = mem.read(at(offset)?, length)?;
        if data.len() != length { return Err("Truncated Nature critical field".into()); }
        Ok(data)
    };
    let seed = read(*rng, 32)?;
    let mut state = std::array::from_fn(|n| u64::from_le_bytes(seed[n * 8..n * 8 + 8].try_into().unwrap()));
    let chance = f32::from_le_bytes(read(*chance, 4)?.try_into().unwrap());
    let multiplier = f32::from_le_bytes(read(*multiplier, 4)?.try_into().unwrap());
    if !chance.is_finite() || !multiplier.is_finite() || multiplier < 0.0
        || !amount.is_finite() || !(0.0..=1e12).contains(&amount) {
        return Err("Invalid Nature critical values".into());
    }
    let (amount, is_critical) = critical(&mut state, chance, multiplier, amount);
    if !amount.is_finite() || amount > 1e12 { return Err("Nature critical damage overflow".into()); }
    let old_flag = read(*flag, 1)?;
    let old_kind = read(*kind, 1)?;
    // Commit RNG before the callback; native on-crit effects may consume it again.
    mem.write(at(*rng)?, &state.map(u64::to_le_bytes).concat())?;
    mem.write(at(*flag)?, &[1])?;
    if let Err(error) = mem.write(at(*kind)?, &[3]) {
        mem.write(at(*flag)?, &old_flag)?;
        return Err(error);
    }
    call(game, guid, amount, 740, 3, is_critical, false);
    let restored_kind = mem.write(at(*kind)?, &old_kind);
    let restored_flag = mem.write(at(*flag)?, &old_flag);
    restored_kind?;
    restored_flag
}

/// `call` must be the selected and byte-verified native Damage entry.
pub unsafe fn call_native_nature_damage(call: NativeDamageCall, mem: &dyn MutableMemory,
    game: u64, guid: u64, amount: f32) -> Result<()> {
    strike(call, mem, crate::current::NATURE_CRITICAL_FIELDS, game, guid, amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [u64; 4] = [u64::MAX, 0x0123456789abcdef, 0xfedcba9876543210, 0x123456789abcdef0];
    const AFTER: [u64; 4] = [0xece8ece0ece8ece0, 0, 0x8bec56301275cdef, 0xe263e262e263e262];

    #[test]
    fn native_critical_boundary_and_rng_state_match_the_pinned_image() {
        for (chance, expected) in [(0., false), (0.23555553, false), (0.25, true), (1., true)] {
            let mut state = SEED;
            let (amount, crit) = critical(&mut state, chance, 1.75, 10.);
            assert_eq!(state, AFTER);
            assert_eq!(crit, expected);
            assert_eq!(amount, if expected { 17.5 } else { 10. });
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn native_damage_abi_and_nested_proc_scope_preserve_all_arguments() {
        use crate::memory::Mem;
        use std::cell::RefCell;
        struct Memory(RefCell<Vec<u8>>);
        impl Mem for Memory {
            fn read(&self, at: u64, size: usize) -> Result<Vec<u8>> {
                let data = self.0.borrow();
                let offset = at.checked_sub(data.as_ptr() as u64).ok_or("Invalid probe address")? as usize;
                data.get(offset..offset + size).map(|v| v.to_vec()).ok_or_else(|| "Invalid probe read".into())
            }
        }
        impl MutableMemory for Memory {
            fn write(&self, at: u64, bytes: &[u8]) -> Result<()> {
                let mut data = self.0.borrow_mut();
                let offset = at.checked_sub(data.as_ptr() as u64).ok_or("Invalid probe address")? as usize;
                data.get_mut(offset..offset + bytes.len()).ok_or("Invalid probe write")?.copy_from_slice(bytes);
                Ok(())
            }
            fn allocate_native(&self, _: &[u8]) -> Result<u64> { Err("Unexpected probe allocation".into()) }
            fn free_native(&self, _: u64, _: usize) -> Result<()> { Err("Unexpected probe free".into()) }
        }
        unsafe extern "system" fn capture(game: u64, guid: u64, amount: f32, spell: u32,
            school: u32, crit: bool, final_flag: bool) {
            let output = [guid, u64::from(amount.to_bits()), u64::from(spell), u64::from(school), u64::from(crit), u64::from(final_flag)];
            std::ptr::copy_nonoverlapping(output.as_ptr().cast::<u8>(), game as *mut u8, 48);
            *((game + 56) as *mut u8) = *((game + 0xa8) as *const u8);
            *((game + 57) as *mut u8) = *((game + 0xaa) as *const u8);
            // Model a reentrant native proc consuming the current RNG.
            std::ptr::write_unaligned((game + 0x80) as *mut u64, 77);
        }
        let memory = Memory(RefCell::new(vec![0; 256]));
        let game = memory.0.borrow().as_ptr() as u64;
        memory.write(game + 0x80, &SEED.map(u64::to_le_bytes).concat()).unwrap();
        memory.write(game + 0xa0, &0.25f32.to_le_bytes()).unwrap();
        memory.write(game + 0xa4, &1.75f32.to_le_bytes()).unwrap();
        memory.write(game + 0xa8, &[9, 91, 7]).unwrap();
        unsafe { strike(capture, &memory, &[0x80, 0xa0, 0xa4, 0xa8, 0xaa], game, 0xf130000100000007, 10.).unwrap(); }
        let values: Vec<u64> = memory.read(game, 48).unwrap().chunks_exact(8)
            .map(|v| u64::from_le_bytes(v.try_into().unwrap())).collect();
        assert_eq!(values, [0xf130000100000007, u64::from(17.5f32.to_bits()), 740, 3, 1, 0]);
        assert_eq!(memory.read(game + 56, 2).unwrap(), [1, 3]);
        assert_eq!(memory.read(game + 0xa8, 3).unwrap(), [9, 91, 7]);
        assert_eq!(memory.read(game + 0x80, 8).unwrap(), 77u64.to_le_bytes());
        unsafe { strike(capture, &memory, &[], game, 0xf130000100000008, 12.5).unwrap(); }
        let values: Vec<u64> = memory.read(game, 48).unwrap().chunks_exact(8)
            .map(|v| u64::from_le_bytes(v.try_into().unwrap())).collect();
        assert_eq!(values, [0xf130000100000008, u64::from(12.5f32.to_bits()), 740, 3, 0, 0]);
        assert_eq!(memory.read(game + 56, 2).unwrap(), [9, 7]);
        assert!(unsafe { strike(capture, &memory, &[0x80], game, 0, 1.) }.is_err());
        assert_eq!(memory.read(game + 0xa8, 3).unwrap(), [9, 91, 7]);
    }
}
