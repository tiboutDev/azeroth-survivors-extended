//! The inspect and armory panels share a formatter. Only extend the owned
//! current-rank string on the shared formatter's stack, after native localization.
use super::Adapter;
use crate::{official::{Entry, MutableMemory}, Result};

fn note(talent: u64) -> Option<&'static str> {
    Some(match talent {
        0x7136ea0 => "When your Power Word: Shield breaks on an ally, the explosion is centered on them.",
        0x712f0d8 => "Only triggers when a shield you placed on yourself breaks.",
        0x7137160 => "If your Power Word: Shield breaks on an ally, it shields that ally again.",
        0x7146d08 => "An ally below 40% health within 30 yards can also trigger your free Regrowth.",
        0x7147288 => "Your free Rejuvenation can heal an ally. Your overhealing shields allies for up to 20% of your maximum health.",
        0x7146fc8 => "When you heal an ally, the burst is centered on them and uses your area bonus.",
        0x713baa0 => "Your free Healing Wave can target a wounded ally.",
        0x712c6d8 => "Shield Slam taunts on hit. Allied absorb shields also grant your damage bonus.",
        0x712eec8 => "Hammer of Justice taunts on hit. Allied absorb shields also grant your damage bonus.",
        0x7146250 => "Shred taunts on hit.",
        0x712f028 => "An absorb shield from an ally also activates your damage bonus.",
        0x713fad0 | 0x7136218 => "The listed effects also count when applied by an ally.",
        _ => return None,
    })
}

impl Adapter {
    pub fn talent_panel_entry(self) -> Entry {
        Entry { rva: self.rva(0x1dca71), bytes: self.expected(0x1dca71, &[]) }
    }

    pub fn enrich_talent_panel(self, m: &dyn MutableMemory, base: u64, frame: u64, talent: u64) -> Result<()> {
        let Some(line) = talent.checked_sub(base).and_then(|rva| rva.checked_add(0x40)).and_then(note) else { return Ok(()) };
        let header = frame - 0x30;
        let raw = m.read(header, 24)?;
        let word = |at| u64::from_le_bytes(raw[at..at+8].try_into().unwrap());
        let (capacity, ptr, len) = (word(0), word(8), word(16));
        if len == 0 || len > capacity || capacity > 16384 || ptr < 65536 {
            return Err("Invalid native talent description".into());
        }
        let mut value = m.read(ptr, len as usize)?;
        value.extend_from_slice(b"\n\nCo-op: ");
        value.extend_from_slice(line.as_bytes());
        let size = value.len();
        let owned = m.allocate_native(&value)?;
        let replacement = [size as u64, owned, size as u64].map(u64::to_le_bytes).concat();
        if let Err(e) = m.write(header, &replacement) {
            m.free_native(owned, size)?;
            return Err(e);
        }
        m.free_native(ptr, capacity as usize)
    }
}
