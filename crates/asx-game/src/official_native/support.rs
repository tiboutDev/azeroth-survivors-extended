//! Reviewed official support queue boundary; no custom peer protocol.
use super::Adapter;
use crate::{
    official::{Entry, MutableMemory, SupportCalls},
    memory::Mem,
    Result,
};
mod advanced;
mod effects;
pub use advanced::{NativeEffect, RetaliationTick};
pub use effects::{
    AuraState, BreakFrame, OwnedEffect, PeriodicTick, PerkLoan, SupportReceipt, SupportTalents,
};
pub const ROUTE: u32 = 0xac115000;
// Internal descriptor padding only, not native protocol opcodes. Three target
// bits cover native slot 4; area delivery has a separate bit and cannot collide.
pub const ROUTE_GROUP: u32 = 8;
const ROUTE_TARGET: u32 = 7;
#[derive(Clone, Debug)]
pub struct SupportCaster {
    pub slot: u32,
    pub hp: f32,
    pub maximum: f32,
    pub position: [f32; 3],
    pub talents: SupportTalents,
    pub guid: u64,
}
#[derive(Clone, Debug)]
pub struct SupportDelivery {
    pub guid: u64,
    pub caster: u32,
    pub target: u32,
    pub spell: u32,
    pub amount: f32,
    pub duration: f32,
    pub eligible: bool,
    pub shield: bool,
    pub row: [u8; 56],
}
impl Adapter {
    pub fn fixture_support_definition(self, spell: u32) -> Result<u64> {
        match spell {
            11426 => Ok(self.contact117(0x72a0ca0)),
            139 => Ok(self.rva(0x6cc5c78)),
            774 => Ok(self.rva(0x6cc72f8)),
            8936 => Ok(self.rva(0x6cc7370)),
            17 => Ok(self.rva(0x6cc5c00)),
            324 => Ok(self.rva(0x6cc61a0)),
            467 => Ok(self.rva(0x6cc73e8)),
            15237 => Ok(self.rva(0x6cc5b88)),
            25912 => Ok(self.rva(0x6cc4c10)),
            635 => Ok(self.rva(0x6cc4df0)),
            331 => Ok(self.rva(0x6cc6380)),
            6673 => Ok(self.rva(0x6cc4850)),
            _ => Err("Unsupported native fixture spell".into()),
        }
    }
    pub fn support_entries(self) -> [Entry; 4] {
        [
            Entry {
                rva: self.rva(0x59317f0),
                bytes: self.expected(0x59317f0, &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x81, 0xec, 0x98, 0x02, 0, 0,
                ]),
            },
            Entry {
                rva: self.rva(0x590bbd0),
                bytes: self.expected(0x590bbd0, &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x81, 0xec, 0x98, 0, 0, 0,
                ]),
            },
            Entry {
                rva: self.rva(0x58ffac0),
                bytes: self.expected(0x58ffac0, &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x81, 0xec, 0x98, 0, 0, 0,
                ]),
            },
            Entry {
                rva: self.rva(0x590bab0),
                bytes: self.expected(0x590bab0, &[0x56, 0x57, 0x53, 0x48, 0x83, 0xec, 0x70]),
            },
        ]
    }
    pub fn flush_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x5901040),
                bytes: self.expected(0x5901040, &[0x56, 0x57, 0x48, 0x83, 0xec, 0x58]),
        }
    }
    pub fn fixture_support_wounds(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        caster: u32,
        target: u32,
    ) -> Result<()> {
        let party = self.world_party(mem, game)?.ok_or("Missing fixture party")?;
        if caster == target || ![caster,target].iter().all(|slot| party.actors.iter().any(|a| a.slot==*slot)) {
            return Err("Invalid fixture support roles".into());
        }
        for actor in &party.actors {
            let slot=actor.slot;
            let ratio = if slot == caster {
                0.9f32
            } else if slot == target {
                0.2
            } else {
                0.45
            };
            let row = self.support_row(mem, game, slot)?;
            let bytes = mem.read(row + self.row_field(row == game, 0x490), 4)?;
            let maximum =
                f32::from_le_bytes(bytes.try_into().map_err(|_| "Truncated fixture maximum")?);
            if !maximum.is_finite() || maximum <= 0.0 {
                return Err("Invalid fixture maximum".into());
            }
            mem.write(row + self.row_field(row == game, 0x4f8), &(maximum * ratio).to_le_bytes())?;
        }
        Ok(())
    }
    pub fn support_buff_slots(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<Vec<u32>> {
        let party = self
            .world_party(mem, game)?
            .ok_or("Missing support party")?;
        let mut slots = vec![];
        for actor in party.actors {
            let row = self.support_row(mem, game, actor.slot)?;
            let(_,rows)=self.aura_rows(mem,row)?;
            if rows.iter().any(|r|u32::from_le_bytes(r[12..16].try_into().unwrap())==spell&&f32::from_le_bytes(r[16..20].try_into().unwrap())>party.clock){
                slots.push(actor.slot);
            }
        }
        Ok(slots)
    }
    pub fn support_spell(self, mem: &dyn Mem, base: u64, ability: u64) -> Result<Option<u32>> {
        let data = mem.read(ability, 8)?;
        let pointer = u64::from_le_bytes(
            data.try_into()
                .map_err(|_| "Truncated native cast definition")?,
        );
        if self.supports_extensions() && pointer.checked_sub(base)==Some(self.contact117(0x72a0ca0)) {
            return Ok((mem.read(pointer+0x40,4)?==11426u32.to_le_bytes()).then_some(11426));
        }
        Ok(match pointer.checked_sub(base).map(|rva|self.baseline_rva(rva)) {
            Some(0x6cc5c78) => Some(139),
            Some(0x6cc72f8) => Some(774),
            Some(0x6cc7370) => Some(8936),
            Some(0x6cc5c00) => Some(17),
            Some(0x6cc61a0) => Some(324),
            Some(0x6cc73e8) => Some(467),
            Some(0x6cc5b88) => Some(15237),
            Some(0x6cc4c10) => Some(25912),
            Some(0x6cc4df0) => Some(635),
            Some(0x6cc6380) => Some(331),
            _ => None,
        })
    }
    pub fn support_caster(self, mem: &dyn Mem, game: u64) -> Result<SupportCaster> {
        let root = self.read_game(mem, game)?;
        if root.len() != 0xc48 {
            return Err("Truncated support context".into());
        }
        if root[0xc41] != 1 || u32::from_le_bytes(root[..4].try_into().unwrap()) == 2 {
            return Err("No native coop support context".into());
        }
        let word = |at| u64::from_le_bytes(root[at..at + 8].try_into().unwrap());
        let scalar = |at| f32::from_le_bytes(root[at..at + 4].try_into().unwrap());
        let slot = word(0xb40);
        if slot as usize >= crate::official_native::NATIVE_PLAYERS {
            return Err("Unsupported support actor".into());
        }
        let maximum = scalar(0x490);
        let hp = scalar(0x4f8);
        let position = std::array::from_fn(|n| scalar(0x4e0 + n * 4));
        if !maximum.is_finite()
            || maximum <= 0.0
            || !hp.is_finite()
            || position.iter().any(|x| !x.is_finite())
        {
            return Err("Invalid native support vitals".into());
        }
        let count = word(0x840);
        let pointer = word(0x838);
        if count > 512
            || (count > 0 && (pointer < 0x10000 || pointer.checked_add(count * 64).is_none()))
        {
            return Err("Invalid support perk extent".into());
        }
        let perks = if count > 0 {
            mem.read(pointer, (count * 64) as usize)?
        } else {
            vec![]
        };
        if perks.len() != (count * 64) as usize {
            return Err("Truncated support perks".into());
        }
        let mut talents = SupportTalents::default();
        for perk in perks.chunks_exact(64) {
            let scalar = |n| f32::from_le_bytes(perk[n..n + 4].try_into().unwrap());
            match perk[0] {
                0x14 => talents.overheal_cap = scalar(4),
                0x15 => {
                    talents.nature_coefficient = scalar(4);
                    talents.nature_radius = scalar(8);
                }
                _ => {}
            }
        }
        if ![
            talents.overheal_cap,
            talents.nature_coefficient,
            talents.nature_radius,
        ]
        .iter()
        .all(|v| v.is_finite() && (0.0..=1e6).contains(v))
        {
            return Err("Invalid outgoing support talents".into());
        }
        talents.maximum = maximum;
        Ok(SupportCaster {
            slot: slot as u32,
            hp,
            maximum,
            position,
            talents,
            guid: word(0x478),
        })
    }
    pub fn support_queue_count(self, mem: &dyn Mem, game: u64) -> Result<u64> {
        let header = mem.read(game.checked_add(self.game_field(0xa20)).ok_or("Support queue overflow")?, 24)?;
        if header.len() != 24 {
            return Err("Truncated support queue".into());
        }
        let at = |n| u64::from_le_bytes(header[n..n + 8].try_into().unwrap());
        let (cap, pointer, count) = (at(0), at(8), at(16));
        if count > cap
            || cap > 4096
            || (count > 0 && (pointer < 0x10000 || pointer.checked_add(count * 56).is_none()))
        {
            return Err("Invalid support queue extent".into());
        }
        Ok(count)
    }
    /// The last four bytes are padding in both reviewed emitters and consumer.
    /// Clear all new tags first, including native casts not eligible for routing,
    /// so allocator reuse cannot resurrect an old mod target.
    pub fn route_support(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        before: u64,
        spell: u32,
        target: Option<u32>,
        maximum: f32,
    ) -> Result<()> {
        let count = self.support_queue_count(mem, game)?;
        if count < before || count - before > 64 {
            return Err("Support queue changed during native cast".into());
        }
        let bytes = mem.read(game + self.game_field(0xa28), 8)?;
        let pointer = u64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| "Truncated support queue pointer")?,
        );
        for index in before..count {
            let address = pointer + index * 56;
            let row = mem.read(address, 56)?;
            if row.len() != 56 {
                return Err("Truncated support event".into());
            }
            mem.write(address + 52, &0u32.to_le_bytes())?;
            let field = |at| u32::from_le_bytes(row[at..at + 4].try_into().unwrap());
            if field(28) == spell && matches!(field(40), 13 | 14 | 15) {
                if let Some(target) = target {
                    if target as usize >= crate::official_native::NATIVE_PLAYERS {
                        return Err("Invalid mod support target".into());
                    }
                    let amount = if field(40) == 13 {
                        0.0
                    } else {
                        f32::from_bits(field(32)) * maximum
                    };
                    if !amount.is_finite() || !(0.0..=1e12).contains(&amount) {
                        return Err("Invalid support amount".into());
                    }
                    mem.write(address + 32, &amount.to_le_bytes())?;
                    mem.write(address + 52, &(ROUTE | target).to_le_bytes())?;
                }
            }
        }
        Ok(())
    }
    /// Direct allied heals update HP in the native consumer itself. Scale its
    /// fraction once; HoT budgets remain unscaled until each tick is consumed.
    pub fn scale_direct_heal_delivery(self, delivery: &mut SupportDelivery, factor: f32) -> Result<bool> {
        if !factor.is_finite() || !(0.0..=2.0).contains(&factor) { return Err("Invalid healing multiplier".into()); }
        let kind=u32::from_le_bytes(delivery.row[40..44].try_into().unwrap());
        if kind!=14 || delivery.duration!=0.0 { return Ok(false); }
        let fraction=f32::from_le_bytes(delivery.row[32..36].try_into().unwrap());
        let fraction=fraction*factor;let amount=delivery.amount*factor;
        if !fraction.is_finite()||fraction<0.||!amount.is_finite()||amount<0.{return Err("Invalid scaled healing amount".into())}
        delivery.row[32..36].copy_from_slice(&fraction.to_le_bytes());
        delivery.amount=amount;
        Ok(true)
    }
    pub fn support_delivery(
        self,
        mem: &dyn Mem,
        game: u64,
        descriptor: u64,
    ) -> Result<Option<SupportDelivery>> {
        let bytes = mem.read(descriptor, 56)?;
        let row: [u8; 56] = bytes
            .try_into()
            .map_err(|_| "Truncated native support delivery")?;
        let at = |n| u32::from_le_bytes(row[n..n + 4].try_into().unwrap());
        let tag = at(52);
        if tag & !(ROUTE_GROUP | ROUTE_TARGET) != ROUTE {
            return Ok(None);
        }
        let caster = u64::from_le_bytes(row[..8].try_into().unwrap());
        let spell = at(28);
        let target = if tag & ROUTE_GROUP != 0 {
            self.support_caster(mem, game)?.slot
        } else {
            tag & ROUTE_TARGET
        };
        if target as usize >= crate::official_native::NATIVE_PLAYERS
            || (tag & ROUTE_GROUP != 0 && tag & ROUTE_TARGET != 0)
        {
            return Err("Invalid routed support recipient".into());
        }
        let amount = f32::from_bits(at(32));
        if caster as usize >= crate::official_native::NATIVE_PLAYERS
            || !matches!(
                spell,
                635 | 331 | 139 | 774 | 8936 | 17 | 11426 | 324 | 467 | 15237 | 25912
            )
            || !matches!(at(40), 13 | 14 | 15)
            || !f32::from_bits(at(36)).is_finite()
            || !(0.0..=600.0).contains(&f32::from_bits(at(36)))
            || !amount.is_finite()
            || !(0.0..=1e12).contains(&amount)
        {
            return Err("Invalid routed support event".into());
        }
        if spell==11426 && target!=caster as u32 {return Err("Ice Barrier is caster-only".into())}
        let expected = match spell {
            324 | 467 => 13,
            17 | 11426 => 15,
            _ => 14,
        };
        if at(40) != expected
            || (tag & ROUTE_GROUP != 0) != matches!(spell, 15237 | 25912)
            || (at(40) == 13
                && (!f32::from_bits(at(44)).is_finite()
                    || !(0.0..=1e12).contains(&f32::from_bits(at(44)))
                    || !f32::from_bits(at(48)).is_finite()))
        {
            return Err("Routed support kind or payload mismatch".into());
        }
        let actor = self.support_caster(mem, game)?;
        let mut row = row;
        let mut eligible = true;
        if tag & ROUTE_GROUP != 0 {
            let origin = std::array::from_fn::<_, 3, _>(|n| f32::from_bits(at(16 + n * 4)));
            let radius = if spell == 15237 { 25.0f32 } else { 30.0 };
            let distance = actor
                .position
                .iter()
                .zip(origin)
                .map(|(a, b)| (*a - b).powi(2))
                .sum::<f32>();
            if origin.iter().any(|v| !v.is_finite())
                || caster as u32 == actor.slot
                || distance > radius * radius
            {
                eligible = false;
            }
        }
        if actor.slot == target && at(40) != 13 {
            let fraction = amount / actor.maximum;
            if !fraction.is_finite() {
                return Err("Support fraction overflow".into());
            }
            row[32..36].copy_from_slice(&fraction.to_le_bytes());
        }
        Ok(Some(SupportDelivery {
            guid: u64::from_le_bytes(row[8..16].try_into().unwrap()),
            caster: caster as u32,
            target,
            spell,
            amount,
            duration: f32::from_bits(at(36)),
            eligible,
            shield: at(40) == 15,
            row,
        }))
    }
    pub(crate) fn support_row(self, mem: &dyn Mem, game: u64, slot: u32) -> Result<u64> {
        if mem.read(game+self.game_field(0xc41),1)?==[0] {
            return if slot==0 && self.run_source(mem,game)?.is_some(){Ok(game)}else{Err("Solo has no other support actor".into())};
        }
        let current = mem.read(game + self.game_field(0xb40), 8)?;
        let current = u64::from_le_bytes(
            current
                .try_into()
                .map_err(|_| "Truncated current support slot")?,
        );
        if current as usize >= crate::official_native::NATIVE_PLAYERS {
            return Err("Invalid current support slot".into());
        }
        if u64::from(slot) == current {
            return Ok(game);
        }
        let slot = if slot == 0 { current as u32 } else { slot };
        let h = mem.read(
            game.checked_add(self.game_field(0x980)).ok_or("Support vector overflow")?,
            16,
        )?;
        if h.len() != 16 {
            return Err("Truncated support vector".into());
        }
        let p = u64::from_le_bytes(h[..8].try_into().unwrap());
        let count = u64::from_le_bytes(h[8..].try_into().unwrap());
        if slot as usize >= crate::official_native::NATIVE_PLAYERS
            || u64::from(slot) > count
            || count > 4
            || p < 0x10000
            || p.checked_add(count * self.context_size()).is_none()
        {
            return Err("Invalid support context binding".into());
        }
        let row = p + u64::from(slot - 1) * self.context_size();
        let tag = mem.read(row, 4)?;
        if tag.len() != 4 || tag == 2u32.to_le_bytes() {
            return Err("Support source context is inactive".into());
        }
        Ok(row)
    }
    pub fn fixture_cast_actor(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        ability: u64,
    ) -> Result<bool> {
        if io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Fixture cast requires restored host context".into());
        }
        if slot == 0 {
            return io.cast_actor(game, ability);
        }
        let row = self.support_row(io, game, slot)?;
        io.swap_actor(game, row)?;
        io.write(game + self.game_field(0xb40), &u64::from(slot).to_le_bytes())?;
        let result = io.cast_actor(game, ability).and_then(|result| {
            io.flush_actor(game)?;
            Ok(result)
        });
        let row = self.support_row(io, game, 0)?;
        io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
        io.swap_actor(game, row)?;
        result
    }
    /// Diagnostic cast: derive this spell's native parameters in its caster's
    /// swapped context. Do not reuse another spell's damage/duration coefficients.
    pub fn fixture_cast_resolved(self,io:&dyn SupportCalls,game:u64,slot:u32,ability:u64)->Result<bool>{
        if io.read(game+self.game_field(0xb40),8)?!=0u64.to_le_bytes(){return Err("Resolved cast requires restored host".into())}
        let row=self.support_row(io,game,slot)?;
        if slot!=0{io.swap_actor(game,row)?;io.write(game+self.game_field(0xb40),&u64::from(slot).to_le_bytes())?;}
        let result=(||{
            let mut owned=io.read(ability,0x68)?;if owned.len()!=0x68{return Err("Truncated diagnostic ability".into())}
            let definition=u64::from_le_bytes(owned[..8].try_into().unwrap());
            io.resolve_glyph_modifiers(owned.as_mut_ptr()as u64+0x38,game+self.field117(crate::official117::GLYPH_PERKS),definition)?;
            let result=io.cast_actor(game,owned.as_ptr()as u64)?;io.flush_actor(game)?;Ok(result)
        })();
        if slot!=0{let row=self.support_row(io,game,0)?;io.write(game+self.game_field(0xb40),&0u64.to_le_bytes())?;io.swap_actor(game,row)?;}
        result
    }
    fn healing_total(self, mem: &dyn Mem, game: u64) -> Result<f64> {
        let h = mem.read(game.checked_add(self.game_field(0x6d8)).ok_or("Healing meter overflow")?, 24)?;
        if h.len() != 24 {
            return Err("Truncated healing meter".into());
        }
        let at = |n| u64::from_le_bytes(h[n..n + 8].try_into().unwrap());
        let (cap, pointer, count) = (at(0), at(8), at(16));
        if count > cap
            || cap > 4096
            || (count > 0 && (pointer < 0x10000 || pointer.checked_add(count * 80).is_none()))
        {
            return Err("Invalid healing meter extent".into());
        }
        if count == 0 {
            return Ok(0.0);
        }
        let rows = mem.read(pointer, (count * 80) as usize)?;
        if rows.len() != (count * 80) as usize {
            return Err("Truncated healing rows".into());
        }
        let total = rows
            .chunks_exact(80)
            .map(|r| f64::from_le_bytes(r[64..72].try_into().unwrap()))
            .sum::<f64>();
        if !total.is_finite() || total < 0.0 {
            return Err("Invalid native healing total".into());
        }
        Ok(total)
    }
    pub fn credit_support(
        self,
        io: &dyn SupportCalls,
        game: u64,
        slot: u32,
        spell: u32,
        amount: f32,
    ) -> Result<(f64, f64)> {
        if io.read(game, 4)? == 2u32.to_le_bytes()
            || io.read(game.checked_add(self.game_field(0xc41)).ok_or("Support game overflow")?, 1)? != [1]
        {
            return Err("Support game is no longer active".into());
        }
        if !amount.is_finite() || !(0.0..=1e12).contains(&amount) {
            return Err("Invalid support meter credit".into());
        }
        if amount == 0.0 {
            return Ok((0.0, 0.0));
        }
        let current = io.read(game + self.game_field(0xb40), 8)?;
        if current != 0u64.to_le_bytes() {
            return Err("Support meter requires restored host context".into());
        }
        let credit = || -> Result<(f64, f64)> {
            let before = self.healing_total(io, game)?;
            io.healing_meter(game, spell, amount)?;
            let after = self.healing_total(io, game)?;
            if (after - before - f64::from(amount)).abs() > 0.02 {
                return Err("Native caster healing meter differs from effective healing".into());
            }
            Ok((before, after))
        };
        if slot == 0 {
            return credit();
        }
        let row = self.support_row(io, game, slot)?;
        io.swap_actor(game, row)?;
        io.write(game + self.game_field(0xb40), &u64::from(slot).to_le_bytes())?;
        let result = credit().and_then(|value| {
            io.flush_actor(game)?;
            Ok(value)
        });
        // Rebind the saved host row while the guest is still loaded.
        let row = self.support_row(io, game, 0)?;
        io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
        io.swap_actor(game, row)?;
        result
    }
}
