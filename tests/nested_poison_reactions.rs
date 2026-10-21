//! The nested toxic reversal application is a distinct, replayable listener
//! target. Settlement/priority tests live in src/lib.rs for private combat access.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero, combat::EventKind,
    event_stack::{Listener, Response},
};

fn bonus_listener(id: &str, target: &str) -> Listener {
    Listener {
        id: id.into(),
        kind: EventKind::StatusApplied,
        source: "poison:toxic_reversal".into(),
        target: Some(target.into()),
        priority: -2,
        charges: 2,
        effect: Response::Reduce(1),
    }
}

#[test]
fn bonus_interception_registration_survives_save_and_action_event_replay() {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 925145, 0, RunMode::Custom);
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::RegisterReaction {
        listener: bonus_listener("nested_guard", "foe:0"),
    }).unwrap();
    assert_eq!(game.verify_action_replay().unwrap(), 2);
    let raw = game.to_json().unwrap();
    let loaded = Game::from_json(&raw).unwrap();
    assert_eq!(loaded.to_json().unwrap(), raw);
    assert_eq!(loaded.verify_action_replay().unwrap(), 2);
    assert_eq!(loaded.reconstruct_event_history().unwrap(),
        loaded.event_history.clone().unwrap());
    assert_eq!(loaded.battle.as_ref().unwrap().reactions.listeners[0].source,
        "poison:toxic_reversal");
}

#[test]
fn nested_source_cannot_be_claimed_for_hero_or_noncanonical_foe() {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 925145, 0, RunMode::Custom);
    game.apply(Action::Choose { index: 1 }).unwrap();
    for target in ["hero", "foe:01", "foe:+1", "foe:4"] {
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::RegisterReaction {
            listener: bonus_listener("invalid", target),
        }).is_err(), "{target}");
        assert_eq!(game.to_json().unwrap(), before, "{target}");
    }
    let before = game.to_json().unwrap();
    let mut invalid = bonus_listener("unknown", "foe:0");
    invalid.source = "poison:imaginary".into();
    assert!(game.apply(Action::RegisterReaction { listener: invalid }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
}

#[test]
fn ranked_mode_rejects_secondary_status_interception() {
    let mut game = Game::new(Hero::ZhugeLiang, 925145);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::RegisterReaction {
        listener: bonus_listener("ranked", "foe:0"),
    }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
}
