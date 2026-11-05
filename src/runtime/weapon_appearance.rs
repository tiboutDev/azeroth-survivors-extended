//! Project the run character's visible hands after the native Look resolver.
use super::{io, Runtime, RUNTIME};
use std::sync::atomic::{AtomicUsize, Ordering};
static LOOK: AtomicUsize = AtomicUsize::new(0);
static ITEM: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(look, 0, project, LOOK);
unsafe extern "system" fn project(c: &mut crate::stub::Context) {
    let Some(r) = RUNTIME.get() else { return };
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    let profile = r.adapter.weapon_run_profile([c.rdi, c.r15]);
    let result = r
        .adapter
        .weapon_projection(&io, r.local.base as u64, profile, c.rax);
    #[cfg(feature = "test-hooks")]
    if std::env::var("AZSX_TEST_WEAPONS").is_ok() {
        r.log.info(&format!(
            "weapon look: class={:?} hands={:?} result={result:?}",
            r.local.read(
                (profile + azsx_game::current::WEAPON_PROFILE_FIELDS[0]) as usize,
                2,
            ),
            r.local.read((c.rax + 60) as usize, 8)
        ));
    }
    if let Err(e) = result {
        r.log.warn(&format!("Weapon appearance rejected: {e}"))
    }
}
type Item = unsafe extern "system" fn(u64, u32) -> u64;
unsafe extern "system" fn item(out: u64, entry: u32) -> u64 {
    let original: Item = std::mem::transmute(ITEM.load(Ordering::Acquire));
    let Some(r) = RUNTIME.get() else {
        return original(out, entry);
    };
    let Some(template) = r.adapter.weapon_template(entry) else {
        return original(out, entry);
    };
    let result = original(out, template);
    let io = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    if let Err(e) = r.adapter.weapon_metadata(&io, out, entry) {
        r.log.warn(&format!("Weapon visual metadata rejected: {e}"))
    }
    #[cfg(feature = "test-hooks")]
    if std::env::var("AZSX_TEST_WEAPONS").is_ok() {
        r.log.info(&format!("weapon visual item: {entry}"))
    }
    result
}
use azsx_game::official::{piece_look, selected_piece, WORN_NATIVE};
const MAIN: usize = 15;
const OFF: usize = 16;
// Collection cells call this predicate to decide whether to draw their native L.
static MARK_LOOK: AtomicUsize = AtomicUsize::new(0);
unsafe extern "system" fn mark_look(looks: u64, item: u32) -> bool {
    let native: unsafe extern "system" fn(u64, u32) -> bool =
        std::mem::transmute(MARK_LOOK.load(Ordering::Acquire));
    if native(looks, item) { return true; }
    for class in [1, 2, 7] {
        for slot in [MAIN, OFF] {
            if let Some(encoded) = piece_look(class, slot, item) {
                return native(looks, encoded);
            }
        }
    }
    false
}
// The Showing row resolves the chosen collection item through the native race
// variant and item-name formatters. Decode only its temporary lookup argument.
static SHOWING: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(showing_entry, 0, showing, SHOWING);
unsafe extern "system" fn showing(c: &mut crate::stub::Context) {
    for class in [1, 2, 7] {
        for slot in [MAIN, OFF] {
            if let Some(item) = selected_piece(class, slot, c.rcx as u32) {
                c.rcx = u64::from(item);
                return;
            }
        }
    }
}
static BUTTON: AtomicUsize = AtomicUsize::new(0);
static VALIDATE: AtomicUsize = AtomicUsize::new(0);
static RESOLVE: AtomicUsize = AtomicUsize::new(0);
type Resolve = unsafe extern "system" fn(u64, u8, u8, u64, u64) -> u64;
unsafe extern "system" fn resolve(out: u64, race: u8, class: u8, equipped: u64, looks: u64) -> u64 {
    let original: Resolve = std::mem::transmute(RESOLVE.load(Ordering::Acquire));
    if !matches!(class, 1 | 2 | 7) {
        return original(out, race, class, equipped, looks);
    }
    let Some(r) = RUNTIME.get() else {
        return original(out, race, class, equipped, looks);
    };
    let Ok(mut copy) = r.local.read(looks as usize, 19 * 8) else {
        return original(out, race, class, equipped, looks);
    };
    for slot in [MAIN, OFF] {
        if copy[slot * 8..slot * 8 + 8] == manual_worn() {
            copy[slot * 8..slot * 8 + 8].fill(0);
        } else if let Some(item) = decode_slot(&copy, class, slot) {
            copy[slot * 8 + 4..slot * 8 + 8].copy_from_slice(&item.to_le_bytes());
        }
    }
    original(out, race, class, equipped, copy.as_ptr() as u64)
}
fn manual_worn() -> [u8; 8] {
    ((u64::from(WORN_NATIVE) << 32) | 1).to_le_bytes()
}
fn decode_slot(gear: &[u8], class: u8, slot: usize) -> Option<u32> {
    let row = &gear[slot * 8..slot * 8 + 8];
    if row[..4] != 1u32.to_le_bytes() {
        return None;
    }
    selected_piece(class, slot, u32::from_le_bytes(row[4..].try_into().ok()?))
}
type Validate = unsafe extern "system" fn(u64, u64, u64, usize, u8) -> u64;
unsafe extern "system" fn validate(out: u64, gear: u64, known: u64, n: usize, race: u8) -> u64 {
    let original: Validate = std::mem::transmute(VALIDATE.load(Ordering::Acquire));
    let Some(r) = RUNTIME.get() else {
        return original(out, gear, known, n, race);
    };
    let Ok(before) = r.local.read(gear as usize, 0xe8) else {
        return original(out, gear, known, n, race);
    };
    let class = before[0xe4];
    for slot in [MAIN, OFF] {
        if let Some(item) = decode_slot(&before, class, slot) {
            let _ = r
                .local
                .write(gear as usize + slot * 8 + 4, &item.to_le_bytes());
        }
    }
    let result = original(out, gear, known, n, race);
    if matches!(before[0xe4], 1 | 2 | 7) {
        for slot in [MAIN, OFF] {
            if before[slot * 8..slot * 8 + 8] == manual_worn() {
                let _ = r.local.write((out as usize) + slot * 8, &manual_worn());
            } else if let Some(item) = decode_slot(&before, class, slot) {
                let at = out as usize + slot * 8;
                let expected = ((u64::from(item) << 32) | 1).to_le_bytes();
                if r.local.read(at, 8).ok().as_deref() == Some(expected.as_slice()) {
                    let _ = r.local.write(at, &before[slot * 8..slot * 8 + 8]);
                }
            }
        }
    }
    result
}
crate::context_stub_rw!(button_entry, 0, button, BUTTON);
unsafe extern "system" fn button(c: &mut crate::stub::Context) {
    let Some(r) = RUNTIME.get() else { return };
    let data = |at: u64, n: usize| r.local.read(at as usize, n).ok();
    let word = |at: u64| -> Option<u64> { Some(u64::from_le_bytes(data(at, 8)?.try_into().ok()?)) };
    let Some(slot) = word(c.rbp + 0x450).and_then(word) else {
        return;
    };
    if !matches!(slot, 15 | 16) {
        return;
    }
    let Some(view) = word(c.rbp + 0x468) else {
        return;
    };
    let Some(selector) = word(view + 0x118) else {
        return;
    };
    if !data(selector + 0x30, 1).is_some_and(|v| matches!(v[0], 1 | 2 | 7)) {
        return;
    }
    let Some(current) = word(c.r15) else { return };
    let Ok(mut action) = r.local.read((c.rbp + 0xb0) as usize, 16) else {
        return;
    };
    // Borrow the same button parent, theme and action type as the Worn button.
    // Native action None is the default automatic look. Explicit Worn is the
    // manual Worn sentinel; selected spec pieces carry their native item ID.
    let [add_button, owned_item] = r.adapter.weapon_button_entries();
    let add: unsafe extern "system" fn(u64, u64, *const u8, usize, f32, bool, *const u8, bool) =
        std::mem::transmute(r.local.base + add_button.rva as usize);
    let label = b"Worn piece (spec)";
    add(
        c.rbp + 0x2a8,
        c.rbx,
        label.as_ptr(),
        label.len(),
        13.,
        current as u32 == 0,
        action.as_ptr(),
        false,
    );
    // Mirror the native This piece checks at 0x3c18f2: selected item, slot and ownership.
    if let (Some(piece), Some(collection), Some(class)) =
        (word(c.r13), word(view + 0x120), data(selector + 0x30, 1))
    {
        if piece != 0 && collection != 0 && data(piece + 0x68, 1).is_some_and(|v| v[0] == action[8])
        {
            if let Some(item) = data(piece + 0x50, 4)
                .and_then(|v| v.try_into().ok())
                .map(u32::from_le_bytes)
            {
                if let Some(encoded) = piece_look(class[0], slot as usize, item) {
                    let owned: unsafe extern "system" fn(u64, u32) -> bool =
                        std::mem::transmute(r.local.base + owned_item.rva as usize);
                    if owned(collection, item) {
                        let choice = (u64::from(encoded) << 32) | 1;
                        action[..8].copy_from_slice(&choice.to_le_bytes());
                        let label = b"This piece (spec)";
                        add(
                            c.rbp + 0x2a8,
                            c.rbx,
                            label.as_ptr(),
                            label.len(),
                            13.,
                            current == choice,
                            action.as_ptr(),
                            false,
                        );
                    }
                }
            }
        }
    }
    action[..8].copy_from_slice(&manual_worn());
    let _ = r.local.write((c.rbp + 0xb0) as usize, &action);
    let _ = r.local.write(
        (c.rsp_entry + 0x28) as usize,
        &[u8::from(current.to_le_bytes() == manual_worn())],
    );
}
static PREVIEW: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(preview_entry, 0, preview, PREVIEW);
unsafe extern "system" fn preview(c: &mut crate::stub::Context) {
    let Some(r) = RUNTIME.get() else { return };
    let access = io::Access {
        local: &r.local,
        adapter: r.adapter,
    };
    let result = r.adapter.preview_weapons(
        &access,
        r.local.base as u64,
        c.rdi,
        c.rsp_entry,
        c.rbp,
        c.rbx,
        c.rax,
    );
    #[cfg(feature = "test-hooks")]
    if std::env::var_os("AZSX_TEST_WEAPONS").is_some() {
        r.log.info(&format!("Weapon preview {result:?}"))
    }
    if let Err(e) = result {
        r.log.warn(&format!("Weapon preview rejected: {e}"))
    }
}
// Bevy's entity handle includes its generation. Remember only cosmetic
// main hands seen by this client's equipment resolver, including remote heroes.
static GRIPS: std::sync::Mutex<Vec<(u64, bool)>> = std::sync::Mutex::new(Vec::new());
static EQUIPMENT: AtomicUsize = AtomicUsize::new(0);
static ACTION: AtomicUsize = AtomicUsize::new(0);
crate::context_stub_rw!(equipment_entry, 0, equipment, EQUIPMENT);
crate::context_stub_rw!(action_entry, 0, action, ACTION, write_rdi);
pub(super) fn reset() {
    GRIPS.lock().unwrap_or_else(|e| e.into_inner()).clear()
}
unsafe extern "system" fn equipment(c: &mut crate::stub::Context) {
    use azsx_game::current::WEAPON_EQUIPMENT_REGISTERS;
    const { assert!(matches!(WEAPON_EQUIPMENT_REGISTERS, [5, 13] | [12, 3])) };
    let (slot, entity) = match WEAPON_EQUIPMENT_REGISTERS {
        [5, 13] => (c.rbp as u8, c.r13),
        [12, 3] => (c.r12 as u8, c.rbx),
        _ => unreachable!(),
    };
    if slot != 15 {
        return;
    }
    let Some(r) = RUNTIME.get() else { return };
    let grip = if c.rax & 1 != 0 {
        r.adapter.weapon_two_handed(c.rdx as u32)
    } else {
        None
    };
    let mut grips = GRIPS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(index) = grips
        .iter()
        .position(|(cached, _)| *cached as u32 == entity as u32)
    {
        if grip.is_some_and(|two| grips[index] == (entity, two)) {
            return;
        }
        grips.remove(index);
    }
    if let Some(two) = grip {
        if grips.len() >= azsx_game::official::NATIVE_PLAYERS {
            grips.remove(0);
        }
        grips.push((entity, two));
    }
}
unsafe extern "system" fn action(c: &mut crate::stub::Context) {
    let before = c.rdi as u16;
    if c.rdx as u8 != 1 || !matches!(before, 17..=19 | 57 | 58) {
        return;
    }
    let Some(r) = RUNTIME.get() else { return };
    let read = |p: u64| {
        r.local
            .read(p as usize, 8)
            .ok()
            .and_then(|v| v.try_into().ok())
            .map(u64::from_le_bytes)
    };
    let Some(entity) = read(c.rbp + 0x10).and_then(read) else {
        return;
    };
    let grip = GRIPS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(key, _)| *key == entity)
        .map(|(_, two)| *two);
    let Some(two) = grip else { return };
    let Some(after) = r.adapter.weapon_melee_animation(two, before) else {
        return;
    };
    c.rdi = u64::from(after);
    #[cfg(feature = "test-hooks")]
    if std::env::var_os("AZSX_TEST_WEAPONS").is_some() {
        r.log
            .info(&format!("Weapon melee animation {before} -> {after}"))
    }
}
pub(super) fn install(r: &Runtime) -> Result<(), String> {
    #[cfg(feature = "test-hooks")]
    if std::env::var_os("AZSX_TEST_WEAPONS").is_some() {
        catalog_probe::install(r)?;
    }
    let entries = r.adapter.weapon_entries();
    for e in entries.iter().chain(r.adapter.weapon_button_entries().iter()) {
        if r.local.read(r.local.base + e.rva as usize, e.bytes.len())? != e.bytes {
            return Err("Native weapon boundary changed".into());
        }
    }
    crate::install::install(&[
        crate::install::HookSpec {
            name: "weapon-render-equipment",
            target: r.local.base + entries[6].rva as usize,
            detour: equipment_entry as *const () as usize,
            trampoline: &EQUIPMENT,
        },
        crate::install::HookSpec {
            name: "weapon-render-action",
            target: r.local.base + entries[7].rva as usize,
            detour: action_entry as *const () as usize,
            trampoline: &ACTION,
        },
        crate::install::HookSpec {
            name: "preview-visible-weapons",
            target: r.local.base + entries[5].rva as usize,
            detour: preview_entry as *const () as usize,
            trampoline: &PREVIEW,
        },
        crate::install::HookSpec {
            name: "run-visible-weapons",
            target: r.local.base + entries[0].rva as usize,
            detour: look as *const () as usize,
            trampoline: &LOOK,
        },
        crate::install::HookSpec {
            name: "weapon-visual-template",
            target: r.local.base + entries[1].rva as usize,
            detour: item as *const () as usize,
            trampoline: &ITEM,
        },
        crate::install::HookSpec {
            name: "collection-look-name",
            target: r.local.base + entries[9].rva as usize,
            detour: showing_entry as *const () as usize,
            trampoline: &SHOWING,
        },
        crate::install::HookSpec {
            name: "collection-look-marker",
            target: r.local.base + entries[8].rva as usize,
            detour: mark_look as *const () as usize,
            trampoline: &MARK_LOOK,
        },
        crate::install::HookSpec {
            name: "weapon-look-button",
            target: r.local.base + entries[2].rva as usize,
            detour: button_entry as *const () as usize,
            trampoline: &BUTTON,
        },
        crate::install::HookSpec {
            name: "weapon-look-validation",
            target: r.local.base + entries[3].rva as usize,
            detour: validate as *const () as usize,
            trampoline: &VALIDATE,
        },
        crate::install::HookSpec {
            name: "weapon-look-resolution",
            target: r.local.base + entries[4].rva as usize,
            detour: resolve as *const () as usize,
            trampoline: &RESOLVE,
        },
    ])
}

#[cfg(feature = "test-hooks")]
mod catalog_probe {
    use super::*;
    static CATALOG: AtomicUsize = AtomicUsize::new(0);
    crate::context_stub_rw!(catalog, 0, inspect, CATALOG);

    // At the successful return from the native ItemDisplayInfo loader, the
    // control bytes and the 360-byte buckets belong to the live game catalog.
    unsafe extern "system" fn inspect(c: &mut crate::stub::Context) {
        let Some(r) = RUNTIME.get() else { return };
        let Ok(h) = r.local.read((c.rbp + r.adapter.weapon_catalog_probe().1) as usize, 32) else {
            return;
        };
        let value = |i| usize::from_le_bytes(h[i..i + 8].try_into().unwrap());
        let (ctrl, mask) = (value(0), value(8));
        if mask >= 65536 {
            return;
        }
        let count = mask + 1;
        let Some(start) = ctrl.checked_sub(count * 360) else {
            return;
        };
        let Ok(flags) = r.local.read(ctrl, count) else {
            return;
        };
        let Ok(rows) = r.local.read(start, count * 360) else {
            return;
        };
        let mut displays = Vec::new();
        for (i, flag) in flags.into_iter().enumerate() {
            if flag >= 128 {
                continue;
            }
            let at = (mask - i) * 360;
            let display = u32::from_le_bytes(rows[at..at + 4].try_into().unwrap());
            if display <= 65535 && r.adapter.weapon_template(0x6a5a0000 + display).is_some() {
                displays.push(display);
            }
        }
        displays.sort_unstable();
        r.log.info(&format!(
            "Weapon display catalog ({}) {displays:?}",
            displays.len()
        ));
    }
    pub(super) fn install(r: &Runtime) -> Result<(), String> {
        let (entry, _) = r.adapter.weapon_catalog_probe();
        let target = r.local.base + entry.rva as usize;
        if r.local.read(target, entry.bytes.len())? != entry.bytes {
            return Err("Weapon catalog probe boundary changed".into());
        }
        crate::install::install(&[crate::install::HookSpec {
            name: "weapon-display-catalog-probe",
            target,
            detour: catalog as *const () as usize,
            trampoline: &CATALOG,
        }])
    }
}

#[cfg(test)]
mod melee_forwarding_tests {
    use super::*;
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    crate::context_stub_rw!(entry, 0, choose, NEXT, write_rdi);

    unsafe extern "system" fn choose(c: &mut crate::stub::Context) {
        assert_eq!(c.rdi, 57);
        c.rdi = 58;
    }

    #[unsafe(naked)]
    unsafe extern "C" fn native_consumer() -> u64 {
        core::arch::naked_asm!("mov eax, edi", "ret");
    }

    #[unsafe(naked)]
    unsafe extern "C" fn swing() -> u64 {
        core::arch::naked_asm!(
            "push rdi", "sub rsp, 32", "mov edi, 57", "call {}", "add rsp, 32",
            "pop rdi", "ret", sym entry
        );
    }

    #[test]
    fn two_hand_special_reaches_native_consumer() {
        NEXT.store(native_consumer as *const () as usize, Ordering::Release);
        assert_eq!(unsafe { swing() }, 58);
    }
}
