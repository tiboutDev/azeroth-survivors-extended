//! Portable parsing for bounded native enemy records.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyLayout {
    pub stride: u64,
    pub guid_offset: u64,
    pub guid_gap: u64,
    pub hp_gap: u64,
}

impl EnemyLayout {
    /// Validated offsets of dead, health, motion and the exclusive combat end.
    pub fn combat_bounds(self) -> crate::Result<[usize; 4]> {
        if self.stride == 0 || self.stride > 0x1000 || self.guid_gap > self.stride
            || self.guid_offset > self.stride || self.hp_gap > self.stride {
            return Err("Invalid enemy record bounds".into());
        }
        let dead = self.guid_offset.checked_sub(4 + self.guid_gap).ok_or("Invalid enemy GUID/dead layout")?;
        let hp = self.guid_offset + 8 + self.hp_gap;
        let motion = hp + 4 + 24;
        let end = motion + 32;
        if end > self.stride { return Err("Enemy fields exceed the record stride".into()); }
        Ok([dead as usize, hp as usize, motion as usize, end as usize])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkippedEnemy {
    pub guid: u64,
    pub now: f64,
    /// hp, x0, y0, z0, x1, y1, z1, began, duration
    pub fields: [f64; 9],
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnemyExact {
    pub guid: u64,
    pub hp: f64,
    pub position: [f64; 3],
}

fn u32_at(d: &[u8], at: usize) -> u32 { u32::from_le_bytes(d[at..at + 4].try_into().unwrap()) }
fn u64_at(d: &[u8], at: usize) -> u64 { u64::from_le_bytes(d[at..at + 8].try_into().unwrap()) }
fn f32_at(d: &[u8], at: usize) -> f32 { f32::from_le_bytes(d[at..at + 4].try_into().unwrap()) }

/// Parses the native enemy record shape supplied by the selected adapter.
pub fn parse_enemies_layout(data: &[u8], now: f32, layout: EnemyLayout) -> crate::Result<(Vec<EnemyExact>, Vec<SkippedEnemy>)> {
    let [dead, hp, _, end] = layout.combat_bounds()?;
    let record = end - dead;
    let mut out = Vec::new();
    let mut skipped = Vec::new();
    let mut offset = dead;
    if data.len() % layout.stride as usize != 0 { return Err("Truncated native enemy array".into()); }
    while offset < data.len() {
        let record_end = offset + record;
        if record_end > data.len() { return Err("unpack requires a buffer of enough bytes".into()); }
        let r = &data[offset..record_end];
        if u32_at(r, 0) != 1 {
            let guid = u64_at(r, 4 + layout.guid_gap as usize);
            let mut at = hp - dead;
            let hp = f32_at(r, at) as f64;
            at += 4 + 24;
            let mut v = [0f64; 8];
            for (i, slot) in v.iter_mut().enumerate() { *slot = f32_at(r, at + 4 * i) as f64; }
            let (x0, y0, z0, x1, y1, z1, began, duration) = (v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
            let now = now as f64;
            if ![now, hp, x0, y0, z0, x1, y1, z1, began, duration].iter().all(|v| v.is_finite()) {
                skipped.push(SkippedEnemy { guid, now, fields: [hp, x0, y0, z0, x1, y1, z1, began, duration] });
                offset += layout.stride as usize;
                continue;
            }
            let t = if duration > 0.0 { ((now - began) / duration).clamp(0.0, 1.0) } else { 1.0 };
            out.push(EnemyExact { guid, hp, position: [x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, z0 + (z1 - z0) * t] });
        }
        offset += layout.stride as usize;
    }
    Ok((out, skipped))
}
