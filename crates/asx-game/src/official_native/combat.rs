//! Official 0.130 combat translation. Opaque identities, not native pointers,
//! escape this boundary; pursuit edits are confined to the basic-action block.
use super::Adapter;
use crate::{
    memory::Mem,
    official::{Entry, MutableMemory, SupportCalls},
    Result,
};
use asx_gameplay::{Enemy, Mark, Talents};
#[derive(Clone, Debug)]
pub struct CombatActor {
    pub slot: u32,
    pub guid: u64,
    pub position: [f32; 3],
    pub mark: Mark,
    pub talents: Talents,
}
#[derive(Clone, Debug)]
pub struct BasicFrame {
    pub game: u64,
    pub enemy: u64,
    pub(super) frame: u64,
    pub(super) position: [f32; 3],
}
impl BasicFrame {
    pub fn position(&self) -> [f32; 3] {
        self.position
    }
}
fn q(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
fn d(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
fn f(b: &[u8], i: usize) -> f32 {
    f32::from_bits(d(b, i))
}
impl Adapter {
    pub fn combat_descriptions(self, mem: &dyn Mem, base: u64) -> Result<Vec<(u64, String)>> {
        let mut out = vec![];
        for (slot, key, spell) in [
            (0x7010bc0, "fortified", "Shield Slam"),
            (0x7013210, "ardent_defender", "Hammer of Justice"),
            (0x7029000, "leader_of_pack", "Shred"),
        ] {
            let h = mem.read(base + self.rva(slot), 80)?;
            if h.len() != 80 || q(&h, 8) != key.len() as u64 {
                return Err("Native tank talent changed".into());
            }
            let ptr = q(&h, 0);
            let desc = q(&h, 64);
            let len = q(&h, 72);
            if [ptr, desc]
                .iter()
                .any(|p| *p < base || *p >= base + 0x8000000)
                || len > 2048
                || desc
                    .checked_add(len)
                    .is_none_or(|end| end > base + 0x8000000)
            {
                return Err("Invalid native talent description".into());
            }
            if mem.read(ptr, key.len())? != key.as_bytes() {
                return Err("Native tank talent key mismatch".into());
            }
            let old = String::from_utf8(mem.read(desc, len as usize)?)
                .map_err(|_| "Invalid native talent text")?;
            let end = if self.supports_immunity_aggro() {
                "die, leave, gain immunity, or another player taunts them. You can taunt again while immune"
            } else {
                "die, leave, or another player taunts them"
            };
            out.push((base+self.rva(slot)+64,format!("{old}\n\nCoop: {spell} hits taunt enemies, making them pursue you and direct basic melee attacks at you until you {end}. Special abilities keep their normal targeting.")));
        }
        Ok(out)
    }
    pub fn combat_entries(self) -> [Entry; 5] {
        [
            Entry {
                rva: self.rva(0x5965190),
                bytes: self.expected(
                    0x5965190,
                    &[
                        0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53,
                        0x48, 0x81, 0xec, 0x38, 3, 0, 0,
                    ],
                ),
            },
            Entry {
                rva: self.rva(0x5954e80),
                bytes: self.expected(
                    0x5954e80,
                    &[
                        0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53,
                        0x48, 0x81, 0xec, 0xe8, 4, 0, 0,
                    ],
                ),
            },
            Entry {
                rva: self.rva(0x5955650),
                bytes: self.expected(0x5955650, &[0x48, 0x3b, 0x5d, 0x50, 0x0f, 0x95, 0xc0]),
            },
            Entry {
                rva: self.rva(0x5958134),
                bytes: self.expected(
                    0x5958134,
                    &[
                        0x48, 0x8b, 0x9d, 0xa0, 3, 0, 0, 0x48, 0x8b, 0x8b, 0, 1, 0, 0,
                    ],
                ),
            },
            Entry {
                rva: self.rva(0x5962610),
                bytes: self.expected(
                    0x5962610,
                    &[
                        0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48, 0x81,
                        0xec, 0xb0, 0, 0, 0,
                    ],
                ),
            },
        ]
    }
    pub fn combat_actor(self, mem: &dyn Mem, game: u64, base: u64) -> Result<CombatActor> {
        let actor = self.support_caster(mem, game)?;
        let header = mem.read(game + self.game_field(0x340), 16)?;
        if header.len() != 16 {
            return Err("Truncated combat abilities".into());
        }
        let (p, count) = (q(&header, 0), q(&header, 8));
        if count > 256 || (count > 0 && (p < 0x10000 || p.checked_add(count * 88).is_none())) {
            return Err("Invalid combat ability extent".into());
        }
        let target = mem.read(game + self.game_field(0x8f0), 8)?;
        let target = q(&target, 0);
        let mut mark = Mark::default();
        if count > 0 && target != 0 {
            let rows = mem.read(p, count as usize * 88)?;
            if rows.len() != count as usize * 88 {
                return Err("Truncated combat abilities".into());
            }
            for row in rows.chunks_exact(88) {
                let def = q(row, 0);
                if def < base || def.checked_sub(base).is_none_or(|v| v > 0x8000000 - 0x78) {
                    continue;
                }
                let id = mem.read(def + 0x40, 4)?;
                if d(&id, 0) != 1130 {
                    continue;
                }
                let power = f(row, 16) * f(row, 56);
                if !power.is_finite() || power < 0.0 {
                    return Err("Invalid native mark power".into());
                }
                let strength = (power.sqrt() * 0.2).min(0.4);
                if strength > 0.0 {
                    mark = Mark::new(Some(target), strength)?;
                }
                break;
            }
        }
        let header = mem.read(game + self.game_field(0x838), 16)?;
        if header.len() != 16 {
            return Err("Truncated tank talents".into());
        }
        let (p, count) = (q(&header, 0), q(&header, 8));
        if count > 512 || (count > 0 && (p < 0x10000 || p.checked_add(count * 64).is_none())) {
            return Err("Invalid tank talent extent".into());
        }
        let rows = if count > 0 {
            mem.read(p, count as usize * 64)?
        } else {
            vec![]
        };
        if rows.len() != count as usize * 64 {
            return Err("Truncated tank talents".into());
        }
        let mut talents = Talents::default();
        let (mut justice, mut consecration) = (false, false);
        for row in rows.chunks_exact(64) {
            let key = |at, wanted: &[u8]| -> Result<bool> {
                if q(row, at + 8) != wanted.len() as u64 {
                    return Ok(false);
                }
                let ptr = q(row, at);
                if ptr < 0x10000 {
                    return Err("Invalid talent key".into());
                }
                Ok(mem.read(ptr, wanted.len())? == wanted)
            };
            match row[0] {
                18 if f(row, 4).is_finite()
                    && f(row, 4) > 0.0
                    && f(row, 8).to_bits() == 0.3f32.to_bits() =>
                {
                    talents.fortified |= key(32, b"shield_slam")?;
                    consecration |= key(32, b"consecration")?;
                }
                27 => justice |= key(8, b"hammer_of_justice")?,
                25 if d(row, 4) == 9634 => talents.dire_bear_form = true,
                _ => {}
            }
        }
        talents.ardent_defender = justice && consecration;
        Ok(CombatActor {
            slot: actor.slot,
            guid: actor.guid,
            position: actor.position,
            mark,
            talents,
        })
    }
    pub(super) fn combat_rows(self, mem: &dyn Mem, game: u64) -> Result<(u64, Vec<u8>)> {
        let h = mem.read(game + self.game_field(0x570), 24)?;
        if h.len() != 24 {
            return Err("Truncated combat enemy header".into());
        }
        let (cap, p, n) = (q(&h, 0), q(&h, 8), q(&h, 16));
        let stride = crate::current::ENEMY_STRIDE;
        let extent = n.checked_mul(stride as u64);
        if n > cap
            || cap > 65536
            || (n > 0 && (p < 0x10000 || extent.is_none_or(|size| p.checked_add(size).is_none())))
        {
            return Err("Invalid combat enemy extent".into());
        }
        let byte_len =
            usize::try_from(extent.unwrap_or(0)).map_err(|_| "Combat enemy extent overflow")?;
        let rows = if n > 0 {
            mem.read(p, byte_len)?
        } else {
            vec![]
        };
        if rows.len() != byte_len {
            return Err("Truncated combat enemies".into());
        }
        Ok((p, rows))
    }
    pub fn combat_enemies(self, mem: &dyn Mem, game: u64) -> Result<Vec<Enemy>> {
        let (_, rows) = self.combat_rows(mem, game)?;
        let mut out = vec![];
        use crate::current as layout;
        for r in rows.chunks_exact(layout::ENEMY_STRIDE) {
            let id = q(r, layout::ENEMY_GUID);
            let hp = f(r, layout::ENEMY_HP);
            let value = f(r, layout::ENEMY_VULNERABILITY);
            if r[layout::ENEMY_DEAD] == 0
                && (!hp.is_finite() || !value.is_finite() || !(0.0..=0.400001).contains(&value))
            {
                return Err("Invalid native combat enemy values".into());
            }
            let a = &r[layout::ENEMY_AURAS_START..layout::ENEMY_AURAS_END];
            out.push(Enemy {
                id,
                alive: r[layout::ENEMY_DEAD] == 0 && hp > 0.0,
                marked: a.chunks_exact(4).any(|r| d(r, 0) == 1130),
                aura_full: a.chunks_exact(4).all(|r| d(r, 0) != 0),
                vulnerability: if value.is_finite() { value } else { 0.0 },
            });
        }
        Ok(out)
    }
    pub fn combat_hp(self, mem: &dyn Mem, game: u64, enemy: u64) -> Result<Option<f32>> {
        let (_, r) = self.combat_rows(mem, game)?;
        use crate::current as layout;
        Ok(r.chunks_exact(layout::ENEMY_STRIDE)
            .find(|r| q(r, layout::ENEMY_GUID) == enemy)
            .map(|r| f(r, layout::ENEMY_HP))
            .filter(|v| v.is_finite()))
    }
    pub fn write_combat_mark(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        enemy: u64,
        strength: f32,
    ) -> Result<()> {
        Mark::new((strength > 0.0).then_some(enemy), strength)?;
        let (p, rows) = self.combat_rows(mem, game)?;
        use crate::current as layout;
        if let Some((i, r)) = rows
            .chunks_exact(layout::ENEMY_STRIDE)
            .enumerate()
            .find(|(_, r)| q(r, layout::ENEMY_GUID) == enemy && r[layout::ENEMY_DEAD] == 0)
        {
            if f(r, layout::ENEMY_VULNERABILITY) != strength {
                mem.write(
                    p + i as u64 * layout::ENEMY_STRIDE as u64 + layout::ENEMY_VULNERABILITY as u64,
                    &strength.to_le_bytes(),
                )?;
            }
        }
        Ok(())
    }
    fn basic_fields(self) -> [u64; 5] {
        [
            crate::current::BASIC_GAME,
            crate::current::BASIC_ENEMY,
            crate::current::BASIC_POSITION,
            crate::current::BASIC_ORIGIN_XY,
            crate::current::BASIC_ORIGIN_Z,
        ]
    }
    pub fn basic_frame(self, mem: &dyn Mem, frame: u64) -> Result<BasicFrame> {
        if frame < 0x10000 || frame.checked_add(self.basic_fields()[1] + 8).is_none() {
            return Err("Invalid basic action frame".into());
        }
        let game = q(&mem.read(frame + self.basic_fields()[0], 8)?, 0);
        let row = q(&mem.read(frame + self.basic_fields()[1], 8)?, 0);
        let enemy = q(&mem.read(row + crate::current::ENEMY_GUID as u64, 8)?, 0);
        let b = mem.read(frame + self.basic_fields()[2], 12)?;
        let position = std::array::from_fn(|i| f(&b, i * 4));
        if position.iter().any(|v| !v.is_finite()) {
            return Err("Invalid basic enemy pose".into());
        }
        Ok(BasicFrame {
            game,
            enemy,
            frame,
            position,
        })
    }
    pub fn restore_basic_origin(
        self,
        mem: &dyn MutableMemory,
        frame: u64,
        game: u64,
        pose: [f32; 3],
    ) -> Result<()> {
        if q(&mem.read(frame + self.basic_fields()[0], 8)?, 0) != game
            || pose.iter().any(|v| !v.is_finite() || v.abs() > 1e6)
        {
            return Err("Basic origin identity changed".into());
        }
        let bytes = pose
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        mem.write(frame + self.basic_fields()[3], &bytes[..8])?;
        mem.write(frame + self.basic_fields()[4], &bytes[8..])
    }
    pub fn prepare_basic_target(
        self,
        mem: &dyn MutableMemory,
        frame: &BasicFrame,
        pose: [f32; 3],
    ) -> Result<f32> {
        self.restore_basic_origin(mem, frame.frame, frame.game, pose)?;
        let distance =
            ((frame.position[0] - pose[0]).powi(2) + (frame.position[1] - pose[1]).powi(2)).sqrt();
        if !distance.is_finite() {
            return Err("Invalid basic target distance".into());
        }
        Ok(distance)
    }
    pub fn hurt_actor(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        guid: u64,
        amount: f32,
        enemy: u64,
        melee: bool,
    ) -> Result<Option<f32>> {
        if !amount.is_finite()
            || !(0.0..=1e12).contains(&amount)
            || io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes()
        {
            return Err("Invalid routed native hurt".into());
        }
        let row = self.support_row(io, game, slot)?;
        if io.read(row + self.row_field(row == game, 0x478), 8)? != guid.to_le_bytes()
            || io.read(
                row + self.row_field(slot == 0, if slot == 0 { 0xc46 } else { 0x97c }),
                1,
            )? != [1]
        {
            return Ok(None);
        }
        if slot != 0 {
            io.swap_actor(game, row)?;
            io.write(
                game + self.game_field(0xb40),
                &u64::from(slot).to_le_bytes(),
            )?;
        }
        let result = (|| {
            let before = f(&io.read(game + self.game_field(0x4f8), 8)?, 0)
                + f(&io.read(game + self.game_field(0x4fc), 4)?, 0);
            io.support_hurt(game, amount, enemy, melee)?;
            let after = f(&io.read(game + self.game_field(0x4f8), 8)?, 0)
                + f(&io.read(game + self.game_field(0x4fc), 4)?, 0);
            io.flush_actor(game)?;
            Ok(Some((before - after).max(0.0)))
        })();
        if slot != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result
    }
    pub fn fixture_combat_enemy(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        avoid_slot: u32,
    ) -> Result<Option<u64>> {
        let (p, rows) = self.combat_rows(mem, game)?;
        use crate::current as layout;
        let selected = rows
            .chunks_exact(layout::ENEMY_STRIDE)
            .enumerate()
            .find(|(_, r)| {
                r[layout::ENEMY_DEAD] == 0
                    && f(r, layout::ENEMY_HP) > 0.0
                    && q(r, 0x18) == u64::MAX
                    && q(r, 0x58) == u64::MAX
                    && q(r, layout::ENEMY_TARGET) != u64::from(avoid_slot)
                    && (q(r, 0x100) == 0 || mem.read(q(r, 0x100), 1).is_ok_and(|v| v[0] & 1 == 0))
            });
        let Some((i, r)) = selected else {
            return Err(format!(
                "No melee candidate: {:?}",
                rows.chunks_exact(layout::ENEMY_STRIDE)
                    .take(8)
                    .map(|r| (
                        r[layout::ENEMY_DEAD],
                        q(r, 0x18),
                        q(r, 0x58),
                        q(r, layout::ENEMY_TARGET),
                        q(r, 0x100)
                    ))
                    .collect::<Vec<_>>()
            ));
        };
        let guid = q(r, layout::ENEMY_GUID);
        let a = p + i as u64 * layout::ENEMY_STRIDE as u64;
        mem.write(a + layout::ENEMY_HP as u64, &1e6f32.to_le_bytes())?;
        mem.write(a + layout::ENEMY_HP as u64 + 4, &1e6f32.to_le_bytes())?;
        Ok(Some(guid))
    }
    pub fn attack_actor(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        enemy: u64,
        spell: u32,
        amount: f32,
    ) -> Result<()> {
        if io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Attack fixture requires restored host".into());
        }
        let row = self.support_row(io, game, slot)?;
        if slot != 0 {
            io.swap_actor(game, row)?;
            io.write(
                game + self.game_field(0xb40),
                &u64::from(slot).to_le_bytes(),
            )?;
        }
        let result = io
            .support_hit(game, enemy, amount, spell)
            .and_then(|_| io.flush_actor(game));
        if slot != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result
    }
}

// Native continuation owns the selected build's nonvolatile distance register.
use std::sync::atomic::AtomicUsize;
static BASIC_CONTINUATION: AtomicUsize = AtomicUsize::new(0);
#[unsafe(naked)]
unsafe extern "C" fn basic_resume() {
    core::arch::naked_asm!("test rax,rax","jz 2f","movss xmm15,xmm5","2:","jmp qword ptr [rip+{original}]",original=sym BASIC_CONTINUATION);
}
impl Adapter {
    pub fn basic_resume_entry(self) -> (usize, &'static AtomicUsize) {
        (basic_resume as *const () as usize, &BASIC_CONTINUATION)
    }
}
#[cfg(test)]
mod resume_tests {
    use super::*;
    use std::sync::atomic::Ordering;
    #[unsafe(naked)]
    unsafe extern "C" fn done() {
        core::arch::naked_asm!("ret");
    }
    static DISTANCE_BITS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    #[unsafe(naked)]
    unsafe extern "win64" fn probe(_flag: u64, _old: f32, _new: f32) -> u64 {
        core::arch::naked_asm!("sub rsp,104","movaps [rsp+32],xmm12","movaps [rsp+48],xmm14","movaps [rsp+64],xmm15","movaps [rsp+80],xmm1","movaps xmm12,xmm1","movaps xmm14,xmm1","movaps xmm15,xmm1","movaps xmm5,xmm2","mov rax,rcx","call {gate}","movd eax,xmm15","mov dword ptr [rip+{distance}],eax","movdqa xmm0,xmm12","pcmpeqb xmm0,[rsp+80]","pmovmskb eax,xmm0","cmp eax,0xffff","jne 2f","movdqa xmm0,xmm14","pcmpeqb xmm0,[rsp+80]","pmovmskb eax,xmm0","cmp eax,0xffff","sete al","movzx eax,al","jmp 3f","2:","xor eax,eax","3:","movaps xmm12,[rsp+32]","movaps xmm14,[rsp+48]","movaps xmm15,[rsp+64]","add rsp,104","ret",gate=sym basic_resume,distance=sym DISTANCE_BITS);
    }
    #[test]
    fn current_native_distance_register_is_selected_and_other_nonvolatiles_survive() {
        let old = BASIC_CONTINUATION.swap(done as *const () as usize, Ordering::SeqCst);
        let native = unsafe { probe(0, 3., 7.) };
        let native_bits = DISTANCE_BITS.load(Ordering::SeqCst);
        let tank = unsafe { probe(1, 3., 7.) };
        let tank_bits = DISTANCE_BITS.load(Ordering::SeqCst);
        BASIC_CONTINUATION.store(old, Ordering::SeqCst);
        assert_eq!((native, tank), (1, 1));
        assert_eq!(
            (f32::from_bits(native_bits), f32::from_bits(tank_bits)),
            (3., 7.)
        );
    }
}
