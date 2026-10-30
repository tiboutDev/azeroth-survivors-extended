//! Scoped use of native amulet generation. No custom card ownership or serializer.
use super::*;
use crate::official::{Entry, MutableMemory, SupportCalls};
fn q(m: &dyn Mem, a: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(
        m.read(a, 8)?
            .try_into()
            .map_err(|_| "Goblin reward pointer")?,
    ))
}
impl Adapter {
    pub fn goblin_reward_entries(self) -> [Entry; 3] {
        [
            crate::current::GOBLIN_AMULET_POOL,
            crate::current::GOBLIN_AMULET_EXCLUDE,
            (
                crate::current::ITEM_CHOICE_RVA,
                crate::current::ITEM_CHOICE_BYTES,
            ),
        ]
        .map(|(rva, bytes)| Entry { rva, bytes })
    }
    pub fn goblin_amulet_pool(self, m: &dyn MutableMemory, frame: u64, game: u64) -> Result<()> {
        if q(m, frame + 0x168)? != game {
            return Err("Amulet generator owner changed".into());
        }
        let cap = q(m, frame + 0x1a0)?;
        let p = q(m, frame + 0x1a8)?;
        let n = q(m, frame + 0x1b0)?;
        if cap < 3 || cap > 256 || n > cap || p < 0x10000 {
            return Err("Invalid native amulet opportunity pool".into());
        }
        // Plain weighted descriptors own no allocations. Kind 1 asks the native
        // generator for a new amulet; it retains rarity, exclusions and card assembly.
        let mut rows = [0u8; 72];
        for i in 0..3 {
            rows[i * 24..i * 24 + 4].copy_from_slice(&1f32.to_le_bytes());
            rows[i * 24 + 8..i * 24 + 16].copy_from_slice(&1u64.to_le_bytes());
        }
        m.write(p, &rows)?;
        m.write(frame + 0x1b0, &3u64.to_le_bytes())
    }
    pub fn goblin_amulet_key(self, m: &dyn Mem, _game: u64, output: u64) -> Result<Option<String>> {
        if q(m, output + 0x78)? != crate::current::AMULET_CHOICE_TAG {
            return Ok(None);
        }
        let p = q(m, output + 0x80)?;
        let key = q(m, p + 8)?;
        let n = q(m, p + 16)?;
        if p < 0x10000 || key < 0x10000 || !(1..=80).contains(&n) {
            return Err("Invalid native amulet identity".into());
        }
        Ok(Some(
            String::from_utf8(m.read(key, n as usize)?)
                .map_err(|_| "Invalid native amulet text")?,
        ))
    }
    pub fn goblin_exclusion_keys(self, m: &dyn Mem, header: u64) -> Result<Vec<[u64; 3]>> {
        let cap = q(m, header)?;
        let p = q(m, header + 8)?;
        let n = q(m, header + 16)?;
        if n > 128 || cap < n || n > 0 && p < 0x10000 {
            return Err("Invalid native amulet exclusions".into());
        }
        if n == 0 {
            return Ok(vec![]);
        }
        Ok(m.read(p, n as usize * 24)?
            .chunks_exact(24)
            .map(|b| {
                [
                    u64::from_le_bytes(b[..8].try_into().unwrap()),
                    u64::from_le_bytes(b[8..16].try_into().unwrap()),
                    u64::from_le_bytes(b[16..].try_into().unwrap()),
                ]
            })
            .collect())
    }
    pub fn goblin_reward_actor(self, m: &dyn Mem, game: u64) -> Result<Option<(u32, u64)>> {
        self.run_source(m, game)
    }
    pub fn goblin_reward_ready(self, m: &dyn Mem, game: u64, slot: u32, guid: u64) -> Result<bool> {
        if self.run_source(m, game)?.is_none_or(|(s, _)| s != 0) {
            return Ok(false);
        }
        let row = self.support_row(m, game, slot)?;
        Ok(q(m, row + self.row_field(slot == 0, 0x478))? == guid
            && m.read(
                row + self.row_field(slot == 0, if slot == 0 { 0xc46 } else { 0x97c }),
                1,
            )? == [1])
    }
    /// Native ordinary-boss progression inputs; no pending talent-XP credit,
    /// boss-clear counter or loot/pity function is invoked by this event.
    pub fn goblin_marks_amount(self, m: &dyn Mem, base: u64, game: u64) -> Result<f32> {
        let (map, _) = self.blessing_map(m, game, base)?;
        let f = |at| -> Result<f32> {
            Ok(f32::from_le_bytes(
                m.read(at, 4)?
                    .try_into()
                    .map_err(|_| "Native mark coefficient")?,
            ))
        };
        let mut amount = f(base + crate::current::BOSS_MARKS_TABLE + (map & !1) * 2)?;
        if m.read(game + crate::current::EVENT_ENDLESS_OFFSET, 1)? != [0] {
            let repeats = u32::from_le_bytes(
                m.read(game + crate::current::GOBLIN_ENDLESS_KILLS_OFFSET, 4)?
                    .try_into()
                    .map_err(|_| "Native endless count")?,
            );
            amount *= repeats.max(1) as f32;
        }
        let heroic = m.read(game + 0x36f, 1)?[0];
        if heroic > 5 {
            return Err("Invalid native mark difficulty".into());
        }
        amount *= if heroic == 0 {
            f(base + crate::current::NORMAL_MARKS_FACTOR_RVA)?
        } else {
            f(base + crate::current::HEROIC_REWARD_TABLE + u64::from(heroic) * 48 - 8)?
        };
        if !amount.is_finite() || amount < 0. || amount > 1e7 {
            return Err("Invalid native mark reward".into());
        }
        Ok(amount)
    }
    pub fn goblin_credit_marks(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        guid: u64,
        amount: f32,
    ) -> Result<()> {
        if !amount.is_finite()
            || amount <= 0.
            || self.run_source(io, game)?.is_none_or(|(s, _)| s != 0)
        {
            return Err("Invalid restored mark award".into());
        }
        let row = self.support_row(io, game, slot)?;
        if q(io, row + self.row_field(slot == 0, 0x478))? != guid {
            return Err("Native marks recipient changed".into());
        }
        if slot != 0 {
            io.swap_actor(game, row)?;
        }
        let result = (|| {
            if slot != 0 {
                io.write(
                    game + self.game_field(0xb40),
                    &u64::from(slot).to_le_bytes(),
                )?;
            }
            if q(io, game + self.game_field(0x478))? != guid {
                return Err("Swapped marks recipient changed".into());
            }
            let mut writes = vec![];
            // Pending personal marks and total run marks. Do not alter the native
            // maximum marks from one BOSS, boss kill counters or talent XP.
            for offset in crate::current::GOBLIN_MARK_FIELDS {
                let old = f32::from_le_bytes(
                    io.read(game + offset, 4)?
                        .try_into()
                        .map_err(|_| "Native marks field")?,
                );
                if !old.is_finite() || old < 0. || !(old + amount).is_finite() {
                    return Err("Invalid native mark total".into());
                }
                writes.push((game + offset, old + amount));
            }
            for (at, value) in writes {
                io.write(at, &value.to_le_bytes())?;
            }
            io.flush_actor(game)
        })();
        if slot != 0 {
            let host = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, host)?;
        }
        result
    }
    pub unsafe fn goblin_open_amulets(
        self,
        io: &dyn SupportCalls,
        base: u64,
        game: u64,
        slot: u32,
        guid: u64,
        amount: f32,
    ) -> Result<(Vec<String>, f32)> {
        if !self.goblin_reward_ready(io, game, slot, guid)? {
            return Err("Goblin reward actor is not ready".into());
        }
        let row = self.support_row(io, game, slot)?;
        if slot != 0 {
            io.swap_actor(game, row)?;
        }
        let result = (|| {
            if slot != 0 {
                io.write(
                    game + self.game_field(0xb40),
                    &u64::from(slot).to_le_bytes(),
                )?;
            }
            type Choice = unsafe extern "system" fn(u64, u8, bool);
            let f: Choice = std::mem::transmute((base + crate::current::ITEM_CHOICE_RVA) as usize);
            f(game, 1, true);
            let ptr = q(io, game + 0x6a8)?;
            let n = q(io, game + 0x6b0)?;
            if !(1..=3).contains(&n) || ptr < 0x10000 {
                return Err("Invalid goblin amulet choices".into());
            }
            let mut keys = vec![];
            for i in 0..n {
                let key = self
                    .goblin_amulet_key(io, game, ptr + i * 0xa0)?
                    .ok_or("Goblin choice is not an amulet")?;
                if keys.contains(&key) {
                    return Err("Duplicate goblin amulet choice".into());
                }
                keys.push(key);
            }
            io.flush_actor(game)?;
            Ok((keys, amount))
        })();
        if slot != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::treasure_goblins::tests::Memory;
    use super::*;
    #[test]
    fn scoped_pool_retains_allocation_ownership_and_does_not_touch_other_generators() {
        let a = Adapter::select(crate::current::SHA256).unwrap();
        let m = Memory::default();
        let f = 0x10000;
        for (off, v) in [(0x168, 7u64), (0x1a0, 8), (0x1a8, 0x30000), (0x1b0, 5)] {
            m.write(f + off, &v.to_le_bytes()).unwrap();
        }
        m.write(0x30000, &[0xa5; 192]).unwrap();
        assert!(a.goblin_amulet_pool(&m, f, 8).is_err());
        assert_eq!(q(&m, f + 0x1b0).unwrap(), 5);
        a.goblin_amulet_pool(&m, f, 7).unwrap();
        assert_eq!(q(&m, f + 0x1a0).unwrap(), 8);
        assert_eq!(q(&m, f + 0x1a8).unwrap(), 0x30000);
        assert_eq!(q(&m, f + 0x1b0).unwrap(), 3);
        assert_eq!(m.read(0x30048, 120).unwrap(), [0xa5; 120]);
        for i in 0..3 {
            assert_eq!(q(&m, 0x30008 + i * 24).unwrap(), 1);
        }
    }
}
