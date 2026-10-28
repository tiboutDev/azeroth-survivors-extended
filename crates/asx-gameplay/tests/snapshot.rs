use asx_gameplay::{
    snapshot::*,
    support::{Active, Member, UnitKind},
    Phase,
};
fn snapshot() -> GameSnapshot {
    GameSnapshot {
        generation: 1,
        observed_at: 10.,
        local: 1,
        players: vec![Member {
            id: 1,
            kind: UnitKind::Player,
            hp: 50.,
            max: 100.,
            position: [0.; 3],
            phase: Phase::Playing,
            seen: 10.,
            life: 2,
            active: Active::default(),
        }],
        enemies: None,
        progress: Progress {
            phase: Phase::Playing,
            level: 2,
            xp: 3,
        },
        resources: Some(Resources {
            gold: 7,
            earned: 10,
            next_chest: 20,
        }),
    }
}
#[test]
fn unavailable_enemies_are_distinct_from_a_confirmed_empty_world() {
    let mut s = snapshot();
    assert_eq!(s.pressure(), None);
    s.enemies = Some(vec![]);
    assert_eq!(s.pressure().unwrap()[&1], 0);
}
#[test]
fn shared_pressure_uses_the_same_observation_and_excludes_dead_enemies() {
    let mut s = snapshot();
    s.enemies = Some(vec![
        EnemyState {
            id: 1,
            hp: 10.,
            position: [3., 0., 0.],
        },
        EnemyState {
            id: 2,
            hp: 0.,
            position: [0.; 3],
        },
        EnemyState {
            id: 3,
            hp: 10.,
            position: [11., 0., 0.],
        },
    ]);
    assert_eq!(s.pressure().unwrap()[&1], 1);
    assert_eq!(s.local_player().unwrap().life, 2);
}
#[test]
fn snapshots_are_owned_and_keep_their_generation_when_new_inputs_change() {
    let s = snapshot();
    let mut next = s.clone();
    next.generation = 2;
    next.players[0].hp = 10.;
    assert_eq!(s.players[0].hp, 50.);
    assert_eq!(s.generation, 1);
}
