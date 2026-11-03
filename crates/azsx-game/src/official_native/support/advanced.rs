//! Retaliation native translations. All ABI/layout facts stay here.
use super::effects::{f32at, u32at};
use super::*;
#[derive(Clone, Debug)]
pub struct NativeEffect {
    pub data: [u8; 12],
    pub duration: f32,
}
#[derive(Clone, Debug)]
pub struct RetaliationTick {
    pub owner: OwnedEffect,
    pub value: f32,
}
impl Adapter {
    pub fn support_emit_entries(self) -> [Entry; 4] {
        [
            Entry {
                rva: self.rva(0x58fec60),
                bytes: self.expected(0x58fec60, &[65, 87, 65, 86, 86, 87, 85, 83, 72, 131, 236, 72]),
            },
            Entry {
                rva: self.rva(0x58fed50),
                bytes: self.expected(0x58fed50, &[65, 86, 86, 87, 85, 83, 72, 131, 236, 80]),
            },
            Entry {
                rva: self.rva(0x590d030),
                bytes: self.expected(0x590d030, &[
                    85, 65, 87, 65, 86, 65, 85, 65, 84, 86, 87, 83, 72, 129, 236, 200, 1, 0, 0,
                ]),
            },
            Entry {
                rva: self.rva(0x5904950),
                bytes: self.expected(0x5904950, &[
                    85, 65, 87, 65, 86, 65, 85, 65, 84, 86, 87, 83, 72, 129, 236, 8, 1, 0, 0,
                ]),
            },
        ]
    }
    pub fn fixture_retaliation_hit(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
    ) -> Result<Option<u64>> {
        if io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Fixture retaliation requires restored host".into());
        }
        // A bounded durable native witness prevents an unrelated attack from
        // killing the queued retaliation target before the next server tick.
        let enemy=self.fixture_combat_enemy(io,game,crate::official_native::NATIVE_PLAYERS as u32)?;
        let row = self.support_row(io, game, slot)?;
        if slot != 0 {
            io.swap_actor(game, row)?;
            io.write(game + self.game_field(0xb40), &u64::from(slot).to_le_bytes())?;
        }
        let result = (|| {
            let Some(enemy)=enemy else{return Ok(None)};
            io.support_hurt(game, 2.0, enemy, true)?;
            io.flush_actor(game)?;
            Ok(Some(enemy))
        })();
        if slot != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result
    }
    pub fn native_effect(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<NativeEffect> {
        let clock = mem.read(game + self.game_field(0xb9c), 4)?;
        let clock = f32at(&clock, 0);
        let (_, rows) = self.aura_rows(mem, game)?;
        let row = rows
            .iter()
            .find(|r| u32at(*r, 12) == spell && u32at(*r, 0) == 13)
            .ok_or("Missing native retaliation aura")?;
        let duration = f32at(row, 16) - clock;
        if !duration.is_finite() || !(0.0..=600.0).contains(&duration) || !f32at(row, 4).is_finite()
        {
            return Err("Invalid retaliation aura".into());
        }
        Ok(NativeEffect {
            data: row[..12].try_into().unwrap(),
            duration,
        })
    }
    pub fn hide_retaliation(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        owners: &[OwnedEffect],
    ) -> Result<Vec<RetaliationTick>> {
        let actor = self.support_caster(mem, game)?;
        let (p, rows) = self.aura_rows(mem, game)?;
        let clock = mem.read(game + self.game_field(0xb9c), 4)?;
        let clock = f32at(&clock, 0);
        let mut pending = vec![];
        for (i, r) in rows.iter().enumerate() {
            let Some(owner) = owners.iter().find(|o| {
                o.target_guid == actor.guid
                    && o.caster != actor.slot
                    && matches!(o.spell, 324 | 467)
                    && o.spell == u32at(r, 12)
                    && o.expires.to_bits() == u32at(r, 16)
                    && o.expires > clock
            }) else {
                continue;
            };
            let value = f32at(r, 4);
            if u32at(r, 0) != 13 || !value.is_finite() || !(0.0..=1e12).contains(&value) {
                return Err("Invalid foreign retaliation amount".into());
            }
            pending.push((
                p + i as u64 * self.aura_stride() + self.aura_field(4),
                RetaliationTick {
                    owner: owner.clone(),
                    value,
                },
            ));
        }
        for (a, _) in &pending {
            mem.write(*a, &0f32.to_le_bytes())?;
        }
        Ok(pending.into_iter().map(|(_, t)| t).collect())
    }
    pub fn restore_retaliation(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        t: &RetaliationTick,
    ) -> Result<bool> {
        if self.support_caster(mem, game)?.guid != t.owner.target_guid {
            return Ok(false);
        }
        let (p, rows) = self.aura_rows(mem, game)?;
        let Some((i, r)) = rows.iter().enumerate().find(|(_, r)| {
            u32at(*r, 12) == t.owner.spell
                && u32at(*r, 16) == t.owner.expires.to_bits()
                && u32at(*r, 0) == 13
        }) else {
            return Ok(false);
        };
        if f32at(r, 4) != 0.0 {
            return Err("Native retaliation refreshed during hurt".into());
        }
        mem.write(p + i as u64 * self.aura_stride() + self.aura_field(4), &t.value.to_le_bytes())?;
        Ok(true)
    }
    pub fn retaliate(
        self,
        io: &dyn SupportCalls,
        game: u64,
        t: &RetaliationTick,
        enemy: u64,
    ) -> Result<(f32, f32)> {
        if io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Retaliation requires restored host".into());
        }
        let row = self.support_row(io, game, t.owner.caster)?;
        if io.read(row + self.row_field(row == game, 0x478), 8)? != t.owner.caster_guid.to_le_bytes() {
            return Err("Retaliation source changed".into());
        }
        if t.owner.caster != 0 {
            io.swap_actor(game, row)?;
            io.write(game + self.game_field(0xb40), &u64::from(t.owner.caster).to_le_bytes())?;
        }
        let result = (|| {
            let power = io.read(game + self.game_field(0x494), 4)?;
            let bonus = io.read(game + self.game_field(0x4d0), 4)?;
            let amount = t.value * f32at(&power, 0) * (1.0 + f32at(&bonus, 0));
            if !amount.is_finite() || !(0.0..=1e12).contains(&amount) {
                return Err("Retaliation damage overflow".into());
            }
            let before = self
                .support_enemies(io, game)?
                .into_iter()
                .find(|e| e.guid == enemy && e.hp > 0.0)
                .map(|e| e.hp)
                .unwrap_or(0.0);
            if before <= 0.0 {
                return Ok((amount, 0.0));
            }
            io.support_hit(game, enemy, amount, t.owner.spell)?;
            let after = self
                .support_enemies(io, game)?
                .into_iter()
                .find(|e| e.guid == enemy)
                .map(|e| e.hp)
                .unwrap_or(0.0);
            let effective = (before - after).max(0.0) as f32;
            io.flush_actor(game)?;
            Ok((amount, effective))
        })();
        if t.owner.caster != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result
    }
}
