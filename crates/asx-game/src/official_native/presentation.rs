//! Native script mailbox boundary for the exact reviewed official client.
use super::Adapter;
use crate::official::Entry;
impl Adapter {
    pub fn ui_mailbox_writer(self) -> Option<Entry> {
        if self.version()!="0.115.0" {return None}
        Some(Entry {
            rva: self.rva(0xf0d870),
                bytes: self.expected(0xf0d870, &[
                0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48, 0x81,
                0xec, 0x18, 1, 0, 0, 0x48, 0x8d, 0xac, 0x24, 0x80, 0, 0, 0,
            ]),
        })
    }
}

impl Adapter {
    pub fn party_addon_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x5821710),
                bytes: self.expected(0x5821710, &[
                0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48, 0x81,
                0xec, 0xf8, 1, 0, 0, 0x48, 0x8d, 0xac, 0x24, 0x80, 0, 0, 0,
            ]),
        }
    }
    pub fn party_build(
        self,
        value: &serde_json::Value,
    ) -> crate::Result<crate::official::PartyBuild> {
        use crate::official::{PartyBuild, PartyIcon};
        let text = |v: &serde_json::Value,
                    key: &str,
                    max: usize,
                    multiline: bool|
         -> crate::Result<String> {
            let s = v[key].as_str().ok_or("Missing native build text")?;
            if s.len() > max
                || s.chars()
                    .any(|c| c.is_control() && !(multiline && c == '\n'))
            {
                return Err("Invalid native build text".into());
            }
            Ok(s.into())
        };
        let decode = |key: &str, spell: bool| -> crate::Result<Vec<PartyIcon>> {
            let rows = value[key].as_array().ok_or("Missing native build rows")?;
            if rows.len() > 16 {
                return Err("Native build row limit".into());
            }
            rows.iter()
                .enumerate()
                .map(|(index, row)| {
                    let rank = row["rank"]
                        .as_u64()
                        .filter(|n| *n >= 1 && *n <= 255)
                        .ok_or("Invalid native build rank")? as u32;
                    let id = if spell {
                        row["spell"]
                            .as_u64()
                            .filter(|n| *n > 0 && *n <= u32::MAX as u64)
                            .ok_or("Invalid native spell ID")? as u32
                    } else {
                        index as u32 + 1
                    };
                    let icon = text(row, "icon", 160, false)?;
                    if icon.is_empty()
                        || !icon
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"_./\\".contains(&b))
                    {
                        return Err("Invalid native build icon".into());
                    }
                    Ok(PartyIcon {
                        id,
                        rank,
                        name: text(row, "name", 256, false)?,
                        icon,
                        spell_tip: None,
                        text: if spell {
                            String::new()
                        } else {
                            text(row, "text", 2048, true)?
                        },
                    })
                })
                .collect()
        };
        Ok(PartyBuild {
            spells: decode("abilities", true)?,
            items: decode("jewelry", false)?,
        })
    }
    pub fn decorate_party_addon(
        self,
        mem: &dyn crate::official::MutableMemory,
        output: u64,
        wire: &str,
    ) -> crate::Result<()> {
        if wire.len() > 32768 || wire.is_empty() || !wire.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Invalid party presentation envelope".into());
        }
        // Native fixed array: 20 (&str, String) rows of 40 bytes. Only extend AZSRUN.
        let row = mem.read(output.checked_add(40).ok_or("Addon output overflow")?, 16)?;
        if row.len() != 16 {
            return Err("Truncated addon prefix".into());
        }
        let p = u64::from_le_bytes(row[..8].try_into().unwrap());
        let n = u64::from_le_bytes(row[8..].try_into().unwrap());
        if n != 6 || p < 0x10000 || mem.read(p, 6)? != b"AZSRUN" {
            return Err("Native addon prefix changed".into());
        }
        let header = output.checked_add(56).ok_or("Addon string overflow")?;
        let old = mem.read(header, 24)?;
        if old.len() != 24 {
            return Err("Truncated addon string".into());
        }
        let q = |at| u64::from_le_bytes(old[at..at + 8].try_into().unwrap());
        let (cap, ptr, len) = (q(0), q(8), q(16));
        if len > 4096 || len > cap || cap > 1_048_576 || (cap > 0 && ptr < 0x10000) {
            return Err("Invalid native addon extent".into());
        }
        let mut bytes = mem.read(ptr, len as usize)?;
        if bytes.len() != len as usize || bytes.windows(b";asx=".len()).any(|s| s == b";asx=") {
            return Err("Invalid original addon fields".into());
        }
        bytes.extend_from_slice(b";asx=");
        bytes.extend_from_slice(wire.as_bytes());
        self.replace_owned_text(mem, header, &bytes)
    }
}

impl Adapter {
    pub fn spell_presentation_entry(self) -> Entry {
        Entry {
            rva: self.rva(0x58629b0),
                bytes: self.expected(0x58629b0, &[
                0x55, 0x41, 0x57, 0x41, 0x56, 0x41, 0x55, 0x41, 0x54, 0x56, 0x57, 0x53, 0x48, 0x81,
                0xec, 0x58, 4, 0, 0,
            ]),
        }
    }
    pub fn spell_presentation_owner(
        self,
        mem: &dyn crate::memory::Mem,
        ability: u64,
        stats: u64,
        kind: u8,
    ) -> crate::Result<Option<(u32, u32, u32)>> {
        if kind != 0 {
            return Ok(None);
        } // actual learned ability; previews/passives/auras have distinct tags
        let Some(game) = stats.checked_sub(self.game_field(0x490)) else {
            return Ok(None);
        };
        let Some(slot) = self.run_actor(mem, game)? else {
            return Ok(None);
        };
        let row = mem.read(ability, 12)?;
        if row.len() != 12 {
            return Err("Truncated native spell presentation owner".into());
        }
        let definition = u64::from_le_bytes(row[..8].try_into().unwrap());
        let rank = u32::from_le_bytes(row[8..].try_into().unwrap());
        if definition < 0x10000 || !(1..=255).contains(&rank) {
            return Err("Invalid native presentation ability".into());
        }
        let id = mem.read(
            definition
                .checked_add(0x40)
                .ok_or("Spell definition overflow")?,
            4,
        )?;
        let id = u32::from_le_bytes(
            id.try_into()
                .map_err(|_| "Truncated native presentation spell ID")?,
        );
        if id == 0 {
            return Err("Invalid native presentation spell ID".into());
        }
        Ok(Some((slot, id, rank)))
    }
    pub fn spell_presentation_strings(
        self,
        mem: &dyn crate::memory::Mem,
        output: u64,
    ) -> crate::Result<Vec<(u32, String)>> {
        // Copy owned strings from the native 0xb0 presentation row. Never retain or free them.
        let row = mem.read(output, 0xb0)?;
        if row.len() != 0xb0 {
            return Err("Truncated native spell presentation".into());
        }
        let mut strings = vec![];
        for offset in [0usize, 24, 48, 72, 96, 120, 144] {
            let q = |i| u64::from_le_bytes(row[i..i + 8].try_into().unwrap());
            let (cap, ptr, len) = (q(offset), q(offset + 8), q(offset + 16));
            if len == 0
                || len > 8192
                || cap < len
                || cap > 1_048_576
                || ptr < 0x10000
                || ptr.checked_add(len).is_none()
            {
                continue;
            }
            let Ok(bytes) = mem.read(ptr, len as usize) else {
                continue;
            };
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            if text.len() != len as usize || text.chars().any(|c| c.is_control() && c != '\n') {
                continue;
            }
            strings.push((offset as u32, text));
        }
        if !strings.iter().any(|(offset, _)| *offset == 0) {
            return Err("Native spell description unavailable".into());
        }
        Ok(strings)
    }
}

impl Adapter {
 pub fn diagnostic_writer_entry(self)->Entry{
  let(rva,bytes)=crate::current::DIAGNOSTIC_WRITER;Entry{rva,bytes}
 }
}
