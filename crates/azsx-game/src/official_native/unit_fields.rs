//! Native unit-field insertion. The selected ABI owns the value register.
use crate::Result;

unsafe fn write_field(entry: usize, abi: u8, map: u64, key: u16, bits: u32) -> Result<()> {
    match abi {
        0 => {
            let call: unsafe extern "system" fn(u64, u16, u32) =
                std::mem::transmute(entry);
            call(map, key, bits);
        }
        1 => {
            // The 0.140.0 helper stores XMM2 with MOVSS, without arithmetic.
            // Preserve arbitrary field bits, including integer and NaN payloads.
            let call: unsafe extern "system" fn(u64, u16, f32) =
                std::mem::transmute(entry);
            call(map, key, f32::from_bits(bits));
        }
        _ => return Err("Unknown native unit-field ABI".into()),
    }
    Ok(())
}

/// The entry must be the verified selected native unit-field writer.
pub unsafe fn call_native_unit_field(entry: usize, map: u64, key: u16, bits: u32) -> Result<()> {
    write_field(entry, crate::current::UNIT_FIELD_VALUE_ABI, map, key, bits)
}

pub fn native_unit_field_value(r8: u64, xmm2: u128) -> Result<u32> {
    match crate::current::UNIT_FIELD_VALUE_ABI {
        0 => Ok(r8 as u32),
        1 => Ok(xmm2 as u32),
        _ => Err("Unknown native unit-field ABI".into()),
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Default)]
    struct Capture {
        key: u16,
        bits: u32,
    }

    unsafe extern "system" fn integer(map: u64, key: u16, bits: u32) {
        let capture = &mut *(map as *mut Capture);
        capture.key = key;
        capture.bits = bits;
    }

    unsafe extern "system" fn float_bits(map: u64, key: u16, value: f32) {
        integer(map, key, value.to_bits());
    }

    #[test]
    fn native_field_abis_preserve_key_and_all_value_bits() {
        for (entry, abi) in [(integer as *const () as usize, 0), (float_bits as *const () as usize, 1)] {
            for bits in [0, 10000, 0x8000_0000, 0x7fc0_1234, u32::MAX] {
                let mut capture = Capture::default();
                unsafe { write_field(entry, abi, &mut capture as *mut _ as u64, 0x498, bits).unwrap() };
                assert_eq!(capture.key, 0x498);
                assert_eq!(capture.bits, bits);
            }
        }
    }

    #[test]
    fn unknown_native_field_abi_never_calls_an_entry() {
        assert!(unsafe { write_field(0, 2, 0, 0, 0) }.is_err());
    }
}
