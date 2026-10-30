//! Retained native Bevy decoration; no lobby actions or replacement roots.
use super::Adapter;
use crate::{official::Entry, memory::Mem, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Clone, Copy)]
pub struct MenuBindings {
    base: usize,
    adapter: Adapter,
}
pub struct MenuAssets {
    image: [u8; 24],
    color: [u8; 20],
}
impl Adapter {
    pub fn menu_bindings(self, base: usize) -> MenuBindings {
        MenuBindings { base, adapter:self }
    }
    /// Extend the native menu cache key only after its phase/modal guards.
    /// RAX is dead here; the selected miss continuation uses the original
    /// owned-string cleanup, root retirement and normal constructor.
    pub fn menu_cache_entries(self) -> [Entry; 2] {
        [
            Entry {
                rva: self.rva(0x10d95b),
                bytes: self.expected(0x10d95b, &[
                    0x44, 0x38, 0x73, 0x38, 0x75, 0x5b, 0x40, 0x38, 0x7b, 0x39, 0x75, 0x55, 0x44,
                    0x38, 0x6b, 0x3a,
                ]),
            },
            Entry {
                rva: self.rva(0x10d9e0),
                bytes: self.expected(0x10d9e0, &[0x8b, 0xb5, 0x38, 2, 0, 0, 0x48, 0x8b, 0x9d, 0x78, 2, 0, 0]),
            },
        ]
    }
    pub fn menu_entries(self) -> [Entry; 7] {
        [
            Entry {
                rva: self.rva(0x322425),
                bytes: self.expected(0x322425, &[
                    0x48, 0x8d, 0x8d, 0x10, 5, 0, 0, 0x48, 0x8b, 0x95, 0x58, 8, 0, 0,
                ]),
            },
            Entry {
                rva: self.rva(0x1003f0),
                bytes: self.expected(0x1003f0, &[
                    0x55, 0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x81, 0xec, 0xb0, 5, 0, 0,
                ]),
            },
            Entry {
                rva: self.rva(0x100a20),
                bytes: self.expected(0x100a20, &[
                    0x55, 0x41, 0x56, 0x56, 0x57, 0x53, 0x48, 0x81, 0xec, 0x40, 3, 0, 0,
                ]),
            },
            Entry {
                rva: self.rva(0xfffa0),
                bytes: self.expected(0xfffa0, &[0x48, 0x89, 0xc8, 0x44, 0x0f, 0xb6, 0x82, 0x8c, 0, 0, 0]),
            },
            Entry {
                rva: self.rva(0x2b57b0),
                bytes: self.expected(0x2b57b0, &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x81, 0xec, 0xe8, 1, 0, 0,
                ]),
            },
            Entry {
                rva: self.rva(0x219dd0),
                bytes: self.expected(0x219dd0, &[
                    0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48,
                    0x81, 0xec, 0xc8, 0, 0, 0,
                ]),
            },
            // Disabled icon cards skip the native action/hover attachment branch.
            Entry {
                rva: self.rva(0x325396),
                bytes: self.expected(0x325396, &[0x80, 0x3b, 0, 0x0f, 0x85, 0x52, 1, 0, 0]),
            },
        ]
    }
}
unsafe fn retain(handle: &[u8; 24]) -> bool {
    if handle[0] == 1 {
        return true;
    }
    if handle[0] != 0 {
        return false;
    }
    let pointer = usize::from_le_bytes(handle[8..16].try_into().unwrap());
    if pointer < 0x10000 {
        return false;
    }
    let count = &*(pointer as *const AtomicUsize);
    let old = count.load(Ordering::Relaxed);
    if old == 0 || old >= 1usize << 32 {
        return false;
    }
    count.fetch_add(1, Ordering::Relaxed);
    true
}
impl MenuBindings {
    pub fn members(self, mem: &dyn Mem, resource: u64) -> Result<Vec<(u64, u8)>> {
        let q = |address| -> Result<u64> {
            Ok(u64::from_le_bytes(
                mem.read(address, 8)?
                    .try_into()
                    .map_err(|_| "Truncated native lobby")?,
            ))
        };
        let state = q(resource)?;
        if state < 0x10000 {
            return Err("Invalid native lobby resource".into());
        }
        let shift = 0x10;
        let cap = q(state + 0x78+shift)?;
        let pointer = q(state + 0x80+shift)?;
        let len = q(state + 0x88+shift)?;
        if len > super::NATIVE_PLAYERS as u64 || cap < len || cap > 64 {
            return Err("Invalid native lobby members".into());
        }
        let mut rows = vec![];
        for i in 0..len {
            let row = pointer
                .checked_add(i * 0x70)
                .ok_or("Native lobby overflow")?;
            let id = q(row + 0x60)?;
            let class = mem.read(row + 0x6f, 1)?[0];
            if id == 0
                || rows.iter().any(|(old, _)| *old == id)
                || ![1, 2, 3, 4, 5, 7, 8, 9, 11].contains(&class)
            {
                return Err("Invalid native lobby identity/class".into());
            }
            rows.push((id, class));
        }
        Ok(rows)
    }
    /// Replay native status formatting with a callback-local Res change marker.
    /// Native data, ECS ticks and the caller's resource wrapper remain unchanged.
    pub unsafe fn refresh_status(
        self,
        mem: &dyn Mem,
        original: usize,
        args: [u64; 4],
    ) -> Result<usize> {
        let mut wrapper: [u8; 32] = mem
            .read(args[1], 32)?
            .try_into()
            .map_err(|_| "Truncated native menu status resource")?;
        let current = u32::from_le_bytes(wrapper[28..32].try_into().unwrap());
        let changed = current;
        wrapper[16..24].copy_from_slice(&((&changed as *const u32) as u64).to_le_bytes());
        wrapper[24..28].copy_from_slice(&current.wrapping_sub(1).to_le_bytes());
        type Status = unsafe extern "system" fn(u64, u64, u64, u64) -> usize;
        let call: Status = std::mem::transmute(original);
        Ok(call(args[0], wrapper.as_ptr() as u64, args[2], args[3]))
    }
    pub fn size_card(
        self,
        mem: &dyn crate::official::MutableMemory,
        caller: u64,
        bundle: u64,
    ) -> Result<()> {
        if caller != self.adapter.rva(0x100c42) {
            return Ok(());
        }
        let node = bundle.checked_add(8).ok_or("Native class node overflow")?;
        for offset in [0x88, 0x90] {
            mem.write(
                node + offset,
                &[1u32.to_le_bytes(), 24f32.to_le_bytes()].concat(),
            )?;
        }
        Ok(())
    }
    /// Called only at the verified main class-grid point after the atlas load.
    pub unsafe fn capture(self, mem: &dyn Mem, frame: u64) -> Result<Option<MenuAssets>> {
        let (atlas, color) = (
            crate::current::MENU_ATLAS_FRAME,
            crate::current::MENU_COLOR_FRAME,
        );
        let image: [u8; 24] = mem
            .read(frame.checked_add(atlas).ok_or("Menu frame overflow")?, 24)?
            .try_into()
            .map_err(|_| "Truncated native class atlas")?;
        let color = mem
            .read(frame.checked_add(color).ok_or("Menu color overflow")?, 20)?
            .try_into()
            .map_err(|_| "Truncated native class border")?;
        if !retain(&image) {
            return Ok(None);
        }
        Ok(Some(MenuAssets { image, color }))
    }
    pub unsafe fn draw(
        self,
        mem: &dyn Mem,
        parent: u64,
        font: usize,
        assets: &MenuAssets,
        classes: &[u8],
    ) -> usize {
        if classes.is_empty() || classes.len() > super::NATIVE_PLAYERS {
            return 0;
        }
        let Ok(bytes) = mem.read(parent, 48) else {
            return 0;
        };
        if bytes.len() != 48 {
            return 0;
        }
        // Borrow the live native ChildSpawner. Its command cursor is mutable;
        // copying the six words would discard mutations when this call returns.
        let mut count = 0;
        type Uv = unsafe extern "system" fn(*mut f32, *const u8) -> usize;
        type Card = unsafe extern "system" fn(
            *mut usize,
            usize,
            *const u8,
            *const f32,
            f32,
            *const u8,
            usize,
            u8,
            u8,
            *const u8,
            *const u8,
        ) -> usize;
        let uv: Uv = std::mem::transmute(self.base + self.adapter.rva(0xfffa0) as usize);
        let card: Card = std::mem::transmute(self.base + self.adapter.rva(0x100a20) as usize);
        for class in classes {
            if ![1, 2, 3, 4, 5, 7, 8, 9, 11].contains(class) || !retain(&assets.image) {
                continue;
            }
            let mut definition = [0u8; 0x90];
            definition[0x8c] = *class;
            let mut rect = [0f32; 4];
            uv(rect.as_mut_ptr(), definition.as_ptr());
            let mut action = [0u8; 80];
            action[0] = 0;
            action[1] = *class; // valid native class variant, NEVER attached: disabled=true
            card(
                parent as *mut usize,
                font,
                assets.image.as_ptr(),
                rect.as_ptr(),
                24.0,
                b"".as_ptr(),
                0,
                0,
                1,
                assets.color.as_ptr(),
                action.as_ptr(),
            );
            count += 1;
        }
        count
    }
}

#[cfg(test)]mod capture_layout_tests{
 use super::*;
 struct Frame{atlas:u64,color:u64}
 impl Mem for Frame{fn read(&self,a:u64,n:usize)->Result<Vec<u8>>{if(a,n)==(self.atlas,24){let mut b=vec![0;24];b[0]=1;b[16]=73;Ok(b)}else if(a,n)==(self.color,20){Ok(vec![9;20])}else{Err("Unexpected native frame access".into())}}}
 #[test]fn selected_revision_uses_its_atlas_and_color_slots(){let a=Adapter::select(crate::current::SHA256).unwrap();let m=Frame{atlas:0x10000+crate::current::MENU_ATLAS_FRAME,color:0x10000+crate::current::MENU_COLOR_FRAME};let assets=unsafe{a.menu_bindings(0).capture(&m,0x10000)}.unwrap().unwrap();assert_eq!(assets.image[16],73);assert_eq!(assets.color,[9;20]);assert!(Adapter::select(crate::official121::SHA256).is_err());}
}
