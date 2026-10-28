use asx_gameplay::{support::*, Phase, Result};
use std::collections::BTreeMap;
fn member(id: u32, hp: f64, max: f64) -> Member {
    Member {
        id,
        kind: UnitKind::Player,
        hp,
        max,
        position: [0.; 3],
        phase: Phase::Playing,
        seen: 10.,
        life: 0,
        active: Active::default(),
    }
}
fn party() -> Vec<Member> {
    vec![
        member(1, 80., 100.),
        member(2, 60., 200.),
        member(3, 20., 100.),
        member(4, 90., 100.),
    ]
}
#[derive(Default)]
struct Port {
    commands: Vec<Apply>,
    state: Option<RecipientState>,
    fail: bool,
}
impl SupportPort for Port {
    fn recipient(&mut self) -> Result<RecipientState> {
        Ok(self.state.unwrap_or(RecipientState {
            clock: 10.,
            active: Active::default(),
            shield: Shield {
                current: 0.,
                total: 0.,
            },
            retaliation_scale: 2.,
        }))
    }
    fn apply(&mut self, c: Apply) -> Result<bool> {
        if self.fail {
            return Err("native failure".into());
        }
        self.commands.push(c);
        Ok(true)
    }
}
fn request(spell: Spell) -> Request {
    Request {
        caster: 1,
        recipient: 2,
        spell,
        amount: 100.,
        buff: None,
        expires: 11.,
        life: 0,
    }
}
fn buff(s: Spell) -> Buff {
    Buff {
        spell: s,
        value: 0.1,
        auxiliary: 0.,
        expires: 20.,
        duration: 10.,
    }
}
#[test]
fn heals_use_percentage_including_caster_and_stable_ties() {
    let mut p = party();
    assert_eq!(
        select(&p, &BTreeMap::new(), 1, Spell::HolyLight, 10.),
        Some(3)
    );
    p[0].hp = 20.;
    assert_eq!(
        select(&p, &BTreeMap::new(), 1, Spell::Regrowth, 10.),
        Some(1)
    );
    p[0].hp = 100.;
    p[2].hp = 100.;
    assert_eq!(
        select(&p, &BTreeMap::new(), 1, Spell::HealingWave, 10.),
        Some(2)
    );
}
#[test]
fn player_only_living_fresh_in_range() {
    let mut p = party();
    p[2].kind = UnitKind::Pet;
    p[1].phase = Phase::Downed;
    p[3].position = [31., 0., 0.];
    assert_eq!(
        select(&p, &BTreeMap::new(), 1, Spell::HolyLight, 10.),
        Some(1)
    );
    p[0].seen = 9.;
    assert_eq!(select(&p, &BTreeMap::new(), 1, Spell::HolyLight, 10.), None);
}
#[test]
fn periodic_heals_do_not_refresh_but_regrowth_is_direct() {
    let mut p = party();
    p[2].active.renew = true;
    p[1].active.renew = true;
    assert_eq!(select(&p, &BTreeMap::new(), 1, Spell::Renew, 10.), Some(1));
    assert_eq!(
        select(&p, &BTreeMap::new(), 1, Spell::Regrowth, 10.),
        Some(3)
    );
    assert!(!Spell::Regrowth.periodic());
}
#[test]
fn pressure_buffs_ignore_health_except_priest_shield() {
    let p = party();
    let pressure = BTreeMap::from([(1, 4), (2, 4), (3, 2), (4, 1)]);
    assert_eq!(select(&p, &pressure, 1, Spell::Thorns, 10.), Some(1));
    assert_eq!(
        select(&p, &pressure, 1, Spell::LightningShield, 10.),
        Some(1)
    );
    assert_eq!(
        select(&p, &pressure, 1, Spell::PowerWordShield, 10.),
        Some(2)
    );
}
#[test]
fn invalid_or_buffed_candidate_cannot_win_pressure() {
    let mut p = party();
    p[0].active.thorns = true;
    p[1].position = [31., 0., 0.];
    let pressure = BTreeMap::from([(1, 100), (2, 99), (3, 2), (4, 1)]);
    assert_eq!(select(&p, &pressure, 1, Spell::Thorns, 10.), Some(3));
}
#[test]
fn pressure_counts_live_nearby_enemies() {
    let p = party();
    let e = [
        TargetEnemy {
            id: 1,
            alive: true,
            position: [10., 0., 0.],
        },
        TargetEnemy {
            id: 2,
            alive: false,
            position: [0.; 3],
        },
        TargetEnemy {
            id: 3,
            alive: true,
            position: [11., 0., 0.],
        },
    ];
    assert_eq!(pressure(&p, &e)[&1], 1);
}
#[test]
fn hybrids_use_per_unit_amount_and_never_redirect_offensive_targets() {
    let mut p = party();
    p[1].position = [26., 0., 0.];
    p[2].hp = 100.;
    assert_eq!(group_targets(&p, 1, Spell::HolyNova, 10.), vec![4]);
    assert_eq!(group_targets(&p, 1, Spell::HolyShock, 10.), vec![2, 4]);
    assert_eq!(group_targets(&p, 1, Spell::BattleShout, 10.), vec![2, 3, 4]);
    assert_eq!(elementary_heal(0.1, 200.), 20.);
    assert!(!redirect_heal(1, 2, Spell::HolyShock, 20.));
    assert!(redirect_heal(1, 2, Spell::HolyLight, 20.));
}
#[test]
fn delivery_uses_semantic_commands_and_propagates_failure() {
    let mut port = Port::default();
    assert!(deliver(&mut port, request(Spell::HolyLight), &party(), 10.).unwrap());
    assert_eq!(port.commands[0].request.recipient, 2);
    port.fail = true;
    assert!(deliver(&mut port, request(Spell::HolyLight), &party(), 10.).is_err());
}
#[test]
fn delivery_rejects_expired_dead_foreign_life_and_invalid_values() {
    for variant in 0..7 {
        let mut p = party();
        let mut r = request(Spell::HolyLight);
        match variant {
            0 => r.expires = 10.,
            1 => p[1].phase = Phase::Downed,
            2 => r.life = 1,
            3 => p[0].seen = 9.,
            4 => p[0].position = [31., 0., 0.],
            5 => p[1].kind = UnitKind::Pet,
            _ => p[0].seen = 11.,
        };
        let mut port = Port::default();
        assert!(!deliver(&mut port, r, &p, 10.).unwrap());
        assert!(port.commands.is_empty());
    }
    let mut r = request(Spell::HolyLight);
    r.amount = f32::NAN;
    assert!(deliver(&mut Port::default(), r, &party(), 10.).is_err());
}
#[test]
fn buff_rebase_preserves_caster_power_and_rejects_overflow() {
    let mut port = Port::default();
    let mut r = request(Spell::Renew);
    r.buff = Some(buff(Spell::Renew));
    deliver(&mut port, r.clone(), &party(), 10.).unwrap();
    assert_eq!(port.commands[0].request.buff.unwrap().value, 0.05);
    r.spell = Spell::Thorns;
    r.buff = Some(buff(Spell::Thorns));
    deliver(&mut port, r.clone(), &party(), 10.).unwrap();
    assert_eq!(port.commands[1].request.buff.unwrap().value, 5.);
    r.amount = 1e12;
    r.buff.as_mut().unwrap().value = 1e12;
    assert!(!deliver(&mut port, r, &party(), 10.).unwrap());
}
#[test]
fn shield_ownership_changes_only_after_valid_nonweaker_application() {
    let mut port = Port::default();
    let mut r = request(Spell::PowerWordShield);
    r.buff = Some(buff(r.spell));
    deliver(&mut port, r.clone(), &party(), 10.).unwrap();
    assert_eq!(
        port.commands[0].shield,
        Some(Shield {
            current: 100.,
            total: 100.
        })
    );
    port.state = Some(RecipientState {
        clock: 10.,
        active: Active {
            shield: true,
            ..Active::default()
        },
        shield: Shield {
            current: 120.,
            total: 150.,
        },
        retaliation_scale: 1.,
    });
    assert!(!deliver(&mut port, r.clone(), &party(), 10.).unwrap());
    port.state.as_mut().unwrap().active.shield = false;
    assert!(!deliver(&mut port, r.clone(), &party(), 10.).unwrap());
    port.state.as_mut().unwrap().shield.current = 20.;
    r.buff.as_mut().unwrap().expires = 9.;
    assert!(!deliver(&mut port, r, &party(), 10.).unwrap());
    assert_eq!(port.commands.len(), 1);
}
#[test]
fn druid_damage_uses_recipient_origin_and_caster_coefficient() {
    let enemies = [
        TargetEnemy {
            id: 1,
            alive: true,
            position: [20., 0., 0.],
        },
        TargetEnemy {
            id: 2,
            alive: true,
            position: [0.; 3],
        },
        TargetEnemy {
            id: 3,
            alive: false,
            position: [20., 0., 0.],
        },
    ];
    assert_eq!(
        nature_burst(40., 1.5, 5., [20., 0., 0.], &enemies),
        vec![Damage {
            enemy: 1,
            amount: 60.
        }]
    );
    assert_eq!(threshold_hp(&member(2, 60., 200.), 100.), 30.);
}
