//! Borrow the native world-text message writer only during its native log callback.
use super::*;
use crate::official::MutableMemory;
use std::sync::atomic::AtomicUsize;
pub static DAMAGE_NEXT: AtomicUsize = AtomicUsize::new(0);
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
static THIRD: f32 = 1.0 / 3.0;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
static HALF: f32 = 0.5;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const DAMAGE_ENTRY: unsafe extern "C" fn() =
    damage_entry::<{ crate::current::DAMAGE_TEXT_ROW_REGISTER }>;

/// Only the selected scalar size may change. Preserve the row register and flags.
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
#[unsafe(naked)]
unsafe extern "C" fn damage_entry<const ROW: usize>() {
    core::arch::naked_asm!(
 "pushfq",
 ".if {row} == 3",
 "cmp dword ptr [rbx + 0x34], {green}", "je 3f",
 "cmp byte ptr [rbx + 0x38], 0", "je 2f",
 "cmp byte ptr [rbx + 0x38], 2", "jne 3f",
 ".elseif {row} == 14",
 "cmp dword ptr [r14 + 0x34], {green}", "je 3f",
 "cmp byte ptr [r14 + 0x38], 0", "je 2f",
 "cmp byte ptr [r14 + 0x38], 2", "jne 3f",
 ".else", ".error \"Unsupported combat-text row register\"", ".endif",
 ".if {fresh} == 8", "mulss xmm8, dword ptr [rip + {half}]",
 ".elseif {fresh} == 7", "mulss xmm7, dword ptr [rip + {half}]", ".endif",
 "jmp 3f",
 "2:",
 ".if {fresh} == 8", "mulss xmm8, dword ptr [rip + {third}]",
 ".elseif {fresh} == 7", "mulss xmm7, dword ptr [rip + {third}]",
 ".else", ".error \"Unsupported combat-text size register\"", ".endif",
 "3:", "popfq", "jmp qword ptr [rip + {next}]",
 row=const ROW,green=const crate::current::HEAL_TEXT_COLOR,fresh=const crate::current::DAMAGE_TEXT_XMM,third=sym THIRD,half=sym HALF,next=sym DAMAGE_NEXT);
}
type NativeEntityQuery = unsafe extern "system" fn(u64, u64, *const u8) -> u8;
type NativeEntityLookup = unsafe extern "system" fn(*const u64, u64) -> u64;
/// 0.133 separates the query predicate from the GUID/entity map lookup. The
/// lookup borrows a pointer to the map reference, not the map pointer directly.
unsafe fn resolve_with_native_parts(predicate: NativeEntityQuery, lookup: NativeEntityLookup, target: u64, objects: u64, query: *const u8) -> Result<u64> {
    match predicate(target, objects, query) {
        0 => Ok(0),
        1 => Ok(lookup(&objects, target)),
        _ => Err("Invalid native text query result".into()),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealText {
    pub caster: u64,
    pub target: u64,
    pub amount: u32,
    pub critical: bool,
}
impl Adapter {
    /// Exact selected renderer seam, after size interpolation/clamping.
    pub fn damage_text_size_entry(self) -> super::super::official::Entry {
        let (rva, bytes) = crate::current::DAMAGE_TEXT_SIZE;
        super::super::official::Entry { rva, bytes }
    }

    pub fn combat_text_entries(self) -> [super::super::official::Entry; 9] {
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
            v::TEXT_QUERY,
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
        let tag = u64_at(bp + crate::current::HEAL_TEXT_TAG[usize::from(periodic)])?;
        let expected = crate::current::HEAL_TEXT_DISCRIMINANTS[usize::from(periodic)];
        if tag != expected { return Ok(vec![]); }
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
        let entity = match crate::current::HEAL_TEXT_RESOLVER_ABI {
            0 => {
                type Entity = unsafe extern "system" fn(u64, u64, u64, u64, *const u8) -> u64;
                let resolve: Entity = std::mem::transmute(base + crate::current::TEXT_ENTITY.0 as usize);
                resolve(heal.target, objects, 0, 0, query.as_ptr())
            }
            1 => {
                let predicate: NativeEntityQuery = std::mem::transmute(base + crate::current::TEXT_QUERY.0 as usize);
                let lookup: NativeEntityLookup = std::mem::transmute(base + crate::current::TEXT_ENTITY.0 as usize);
                resolve_with_native_parts(predicate, lookup, heal.target, objects, query.as_ptr())?
            }
            _ => return Err("Unsupported native text resolver ABI".into()),
        };
        let params = bp + crate::current::HEAL_TEXT_PARAMS[usize::from(periodic)];
        self.append_native_heal_text(m, base, params, heal, entity)
    }

    /// Publish through the native writer after resolving the native entity.
    /// Separating resolution from ownership transfer keeps both ABIs independently testable.
    pub unsafe fn append_native_heal_text(
        self, m: &dyn MutableMemory, base: usize, params: u64, heal: HealText, entity: u64,
    ) -> Result<bool> {
        if entity as u32 == 0 { return Ok(false); }
        let word = |at| -> Result<u64> {
            Ok(u64::from_le_bytes(m.read(at, 8)?.try_into().map_err(|_| "Truncated native text writer")?))
        };
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
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    #[test]
    fn damage_size_uses_fresh_baseline_preserves_other_categories_and_heals() {
        #[unsafe(naked)]
        unsafe extern "C" fn end() {
            core::arch::naked_asm!(
                "pushfq", "pop rax", "mov [rcx + 0x20], rax",
                "movups [rcx + 0x50], xmm7", "movups [rcx + 0x60], xmm8", "ret"
            );
        }
        #[unsafe(naked)]
        unsafe extern "system" fn invoke(_row: *mut u8, _size: f32, _stub: unsafe extern "C" fn(), _row_register: usize) {
            core::arch::naked_asm!(
                "push rbx", "push r14", "sub rsp, 72", "movaps [rsp + 32], xmm7", "movaps [rsp + 48], xmm8",
                "lea rbx, [rcx + 128]", "mov r14, rbx", "cmp r9, 14", "cmove r14, rcx", "cmovne rbx, rcx",
                "movups xmm7, [rcx + 0x50]", "movups xmm8, [rcx + 0x60]",
                "movss xmm7, xmm1", "movss xmm8, xmm1", "cmp rsp, rsp", "call r8",
                "movaps xmm7, [rsp + 32]", "movaps xmm8, [rsp + 48]", "add rsp, 72", "pop r14", "pop rbx", "ret"
            );
        }
        let expected_register = match crate::current::VERSION {
            "0.131.0" => 7,
            "0.133.0" | "0.135.1" | "0.138.0" | "0.139.2" => 8,
            _ => panic!("Fresh-size fixture requires native register review"),
        };
        assert_eq!(crate::current::DAMAGE_TEXT_XMM, expected_register);
        let previous = DAMAGE_NEXT.swap(end as *const () as usize, std::sync::atomic::Ordering::Release);
        struct Restore(usize);
        impl Drop for Restore { fn drop(&mut self) { DAMAGE_NEXT.store(self.0, std::sync::atomic::Ordering::Release); } }
        let _restore = Restore(previous);
        for cat in 0..6 {
            for green in [false, true] {
                let mut row = [0u8; 256];
                row[128 + 0x38] = 5;
                row[0x38] = cat;
                row[0x50..0x60].fill(0x77);
                row[0x60..0x70].fill(0x88);
                if green { row[0x34..0x38].copy_from_slice(&crate::current::HEAL_TEXT_COLOR.to_le_bytes()); }
                for (register, stub) in [(3, damage_entry::<3> as unsafe extern "C" fn()),
                                         (14, damage_entry::<14>), (crate::current::DAMAGE_TEXT_ROW_REGISTER, DAMAGE_ENTRY)] {
                    unsafe { invoke(row.as_mut_ptr(), 0.0275, stub, register); }
                    let expected = match (green, cat) { (false, 0) => 0.0275 / 3.0, (false, 2) => 0.0275 / 2.0, _ => 0.0275 };
                    let (fresh, other) = if expected_register == 8 { (0x60, 0x50) } else { (0x50, 0x60) };
                    let read = |at| f32::from_le_bytes(row[at..at + 4].try_into().unwrap());
                    assert!((read(fresh) - expected).abs() < 1e-8);
                    assert_eq!(read(other), 0.0275);
                    assert_eq!(&row[0x54..0x60], &[0x77; 12]);
                    assert_eq!(&row[0x64..0x70], &[0x88; 12]);
                    assert_eq!(u64::from_le_bytes(row[0x20..0x28].try_into().unwrap()) & 0x41, 0x40);
                }
            }
        }
    }
    #[test]
    fn split_native_query_preserves_its_three_args_and_lookup_pointer_wrapper() {
        use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
        static ACCEPT: AtomicU8 = AtomicU8::new(1);
        static LOOKUPS: AtomicUsize = AtomicUsize::new(0);
        unsafe extern "system" fn predicate(target: u64, objects: u64, query: *const u8) -> u8 {
            assert_eq!((target, objects), (0x123456789abcdef0, 0x23456789abcdef01));
            assert_eq!(std::slice::from_raw_parts(query, 24), &[0x42; 24]);
            ACCEPT.load(Ordering::SeqCst)
        }
        unsafe extern "system" fn lookup(map_reference: *const u64, target: u64) -> u64 {
            assert_eq!((*map_reference, target), (0x23456789abcdef01, 0x123456789abcdef0));
            LOOKUPS.fetch_add(1, Ordering::SeqCst);
            0x3456789abcdef012
        }
        let query = [0x42u8; 24];
        let resolve = || unsafe { resolve_with_native_parts(predicate, lookup, 0x123456789abcdef0, 0x23456789abcdef01, query.as_ptr()) };
        assert_eq!(resolve().unwrap(), 0x3456789abcdef012);
        ACCEPT.store(0, Ordering::SeqCst);
        assert_eq!(resolve().unwrap(), 0);
        ACCEPT.store(2, Ordering::SeqCst);
        assert!(resolve().is_err());
        assert_eq!(LOOKUPS.load(Ordering::SeqCst), 1);
    }
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
        put(&mut m, 0xc8, 0x8000_0000_0000_0097);
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
        put(&mut m, 0xc8, 0x8000_0000_0000_0096);
        assert!(adapter().native_heal_texts(&m, 0, false, &[11, 22]).unwrap().is_empty());
    }
    #[test]
    fn periodic_heals_preserve_packet_order_and_exclude_native_damage() {
        let mut m = Bytes {
            data: vec![0; 0x1000],
        };
        put(&mut m, 0x2d0, 0x8000_0000_0000_0096);
        put(&mut m, 0x2d8, 3);
        put(&mut m, 0x2e0, 0x700);
        put(&mut m, 0x2e8, 3);
        put(&mut m, 0x2f0, 11);
        put(&mut m, 0x2f8, 22);
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
        put(&mut m, 0x2e8, 33);
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
