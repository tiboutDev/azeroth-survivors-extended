//! Native field translation for the sole supported official client, 0.130.0.
use super::Adapter;
use crate::{memory::Mem, Result};

impl Adapter {
    pub(super) fn game_field(self, old: u64) -> u64 {
        crate::current::game_field(old)
    }

    pub(super) fn row_field(self, host: bool, old: u64) -> u64 {
        if host {
            self.game_field(old)
        } else {
            crate::current::actor_field(old)
        }
    }

    pub(super) fn context_size(self) -> u64 {
        crate::current::CONTEXT_SIZE
    }

    pub(super) fn read_game(self, mem: &dyn Mem, game: u64) -> Result<Vec<u8>> {
        self.read_context(mem, game, true)
    }

    pub(super) fn read_context(self, mem: &dyn Mem, address: u64, host: bool) -> Result<Vec<u8>> {
        let size = if host { 0xc48 } else { 0x980 };
        let native_size = if host {
            self.game_field((size - 1) as u64) as usize + 1
        } else {
            crate::current::CONTEXT_SIZE as usize
        };
        let raw = mem.read(address, native_size)?;
        if raw.len() != native_size {
            return Err("Truncated selected native context".into());
        }
        let mut normalized = vec![0; size];
        const ACTOR_FIELDS: &[(usize, usize)] = &[
            (0x00, 0x04),
            (0x32c, 0x32e),
            (0x478, 0x480),
            (0x490, 0x494),
            (0x4ac, 0x4b0),
            (0x4c0, 0x4c4),
            (0x4e0, 0x4ec),
            (0x4f0, 0x4fc),
            (0x500, 0x504),
        ];
        const HOST_FIELDS: &[(usize, usize)] = &[
            (0x838, 0x848),
            (0x980, 0x990),
            (0xb40, 0xb48),
            (0xb9c, 0xba0),
            (0xc41, 0xc42),
            (0xc46, 0xc47),
        ];
        const PRIVATE_FIELDS: &[(usize, usize)] = &[(0x97c, 0x97d)];
        let spans = if host { HOST_FIELDS } else { PRIVATE_FIELDS };
        for &(start, end) in ACTOR_FIELDS.iter().chain(spans) {
            let length = end - start;
            let native = self.row_field(host, start as u64);
            let last = self.row_field(host, (end - 1) as u64);
            if last != native + length as u64 - 1 {
                return Err("Selected context field crosses a native layout boundary".into());
            }
            let native = usize::try_from(native)
                .map_err(|_| "Selected native context field offset overflow")?;
            let native_end = native
                .checked_add(length)
                .ok_or("Selected native context field extent overflow")?;
            let input = raw
                .get(native..native_end)
                .ok_or("Selected native field outside context")?;
            normalized[start..end].copy_from_slice(input);
        }
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ContextMemory {
        base: u64,
        bytes: Vec<u8>,
    }

    impl Mem for ContextMemory {
        fn read(&self, address: u64, length: usize) -> Result<Vec<u8>> {
            let start = address
                .checked_sub(self.base)
                .and_then(|value| usize::try_from(value).ok())
                .ok_or("Context read before fixture")?;
            self.bytes
                .get(start..start.checked_add(length).ok_or("Context extent overflow")?)
                .map(<[u8]>::to_vec)
                .ok_or_else(|| "Context read outside fixture".into())
        }
    }

    #[test]
    fn snapshots_copy_only_the_fields_consumed_by_party_and_support() {
        let adapter = Adapter::select(crate::current::SHA256).unwrap();
        let spans: [(usize, usize); 15] = [
            (0x00, 0x04),
            (0x32c, 0x32e),
            (0x478, 0x480),
            (0x490, 0x494),
            (0x4ac, 0x4b0),
            (0x4c0, 0x4c4),
            (0x4e0, 0x4ec),
            (0x4f0, 0x4fc),
            (0x500, 0x504),
            (0x838, 0x848),
            (0x980, 0x990),
            (0xb40, 0xb48),
            (0xb9c, 0xba0),
            (0xc41, 0xc42),
            (0xc46, 0xc47),
        ];

        let mut native = vec![0xa5; 0xe00];
        for &(start, end) in &spans {
            let offset = usize::try_from(adapter.row_field(true, start as u64)).unwrap();
            for byte in offset..offset + end - start {
                native[byte] = (byte as u8).wrapping_mul(37).wrapping_add(11);
            }
        }

        let memory = ContextMemory {
            base: 0x10000,
            bytes: native.clone(),
        };
        let snapshot = adapter.read_context(&memory, memory.base, true).unwrap();
        assert_eq!(snapshot.len(), 0xc48);

        let mut copied = vec![false; snapshot.len()];
        for &(start, end) in &spans {
            let offset = usize::try_from(adapter.row_field(true, start as u64)).unwrap();
            assert_eq!(snapshot[start..end], native[offset..offset + end - start]);
            copied[start..end].fill(true);
        }
        assert!(snapshot
            .iter()
            .zip(copied)
            .all(|(byte, selected)| selected || *byte == 0));

        let private = adapter.read_context(&memory, memory.base, false).unwrap();
        assert_eq!(private.len(), 0x980);
        for &(start, end) in &spans[..9] {
            let offset = usize::try_from(adapter.row_field(false, start as u64)).unwrap();
            assert_eq!(private[start..end], native[offset..offset + end - start]);
        }
        let phase = adapter.row_field(false, 0x97c) as usize;
        assert_eq!(private[0x97c], native[phase]);
        assert!(private[0x838..0x848].iter().all(|byte| *byte == 0));
    }
}
