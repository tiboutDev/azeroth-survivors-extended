//! Selected native immunity and NPC target-pool translation.
use super::Adapter;
use crate::{memory::Mem, official::MutableMemory, Result};
use asx_gameplay::immunity_aggro::Candidate;
#[derive(Clone, Debug)]
pub struct AggroEnemy {
    pub id: u64,
    pub target: Option<u32>,
    pub position: [f32; 3],
}
/// A scoped mask of target eligibility, not the actor's actual gameplay phase.
pub struct AggroLoan {
    game: u64,
    header: Vec<u8>,
    phases: Vec<(u32, u8)>,
}
fn q(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
fn f(b: &[u8], i: usize) -> f32 {
    f32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
impl Adapter {
    pub fn supports_immunity_aggro(self) -> bool {
        true
    }
    pub fn combat_immune(self, m: &dyn Mem, game: u64, slot: u32, guid: u64) -> Result<bool> {
        if !self.supports_immunity_aggro() {
            return Ok(false);
        }
        let row = self.support_row(m, game, slot)?;
        if guid == 0 || m.read(row + self.row_field(row == game, 0x478), 8)? != guid.to_le_bytes() {
            return Err("Immunity actor identity changed".into());
        }
        let clock = f(&m.read(game + self.game_field(0xb9c), 4)?, 0);
        if !clock.is_finite() {
            return Err("Invalid immunity clock".into());
        }
        let (_, rows) = self.aura_rows(m, row)?;
        // Native hurt rejects precisely unexpired effect 12, independent of spell,
        // talent, race, rank and glyph-adjusted duration. Absorption is effect 7.
        Ok(rows
            .iter()
            .any(|r| u32::from_le_bytes(r[..4].try_into().unwrap()) == 12 && f(r, 16) > clock))
    }
    fn aggro_header(self, m: &dyn Mem, game: u64) -> Result<Vec<u8>> {
        if !self.supports_immunity_aggro() {
            return Err("Unsupported immunity targeting".into());
        }
        if m.read(game + self.game_field(0xc41), 1)? != [1] {
            return Err("Immunity targeting requires native coop".into());
        }
        let h = m.read(
            game + self.field1201(crate::current::AGGRO_CANDIDATES),
            24,
        )?;
        if h.len() != 24 {
            return Err("Truncated target pool".into());
        }
        let (cap, p, n) = (q(&h, 0), q(&h, 8), q(&h, 16));
        if n > cap
            || n > 5
            || cap > 16
            || (n > 0 && (p < 0x10000 || p.checked_add(n * 16).is_none()))
        {
            return Err("Invalid target pool".into());
        }
        Ok(h)
    }
    pub fn aggro_snapshot(
        self,
        m: &dyn Mem,
        game: u64,
    ) -> Result<(Vec<Candidate>, Vec<AggroEnemy>)> {
        let h = self.aggro_header(m, game)?;
        let p = q(&h, 8);
        let n = q(&h, 16);
        let rows = if n == 0 {
            vec![]
        } else {
            m.read(p, (n * 16) as usize)?
        };
        let mut candidates = vec![];
        for (slot, r) in rows.chunks_exact(16).enumerate() {
            let position = std::array::from_fn(|i| f(r, i * 4));
            if position.iter().any(|v| !v.is_finite()) {
                return Err("Invalid target position".into());
            }
            candidates.push(Candidate {
                slot: slot as u32,
                position,
                available: r[12] == 1,
            });
        }
        let (_, rows) = self.combat_rows(m, game)?;
        use crate::current as layout;
        let enemies = rows
            .chunks_exact(layout::ENEMY_STRIDE)
            .filter(|r| r[layout::ENEMY_DEAD] == 0 && f(r, layout::ENEMY_HP) > 0.)
            .map(|r| {
                let target = q(r, layout::ENEMY_TARGET);
                AggroEnemy {
                    id: q(r, layout::ENEMY_GUID),
                    target: (target < n).then_some(target as u32),
                    position: std::array::from_fn(|i| f(r, layout::ENEMY_POSITION + i * 4)),
                }
            })
            .collect::<Vec<_>>();
        if enemies
            .iter()
            .any(|e| e.position.iter().any(|v| !v.is_finite()))
        {
            return Err("Invalid enemy target position".into());
        }
        Ok((candidates, enemies))
    }
    pub fn aggro_prepare(
        self,
        m: &dyn MutableMemory,
        game: u64,
        blocked: &[u32],
        changes: &[(u64, Option<u32>)],
    ) -> Result<AggroLoan> {
        let header = self.aggro_header(m, game)?;
        let pointer = q(&header, 8);
        let count = q(&header, 16);
        if blocked.iter().any(|s| u64::from(*s) >= count)
            || changes
                .iter()
                .any(|(_, s)| s.is_some_and(|s| u64::from(s) >= count))
        {
            return Err("Target slot outside native pool".into());
        }
        // Validate and collect all IO before mutations. The target vector is only read
        // and copied by NPC update, never resized there. No game-state mutex spans it.
        let (p, rows) = self.combat_rows(m, game)?;
        let mut writes = vec![];
        let changes = changes
            .iter()
            .copied()
            .collect::<std::collections::BTreeMap<_, _>>();
        use crate::current as layout;
        for (i, row) in rows.chunks_exact(layout::ENEMY_STRIDE).enumerate() {
            if let Some(target) = changes.get(&q(row, layout::ENEMY_GUID)) {
                if row[layout::ENEMY_DEAD] == 0 {
                    writes.push((
                        p + i as u64 * layout::ENEMY_STRIDE as u64 + layout::ENEMY_TARGET as u64,
                        target.map(u64::from).unwrap_or(u64::MAX),
                    ));
                }
            }
        }
        let mut phases = vec![];
        for slot in blocked {
            let old = m.read(pointer + u64::from(*slot) * 16 + 12, 1)?[0];
            if old == 1 {
                phases.push((*slot, old));
            }
        }
        for (at, target) in writes {
            m.write(at, &target.to_le_bytes())?;
        }
        let loan = AggroLoan {
            game,
            header,
            phases,
        };
        for (slot, _) in &loan.phases {
            if let Err(e) = m.write(pointer + u64::from(*slot) * 16 + 12, &[0]) {
                let _ = self.aggro_restore(m, loan);
                return Err(e);
            }
        }
        Ok(loan)
    }
    pub fn aggro_restore(self, m: &dyn MutableMemory, loan: AggroLoan) -> Result<()> {
        if self.aggro_header(m, loan.game)? != loan.header {
            return Err("Native target pool changed during NPC pass".into());
        }
        let p = q(&loan.header, 8);
        for (slot, phase) in loan.phases {
            m.write(p + u64::from(slot) * 16 + 12, &[phase])?;
        }
        Ok(())
    }
}

impl Adapter {
    /// Controlled diagnostic aura insertion. Four bounded allocations per test;
    /// previous backing buffers remain alive until the disposable process exits.
    /// This intentionally does not call the network-only support emitter.
    pub fn fixture_immunity(
        self,
        io: &dyn crate::official::SupportCalls,
        game: u64,
        slot: u32,
        spell: u32,
        duration: f32,
    ) -> Result<()> {
        if !self.supports_immunity_aggro()
            || !matches!(spell, 642 | 20580 | 7744 | 1856)
            || !(0.1..=10.).contains(&duration)
        {
            return Err("Invalid immunity fixture".into());
        }
        if io.read(game + self.game_field(0xb40), 8)? != 0u64.to_le_bytes() {
            return Err("Immunity fixture requires restored host".into());
        }
        let row = self.support_row(io, game, slot)?;
        let at = row + self.row_field(row == game, 0x398);
        let h = io.read(at, 24)?;
        let count = q(&h, 16);
        let cap = q(&h, 0);
        let p = q(&h, 8);
        if count > cap || cap > 4096 || count >= 128 || (count > 0 && p < 0x10000) {
            return Err("Invalid fixture aura vector".into());
        }
        let mut rows = if count == 0 {
            vec![]
        } else {
            io.read(p, (count * 32) as usize)?
        };
        let clock = f(&io.read(game + self.game_field(0xb9c), 4)?, 0);
        if !clock.is_finite() {
            return Err("Invalid fixture clock".into());
        }
        let mut aura = [0; 32];
        aura[8..12].copy_from_slice(&12u32.to_le_bytes());
        aura[20..24].copy_from_slice(&spell.to_le_bytes());
        aura[24..28].copy_from_slice(&(clock + duration).to_le_bytes());
        rows.extend(aura);
        let owned = io.allocate_native(&rows)?;
        let n = count + 1;
        io.write(
            at,
            &[n.to_le_bytes(), owned.to_le_bytes(), n.to_le_bytes()].concat(),
        )
    }
}
