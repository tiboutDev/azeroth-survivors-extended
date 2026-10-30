//! Borrow the native world-text message writer only during its native log callback.
use super::*;
use crate::official::MutableMemory;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealText {
    pub caster: u64,
    pub target: u64,
    pub amount: u32,
    pub critical: bool,
}
impl Adapter {
    pub fn combat_text_entries(self) -> [super::super::official::Entry; 8] {
        use crate::current as v;
        [
            v::DIRECT_HEAL_TEXT,
            v::PERIODIC_HEAL_TEXT,
            v::TEXT_ENTITY,
            v::TEXT_RESERVE,
            v::FRACTIONAL_HEAL_POINT,
            v::HEAL_LOG_PACKET,
            v::HEAL_LOG_RESERVE,
            v::PERIODIC_TEXT_POINT,
        ]
        .map(|(rva, bytes)| super::super::official::Entry { rva, bytes })
    }
    pub fn native_heal_texts(
        self,
        m: &dyn Mem,
        bp: u64,
        periodic: bool,
        players: &[u64],
    ) -> Result<Vec<HealText>> {
        let u64_at = |at| -> Result<u64> {
            Ok(u64::from_le_bytes(
                m.read(at, 8)?
                    .try_into()
                    .map_err(|_| "Truncated text identity")?,
            ))
        };
        let payload = bp + crate::current::HEAL_TEXT_PAYLOAD[usize::from(periodic)];
        let (target, caster) = if periodic {
            (u64_at(payload + 0x18)?, u64_at(payload + 0x20)?)
        } else {
            (u64_at(payload)?, u64_at(payload + 8)?)
        };
        if target == 0 || !players.contains(&target) || !players.contains(&caster) {
            return Ok(vec![]);
        }
        let mut result = vec![];
        if periodic {
            let len = u64_at(payload + 0x10)?;
            let cap = u64_at(payload)?;
            let rows = u64_at(payload + 8)?;
            if len > 32 || cap < len || cap > 4096 {
                return Err("Invalid periodic text rows".into());
            }
            for row in m.read(rows, (len * 20) as usize)?.chunks_exact(20) {
                // Native periodic variant 1 is healing; variant 0 already emits damage text.
                if u32::from_le_bytes(row[..4].try_into().unwrap()) == 1 {
                    let amount = u32::from_le_bytes(row[4..8].try_into().unwrap());
                    if amount > 0 {
                        result.push(HealText {
                            caster,
                            target,
                            amount,
                            critical: false,
                        });
                    }
                }
            }
        } else {
            let raw = m.read(payload + 0x14, 5)?;
            let amount = u32::from_le_bytes(raw[..4].try_into().unwrap());
            if amount > 0 {
                result.push(HealText {
                    caster,
                    target,
                    amount,
                    critical: raw[4] == 1,
                });
            }
        }
        Ok(result)
    }
    /// Append exactly one message per actual native healing log. No simulation,
    /// healing, meters or network calls. Native renderer owns lifetime and animation.
    pub unsafe fn draw_native_heal_text(
        self,
        m: &dyn MutableMemory,
        base: usize,
        bp: u64,
        periodic: bool,
        heal: HealText,
    ) -> Result<bool> {
        let word = |at| -> Result<u64> {
            Ok(u64::from_le_bytes(
                m.read(at, 8)?
                    .try_into()
                    .map_err(|_| "Truncated native text writer")?,
            ))
        };
        let objects = word(bp + crate::current::HEAL_TEXT_OBJECTS[usize::from(periodic)])?;
        let query = m.read(
            bp + crate::current::HEAL_TEXT_QUERY[usize::from(periodic)],
            24,
        )?;
        type Entity = unsafe extern "system" fn(u64, u64, u64, u64, *const u8) -> u64;
        let resolve: Entity = std::mem::transmute(base + crate::current::TEXT_ENTITY.0 as usize);
        let entity = resolve(heal.target, objects, 0, 0, query.as_ptr());
        if entity as u32 == 0 {
            return Ok(false);
        }
        let params = bp + crate::current::HEAL_TEXT_PARAMS[usize::from(periodic)];
        let writer = word(params + 0x70)?;
        let tick_ptr = word(params + 0x80)?;
        let tick = m.read(params + 0x8c, 4)?;
        let len = word(writer + 0x30)?;
        let cap = word(writer + 0x20)?;
        if len >= 4096 || cap < len || cap > 1 << 20 {
            return Err("Native text capacity rejected".into());
        }
        let sequence = word(writer + 0x40)?;
        let next_sequence = sequence
            .checked_add(1)
            .ok_or("Native text sequence overflow")?;
        // Validate the changed-tick destination before allocating any owned value.
        m.read(tick_ptr, 4)?;
        type Reserve = unsafe extern "system" fn(u64);
        if len == cap {
            let reserve: Reserve =
                std::mem::transmute(base + crate::current::TEXT_RESERVE.0 as usize);
            reserve(writer + 0x20);
        }
        let buffer = word(writer + 0x28)?;
        if word(writer + 0x20)? <= len {
            return Err("Native text reserve did not grow".into());
        }
        let cursor = buffer
            .checked_add(len.checked_mul(0x38).ok_or("Text offset overflow")?)
            .ok_or("Text pointer overflow")?;
        m.read(cursor, 0x38)?;
        let text = format!("+{}", heal.amount);
        let size = text.len();
        let owned = m.allocate_native(text.as_bytes())?;
        // The native message destructor owns this String after the single commit.
        let mut row = [0u8; 0x38];
        row[..8].copy_from_slice(&(size as u64).to_le_bytes());
        row[8..16].copy_from_slice(&owned.to_le_bytes());
        row[16..24].copy_from_slice(&(size as u64).to_le_bytes());
        row[24..28].copy_from_slice(&1u32.to_le_bytes());
        row[28..32].copy_from_slice(&crate::current::HEAL_TEXT_COLOR.to_le_bytes());
        row[32..40].copy_from_slice(&entity.to_le_bytes());
        row[40] = if heal.critical { 2 } else { 0 };
        row[48..].copy_from_slice(&sequence.to_le_bytes());
        // Destinations were validated above; no native callback holds this borrowed row.
        if let Err(error) = m.write(cursor, &row) {
            m.free_native(owned, size)?;
            return Err(error);
        }
        m.write(writer + 0x30, &(len + 1).to_le_bytes())?;
        m.write(writer + 0x40, &next_sequence.to_le_bytes())?;
        m.write(tick_ptr, &tick)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Bytes {
        data: Vec<u8>,
    }
    impl Mem for Bytes {
        fn read(&self, at: u64, n: usize) -> Result<Vec<u8>> {
            self.data
                .get(at as usize..at as usize + n)
                .map(|b| b.to_vec())
                .ok_or("Outside fixture".into())
        }
    }
    fn put(m: &mut Bytes, at: usize, n: u64) {
        m.data[at..at + 8].copy_from_slice(&n.to_le_bytes());
    }
    fn adapter() -> Adapter {
        Adapter::select(crate::current::SHA256).unwrap()
    }
    #[test]
    fn direct_native_amount_critical_and_both_identities_are_required() {
        let mut m = Bytes {
            data: vec![0; 0x1000],
        };
        put(&mut m, 0xd0, 11);
        put(&mut m, 0xd8, 22);
        m.data[0xe4..0xe8].copy_from_slice(&37u32.to_le_bytes());
        m.data[0xe8] = 1;
        assert_eq!(
            adapter()
                .native_heal_texts(&m, 0, false, &[11, 22])
                .unwrap(),
            vec![HealText {
                target: 11,
                caster: 22,
                amount: 37,
                critical: true
            }]
        );
        assert!(adapter()
            .native_heal_texts(&m, 0, false, &[11])
            .unwrap()
            .is_empty());
        assert!(adapter()
            .native_heal_texts(&m, 0, false, &[22])
            .unwrap()
            .is_empty());
    }
    #[test]
    fn periodic_heals_preserve_packet_order_and_exclude_native_damage() {
        let mut m = Bytes {
            data: vec![0; 0x1000],
        };
        put(&mut m, 0x4a0, 3);
        put(&mut m, 0x4a8, 0x700);
        put(&mut m, 0x4b0, 3);
        put(&mut m, 0x4b8, 11);
        put(&mut m, 0x4c0, 22);
        for (i, (kind, amount)) in [(0u32, 77u32), (1, 13), (1, 17)].into_iter().enumerate() {
            m.data[0x700 + i * 20..0x708 + i * 20]
                .copy_from_slice(&[kind.to_le_bytes(), amount.to_le_bytes()].concat());
        }
        let rows = adapter().native_heal_texts(&m, 0, true, &[11, 22]).unwrap();
        assert_eq!(
            rows.iter().map(|r| r.amount).collect::<Vec<_>>(),
            vec![13, 17]
        );
        assert!(rows
            .iter()
            .all(|r| r.target == 11 && r.caster == 22 && !r.critical));
        put(&mut m, 0x4b0, 33);
        assert!(adapter().native_heal_texts(&m, 0, true, &[11, 22]).is_err());
    }

}

#[cfg(feature="test-hooks")]
impl Adapter {
 pub fn fixture_heal_packet(self,m:&dyn Mem,bp:u64,periodic:bool)->Result<Vec<u8>>{
  m.read(bp+crate::current::HEAL_TEXT_PAYLOAD[usize::from(periodic)],if periodic{48}else{32})
 }
}

impl Adapter {
 pub fn fractional_heal_clock(self,m:&dyn Mem,game:u64)->Result<f32>{
  let clock=f32::from_le_bytes(m.read(game+self.game_field(0xb9c),4)?.try_into().map_err(|_|"Truncated heal clock")?);
  if !clock.is_finite()||clock<0.{return Err("Invalid heal clock".into())}
  Ok(clock)
 }
 pub fn fractional_heal_context(self,m:&dyn Mem,game:u64)->Result<(f32,super::support::SupportCaster)>{
  Ok((self.fractional_heal_clock(m,game)?,self.support_caster(m,game)?))
 }
 /// Native cosmetic SPELLHEALLOG constructor and outbound UI queue only. No HP,
 /// proc, meter or spell invocation. Pending credit is consumed before this call.
 pub unsafe fn emit_fractional_heal_text(self,m:&dyn MutableMemory,base:usize,game:u64,heal:HealText,spell:u32)->Result<()> {
  let queue=game+crate::current::HEAL_LOG_QUEUE;
  let q=|at|->Result<u64>{Ok(u64::from_le_bytes(m.read(at,8)?.try_into().map_err(|_|"Truncated UI queue")?))};
  let(cap,len)=(q(queue)?,q(queue+16)?);
  if len>cap||cap>65536||len>=1024||heal.amount==0||heal.caster==0||heal.target==0{return Err("Invalid cosmetic heal queue".into())}
  if len==cap{let reserve:unsafe extern "system" fn(u64)=std::mem::transmute(base+crate::current::HEAL_LOG_RESERVE.0 as usize);reserve(queue);}
  let(cap,ptr,n)=(q(queue)?,q(queue+8)?,q(queue+16)?);
  if n!=len||cap<=len||cap>65536||ptr<0x10000{return Err("UI queue reserve changed identity".into())}
  let cursor=ptr.checked_add(len*32).ok_or("UI queue overflow")?;m.read(cursor,32)?;
  let mut packet=[0u8;32];
  let make:unsafe extern "system" fn(*mut u8,u64,u64,u32,u32,bool)->u64=std::mem::transmute(base+crate::current::HEAL_LOG_PACKET.0 as usize);
  make(packet.as_mut_ptr(),heal.target,heal.caster,spell,heal.amount,heal.critical);
  struct PacketOwner<'a>{memory:&'a dyn MutableMemory,bytes:u64,capacity:usize,transferred:bool}
  impl Drop for PacketOwner<'_>{fn drop(&mut self){if !self.transferred&&self.bytes>=0x10000&&self.capacity>0&&self.capacity<=4096{let _=self.memory.free_native(self.bytes,self.capacity);}}}
  let mut owned=PacketOwner{memory:m,capacity:u64::from_le_bytes(packet[..8].try_into().unwrap())as usize,bytes:u64::from_le_bytes(packet[8..16].try_into().unwrap()),transferred:false};
  // Rebind after the native constructor, before transferring its owned buffer.
  let current=(q(queue)?,q(queue+8)?,q(queue+16)?);
  if current.2!=len||current.0<=len||current.1<0x10000{return Err("Native UI queue changed during packet construction".into())}
  let cursor=current.1.checked_add(len*32).ok_or("UI queue overflow")?;
  m.write(cursor,&packet)?;
  owned.transferred=true;
  // An uncertain final commit is never retried or freed underneath a consumer.
  m.write(queue+16,&(len+1).to_le_bytes())
 }
}
