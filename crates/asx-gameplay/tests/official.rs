use asx_gameplay::official::*;
fn party() -> Session {
    Session::new(1, 1, [1, 2, 3], 20).unwrap()
}
fn apply(s: &Session, e: Event) -> Session {
    s.prepare(s.generation(), s.sequence() + 1, e).unwrap()
}
#[test]
fn converted_gold_floors_once_for_three_and_spending_is_atomic() {
    let s = apply(
        &party(),
        Event::Gold {
            amount: 10,
            source: Award::ConvertedReward,
        },
    );
    assert_eq!(s.gold(), 23);
    assert!(s
        .prepare(
            1,
            2,
            Event::Spend {
                actor: 2,
                amount: 24
            }
        )
        .is_err());
    assert_eq!(s.gold(), 23);
    let next = apply(
        &s,
        Event::Spend {
            actor: 2,
            amount: 20,
        },
    );
    assert_eq!(next.gold(), 3);
    assert!(next
        .prepare(
            1,
            2,
            Event::Spend {
                actor: 2,
                amount: 1
            }
        )
        .is_err());
    assert!(next.prepare(2, 3, Event::Experience { amount: 3 }).is_err());
    assert!(next
        .prepare(
            1,
            3,
            Event::Spend {
                actor: 99,
                amount: 1
            }
        )
        .is_err());
}
#[test]
fn dead_member_cannot_hold_augment_pause_and_resurrection_does_not_requeue() {
    let s = apply(
        &party(),
        Event::SetState {
            actor: 3,
            state: MemberState::Downed,
        },
    );
    let s = apply(
        &s,
        Event::OpenChoice {
            id: 1,
            kind: ChoiceKind::Augment,
        },
    );
    let s = apply(&s, Event::CompleteChoice { id: 1, actor: 1 });
    let s = apply(
        &s,
        Event::SetState {
            actor: 3,
            state: MemberState::Alive,
        },
    );
    assert_eq!(
        s.pause(),
        Some(PauseReason::Choices {
            kind: ChoiceKind::Augment,
            waiting: vec![2]
        })
    );
    assert!(apply(&s, Event::CompleteChoice { id: 1, actor: 2 })
        .pause()
        .is_none());
}
#[test]
fn overlapping_choices_and_menu_waits_do_not_resume_early() {
    let s = apply(
        &party(),
        Event::SetMenu {
            actor: 1,
            open: true,
        },
    );
    let s = apply(
        &s,
        Event::OpenChoice {
            id: 1,
            kind: ChoiceKind::EliteSkill,
        },
    );
    let s = apply(
        &s,
        Event::OpenChoice {
            id: 2,
            kind: ChoiceKind::EliteItem,
        },
    );
    let s = apply(&s, Event::CompleteChoice { id: 1, actor: 1 });
    let s = apply(
        &s,
        Event::SetState {
            actor: 2,
            state: MemberState::Downed,
        },
    );
    let s = apply(&s, Event::Leave { actor: 3 });
    assert_eq!(
        s.pause(),
        Some(PauseReason::Choices {
            kind: ChoiceKind::EliteItem,
            waiting: vec![1]
        })
    );
    let s = apply(&s, Event::CompleteChoice { id: 2, actor: 1 });
    assert_eq!(s.pause(), Some(PauseReason::Menus { players: vec![1] }));
    assert!(apply(
        &s,
        Event::SetMenu {
            actor: 1,
            open: false
        }
    )
    .pause()
    .is_none());
}
#[test]
fn identity_and_reward_replay_guards_preserve_state() {
    assert!(Session::new(1, 1, [1, 1, 2], 0).is_err());
    assert!(Session::new(1, 5, [1, 2, 3], 0).is_err());
    let s = apply(
        &party(),
        Event::OpenChoice {
            id: 1,
            kind: ChoiceKind::Chest,
        },
    );
    assert!(s
        .prepare(
            1,
            2,
            Event::OpenChoice {
                id: 1,
                kind: ChoiceKind::Chest
            }
        )
        .is_err());
    assert!(s.prepare(1, 2, Event::Leave { actor: 1 }).is_err());
    let s = apply(&s, Event::Experience { amount: 500 });
    assert_eq!(s.experience(), 500);
    assert_eq!(best_racial_bonus([0.1, 0.1, 0.05]).unwrap(), 0.1);
    assert!(best_racial_bonus([f64::NAN]).is_err());
}

#[test]
fn native_choice_barrier_excludes_dead_and_inactive_but_retains_manual_pause() {
    use asx_gameplay::Phase;
    assert_eq!(
        group_pause([
            (1, Phase::Choosing),
            (2, Phase::Playing),
            (3, Phase::Downed)
        ]),
        Some(PauseReason::Choices {
            kind: ChoiceKind::Reward,
            waiting: vec![1]
        })
    );
    assert!(group_pause([
        (1, Phase::Playing),
        (2, Phase::Inactive),
        (3, Phase::Downed)
    ])
    .is_none());
    assert_eq!(
        group_pause([(1, Phase::Paused), (2, Phase::Downed)]),
        Some(PauseReason::Menus { players: vec![1] })
    );
}

#[test]
fn waiting_view_does_not_replace_owner_choice_or_dead_rescue_ui() {
    use asx_gameplay::Phase;
    let reason = Some(PauseReason::Choices {
        kind: ChoiceKind::Reward,
        waiting: vec![3],
    });
    let view = pause_presentation(reason.clone(), Phase::Playing, |_| None);
    assert!(view.active && view.visible && !view.local_menu);
    assert_eq!(view.detail, "Player 3 is choosing a reward.");
    for phase in [
        Phase::Choosing,
        Phase::Downed,
        Phase::Inactive,
        Phase::Paused,
    ] {
        assert!(!pause_presentation(reason.clone(), phase, |_| None).visible);
    }
    assert!(!pause_presentation(None, Phase::Playing, |_| None).active);
}

#[test]
fn shared_xp_uses_strongest_racial_and_preserves_personal_item_bonuses() {
    for count in 1..=4 {
        let bonuses = vec![(1.1, 0.1); count];
        assert_eq!(shared_xp_multiplier(bonuses).unwrap(), 1.1);
    }
    assert!(
        (shared_xp_multiplier([(1.3, 0.1), (1.4, 0.1), (1.0, 0.0)]).unwrap() - 1.6).abs() < 1e-6
    );
    assert_eq!(shared_xp_award(100, 1.1).unwrap(), 110);
    assert_eq!(shared_xp_award(3, 1.5).unwrap(), 5);
    assert_eq!(shared_xp_award(0, 0.0).unwrap(), 1);
    assert!(shared_xp_multiplier([]).is_err());
    assert!(shared_xp_multiplier([(f32::NAN, 0.0)]).is_err());
    assert!(shared_xp_award(u32::MAX, 2.0).is_err());
    assert!(shared_xp_award(1, f32::INFINITY).is_err());
}

#[test]
fn canonical_wallet_rejects_overspend_and_reward_conversion_discards_fractions() {
    let start = SharedWallet {
        gold: 12,
        earned: 12,
    };
    let next = start.spend(10).unwrap();
    assert_eq!(next.gold, 2);
    assert_eq!(next.earned, 12);
    assert!(next.spend(3).is_err());
    assert!(next.native_debit(3).is_err());
    assert_eq!(
        shared_gold_award(101, 1.0, 0.0, 0.0, 3, Award::ConvertedReward).unwrap(),
        33
    );
    assert_eq!(
        shared_gold_award(10, 1.2, 0.2, 0.2, 3, Award::ConvertedReward).unwrap(),
        4
    );
    assert_eq!(
        shared_gold_award(10, 1.3, 0.0, 0.2, 3, Award::World).unwrap(),
        15
    );
    assert!(start.credit(u32::MAX).is_err());
    assert!(shared_gold_award(10, f32::NAN, 0.0, 0.2, 3, Award::World).is_err());
}

#[test]
fn deferred_xp_keeps_normalized_integers_and_is_consumed_once_per_life() {
    let mut bank = DeferredExperience::default();
    assert_eq!(bank.credit(3, 77, 5).unwrap(), 5);
    assert_eq!(bank.credit(3, 77, 7).unwrap(), 12);
    assert_eq!(bank.credit(1, 88, 6).unwrap(), 6);
    assert_eq!(bank.take(3, 77), 12);
    assert_eq!(bank.take(3, 77), 0);
    bank.credit(3, 77, 8).unwrap();
    bank.retain(&[(1, 88), (3, 99)]);
    assert_eq!(bank.take(3, 77), 0);
    assert_eq!(bank.take(1, 88), 6);
    assert!(bank.credit(0, 1, 5).is_err());
    assert!(bank.credit(1, 0, 5).is_err());
    bank.credit(2, 22, u32::MAX).unwrap();
    assert!(bank.credit(2, 22, 1).is_err());
    assert_eq!(bank.take(2, 22), u32::MAX);
}

#[test]
fn five_native_participants_do_not_hit_historical_product_limit() {
    assert!(Session::new(1, 1, 1..=5, 20).is_ok());
    assert!((shared_xp_multiplier(vec![(1.1, 0.1); 5]).unwrap() - 1.1).abs() < 1e-6);
    assert_eq!(
        shared_gold_award(11, 1.0, 0., 0., 5, Award::ConvertedReward).unwrap(),
        2
    );
    let mut b = DeferredExperience::default();
    b.credit(5, 55, 7).unwrap();
    assert_eq!(b.take(5, 55), 7);
}

#[test]
fn pause_text_uses_character_names_for_reward_and_menu_waits() {
    use asx_gameplay::Phase;
    let names = |id| match id {
        1 => Some("Tibout".into()),
        3 => Some("Élyne".into()),
        _ => None,
    };
    let reward = pause_presentation(Some(PauseReason::Choices {
        kind: ChoiceKind::Reward, waiting: vec![3],
    }), Phase::Playing, names);
    assert_eq!(reward.detail, "Élyne is choosing a reward.");
    let menus = pause_presentation(Some(PauseReason::Menus { players: vec![3, 1] }), Phase::Playing, names);
    assert_eq!(menus.detail, "Élyne, Tibout are in the menu.");
    let unknown = pause_presentation(Some(PauseReason::Menus { players: vec![2] }), Phase::Playing, names);
    assert_eq!(unknown.detail, "Player 2 is in the menu.");
}

#[test]
fn pause_names_are_bounded_literal_utf8_and_invalid_names_fall_back() {
    use asx_gameplay::Phase;
    let reason = Some(PauseReason::Menus { players: vec![1, 2, 3, 4, 5] });
    let view = pause_presentation(reason, Phase::Playing, |id| Some(match id {
        1 => "|cffff0000Name".into(),
        2 => " ".into(),
        3 => "Bad\nName".into(),
        _ => "É".repeat(256),
    }));
    assert!(view.detail.starts_with("||cffff0000Name, Player 2, Player 3, "));
    assert!(view.detail.len() <= 512);
    assert!(!view.detail.chars().any(char::is_control));
}

#[test]
fn identical_baseline_characters_do_not_create_a_party_size_xp_multiplier(){
 for members in 1..=5 {
  let multiplier=shared_xp_multiplier((0..members).map(|_|(1.1,0.1))).unwrap();
  assert_eq!(multiplier,1.1);
  assert_eq!(shared_xp_award(100,multiplier).unwrap(),110);
 }
}

#[test]fn shared_gold_uses_best_armory_bonus_independent_of_collector(){
 let a=shared_gold_equipment_multiplier(1.1,0.1,[0.1,0.3]).unwrap();
 let b=shared_gold_equipment_multiplier(1.3,0.3,[0.1,0.3]).unwrap();
 assert!((a-1.3).abs()<1e-6);assert!((a-b).abs()<1e-6);
 assert_eq!(shared_gold_award(100,a,0.,0.,2,Award::World).unwrap(),130);
 let preserved=shared_gold_equipment_multiplier(1.75,0.1,[0.1,0.3]).unwrap();
 assert!((preserved-1.95).abs()<1e-6); // retain collector racial/fortune/jewelry contributions
 assert_eq!(shared_gold_equipment_multiplier(1.2,0.,[0.,0.]).unwrap(),1.2);
 assert!((shared_gold_equipment_multiplier(1.3,0.3,[0.3]).unwrap()-1.3).abs()<1e-6);
 assert!(shared_gold_equipment_multiplier(1.,0.,[]).is_err());
 assert!(shared_gold_equipment_multiplier(1.,0.,[f32::NAN]).is_err());
}

#[test]
fn voluntarily_skipped_rewards_credit_full_gold_for_any_party_size(){
    for members in 1..=5 {
        let amount=shared_gold_award(10,1.2,0.2,0.2,members,Award::SkippedReward).unwrap();
        assert_eq!(amount,12);
        let wallet=SharedWallet{gold:100,earned:100}.credit(amount).unwrap();
        assert_eq!(wallet,SharedWallet{gold:112,earned:112});
        // Ordinary converted rewards retain their previous policy.
        assert_eq!(shared_gold_award(10,1.2,0.2,0.2,members,Award::ConvertedReward).unwrap(),12/members as u32);
    }
    assert_eq!(shared_gold_award(1,1.5,0.,0.,2,Award::SkippedReward).unwrap(),2);
    assert_eq!(shared_gold_award(10,1.,0.,0.2,2,Award::SkippedReward).unwrap(),12);
    assert!(shared_gold_award(10,1.,0.,0.,0,Award::SkippedReward).is_err());
}
