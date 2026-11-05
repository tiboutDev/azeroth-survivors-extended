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
            // Native HUD slot metadata, never names or icon guesses. Preserve
            // order within a category and original cosmetic row identities.
            let mut ordered: Vec<_> = rows.iter().enumerate().collect();
            if !spell {
                ordered.sort_by_key(|(_, row)| match row["slot"].as_str() {
                    Some("amulet") => 0,
                    Some("ring") => 1,
                    Some("trinket") => 2,
                    _ => 3,
                });
            }
            ordered.into_iter()
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
                        category: if spell { None } else { match row["slot"].as_str() {
                            Some("amulet") => Some("amulet"), Some("ring") => Some("ring"),
                            Some("trinket") => Some("trinket"), _ => None,
                        } },
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
        // AZSRUN is the second 40-byte (&str, String) row in the native output.
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
        if bytes.len() != len as usize || bytes.windows(b";azsx=".len()).any(|s| s == b";azsx=") {
            return Err("Invalid original addon fields".into());
        }
        bytes.extend_from_slice(b";azsx=");
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
        let rank_at = crate::current::SPELL_PRESENTATION_RANK_OFFSET as usize;
        let row = mem.read(ability, rank_at + 4)?;
        if row.len() != rank_at + 4 {
            return Err("Truncated native spell presentation owner".into());
        }
        let definition = u64::from_le_bytes(row[..8].try_into().unwrap());
        let rank = u32::from_le_bytes(row[rank_at..rank_at + 4].try_into().unwrap());
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

impl Adapter {
    pub fn addon_api_reader_entry(self) -> Entry {
        Entry { rva: crate::current::ADDON_API_READER, bytes: crate::current::ADDON_API_READER_BYTES }
    }
    /// Extend the native WorldStateFrame API script using the native allocator.
    /// No addon file, Lua evaluator or arbitrary script/file interception.
    pub fn extend_addon_api(self, mem: &dyn crate::official::MutableMemory, output: u64, extension: &[u8]) -> crate::Result<()> {
        if extension.is_empty() || extension.len()>98304 {return Err("Addon API extension exceeds limit".into());}
        let header = mem.read(output,24)?;
        if header.len()!=24 { return Err("Truncated native API string".into()); }
        let capacity=u64::from_le_bytes(header[0..8].try_into().unwrap());
        let pointer=u64::from_le_bytes(header[8..16].try_into().unwrap());
        let length=u64::from_le_bytes(header[16..24].try_into().unwrap());
        if !(1..=131072).contains(&length) || length>capacity || capacity>1_048_576 || pointer<0x10000 {
            return Err("Invalid native API string".into());
        }
        let mut script=mem.read(pointer,length as usize)?;
        if script.len()!=length as usize || !script.windows(11).any(|s|s==b"AZSApiFrame") {
            return Err("Native API script identity changed".into());
        }
        script.extend_from_slice(b"\n"); script.extend_from_slice(extension);
        self.replace_owned_text_bounded(mem,output,&script,262144)
    }
}

impl Adapter {
    pub fn lobby_submit_entry(self) -> Entry {
        Entry { rva: crate::current::LOBBY_SUBMIT, bytes: crate::current::LOBBY_SUBMIT_BYTES }
    }
    pub fn prefix_lobby_name(self, mem: &dyn crate::official::MutableMemory, resource: u64) -> crate::Result<()> {
        let at=resource.checked_add(crate::current::LOBBY_NAME).ok_or("Invalid lobby resource")?;
        let h=mem.read(at,24)?;
        if h.len()!=24 {return Err("Truncated lobby name".into());}
        let cap=u64::from_le_bytes(h[..8].try_into().unwrap());
        let ptr=u64::from_le_bytes(h[8..16].try_into().unwrap());
        let len=u64::from_le_bytes(h[16..].try_into().unwrap());
        if len>128 || cap<len || cap>1_048_576 || (len>0&&ptr<0x10000) {return Err("Invalid lobby name".into());}
        let bytes=if len==0 {Vec::new()}else{mem.read(ptr,len as usize)?};
        let text=std::str::from_utf8(&bytes).map_err(|_|"Invalid UTF-8 lobby name")?;
        let name=azsx_gameplay::lobby::name(text);
        if name!=text {self.replace_owned_text(mem,at,name.as_bytes())?;}
        Ok(())
    }
}

// The Collection Log filters the u32 inventory at +0x30 and calls the same
// class/map/boss drop builder for each of the five encounters (0x28de0e).
impl Adapter {
    pub fn run_owned_items(self, mem: &dyn crate::memory::Mem, collection: u64) -> crate::Result<Vec<u32>> {
        let header = mem.read(collection + 0x30, 24)?;
        let word = |i| u64::from_le_bytes(header[i..i+8].try_into().unwrap());
        let count = word(16);
        if count == 0 { return Ok(Vec::new()) }
        if count > word(0) || count > 4096 || word(8) < 65536 { return Err("Invalid gear collection".into()) }
        Ok(mem.read(word(8), count as usize * 4)?.chunks_exact(4)
            .map(|v| u32::from_le_bytes(v.try_into().unwrap())).collect())
    }

    pub unsafe fn run_drop_lua(self, mem: &dyn crate::memory::Mem, base: usize, owned: &[u32]) -> crate::Result<String> {
        use std::collections::BTreeMap;
        use std::fmt::Write;
        let map_table = base as u64 + crate::current::GOBLIN_MAP_TABLE;
        // Validate the Armory call site because AZSX may detour the item resolver itself.
        for offset in [0x5bfd310,0x3c1ccf,0x21d0,0x5c4d7a0] {
            let bytes=self.expected(offset,&[]);
            if bytes.is_empty() || mem.read(base as u64+self.rva(offset),bytes.len())? != bytes {
                return Err(format!("Run drop boundary changed at {offset:x}"));
            }
        }
        let drops: unsafe extern "system" fn(*mut usize,u8,usize,usize) = std::mem::transmute(base+self.rva(0x5bfd310) as usize);
        let item: unsafe extern "system" fn(*mut usize,u32) = std::mem::transmute(base+self.rva(0x5bffe00) as usize);
        let free: unsafe extern "system" fn(usize,usize,usize) = std::mem::transmute(base+self.rva(0x21d0) as usize);
        let string = |ptr:u64,len:u64| -> crate::Result<String> {
            if ptr<65536 || len==0 || len>180 {return Err("Invalid run drop name".into())}
            let name=String::from_utf8(mem.read(ptr,len as usize)?).map_err(|_|"Invalid item name encoding")?;
            if name.chars().any(char::is_control){return Err("Invalid run drop text".into())}
            serde_json::to_string(&name).map_err(|e|e.to_string())
        };
        let slice = |at:u64| -> crate::Result<(u64,u64)> {
            let b=mem.read(at,16)?;
            Ok((u64::from_le_bytes(b[..8].try_into().unwrap()),u64::from_le_bytes(b[8..].try_into().unwrap())))
        };
        let mut names=BTreeMap::new();
        let mut lua=String::from("{");
        for zone in 0..crate::current::BLESSING_MAP_COUNT as usize {
            let at=map_table+zone as u64*240;
            let (key,len)=slice(at+8)?;
            let (bosses,count)=slice(at+136)?;
            if count!=5 || bosses<65536{return Err("Unknown run boss table".into())}
            write!(lua,"[{}]={{",string(key,len)?).unwrap();
            for index in 0..5 {
                let (ptr,len)=slice(bosses+index as u64*104)?;
                write!(lua,"{{name={},",string(ptr,len)?).unwrap();
                for class in [1,2,3,4,5,7,8,9,11] {
                    let mut v=[0usize;3];
                    drops(v.as_mut_ptr(),class,zone,index);
                    let entries=if v[2]<=v[0] && v[2]<=64 && v[2]>0 {mem.read(v[1] as u64,v[2]*8)}else if v[2]==0 {Ok(Vec::new())}else{Err("Invalid boss loot vector".into())};
                    if v[0]>0 {free(v[1],v[0]*8,4)}
                    write!(lua,"[{class}]={{").unwrap();
                    for row in entries?.chunks_exact(8) {
                        let id=u32::from_le_bytes(row[..4].try_into().unwrap());
                        if owned.contains(&id){continue}
                        if !names.contains_key(&id) {
                            let mut v=[0usize;18];
                            item(v.as_mut_ptr(),id);
                            if v[0]==usize::MAX {return Err("Unknown generated set item".into())}
                            let name=string(v[1] as u64,v[2] as u64);
                            if v[0]>0 {free(v[1],v[0],1)}
                            if v[3]>0 {free(v[4],v[3]*8,4)}
                            names.insert(id,name?);
                        }
                        write!(lua,"{},",names[&id]).unwrap();
                    }
                    lua.push_str("},");
                }
                lua.push_str("},");
            }
            lua.push_str("},");
        }
        lua.push('}');
        Ok(lua)
    }
}
