//! Adapt the native vplates inputs before construction. No Lua discovery or bars.
use super::*;
use crate::official::{Entry, MutableMemory};
fn word(m: &dyn Mem, a: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(
        m.read(a, 8)?
            .try_into()
            .map_err(|_| "Truncated plate pointer")?,
    ))
}
fn enemy(row: &[u8]) -> bool {
    let f = |n| f32::from_le_bytes(row[n..n + 4].try_into().unwrap());
    let (r, g, b) = (f(0x38), f(0x3c), f(0x40));
    r.is_finite() && g.is_finite() && b.is_finite() && r > 0.75 && b < 0.25
}
// An unavailable classification is not evidence that a native name is ordinary.
fn keep_native_name(rank:Option<u32>)->bool{rank!=Some(0)}
#[cfg(feature="test-hooks")]
#[derive(Debug)]
pub struct PlateWitness{pub guid:u64,pub rank:Option<u32>,pub name:Option<String>,pub retained:bool,pub unit_count:u64}
#[cfg(feature="test-hooks")]
static WITNESSES:std::sync::Mutex<(Vec<u64>,Vec<PlateWitness>)>=std::sync::Mutex::new((Vec::new(),Vec::new()));
#[cfg(feature="test-hooks")]
pub fn take_plate_witnesses()->Vec<PlateWitness>{WITNESSES.lock().map(|mut v|v.1.drain(..).collect()).unwrap_or_default()}
impl Adapter {
    pub fn nameplate_entries(self) -> [Entry; 2] {
        [crate::current::PLATE_UPDATE, crate::current::PLATE_STYLE]
            .map(|(rva, bytes)| Entry { rva, bytes })
    }
    pub fn native_ui_participants(self, m: &dyn Mem, script: u64) -> Result<Vec<u64>> {
        if script < 0x10000 {
            return Ok(vec![]);
        }
        let aliases = script + crate::current::UI_ALIASES;
        let own = word(m, aliases + crate::current::UI_SELF_GUID)?;
        if own == 0 {
            return Ok(vec![]);
        }
        let mut ids = vec![own];
        for raw in m
            .read(aliases + crate::current::UI_PARTY_GUIDS, 32)?
            .chunks_exact(8)
        {
            let guid = u64::from_le_bytes(raw.try_into().unwrap());
            if guid != 0 && !ids.contains(&guid) {
                ids.push(guid);
            }
        }
        Ok(ids)
    }
    /// Same GUID-keyed UnitInfo table and rank used by native UnitClassification.
    /// Bound hash probing; do not resolve enemies by duplicated display names.
    pub unsafe fn native_unit_rank(
        self,
        m: &dyn Mem,
        base: usize,
        script: u64,
        guid: u64,
    ) -> Result<Option<u32>> {
        use crate::current as v;
        let map = script + v::UI_UNIT_MAP;
        let ctrl = word(m, map)?;
        let mask = word(m, map + 8)?;
        let len = word(m, map + 24)?;
        if len == 0 {
            return Ok(None);
        }
        let capacity = mask.checked_add(1).ok_or("Native unit-map overflow")?;
        if ctrl < 0x10000 || !capacity.is_power_of_two() || capacity > 1 << 20 || len > capacity {
            return Err("Invalid native unit map".into());
        }
        type Hash = unsafe extern "system" fn(u64, *const u64) -> u64;
        let hash: Hash = std::mem::transmute(base + v::UI_UNIT_HASH.0 as usize);
        let hash = hash(map + 32, &guid);
        let tag = (hash >> 57) as u8;
        let (mut pos, mut step) = (hash & mask, 0u64);
        for _ in 0..16 {
            let group = m.read(ctrl + pos, 16)?;
            for (i, byte) in group.iter().enumerate() {
                if *byte != tag {
                    continue;
                }
                let slot = (pos + i as u64) & mask;
                let at = ctrl
                    .checked_sub((slot + 1) * v::UI_UNIT_STRIDE)
                    .ok_or("Native unit slot underflow")?;
                if word(m, at)? != guid {
                    continue;
                }
                let rank = u32::from_le_bytes(
                    m.read(at + 8 + v::UI_UNIT_RANK, 4)?
                        .try_into()
                        .map_err(|_| "Truncated native rank")?,
                );
                return Ok((rank <= 4).then_some(rank));
            }
            if group.contains(&0xff) {
                return Ok(None);
            }
            step += 16;
            pos = (pos + step) & mask;
        }
        Ok(None)
    }
    pub unsafe fn nameplate_rows(
        self,
        m: &dyn Mem,
        base: usize,
        script: u64,
        ptr: u64,
        len: u64,
    ) -> Result<Option<Vec<u8>>> {
        use crate::current as v;
        if len > 256 {
            return Err("Native plate slice exceeds ceiling".into());
        }
        if len == 0 {
            return Ok(None);
        }
        let mut rows = m.read(ptr, len as usize * v::PLATE_ROW_SIZE)?;
        let mut changed = false;
        for row in rows.chunks_exact_mut(v::PLATE_ROW_SIZE) {
            if enemy(row) {
                let guid = u64::from_le_bytes(row[0x20..0x28].try_into().unwrap());
                let rank = self.native_unit_rank(m, base, script, guid)?;
                let special = keep_native_name(rank);
                #[cfg(feature="test-hooks")]
                if std::env::var("ASX_TEST_UI_REMAINING").as_deref()==Ok("1"){
                    let has_name=u64::from_le_bytes(row[16..24].try_into().unwrap())>0;
                    let first=has_name&&WITNESSES.lock().map(|mut seen|{if seen.0.len()>=128||seen.0.contains(&guid){false}else{seen.0.push(guid);true}}).unwrap_or(false);
                    if first{
                        let ptr=u64::from_le_bytes(row[8..16].try_into().unwrap());
                        let len=u64::from_le_bytes(row[16..24].try_into().unwrap());
                        let name=if ptr>=0x10000&&(1..=128).contains(&len){m.read(ptr,len as usize).ok().and_then(|v|String::from_utf8(v).ok())}else{None};
                        let witness=PlateWitness{guid,rank,name,retained:special&&len>0,unit_count:word(m,script+v::UI_UNIT_MAP+24).unwrap_or(0)};
                        if let Ok(mut state)=WITNESSES.lock(){state.1.push(witness);}
                    }
                }
                if !special {
                    row[0x10..0x18].fill(0);
                }
                row[0x18..0x1c].fill(0);
                row[0x52] = 0;
                // Native anchors this root by its top midpoint. Screen X/Y are already
                // projected attachment coordinates, independent of the plate dimensions.
                // Preserve them exactly; applying a width-based shift misplaces the bar.
                changed = true;
            }
        }
        Ok(changed.then_some(rows))
    }
    pub fn nameplate_style(
        self,
        m: &dyn Mem,
        caller: u64,
        bp: u64,
        script: u64,
        index: u64,
        style: u64,
    ) -> Result<Option<([f32; 13], u64, [u8; 52])>> {
        if !crate::current::PLATE_STYLE_CALLERS.contains(&caller) {
            return Ok(None);
        }
        let len = word(m, script + crate::current::PLATE_CACHE_LEN)?;
        if len > 256 || index >= len {
            return Err("Invalid native plate cache index".into());
        }
        let cache = word(m, script + crate::current::PLATE_CACHE_PTR)?
            + index * crate::current::PLATE_CACHE_ROW;
        let row = if caller == crate::current::PLATE_STYLE_CALLERS[0] {
            word(m, bp + 0x290)?
        } else {
            cache
        };
        if !enemy(&m.read(row, crate::current::PLATE_ROW_SIZE)?) {
            return Ok(None);
        }
        let original: [u8; 52] = m
            .read(style, 52)?
            .try_into()
            .map_err(|_| "Truncated native plate style")?;
        let mut scaled = std::array::from_fn(|i| {
            f32::from_le_bytes(original[i * 4..i * 4 + 4].try_into().unwrap())
        });
        if !scaled.iter().all(|f| f.is_finite() && f.abs() <= 4096.) {
            return Err("Invalid native plate geometry".into());
        }
        // These are the native width/height, bar insets/dimensions, label/icon sizes
        // and offsets. Resize native layout together; never scale world coordinates.
        let name_font = scaled[10];
        scaled[0] = scaled[4] + 2. * scaled[2];
        for f in &mut scaled {
            *f *= crate::current::NAMEPLATE_SCALE;
        }
        scaled[10] = name_font;
        m.read(cache + 0x5c, 52)?;
        Ok(Some((scaled, cache + 0x5c, original)))
    }
    pub fn restore_nameplate_style_cache(
        self,
        m: &dyn MutableMemory,
        cache: u64,
        original: &[u8; 52],
    ) -> Result<()> {
        m.write(cache, original)
    }
}

#[cfg(test)]mod name_policy_tests{
 use super::*;
 #[test]fn missing_rank_does_not_destroy_a_native_special_name(){
  assert!(keep_native_name(None));for rank in 1..=4{assert!(keep_native_name(Some(rank)));}assert!(!keep_native_name(Some(0)));
 }
}
