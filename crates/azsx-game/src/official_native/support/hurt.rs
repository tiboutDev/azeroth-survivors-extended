//! Native incoming-hit attribution and owned snapshots for deferred taunt delivery.
use super::*;

/// Historical native caller: source kind and spell occupy two stack words.
pub type NativeHurtCall = unsafe extern "system" fn(u64, f32, u64, bool, u32, u32);
/// Reviewed descriptor caller: the fifth argument borrows a 24-byte source enum.
pub type NativeHurtBorrowedCall = unsafe extern "system" fn(u64, f32, u64, bool, *const u8);

#[derive(Clone, Debug)]
pub struct NativeHurtSource(Source);
#[derive(Clone, Debug)]
enum Source {
    Words { kind: u32, spell: u32 },
    Descriptor { bytes: [u8; 24], key: Option<Vec<u8>> },
}
impl NativeHurtSource {
    pub fn for_hit(melee: bool) -> Self {
        Self(Source::Words { kind: u32::from(!melee), spell: 0 })
    }
}

fn capture(mem: &dyn Mem, layout: u8, source: u64, spell: u32) -> Result<NativeHurtSource> {
    if layout == 0 {
        return Ok(NativeHurtSource(Source::Words { kind: source as u32, spell }));
    }
    if layout != 1 { return Err("Unreviewed native hurt source layout".into()); }
    if source < 0x10000 || source % 8 != 0 || source.checked_add(24).is_none() {
        return Err("Invalid native hurt source".into());
    }
    let bytes: [u8; 24] = mem.read(source, 24)?.try_into()
        .map_err(|_| "Truncated native hurt source")?;
    let tag = u32::from_le_bytes(bytes[..4].try_into().unwrap());
    let key = match tag {
        0..=2 => None,
        3 => {
            let pointer = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
            let length = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
            if length > 4096 || length > 0 && (pointer < 0x10000 || pointer.checked_add(length).is_none()) {
                return Err("Invalid native hurt source key".into());
            }
            let key = if length == 0 { vec![] } else { mem.read(pointer, length as usize)? };
            if key.len() != length as usize || std::str::from_utf8(&key).is_err() {
                return Err("Invalid native hurt source text".into());
            }
            Some(key)
        }
        _ => return Err("Unknown native hurt source tag".into()),
    };
    Ok(NativeHurtSource(Source::Descriptor { bytes, key }))
}

unsafe fn call(entry: usize, layout: u8, game: u64, amount: f32, enemy: u64,
               melee: bool, source: &NativeHurtSource) -> Result<()> {
    match (layout, &source.0) {
        (0, Source::Words { kind, spell }) => {
            let native: NativeHurtCall = std::mem::transmute(entry);
            native(game, amount, enemy, melee, *kind, *spell);
        }
        (1, source) => {
            let mut descriptor = match source {
                Source::Words { kind, spell } if *kind <= 2 => {
                    let mut bytes = [0u8; 24];
                    bytes[..4].copy_from_slice(&kind.to_le_bytes());
                    bytes[4..8].copy_from_slice(&spell.to_le_bytes());
                    bytes
                }
                Source::Words { .. } => return Err("Unsupported generated hurt source".into()),
                Source::Descriptor { bytes, .. } => *bytes,
            };
            if let Source::Descriptor { key: Some(key), .. } = source {
                descriptor[8..16].copy_from_slice(&(key.as_ptr() as u64).to_le_bytes());
                descriptor[16..24].copy_from_slice(&(key.len() as u64).to_le_bytes());
            }
            let native: NativeHurtBorrowedCall = std::mem::transmute(entry);
            // The native enum contains 64-bit fields and requires eight-byte alignment.
            let words: [u64; 3] = std::array::from_fn(|i|
                u64::from_le_bytes(descriptor[i * 8..i * 8 + 8].try_into().unwrap()));
            native(game, amount, enemy, melee, words.as_ptr().cast());
        }
        _ => return Err("Native hurt source/layout mismatch".into()),
    }
    Ok(())
}

impl Adapter {
    pub fn hurt_source_layout(self) -> u8 { crate::current::HURT_SOURCE_LAYOUT }
    pub fn capture_hurt_source(self, mem: &dyn Mem, source: u64, spell: u32) -> Result<NativeHurtSource> {
        capture(mem, self.hurt_source_layout(), source, spell)
    }
    /// # Safety
    /// `entry` must be the reviewed original incoming-hit routine for the selected
    /// layout. Game must remain valid; enemy must be a permitted native GUID.
    pub unsafe fn call_hurt(self, entry: usize, game: u64, amount: f32, enemy: u64,
                            melee: bool, source: &NativeHurtSource) -> Result<()> {
        call(entry, self.hurt_source_layout(), game, amount, enemy, melee, source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Memory(Vec<u8>);
    impl Mem for Memory {
        fn read(&self, at: u64, size: usize) -> Result<Vec<u8>> {
            self.0.get(at as usize..at as usize + size).map(Vec::from).ok_or("Out of bounds".into())
        }
    }
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    static OBSERVED: std::sync::Mutex<Vec<(u64, f32, u64, bool, u32, u32, Vec<u8>)>> = std::sync::Mutex::new(vec![]);
    unsafe extern "system" fn words(game: u64, amount: f32, enemy: u64, melee: bool, kind: u32, spell: u32) {
        OBSERVED.lock().unwrap().push((game, amount, enemy, melee, kind, spell, vec![]));
    }
    unsafe extern "system" fn descriptor(game: u64, amount: f32, enemy: u64, melee: bool, source: *const u8) {
        assert_eq!(source as usize % 8, 0);
        let bytes = std::slice::from_raw_parts(source, 24);
        let kind = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let spell = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let key = if kind == 3 {
            let pointer = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
            let length = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
            std::slice::from_raw_parts(pointer as *const u8, length as usize).to_vec()
        } else { vec![] };
        OBSERVED.lock().unwrap().push((game, amount, enemy, melee, kind, spell, key));
    }
    #[test]
    fn source_snapshot_owns_custom_text_after_native_storage_is_reused() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut mem = Memory(vec![0; 0x12000]);
        mem.0[0x10000..0x10008].copy_from_slice(&[3u32.to_le_bytes(), 20475u32.to_le_bytes()].concat());
        mem.0[0x10008..0x10010].copy_from_slice(&0x11000u64.to_le_bytes());
        mem.0[0x10010..0x10018].copy_from_slice(&13u64.to_le_bytes());
        mem.0[0x11000..0x1100d].copy_from_slice(b"boss_mechanic");
        let source = capture(&mem, 1, 0x10000, 0).unwrap();
        mem.0.fill(0);
        let clone = source.clone();
        drop(source);
        unsafe { call(descriptor as *const () as usize, 1, 11, 3.5, 13, true, &clone).unwrap(); }
        assert_eq!(OBSERVED.lock().unwrap().pop(), Some((11, 3.5, 13, true, 3, 20475, b"boss_mechanic".to_vec())));
    }
    #[test]
    fn native_hurt_calls_preserve_both_selected_stack_contracts() {
        let _guard = TEST_LOCK.lock().unwrap();
        for (melee, kind, spell) in [(true, 0x11223344, 0x55667788), (false, 1, 17)] {
            let source = capture(&Memory(vec![]), 0, u64::from(kind), spell).unwrap();
            unsafe { call(words as *const () as usize, 0, 0x123456789abcdef0, 1.25, 0xf130000012345678, melee, &source).unwrap(); }
            assert_eq!(OBSERVED.lock().unwrap().pop(), Some((0x123456789abcdef0, 1.25, 0xf130000012345678, melee, kind, spell, vec![])));
        }
        for melee in [false, true] {
            unsafe { call(descriptor as *const () as usize, 1, 11, 2.5, 13, melee, &NativeHurtSource::for_hit(melee)).unwrap(); }
            assert_eq!(OBSERVED.lock().unwrap().pop(), Some((11, 2.5, 13, melee, u32::from(!melee), 0, vec![])));
        }
    }
    #[test]
    fn invalid_source_tags_and_key_extents_reject_deferred_delivery() {
        let mut mem = Memory(vec![0; 0x12000]);
        for (tag, pointer, length) in [(4u32, 0x11000u64, 1u64), (3, 0, 1), (3, u64::MAX - 3, 8), (3, 0x11000, 4097)] {
            mem.0[0x10000..0x10004].copy_from_slice(&tag.to_le_bytes());
            mem.0[0x10008..0x10010].copy_from_slice(&pointer.to_le_bytes());
            mem.0[0x10010..0x10018].copy_from_slice(&length.to_le_bytes());
            assert!(capture(&mem, 1, 0x10000, 0).is_err());
        }
        assert!(capture(&mem, 1, 3, 0).is_err());
        assert!(capture(&mem, 1, 0x10001, 0).is_err());
        assert!(capture(&mem, 2, 0x10000, 0).is_err());
    }
}
