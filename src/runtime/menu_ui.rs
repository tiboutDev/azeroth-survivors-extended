//! Decorations only: native lobby, Co-op button and menu lifecycle stay native.
use super::*;
use asx_game::official_native::MenuAssets;
static CAPTURE: AtomicUsize = AtomicUsize::new(0);
static BUTTON: AtomicUsize = AtomicUsize::new(0);
static ASSETS: OnceLock<MenuAssets> = OnceLock::new();
static GROUP: AtomicUsize = AtomicUsize::new(0);
static STATUS: AtomicUsize = AtomicUsize::new(0);
static CACHE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static CACHE_DISPATCH: AtomicUsize = AtomicUsize::new(0);
static CACHE_MISS: AtomicUsize = AtomicUsize::new(0);
static PROJECTION: std::sync::Mutex<Option<asx_gameplay::menu_projection::MenuProjection>> =
    std::sync::Mutex::new(None);
crate::context_stub_rw!(cache_entry, 0, cache, CACHE_DISPATCH);
#[unsafe(naked)]
unsafe extern "C" fn resume_cache() {
    core::arch::naked_asm!("pushfq","test rax,rax","jz 2f","popfq","jmp qword ptr [rip+{miss}]","2:","popfq","jmp qword ptr [rip+{original}]",miss=sym CACHE_MISS,original=sym CACHE_ORIGINAL);
}
unsafe extern "system" fn cache(c: &mut crate::stub::Context) {
    c.rax = u64::from(
        PROJECTION
            .lock()
            .ok()
            .is_some_and(|mut p| p.as_mut().is_some_and(|p| p.request())),
    );
    #[cfg(feature = "test-hooks")]
    if c.rax != 0 {
        if let Some(r) = RUNTIME.get() {
            r.log
                .info("official native menu cache: roster changed; one native rebuild requested");
        }
    }
}
static MEMBERS: std::sync::Mutex<Option<Vec<(u64, u8)>>> = std::sync::Mutex::new(None);
thread_local! {static CARDS:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
crate::context_stub_rw!(group_entry, 0, group, GROUP);

unsafe extern "system" fn group(c: &mut crate::stub::Context) {
    if let Some(r)=RUNTIME.get(){bonus_ui::size_header(r,c);}
    if !CARDS.with(|v| v.get()) {
        return;
    }
    if let Some(r) = RUNTIME.get() {
        if let Some(caller) = (c.return_address() as u64).checked_sub(local_base() as u64) {
            let io = io::Access {
                local: &r.local,
                adapter: r.adapter,
            };
            let _ = r
                .adapter
                .menu_bindings(local_base())
                .size_card(&io, caller, c.r8);
        }
    }
}
type Status = unsafe extern "system" fn(u64, u64, u64, u64) -> usize;
unsafe extern "system" fn status(a: u64, resource: u64, query: u64, extra: u64) -> usize {
    let original = STATUS.load(Ordering::Acquire);
    if let Some(r) = RUNTIME.get() {
        if let Ok(rows) = r
            .adapter
            .menu_bindings(local_base())
            .members(&r.local, resource)
        {
            if let Ok(mut members) = MEMBERS.lock() {
                *members = Some(rows.clone());
            }
            if let Ok(mut projection) = PROJECTION.lock() {
                projection.get_or_insert_with(Default::default).update(rows);
            }
        }
        let refresh = PROJECTION
            .lock()
            .ok()
            .is_some_and(|mut p| p.as_mut().is_some_and(|p| p.refresh_caption()));
        if refresh {
            match r.adapter.menu_bindings(local_base()).refresh_status(
                &r.local,
                original,
                [a, resource, query, extra],
            ) {
                Ok(result) => return result,
                Err(error) => r
                    .log
                    .warn(&format!("Native menu status refresh rejected: {error}")),
            }
        }
    }
    let call: Status = std::mem::transmute(original);
    call(a, resource, query, extra)
}
crate::context_stub_rw!(capture_entry, 0, capture, CAPTURE);
unsafe extern "system" fn capture(c: &mut crate::stub::Context) {
    if ASSETS.get().is_some() {
        return;
    }
    let Some(r) = RUNTIME.get() else { return };
    if let Ok(Some(assets)) = r
        .adapter
        .menu_bindings(local_base())
        .capture(&r.local, c.rbp)
    {
        let _ = ASSETS.set(assets);
        #[cfg(feature = "test-hooks")]
        r.log.info("official menu atlas captured");
    }
}
type Button = unsafe extern "system" fn(*mut u8, usize) -> usize;
unsafe extern "system" fn button(parent: *mut u8, font: usize) -> usize {
    let original: Button = std::mem::transmute(BUTTON.load(Ordering::Acquire));
    let result = original(parent, font);
    if let (Some(r), Some(assets)) = (RUNTIME.get(), ASSETS.get()) {
        let members = MEMBERS
            .lock()
            .ok()
            .and_then(|v| v.clone())
            .unwrap_or_else(party_ui::lobby_classes);
        if let Ok(mut projection) = PROJECTION.lock() {
            projection
                .get_or_insert_with(Default::default)
                .update(members.clone());
        }
        let classes = members.iter().map(|(_, class)| *class).collect::<Vec<_>>();
        let previous = CARDS.with(|v| v.replace(true));
        let count = r.adapter.menu_bindings(local_base()).draw(
            &r.local,
            parent as u64,
            font,
            assets,
            &classes,
        );
        CARDS.with(|v| v.set(previous));
        if count == members.len() {
            if let Ok(mut projection) = PROJECTION.lock() {
                projection
                    .get_or_insert_with(Default::default)
                    .rendered(members.clone());
            }
        }
        #[cfg(feature = "test-hooks")]
        if std::env::var("AZCOOP_TEST_OFFICIAL_PARTY_UI").as_deref() == Ok("1") {
            r.log.info(&format!("official native menu icons: members={members:?} count={count} retained=true native_button=true"));
        }
        #[cfg(not(feature = "test-hooks"))]
        let _ = count;
    }
    result
}
pub fn hooks(adapter: Adapter) -> Vec<crate::install::HookSpec> {
    let entries = adapter.menu_entries();
    CACHE_DISPATCH.store(resume_cache as *const () as usize, Ordering::Release);
    CACHE_MISS.store(
        local_base() + adapter.menu_cache_entries()[1].rva as usize,
        Ordering::Release,
    );
    [
        (0, capture_entry as *const () as usize, &CAPTURE),
        (1, button as *const () as usize, &BUTTON),
        (4, group_entry as *const () as usize, &GROUP),
        (5, status as *const () as usize, &STATUS),
    ]
    .into_iter()
    .map(|(index, detour, trampoline)| crate::install::HookSpec {
        name: "official-native-menu-class-decoration",
        target: local_base() + entries[index].rva as usize,
        detour,
        trampoline,
    })
    .chain(std::iter::once(crate::install::HookSpec {
        name: "official-native-menu-cache-roster",
        target: local_base() + adapter.menu_cache_entries()[0].rva as usize,
        detour: cache_entry as *const () as usize,
        trampoline: &CACHE_ORIGINAL,
    }))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "system" fn native(parent: *mut u8, font: usize) -> usize {
        assert_eq!((parent as usize, font), (0x10000, 0x20000));
        99
    }
    #[test]
    fn native_button_constructor_keeps_parent_font_return_and_scope() {
        BUTTON.store(native as *const () as usize, Ordering::Release);
        assert_eq!(unsafe { button(0x10000 as *mut u8, 0x20000) }, 99);
    }
    #[unsafe(naked)]
    unsafe extern "C" fn ordinary() -> u64 {
        core::arch::naked_asm!("mov rax,11", "ret");
    }
    #[unsafe(naked)]
    unsafe extern "C" fn missed() -> u64 {
        core::arch::naked_asm!("mov rax,22", "ret");
    }
    #[unsafe(naked)]
    unsafe extern "system" fn probe(out: *mut u64, decision: u64) {
        core::arch::naked_asm!("push rbx","sub rsp,32","mov rbx,rcx","mov rax,rdx","cmp rax,rax","stc","call {resume}","pushfq","pop rdx","mov [rbx],rax","mov [rbx+8],rdx","add rsp,32","pop rbx","ret",resume=sym resume_cache);
    }
    #[test]
    fn native_menu_cache_continuation_selects_miss_and_preserves_flags() {
        CACHE_ORIGINAL.store(ordinary as *const () as usize, Ordering::Release);
        CACHE_MISS.store(missed as *const () as usize, Ordering::Release);
        for (decision, expected) in [(0, 11), (1, 22)] {
            let mut out = [0u64; 2];
            unsafe {
                probe(out.as_mut_ptr(), decision);
            }
            assert_eq!(out[0], expected);
            assert_eq!(out[1] & 0x41, 0x41);
        }
    }
    unsafe extern "system" fn native_status(a: u64, b: u64, c: u64, d: u64) -> usize {
        assert_eq!([a, b, c, d], [11, 22, 33, 44]);
        77
    }
    #[test]
    fn native_status_preserves_all_arguments_and_original_return() {
        STATUS.store(native_status as *const () as usize, Ordering::Release);
        assert_eq!(unsafe { status(11, 22, 33, 44) }, 77);
    }
}
