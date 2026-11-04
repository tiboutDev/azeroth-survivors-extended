//! The inspect and armory panels share a formatter. Only extend the owned
//! current-rank string on the shared formatter's stack, after native localization.
use super::Adapter;
use crate::{official::{Entry, MutableMemory}, Result};

fn note(talent: &str) -> Option<&'static str> {
    Some(match talent {
        "divine_aegis" => "When your Power Word: Shield breaks on an ally, the explosion is centered on them.",
        "hammer_righteous" => "Only triggers when a shield you placed on yourself breaks.",
        "rapture" => "If your Power Word: Shield breaks on an ally, it shields that ally again.",
        "swiftmend" => "An ally below 40% health within 30 yards can also trigger your free Regrowth.",
        "tree_of_life" => "Your free Rejuvenation can heal an ally. Your overhealing shields allies for up to 20% of your maximum health.",
        "tranquility" => "When you heal an ally, the burst is centered on them and uses your area bonus.",
        "enh_ancestral_fortitude" => "Your free Healing Wave can target a wounded ally.",
        "fortified" => "Shield Slam taunts on hit. Allied absorb shields also grant your damage bonus.",
        "ardent_defender" => "Hammer of Justice taunts on hit. Allied absorb shields also grant your damage bonus.",
        "leader_of_pack" => "Shred taunts on hit.",
        "sacred_duty" => "An absorb shield from an ally also activates your damage bonus.",
        "shatter" | "dirty_deeds" => "The listed effects also count when applied by an ally.",
        _ => return None,
    })
}

impl Adapter {
    pub fn talent_panel_entry(self) -> Entry {
        Entry { rva: self.rva(0x1dca71), bytes: self.expected(0x1dca71, &[]) }
    }

    pub fn enrich_talent_panel(self, m: &dyn MutableMemory, base: u64, frame: u64, talent: u64) -> Result<()> {
        let Some(header_rva) = talent.checked_sub(base).and_then(|rva| rva.checked_add(0x40)) else { return Ok(()) };
        let Some((_, key)) = crate::current::COOP_TALENT_KEYS.iter().find(|(rva, _)| *rva == header_rva) else {
            return Ok(());
        };
        let Some(line) = note(key) else { return Ok(()) };
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
