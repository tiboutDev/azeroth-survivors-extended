//! Scoped ownership of the exact native Polymorph rupture branch.
//! The triggering hit has already committed; no hit or explosion is replayed.
use super::*;
use crate::official::{Entry, SupportCalls};

#[derive(Debug)]
pub struct PolymorphLoan {
    game: u64,
    original: (u32, u64),
    owner: (u32, u64),
}
impl Adapter {
    pub fn polymorph_entries(self) -> [Entry; 4] {
        [0x5cfeb6c, 0x5cfed67, 0x5cfeadd, 0x5cfeb08].map(|v| Entry {
            rva: self.rva(v),
            bytes: self.expected(v, &[]),
        })
    }
    fn polymorph_owned(
        self,
        mem: &dyn Mem,
        game: u64,
        slot: u32,
        guid: u64,
        enemy: u64,
    ) -> Result<bool> {
        let row = self.support_row(mem, game, slot)?;
        let read = |at, len| mem.read(row + self.row_field(row == game, at), len);
        if read(0x478, 8)? != guid.to_le_bytes() || guid == 0 {
            return Ok(false);
        }
        let hp = read(0x4f8, 4)?;
        let hp = f32::from_le_bytes(hp.try_into().map_err(|_| "Truncated Polymorph owner HP")?);
        if !hp.is_finite() || hp <= 0.0 {
            return Ok(false);
        }
        // Exact native swap pairs Game+ad8/ae0 with Actor+9d0/9d8.
        // These newly consumed fields are not covered by the legacy actor map.
        let pointer = u64::from_le_bytes(
            mem.read(row + if row == game { 0xad8 } else { 0x9d0 }, 8)?
                .try_into()
                .map_err(|_| "Truncated Polymorph pointer")?,
        );
        let count = u64::from_le_bytes(
            mem.read(row + if row == game { 0xae0 } else { 0x9d8 }, 8)?
                .try_into()
                .map_err(|_| "Truncated Polymorph count")?,
        );
        if count > 3
            || (count > 0 && (pointer < 0x10000 || pointer.checked_add(count * 8).is_none()))
        {
            return Err("Invalid native Polymorph ownership extent".into());
        }
        if count == 0 {
            return Ok(false);
        }
        let values = mem.read(pointer, count as usize * 8)?;
        if values.len() != count as usize * 8 {
            return Err("Truncated Polymorph owners".into());
        }
        Ok(values.chunks_exact(8).any(|v| v == enemy.to_le_bytes()))
    }
    pub(super) fn polymorph_select(self, io: &dyn SupportCalls, game: u64, target: u32) -> Result<()> {
        let current = u64::from_le_bytes(
            io.read(game + self.game_field(0xb40), 8)?
                .try_into()
                .map_err(|_| "Truncated Polymorph context")?,
        );
        if current >= NATIVE_PLAYERS as u64 || target >= NATIVE_PLAYERS as u32 {
            return Err("Invalid Polymorph context".into());
        }
        let current = current as u32;
        if current == target {
            return Ok(());
        }
        // The native context vector always stores the host in the active guest's
        // row. Normalize through host; a direct guest-to-guest swap is invalid.
        if current != 0 {
            let host = self.support_row(io, game, 0)?;
            io.swap_actor(game, host)?;
            io.write(game + self.game_field(0xb40), &0u64.to_le_bytes())?;
        }
        if target != 0 {
            let row = self.support_row(io, game, target)?;
            io.swap_actor(game, row)?;
            io.write(
                game + self.game_field(0xb40),
                &u64::from(target).to_le_bytes(),
            )?;
        }
        Ok(())
    }
    pub fn polymorph_candidate(self, mem:&dyn Mem, game:u64, enemy:u64, players:&[(u32,u64)]) -> Result<Option<(u32,u64)>> {
        let original = self.support_caster(mem, game)?;
        if !players.contains(&(original.slot, original.guid)) {
            return Ok(None);
        }
        let clock = f32::from_le_bytes(
            mem.read(game + self.game_field(0xb9c), 4)?
                .try_into()
                .map_err(|_| "Truncated Polymorph clock")?,
        );
        let (_, enemies) = self.combat_rows(mem, game)?;
        // Exact 0.138.0 expiry read by the native application gate. Never revive
        // an expired sheep merely because an actor's GUID list is not pruned yet.
        let active = enemies.chunks_exact(crate::current::ENEMY_STRIDE).any(|r| {
            u64::from_le_bytes(r[0xf0..0xf8].try_into().unwrap()) == enemy
                && f32::from_le_bytes(r[0x1d4..0x1d8].try_into().unwrap()) > clock
        });
        if !clock.is_finite() || !active {
            return Ok(None);
        }
        if self.polymorph_owned(mem, game, original.slot, original.guid, enemy)? {
            return Ok(None);
        }
        let mut owner = None;
        for &(slot, guid) in players {
            if slot == original.slot {
                continue;
            }
            if self.polymorph_owned(mem, game, slot, guid, enemy)? {
                if owner.is_some() {
                    return Ok(None);
                } // Ambiguous ownership fails closed.
                owner = Some((slot, guid));
            }
        }
        let Some(owner) = owner else {
            return Ok(None);
        };
        Ok(Some(owner))
    }
    pub fn polymorph_begin(
        self,
        io: &dyn SupportCalls,
        game: u64,
        enemy: u64,
        players: &[(u32, u64)],
    ) -> Result<Option<PolymorphLoan>> {
        // Both clients load the mode from the active actor before this branch,
        // but keep it in different registers until the ownership scan.
        if !matches!(io.read(game + 0x236, 1)?.as_slice(), [0 | 1]) {
            return Ok(None);
        }
        let original = self.support_caster(io, game)?;
        let Some(owner) = self.polymorph_candidate(io, game, enemy, players)? else {return Ok(None)};
        let loan = PolymorphLoan {
            game,
            original: (original.slot, original.guid),
            owner,
        };
        if let Err(error) = self.polymorph_select(io, game, owner.0) {
            let _ = self.polymorph_select(io, game, original.slot);
            return Err(error);
        }
        let selected = self.support_caster(io, game);
        if !selected.is_ok_and(|selected| (selected.slot, selected.guid) == owner) {
            self.polymorph_select(io, game, original.slot)?;
            return Err("Polymorph owner changed during context selection".into());
        }
        Ok(Some(loan))
    }
    pub fn polymorph_end(self, io: &dyn SupportCalls, loan: PolymorphLoan) -> Result<()> {
        let identity = self
            .support_caster(io, loan.game)
            .is_ok_and(|current| (current.slot, current.guid) == loan.owner);
        // Native burst meter/packets belong to the producer, then the original
        // actor resumes the remaining on-hit effects (including Faerie Fire).
        let flushed = if identity {
            io.flush_actor(loan.game)
        } else {
            Err("Polymorph scope identity changed".into())
        };
        self.polymorph_select(io, loan.game, loan.original.0)?;
        let restored = self.support_caster(io, loan.game)?;
        if (restored.slot, restored.guid) != loan.original {
            return Err("Polymorph original actor changed".into());
        }
        flushed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::official::{CurrencyMemory, MutableMemory};
    use std::cell::RefCell;
    const GAME: u64 = 0x10000;
    const ROWS: u64 = 0x20000;
    const ENEMIES: u64 = 0x40000;
    struct Fixture {
        data: RefCell<Vec<u8>>,
        flushed: RefCell<Vec<u64>>,
        fail_flush: bool,
    }
    impl Mem for Fixture {
        fn read(&self, at: u64, n: usize) -> Result<Vec<u8>> {
            self.data
                .borrow()
                .get(at as usize..at as usize + n)
                .map(|v| v.to_vec())
                .ok_or("fixture read overflow".into())
        }
    }
    impl MutableMemory for Fixture {
        fn write(&self, at: u64, v: &[u8]) -> Result<()> {
            self.data.borrow_mut()[at as usize..at as usize + v.len()].copy_from_slice(v);
            Ok(())
        }
        fn allocate_native(&self, _: &[u8]) -> Result<u64> {
            panic!("unexpected allocation")
        }
        fn free_native(&self, _: u64, _: usize) -> Result<()> {
            panic!("unexpected free")
        }
    }
    impl CurrencyMemory for Fixture {
        fn coinage(&self, _: u64, _: u32) -> Result<()> {
            panic!("unexpected gold")
        }
    }
    impl SupportCalls for Fixture {
        fn native_boss_health(&self, _: u64, _: u64) -> Result<f32> {
            unreachable!()
        }
        fn emit_support_effect(&self, _: u64, _: u32, _: &[u8; 12], _: f32) -> Result<()> {
            unreachable!()
        }
        fn refresh_support(&self, _: u64) -> Result<()> {
            unreachable!()
        }
        fn support_hit(&self, _: u64, _: u64, _: f32, _: u32) -> Result<()> {
            panic!("must not replay damage")
        }
        fn support_hurt_tagged(
            &self,
            _: u64,
            _: f32,
            _: u64,
            _: bool,
            _: u32,
            _: u32,
        ) -> Result<()> {
            unreachable!()
        }
        fn support_damage(&self, _: u64, _: u64, _: f32) -> Result<()> {
            unreachable!()
        }
        fn shield_proc(&self, _: u64, _: &[u8; 40], _: [f32; 3], _: u64, _:u32) -> Result<()> {
            unreachable!()
        }
        fn cast_actor(&self, _: u64, _: u64) -> Result<bool> {
            unreachable!()
        }
        fn healing_meter(&self, _: u64, _: u32, _: f32) -> Result<()> {
            unreachable!()
        }
        fn swap_actor(&self, game: u64, row: u64) -> Result<()> {
            // Independent reviewed subset of the native swap, including the
            // newly consumed ownership fields absent from the old actor map.
            for (g, a, n) in [
                (0x538, 0x538, 8),
                (0x5c8, 0x5c8, 4),
                (0x590, 0x568, 4),
                (0xad8, 0x9d0, 16),
            ] {
                let left = self.read(game + g, n)?;
                let right = self.read(row + a, n)?;
                self.write(game + g, &right)?;
                self.write(row + a, &left)?;
            }
            Ok(())
        }
        fn flush_actor(&self, game: u64) -> Result<()> {
            self.flushed.borrow_mut().push(u64::from_le_bytes(
                self.read(game + 0x538, 8)?.try_into().unwrap(),
            ));
            if self.fail_flush {
                Err("injected flush failure".into())
            } else {
                Ok(())
            }
        }
    }
    fn fixture() -> (Adapter, Fixture) {
        let a = Adapter::select(crate::current::SHA256).unwrap();
        let m = Fixture {
            data: RefCell::new(vec![0; 0x70000]),
            flushed: RefCell::new(vec![]),
            fail_flush: false,
        };
        for (at, v) in [
            (0xb90, ROWS),
            (0xb98, 2),
            (0x640, 1),
            (0x648, ENEMIES),
            (0x650, 1),
        ] {
            m.write(GAME + at, &v.to_le_bytes()).unwrap();
        }
        m.write(GAME + a.game_field(0xc41), &[1]).unwrap();
        m.write(GAME + 0xea0, &10f32.to_le_bytes()).unwrap();
        for slot in 0..3 {
            let row = if slot == 0 {
                GAME
            } else {
                ROWS + (slot - 1) * a.context_size()
            };
            m.write(row + 0x538, &(100 + slot).to_le_bytes()).unwrap();
            m.write(
                row + if row == GAME { 0x590 } else { 0x568 },
                &50f32.to_le_bytes(),
            )
            .unwrap();
            m.write(row + 0x5c8, &100f32.to_le_bytes()).unwrap();
        }
        let sheep = ROWS + a.context_size();
        m.write(sheep + 0x9d0, &0x50000u64.to_le_bytes()).unwrap();
        m.write(sheep + 0x9d8, &1u64.to_le_bytes()).unwrap();
        m.write(0x50000, &777u64.to_le_bytes()).unwrap();
        m.write(ENEMIES + 0xf0, &777u64.to_le_bytes()).unwrap();
        m.write(ENEMIES + 0x1d4, &20f32.to_le_bytes()).unwrap();
        (a, m)
    }
    const PLAYERS: &[(u32, u64)] = &[(0, 100), (1, 101), (2, 102)];
    #[test]
    fn ally_rupture_borrows_producer_and_restores_each_original_slot() {
        for slot in [0, 1] {
            let (a, m) = fixture();
            a.polymorph_select(&m, GAME, slot).unwrap();
            let loan = a.polymorph_begin(&m, GAME, 777, PLAYERS).unwrap().unwrap();
            assert_eq!(a.support_caster(&m, GAME).unwrap().guid, 102);
            // Native rupture consumes its owner's list before recursive damage.
            m.write(GAME + 0xae0, &0u64.to_le_bytes()).unwrap();
            assert!(a.polymorph_begin(&m, GAME, 777, PLAYERS).unwrap().is_none());
            a.polymorph_end(&m, loan).unwrap();
            assert_eq!(
                a.support_caster(&m, GAME).unwrap().guid,
                100 + u64::from(slot)
            );
            assert_eq!(*m.flushed.borrow(), [102]);
            let row = a.support_row(&m, GAME, 2).unwrap();
            assert_eq!(m.read(row + 0x9d8, 8).unwrap(), 0u64.to_le_bytes());
        }
    }
    #[test]
    fn stale_expired_or_own_sheep_never_borrow_another_actor() {
        let (a, m) = fixture();
        assert!(a
            .polymorph_begin(&m, GAME, 777, &[(0, 100), (2, 999)])
            .unwrap()
            .is_none());
        m.write(ENEMIES + 0x1d4, &10f32.to_le_bytes()).unwrap();
        assert!(a.polymorph_begin(&m, GAME, 777, PLAYERS).unwrap().is_none());
        a.polymorph_select(&m, GAME, 2).unwrap();
        assert!(a.polymorph_begin(&m, GAME, 777, PLAYERS).unwrap().is_none());
        assert!(m.flushed.borrow().is_empty());
    }
    #[test]
    fn ordinary_hits_do_not_bind_foreign_polymorph_contexts() {
        let (a,m)=fixture();
        m.write(ENEMIES+0x1d4,&0f32.to_le_bytes()).unwrap();
        m.write(GAME+a.game_field(0x980)+8,&0u64.to_le_bytes()).unwrap();
        assert!(a.polymorph_begin(&m,GAME,777,PLAYERS).unwrap().is_none());
        assert!(m.flushed.borrow().is_empty());
    }
    #[test]
    fn nested_foreign_explosions_restore_the_outer_producer_before_the_attacker() {
        let (a, m) = fixture();
        m.write(ROWS + 0x9d0, &0x50010u64.to_le_bytes()).unwrap();
        m.write(ROWS + 0x9d8, &1u64.to_le_bytes()).unwrap();
        m.write(0x50010, &888u64.to_le_bytes()).unwrap();
        m.write(GAME + 0x640, &2u64.to_le_bytes()).unwrap();
        m.write(GAME + 0x650, &2u64.to_le_bytes()).unwrap();
        m.write(ENEMIES + 0x208 + 0xf0, &888u64.to_le_bytes())
            .unwrap();
        m.write(ENEMIES + 0x208 + 0x1d4, &20f32.to_le_bytes())
            .unwrap();
        let outer = a.polymorph_begin(&m, GAME, 777, PLAYERS).unwrap().unwrap();
        let inner = a.polymorph_begin(&m, GAME, 888, PLAYERS).unwrap().unwrap();
        assert_eq!(a.support_caster(&m, GAME).unwrap().guid, 101);
        a.polymorph_end(&m, inner).unwrap();
        assert_eq!(a.support_caster(&m, GAME).unwrap().guid, 102);
        a.polymorph_end(&m, outer).unwrap();
        assert_eq!(a.support_caster(&m, GAME).unwrap().guid, 100);
        assert_eq!(*m.flushed.borrow(), [101, 102]);
    }
    #[test]
    fn flush_failure_still_restores_triggering_actor() {
        let (a, mut m) = fixture();
        m.fail_flush = true;
        a.polymorph_select(&m, GAME, 1).unwrap();
        let loan = a.polymorph_begin(&m, GAME, 777, PLAYERS).unwrap().unwrap();
        assert!(a.polymorph_end(&m, loan).is_err());
        assert_eq!(a.support_caster(&m, GAME).unwrap().guid, 101);
    }
}

#[cfg(feature = "test-hooks")]
impl Adapter {
    /// Disposable diagnostic: borrow one real native ability row, apply sheep
    /// through the native chance/cooldown gate, then hit it as the other actor.
    /// The production burst is neither invoked directly nor synthesized.
    pub unsafe fn fixture_polymorph_probe(&self, io: &dyn SupportCalls, base: u64, game: u64, caster: u32, prototype: &[u8]) -> Result<(u64, bool, bool)> {
        if prototype.len()<88 || caster>1 || self.support_caster(io,game)?.slot!=0 { return Err("Invalid Polymorph probe context".into()); }
        let entry=base+self.rva(0x5d4c0e0);
        let expected=self.expected(0x5d4c0e0,&[]);
        if io.read(entry,expected.len())?!=expected || io.read(base+self.rva(0x711edb0)+0x40,4)?!=118u32.to_le_bytes() { return Err("Polymorph probe native guard mismatch".into()); }
        let enemy=self.fixture_combat_enemy(io,game,5)?.ok_or("No Polymorph probe target")?;
        self.polymorph_select(io,game,caster)?;
        let mut ability=prototype[..88].to_vec();ability[..8].copy_from_slice(&(base+self.rva(0x711edb0)).to_le_bytes());
        let header=io.read(game+0x3f8,24)?;
        let storage=io.allocate_native(&ability)?;
        let mut loan=1u64.to_le_bytes().to_vec();loan.extend(storage.to_le_bytes());loan.extend(1u64.to_le_bytes());
        let result=(|| {
            io.write(game+0x3f8,&loan)?;
            let guid=self.support_caster(io,game)?.guid;
            let apply:unsafe extern "system" fn(u64,u64)=std::mem::transmute(entry as usize);
            let mut applied=false;
            for _ in 0..64 {
                apply(game,enemy);
                if self.polymorph_owned(io,game,caster,guid,enemy)? {applied=true;break;}
            }
            self.polymorph_select(io,game,0)?;
            if applied { self.attack_actor(io,game,1-caster,enemy,845,1.)?; }
            self.polymorph_select(io,game,caster)?;
            let consumed=applied && !self.polymorph_owned(io,game,caster,guid,enemy)?;
            Ok((enemy,applied,consumed))
        })();
        self.polymorph_select(io,game,caster)?;
        let unchanged=io.read(game+0x3f8,24)?==loan;
        if unchanged {io.write(game+0x3f8,&header)?;}
        self.polymorph_select(io,game,0)?;
        if !unchanged { return Err("Polymorph probe ability storage changed; retaining allocation".into()); }
        io.free_native(storage,88)?;
        result
    }
}

#[cfg(feature="test-hooks")]
impl Adapter {
    pub unsafe fn fixture_two_hunters(&self,io:&dyn SupportCalls,base:u64,game:u64,prototypes:&[Vec<u8>])->Result<Vec<(u64,f32)>> {
        if prototypes.len()!=2 || prototypes.iter().any(|p|p.len()<88) || self.support_caster(io,game)?.slot!=0 {return Err("Invalid two-hunter probe".into());}
        if io.read(base+self.rva(0x711d7a8)+0x40,4)?!=1130u32.to_le_bytes(){return Err("Hunter definition mismatch".into());}
        let tick:unsafe extern "system" fn(u64,f32)=std::mem::transmute((base+self.combat_entries()[0].rva)as usize);
        let mut loans=Vec::new();
        let result=(|| {
            for slot in 0..2u32 {
                self.polymorph_select(io,game,slot)?;
                let header=io.read(game+0x3f8,24)?;let target=io.read(game+0xae8,8)?;
                let mut row=prototypes[slot as usize][..88].to_vec();row[..8].copy_from_slice(&(base+self.rva(0x711d7a8)).to_le_bytes());
                row[16..20].copy_from_slice(&1f32.to_le_bytes());
                row[56..60].copy_from_slice(&(if slot==0{1f32}else{3.0625}).to_le_bytes());
                let storage=io.allocate_native(&row)?;
                loans.push((slot,header,target,storage));
                let mut loan=1u64.to_le_bytes().to_vec();loan.extend(storage.to_le_bytes());loan.extend(1u64.to_le_bytes());
                io.write(game+0x3f8,&loan)?;io.write(game+0xae8,&0u64.to_le_bytes())?;
                tick(game,0.);
            }
            let strong=self.combat_actor(io,game,base)?.mark;
            self.polymorph_select(io,game,0)?;
            let weak=self.combat_actor(io,game,base)?.mark;
            let enemy=weak.enemy.ok_or("Weak native mark missing")?;
            if strong.enemy!=Some(enemy){return Err("Native hunters selected different targets".into());}
            let read=||->Result<(u64,f32)>{Ok((enemy,self.combat_enemies(io,game)?.into_iter().find(|e|e.id==enemy).ok_or("Marked target missing")?.vulnerability))};
            let mut observations=vec![read()?];
            // Native weak caster writes last; production reconciliation must
            // still project the stronger living producer, rather than last writer.
            tick(game,0.);observations.push(read()?);
            for slot in [1,0] {
                self.polymorph_select(io,game,slot)?;
                let (_,header,target,_)=&loans[slot as usize];
                io.write(game+0x3f8,header)?;io.write(game+0xae8,target)?;
                tick(game,0.);observations.push(read()?);
            }
            Ok(observations)
        })();
        for (slot,header,target,storage) in loans {
            self.polymorph_select(io,game,slot)?;
            io.write(game+0x3f8,&header)?;io.write(game+0xae8,&target)?;
            io.free_native(storage,88)?;
        }
        self.polymorph_select(io,game,0)?;
        result
    }
}
