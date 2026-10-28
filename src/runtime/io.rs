//! Own-process native heap access. Calls happen only at reviewed owner-thread
//! boundaries, with no gameplay-state lock held across a game allocator call.
use crate::proc::Local;
use asx_game::{
    official::{MutableMemory, SelectedAdapter},
    memory::Mem,
};
#[cfg(feature = "test-hooks")]
static RAPTURE_COOLDOWNS: std::sync::Mutex<Vec<(u64, f32)>> = std::sync::Mutex::new(vec![]);
pub struct Access<'a> {
    pub local: &'a Local,
    pub adapter: SelectedAdapter,
}
impl Mem for Access<'_> {
    fn read(&self, address: u64, length: usize) -> Result<Vec<u8>, String> {
        Mem::read(self.local, address, length)
    }
}
impl MutableMemory for Access<'_> {
    fn write(&self, address: u64, bytes: &[u8]) -> Result<(), String> {
        self.local.write(address as usize, bytes)
    }
    fn allocate_native(&self, bytes: &[u8]) -> Result<u64, String> {
        type Alloc = unsafe extern "system" fn(usize, usize) -> usize;
        let entry = self
            .local
            .base
            .checked_add(self.adapter.heap()[0].rva as usize)
            .ok_or("Native heap address overflow")?;
        let alloc: Alloc = unsafe { std::mem::transmute(entry) };
        let pointer = unsafe { alloc(bytes.len(), 1) };
        if pointer < 0x10000 {
            return Err("Native HUD allocation failed".into());
        }
        if let Err(error) = self.local.write(pointer, bytes) {
            self.free_native(pointer as u64, bytes.len())?;
            return Err(error);
        }
        Ok(pointer as u64)
    }
    fn free_native(&self, address: u64, capacity: usize) -> Result<(), String> {
        if capacity == 0 {
            return Ok(());
        }
        if address < 0x10000 || capacity > 1_048_576 {
            return Err("Invalid native HUD release".into());
        }
        type Free = unsafe extern "system" fn(usize, usize, usize);
        let entry = self
            .local
            .base
            .checked_add(self.adapter.heap()[1].rva as usize)
            .ok_or("Native heap address overflow")?;
        let free: Free = unsafe { std::mem::transmute(entry) };
        unsafe { free(address as usize, capacity, 1) };
        Ok(())
    }
}

impl asx_game::official::CurrencyMemory for Access<'_> {
    fn coinage(&self, map: u64, gold: u32) -> Result<(), String> {
        let copper = gold.checked_mul(10000).ok_or("Native coinage overflow")?;
        type Set = unsafe extern "system" fn(u64, u16, u32) -> usize;
        let set: Set = unsafe {
            std::mem::transmute(self.local.base + self.adapter.currency_entry().rva as usize)
        };
        unsafe { set(map, 0x498, copper) };
        Ok(())
    }
}

impl asx_game::official::SupportCalls for Access<'_> {
    fn loot_send(&self,server:u64,peer:Option<u64>,opcode:u16,bytes:&[u8])->Result<(),String>{
        let entry=self.adapter.loot_send_entry(peer.is_some()).ok_or("Loot transport unavailable")?;
        if self.local.read(self.local.base+entry.rva as usize,entry.bytes.len())?!=entry.bytes{return Err("Native loot transport bytes changed".into())}
        if let Some(peer)=peer {
            let ptr=self.allocate_native(bytes)?;
            let mut packet=[0u8;32];
            for (at,v) in [(0,bytes.len()as u64),(8,ptr),(16,bytes.len()as u64)]{packet[at..at+8].copy_from_slice(&v.to_le_bytes());}
            packet[24..26].copy_from_slice(&opcode.to_le_bytes());
            type Send=unsafe extern "system" fn(u64,u64,u64,*const u8);
            let call:Send=unsafe{std::mem::transmute(self.local.base+entry.rva as usize)};
            // Native output consumes the owned payload, including disconnected peers.
            unsafe{call(server,1,peer,packet.as_ptr())};
        }else{
            type Send=unsafe extern "system" fn(u64,u64,u16,*const u8,usize);
            let call:Send=unsafe{std::mem::transmute(self.local.base+entry.rva as usize)};
            // Native input clones the borrowed bytes into its reliable stream queue.
            unsafe{call(server,0,opcode,bytes.as_ptr(),bytes.len())};
        }
        Ok(())
    }
    fn loot_ui(&self,server:u64,value:&serde_json::Value)->Result<(),String>{
        let (context,entry)=self.adapter.loot_ui_sink(self,server)?;
        let bytes=serde_json::to_vec(value).map_err(|e|e.to_string())?;
        type Emit=unsafe extern "system" fn(u64,*const u8,usize);
        let call:Emit=unsafe{std::mem::transmute(entry as usize)};
        unsafe{call(context,bytes.as_ptr(),bytes.len())};Ok(())
    }

    fn open_item_choice(&self,game:u64)->Result<(),String>{
        let entry=self.adapter.item_choice_entry().ok_or("Native item presentation unavailable")?;
        if self.local.read(self.local.base+entry.rva as usize,entry.bytes.len())?!=entry.bytes{return Err("Native item presentation bytes changed".into())}
        type Open=unsafe extern "system" fn(u64,u8,u8);
        let call:Open=unsafe{std::mem::transmute(self.local.base+entry.rva as usize)};
        unsafe{call(game,1,1)};Ok(())
    }

    fn resolve_glyph_modifiers(&self,output:u64,perks:u64,definition:u64)->Result<(),String>{
        type Resolve=unsafe extern "system" fn(u64,u64,u64)->u64;
        let entry=self.adapter.glyph_entries().get(2).copied().ok_or("Glyph calls unavailable")?;
        let call:Resolve=unsafe{std::mem::transmute(self.local.base+entry.rva as usize)};
        unsafe{call(output,perks,definition)};Ok(())
    }

    fn native_boss_health(&self, game: u64, index: u64) -> Result<f32, String> {
        type Call = unsafe extern "system" fn(u64, u64) -> f32;
        let call: Call = unsafe {
            std::mem::transmute(self.local.base + self.adapter.scaling_probe_entry().rva as usize)
        };
        Ok(unsafe { call(game, index) })
    }

    fn award_experience(&self, game: u64, amount: u32) -> Result<(), String> {
        type Award = unsafe extern "system" fn(u64, u32);
        let call: Award =
            unsafe { std::mem::transmute(self.local.base + self.adapter.award_xp().rva as usize) };
        unsafe { call(game, amount) };
        Ok(())
    }
    fn emit_support_effect(
        &self,
        game: u64,
        spell: u32,
        data: &[u8; 12],
        duration: f32,
    ) -> Result<(), String> {
        type Emit = unsafe extern "system" fn(u64, u32, *const u8, f32);
        let call: Emit = unsafe {
            std::mem::transmute(
                self.local.base + self.adapter.support_emit_entries()[0].rva as usize,
            )
        };
        unsafe { call(game, spell, data.as_ptr(), duration) };
        Ok(())
    }
    fn emit_support_heal(&self, game: u64, spell: u32, fraction: f32) -> Result<(), String> {
        type Emit = unsafe extern "system" fn(u64, u32, f32, f32);
        let call: Emit = unsafe {
            std::mem::transmute(
                self.local.base + self.adapter.support_emit_entries()[1].rva as usize,
            )
        };
        unsafe { call(game, spell, fraction, 0.0) };
        Ok(())
    }
    fn refresh_support(&self, game: u64) -> Result<(), String> {
        type Refresh = unsafe extern "system" fn(u64);
        for entry in &self.adapter.support_emit_entries()[2..] {
            let call: Refresh =
                unsafe { std::mem::transmute(self.local.base + entry.rva as usize) };
            unsafe { call(game) };
        }
        Ok(())
    }
    fn support_hit(&self, game: u64, guid: u64, amount: f32, spell: u32) -> Result<(), String> {
        type Damage = unsafe extern "system" fn(u64, u64, f32, u32, u32, bool, bool);
        let call: Damage = unsafe {
            std::mem::transmute(
                self.local.base + self.adapter.support_proc_entries()[0].rva as usize,
            )
        };
        unsafe { call(game, guid, amount, spell, 3, false, false) };
        Ok(())
    }
    fn support_hurt(&self, game: u64, amount: f32, enemy: u64, melee: bool) -> Result<(), String> {
        type Hurt = unsafe extern "system" fn(u64, f32, u64, bool);
        let call: Hurt = unsafe {
            std::mem::transmute(self.local.base + self.adapter.hurt_entry().rva as usize)
        };
        unsafe { call(game, amount, enemy, melee) };
        Ok(())
    }
    fn support_damage(&self, game: u64, guid: u64, amount: f32) -> Result<(), String> {
        type Damage = unsafe extern "system" fn(u64, u64, f32, u32, u32, bool, bool);
        let call: Damage = unsafe {
            std::mem::transmute(
                self.local.base + self.adapter.support_proc_entries()[0].rva as usize,
            )
        };
        // The native Nature damage callback itself owns all hit/kill/packet logic.
        unsafe { call(game, guid, amount, 740, 3, false, false) };
        Ok(())
    }
    fn shield_proc(
        &self,
        game: u64,
        event: &[u8; 40],
        origin: [f32; 3],
        recipient: u64,
    ) -> Result<(), String> {
        type Proc = unsafe extern "system" fn(u64, *const u8, u64);
        let call: Proc = unsafe {
            std::mem::transmute(
                self.local.base + self.adapter.support_proc_entries()[1].rva as usize,
            )
        };
        #[cfg(feature = "test-hooks")]
        let source = self.adapter.support_caster(self.local, game)?.guid;
        #[cfg(feature = "test-hooks")]
        let next = RAPTURE_COOLDOWNS
            .lock()
            .ok()
            .and_then(|v| v.iter().find(|(g, _)| *g == source).map(|(_, t)| *t))
            .unwrap_or(0.0);
        #[cfg(feature = "test-hooks")]
        let loan = if std::env::var("AZCOOP_TEST_OFFICIAL_SUPPORT_HOST").as_deref() == Ok("1") {
            Some(self.adapter.fixture_perk_loan(
                self,
                game,
                self.local.base as u64,
                std::env::var("AZCOOP_TEST_OFFICIAL_RAPTURE").as_deref() == Ok("1"),
                next,
            )?)
        } else {
            None
        };
        super::support::with_recipient(game, recipient, || {
            super::support::with_origin(origin, || unsafe { call(game, event.as_ptr(), 0) })
        });
        #[cfg(feature = "test-hooks")]
        if let Some(loan) = loan {
            if let Some(next) = self.adapter.fixture_restore_perks(self, game, loan)? {
                if let Ok(mut values) = RAPTURE_COOLDOWNS.lock() {
                    values.retain(|(g, _)| *g != source);
                    if values.len() < 64 {
                        values.push((source, next));
                    }
                }
            }
        }
        Ok(())
    }
    fn cast_actor(&self, game: u64, ability: u64) -> Result<bool, String> {
        type Cast = unsafe extern "system" fn(u64, u64) -> bool;
        let call: Cast = unsafe {
            std::mem::transmute(self.local.base + self.adapter.support_entries()[0].rva as usize)
        };
        Ok(unsafe { call(game, ability) })
    }
    fn swap_actor(&self, game: u64, row: u64) -> Result<(), String> {
        type Swap = unsafe extern "system" fn(u64, u64);
        let call: Swap = unsafe {
            std::mem::transmute(self.local.base + self.adapter.swap_entry().rva as usize)
        };
        unsafe { call(game, row) };
        Ok(())
    }
    fn flush_actor(&self, game: u64) -> Result<(), String> {
        type Flush = unsafe extern "system" fn(u64);
        let call: Flush = unsafe {
            std::mem::transmute(self.local.base + self.adapter.flush_entry().rva as usize)
        };
        unsafe { call(game) };
        Ok(())
    }
    fn healing_meter(&self, game: u64, spell: u32, amount: f32) -> Result<(), String> {
        type Meter = unsafe extern "system" fn(u64, f32, u32, *const u64);
        let call: Meter = unsafe {
            std::mem::transmute(self.local.base + self.adapter.support_entries()[3].rva as usize)
        };
        let metadata = [(-2i64) as u64, 0, 0, 0, 0, 0];
        unsafe { call(game, amount, spell, metadata.as_ptr()) };
        Ok(())
    }
}
