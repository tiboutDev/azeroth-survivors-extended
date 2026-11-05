//! Exact 0.138.0 talent conditions. Native proc budgets remain authoritative.
use super::*;
use super::effects::f32at;
impl Adapter {
    pub fn necklace_cast_event(self,mem:&dyn Mem,event:u64,spell:u32)->Result<bool>{
        Ok(event!=0 && mem.read(event+29,1)?==[3] && mem.read(event+8,4)?==spell.to_le_bytes())
    }
    pub fn necklace_leech_entry(self) -> Entry {
        Entry { rva: self.rva(0x5cf1c70), bytes: self.expected(0x5cf1c70, &[]) }
    }
    pub fn necklace_leech_window(self, mem: &dyn Mem, game: u64, spell: u32) -> Result<Option<(f32, f32)>> {
        let (key, duration): (&[u8], f32) = match spell {
            139 => (b"renew", 10.), 774 => (b"rejuvenation", 12.), _ => return Ok(None)
        };
        let Some(row) = self.active_proc(mem, game)? else { return Ok(None) };
        if row[..2] != [15, 14] || f32at(&row, 4) != 0.05 || f32at(&row, 12) != duration
            || row[16] != 7 || row[24] != 1 || !self.proc_key(mem, &row, 32, key)? {
            return Ok(None);
        }
        let now = self.set_clock(mem, game)?;
        Ok(now.is_finite().then_some((now, now + duration)))
    }
    pub fn talent_condition_entry(self) -> Entry {
        Entry { rva:self.rva(0x5d3ef10), bytes:self.expected(0x5d3ef10,&[]) }
    }
    pub fn talent_area_entry(self)->Entry{Entry{rva:self.rva(0x5d39516),bytes:self.expected(0x5d39516,&[])}}
    fn active_proc(self, mem:&dyn Mem, game:u64) -> Result<Option<Vec<u8>>> {
        let q=|at|->Result<u64>{Ok(u64::from_le_bytes(mem.read(game+self.game_field(at),8)?.try_into().map_err(|_|"Truncated talent context")?))};
        if q(0x20)?!=1{return Ok(None)}
        let index=q(0x28)?;let pointer=q(0x838)?;let count=q(0x840)?;
        if count>512 || index>=count || pointer<0x10000 || pointer.checked_add(count*64).is_none(){return Ok(None)}
        let row=mem.read(pointer+index*64,64)?;
        Ok(super::super::perks::observe(crate::current::PERK_OBSERVATION_LAYOUT, &row)?.map(Vec::from))
    }
    // Native 5ce49b0 resolves the active perk index into a 40-byte origin.
    // Its icon distinguishes the necklace from the byte-identical Hammer perk.
    fn active_necklace_origin(self, mem: &dyn Mem, game: u64) -> Result<bool> {
        let read = |at| -> Result<u64> {
            Ok(u64::from_le_bytes(mem.read(at, 8)?.try_into().map_err(|_| "Truncated perk origin")?))
        };
        let index = read(game + self.game_field(0x28))?;
        let count = read(game + crate::current::PERK_ORIGINS_HEADER + 16)?;
        let origins = read(game + crate::current::PERK_ORIGINS_HEADER + 8)?;
        if count > 512 || index >= count || origins < 65536 || origins.checked_add(count * 40).is_none() {
            return Ok(false);
        }
        let row = mem.read(origins + index * 40, 40)?;
        self.proc_key(mem, &row, 24, b"INV_Jewelry_Necklace_21")
    }
    fn proc_key(self, mem:&dyn Mem, row:&[u8], at:usize, key:&[u8])->Result<bool>{
        let q=|at|u64::from_le_bytes(row[at..at+8].try_into().unwrap());
        if q(at+8)!=key.len()as u64{return Ok(false)}let pointer=q(at);
        if pointer<0x10000 || pointer.checked_add(key.len()as u64).is_none(){return Err("Invalid talent key".into())}
        Ok(mem.read(pointer,key.len())?==key)
    }
    /// Native perk 18 writes a shared absorb pool once on a matching cast.
    pub fn route_necklace_absorb(self, mem: &dyn MutableMemory, game: u64, spell: u32,
        caster: (u32, u64), recipient: (u32, u64)) -> Result<bool> {
        let key: &[u8] = match spell { 635 => b"holy_light", 8936 => b"regrowth", _ => return Ok(false) };
        let Some(row) = self.active_proc(mem, game)? else { return Ok(false) };
        if row[0] != 18 || f32at(&row, 4) != 0.4 || f32at(&row, 8) != 0.4
            || row[16] != 7 || row[24] != 1 || !self.proc_key(mem, &row, 32, key)? {
            return Ok(false);
        }
        let source = self.support_caster(mem, game)?;
        if (source.slot, source.guid) != caster || caster.0 == recipient.0 { return Ok(false) }
        let target = self.support_row(mem, game, recipient.0)?;
        let field = |off| target + self.row_field(target == game, off);
        if mem.read(field(0x478), 8)? != recipient.1.to_le_bytes() || recipient.1 == 0
            || !(f32at(&mem.read(field(0x4f8), 4)?, 0) > 0.) { return Ok(false) }
        let address = field(0x4fc);
        let old = f32at(&mem.read(address, 4)?, 0);
        let cap = source.maximum * f32at(&row, 8);
        if !old.is_finite() || !(0.0..=1e12).contains(&cap) { return Err("Invalid necklace absorb pool".into()) }
        if old < cap { mem.write(address, &(old + source.maximum * f32at(&row, 4)).min(cap).to_le_bytes())?; }
        Ok(true)
    }
    pub fn support_area_factor(self, mem:&dyn Mem, game:u64)->Result<f32>{
        let value=f32at(&mem.read(game+self.field1201(crate::current::RING_STAT_OUTPUTS[7]),4)?,0);
        if !value.is_finite() || !(0.0..=1000.0).contains(&value){return Err("Invalid support area factor".into())}Ok(value)
    }
    pub fn talent_query_radius(self,mem:&dyn Mem,game:u64,caller:u64,radius:f32)->Result<f32>{
        let eligible=if caller==self.rva(0x5d39516){true}else if caller==self.rva(0x5944de9){
            self.active_proc(mem,game)?.is_some_and(|p|shield_burst(&p).is_some())
        }else{false};
        if !eligible{return Ok(radius)}
        if !radius.is_finite()||radius<0.{return Err("Invalid talent radius".into())}
        Ok(radius*self.support_area_factor(mem,game)?)
    }
    pub fn talent_condition_override(self,mem:&dyn Mem,game:u64,condition:u64,event:u64,shield:Option<(u32,u64)>,players:&[(u32,u64)])->Result<Option<bool>>{
        let Some(row)=self.active_proc(mem,game)?else{return Ok(None)};
        let c=mem.read(condition,8)?;if c.len()!=8{return Err("Truncated talent condition".into())}let kind=u32::from_le_bytes(c[..4].try_into().map_err(|_|"Truncated talent condition")?);
        let e=mem.read(event+0x1d,1)?;let event_kind=*e.first().ok_or("Truncated talent event")?;
        if kind==10 && event_kind==6 {
            let Some((spell,recipient))=shield else{return Ok(None)};
            let source=self.support_caster(mem,game)?;
            if let Some(burst) = shield_burst(&row) {
                return Ok(Some(match burst {
                    ShieldBurst::Hammer if self.active_necklace_origin(mem, game)? => spell == 17,
                    ShieldBurst::Hammer => source.guid == recipient,
                    ShieldBurst::Aegis | ShieldBurst::Devout => spell == 17,
                }));
            }
            if row[0]==0 && self.proc_key(mem,&row,16,b"pw_shield")?{return Ok(Some(spell==17))}
        }
        if kind!=9 || event_kind!=5 || row[0]!=0 || f32at(&c,4).to_bits()!=0.4f32.to_bits() || !self.proc_key(mem,&row,16,b"regrowth")? {return Ok(None)}
        let source=self.support_caster(mem,game)?;
        if source.hp<=0. || !players.contains(&(source.slot,source.guid)){return Ok(None)}
        for &(slot,guid) in players{
            let target=self.support_row(mem,game,slot)?;
            let at=|n|target+self.row_field(target==game,n);
            let identity=u64::from_le_bytes(mem.read(at(0x478),8)?.try_into().map_err(|_|"Truncated rescue identity")?);
            if identity!=guid || guid==0{continue}
            let hp=f32at(&mem.read(at(0x4f8),4)?,0);let max=f32at(&mem.read(at(0x490),4)?,0);
            let pos=mem.read(at(0x4e0),12)?;let distance=(0..3).map(|n|(f32at(&pos,n*4)-source.position[n]).powi(2)).sum::<f32>();
            if hp.is_finite()&&max.is_finite()&&hp>0.&&hp<max*0.4&&distance<=900.{return Ok(Some(true))}
        }
        Ok(None)
    }
}
#[derive(Clone, Copy)]
enum ShieldBurst { Aegis, Hammer, Devout }

fn shield_burst(row: &[u8]) -> Option<ShieldBurst> {
    if row.len() != 64 || row[..2] != [3, 1]
        || u32::from_le_bytes(row[12..16].try_into().ok()?) != 15237
        || u32::from_le_bytes(row[16..20].try_into().ok()?) != 10 {
        return None;
    }
    match (f32at(row, 8), f32at(row, 4)) {
        (10.0, 0.1) => Some(ShieldBurst::Aegis),
        (10.0, 0.5) => Some(ShieldBurst::Hammer),
        (8.0, 2.0 | 3.0) => Some(ShieldBurst::Devout),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const GAME:u64=0x10000;const ROW:u64=0x20000;const PERK:u64=0x30000;const KEY:u64=0x31000;const COND:u64=0x32000;const EVENT:u64=0x33000;
    struct Memory(Vec<u8>);
    impl Mem for Memory{fn read(&self,a:u64,n:usize)->Result<Vec<u8>>{self.0.get(a as usize..a as usize+n).map(|b|b.to_vec()).ok_or("test read out of bounds".into())}}
    impl Memory{fn put(&mut self,a:u64,b:&[u8]){self.0[a as usize..a as usize+b.len()].copy_from_slice(b)} fn float(&mut self,a:u64,v:f32){self.put(a,&v.to_le_bytes())}fn word(&mut self,a:u64,v:u64){self.put(a,&v.to_le_bytes())}}
    fn fixture()->(Adapter,Memory){
        let a=Adapter::select(crate::current::SHA256).unwrap();let mut m=Memory(vec![0;0x50000]);
        m.put(GAME+a.game_field(0xc41),&[1]);m.word(GAME+0x20,1);m.word(GAME+0xb70,PERK);m.word(GAME+0xb78,1);
        m.word(GAME+a.game_field(0x980),ROW);m.word(GAME+a.game_field(0x980)+8,1);
        for(row,guid,hp)in[(GAME,100,100.),(ROW,101,39.)]{
            m.word(row+a.row_field(row==GAME,0x478),guid);m.float(row+a.row_field(row==GAME,0x490),100.);m.float(row+a.row_field(row==GAME,0x4f8),hp);
        }
        m.float(GAME+a.field1201(crate::current::RING_STAT_OUTPUTS[7]),1.5);
        (a,m)
    }
    fn burst(m:&mut Memory,hammer:bool){m.put(PERK,&15u32.to_le_bytes());m.put(PERK+52,&1u32.to_le_bytes());m.float(PERK+40,if hammer{0.5}else{0.1});m.float(PERK+44,10.);m.put(PERK+48,&15237u32.to_le_bytes());m.put(PERK+8,&10u32.to_le_bytes());m.put(COND,&10u32.to_le_bytes());m.put(EVENT+0x1d,&[6]);}
    #[test]fn shield_producer_rules_distinguish_aegis_hammer_and_lightning(){
        let(a,mut m)=fixture();burst(&mut m,false);
        for(recipient,spell,expected)in[(100,17,true),(101,17,true),(100,324,false),(101,11426,false)]{
            assert_eq!(a.talent_condition_override(&m,GAME,COND,EVENT,Some((spell,recipient)),&[]).unwrap(),Some(expected));
        }
        burst(&mut m,true);
        for(recipient,expected)in[(100,true),(101,false)]{assert_eq!(a.talent_condition_override(&m,GAME,COND,EVENT,Some((17,recipient)),&[]).unwrap(),Some(expected));}
        m.word(GAME+crate::current::PERK_ORIGINS_HEADER+8,0x38000);m.word(GAME+crate::current::PERK_ORIGINS_HEADER+16,1);
        let icon=b"INV_Jewelry_Necklace_21";m.word(0x38018,KEY);m.word(0x38020,icon.len() as u64);m.put(KEY,icon);
        assert_eq!(a.talent_condition_override(&m,GAME,COND,EVENT,Some((17,101)),&[]).unwrap(),Some(true));
        assert_eq!(a.talent_condition_override(&m,GAME,COND,EVENT,Some((11426,100)),&[]).unwrap(),Some(false));
        m.float(PERK+40,0.7);assert_eq!(a.talent_condition_override(&m,GAME,COND,EVENT,Some((17,101)),&[]).unwrap(),None,"unrelated burst must retain native behavior");
    }
    #[test]
    fn both_devout_tiers_use_the_shield_producers_area_once() {
        let (a, mut m) = fixture();
        burst(&mut m, false);
        m.float(PERK+44, 8.0);
        for coefficient in [2.0, 3.0] {
            m.float(PERK+40, coefficient);
            assert_eq!(a.talent_query_radius(&m,GAME,a.rva(0x5944de9),8.0).unwrap(),12.0);
            for (spell, allowed) in [(17,true),(324,false),(11426,false)] {
                assert_eq!(a.talent_condition_override(&m,GAME,COND,EVENT,Some((spell,101)),&[]).unwrap(),Some(allowed));
            }
        }
    }
    #[test]fn swiftmend_requires_eligible_identity_range_health_and_native_event(){
        let(a,mut m)=fixture();m.put(PERK,&12u32.to_le_bytes());m.word(PERK+40,KEY);m.word(PERK+48,8);m.put(KEY,b"regrowth");m.put(COND,&9u32.to_le_bytes());m.float(COND+4,0.4);m.put(EVENT+0x1d,&[5]);
        let run=|m:&Memory,players:&[(u32,u64)]|a.talent_condition_override(m,GAME,COND,EVENT,None,players).unwrap();
        assert_eq!(run(&m,&[(0,100),(1,101)]),Some(true));
        assert_eq!(run(&m,&[(1,101)]),None,"missing source identity");
        assert_eq!(run(&m,&[(0,100),(1,999)]),None);
        m.float(ROW+a.row_field(false,0x4f8),40.);assert_eq!(run(&m,&[(0,100),(1,101)]),None);
        m.float(ROW+a.row_field(false,0x4f8),0.);assert_eq!(run(&m,&[(0,100),(1,101)]),None);
        m.float(ROW+a.row_field(false,0x4f8),39.);m.float(ROW+a.row_field(false,0x4e0),31.);assert_eq!(run(&m,&[(0,100),(1,101)]),None);
        m.float(ROW+a.row_field(false,0x4e0),0.);m.put(EVENT+0x1d,&[6]);assert_eq!(run(&m,&[(0,100),(1,101)]),None);
    }
    #[test]fn area_scaling_uses_current_producer_and_only_reviewed_queries(){
        let(a,mut m)=fixture();burst(&mut m,false);
        assert_eq!(a.talent_query_radius(&m,GAME,a.rva(0x5d39516),12.).unwrap(),18.);
        assert_eq!(a.talent_query_radius(&m,GAME,a.rva(0x5944de9),10.).unwrap(),15.);
        assert_eq!(a.talent_query_radius(&m,GAME,0,12.).unwrap(),12.);
        m.word(GAME+crate::current::PERK_ORIGINS_HEADER+8,0x38000);m.word(GAME+crate::current::PERK_ORIGINS_HEADER+16,1);
        let icon=b"INV_Jewelry_Necklace_21";m.word(0x38018,KEY);m.word(0x38020,icon.len() as u64);m.put(KEY,icon);
        assert_eq!(a.talent_condition_override(&m,GAME,COND,EVENT,Some((17,101)),&[]).unwrap(),Some(true));
        assert_eq!(a.talent_condition_override(&m,GAME,COND,EVENT,Some((11426,100)),&[]).unwrap(),Some(false));
        m.float(PERK+40,0.7);assert_eq!(a.talent_query_radius(&m,GAME,a.rva(0x5944de9),10.).unwrap(),10.);
    }
}
