//! Cosmetic native debuff projection. Never inserts a gameplay/stat aura.
use super::*;
use crate::official::{Entry, MutableMemory};
pub struct FatigueAura {
    pub game: u64,
    pub slot: u32,
    pub guid: u64,
    pub(super) rows: u64,
    pub(super) clock: f32,
}
impl Adapter {
    pub fn fatigue_aura_entry(self) -> Option<Entry> {
        self.supports_extensions().then_some(Entry {
            rva: self.contact117(crate::current::FATIGUE_AURA_RVA),
            bytes: self.extension_bytes117(
                crate::current::FATIGUE_AURA_RVA,
                crate::current::FATIGUE_AURA_BYTES,
            ),
        })
    }
    pub fn fatigue_aura_frame(
        self,
        mem: &dyn Mem,
        registers: [u64; 15],
    ) -> Result<Option<FatigueAura>> {
        if self.fatigue_aura_entry().is_none() {
            return Ok(None);
        }
        let game = registers[0];
        let Some((slot, guid)) = self.run_source(mem, game)? else {
            return Ok(None);
        };
        let at = registers[8]
            .checked_add_signed(crate::current::AURA_ROWS_FRAME_SLOT)
            .ok_or("Aura frame overflow")?;
        let rows = u64::from_le_bytes(
            mem.read(at, 8)?
                .try_into()
                .map_err(|_| "Truncated aura frame")?,
        );
        if rows < 0x10000 || rows.checked_add(48 * 16).is_none() {
            return Err("Invalid aura projection extent".into());
        }
        let clock = f32::from_le_bytes(
            mem.read(game + self.game_field(0xb9c), 4)?
                .try_into()
                .map_err(|_| "Truncated aura clock")?,
        );
        if !clock.is_finite() || clock < 0.0 {
            return Err("Invalid aura projection clock".into());
        }
        Ok(Some(FatigueAura {
            game,
            slot,
            guid,
            rows,
            clock,
        }))
    }
    pub fn project_fatigue_aura(
        self,
        mem: &dyn MutableMemory,
        frame: &FatigueAura,
        stacks: u8,
        remaining: f32,
    ) -> Result<bool> {
        if stacks > 3 || !remaining.is_finite() || !(0.0..=60.0).contains(&remaining) {
            return Err("Invalid fatigue presentation".into());
        }
        if stacks == 0 {
            return Ok(false);
        }
        let bytes = mem.read(frame.rows + 32 * 16, 16 * 16)?;
        if bytes.len() != 256 {
            return Err("Truncated harmful aura slots".into());
        }
        let mut free = None;
        for (index, row) in bytes.chunks_exact(16).enumerate() {
            let tag = u32::from_le_bytes(row[..4].try_into().unwrap());
            if tag > 1 {
                return Err("Invalid native aura variant".into());
            }
            if tag == 0 && free.is_none() {
                free = Some(index);
            }
            if tag == 1 && u32::from_le_bytes(row[4..8].try_into().unwrap()) == 15007 {
                return Ok(false); // Preserve a real native Resurrection Sickness.
            }
        }
        let Some(index) = free else {
            return Ok(false);
        }; // Never displace native debuffs.
        let mut row = [0u8; 16];
        row[..4].copy_from_slice(&1u32.to_le_bytes());
        row[4..8].copy_from_slice(&15007u32.to_le_bytes());
        row[8..12].copy_from_slice(&(frame.clock + remaining).to_le_bytes());
        row[12..16].copy_from_slice(&u32::from(stacks).to_le_bytes());
        mem.write(frame.rows + (32 + index as u64) * 16, &row)?;
        Ok(true)
    }
}

#[cfg(test)]
mod current_frame_tests {
    use super::*;
    struct Bytes(Vec<u8>);
    impl Mem for Bytes {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>> {
            self.0
                .get(a as usize..a as usize + n)
                .map(|v| v.to_vec())
                .ok_or("outside fixture".into())
        }
    }
    #[test]
    fn aura_pointer_uses_the_current_negative_stack_slot() {
        let a = Adapter::select(crate::current::SHA256).unwrap();
        let mut m = Bytes(vec![0; 0x50000]);
        let game = 0x10000;
        m.0[game + 0xd9e] = 1;
        m.0[game + 0xda3] = 1;
        m.0[game + 0x4b0..game + 0x4b8].copy_from_slice(&123u64.to_le_bytes());
        m.0[game + 0xce0..game + 0xce4].copy_from_slice(&42f32.to_le_bytes());
        m.0[0x30000..0x30008].copy_from_slice(&0x40000u64.to_le_bytes());
        let mut r = [0u64; 15];
        r[0] = game as u64;
        r[8] = 0x30008;
        let f = a.fatigue_aura_frame(&m, r).unwrap().unwrap();
        assert_eq!(
            (f.game, f.guid, f.rows, f.clock),
            (0x10000, 123, 0x40000, 42.)
        );
    }
}
