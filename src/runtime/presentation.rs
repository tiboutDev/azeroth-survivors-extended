//! Bounded script diagnostics; never persist mailbox contents as player settings.
use super::*;
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
type Write = unsafe extern "system" fn(usize, *const u8, usize, *const u8, usize, u8) -> u8;
unsafe extern "system" fn write(
    cvars: usize,
    name: *const u8,
    len: usize,
    value: *const u8,
    size: usize,
    from_vm: u8,
) -> u8 {
    if from_vm == 1
        && len == 19
        && !name.is_null()
        && std::slice::from_raw_parts(name, len) == b"azcoop_native_probe"
    {
        #[cfg(feature = "test-hooks")]
        if std::env::var("AZCOOP_TEST_OFFICIAL_PARTY_UI").as_deref() == Ok("1")
            && !value.is_null()
            && size <= 16384
        {
            if let Ok(text) = std::str::from_utf8(std::slice::from_raw_parts(value, size)) {
                if text.starts_with("AZCOOP_PARTY115 ") && !text.chars().any(char::is_control) {
                    if let Some(r) = RUNTIME.get() {
                        r.log.info(text);
                    }
                }
            }
        }
        return 1;
    }
    let original: Write = std::mem::transmute(ORIGINAL.load(Ordering::Acquire));
    original(cvars, name, len, value, size, from_vm)
}
pub fn hook(adapter: Adapter) -> crate::install::HookSpec {
    crate::install::HookSpec {
        name: "official-native-ui-mailbox",
        target: local_base() + adapter.ui_mailbox_writer().expect("Selected diagnostic capability").rva as usize,
        detour: write as *const () as usize,
        trampoline: &ORIGINAL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "system" fn native(
        cvars: usize,
        name: *const u8,
        len: usize,
        value: *const u8,
        size: usize,
        from_vm: u8,
    ) -> u8 {
        assert_eq!(cvars, 0x1234);
        assert!(!name.is_null());
        assert!(len > 0);
        assert!(!value.is_null());
        assert_eq!(size,3);
        assert!(from_vm <= 1);
        7
    }
    #[test]
    fn reserved_probe_is_ephemeral_and_unrelated_calls_preserve_native_abi() {
        ORIGINAL.store(native as *const () as usize, Ordering::Release);
        unsafe {
            let key = b"azcoop_native_probe";
            // Consumed without a native CVar map or persisted setting, even for invalid diagnostics.
            assert_eq!(
                write(0, key.as_ptr(), key.len(), std::ptr::null(), usize::MAX, 1),
                1
            );
            assert_eq!(
                write(0x1234, key.as_ptr(), key.len(), b"abc".as_ptr(), 3, 0),
                7
            );
            let setting = b"MasterVolume";
            assert_eq!(
                write(
                    0x1234,
                    setting.as_ptr(),
                    setting.len(),
                    b"abc".as_ptr(),
                    3,
                    1
                ),
                7
            );
            // A prefix collision must not consume an ordinary native setting.
            let other = b"azcoop_native_probe_extra";
            assert_eq!(
                write(0x1234, other.as_ptr(), other.len(), b"abc".as_ptr(), 3, 1),
                7
            );
        }
    }
}
