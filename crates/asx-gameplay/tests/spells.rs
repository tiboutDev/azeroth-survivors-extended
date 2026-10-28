use asx_gameplay::{spells::*, support::Spell};
#[test]
fn successful_completion_is_once_and_drives_existing_hybrid_rules() {
    let mut e = Events::default();
    let id = e.begin(1, 2, Spell::HolyNova).unwrap();
    let c = e.finish(id).unwrap();
    assert!(c.group_heal);
    assert!(!c.clear_self_owner);
    assert_eq!(e.finish(id), None);
    assert_eq!(e.completed, 1);
    let id = e.begin(2, 2, Spell::HolyLight).unwrap();
    let c = e.finish(id).unwrap();
    assert!(c.clear_self_owner);
    assert!(!c.group_heal);
}
#[test]
fn an_unobserved_return_is_replacement_not_a_successful_or_failed_cast() {
    let mut e = Events::default();
    let id = e.begin(1, 2, Spell::Renew).unwrap();
    assert!(e.replace(id).is_some());
    assert_eq!(e.finish(id), None);
    assert_eq!(e.completed, 0);
    assert_eq!(e.replaced, 1);
    assert_eq!(e.active(), 0);
    assert!(matches!(e.recent.back(), Some(SpellEvent::Replaced(_))));
}
#[test]
fn active_bound_and_invalid_identity_fail_before_mutating_state() {
    let mut e = Events::default();
    assert!(e.begin(0, 2, Spell::HolyLight).is_err());
    for i in 1..=LIMIT {
        assert_eq!(e.begin(1, 2, Spell::HolyLight).unwrap(), i as u64);
    }
    assert!(e.begin(1, 2, Spell::HolyLight).is_err());
    e.replace(1);
    assert_eq!(e.begin(1, 2, Spell::HolyLight).unwrap(), 129);
}
#[test]
fn effective_credit_and_raw_proc_power_keep_the_original_caster() {
    let mut e = Events::default();
    let h = Healing {
        caster: 2,
        target: 1,
        spell: Spell::Renew,
        raw: 80.,
        effective: 20.,
        periodic: true,
    };
    let c = e.healing(h).unwrap();
    assert_eq!(
        (c.caster, c.target, c.raw, c.effective, c.foreign_periodic),
        (2, 1, 80., 20., 80.)
    );
    let c = e
        .healing(Healing {
            periodic: false,
            ..h
        })
        .unwrap();
    assert_eq!(c.foreign_periodic, 0.);
    for value in [f32::NAN, f32::INFINITY, -1., 1e13] {
        assert!(e.healing(Healing { raw: value, ..h }).is_err());
        assert!(e
            .healing(Healing {
                effective: value,
                ..h
            })
            .is_err());
    }
    assert_eq!(e.healed, 2);
}
#[test]
fn journal_is_bounded_without_losing_live_casts() {
    let mut e = Events::default();
    for _ in 0..1000 {
        let id = e.begin(1, 2, Spell::HolyLight).unwrap();
        e.finish(id);
    }
    assert_eq!(e.active(), 0);
    assert_eq!(e.started, e.completed);
    assert_eq!(e.recent.len(), 64);
}

#[test]
fn impacts_keep_caster_and_life_but_require_the_matching_talent() {
    use asx_gameplay::{Talents, TauntSpell as T};
    for (spell, talents) in [
        (
            T::ShieldSlam,
            Talents {
                fortified: true,
                ..Talents::default()
            },
        ),
        (
            T::HammerOfJustice,
            Talents {
                ardent_defender: true,
                ..Talents::default()
            },
        ),
        (
            T::Shred,
            Talents {
                dire_bear_form: true,
                ..Talents::default()
            },
        ),
    ] {
        let event = Impact {
            caster: 2,
            enemy: 42,
            spell,
            life: 3,
        };
        let mut e = Events::default();
        assert_eq!(e.impact(event, Talents::default()).unwrap(), None);
        assert_eq!(e.impacts, 1);
        let hit = e.impact(event, talents).unwrap().unwrap();
        assert_eq!(
            (hit.player, hit.enemy, hit.life, hit.spell),
            (2, 42, 3, spell)
        );
        assert_eq!(e.impacts, 2);
    }
}
#[test]
fn invalid_impacts_do_not_mutate_the_event_journal() {
    use asx_gameplay::{Talents, TauntSpell};
    let base = Impact {
        caster: 1,
        enemy: 42,
        spell: TauntSpell::ShieldSlam,
        life: 0,
    };
    let mut e = Events::default();
    for event in [
        Impact { caster: 0, ..base },
        Impact { enemy: 0, ..base },
        Impact {
            life: 1_000_001,
            ..base
        },
    ] {
        assert!(e.impact(event, Talents::default()).is_err());
    }
    assert_eq!(e.impacts, 0);
    assert!(e.recent.is_empty());
}
#[test]
fn impact_bursts_do_not_complete_casts_or_deduplicate_separate_native_hits() {
    use asx_gameplay::{Talents, TauntSpell};
    let mut e = Events::default();
    let id = e.begin(1, 2, Spell::HolyLight).unwrap();
    let event = Impact {
        caster: 1,
        enemy: 42,
        spell: TauntSpell::ShieldSlam,
        life: 0,
    };
    let talents = Talents {
        fortified: true,
        ..Talents::default()
    };
    for _ in 0..1000 {
        assert!(e.impact(event, talents).unwrap().is_some());
    }
    assert_eq!(e.impacts, 1000);
    assert_eq!(e.completed, 0);
    assert!(e.get(id).is_some());
    assert_eq!(e.recent.len(), 64);
    assert!(e.finish(id).is_some());
}

#[test]
fn confirmed_failure_consumes_once_without_success_effects() {
    let mut events = Events::default();
    for spell in [Spell::HolyNova, Spell::HolyShock, Spell::PowerWordShield] {
        let id = events.begin(1, 2, spell).unwrap();
        assert!(events.fail(id).is_some());
        assert!(matches!(events.recent.back(), Some(SpellEvent::Failed(_))));
        assert!(events.fail(id).is_none());
        assert!(events.finish(id).is_none());
        assert!(events.replace(id).is_none());
    }
    assert_eq!((events.started, events.failed, events.completed, events.active()), (3, 3, 0, 0));
}
