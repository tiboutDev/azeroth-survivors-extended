use asx_gameplay::{offensive::*, spells::{Events, SpellEvent}, support::Spell};
fn command(effect: Effect) -> Command { Command { enemy: 17, spell: 853, effect } }
#[test]
fn effects_keep_requests_distinct_from_commands_and_success() {
    let mut events = Events::default();
    let cast = events.begin(2, 2, Spell::HolyLight).unwrap();
    let cmd = command(Effect::ActionControl { seconds: 3. });
    assert_eq!(events.offensive_request(Request { observed_by: 2, command: cmd }).unwrap(), cmd);
    assert!(matches!(events.recent.back(), Some(SpellEvent::OffensiveRequested(_))));
    events.offensive_command(cmd).unwrap();
    assert!(matches!(events.recent.back(), Some(SpellEvent::OffensivePrepared(_))));
    assert_eq!((events.effect_requests, events.effect_commands, events.completed), (1, 1, 0));
    assert!(events.get(cast).is_some());
}
#[test]
fn invalid_requests_cannot_partially_mutate_the_journal() {
    let mut events = Events::default();
    for effect in [Effect::MovementRate { strength: f32::NAN, seconds: 3. }, Effect::MovementRate { strength: 1.1, seconds: 3. }, Effect::ActionControl { seconds: -1. }, Effect::MovementControl { seconds: f32::INFINITY }, Effect::LegacyTimed { strength: 0.5, seconds: 3601. }] {
        assert!(events.offensive_request(Request { observed_by: 2, command: command(effect) }).is_err());
        assert!(events.offensive_command(command(effect)).is_err());
    }
    let cmd = command(Effect::ActionControl { seconds: 3. });
    assert!(events.offensive_request(Request { observed_by: 0, command: cmd }).is_err());
    assert!(events.offensive_command(Command { enemy: 0, ..cmd }).is_err());
    assert_eq!((events.effect_requests, events.effect_commands, events.recent.len()), (0, 0, 0));
}
#[test]
fn repeated_distinct_requests_are_not_deduplicated_by_target_or_spell() {
    let mut events = Events::default();
    for _ in 0..1000 { events.offensive_request(Request { observed_by: 2, command: command(Effect::ActionControl { seconds: 3. }) }).unwrap(); }
    assert_eq!(events.effect_requests, 1000); assert_eq!(events.recent.len(), 64); assert_eq!(events.active(), 0);
}
