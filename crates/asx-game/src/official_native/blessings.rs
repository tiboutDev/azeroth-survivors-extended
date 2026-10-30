//! Exact-build event/stat/pool boundaries; no runtime entity scan.
use super::*;
use crate::official::{Entry, MutableMemory, SupportCalls};
use crate::current as v;
fn q(m: &dyn Mem, a: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(
        m.read(a, 8)?
            .try_into()
            .map_err(|_| "Truncated blessing pointer")?,
    ))
}
impl Adapter {
    pub fn blessing_entries(self) -> Vec<Entry> {
        if !self.supports_extensions() {
            return vec![];
        }
        vec![
            Entry {
                rva: self.contact117(v::BLESSING_EVENT_RVA),
                bytes: self.extension_bytes117(0x5bd7960, v::BLESSING_EVENT_BYTES),
            },
            Entry {
                rva: self.contact117(v::BLESSING_POOL_RVA),
                bytes: self.extension_bytes117(0x5c03b20, v::BLESSING_POOL_BYTES),
            },
            Entry {
                rva: self.contact117(v::BLESSING_STATS_RVA),
                bytes: self.extension_bytes117(0x5b94210, v::BLESSING_STATS_BYTES),
            },
        ]
    }
    pub fn blessing_map(self, m: &dyn Mem, game: u64, base: u64) -> Result<(u64, u64)> {
        let ptr = q(m, game + self.field117(v::BLESSING_GAME_MAP[0]))?;
        let len = q(m, game + self.field117(v::BLESSING_GAME_MAP[1]))?;
        if !(1..=32).contains(&len) {
            return Err("Invalid map name extent".into());
        }
        let name = m.read(ptr, len as usize)?;
        for index in 0..v::BLESSING_MAP_COUNT {
            let row =
                base + self.contact117(v::BLESSING_MAP_TABLE) + index * v::BLESSING_MAP_STRIDE;
            if q(m, row + 16)? == len && m.read(q(m, row + 8)?, len as usize)? == name {
                if q(m, row + 0x90)? != 5 {
                    return Err("Unexpected unique boss count".into());
                }
                return Ok((index, q(m, row + 0x88)?));
            }
        }
        Err("Unknown blessing map".into())
    }
    pub fn blessing_event(
        self,
        m: &dyn Mem,
        frame: u64,
        definition: u64,
        base: u64,
    ) -> Result<Option<(u64, u64)>> {
        let game = q(
            m,
            frame
                .checked_add(crate::current::ENCOUNTER_GAME_FRAME)
                .ok_or("Encounter frame overflow")?,
        )?;
        if self.run_source(m, game)?.is_none() {
            return Ok(None);
        }
        if m.read(game + self.field117(v::BLESSING_ENCOUNTER_PHASE), 4)? != 0u32.to_le_bytes() {
            return Ok(None);
        }
        let (_map, defs) = self.blessing_map(m, game, base)?;
        let offset = definition
            .checked_sub(defs)
            .ok_or("Boss does not belong to current map")?;
        if offset >= 5 * v::BLESSING_BOSS_STRIDE || offset % v::BLESSING_BOSS_STRIDE != 0 {
            return Err("Invalid unique encounter definition".into());
        }
        // The native phase transition runs once per completed encounter, including endless.
        // Return the normal-map encounter count, not a boss identity to deduplicate.
        Ok(Some((game, 5)))
    }
    pub fn blessing_pool_index(self, m: &dyn Mem, frame: u64, index: u64) -> Result<u64> {
        if !(16 * 72..19 * 72).contains(&index) {
            return Ok(index);
        }
        let game = q(
            m,
            frame
                .checked_add(v::BLESSING_POOL_FRAME)
                .ok_or("Pool frame overflow")?,
        )?;
        Ok(if self.run_source(m, game)?.is_some() {
            19 * 72
        } else {
            index
        })
    }
    pub fn blessing_stats(
        self,
        m: &dyn MutableMemory,
        game: u64,
        b: Option<asx_gameplay::blessings::Blessing>,
    ) -> Result<()> {
        let Some(b) = b else {
            return Ok(());
        };
        if !self.supports_extensions() {
            return Err("Unsupported fortune stats".into());
        }
        let field = *v::BLESSING_STAT_FIELDS
            .get(b.category as usize)
            .ok_or("Invalid fortune category")?;
        let v = f32::from_le_bytes(
            m.read(game + self.field117(field), 4)?
                .try_into()
                .map_err(|_| "Blessing stat")?,
        );
        if !v.is_finite() || v < 0.0 {
            return Err("Invalid blessing base stat".into());
        }
        let next = if b.category == 3 {
            v * (1.0 + b.factor())
        } else {
            v + b.factor()
        };
        m.write(game + self.field117(field), &next.to_le_bytes())
    }
    pub fn refresh_blessings(
        self,
        io: &dyn SupportCalls,
        game: u64,
        ids: &[(u32, u64)],
    ) -> Result<()> {
        if self.run_source(io, game)?.is_none_or(|(slot, _)| slot != 0) {
            return Err("Blessing refresh requires restored host".into());
        }
        for &(slot, guid) in ids {
            let row = self.support_row(io, game, slot)?;
            if q(io, row + self.row_field(row == game, 0x478))? != guid {
                continue;
            }
            if slot != 0 {
                io.swap_actor(game, row)?;
                io.write(
                    game + self.game_field(0xb40),
                    &u64::from(slot).to_le_bytes(),
                )?;
            }
            let result = io.refresh_support(game);
            if slot != 0 {
                let saved = self.support_row(io, game, 0)?;
                io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
                io.swap_actor(game, saved)?;
            }
            result?;
        }
        Ok(())
    }
    pub fn blessing_fixture_targets(self, m: &dyn Mem, game: u64) -> Result<Vec<u64>> {
        let (_, rows) = self.combat_rows(m, game)?;
        use crate::current as layout;
        Ok(rows
            .chunks_exact(layout::ENEMY_STRIDE)
            .filter(|row| {
                row[layout::ENEMY_DEAD] == 0
                    && u64::from_le_bytes(row[0x18..0x20].try_into().unwrap()) != u64::MAX
            })
            .map(|row| {
                u64::from_le_bytes(
                    row[layout::ENEMY_GUID..layout::ENEMY_GUID + 8]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect())
    }
    pub fn blessing_stat_values(self, m: &dyn Mem, game: u64) -> Result<[f32; 5]> {
        let mut out = [0.; 5];
        for (i, offset) in v::BLESSING_STAT_FIELDS.iter().enumerate() {
            out[i] = f32::from_le_bytes(
                m.read(game + self.field117(*offset), 4)?
                    .try_into()
                    .map_err(|_| "Truncated blessing stat")?,
            );
        }
        Ok(out)
    }
    pub fn blessing_endless_entry(self) -> Entry {
        Entry {
            rva: self.contact117(v::BLESSING_ENDLESS_RVA),
            bytes: self.extension_bytes117(0x5b853e0, v::BLESSING_ENDLESS_BYTES),
        }
    }
    pub fn blessing_fixture_victory(self, m: &dyn Mem, game: u64) -> Result<bool> {
        Ok(m.read(game + self.field117(v::BLESSING_GROUP_PHASE), 1)? == [5])
    }
    pub fn blessing_spawn_entry(self) -> Entry {
        Entry {
            rva: self.contact117(v::BLESSING_SPAWN_RVA),
            bytes: crate::current::BLESSING_SPAWN_BYTES,
        }
    }
    pub fn blessing_fixture_definition(
        self,
        m: &dyn Mem,
        game: u64,
        base: u64,
        index: u32,
    ) -> Result<u64> {
        if index >= 5 {
            return Err("Invalid fixture boss index".into());
        }
        Ok(self.blessing_map(m, game, base)?.1 + u64::from(index) * v::BLESSING_BOSS_STRIDE)
    }
    pub fn project_blessing_aura(self, m: &dyn MutableMemory, f: &FatigueAura) -> Result<bool> {
        let bytes = m.read(f.rows, 32 * 16)?;
        if bytes.len() != 512 {
            return Err("Truncated positive aura slots".into());
        }
        let mut free = None;
        for (index, row) in bytes.chunks_exact(16).enumerate() {
            let tag = u32::from_le_bytes(row[..4].try_into().unwrap());
            if tag > 1 {
                return Err("Invalid positive aura variant".into());
            }
            if tag == 0 && free.is_none() {
                free = Some(index);
            }
            if tag == 1 && u32::from_le_bytes(row[4..8].try_into().unwrap()) == 23768 {
                return Ok(false);
            }
        }
        let Some(index) = free else {
            return Ok(false);
        };
        let mut row = [0u8; 16];
        row[..4].copy_from_slice(&1u32.to_le_bytes());
        row[4..8].copy_from_slice(&23768u32.to_le_bytes());
        m.write(f.rows + index as u64 * 16, &row)?;
        Ok(true)
    }
}

impl Adapter {
    pub fn blessing_event_context(
        self,
        m: &dyn Mem,
        registers: [u64; 3],
        base: u64,
    ) -> Result<Option<(u64, u64)>> {
        let [frame, _, definition] = registers;
        self.blessing_event(m, frame, definition, base)
    }
}

impl Adapter {
    pub unsafe fn fixture_spawn_encounter(self, base: usize, game: u64, definition: u64) {
        let at = base + self.blessing_spawn_entry().rva as usize;
        let call: unsafe extern "system" fn(u64, u64, f32) = std::mem::transmute(at);
        call(game, definition, 1.0)
    }
}
