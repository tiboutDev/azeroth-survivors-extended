//! Seed only the Codex entries for trinkets excluded from AZSX rewards.
use super::{Runtime, RUNTIME};
use std::sync::atomic::AtomicUsize;

static OWNED: std::sync::Mutex<Option<Vec<u32>>> = std::sync::Mutex::new(None);
static SAVED: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(saving, 0, capture_save, SAVED);
fn capture(r:&Runtime, collection:u64) {
    let value=r.adapter.run_owned_items(&super::io::Access{local:&r.local,adapter:r.adapter},collection).ok();
    if let Ok(mut current)=OWNED.lock(){*current=value;}
}
unsafe extern "system" fn capture_save(c:&mut crate::stub::Context){
    if let Some(r)=RUNTIME.get(){capture(r,c.rcx)}
}
pub(super) fn drop_table(r:&Runtime)->Result<String,String>{
    let owned=OWNED.lock().map_err(|_|"Collection unavailable")?.clone().ok_or("Collection not loaded")?;
    unsafe {r.adapter.run_drop_lua(&super::io::Access{local:&r.local,adapter:r.adapter},r.local.base,&owned)}
}
static AUTHENTICATED: AtomicUsize = AtomicUsize::new(0);
static LOADED: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(authenticated, 1, discover, AUTHENTICATED);
crate::context_stub_rw!(loaded, 0, discover, LOADED);

// The native loader also returns a default collection on read/authentication errors.
// Only a genuinely absent profile is safe to initialize through that path.
fn missing_native_profile() -> bool {
    use std::{env, fs, io::ErrorKind, path::PathBuf};
    let path = env::var_os("WOW_ROGUE_SAVE")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("APPDATA")
                .map(|dir| PathBuf::from(dir).join("wenilla-rogue/characters.txt"))
        });
    path.is_some_and(|p| matches!(fs::symlink_metadata(p), Err(e) if e.kind()==ErrorKind::NotFound))
}

unsafe extern "system" fn discover(context: &mut crate::stub::Context) {
    if let Some(r) = RUNTIME.get() {
        capture(r,context.rsi);
        let count = r.adapter.discover_economy_trinkets(
            r.local.base,
            context.rsi as usize,
            context.id == 1 || missing_native_profile(),
        );
        if count > 0 {
            r.log.info(&format!(
                "Codex: discovered {count} excluded economy trinket(s)."
            ));
        }
    }
}

pub(super) fn install(r: &Runtime) -> Result<(), String> {
    let entries = r.adapter.codex_entries();
    for entry in &entries {
        if r.local
            .read(r.local.base + entry.rva as usize, entry.bytes.len())?
            != entry.bytes
        {
            return Err("Native Codex boundary changed".into());
        }
    }
    crate::install::install(&[
        crate::install::HookSpec {
            name: "run-info-collection-save",
            target: r.local.base+entries[3].rva as usize,
            detour: saving as *const () as usize,
            trampoline: &SAVED,
        },
        crate::install::HookSpec {
            name: "codex-authenticated-profile",
            target: r.local.base + entries[0].rva as usize,
            detour: authenticated as *const () as usize,
            trampoline: &AUTHENTICATED,
        },
        crate::install::HookSpec {
            name: "codex-loaded-profile",
            target: r.local.base + entries[1].rva as usize,
            detour: loaded as *const () as usize,
            trampoline: &LOADED,
        },
    ])
}
