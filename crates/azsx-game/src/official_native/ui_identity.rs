//! Bounded current UI aliases for native healing and event presentation.
use super::*;
fn word(m: &dyn Mem, a: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(
        m.read(a, 8)?
            .try_into()
            .map_err(|_| "Truncated plate pointer")?,
    ))
}
impl Adapter {
    pub fn native_ui_participants(self, m: &dyn Mem, script: u64) -> Result<Vec<u64>> {
        if script < 0x10000 {
            return Ok(vec![]);
        }
        let aliases = script + crate::current::UI_ALIASES;
        let own = word(m, aliases + crate::current::UI_SELF_GUID)?;
        if own == 0 {
            return Ok(vec![]);
        }
        let mut ids = vec![own];
        for raw in m
            .read(aliases + crate::current::UI_PARTY_GUIDS, 32)?
            .chunks_exact(8)
        {
            let guid = u64::from_le_bytes(raw.try_into().unwrap());
            if guid != 0 && !ids.contains(&guid) {
                ids.push(guid);
            }
        }
        Ok(ids)
    }
}
