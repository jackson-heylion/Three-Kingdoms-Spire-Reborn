//! Damage listeners must target the exact runtime foe slot, not a parseable
//! alias such as foe:01. Otherwise the rule persists but can never fire.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, Stage,
    combat::EventKind,
    content::Hero,
    event_stack::{Listener, Response},
};

fn damage_rule(id: &str, target: &str) -> Listener {
    Listener {
        id: id.into(),
        kind: EventKind::DamageDealt,
        source: "hero".into(),
        target: Some(target.into()),
        priority: 0,
        charges: 1,
        effect: Response::Reduce(1),
    }
}

fn combat() -> Game {
    let mut game = Game::new_with_mode(Hero::GuanYu, 20261023, 0, RunMode::Custom);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.stage, Stage::Combat);
    game
}

#[test]
fn malformed_damage_targets_are_rejected_without_mutating_the_campaign() {
    let mut game = combat();
    let before = game.to_json().unwrap();
    for (index, bad) in [
        "foe:", "foe:00", "foe:01", "foe:+1", "foe:-1",
        "foe:4", "foe:999", "foe:0suffix", "foe: 1",
    ].into_iter().enumerate() {
        let result = game.apply(Action::RegisterReaction {
            listener: damage_rule(&format!("invalid_{index}"), bad),
        });
        assert!(result.is_err(), "accepted noncanonical damage target {bad}");
        assert_eq!(
            game.to_json().unwrap(), before,
            "failed registration changed the campaign for {bad}"
        );
    }
}

#[test]
fn canonical_damage_targets_remain_persistent_and_replayable() {
    for target in ["foe:0", "foe:1", "foe:2", "foe:3"] {
        let mut game = combat();
        game.apply(Action::RegisterReaction {
            listener: damage_rule("canonical_damage", target),
        }).unwrap();
        let saved = game.to_json().unwrap();
        let loaded = Game::from_json(&saved).unwrap();
        assert_eq!(loaded.to_json().unwrap(), saved);
        assert_eq!(loaded.verify_action_replay().unwrap(), 2);
        assert_eq!(
            loaded.reconstruct_event_history().unwrap(),
            loaded.event_history.clone().unwrap(),
            "event replay diverged for {target}"
        );
    }
}
