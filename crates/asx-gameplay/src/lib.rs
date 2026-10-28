//! Stable, in-process gameplay boundary. No memory addresses, native phases,
//! registers, executable versions, packet opcodes or platform dependencies.
//! IDs are opaque gameplay identities; they must never be dereferenced.
pub mod client_contract;
pub mod marks;
pub mod support;
pub mod spells;
pub mod periodic;
pub mod offensive;
pub mod snapshot;
pub mod taunts;
pub type Result<T> = std::result::Result<T, String>;
pub type EnemyId = u64;
pub type PlayerId = u32;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub marks: bool,
    pub taunts: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Choosing,
    Paused,
    Downed,
    #[default]
    Inactive,
}
impl Phase {
    pub fn can_tank(self) -> bool {
        matches!(self, Self::Playing | Self::Choosing | Self::Paused)
    }
    pub fn retains_mark(self) -> bool {
        self.can_tank() || self == Self::Downed
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub id: PlayerId,
    pub phase: Phase,
    pub life: u64,
    pub position: Option<[f32; 3]>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Enemy {
    pub id: EnemyId,
    pub alive: bool,
    pub marked: bool,
    pub aura_full: bool,
    pub vulnerability: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mark {
    pub enemy: Option<EnemyId>,
    pub strength: f32,
}
impl Mark {
    pub fn new(enemy: Option<EnemyId>, strength: f32) -> Result<Self> {
        if !strength.is_finite()
            || !(0. ..=0.400001).contains(&strength)
            || enemy == Some(0)
            || (enemy.is_none() != (strength == 0.))
        {
            return Err("Invalid mark observation".into());
        }
        Ok(Self { enemy, strength })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TauntSpell {
    ShieldSlam,
    HammerOfJustice,
    Shred,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Talents {
    pub fortified: bool,
    pub ardent_defender: bool,
    pub dire_bear_form: bool,
}
impl Talents {
    pub fn allows(self, spell: TauntSpell) -> bool {
        match spell {
            TauntSpell::ShieldSlam => self.fortified,
            TauntSpell::HammerOfJustice => self.ardent_defender,
            TauntSpell::Shred => self.dire_bear_form,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarkWrite {
    pub enemy: EnemyId,
    pub strength: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkVisual {
    pub enemy: EnemyId,
    pub active: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pursuit {
    pub enemy: EnemyId,
    pub player: PlayerId,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalPlayer {
    pub phase: Phase,
    pub position: Option<[f32; 3]>,
}
/// Implemented by the native adapter and fake test worlds. Native calls execute
/// synchronously on the owning game thread. A visual may be deferred to a safe
/// entry boundary; it is not sent to an asynchronous gameplay service.
pub trait CombatPort {
    fn capabilities(&self) -> Capabilities;
    fn observe_mark(&mut self, active: bool) -> Result<Mark>;
    fn observe_talents(&mut self) -> Result<Talents>;
    fn local_player(&mut self) -> Result<LocalPlayer>;
    fn enemies(&mut self) -> Result<Vec<Enemy>>;
    fn write_mark(&mut self, command: MarkWrite) -> Result<()>;
    fn show_mark(&mut self, command: MarkVisual) -> Result<bool>;
    fn publish_pursuit(&mut self, commands: &[Pursuit], players: &[Player]) -> Result<()>;
}

/// Execute the same mark orchestration against a native or fake world.
pub fn sync_marks(
    port: &mut dyn CombatPort,
    rules: &mut marks::Marks,
    party: &[Player],
    owner: PlayerId,
) -> Result<bool> {
    if !port.capabilities().marks {
        return Err("Mark capability unavailable".into());
    }
    let active = party
        .iter()
        .any(|p| p.id == owner && p.phase.retains_mark());
    let own = port.observe_mark(active)?;
    let own = Mark::new(own.enemy, own.strength)?;
    let changed = rules.players.get(&owner) != Some(&own);
    rules.players.insert(owner, own);
    if rules.needs_world(party) {
        let enemies = port.enemies()?;
        for command in rules.reconcile(party, &enemies) {
            port.write_mark(command)?;
        }
    }
    Ok(changed)
}
pub fn sync_taunts(
    port: &mut dyn CombatPort,
    rules: &mut taunts::Taunts,
    players: &mut [Player],
    owner: PlayerId,
    hits: Vec<taunts::Hit>,
) -> Result<Vec<taunts::Change>> {
    if !port.capabilities().taunts {
        return Err("Taunt capability unavailable".into());
    }
    let local = port.local_player()?;
    if let Some(p) = players.iter_mut().find(|p| p.id == owner) {
        p.phase = local.phase;
        p.position = local.position;
    }
    let enemies = port.enemies()?;
    let changes = rules.settle(players, &enemies, hits);
    port.publish_pursuit(&rules.pursuit(players), players)?;
    Ok(changes)
}

pub mod official;


pub mod party_text;

pub mod menu_projection;

pub mod resurrection_fatigue;

pub mod blessings;

pub mod glyphs;

pub mod enchantments;

pub mod immunity_aggro;

pub mod duplicate_loot;

pub mod gems;

pub mod kept_bonus;
