//! Retained loot ownership codec and native reward UI helpers; no loot interception.
use super::*;
use crate::official::{Entry, MutableMemory};
fn q(m: &dyn Mem, a: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(
        m.read(a, 8)?
            .try_into()
            .map_err(|_| "Truncated loot pointer")?,
    ))
}
impl Adapter {
    pub fn loot_victory(self,m:&dyn Mem,game:u64)->Result<bool>{self.blessing_fixture_victory(m,game)}
    pub fn loot_actor_gold(self,m:&dyn Mem,game:u64,slot:u32,guid:u64)->Result<(f32,u8)>{
      let row=self.support_row(m,game,slot)?;
      if guid==0||m.read(row+self.row_field(row==game,0x478),8)?!=guid.to_le_bytes(){return Err("Loot gold recipient changed".into())}
      let multiplier=f32::from_le_bytes(m.read(row+self.row_field(row==game,0x4c0),4)?.try_into().map_err(|_|"Truncated loot gold multiplier")?);
      if !multiplier.is_finite()||multiplier<0.0{return Err("Invalid loot gold multiplier".into())}
      Ok((multiplier,m.read(row+self.row_field(row==game,0x32d),1)?[0]))
    }

    pub fn loot_entries(self) -> Vec<Entry> { vec![] }
    pub fn loot_send_entry(self, _peer: bool) -> Option<Entry> { None }
    /// Copy only the native-issued item IDs, after actor restoration. Source peer
    /// and recipient actor are distinct native identities; validate their binding.
    pub fn loot_offer(self, m: &dyn Mem, frame: u64, peer: u64) -> Result<(u64, u32, Vec<u32>)> {
        let server = q(m, frame + 0xc80)?;
        let slot = q(m, frame + 0xc48)?;
        if m.read(server + 0xcb8, 4)? != 3u32.to_le_bytes()
            || peer >= q(m, server + 0xcf8)?
            || slot == 0
            || slot >= NATIVE_PLAYERS as u64
        {
            return Err("Invalid native loot recipient".into());
        }
        let row = q(m, server + 0xcf0)?
            .checked_add(peer.checked_mul(0x480).ok_or("Loot peer overflow")?)
            .ok_or("Loot peer overflow")?;
        if m.read(row, 4)? != 1u32.to_le_bytes() || q(m, row + 8)? != slot {
            return Err("Native loot peer changed".into());
        }
        let ptr = q(m, frame + 0x5b0)?;
        let len = q(m, frame + 0x5b8)?;
        let cap = q(m, frame + 0x5a8)?;
        if len == 0 || len > 64 || len > cap || ptr < 0x10000 {
            return Err("Invalid native loot extent".into());
        }
        let items = m
            .read(ptr, len as usize * 4)?
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        Ok((server, slot as u32, items))
    }
    pub fn loot_known(self, m: &dyn Mem, frame: u64) -> Result<bool> {
        match m.read(frame + 0x158, 1)?.as_slice() {
            [0] => Ok(true),
            [1] => Ok(false),
            _ => Err("Invalid native loot collection flag".into()),
        }
    }
    pub fn loot_packet(self, m: &dyn Mem, packet: u64) -> Result<Option<(u16, Vec<u8>)>> {
        if q(m, packet)? != 0x8000000000000006 {
            return Ok(None);
        }
        let op = u16::from_le_bytes(
            m.read(packet + 32, 2)?
                .try_into()
                .map_err(|_| "Truncated loot opcode")?,
        );
        if !(0x7a51..=0x7a53).contains(&op) {
            return Ok(None);
        }
        Ok(Some((
            op,
            self.loot_bytes(m, q(m, packet + 16)?, q(m, packet + 24)?)?,
        )))
    }
    pub fn loot_bytes(self, m: &dyn Mem, ptr: u64, len: u64) -> Result<Vec<u8>> {
        if !(1..=8192).contains(&len) || ptr < 0x10000 {
            return Err("Invalid loot message extent".into());
        }
        m.read(ptr, len as usize)
    }
    /// Change only the native event's type (same byte length). Original owned JSON
    /// allocation/length and collection persistence remain entirely native.
    pub fn loot_hide_drop(
        self,
        m: &dyn MutableMemory,
        ptr: u64,
        len: u64,
    ) -> Result<serde_json::Value> {
        let bytes = self.loot_bytes(m, ptr, len)?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| "Invalid native drop JSON")?;
        if value["type"] != "drop" || value["new"] != false {
            return Err("Not a native duplicate drop".into());
        }
        let at = bytes
            .windows(6)
            .position(|b| b == b"\"drop\"")
            .ok_or("Missing native drop type")?;
        m.write(ptr + at as u64 + 1, b"noop")?;
        Ok(value)
    }
    pub fn loot_ui_sink(self, m: &dyn Mem, server: u64) -> Result<(u64, u64)> {
        let ctx = q(m, server + 0x17d0)?;
        let vt = q(m, server + 0x17d8)?;
        let call = q(m, vt + 24)?;
        if ctx < 0x10000 || call < 0x10000 {
            return Err("Missing native loot UI sink".into());
        }
        Ok((ctx, call))
    }
}

impl Adapter {
 /// Diagnostic-only caller. Grow and retain a real native Vec<u32>; never fake a
 /// collection result or an ASX credit. No gameplay lock may span native reserve.
 pub unsafe fn loot_fixture_append(self,_m:&dyn MutableMemory,_game:u64,_base:u64,_slot:u32,_item:u32)->Result<bool>{Err("Duplicate-loot fixture is retired for 0.125.0".into())}
}

impl Adapter {
 pub fn loot_fixture_entry(self)->Option<Entry>{None}
 /// This owned Vec has already been captured from the native guest context.
 /// Reserve before the native length branch, then refresh its cached cap/ptr.
 pub unsafe fn loot_fixture_dispatch(self,_m:&dyn MutableMemory,_frame:u64,_base:u64,_peer:u64,_item:u32)->Result<u64>{Err("Duplicate-loot fixture is retired for 0.125.0".into())}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::BTreeMap};
    #[derive(Default)]
    struct Memory(RefCell<BTreeMap<u64, u8>>);
    impl Mem for Memory {
        fn read(&self, a: u64, n: usize) -> Result<Vec<u8>> {
            (0..n)
                .map(|i| {
                    self.0
                        .borrow()
                        .get(&(a + i as u64))
                        .copied()
                        .ok_or("Unmapped loot read".into())
                })
                .collect()
        }
    }
    impl MutableMemory for Memory {
        fn write(&self, a: u64, b: &[u8]) -> Result<()> {
            for (i, v) in b.iter().enumerate() {
                self.0.borrow_mut().insert(a + i as u64, *v);
            }
            Ok(())
        }
        fn allocate_native(&self, _: &[u8]) -> Result<u64> {
            Err("Unexpected allocation".into())
        }
        fn free_native(&self, _: u64, _: usize) -> Result<()> {
            Err("Unexpected free".into())
        }
    }
    #[test]
    fn only_reviewed_build_and_bounded_packets() {
        let a = Adapter::select(crate::official125::SHA256).unwrap();
        let m = Memory::default();
        assert!(a.loot_entries().is_empty());
        m.write(0x10000, &0x8000000000000006u64.to_le_bytes())
            .unwrap();
        m.write(0x10020, &0x7a51u16.to_le_bytes()).unwrap();
        m.write(0x10010, &0x20000u64.to_le_bytes()).unwrap();
        m.write(0x10018, &2u64.to_le_bytes()).unwrap();
        m.write(0x20000, b"{}").unwrap();
        assert_eq!(
            a.loot_packet(&m, 0x10000).unwrap(),
            Some((0x7a51, b"{}".to_vec()))
        );
        m.write(0x10018, &8193u64.to_le_bytes()).unwrap();
        assert!(a.loot_packet(&m, 0x10000).is_err());
        m.write(0x10020, &99u16.to_le_bytes()).unwrap();
        assert!(a.loot_packet(&m, 0x10000).unwrap().is_none());
    }
    #[test]
    fn inactive_duplicate_ui_codec_keeps_its_ownership_guards() {
        let a = Adapter::select(crate::official125::SHA256).unwrap();
        assert!(a.loot_entries().is_empty());
        let m = Memory::default();
        let original = br#"{"type":"drop","name":"Sword","new":false,"icon":"Sword_01"}"#;
        m.write(0x20000, original).unwrap();
        let v = a
            .loot_hide_drop(&m, 0x20000, original.len() as u64)
            .unwrap();
        assert_eq!(v["type"], "drop");
        assert_eq!(v["icon"], "Sword_01");
        let after = m.read(0x20000, original.len()).unwrap();
        let after: serde_json::Value = serde_json::from_slice(&after).unwrap();
        assert_eq!(after["type"], "noop");
        assert_eq!(after["name"], "Sword");
        let new = br#"{"type":"drop","new":true}"#;
        m.write(0x20000, new).unwrap();
        assert!(a.loot_hide_drop(&m, 0x20000, new.len() as u64).is_err());
        assert_eq!(m.read(0x20000, new.len()).unwrap(), new);
    }
}
