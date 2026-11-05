//! Native 0.138.0 death snapshots and eight-piece perk rows.
use super::{Adapter, NATIVE_PLAYERS};
use crate::{
    Result,
    memory::Mem,
    official::{Entry, MutableMemory, SupportCalls},
};

const DOTS: [(u32, &[u8]); 12] = [
    (1978, b"serpent_sting"),
    (1943, b"rupture"),
    (8050, b"flame_shock"),
    (980, b"curse_of_agony"),
    (772, b"rend"),
    (589, b"sw_pain"),
    (2944, b"devouring_plague"),
    (172, b"corruption"),
    (348, b"immolate"),
    (689, b"drain_life"),
    (8921, b"moonfire"),
    (5570, b"insect_swarm"),
];
fn word(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}
fn dword(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn scalar(bytes: &[u8], at: usize) -> f32 {
    f32::from_bits(dword(bytes, at))
}
fn extent(ptr: u64, count: u64, stride: u64) -> bool {
    count <= 512 && (count == 0 || ptr >= 65536 && ptr.checked_add(count * stride).is_some())
}
fn dot_owner(m: &dyn Mem, stack: u64, base: u64, slots: [u64; 2], callers: &[(u64, u8, u64)]) -> Result<Option<u64>> {
    let read = |base: u64, offset| -> Result<u64> {
        let at = base.checked_add(offset).ok_or("DoT source frame overflow")?;
        let raw: [u8; 8] = m.read(at, 8)?.try_into().map_err(|_| "Truncated DoT source frame")?;
        Ok(u64::from_le_bytes(raw))
    };
    let Some(caller) = read(stack, slots[0])?.checked_sub(base) else { return Ok(None) };
    let Some(&(_, kind, offset)) = callers.iter().find(|row| row.0 == caller) else { return Ok(None) };
    let game = match kind {
        0 => read(stack, offset)?,
        1 => read(read(stack, slots[1])?, offset)?,
        _ => return Err("Unreviewed DoT source carrier".into()),
    };
    if game < 0x10000 { return Err("Invalid DoT source Game".into()); }
    Ok(Some(game))
}
fn clone_context(m: &dyn Mem, slots: [u64; 2], frame: u64, registers: [u64; 4]) -> Result<(u64, u64, u64, u64)> {
    let [game, event, pointer, count] = registers;
    let [event_slot, count_slot] = slots;
    let read = |offset| -> Result<u64> {
        let address = frame.checked_add(offset).ok_or("Set clone frame overflow")?;
        let bytes: [u8; 8] = m.read(address, 8)?.try_into().map_err(|_| "Truncated set clone frame")?;
        Ok(u64::from_le_bytes(bytes))
    };
    let (event, count) = match (event_slot, count_slot) {
        (0, 0) => (event, count),
        (0, _) | (_, 0) => return Err("Unreviewed set clone frame".into()),
        _ => (read(event_slot)?, read(count_slot)?),
    };
    if game < 0x10000 || event < 0x10000 || !extent(pointer, count, 64) {
        return Err("Invalid private set clone identity or extent".into());
    }
    Ok((game, event, pointer, count))
}
#[derive(Clone)]
pub struct SetDeath {
    pub event: [u8; 40],
    corpse: [u8; 40],
    pub mask: u16,
    rows: Vec<[u8; 56]>,
}
impl SetDeath {
    pub fn invoke(&self, call: impl FnOnce(u64, u64)) {
        let mut corpse = self.corpse;
        let count = self.rows.len() as u64;
        corpse[..8].copy_from_slice(&count.to_le_bytes());
        corpse[8..16].copy_from_slice(&(self.rows.as_ptr() as u64).to_le_bytes());
        corpse[16..24].copy_from_slice(&count.to_le_bytes());
        call(self.event.as_ptr() as u64, corpse.as_ptr() as u64);
    }
}
impl Adapter {
    /// R15/R13/R14/RBX observations immediately after the native private copy.
    pub fn set_clone_context(self, m: &dyn Mem, frame: u64, registers: [u64; 4]) -> Result<(u64, u64, u64, u64)> {
        clone_context(m, crate::current::SET_CLONE_FRAME_FIELDS, frame, registers)
    }
    pub fn set_entries(self) -> [Entry; 3] {
        [0x5d3f30e, 0x5d426d7, 0x5b8e99b].map(|a| Entry {
            rva: self.rva(a),
            bytes: self.expected(a, &[]),
        })
    }
    /// The native tick at 5d058d0 compares DoT+0x10 to the active coop slot.
    /// Split only spells with a foreign producer. Other kill effects run normally.
    pub fn set_deaths(
        self,
        m: &dyn Mem,
        event: u64,
        corpse: u64,
        killer: u32,
    ) -> Result<(u16, Vec<(u32, SetDeath)>)> {
        let event: [u8; 40] = m
            .read(event, 40)?
            .try_into()
            .map_err(|_| "Invalid death event")?;
        if event[29] != 2 || corpse == 0 {
            return Ok((0, Vec::new()));
        }
        let corpse: [u8; 40] = m
            .read(corpse, 40)?
            .try_into()
            .map_err(|_| "Invalid native corpse")?;
        let (ptr, count) = (word(&corpse, 8), word(&corpse, 16));
        if count > word(&corpse, 0) || !extent(ptr, count, 56) {
            return Err("Invalid corpse effects".into());
        }
        if count == 0 {
            return Ok((0, Vec::new()));
        }
        let rows = m.read(ptr, count as usize * 56)?;
        let mut grouped = std::collections::BTreeMap::new();
        let mut foreign = 0;
        for row in rows.chunks_exact(56) {
            let Some(index) = DOTS.iter().position(|d| d.0 == dword(row, 24)) else {
                continue;
            };
            let bit = 1 << index;
            let owner = word(row, 16);
            // A missing producer or an exhausted row must not lend an effect
            // to the killer. Valid active rows of the same spell still count.
            if owner >= NATIVE_PLAYERS as u64 || dword(row, 40) == 0 {
                foreign |= bit;
                continue;
            }
            let slot = owner as u32;
            if slot != killer {
                foreign |= bit;
            }
            let entry = grouped.entry(slot).or_insert_with(|| SetDeath {
                event,
                corpse,
                mask: 0,
                rows: vec![],
            });
            entry.mask |= bit;
            entry.rows.push(row.try_into().unwrap());
        }
        Ok((
            foreign,
            grouped
                .into_iter()
                .filter_map(|(slot, mut d)| {
                    d.mask &= foreign;
                    (d.mask != 0).then_some((slot, d))
                })
                .collect(),
        ))
    }
    fn set_row_bit(self, m: &dyn Mem, row: &[u8]) -> Result<u16> {
        let Some(row) = super::perks::observe(crate::current::PERK_OBSERVATION_LAYOUT, row)? else { return Ok(0) };
        let row = &row[..];
        // Legendary corpse explosions encode both a spell id and a key.
        // Require both so unrelated kill perks remain in the original dispatch.
        if row[..2] == [11, 1] && dword(row, 16) == 2 && row[24] == 1
            && scalar(row, 4) == 0.2 && scalar(row, 8) == 7.
        {
            if let Some((i, (_, key))) = DOTS.iter().enumerate()
                .find(|(_, (id, _))| *id == dword(row, 12) && *id != 8050)
            {
                if word(row, 40) == key.len() as u64 && m.read(word(row, 32), key.len())? == *key {
                    return Ok(1 << i);
                }
            }
            return Ok(0);
        }
        let (at, possible): (usize, &[usize]) = match row[0] {
            11 if row[1] == 1
                && scalar(row, 8) == 8.
                && matches!(scalar(row,4),v if v==0.15 || v==0.15f32*1.5)
                && row[24] == 1 =>
            {
                (32, &[1, 2])
            }
            14 if matches!(dword(row, 4), 2 | 3) && row[8] == 1 => (16, &[0, 3]),
            _ => return Ok(0),
        };
        for &i in possible {
            let key = DOTS[i].1;
            if word(row, at + 8) == key.len() as u64 && m.read(word(row, at), key.len())? == key {
                return Ok(1 << i);
            }
        }
        Ok(0)
    }
    /// Edit only the private POD clone made by the native dispatcher. Its
    /// indexes, cooldown vectors, source metadata and equipped rows stay intact.
    pub fn set_filter_clone(
        self,
        m: &dyn MutableMemory,
        ptr: u64,
        count: u64,
        mask: u16,
        only: bool,
    ) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        if !extent(ptr, count, 64) {
            return Err("Invalid native perk clone".into());
        }
        let mut rows = m.read(ptr, count as usize * 64)?;
        if only && count > 0 {
            m.write(ptr, super::perks::terminator(crate::current::PERK_OBSERVATION_LAYOUT)?)?;
        }
        for row in rows.chunks_exact_mut(64) {
            if (self.set_row_bit(m, row)? & mask != 0) != only {
                super::perks::disable(crate::current::PERK_OBSERVATION_LAYOUT, row)?;
            }
        }
        m.write(ptr, &rows)
    }
    pub fn set_actor<T>(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        guid: u64,
        f: impl FnOnce() -> Result<T>,
    ) -> Result<Option<T>> {
        if self.effect_source(io, game)?.is_none_or(|(s, _)| s != 0) {
            return Ok(None);
        }
        let row = self.support_row(io, game, slot)?;
        if io.read(row + self.row_field(row == game, 0x478), 8)? != guid.to_le_bytes()
            || scalar(&io.read(row + self.row_field(row == game, 0x4f8), 4)?, 0) <= 0.
        {
            return Ok(None);
        }
        if slot == 0 {
            return f().map(Some);
        }
        let at = game + self.game_field(0xb40);
        io.swap_actor(game, row)?;
        if let Err(e) = io.write(at, &u64::from(slot).to_le_bytes()) {
            io.swap_actor(game, row)?;
            return Err(e);
        }
        let result = f();
        let flush = io.flush_actor(game);
        let host = self.support_row(io, game, 0)?;
        io.write(at, &0u64.to_le_bytes())?;
        io.swap_actor(game, host)?;
        flush?;
        result.map(Some)
    }
    /// Read the engine actor that actually placed this row. Cover both direct
    /// spell paths and the native proc/spread paths in the pinned image.
    pub fn set_stamp_dot(
        self,
        m: &dyn MutableMemory,
        stack: u64,
        base: u64,
        registers: [u64; 4],
    ) -> Result<()> {
        let ptr = registers[0];
        let offset = *registers.get(crate::current::SET_DOT_ROW_REGISTER).ok_or("Unreviewed DoT row register")?;
        let spell = registers[3] as u32;
        if !DOTS.iter().any(|d| d.0 == spell) {
            return Ok(());
        }
        let Some(game) = dot_owner(m, stack, base, crate::current::SET_DOT_STACK_FIELDS,
                                  crate::current::SET_DOT_OWNER_CALLERS)? else { return Ok(()) };
        let Some((slot, _)) = self.effect_source(m, game)? else {
            return Ok(());
        };
        let Some(address) = ptr.checked_add(offset).and_then(|p| p.checked_add(16)) else {
            return Err("Invalid new native DoT".into());
        };
        if word(&m.read(address, 8)?, 0) == u64::MAX {
            m.write(address, &u64::from(slot).to_le_bytes())?;
        }
        Ok(())
    }
    pub fn set_clock(self, m: &dyn Mem, game: u64) -> Result<f32> {
        Ok(scalar(&m.read(game + self.game_field(0xb9c), 4)?, 0))
    }
    pub fn set_vulnerability(
        self,
        m: &dyn Mem,
        frame: u64,
        field: u64,
    ) -> Result<(u64, u64, u32, f32, [f32; 2])> {
        let [game_slot, spell_slot, coefficient] = crate::current::SET_VULNERABILITY_LAYOUT;
        let game_at = frame.checked_add(game_slot).ok_or("Set Game frame overflow")?;
        let spell_at = frame.checked_add(spell_slot).ok_or("Set spell frame overflow")?;
        let distance = coefficient.checked_add(8)
            .and_then(|at| at.checked_sub(crate::current::ENEMY_GUID as u64))
            .ok_or("Unreviewed set vulnerability fields")?;
        let guid_at = field.checked_sub(distance).ok_or("Set enemy field underflow")?;
        let values_at = field.checked_sub(8).ok_or("Set vulnerability underflow")?;
        let game = word(&m.read(game_at, 8)?, 0);
        let spell = dword(&m.read(spell_at, 4)?, 0);
        let enemy = word(&m.read(guid_at, 8)?, 0);
        let values = m.read(values_at, 8)?;
        Ok((
            game,
            enemy,
            spell,
            self.set_clock(m, game)?,
            [scalar(&values, 0), scalar(&values, 4)],
        ))
    }
    pub fn set_vulnerability_write(
        self,
        m: &dyn MutableMemory,
        game: u64,
        enemy: u64,
        expected: [f32; 2],
        next: [f32; 2],
    ) -> Result<bool> {
        let coefficient = crate::current::SET_VULNERABILITY_LAYOUT[2] as usize;
        let (p, rows) = self.combat_rows(m, game)?;
        for (i, r) in rows.chunks_exact(crate::current::ENEMY_STRIDE).enumerate() {
            if word(r, crate::current::ENEMY_GUID) == enemy
                && r[crate::current::ENEMY_DEAD] == 0
                && [scalar(r, coefficient), scalar(r, coefficient + 4)] == expected
            {
                m.write(
                    p + (i * crate::current::ENEMY_STRIDE) as u64 + coefficient as u64,
                    &next.map(f32::to_le_bytes).concat(),
                )?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(feature = "test-hooks")]
impl Adapter {
    fn fixture_necklace_rows(self, m: &dyn Mem, base:u64, definition:u64) -> Result<Vec<u8>> {
        type Build = unsafe extern "system" fn(*mut u64,u64);
        type Free = unsafe extern "system" fn(u64,usize,usize);
        let entry = self.rva(0x5c83520);
        let expected = self.expected(0x5c83520, &[]);
        if m.read(base + entry, expected.len())? != expected { return Err("Native necklace fixture constructor changed".into()); }
        let build:Build=unsafe{std::mem::transmute((base+entry) as usize)};
        let free:Free=unsafe{std::mem::transmute((base+self.rva(0x21d0)) as usize)};
        let mut output=[0u64;16];
        unsafe{build(output.as_mut_ptr(),base+definition)};
        let n=output[8];
        let result=if n <= output[6] && n <= 32 {m.read(output[7],n as usize*64)}else{Err("Invalid necklace fixture".into())};
        for (at,stride,align) in [(0,1,1),(3,1,1),(6,64,8)] {
            if output[at]>0 {unsafe{free(output[at+1],output[at] as usize*stride,align)}}
        }
        result
    }
    pub fn fixture_necklace_cast(self,io:&dyn SupportCalls,base:u64,game:u64,spell:u32,ability:u64)->Result<(f32,f32)>{
        let party=self.world_party(io,game)?.ok_or("Missing necklace party")?;
        let rows=self.fixture_necklace_rows(io,base,self.fixture_support_definition(spell)?)?;
        let factor:unsafe extern "system" fn(u64)->f32=unsafe{std::mem::transmute((base+self.necklace_leech_entry().rva)as usize)};
        let read=|slot:u32|->Result<f32>{
            if matches!(spell,139|774){
                return self.set_actor(io,game,slot,party.actors[slot as usize].guid,||Ok(unsafe{factor(game)}))?.ok_or("Missing leech actor".into());
            }
            let row=self.support_row(io,game,slot)?;
            Ok(scalar(&io.read(row+self.row_field(row==game,0x4fc),4)?,0))
        };
        self.fixture_support_wounds(io,game,0,1)?;
        // The native healer also checks the active caster's HP before casting.
        let maximum=self.support_caster(io,game)?.maximum;
        io.write(game+self.game_field(0x4f8),&(maximum*0.25).to_le_bytes())?;
        let guest=self.support_row(io,game,1)?;
        if matches!(spell,635|8936){io.write(guest+self.row_field(false,0x4fc),&0f32.to_le_bytes())?;}
        let before=[read(0)?,read(1)?];
        self.fixture_set_scope(io,game,&rows,base,||{
            if !self.fixture_cast_resolved(io,game,0,ability)? {return Err("Necklace cast failed".into())}
            let mut cast_event=[0u8;40];
            cast_event[29]=3;
            cast_event[8..12].copy_from_slice(&spell.to_le_bytes());
            let dispatch:unsafe extern "system" fn(u64,u64,u64)=unsafe{std::mem::transmute((base+self.support_proc_entries()[1].rva)as usize)};
            unsafe{dispatch(game,cast_event.as_ptr()as u64,0)};
            Ok(())
        })?;
        let scale=if matches!(spell,635|8936){maximum}else{1.};
        Ok(((read(0)?-before[0])/scale,(read(1)?-before[1])/scale))
    }
    fn fixture_set_rows(
        self,
        m: &dyn Mem,
        base: u64,
        class: u8,
        tier: usize,
        scale: f32,
    ) -> Result<Vec<u8>> {
        type Build = unsafe extern "system" fn(*mut u64, u8, usize, f32);
        type Free = unsafe extern "system" fn(u64, usize, usize);
        let entry = self.rva(0x5c88f50);
        let expected = self.expected(0x5c88f50, &[]);
        if m.read(base + entry, expected.len())? != expected { return Err("Native set fixture constructor changed".into()); }
        let build: Build = unsafe { std::mem::transmute((base + entry) as usize) };
        let free: Free = unsafe { std::mem::transmute((base + self.rva(0x21d0)) as usize) };
        let mut output = [0u64; 16];
        unsafe { build(output.as_mut_ptr(), class, tier, scale) };
        let [cap, p, n, ..] = output;
        if cap > 32 || n > cap || n == 0 {
            return Err("Missing native eight-piece definition".into());
        }
        let result = m.read(p, n as usize * 64);
        unsafe { free(p, cap as usize * 64, 8) };
        result
    }
    fn fixture_set_scope<T>(
        self,
        m: &dyn MutableMemory,
        game: u64,
        rows: &[u8],
        base: u64,
        run: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        if rows.len() % 64 != 0 { return Err("Truncated set fixture rows".into()); }
        let mut selected = Vec::with_capacity(rows.len());
        for row in rows.chunks_exact(64) {
            if super::perks::dispatch_effect(crate::current::PERK_OBSERVATION_LAYOUT, row)? {
                selected.extend_from_slice(row);
            }
        }
        let rows = selected;
        let count = rows.len() / 64;
        if count == 0 || count > 32 || rows.len() != count * 64 {
            return Err("Invalid set fixture rows".into());
        }
        let h = m.read(game + self.game_field(0x830), 24)?;
        let cd = m.read(game + self.game_field(0x68), 24)?;
        let mut timers = vec![0f32; count];
        let header = |p: u64| {
            [count as u64, p, count as u64]
                .map(u64::to_le_bytes)
                .concat()
        };
        // The native resolver checks the active class and learned abilities.
        // These disposable actors exercise sets from other classes in one run.
        let stride = crate::current::LEARNED_ABILITY_STRIDE;
        let learned = m.read(game + self.game_field(0xb0), 24)?;
        let old_len = word(&learned, 16);
        if !extent(word(&learned, 8), old_len, stride as u64) {
            return Err("Invalid diagnostic ability list".into());
        }
        let mut definitions = if old_len > 0 {
            m.read(word(&learned, 8), old_len as usize * stride)?
        } else {
            vec![]
        };
        for rva in [
            0x711ca10, 0x711d460, 0x711d988, 0x711e360, 0x711e3d8, 0x711ef90,
            0x6cc4df0, 0x6cc5c78, 0x6cc72f8, 0x6cc7370,
        ] {
            let mut instance = vec![0u8; stride];
            instance[..8].copy_from_slice(&(base + self.rva(rva)).to_le_bytes());
            definitions.extend(instance);
        }
        let len = (definitions.len() / stride) as u64;
        let learned_header = [len, definitions.as_ptr() as u64, len]
            .map(u64::to_le_bytes)
            .concat();
        m.write(game + self.game_field(0x68), &header(timers.as_mut_ptr() as u64))?;
        if let Err(e) = m.write(game + self.game_field(0x830), &header(rows.as_ptr() as u64)) {
            m.write(game + self.game_field(0x68), &cd)?;
            return Err(e);
        }
        if let Err(e) = m.write(game + self.game_field(0xb0), &learned_header) {
            let restored = m.write(game + self.game_field(0x830), &h);
            m.write(game + self.game_field(0x68), &cd)?;
            restored?;
            return Err(e);
        }
        let result = run();
        let restored_abilities = m.write(game + self.game_field(0xb0), &learned);
        let restored = m.write(game + self.game_field(0x830), &h);
        m.write(game + self.game_field(0x68), &cd)?;
        restored_abilities?;
        restored?;
        result
    }
    pub fn fixture_sets(
        self,
        io: &dyn SupportCalls,
        game: u64,
        base: u64,
        case: u32,
        drain: impl Fn(),
    ) -> Result<(f32, f32)> {
        type Proc = unsafe extern "system" fn(u64, u64, u64);
        let proc: Proc =
            unsafe { std::mem::transmute((base + self.support_proc_entries()[1].rva) as usize) };
        let p = self.world_party(io, game)?.ok_or("Missing fixture party")?;
        if p.actors.len() != 2 {
            return Err("Set scenario requires two players".into());
        }
        let id = self
            .fixture_combat_enemy(io, game, 5)?
            .ok_or("Missing witness")?;
        let target = self
            .support_enemies(io, game)?
            .into_iter()
            .find(|e| e.guid == id)
            .ok_or("Missing set target")?;
        let pos = target.position.map(|x| x as f32);
        let mut event = [0u8; 40];
        event[..8].copy_from_slice(&id.to_le_bytes());
        for (j, v) in pos.iter().enumerate() {
            event[12 + 4 * j..16 + 4 * j].copy_from_slice(&v.to_le_bytes());
        }
        event[24..28].copy_from_slice(&100f32.to_le_bytes());
        let hp = || {
            self.combat_hp(io, game, id)?
                .ok_or("Set witness disappeared".to_owned())
        };
        let before = hp()?;
        if case < 4 || case == 9 || case == 10 || case == 18 {
            let (class, tier, dot) = match case {
                0 => (3, 1, 1978u32),
                1 => (4, 1, 1943),
                2 => (7, 1, 8050),
                3 => (9, 1, 980),
                18 => (4, 1, 1943),
                _ => (2, 3, 1943),
            };
            let rows = if case == 18 {self.fixture_necklace_rows(io, base, self.rva(0x711d988))?} else {self.fixture_set_rows(io, base, class, tier, 1.)?};
            event[29] = 2;
            event[8..12].copy_from_slice(&(if matches!(case,9|10) { 20271u32 } else { 845 }).to_le_bytes());
            let mut effect = [0u8; 56];
            let owner = u32::from(case == 1 || case == 3);
            effect[16..24].copy_from_slice(&u64::from(owner).to_le_bytes());
            effect[24..28].copy_from_slice(&dot.to_le_bytes());
            for at in [28, 32] {
                effect[at..at + 4].copy_from_slice(&10f32.to_le_bytes());
            }
            for at in [40, 44] {
                effect[at..at + 4].copy_from_slice(&3u32.to_le_bytes());
            }
            let mut corpse = [0u8; 40];
            corpse[..24].copy_from_slice(
                &[1, effect.as_ptr() as u64, 1]
                    .map(u64::to_le_bytes)
                    .concat(),
            );
            corpse[24..28].copy_from_slice(&1000f32.to_le_bytes());
            corpse[28..40].copy_from_slice(&event[12..24]);
            // A distinct dying victim leaves the durable witness inside the native spatial query.
            event[..8].copy_from_slice(&(id ^ 0x10000000).to_le_bytes());
            if owner == 1 {
                self.set_actor(io, game, 1, p.actors[1].guid, || {
                    self.fixture_set_scope(io, game, &rows, base, || {
                        self.polymorph_select(io, game, 0)?;
                        let result = self.fixture_set_scope(io, game, &rows, base, || {
                            unsafe { proc(game, event.as_ptr() as u64, corpse.as_ptr() as u64) };
                            drain();
                            Ok(())
                        });
                        self.polymorph_select(io, game, 1)?;
                        result
                    })
                })?;
            } else {
                self.fixture_set_scope(io, game, &rows, base, || {
                    if case == 10 {
                        unsafe { proc(game, event.as_ptr() as u64, corpse.as_ptr() as u64) }
                    } else {
                        self.set_actor(io, game, 1, p.actors[1].guid, || {
                            // Same set on both actors for the foreign DoT controls. For
                            // Judgement only the non-killer owns the bonus.
                            if case < 4 || case == 18 {
                                self.fixture_set_scope(io, game, &rows, base, || {
                                    unsafe {
                                        proc(game, event.as_ptr() as u64, corpse.as_ptr() as u64)
                                    };
                                    Ok(())
                                })?
                            } else {
                                unsafe { proc(game, event.as_ptr() as u64, corpse.as_ptr() as u64) }
                            }
                            Ok(())
                        })?;
                    }
                    drain();
                    Ok(())
                })?;
            }
            let (_, enemies) = self.combat_rows(io, game)?;
            let mut propagated = 0;
            for enemy in enemies.chunks_exact(crate::current::ENEMY_STRIDE) {
                let n = word(enemy, 16);
                let ptr = word(enemy, 8);
                if n > 0 && extent(ptr, n, 56) {
                    for row in io.read(ptr, n as usize * 56)?.chunks_exact(56) {
                        if dword(row, 24) == dot
                            && word(row, 16) == u64::from(owner)
                            && dword(row, 40) > 0
                        {
                            propagated += 1
                        }
                    }
                }
            }
            return Ok((before - hp()?, propagated as f32));
        }
        if (4..=6).contains(&case) || case == 19 {
            let mut rows =
                self.fixture_set_rows(io, base, 5, 1, if case == 6 { 1.5 } else { 1. })?;
            if case == 19 { rows=self.fixture_necklace_rows(io,base,self.fixture_support_definition(17)?)?; }
            if case == 4 {
                rows.extend(io.read(base + self.rva(0x7136740), 64)?);
            }
            let mut origin = pos;
            origin[0] += if case == 6 { 10.0 } else { 0.0 };
            let mut distant = pos;
            distant[0] += 40.;
            self.fixture_event_pose(io, game, 0, p.actors[0].guid, distant)?;
            self.fixture_event_pose(io, game, 1, p.actors[1].guid, origin)?;
            let area = io.read(game + self.game_field(0x4b8), 4)?;
            io.write(game + self.game_field(0x4b8), &1.5f32.to_le_bytes())?;
            event[29] = 6;
            event[12..24].copy_from_slice(&origin.map(f32::to_le_bytes).concat());
            let old_origin=io.read(game + crate::current::PERK_ORIGINS_HEADER,24)?;
            let icon=b"INV_Jewelry_Necklace_21";
            let origin_row=[u64::MAX,1,0,icon.as_ptr() as u64,icon.len() as u64];
            if case==19 {io.write(game + crate::current::PERK_ORIGINS_HEADER,&[1,origin_row.as_ptr() as u64,1].map(u64::to_le_bytes).concat())?;}
            let result = self.fixture_set_scope(io, game, &rows, base, || {
                io.shield_proc(
                    game,
                    &event,
                    origin,
                    p.actors[1].guid,
                    if case == 5 { 324 } else { 17 },
                )
            });
            if case==19 {io.write(game + crate::current::PERK_ORIGINS_HEADER,&old_origin)?;}
            io.write(game + self.game_field(0x4b8), &area)?;
            for a in &p.actors {
                self.fixture_event_pose(io, game, a.slot, a.guid, a.position)?;
            }
            result?;
            return Ok((before - hp()?, 0.));
        }
        if case == 7 || case == 8 {
            let (class, spell) = if case == 7 {
                (1, 12294u32)
            } else {
                (7, 8042u32)
            };
            let read = || -> Result<f32> {
                let (_, v) = self.combat_rows(io, game)?;
                Ok(scalar(
                    v.chunks_exact(crate::current::ENEMY_STRIDE)
                        .find(|r| word(r, crate::current::ENEMY_GUID) == id)
                        .ok_or("Missing target")?,
                    crate::current::ENEMY_VULNERABILITY,
                ))
            };
            event[8..12].copy_from_slice(&spell.to_le_bytes());
            let strong = self.fixture_set_rows(io, base, class, 5, 1.5)?;
            self.fixture_set_scope(io, game, &strong, base, || {
                unsafe { proc(game, event.as_ptr() as u64, 0) };
                Ok(())
            })?;
            let first = read()?;
            self.set_actor(io, game, 1, p.actors[1].guid, || {
                let weak = self.fixture_set_rows(io, base, class, 5, 1.)?;
                self.fixture_set_scope(io, game, &weak, base, || {
                    unsafe { proc(game, event.as_ptr() as u64, 0) };
                    Ok(())
                })
            })?;
            return Ok((first, read()?));
        }
        if (11..=17).contains(&case) {
            let class = [2, 3, 4, 5, 8, 9, 11][(case - 11) as usize];
            let rows = self.fixture_set_rows(io, base, class, 5, 1.)?;
            let mut modifier = None;
            for row in rows.chunks_exact(64) {
                if let Some((18, value)) = super::perks::stat_modifier(
                    crate::current::PERK_OBSERVATION_LAYOUT, row,
                )? { modifier = Some(value); break; }
            }
            let modifier = modifier.ok_or("Missing controlled-target modifier")?;
            // The ally runs the ordinary stun impact, then only the wearer's
            // native stat vector changes between the two otherwise equal hits.
            type Impact = unsafe extern "system" fn(u64, u64, *const u8, f32, u32);
            let impact: Impact = unsafe {
                std::mem::transmute((base + self.native_hit_effect_entry().rva) as usize)
            };
            let mut stun = [0u8; 32];
            stun[..4].copy_from_slice(&3u32.to_le_bytes());
            stun[4..8].copy_from_slice(&4f32.to_le_bytes());
            // One allied stun supplies the same control window to all seven
            // classes. Reapplying it for every class would trigger native DR.
            if case == 11 {
                self.set_actor(io, game, 1, p.actors[1].guid, || {
                    unsafe { impact(game, id, stun.as_ptr(), 0., 853) };
                    Ok(())
                })?;
            }
            let (_, enemies) = self.combat_rows(io, game)?;
            let witness = enemies
                .chunks_exact(crate::current::ENEMY_STRIDE)
                .find(|row| word(row, crate::current::ENEMY_GUID) == id)
                .ok_or("Missing stun witness")?;
            if scalar(witness, crate::current::SOLO_ENEMY_ROOT_UNTIL) <= self.set_clock(io, game)? {
                return Err("Ally stun has expired on the witness".into());
            }
            let before = hp()?;
            let rng = io.read(game + self.game_field(0xae8), 32)?;
            io.support_hit(game, id, 100., 845)?;
            let after = hp()?;
            let h = io.read(game + self.game_field(0x800), 24)?;
            let n = word(&h, 16);
            if !extent(word(&h, 8), n, 8) {
                return Err("Invalid fixture modifier storage".into());
            }
            let mut modifiers = if n == 0 {
                vec![]
            } else {
                io.read(word(&h, 8), n as usize * 8)?
            };
            modifiers.extend(18u32.to_le_bytes());
            modifiers.extend(modifier.to_le_bytes());
            io.write(
                game + self.game_field(0x800),
                &[n + 1, modifiers.as_ptr() as u64, n + 1]
                    .map(u64::to_le_bytes)
                    .concat(),
            )?;
            io.write(game + self.game_field(0xae8), &rng)?;
            let hit = io.support_hit(game, id, 100., 845);
            io.write(game + self.game_field(0x800), &h)?;
            hit?;
            return Ok((before - after, after - hp()?));
        }
        Err("Unknown set fixture".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Memory(Vec<u8>);
    impl Mem for Memory {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>> {
            self.0
                .get(a as usize..a as usize + n)
                .map(Vec::from)
                .ok_or("Invalid mock read".into())
        }
    }
    #[test]
    fn dot_sources_distinguish_saved_registers_and_parent_frame_spills() {
        let mut memory = Memory(vec![0; 0x22000]);
        let (stack, frame, game, base) = (0x10000u64, 0x18000u64, 0x20000u64, 0x140000000u64);
        let cases = [
            ([0xe8, 0xb0], (0x5ec7eee, 0, 0xc0)),
            ([0xe8, 0xb0], (0x5f55e0f, 1, 0x1b8)),
            ([0xd8, 0xa0], (0x5fb67ef, 0, 0x98)),
            ([0xd8, 0xa0], (0x5fbf2ac, 1, 0x1e0)),
        ];
        for (slots, caller) in cases {
            memory.0.fill(0);
            let put = |memory: &mut Memory, at: u64, value: u64| {
                memory.0[at as usize..at as usize + 8].copy_from_slice(&value.to_le_bytes());
            };
            put(&mut memory, stack + slots[0], base + caller.0);
            put(&mut memory, stack + slots[1], frame);
            put(&mut memory, if caller.1 == 0 { stack + caller.2 } else { frame + caller.2 }, game);
            assert_eq!(dot_owner(&memory, stack, base, slots, &[caller]).unwrap(), Some(game));
            assert_eq!(dot_owner(&memory, stack, base, slots, &[]).unwrap(), None);
            assert!(dot_owner(&memory, stack, base, slots, &[(caller.0, 2, caller.2)]).is_err());
        }
        assert!(dot_owner(&memory, u64::MAX, base, [0xd8, 0xa0], &[]).is_err());
    }
    #[test]
    fn private_clone_uses_selected_spills_without_confusing_event_and_count() {
        let mut memory = Memory(vec![0; 0x12000]);
        let frame = 0x10000;
        memory.0[frame + 0x1b8..frame + 0x1c0].copy_from_slice(&0x30000u64.to_le_bytes());
        memory.0[frame + 0x58..frame + 0x60].copy_from_slice(&2u64.to_le_bytes());
        let registers = [0x20000, 0xdeadbeef, 0x40000, 8];
        assert_eq!(clone_context(&memory, [0, 0], frame as u64, registers).unwrap(),
                   (0x20000, 0xdeadbeef, 0x40000, 8));
        assert_eq!(clone_context(&memory, [0x1b8, 0x58], frame as u64, registers).unwrap(),
                   (0x20000, 0x30000, 0x40000, 2));
        assert!(clone_context(&memory, [0, 0x58], frame as u64, registers).is_err());
        assert!(clone_context(&memory, [0x1b8, 0x58], u64::MAX, registers).is_err());
        assert!(clone_context(&memory, [0, 0], frame as u64, [42, 0x30000, 0x40000, 2]).is_err());
        assert!(clone_context(&memory, [0, 0], frame as u64, [0x20000, 0x30000, 0x40000, 513]).is_err());
    }
    #[test]
    fn matching_uses_native_tag_bytes_and_ignores_padding() {
        let a = Adapter::select(crate::current::SHA256).unwrap();
        let key = b"serpent_sting";
        let mut m = Memory(vec![0; 0x10100]);
        m.0[0x10000..0x10000 + key.len()].copy_from_slice(key);
        let mut row = [0xdb; 64];
        row[..4].copy_from_slice(&26u32.to_le_bytes());
        row[32..36].copy_from_slice(&2u32.to_le_bytes());
        row[8..12].copy_from_slice(&1u32.to_le_bytes());
        row[16..24].copy_from_slice(&0x10000u64.to_le_bytes());
        row[24..32].copy_from_slice(&(key.len() as u64).to_le_bytes());
        assert_eq!(a.set_row_bit(&m, &row).unwrap(), 1);
        row[8] = 2;
        assert_eq!(a.set_row_bit(&m, &row).unwrap(), 0);
    }
    #[test]
    fn corpse_splits_only_active_owned_rows_and_never_grants_an_unassigned_tick() {
        let a = Adapter::select(crate::current::SHA256).unwrap();
        let mut m = Memory(vec![0; 0x12000]);
        m.0[29] = 2;
        let c = 0x1000;
        let p = 0x10000;
        m.0[c..c + 24].copy_from_slice(&[4u64, p as u64, 4].map(u64::to_le_bytes).concat());
        for (j, (owner, spell, ticks)) in [
            (1u64, 1978u32, 2u32),
            (0, 1978, 1),
            (0, 1943, 0),
            (u64::MAX, 980, 2),
        ]
        .into_iter()
        .enumerate()
        {
            let at = p + j * 56;
            m.0[at + 16..at + 24].copy_from_slice(&owner.to_le_bytes());
            m.0[at + 24..at + 28].copy_from_slice(&spell.to_le_bytes());
            m.0[at + 40..at + 44].copy_from_slice(&ticks.to_le_bytes());
        }
        let (mask, work) = a.set_deaths(&m, 0, c as u64, 0).unwrap();
        assert_eq!(mask, 0b1011);
        assert_eq!(
            work.iter()
                .map(|(slot, d)| (*slot, d.mask, d.rows.len()))
                .collect::<Vec<_>>(),
            [(0, 1, 1), (1, 1, 1)]
        );
    }
}
