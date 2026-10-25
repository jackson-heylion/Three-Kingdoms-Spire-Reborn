//! Causal RNG metadata must name a real synchronous observer, never a
//! neighboring event guessed from time order. This is read-only replay evidence.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode,
    content::Hero,
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

#[test]
fn ten_persisted_campaigns_have_only_real_prior_same_action_rng_parents() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let mut game = Game::new_with_identity(
                hero, 20261025, ascension, RunMode::Custom, "rng-parent".into(),
            );
            for action in [
                Action::SetDoctrine { id: "auto".into() },
                Action::Choose { index: 1 },
                Action::EndTurn,
            ] {
                game.apply(action).unwrap();
            }
            let saved = game.to_json().unwrap();
            let restored = Game::from_json(&saved).unwrap();
            let archive = restored.event_history.as_ref().unwrap();
            let actions = restored.reconstruct_rng_draw_attribution().unwrap();
            let mut calls = 0;
            for action in actions {
                let start = action.event_start.unwrap();
                let end = action.event_end.unwrap();
                for call in action.calls {
                    calls += 1;
                    let before = call.event_cursor_before.unwrap();
                    assert!((start..=end).contains(&before));
                    if let Some(parent) = call.dispatch_parent {
                        assert!(archive[start..before].iter().any(|event|
                            event.encounter == parent.encounter
                                && event.event.seq == parent.seq),
                            "an RNG parent must be an earlier event in the same action");
                    }
                }
            }
            assert!(calls > 0, "a real first-floor battle must consume RNG");
            assert_eq!(restored.to_json().unwrap(), saved);
            assert!(!saved.contains("dispatch_parent"),
                "RNG observer attribution is diagnostic, never saved");
        }
    }
}

#[test]
fn incomplete_or_rewritten_history_cannot_claim_rng_observer_causality() {
    let mut game = Game::new(Hero::ZhugeLiang, 99123);
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    assert!(game.reconstruct_rng_draw_attribution().is_ok());

    let mut missing = game.clone();
    missing.event_history = None;
    assert!(missing.reconstruct_rng_draw_attribution().is_err());

    let mut forged = game.clone();
    forged.event_history.as_mut().unwrap()[0].event.amount += 1;
    assert!(forged.reconstruct_rng_draw_attribution().is_err());

    let mut truncated = game.clone();
    truncated.action_history.as_mut().unwrap().pop();
    assert!(truncated.reconstruct_rng_draw_attribution().is_err());
}
