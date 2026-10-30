//! Native aura storage remains opaque to the coordinator. All rows are copied
//! and rebound after native callbacks; no native pointer survives a call.
use super::*;
#[derive(Clone, Copy, Debug, Default)]
pub struct SupportTalents {
    pub overheal_cap: f32,
    pub nature_coefficient: f32,
    pub nature_radius: f32,
    pub maximum: f32,
}
#[derive(Clone, Debug)]
pub struct SupportReceipt {
    pub caster: u32,
    pub caster_guid: u64,
    pub target_guid: u64,
    pub spell: u32,
    pub raw: f32,
    pub effective: f32,
    pub origin: [f32; 3],
    pub talents: SupportTalents,
}
#[derive(Clone, Debug)]
pub struct BreakFrame {
    pub game: u64,
    pub spell: u32,
    pub amount: f32,
}

#[derive(Clone, Debug)]
pub struct PerkLoan {
    header: Vec<u8>,
    storage: Vec<u8>,
    cooldown_header: Vec<u8>,
    cooldowns: Vec<f32>,
    original_cooldowns: usize,
    rapture_index: Option<usize>,
}
#[derive(Clone, Debug)]
pub struct AuraState {
    pub(super) spell: u32,
    pub(super) rows: Vec<[u8; 32]>,
    pub(super) shield: [u8; 12],
    pub(super) absorb: [u8; 8],
}
#[derive(Clone, Debug)]
pub struct OwnedEffect {
    pub caster: u32,
    pub caster_guid: u64,
    pub target_guid: u64,
    pub spell: u32,
    pub expires: f32,
    pub shield: bool,
}
#[derive(Clone, Debug)]
pub struct PeriodicTick {
    pub owner: OwnedEffect,
    pub raw: f32,
    pub(super) remaining: f32,
}
pub(super) fn u32at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
pub(super) fn f32at(b: &[u8], at: usize) -> f32 {
    f32::from_bits(u32at(b, at))
}
impl Adapter {
    pub fn fixture_perk_loan(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        base: u64,
        rapture: bool,
        next_allowed: f32,
    ) -> Result<PerkLoan> {
        let header = mem.read(game + self.game_field(0x830), 24)?;
        if header.len() != 24 {
            return Err("Truncated fixture perks".into());
        }
        let at = |n| u64::from_le_bytes(header[n..n + 8].try_into().unwrap());
        let (cap, p, count) = (at(0), at(8), at(16));
        if count > cap
            || cap > 512
            || (count > 0 && (p < 0x10000 || p.checked_add(count * 64).is_none()))
        {
            return Err("Invalid fixture perks".into());
        }
        let mut storage = if count > 0 {
            mem.read(p, count as usize * 64)?
        } else {
            vec![]
        };
        let mut perk = [0u8; 64];
        perk[0] = 3;
        perk[1] = 1;
        perk[4..8].copy_from_slice(&0.5f32.to_le_bytes());
        perk[8..12].copy_from_slice(&10f32.to_le_bytes());
        perk[12..16].copy_from_slice(&17u32.to_le_bytes());
        // Reviewed native condition variant 10 matches event kind 6 (shield break).
        perk[16..20].copy_from_slice(&10u32.to_le_bytes());
        storage.extend_from_slice(&perk);
        if rapture {
            let perk = mem.read(base + self.rva(0x701a618), 64)?;
            if perk.len() != 64
                || u32at(&perk, 0) != 0
                || f32at(&perk, 8) != 15.0
                || u32at(&perk, 32) != 10
            {
                return Err("Unexpected native Rapture definition".into());
            }
            storage.extend_from_slice(&perk);
        }
        let count = count + u64::from(rapture);
        let mut h = [0u8; 24];
        h[..8].copy_from_slice(&(count + 1).to_le_bytes());
        h[8..16].copy_from_slice(&(storage.as_ptr() as u64).to_le_bytes());
        h[16..].copy_from_slice(&(count + 1).to_le_bytes());
        let cooldown_header = mem.read(game + self.game_field(0x68), 24)?;
        if cooldown_header.len() != 24 {
            return Err("Truncated native proc cooldown vector".into());
        }
        let at = |n| u64::from_le_bytes(cooldown_header[n..n + 8].try_into().unwrap());
        let (ccap, cp, clen) = (at(0), at(8), at(16));
        if clen > ccap
            || ccap > 512
            || (clen > 0 && (cp < 0x10000 || cp.checked_add(clen * 4).is_none()))
        {
            return Err("Invalid native proc cooldown vector".into());
        }
        let bytes = if clen > 0 {
            mem.read(cp, clen as usize * 4)?
        } else {
            vec![]
        };
        let mut cooldowns = bytes
            .chunks_exact(4)
            .map(|b| f32at(b, 0))
            .collect::<Vec<_>>();
        cooldowns.resize(cooldowns.len().max(count as usize + 1), 0.0);
        let rapture_index = rapture.then_some(count as usize);
        if let Some(index) = rapture_index {
            if !next_allowed.is_finite() {
                return Err("Invalid fixture proc throttle".into());
            }
            cooldowns[index] = next_allowed;
        }
        let mut ch = [0u8; 24];
        let len = cooldowns.len() as u64;
        ch[..8].copy_from_slice(&len.to_le_bytes());
        ch[8..16].copy_from_slice(&(cooldowns.as_mut_ptr() as u64).to_le_bytes());
        ch[16..].copy_from_slice(&len.to_le_bytes());
        mem.write(game + self.game_field(0x68), &ch)?;
        mem.write(game + self.game_field(0x830), &h)?;
        Ok(PerkLoan {
            header,
            storage,
            cooldown_header,
            cooldowns,
            original_cooldowns: clen as usize,
            rapture_index,
        })
    }
    pub fn fixture_restore_perks(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        loan: PerkLoan,
    ) -> Result<Option<f32>> {
        // The native proc clones POD perks at entry; it never owns this borrowed
        // fixture storage. Restore before Rust drops it.
        if mem.read(game + self.game_field(0x838), 8)?
            != (loan.storage.as_ptr() as u64).to_le_bytes()
        {
            return Err("Native fixture perk header changed".into());
        }
        let next = loan.rapture_index.map(|i| loan.cooldowns[i]);
        let original_pointer = u64::from_le_bytes(loan.cooldown_header[8..16].try_into().unwrap());
        if loan.original_cooldowns > 0 {
            let bytes = loan.cooldowns[..loan.original_cooldowns]
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<_>>();
            mem.write(original_pointer, &bytes)?;
        }
        mem.write(game + self.game_field(0x68), &loan.cooldown_header)?;
        mem.write(game + self.game_field(0x830), &loan.header)?;
        Ok(next)
    }
    pub fn hurt_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x5908cc0),
            bytes: self.expected(
                0x5908cc0,
                &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x81, 0xec, 0xa8, 1, 0, 0,
                ],
            ),
        }
    }
    pub fn fixture_break_shield(self, io: &dyn SupportCalls, game: u64, slot: u32) -> Result<bool> {
        if io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Fixture hurt requires restored host".into());
        }
        let row = self.support_row(io, game, slot)?;
        if slot != 0 {
            io.swap_actor(game, row)?;
            io.write(
                game + self.game_field(0xb40),
                &u64::from(slot).to_le_bytes(),
            )?;
        }
        let result = (|| {
            let shield = io.read(game + self.game_field(0x4fc), 4)?;
            let shield = f32at(&shield, 0);
            if !shield.is_finite() || shield <= 0.0 {
                return Err(format!("Fixture shield absent before hurt: amount={shield} state={:?}", io.read(game + self.game_field(0x480), 8)?));
            }
            let maximum = io.read(game + self.game_field(0x490), 4)?;
            io.write(game + self.game_field(0x4f8), &maximum)?;
            let reduction = io.read(game + self.game_field(0x4b0), 4)?;
            let reduction = f32at(&reduction, 0);
            if !reduction.is_finite() || !(0.0..0.999).contains(&reduction) {
                return Err("Fixture cannot break an invulnerable shield".into());
            }
            // Follow actual native absorption rather than assuming one mitigation
            // coefficient covers armor, talents and racials. Each additional hit
            // requires confirmed progress; no protection is removed or bypassed.
            let mut remaining=shield;
            for _ in 0..8 {
                io.support_hurt(game,(remaining+1.0)/(1.0-reduction),0,false)?;
                io.flush_actor(game)?;
                let id=io.read(game+self.game_field(0x480),4)?;
                let left=io.read(game+self.game_field(0x484),4)?;
                if u32at(&id,0)==0 && f32at(&left,0)<=0.0{return Ok(true);}
                let next=f32at(&io.read(game+self.game_field(0x4fc),4)?,0);
                if !next.is_finite() || next>=remaining-0.001 {
                    return Err(format!("Fixture shield made no native progress: before={remaining} after={next}"));
                }
                remaining=next;
            }
            Err("Fixture shield exceeded bounded native-hit budget".into())
        })();
        if slot != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result
    }
    pub fn elite_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x595066a),
            bytes: self.expected(0x595066a, &[0x48, 0xf, 0x45, 0xc1, 0xff, 0x0]),
        }
    }
    pub fn elite_game(self, mem: &dyn Mem, registers: [u64; 3]) -> Result<Option<u64>> {
        // The reviewed selected site carries Game in RAX and branches before the
        // cooperative elite counter is incremented.
        let game = registers[0];
        if game < 0x10000 || mem.read(game + self.game_field(0xc41), 1)? != [1] {
            return Ok(None);
        }
        Ok(Some(game))
    }
    /// 0.120 already distributes one native guardian item to every member.
    pub fn needs_guardian_item_supplement(self) -> bool {
        false
    }
    pub fn deliver_elite_item(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        slot: u32,
        guid: u64,
    ) -> Result<bool> {
        if mem.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Elite rewards require restored host".into());
        }
        let row = self.support_row(mem, game, slot)?;
        if mem.read(row + self.row_field(row == game, 0x478), 8)? != guid.to_le_bytes() {
            return Ok(false);
        }
        let phase = mem.read(
            row + self.row_field(slot == 0, if slot == 0 { 0xc46 } else { 0x97c }),
            1,
        )?;
        if !matches!(phase.as_slice(), [1] | [2] | [3]) {
            return Ok(false);
        }
        let address = row + self.row_field(slot == 0, if slot == 0 { 0xbb4 } else { 0x92c });
        let bytes = mem.read(address, 4)?;
        let next = u32at(&bytes, 0)
            .checked_add(1)
            .ok_or("Elite item reward overflow")?;
        mem.write(address, &next.to_le_bytes())?;
        Ok(true)
    }
    pub fn fixture_guardian(self, io: &dyn SupportCalls, game: u64) -> Result<Option<u64>> {
        let Some((guid, hp)) = self.fixture_guardian_target(io, game)? else {
            return Ok(None);
        };
        io.support_damage(game, guid, hp * 100.0 + 1e6)?;
        Ok(Some(guid))
    }
    pub fn fixture_guardian_target(
        self,
        io: &dyn SupportCalls,
        game: u64,
    ) -> Result<Option<(u64, f32)>> {
        let h = io.read(game + self.game_field(0x570), 24)?;
        if h.len() != 24 {
            return Err("Truncated guardian vector".into());
        }
        let at = |n| u64::from_le_bytes(h[n..n + 8].try_into().unwrap());
        let (cap, p, count) = (at(0), at(8), at(16));
        use crate::current as layout;
        let stride = layout::ENEMY_STRIDE;
        let extent = count.checked_mul(stride as u64);
        if count > cap
            || cap > 65536
            || (count > 0
                && (p < 0x10000 || extent.is_none_or(|size| p.checked_add(size).is_none())))
        {
            return Err("Invalid guardian vector".into());
        }
        for i in 0..count {
            let r = io.read(p + i * stride as u64, stride)?;
            if r.len() != stride {
                return Err("Truncated guardian".into());
            }
            if u32at(&r, layout::ENEMY_DEAD) != 1
                && (u64::from_le_bytes(r[0x18..0x20].try_into().unwrap()) == u64::MAX
                    || u64::from_le_bytes(r[0x30..0x38].try_into().unwrap()) == 0)
                && u64::from_le_bytes(r[0x58..0x60].try_into().unwrap()) != u64::MAX
            {
                let hp = f32at(&r, layout::ENEMY_HP);
                if !hp.is_finite() || !(0.0..1e10).contains(&hp) {
                    continue;
                }
                let guid = u64::from_le_bytes(
                    r[layout::ENEMY_GUID..layout::ENEMY_GUID + 8]
                        .try_into()
                        .unwrap(),
                );
                return Ok(Some((guid, hp)));
            }
        }
        Ok(None)
    }
    pub fn support_query_origin(
        self,
        mem: &dyn MutableMemory,
        position: u64,
        stack: u64,
        return_rva: u64,
        origin: [f32; 3],
    ) -> Result<bool> {
        if self.baseline_rva(return_rva) != 0x5944de9 {
            return Ok(false);
        }
        if origin.iter().any(|v| !v.is_finite())
            || position < stack.saturating_add(40)
            || position
                .checked_add(12)
                .is_none_or(|end| end > stack.saturating_add(8192))
        {
            return Err("Invalid native proc origin storage".into());
        }
        let bytes = origin
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        mem.write(position, &bytes)?;
        Ok(true)
    }
    pub fn support_proc_entries(self) -> [Entry; 4] {
        [
            Entry {
                rva: self.rva(0x594edf0),
                bytes: self.expected(
                    0x594edf0,
                    &[
                        0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53,
                        0x48, 0x81, 0xec, 0xb8, 2, 0, 0,
                    ],
                ),
            },
            Entry {
                rva: self.rva(0x59436a0),
                bytes: self.expected(
                    0x59436a0,
                    &[
                        0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53,
                        0x48, 0x81, 0xec, 0x38, 3, 0, 0,
                    ],
                ),
            },
            Entry {
                rva: self.rva(0x5938db0),
                bytes: self.expected(
                    0x5938db0,
                    &[0x55, 0x56, 0x57, 0x48, 0x81, 0xec, 0x90, 0, 0, 0],
                ),
            },
            Entry {
                rva: self.rva(0x59090c5),
                bytes: self.expected(0x59090c5, &[0x49, 0xc7, 0x86, 0x90, 1, 0, 0, 0, 0, 0, 0]),
            },
        ]
    }
    pub fn support_break_frame(self, registers: [u64; 15], values: [f32; 2]) -> Result<BreakFrame> {
        if values.iter().any(|v| !v.is_finite() || v.abs() > 1e12) {
            return Err("Invalid shield break values".into());
        }
        Ok(BreakFrame {
            game: registers[crate::current::SHIELD_BREAK_GAME_REGISTER],
            spell: registers[crate::current::SHIELD_BREAK_SPELL_REGISTER] as u32,
            amount: values[0].min(values[1]).max(0.0),
        })
    }
    pub fn support_overheal(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        talents: SupportTalents,
        raw: f32,
        effective: f32,
    ) -> Result<()> {
        let extra = (raw - effective).max(0.0);
        if extra == 0.0 || talents.overheal_cap <= 0.0 {
            return Ok(());
        }
        let bytes = mem.read(game + self.game_field(0x4fc), 4)?;
        let old = f32at(&bytes, 0);
        let cap = talents.overheal_cap * talents.maximum;
        if !old.is_finite() || !cap.is_finite() || cap > 1e12 {
            return Err("Invalid overheal shield bounds".into());
        }
        mem.write(
            game + self.game_field(0x4fc),
            &old.max((old + extra).min(cap)).to_le_bytes(),
        )
    }
    pub fn support_receipt(
        self,
        io: &dyn SupportCalls,
        game: u64,
        receipt: &SupportReceipt,
    ) -> Result<Option<usize>> {
        if !receipt.raw.is_finite()
            || !(0.0..=1e12).contains(&receipt.raw)
            || receipt.origin.iter().any(|v| !v.is_finite())
        {
            return Err("Invalid support talent receipt".into());
        }
        if io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Support talent requires restored host".into());
        }
        let row = self.support_row(io, game, receipt.caster)?;
        let guid = io.read(row + self.row_field(row == game, 0x478), 8)?;
        if guid != receipt.caster_guid.to_le_bytes() {
            return Err("Support talent source changed".into());
        }
        if receipt.caster != 0 {
            io.swap_actor(game, row)?;
            io.write(
                game + self.game_field(0xb40),
                &u64::from(receipt.caster).to_le_bytes(),
            )?;
        }
        let result = (|| {
            let clock = io.read(game + self.game_field(0xb9c), 4)?;
            let clock = f32at(&clock, 0);
            let cooldown = io.read(game + self.game_field(0x1a4), 4)?;
            let cooldown = f32at(&cooldown, 0);
            if !clock.is_finite() || !cooldown.is_finite() {
                return Err("Invalid Nature throttle".into());
            }
            if receipt.talents.nature_coefficient <= 0.0 {
                return Ok(Some(0));
            }
            if receipt.raw < 1.0 || clock < cooldown {
                return Ok(None);
            }
            let amount = receipt.raw * receipt.talents.nature_coefficient;
            if !amount.is_finite() || amount > 1e12 {
                return Err("Nature burst overflow".into());
            }
            let enemies = self.support_enemies(io, game)?;
            let radius2 = receipt.talents.nature_radius.powi(2);
            io.write(game + self.game_field(0x1a4), &(clock + 1.0).to_le_bytes())?;
            let guard = io.read(game + self.game_field(0x1ad), 1)?;
            io.write(game + self.game_field(0x1ad), &[3])?;
            let mut hits = 0;
            let damage_result = (|| {
                for enemy in enemies {
                    let distance = enemy
                        .position
                        .iter()
                        .zip(receipt.origin)
                        .map(|(a, b)| (a - f64::from(b)).powi(2))
                        .sum::<f64>();
                    if enemy.hp > 0.0 && distance <= f64::from(radius2) {
                        io.support_damage(game, enemy.guid, amount)?;
                        hits += 1;
                    }
                }
                Ok::<(), String>(())
            })();
            io.write(game + self.game_field(0x1ad), &guard)?;
            damage_result?;
            io.flush_actor(game)?;
            Ok(Some(hits))
        })();
        if receipt.caster != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result
    }
    pub fn support_shield_break(
        self,
        io: &dyn SupportCalls,
        game: u64,
        owner: &OwnedEffect,
        origin: [f32; 3],
        amount: f32,
    ) -> Result<()> {
        if !amount.is_finite()
            || !(0.0..=1e12).contains(&amount)
            || origin.iter().any(|v| !v.is_finite())
        {
            return Err("Invalid shield proc".into());
        }
        if io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Shield proc requires restored host".into());
        }
        let row = self.support_row(io, game, owner.caster)?;
        if io.read(row + self.row_field(row == game, 0x478), 8)? != owner.caster_guid.to_le_bytes()
        {
            return Err("Shield source changed".into());
        }
        if owner.caster != 0 {
            io.swap_actor(game, row)?;
            io.write(
                game + self.game_field(0xb40),
                &u64::from(owner.caster).to_le_bytes(),
            )?;
        }
        let mut event = [0u8; 40];
        for (i, v) in origin.iter().enumerate() {
            event[12 + i * 4..16 + i * 4].copy_from_slice(&v.to_le_bytes());
        }
        event[24..28].copy_from_slice(&amount.to_le_bytes());
        event[28..32].copy_from_slice(&0x600u32.to_le_bytes());
        let result = io
            .shield_proc(game, &event, origin, owner.target_guid)
            .and_then(|_| io.flush_actor(game));
        if owner.caster != 0 {
            let row = self.support_row(io, game, 0)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
            io.swap_actor(game, row)?;
        }
        result
    }
    pub fn support_enemies(
        self,
        mem: &dyn Mem,
        game: u64,
    ) -> Result<Vec<crate::enemies::EnemyExact>> {
        let h = mem.read(game + self.game_field(0x570), 24)?;
        if h.len() != 24 {
            return Err("Truncated support enemies".into());
        }
        let at = |n| u64::from_le_bytes(h[n..n + 8].try_into().unwrap());
        let (cap, pointer, count) = (at(0), at(8), at(16));
        use crate::current as layout;
        let stride = layout::ENEMY_STRIDE;
        let extent = count.checked_mul(stride as u64);
        if count > cap
            || cap > 65536
            || (count > 0
                && (pointer < 0x10000
                    || extent.is_none_or(|size| pointer.checked_add(size).is_none())))
        {
            return Err("Invalid support enemy extent".into());
        }
        let clock = mem.read(game + self.game_field(0xb9c), 4)?;
        let clock = f32at(&clock, 0);
        let byte_len =
            usize::try_from(extent.unwrap_or(0)).map_err(|_| "Support enemy extent overflow")?;
        let bytes = if count > 0 {
            mem.read(pointer, byte_len)?
        } else {
            vec![]
        };
        Ok(crate::enemies::parse_enemies_layout(
            &bytes,
            clock,
            crate::enemies::EnemyLayout {
                stride: stride as u64,
                guid_offset: layout::ENEMY_GUID as u64,
                guid_gap: (layout::ENEMY_GUID - layout::ENEMY_DEAD - 4) as u64,
                hp_gap: (layout::ENEMY_HP - (layout::ENEMY_GUID + 8)) as u64,
            },
        )?
        .0)
    }
    pub fn periodic_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x5906150),
            bytes: self.expected(
                0x5906150,
                &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x81, 0xec, 0xc8, 1, 0, 0,
                ],
            ),
        }
    }
    pub(super) fn aura_stride(self) -> u64 {
        32
    }
    /// Canonical rows keep the legacy effect/spell/expiry/budget prefix and
    /// retain the new native optional producer GUID (zero means recipient) in the final eight bytes.
    pub(super) fn aura_field(self, canonical: u64) -> u64 {
        canonical + 8
    }
    pub(in crate::official_native) fn aura_rows(
        self,
        mem: &dyn Mem,
        game: u64,
    ) -> Result<(u64, Vec<[u8; 32]>)> {
        let h = mem.read(game + self.game_field(0x398), 24)?;
        if h.len() != 24 {
            return Err("Truncated aura vector".into());
        }
        let q = |i| u64::from_le_bytes(h[i..i + 8].try_into().unwrap());
        let (cap, pointer, count) = (q(0), q(8), q(16));
        let stride = self.aura_stride();
        if count > cap
            || cap > 4096
            || (count > 0 && (pointer < 0x10000 || pointer.checked_add(count * stride).is_none()))
        {
            return Err("Invalid aura extent".into());
        }
        let bytes = if count > 0 {
            mem.read(pointer, (count * stride) as usize)?
        } else {
            vec![]
        };
        if bytes.len() != count as usize * stride as usize {
            return Err("Truncated aura rows".into());
        }
        let rows = bytes
            .chunks_exact(stride as usize)
            .map(|raw| {
                let mut row = [0; 32];
                if stride == 32 {
                    row[..24].copy_from_slice(&raw[8..32]);
                    row[24..].copy_from_slice(&raw[..8]);
                } else {
                    row[..24].copy_from_slice(raw);
                }
                row
            })
            .collect();
        Ok((pointer, rows))
    }
    pub(super) fn write_aura_rows(
        self,
        mem: &dyn MutableMemory,
        pointer: u64,
        rows: &[[u8; 32]],
    ) -> Result<()> {
        let mut bytes = Vec::with_capacity(rows.len() * self.aura_stride() as usize);
        for row in rows {
            if self.aura_stride() == 32 {
                bytes.extend_from_slice(&row[24..]);
            }
            bytes.extend_from_slice(&row[..24]);
        }
        if bytes.is_empty() {
            Ok(())
        } else {
            mem.write(pointer, &bytes)
        }
    }
    fn aura_has_caster(self, row: &[u8], spell: u32, guid: u64, recipient: u64) -> bool {
        if spell != 774 {
            return true;
        }
        let producer = u64::from_le_bytes(row[24..32].try_into().unwrap());
        producer == guid || (producer == 0 && guid == recipient)
    }
    pub fn support_active(self, mem: &dyn Mem, game: u64) -> Result<asx_gameplay::support::Active> {
        let clock = mem.read(game + self.game_field(0xb9c), 4)?;
        let clock = f32at(&clock, 0);
        let (_, rows) = self.aura_rows(mem, game)?;
        let has = |spell| {
            rows.iter()
                .any(|r| u32at(r, 12) == spell && f32at(r, 16) > clock)
        };
        Ok(asx_gameplay::support::Active {
            renew: has(139),
            rejuvenation: has(774),
            shield: has(17),
            lightning: has(324),
            thorns: has(467),
            ..Default::default()
        })
    }
    /// Native Rejuvenation eligibility is per producer, not per spell.
    pub fn rejuvenation_source_slots(
        self,
        mem: &dyn Mem,
        game: u64,
        caster: u64,
    ) -> Result<Option<Vec<u32>>> {
        // Casts can run while a guest is swapped into Game; do not request the
        // restored-host party snapshot here. support_row translates that scope.
        let count = u64::from_le_bytes(
            mem.read(game + self.game_field(0x988), 8)?
                .try_into()
                .map_err(|_| "Truncated source-aware party count")?,
        );
        if count > 4 {
            return Err("Invalid source-aware party count".into());
        }
        let clock = f32at(&mem.read(game + self.game_field(0xb9c), 4)?, 0);
        if !clock.is_finite() {
            return Err("Invalid source-aware clock".into());
        }
        let mut slots = vec![];
        for slot in 0..=count as u32 {
            let row = self.support_row(mem, game, slot)?;
            let recipient = u64::from_le_bytes(
                mem.read(row + self.row_field(row == game, 0x478), 8)?
                    .try_into()
                    .map_err(|_| "Truncated source-aware identity")?,
            );
            let (_, auras) = self.aura_rows(mem, row)?;
            if auras.iter().any(|r| {
                u32at(r, 12) == 774
                    && f32at(r, 16) > clock
                    && self.aura_has_caster(r, 774, caster, recipient)
            }) {
                slots.push(slot);
            }
        }
        Ok(Some(slots))
    }
    /// Clear only this source's own support aura for a bounded coefficient probe.
    pub fn glyph_fixture_support_clear(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        spell: u32,
    ) -> Result<AuraState> {
        if !self.supports_extensions()
            || !matches!(spell, 17 | 139 | 324 | 11426 | 18708 | 774 | 467)
        {
            return Err("Unsupported glyph support probe".into());
        }
        let saved = self.capture_aura(mem, game, spell)?;
        if matches!(spell, 17 | 11426) {
            self.prepare_foreign_shield(mem, game, &saved)?;
        } else {
            let (pointer, rows) = self.aura_rows(mem, game)?;
            let rows = rows
                .into_iter()
                .filter(|r| u32at(r, 12) != spell)
                .collect::<Vec<_>>();
            if !rows.is_empty() {
                self.write_aura_rows(mem, pointer, &rows)?;
            }
            mem.write(
                game + self.game_field(0x3a8),
                &(rows.len() as u64).to_le_bytes(),
            )?;
        }
        Ok(saved)
    }
    pub fn glyph_fixture_support_value(
        self,
        mem: &dyn Mem,
        game: u64,
        spell: u32,
    ) -> Result<(f32, f32)> {
        if !self.supports_extensions() {
            return Err("Unsupported glyph support probe".into());
        }
        if matches!(spell, 17 | 11426) {
            let power = f32at(&mem.read(game + self.game_field(0x484), 4)?, 0);
            if !power.is_finite() || power < 0. {
                return Err("Invalid glyph shield power".into());
            }
            let duration = if spell == 11426 {
                let clock = f32at(&mem.read(game + self.game_field(0xb9c), 4)?, 0);
                let (_, rows) = self.aura_rows(mem, game)?;
                let row = rows
                    .iter()
                    .find(|r| u32at(*r, 12) == spell)
                    .ok_or("Missing Ice Barrier aura")?;
                f32at(row, 16) - clock
            } else {
                0.
            };
            return Ok((power, duration));
        }
        if !matches!(spell, 139 | 774) {
            return Err("Unsupported glyph support value".into());
        }
        let clock = f32at(&mem.read(game + self.game_field(0xb9c), 4)?, 0);
        let (_, rows) = self.aura_rows(mem, game)?;
        let row = rows
            .iter()
            .find(|r| u32at(*r, 0) == 6 && u32at(*r, 12) == spell)
            .ok_or("Missing native periodic heal")?;
        let power = f32at(row, 20);
        let duration = f32at(row, 16) - clock;
        if !power.is_finite() || power < 0. || !duration.is_finite() || duration <= 0. {
            return Err("Invalid glyph Renew budget".into());
        }
        Ok((power, duration))
    }
    /// Bounded glyph diagnostic readback; do not broaden retaliation decoding.
    pub fn glyph_fixture_buff(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<NativeEffect> {
        let kind = match spell {
            13750 => 9,
            5171 => 8,
            18708 => 10,
            _ => return Err("Unsupported glyph buff probe".into()),
        };
        if !self.supports_extensions() {
            return Err("Glyph buff probe requires reviewed client".into());
        }
        let clock = f32at(&mem.read(game + self.game_field(0xb9c), 4)?, 0);
        let (_, rows) = self.aura_rows(mem, game)?;
        let row = rows
            .iter()
            .find(|r| u32at(*r, 12) == spell && u32at(*r, 0) == kind)
            .ok_or("Missing native glyph buff")?;
        let duration = f32at(row, 16) - clock;
        if !duration.is_finite() || !(0.0..=600.).contains(&duration) || !f32at(row, 4).is_finite()
        {
            return Err("Invalid native glyph buff".into());
        }
        Ok(NativeEffect {
            data: row[..12].try_into().unwrap(),
            duration,
        })
    }
    pub fn capture_aura(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<AuraState> {
        let (_, all) = self.aura_rows(mem, game)?;
        let mut shield = [0; 12];
        shield[..8].copy_from_slice(&mem.read(game + self.game_field(0x480), 8)?);
        shield[8..].copy_from_slice(&mem.read(game + self.game_field(0x4fc), 4)?);
        Ok(AuraState {
            spell,
            rows: all.into_iter().filter(|r| u32at(r, 12) == spell).collect(),
            shield,
            absorb: mem
                .read(game + self.game_field(0x190), 8)?
                .try_into()
                .map_err(|_| "Truncated absorb budget")?,
        })
    }
    pub fn prepare_foreign_shield(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        state: &AuraState,
    ) -> Result<()> {
        if !matches!(state.spell, 17 | 11426) {
            return Err("Only Power Word Shield needs native self eligibility isolation".into());
        }
        let (pointer, rows) = self.aura_rows(mem, game)?;
        let filtered = rows
            .into_iter()
            .filter(|r| u32at(r, 12) != state.spell)
            .collect::<Vec<_>>();
        let current = f32at(&state.shield, 4);
        let total = f32at(&state.shield, 8);
        if !current.is_finite() || !total.is_finite() {
            return Err("Invalid source shield state".into());
        }
        if !filtered.is_empty() {
            self.write_aura_rows(mem, pointer, &filtered)?;
        }
        mem.write(
            game + self.game_field(0x3a8),
            &(filtered.len() as u64).to_le_bytes(),
        )?;
        mem.write(game + self.game_field(0x480), &[0u8; 8])?;
        mem.write(
            game + self.game_field(0x4fc),
            &(total - current).max(0.0).to_le_bytes(),
        )?;
        Ok(())
    }
    pub fn restore_aura(self, mem: &dyn MutableMemory, game: u64, state: &AuraState) -> Result<()> {
        let (pointer, all) = self.aura_rows(mem, game)?;
        let original_len = all.len();
        let mut rows = all
            .into_iter()
            .filter(|r| u32at(r, 12) != state.spell)
            .collect::<Vec<_>>();
        rows.extend_from_slice(&state.rows);
        // Native casts replace or append, so rollback must never grow capacity.
        if rows.len() > original_len {
            return Err("Aura rollback cannot grow native storage".into());
        }
        if !rows.is_empty() {
            self.write_aura_rows(mem, pointer, &rows)?;
        }
        mem.write(
            game + self.game_field(0x3a8),
            &(rows.len() as u64).to_le_bytes(),
        )?;
        if matches!(state.spell, 17 | 11426) {
            mem.write(game + self.game_field(0x480), &state.shield[..8])?;
            mem.write(game + self.game_field(0x4fc), &state.shield[8..])?;
            mem.write(game + self.game_field(0x190), &state.absorb)?;
        }
        Ok(())
    }
    pub fn effect_owner(
        self,
        mem: &dyn Mem,
        game: u64,
        caster: u32,
        caster_guid: u64,
        spell: u32,
        shield: bool,
    ) -> Result<OwnedEffect> {
        let actor = self.support_caster(mem, game)?;
        let (_, rows) = self.aura_rows(mem, game)?;
        let row = rows
            .iter()
            .find(|r| {
                u32at(*r, 12) == spell
                    && self.aura_has_caster(*r, spell, caster_guid, actor.guid)
                    && u32at(*r, 0)
                        == if matches!(spell, 324 | 467) {
                            13
                        } else if shield {
                            7
                        } else {
                            6
                        }
            })
            .ok_or("Missing delivered native aura")?;
        let expires = f32at(row, 16);
        if !expires.is_finite() {
            return Err("Invalid aura expiry".into());
        }
        Ok(OwnedEffect {
            caster,
            caster_guid,
            target_guid: actor.guid,
            spell,
            expires,
            shield,
        })
    }
    pub fn hide_foreign_periodic(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        dt: f32,
        owners: &[OwnedEffect],
    ) -> Result<Vec<PeriodicTick>> {
        self.hide_periodic(mem, game, dt, owners, false)
    }
    /// Also replay unowned personal HoTs while their producer is fatigued.
    /// Explicit foreign ownership always wins over the current recipient.
    pub fn hide_fatigued_periodic(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        dt: f32,
        owners: &[OwnedEffect],
    ) -> Result<Vec<PeriodicTick>> {
        self.hide_periodic(mem, game, dt, owners, true)
    }
    fn hide_periodic(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        dt: f32,
        owners: &[OwnedEffect],
        personal: bool,
    ) -> Result<Vec<PeriodicTick>> {
        if !dt.is_finite() || !(0.0..=1.0).contains(&dt) {
            return Err("Invalid periodic delta".into());
        }
        let actor = self.support_caster(mem, game)?;
        let clock = mem.read(game + self.game_field(0xb9c), 4)?;
        let clock = f32at(&clock, 0);
        if !clock.is_finite() {
            return Err("Invalid periodic clock".into());
        }
        let (pointer, rows) = self.aura_rows(mem, game)?;
        let mut plan = vec![];
        for (i, row) in rows.iter().enumerate() {
            let known = owners.iter().find(|o| {
                !o.shield
                    && matches!(o.spell, 139 | 774 | 8936)
                    && o.target_guid == actor.guid
                    && o.spell == u32at(row, 12)
                    && o.expires.to_bits() == u32at(row, 16)
                    && self.aura_has_caster(row, o.spell, o.caster_guid, actor.guid)
            });
            let own = OwnedEffect {
                caster: actor.slot,
                caster_guid: actor.guid,
                target_guid: actor.guid,
                spell: u32at(row, 12),
                expires: f32at(row, 16),
                shield: false,
            };
            let owner = match known {
                Some(o) if o.caster != actor.slot || personal => o,
                Some(_) => continue,
                None if personal
                    && matches!(own.spell, 139 | 774 | 8936)
                    && u32at(row, 0) == 6
                    && own.expires.is_finite()
                    && own.expires > clock
                    && self.aura_has_caster(row, own.spell, own.caster_guid, actor.guid) =>
                {
                    &own
                }
                None => continue,
            };
            let remaining = f32at(row, 20);
            if u32at(row, 0) != 6 || !remaining.is_finite() || !(0.0..=1e12).contains(&remaining) {
                return Err("Invalid foreign periodic budget".into());
            }
            if remaining <= 0.0 {
                continue;
            }
            let raw = if dt == 0.0 {
                0.0
            } else {
                (dt / (owner.expires - clock).max(dt)).min(1.0) * remaining
            };
            plan.push((
                pointer + i as u64 * self.aura_stride() + self.aura_field(20),
                PeriodicTick {
                    owner: owner.clone(),
                    raw,
                    remaining: remaining - raw,
                },
            ));
        }
        // Validate every row before the first mutation.
        for (address, _) in &plan {
            mem.write(*address, &0f32.to_le_bytes())?;
        }
        Ok(plan.into_iter().map(|(_, p)| p).collect())
    }
    pub fn restore_periodic(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        tick: &PeriodicTick,
    ) -> Result<bool> {
        let actor = self.support_caster(mem, game)?;
        if actor.guid != tick.owner.target_guid || actor.hp <= 0.0 {
            return Ok(false);
        }
        let (pointer, rows) = self.aura_rows(mem, game)?;
        let Some((index, row)) = rows.iter().enumerate().find(|(_, r)| {
            u32at(*r, 12) == tick.owner.spell
                && u32at(*r, 16) == tick.owner.expires.to_bits()
                && self.aura_has_caster(*r, tick.owner.spell, tick.owner.caster_guid, actor.guid)
        }) else {
            return Ok(false);
        };
        if f32at(row, 20) != 0.0 {
            return Err("Periodic aura changed during native callback".into());
        }
        mem.write(
            pointer + index as u64 * self.aura_stride() + self.aura_field(20),
            &tick.remaining.to_le_bytes(),
        )?;
        Ok(true)
    }
}

impl Adapter {
    /// Only Holy Shock's borrowed on-hit healing effect is eligible. Other effect
    /// variants and their payloads remain byte-identical in the native dispatcher.
    pub fn scaled_heal_effect(
        self,
        m: &dyn Mem,
        effect: u64,
        factor: f32,
    ) -> Result<Option<([u8; 12], f32, f32)>> {
        if !self.supports_extensions() {
            return Ok(None);
        }
        if !factor.is_finite() || !(0.0..=2.).contains(&factor) {
            return Err("Invalid glyph effect healing factor".into());
        }
        let mut bytes: [u8; 12] = m
            .read(effect, 12)?
            .try_into()
            .map_err(|_| "Truncated glyph healing effect")?;
        if u32::from_le_bytes(bytes[..4].try_into().unwrap()) != 6 {
            return Ok(None);
        }
        let before = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let fraction = before * factor;
        if !fraction.is_finite() || !(0.0..=1.).contains(&fraction) {
            return Err("Invalid glyph on-hit healing".into());
        }
        bytes[4..8].copy_from_slice(&fraction.to_le_bytes());
        Ok(Some((bytes, before, fraction)))
    }
    pub fn native_hit_effect_entry(self) -> Entry {
        let v = crate::current::GLYPH_ON_HIT_RVA;
        Entry {
            rva: self.contact117(v),
            bytes: self.extension_bytes117(0x5bbf7b0, crate::current::GLYPH_ON_HIT_BYTES),
        }
    }
}
