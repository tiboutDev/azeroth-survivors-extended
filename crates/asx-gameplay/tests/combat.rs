use asx_gameplay::*;
use asx_gameplay::{
    marks::Marks,
    taunts::{Change, Hit, Taunts},
};
fn player(id: u32) -> Player {
    Player {
        id,
        phase: Phase::Playing,
        life: 0,
        position: Some([id as f32, 0., 0.]),
    }
}
fn enemy(id: u64) -> Enemy {
    Enemy {
        id,
        alive: true,
        marked: false,
        aura_full: false,
        vulnerability: 0.,
    }
}
#[derive(Default)]
struct Fake {
    supported: bool,
    own: Mark,
    enemies: Vec<Enemy>,
    writes: Vec<MarkWrite>,
    visuals: Vec<MarkVisual>,
    pursuit: Vec<Pursuit>,
    fail: bool,
}
impl CombatPort for Fake {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            marks: self.supported,
            taunts: self.supported,
        }
    }
    fn observe_mark(&mut self, active: bool) -> Result<Mark> {
        Ok(if active { self.own } else { Mark::default() })
    }
    fn observe_talents(&mut self) -> Result<Talents> {
        Ok(Talents::default())
    }
    fn local_player(&mut self) -> Result<LocalPlayer> {
        Ok(LocalPlayer {
            phase: Phase::Playing,
            position: Some([0., 0., 0.]),
        })
    }
    fn enemies(&mut self) -> Result<Vec<Enemy>> {
        Ok(self.enemies.clone())
    }
    fn write_mark(&mut self, c: MarkWrite) -> Result<()> {
        if self.fail {
            return Err("write failed".into());
        }
        self.enemies
            .iter_mut()
            .find(|e| e.id == c.enemy)
            .unwrap()
            .vulnerability = c.strength;
        self.writes.push(c);
        Ok(())
    }
    fn show_mark(&mut self, c: MarkVisual) -> Result<bool> {
        self.enemies
            .iter_mut()
            .find(|e| e.id == c.enemy)
            .unwrap()
            .marked = c.active;
        self.visuals.push(c);
        Ok(true)
    }
    fn publish_pursuit(&mut self, c: &[Pursuit], _: &[Player]) -> Result<()> {
        self.pursuit = c.to_vec();
        Ok(())
    }
}
#[test]
fn the_same_rules_run_against_an_unrelated_world_without_native_addresses() {
    let mut world = Fake {
        supported: true,
        own: Mark::new(Some(42), 0.2).unwrap(),
        enemies: vec![enemy(42)],
        ..Default::default()
    };
    let mut rules = Marks::default();
    let party = vec![player(1), player(2), player(3), player(4)];
    rules.players.insert(2, Mark::new(Some(42), 0.4).unwrap());
    assert!(sync_marks(&mut world, &mut rules, &party, 1).unwrap());
    assert_eq!(
        world.writes,
        vec![MarkWrite {
            enemy: 42,
            strength: 0.4
        }]
    );
    assert!(world.show_mark(rules.next_visual().unwrap()).unwrap());
    assert!(!sync_marks(&mut world, &mut rules, &party, 1).unwrap());
    assert_eq!(world.writes.len(), 1);
    assert!(rules.next_visual().is_none());
    rules.players.remove(&2);
    world.own = Mark::default();
    sync_marks(&mut world, &mut rules, &party, 1).unwrap();
    assert_eq!(world.writes.last().unwrap().strength, 0.);
    assert!(!rules.next_visual().unwrap().active);
}
#[test]
fn mark_lifetime_and_owner_departure_preserve_existing_rules() {
    let mut rules = Marks::default();
    rules.players.insert(1, Mark::new(Some(1), 0.2).unwrap());
    rules.players.insert(2, Mark::new(Some(1), 0.4).unwrap());
    let mut party = vec![player(1), player(2)];
    for phase in [Phase::Choosing, Phase::Paused, Phase::Downed] {
        party[1].phase = phase;
        assert_eq!(rules.effective(&party)[&1], 0.4);
    }
    party[1].phase = Phase::Inactive;
    assert_eq!(rules.effective(&party)[&1], 0.2);
    party.remove(0);
    assert!(rules.effective(&party).is_empty());
    for value in [f32::NAN, f32::INFINITY, -0.1, 0.5] {
        assert!(Mark::new(Some(1), value).is_err());
    }
    assert!(Mark::new(None, 0.2).is_err());
    assert!(Mark::new(Some(1), 0.).is_err());
    assert!(Mark::new(Some(0), 0.2).is_err());
}
#[test]
fn native_aura_capacity_and_dead_enemies_never_create_a_visual_loop() {
    let mut rules = Marks::default();
    rules.players.insert(1, Mark::new(Some(1), 0.2).unwrap());
    let party = vec![player(1)];
    let mut e = enemy(1);
    e.aura_full = true;
    assert_eq!(rules.reconcile(&party, &[e]).len(), 1);
    assert!(rules.next_visual().is_none());
    e.aura_full = false;
    rules.reconcile(&party, &[e]);
    e.alive = false;
    rules.reconcile(&party, &[e]);
    assert!(rules.next_visual().is_none());
}
#[test]
fn every_agreed_talent_gates_only_its_spell() {
    for (talents, spell) in [
        (
            Talents {
                fortified: true,
                ..Default::default()
            },
            TauntSpell::ShieldSlam,
        ),
        (
            Talents {
                ardent_defender: true,
                ..Default::default()
            },
            TauntSpell::HammerOfJustice,
        ),
        (
            Talents {
                dire_bear_form: true,
                ..Default::default()
            },
            TauntSpell::Shred,
        ),
    ] {
        for candidate in [
            TauntSpell::ShieldSlam,
            TauntSpell::HammerOfJustice,
            TauntSpell::Shred,
        ] {
            assert_eq!(Taunts::eligible(talents, candidate), candidate == spell);
            assert!(!Taunts::eligible(Talents::default(), candidate));
        }
    }
}
#[test]
fn taunts_keep_last_hit_ownership_and_clear_on_death_departure_or_old_life() {
    let mut rules = Taunts::default();
    let mut party = vec![player(1), player(2), player(3), player(4)];
    let enemies = vec![enemy(1), enemy(2)];
    let hit = |enemy, player, life| Hit {
        enemy,
        player,
        life,
        spell: TauntSpell::Shred,
    };
    let changes = rules.settle(&party, &enemies, [hit(1, 1, 0), hit(1, 2, 0), hit(2, 2, 0)]);
    assert_eq!(changes.len(), 3);
    assert_eq!(rules.target(1, &party), Some(2));
    for phase in [Phase::Choosing, Phase::Paused] {
        party[1].phase = phase;
        rules.settle(&party, &enemies, []);
        assert_eq!(rules.target(1, &party), Some(2));
    }
    party[1].phase = Phase::Downed;
    let changes = rules.settle(&party, &enemies, []);
    assert_eq!(changes.len(), 2);
    assert!(rules.bindings.is_empty());
    party[1].phase = Phase::Playing;
    party[1].life = 1;
    rules.settle(&party, &enemies, [hit(1, 2, 0)]);
    assert!(rules.bindings.is_empty());
    rules.settle(&party, &enemies, [hit(1, 2, 1)]);
    assert_eq!(rules.target(1, &party), Some(2));
    party.retain(|p| p.id != 2);
    rules.settle(&party, &enemies, []);
    assert!(rules.bindings.is_empty());
    rules.settle(&party, &enemies, [hit(1, 1, 0)]);
    rules.settle(&party, &[], []);
    assert!(rules.bindings.is_empty());
}
#[test]
fn pursuit_service_uses_semantic_commands_and_never_retargets_special_spells() {
    let mut world = Fake {
        supported: true,
        enemies: vec![enemy(12)],
        ..Default::default()
    };
    let mut rules = Taunts::default();
    let mut party = vec![player(1), player(2), player(3), player(4)];
    let changes = sync_taunts(
        &mut world,
        &mut rules,
        &mut party,
        1,
        vec![Hit {
            enemy: 12,
            player: 3,
            life: 0,
            spell: TauntSpell::HammerOfJustice,
        }],
    )
    .unwrap();
    assert!(matches!(changes[0], Change::Applied { .. }));
    assert_eq!(
        world.pursuit,
        vec![Pursuit {
            enemy: 12,
            player: 3
        }]
    );
    // This port intentionally has no special-spell-target command.
    party[2].phase = Phase::Downed;
    sync_taunts(&mut world, &mut rules, &mut party, 1, vec![]).unwrap();
    assert!(world.pursuit.is_empty());
}
#[test]
fn missing_capability_and_write_errors_are_explicit() {
    let mut world = Fake::default();
    let mut rules = Marks::default();
    assert!(sync_marks(&mut world, &mut rules, &[player(1)], 1).is_err());
    assert!(rules.players.is_empty());
    world.supported = true;
    world.fail = true;
    world.enemies = vec![enemy(42)];
    world.own = Mark::new(Some(42), 0.2).unwrap();
    assert!(sync_marks(&mut world, &mut rules, &[player(1)], 1).is_err());
    assert!(world.writes.is_empty());
}
#[test]
fn replaying_settled_state_does_not_duplicate_transitions() {
    let mut rules = Taunts::default();
    let party = vec![player(1)];
    let enemy = enemy(3);
    let hit = Hit {
        enemy: 3,
        player: 1,
        life: 0,
        spell: TauntSpell::ShieldSlam,
    };
    assert_eq!(rules.settle(&party, &[enemy], [hit]).len(), 1);
    assert!(rules.settle(&party, &[enemy], []).is_empty());
    assert_eq!(rules.bindings.len(), 1);
}
