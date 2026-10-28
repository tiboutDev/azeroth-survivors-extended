//! ASX native loader for the reviewed official game engine.
//!
//! Direct native launch only forwards avrt.dll exports. The ASX launcher opts in
//! with AZCOOP_ENGINE=official; exact image selection and byte verification run
//! before the official feature hooks are installed. Custom simulation, transport,
//! lobby and measurement entry points are not part of this build.

#[macro_use]
mod proc;
pub mod heap_count;
#[global_allocator]
static HEAP: heap_count::Counting = heap_count::Counting;
mod stub;
mod install;
mod log;
pub mod crash;
mod runtime;
mod presentation;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod stub_tests;

use std::ffi::c_void;
use std::io::Read;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use sha2::{Digest, Sha256};
use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{CloseHandle, HINSTANCE, TRUE};
use windows_sys::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, GetModuleFileNameW};
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_sys::Win32::System::Threading::{CreateThread, GetCurrentProcessId};
use log::Log;

const LOG_NAME: &str = "azeroth-coop-native.log";

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(module: HINSTANCE, reason: u32, reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_DETACH {
        crash::process_detach(!reserved.is_null());
    }
    if reason == DLL_PROCESS_ATTACH {
        // Loader lock: no allocation, file access or native hooks here.
        unsafe {
            DisableThreadLibraryCalls(module);
            if attach_requested() {
                let thread = CreateThread(std::ptr::null(), 0, Some(worker), std::ptr::null(), 0, std::ptr::null_mut());
                if !thread.is_null() { CloseHandle(thread); }
            }
        }
    }
    TRUE
}

unsafe extern "system" fn worker(_: *mut c_void) -> u32 {
    if std::env::var_os("AZCOOP_NATIVE_DISABLE").is_some_and(|v| v == "1") {
        return 0;
    }
    // Direct native launch stays inert. Only the explicit official launcher route
    // may install ASX; retired custom-engine and measuring modes are ignored.
    if !requested() {
        return 0;
    }
    let Some(exe) = executable_path() else { return 0 };
    let game = exe.parent().map(PathBuf::from).unwrap_or_default();
    let diagnostics = std::env::var_os("AZCOOP_NATIVE_LOG_DIRECTORY").map(PathBuf::from).unwrap_or_else(|| game.clone());
    let _ = std::fs::create_dir_all(&diagnostics);
    let log_name = LOG_NAME.replace(".log", &format!("{}.log", crash::player_suffix()));
    let log: &'static Log = Box::leak(Box::new(Log::open_rotating(&diagnostics.join(log_name), 3)));
    crash::init(&diagnostics, log);
    crash::test_crash_from_env();
    log.info(&format!(
        "ASX {} (avrt.dll proxy), pid {}, exe {}",
        env!("CARGO_PKG_VERSION"),
        GetCurrentProcessId(),
        exe.display()
    ));
    let root = std::env::var("SystemRoot").unwrap_or_default();
    if !root.eq_ignore_ascii_case(env!("AZCOOP_FORWARD_ROOT")) {
        log.warn(&format!("avrt exports forward to {}, but SystemRoot is {root}", env!("AZCOOP_FORWARD_ROOT")));
    }
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(log, &exe)));
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(reason)) => {
            log.info(&format!("ASX inactive: {reason}; the game runs unmodified"));
        }
        Err(_) => {
            log.failed("worker", "panic; the game runs unmodified");
        }
    }
    0
}

fn attach_requested() -> bool {
    fn is_value(name: &str, expected: &str) -> bool {
        let mut wide = [0u16; 32];
        for (i, c) in name.encode_utf16().enumerate() {
            wide[i] = c;
        }
        let mut value = [0u16; 16];
        let n = unsafe { windows_sys::Win32::System::Environment::GetEnvironmentVariableW(wide.as_ptr(), value.as_mut_ptr(), value.len() as u32) };
        n > 0 && (n as usize) < value.len() && value[..n as usize].iter().copied().eq(expected.encode_utf16())
    }
    !is_value("AZCOOP_NATIVE_DISABLE", "1") && is_value("AZCOOP_ENGINE", "official")
}

fn official_requested() -> bool {
    std::env::var("AZCOOP_ENGINE").as_deref() == Ok("official")
}

fn requested() -> bool {
    !std::env::var_os("AZCOOP_NATIVE_DISABLE").is_some_and(|v| v == "1") && official_requested()
}

fn executable_path() -> Option<PathBuf> {
    let mut buffer = vec![0u16; 32_768];
    let length = unsafe { GetModuleFileNameW(std::ptr::null_mut(), buffer.as_mut_ptr(), buffer.len() as u32) };
    (length > 0).then(|| std::ffi::OsString::from_wide(&buffer[..length as usize]).into())
}

fn sha256_file(path: &std::path::Path) -> std::io::Result<(String, u64)> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 20];
    let mut total = 0u64;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        total += n as u64;
    }
    let digest = hasher.finalize();
    Ok((digest.iter().map(|b| format!("{b:02x}")).collect(), total))
}

fn run(log: &'static Log, exe: &std::path::Path) -> Result<(), String> {
    let (hash, _) = sha256_file(exe).map_err(|e| format!("cannot read executable: {e}"))?;
    runtime::install(log, &hash)
}

#[cfg(test)]
mod activation_tests {
    use super::*;
    #[test]
    fn native_and_retired_launcher_modes_cannot_activate_the_dll() {
        let _turn = crate::test_support::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        struct Restore(Vec<(&'static str, Option<std::ffi::OsString>)>);
        impl Drop for Restore {
            fn drop(&mut self) {
                for (name, value) in self.0.drain(..) {
                    if let Some(value) = value { std::env::set_var(name, value); }
                    else { std::env::remove_var(name); }
                }
            }
        }
        let _restore = Restore(["AZCOOP_ENGINE", "AZCOOP_MEASURE", "AZCOOP_NATIVE_DISABLE"].into_iter().map(|name| (name, std::env::var_os(name))).collect());
        std::env::remove_var("AZCOOP_NATIVE_DISABLE");
        for engine in [None, Some("1"), Some("unknown"), Some("official")] {
            if let Some(engine) = engine { std::env::set_var("AZCOOP_ENGINE", engine); }
            else { std::env::remove_var("AZCOOP_ENGINE"); }
            std::env::set_var("AZCOOP_MEASURE", "1");
            let expected = engine == Some("official");
            assert_eq!(requested(), expected);
            assert_eq!(attach_requested(), expected);
        }
        std::env::set_var("AZCOOP_NATIVE_DISABLE", "1");
        assert!(!requested());
        assert!(!attach_requested());
    }
}
