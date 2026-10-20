//! Regression: dynamic listeners must never prevent a defensive card from
//! installing its built-in reaction or crash the process at full capacity.
use three_kingdoms_spire_reborn::{
    Action, Card, Game, RunMode, Stage,
    content::Hero,
    event_stack::{Listener, Response, Stack},
};

#[test]
fn public_quota_and_builtin_quota_are_independent() {
    let mut stack = Stack::default();
    for index in 0..16 {
        let listener = Listener::once(
            &format!("custom-{index:02}"), "weak", Response::Reduce(1),
        );
        stack.install_custom(listener).expect("16 public slots");
    }
    let original = stack.clone();
    assert_eq!(
        stack.install_custom(Listener::once("extra", "weak", Response::Cancel)),
        Err("reaction stack full"),
    );
    assert_eq!(stack, original, "rejected listener must not mutate the stack");

    for id in ["hold_line:weak", "kongcheng:vulnerable", "qiaoying:weak", "zhongyi:weak"] {
        assert_eq!(
            stack.install_custom(Listener::once(id, "weak", Response::Cancel)),
            Err("reserved builtin reaction id"),
        );
        assert_eq!(stack, original, "reserved IDs cannot overwrite a card");
    }
    for (id, source, effect) in [
        ("hold_line:weak", "weak", Response::Cancel),
        ("kongcheng:vulnerable", "vulnerable", Response::Reduce(1)),
        ("qiaoying:weak", "weak", Response::Reduce(1)),
        ("zhongyi:weak", "weak", Response::Replace(1)),
    ] {
        stack.install(Listener::once(id, source, effect)).unwrap();
    }
    assert_eq!(stack.listeners.len(), 20);
    assert!(stack.valid());
    let serialized = serde_json::to_string(&stack).unwrap();
    let roundtrip: Stack = serde_json::from_str(&serialized).unwrap();
    assert_eq!(stack, roundtrip);
    // Replaying a card refreshes its own pending defense even at full occupancy.
    stack.install(Listener::once("hold_line:weak", "weak", Response::Cancel))
        .expect("a repeated defense must replace its own slot");
    assert_eq!(stack.listeners.len(), 20);
}

#[test]
fn played_defense_at_sixteen_custom_listeners_is_atomic_and_saveable() {
    let mut game = Game::new_with_mode(Hero::GuanYu, 20261020, 0, RunMode::Custom);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.stage, Stage::Combat);
    for index in 0..16 {
        game.apply(Action::RegisterReaction {
            listener: Listener::once(
                &format!("custom-{index:02}"), "weak", Response::Reduce(1),
            ),
        }).unwrap();
    }
    let unchanged = game.to_json().unwrap();
    for listener in [
        Listener::once("custom-16", "weak", Response::Cancel),
        Listener::once("hold_line:weak", "weak", Response::Cancel),
    ] {
        assert!(game.apply(Action::RegisterReaction { listener }).is_err());
        assert_eq!(game.to_json().unwrap(), unchanged,
            "failed registration changed game, history, or RNG");
    }
    assert_eq!(game.verify_action_replay().unwrap(), 17);
    assert_eq!(game.reconstruct_event_history().unwrap(),
        game.event_history.clone().unwrap());

    // Owned defensive card in an otherwise ordinary live combat; only the
    // current hand is injected so this is not a natural draft/balance result.
    let battle = game.battle.as_mut().unwrap();
    battle.hand = vec![Card::new("hold_line")];
    battle.energy = 5;
    game.apply(Action::Play { index: 1, target: 1 })
        .expect("playing a defensive card with a full custom stack must not panic");
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.reactions.listeners.len(), 17);
    assert!(battle.reactions.listeners.iter().any(|r| r.id == "hold_line:weak"));
    assert!(battle.reactions.valid());
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}
