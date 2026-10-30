//! Native wallet projection and currency calls through the selected adapter.
use super::Adapter;
use crate::{
    official::{CurrencyMemory, Entry, MutableMemory},
    memory::Mem,
    Result,
};
impl Adapter {
    pub fn chest_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x5963be0),
                bytes: self.expected(0x5963be0, &[
                0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48, 0x81, 0xec, 0xb0,
                0, 0, 0,
            ]),
        }
    }
    pub fn convert_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x597e580),
                bytes: self.expected(0x597e580, &[0x56, 0x53, 0x48, 0x83, 0xec, 0x28]),
        }
    }
    /// Disposable fixture unlock only; the production coordinator never calls it.
    pub fn fixture_conversion_flag(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        enabled: bool,
    ) -> Result<bool> {
        let address = game.checked_add(self.game_field(0x2a6)).ok_or("Conversion flag overflow")?;
        let old = mem.read(address, 1)?;
        if !matches!(old.as_slice(), [0] | [1]) {
            return Err("Invalid conversion unlock flag".into());
        }
        mem.write(address, &[u8::from(enabled)])?;
        Ok(old == [1])
    }
    pub fn conversion_ready(self, mem: &dyn Mem, game: u64) -> Result<bool> {
        let flag = |offset: u64| {
            mem.read(
                game.checked_add(self.game_field(offset))
                    .ok_or("Conversion field overflow")?,
                1,
            )
        };
        Ok(flag(0xc46)? == [2] && flag(0x2a6)? == [1] && flag(0xc44)? != [0xff])
    }
    pub fn chest_state(self, mem: &dyn Mem, game: u64) -> Result<(usize, u32, u32)> {
        let read = |offset: u64, n| {
            let b = mem.read(game.checked_add(self.game_field(offset)).ok_or("Chest field overflow")?, n)?;
            if b.len() != n {
                return Err("Truncated chest state".to_string());
            }
            Ok(b)
        };
        let slot = u64::from_le_bytes(read(0xb40, 8)?.try_into().unwrap());
        if slot as usize >= crate::official_native::NATIVE_PLAYERS {
            return Err("Chest actor exceeds mod capacity".into());
        }
        Ok((
            slot as usize,
            u32::from_le_bytes(read(0xbf4, 4)?.try_into().unwrap()),
            u32::from_le_bytes(read(0x7b0, 4)?.try_into().unwrap()),
        ))
    }
    pub fn project_current_chest(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        cost: u32,
        opened: u32,
    ) -> Result<()> {
        let tag = mem.read(game, 4)?;
        if tag == 2u32.to_le_bytes() {
            return Ok(());
        }
        if tag.len() != 4 {
            return Err("Truncated chest actor".into());
        }
        for (offset, value) in [(0xbf4u64, cost), (0x7b0, opened)] {
            let address = game.checked_add(self.game_field(offset)).ok_or("Chest metadata overflow")?;
            if mem.read(address, 4)? != value.to_le_bytes() {
                mem.write(address, &value.to_le_bytes())?;
            }
        }
        Ok(())
    }
    pub fn deliver_chests(
        self,
        mem: &dyn MutableMemory,
        game: u64,
        slot: u32,
        amount: u32,
        cost: u32,
        opened: u32,
    ) -> Result<bool> {
        let (current, _, _) = self.chest_state(mem, game)?;
        if current != 0 || slot as usize >= crate::official_native::NATIVE_PLAYERS {
            return Err("Chest projection requires restored host context".into());
        }
        let row = if slot == 0 {
            game
        } else {
            let h = mem.read(game.checked_add(self.game_field(0x980)).ok_or("Chest vector overflow")?, 16)?;
            if h.len() != 16 {
                return Err("Truncated chest vector".into());
            }
            let pointer = u64::from_le_bytes(h[..8].try_into().unwrap());
            let count = u64::from_le_bytes(h[8..].try_into().unwrap());
            if count > 4
                || u64::from(slot) > count
                || pointer < 0x10000
                || pointer.checked_add(count * self.context_size()).is_none()
            {
                return Err("Invalid chest context extent".into());
            }
            pointer + u64::from(slot - 1) * self.context_size()
        };
        let tag = mem.read(row, 4)?;
        if tag == 2u32.to_le_bytes() {
            return Ok(false);
        }
        if tag.len() != 4 {
            return Err("Truncated chest actor".into());
        }
        let phase = mem.read(row + self.row_field(slot == 0, if slot == 0 { 0xc46 } else { 0x97c }), 1)?;
        if !matches!(phase.as_slice(), [1] | [2] | [3]) {
            return Ok(false);
        }
        let (bank, price, counter) = if slot == 0 {
            (0xbb4, 0xbf4, 0x7b0)
        } else {
            (0x92c, 0x96c, 0x6f0)
        };
        let bank = self.row_field(slot == 0, bank);
        let bytes = mem.read(row + bank, 4)?;
        let next = u32::from_le_bytes(bytes.try_into().map_err(|_| "Truncated item reward bank")?)
            .checked_add(amount)
            .ok_or("Item reward bank overflow")?;
        for (offset, value) in [(price, cost), (counter, opened)] {
            let offset = self.row_field(slot == 0, offset);
            if mem.read(row + offset, 4)? != value.to_le_bytes() {
                mem.write(row + offset, &value.to_le_bytes())?;
            }
        }
        if amount > 0 {
            mem.write(row + bank, &next.to_le_bytes())?;
        }
        Ok(true)
    }
    pub fn gold_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x5920470),
                bytes: self.expected(0x5920470, &[0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x83, 0xec, 0x38]),
        }
    }
    pub fn swap_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x5920ed0),
                bytes: self.expected(0x5920ed0, &[
                0x0f, 0x10, 0x81, 0xb0, 0x01, 0, 0, 0x0f, 0x10, 0x89, 0xc0, 0x01, 0, 0,
            ]),
        }
    }
    pub fn currency_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x58092f0),
                bytes: self.expected(0x58092f0, &[0x56, 0x57, 0x53, 0x48, 0x83, 0xec, 0x70]),
        }
    }
    pub fn gold_source(self, return_rva: u64) -> asx_gameplay::official::Award {
        match self.baseline_rva(return_rva) {
            // Explicit skip paths: native UI, local command and coop command.
            0x58dc5f7 | 0x58ee94e | 0x5978a43 | 0x597e5c0 => {
                asx_gameplay::official::Award::SkippedReward
            }
            // Automatic reward conversion retains its existing party normalization.
            0x597ee62 => asx_gameplay::official::Award::ConvertedReward,
            _ => asx_gameplay::official::Award::World,
        }
    }
    pub fn racial_gold_bonus(self, race: u8) -> f32 {
        if race == 1 {
            0.2
        } else {
            0.0
        }
    }
    pub fn wallet_seed(
        self,
        mem: &dyn Mem,
        game: u64,
    ) -> Result<asx_gameplay::official::SharedWallet> {
        let (gold, _, _) = self.current_gold(mem, game)?;
        let bytes = mem.read(game.checked_add(self.game_field(0x7ac)).ok_or("Earned gold overflow")?, 4)?;
        let earned = u32::from_le_bytes(bytes.try_into().map_err(|_| "Truncated earned gold")?);
        gold.checked_mul(10000).ok_or("Native coinage overflow")?;
        Ok(asx_gameplay::official::SharedWallet { gold, earned })
    }
    pub fn current_gold(self, mem: &dyn Mem, game: u64) -> Result<(u32, f32, u8)> {
        // Native revisions insert fields within this span. Translate each
        // scalar independently instead of assuming a contiguous legacy layout.
        let read = |offset, length| {
            let address = game.checked_add(self.game_field(offset))
                .ok_or("Gold actor overflow")?;
            let bytes = mem.read(address, length)?;
            if bytes.len() != length { return Err("Truncated gold actor".to_string()); }
            Ok(bytes)
        };
        let gold = u32::from_le_bytes(read(0x500, 4)?.try_into().unwrap());
        let multiplier = f32::from_le_bytes(read(0x4c0, 4)?.try_into().unwrap());
        if !multiplier.is_finite() || multiplier < 0.0 {
            return Err("Invalid native gold multiplier".into());
        }
        Ok((gold, multiplier, read(0x32d, 1)?[0]))
    }

    pub fn is_currency(self, game: u64, map: u64, field: u16) -> bool {
        game.checked_add(self.game_field(0xb20)) == Some(map) && field == 0x498
    }
    pub fn project_current_wallet(
        self,
        mem: &dyn CurrencyMemory,
        game: u64,
        wallet: asx_gameplay::official::SharedWallet,
    ) -> Result<()> {
        let tag = mem.read(game, 4)?;
        if tag == 2u32.to_le_bytes() {
            return Ok(());
        }
        if tag.len() != 4 {
            return Err("Truncated currency actor union".into());
        }
        wallet
            .gold
            .checked_mul(10000)
            .ok_or("Native coinage overflow")?;
        let gold = game.checked_add(self.game_field(0x500))
            .ok_or("Native gold address overflow")?;
        let earned = game.checked_add(self.game_field(0x7ac))
            .ok_or("Native gold address overflow")?;
        let map = game.checked_add(self.game_field(0xb20))
            .ok_or("Native coinage owner overflow")?;
        let old_gold = mem.read(gold, 4)?;
        let old_earned = mem.read(earned, 4)?;
        if old_gold != wallet.gold.to_le_bytes() {
            mem.write(gold, &wallet.gold.to_le_bytes())?;
            mem.coinage(map, wallet.gold)?;
        }
        if old_earned != wallet.earned.to_le_bytes() {
            mem.write(earned, &wallet.earned.to_le_bytes())?;
        }
        Ok(())
    }
    /// Safe restored-host boundary only. Native BTree field setters allocate but
    /// never enter the simulation or mutate the context vector. Re-read each row
    /// before using it; no gameplay-state mutex may span a setter call.
    pub fn project_wallet(
        self,
        mem: &dyn CurrencyMemory,
        game: u64,
        wallet: asx_gameplay::official::SharedWallet,
    ) -> Result<()> {
        let snapshot = self
            .world_party(mem, game)?
            .ok_or("No active native party for wallet")?;
        let header = mem.read(game.checked_add(self.game_field(0x980)).ok_or("Wallet vector overflow")?, 16)?;
        if header.len() != 16 {
            return Err("Truncated wallet context header".into());
        }
        let pointer = u64::from_le_bytes(header[..8].try_into().unwrap());
        let count = u64::from_le_bytes(header[8..].try_into().unwrap());
        if count > 4
            || (count > 0 && (pointer < 0x10000 || pointer.checked_add(count * self.context_size()).is_none()))
        {
            return Err("Invalid wallet row extent".into());
        }
        self.project_current_wallet(mem, game, wallet)?;
        for actor in snapshot.actors.into_iter().filter(|a| a.slot != 0) {
            let row = pointer
                .checked_add(u64::from(actor.slot - 1) * self.context_size())
                .ok_or("Wallet row overflow")?;
            if u64::from(actor.slot) > count {
                return Err("Wallet snapshot membership changed".into());
            }
            if mem.read(row + self.row_field(row == game, 0x500), 4)? != wallet.gold.to_le_bytes() {
                mem.write(row + self.row_field(row == game, 0x500), &wallet.gold.to_le_bytes())?;
                mem.coinage(row + self.row_field(row == game, 0x8c0), wallet.gold)?;
            }
            if mem.read(row + self.row_field(row == game, 0x6ec), 4)? != wallet.earned.to_le_bytes() {
                mem.write(row + self.row_field(row == game, 0x6ec), &wallet.earned.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

impl Adapter {
    pub fn item_choice_entry(self)->Option<Entry>{
        Some(Entry{rva:crate::current::ITEM_CHOICE_RVA,bytes:crate::current::ITEM_CHOICE_BYTES})
    }
    /// Present an already earned item, without incrementing its bank or charging gold.
    /// Invoke only at a restored-host boundary; callbacks may replace private rows.
    pub fn present_pending_item(self,io:&dyn crate::official::SupportCalls,game:u64,slot:u32,guid:u64)->Result<bool>{
        if self.item_choice_entry().is_none(){return Ok(false)}
        if io.read(game+self.game_field(0xc41),1)?!=[1]{return Err("Item presentation requires cooperative play".into())}
        if io.read(game+self.game_field(0xb40),8)?!=0u64.to_le_bytes(){return Err("Item presentation requires restored host".into())}
        let row=self.support_row(io,game,slot)?;
        if io.read(row+self.row_field(row==game,0x478),8)?!=guid.to_le_bytes(){return Ok(false)}
        if io.read(row+self.row_field(row==game,if slot==0{0xc46}else{0x97c}),1)?!=[1]{return Ok(false)}
        let bank=u32::from_le_bytes(io.read(row+self.row_field(row==game,if slot==0{0xbb4}else{0x92c}),4)?.try_into().map_err(|_|"Truncated item presentation bank")?);
        if bank==0{return Ok(false)}
        if slot!=0{io.swap_actor(game,row)?;io.write(game+self.game_field(0xb40),&u64::from(slot).to_le_bytes())?;}
        // Flush is required to deliver a guest choice; identical UI re-emissions
        // are presentation, not an additional item grant.
        let result=io.open_item_choice(game).and_then(|_|io.flush_actor(game));
        if slot!=0{let saved=self.support_row(io,game,0)?;io.write(game+self.game_field(0xb40),&0u64.to_le_bytes())?;io.swap_actor(game,saved)?;}
        result.map(|_|true)
    }
}

impl Adapter {
    /// Permanent armory stats, distinct from run jewelry and the total multiplier.
    /// support_row also handles a guest temporarily swapped into Game.
    pub fn armory_gold_bonus(self,mem:&dyn crate::memory::Mem,game:u64,slot:u32,guid:u64)->Result<Option<f32>>{
        let row=self.support_row(mem,game,slot)?;
        if guid==0 || mem.read(row+self.row_field(row==game,0x478),8)?!=guid.to_le_bytes(){return Err("Armory gold owner changed".into())}
        let header=mem.read(row+self.row_field(row==game,crate::current::ARMORY_STATS_HEADER),24)?;
        if header.len()!=24{return Err("Truncated armory stat header".into())}
        let q=|at|u64::from_le_bytes(header[at..at+8].try_into().unwrap());
        let(cap,ptr,count)=(q(0),q(8),q(16));
        if count>cap || cap>4096 || count>64 || (count>0&&(ptr<0x10000||ptr.checked_add(count*8).is_none())){return Err("Invalid armory stat extent".into())}
        if count==0{return Ok(Some(0.0))}
        let bytes=mem.read(ptr,(count*8)as usize)?;
        if bytes.len()!=count as usize*8{return Err("Truncated armory stats".into())}
        let mut bonus=0.0f32;
        for stat in bytes.chunks_exact(8){
            if u32::from_le_bytes(stat[..4].try_into().unwrap())==crate::current::ARMORY_GOLD_STAT{
                let value=f32::from_le_bytes(stat[4..].try_into().unwrap());
                if !value.is_finite(){return Err("Invalid armory gold bonus".into())}
                bonus+=value;
            }
        }
        if !bonus.is_finite(){return Err("Armory gold bonus overflow".into())}
        Ok(Some(bonus))
    }
}

impl Adapter {
    /// Controlled diagnostic only: add an armory stat to a disposable native run.
    /// Old backing arrays stay alive until the owned test process exits; called
    /// once per fixture member, never from production or a recurring observer.
    pub fn fixture_armory_gold(self,io:&dyn crate::official::SupportCalls,game:u64,slot:u32,guid:u64,bonus:f32)->Result<()>{
        self.armory_gold_bonus(io,game,slot,guid)?.ok_or("Unreviewed armory gold fixture")?;
        if !bonus.is_finite()||!(0.0..=1.0).contains(&bonus){return Err("Invalid armory gold fixture amount".into())}
        let row=self.support_row(io,game,slot)?;let at=row+self.row_field(row==game,crate::current::ARMORY_STATS_HEADER);
        let h=io.read(at,24)?;let ptr=u64::from_le_bytes(h[8..16].try_into().unwrap());let count=u64::from_le_bytes(h[16..24].try_into().unwrap());
        if count>=64{return Err("Armory gold fixture extent".into())}
        let mut data=if count==0{vec![]}else{io.read(ptr,count as usize*8)?};
        data.extend_from_slice(&crate::current::ARMORY_GOLD_STAT.to_le_bytes());data.extend_from_slice(&bonus.to_le_bytes());
        let owned=io.allocate_native(&data)?;let n=count+1;
        io.write(at,&[n.to_le_bytes(),owned.to_le_bytes(),n.to_le_bytes()].concat())
    }
}

impl Adapter {
    /// Disposable reward-action fixture. Keep counters, selected ABI and native
    /// caller identities inside the reviewed adapter, never in wallet policy.
    pub fn reward_choice_kind(self,m:&dyn Mem,game:u64)->Result<Option<u8>>{
        if m.read(game+self.game_field(0xc46),1)?!=[2]{return Ok(None)}
        let kind=m.read(game+self.game_field(0xc44),1)?[0];
        Ok((kind<=1).then_some(kind))
    }
    pub unsafe fn fixture_reward_action(self,m:&dyn MutableMemory,base:usize,game:u64,action:u8)->Result<()>{
        self.reward_choice_kind(m,game)?.ok_or("Reward fixture requires native choices")?;
        let (rva,bytes)=if action==2{let e=self.convert_entry();(e.rva,e.bytes)}else{(self.contact117(crate::current::GLYPH_REROLL_RVA),self.extension_bytes117(0x5c09350,crate::current::GLYPH_REROLL_BYTES))};
        if m.read(base as u64+rva,bytes.len())?!=bytes{return Err("Reward fixture action binding".into())}
        if action==2{self.fixture_conversion_flag(m,game,true)?;}else if action<=1{m.write(game+self.field117(crate::current::GLYPH_REROLLS),&u32::from(action==0).to_le_bytes())?;}else{return Err("Invalid reward fixture action".into())}
        let call:unsafe extern "system" fn(u64)=std::mem::transmute(base+rva as usize);call(game);Ok(())
    }
}
