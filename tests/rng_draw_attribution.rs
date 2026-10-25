use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero, rng_events::{RngEvent, RngDrawAttribution},
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

fn campaign(hero: Hero, ascension: u8) -> Game {
    let mut game = Game::new_with_identity(
        hero, 20261025, ascension, RunMode::Custom, "rng-draw-attribution".into(),
    );
    for action in [
        Action::SetDoctrine { id: "auto".into() },
        Action::Choose { index: 1 },
        Action::EndTurn,
    ] {
        game.apply(action).unwrap();
    }
    game
}

fn advance(mut state: u64) -> u64 {
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;
    state
}

#[test]
fn five_heroes_two_difficulties_attribute_each_committed_rng_draw() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let game = campaign(hero, ascension);
            let saved = game.to_json().unwrap();
            let restored = Game::from_json(&saved).unwrap();
            let trace: Vec<RngDrawAttribution> =
                restored.reconstruct_rng_draw_attribution().unwrap();
            assert_eq!(trace.len(), 3, "{} ascension {ascension}", hero.name());
            assert_eq!(trace.iter().map(|item| item.revision).collect::<Vec<_>>(),
                vec![1, 2, 3]);
            assert_eq!(trace[0].draws, 0, "doctrine selection consumes no RNG");
            assert!(trace[1].draws > 0, "first battle shuffles a nontrivial deck");
            for (index, item) in trace.iter().enumerate() {
                let mut expected = item.state_before;
                for _ in 0..item.draws {
                    expected = advance(expected);
                }
                assert_eq!(expected, item.state_after);
                assert_eq!(game.rng_history.as_ref().unwrap()[index + 1],
                    RngEvent::Committed {
                        revision: item.revision,
                        state: item.state_after,
                    });
            }
            for pair in trace.windows(2) {
                assert_eq!(pair[0].state_after, pair[1].state_before);
            }
            assert_eq!(trace.last().unwrap().state_after, game.rng_state);
            assert_eq!(restored.to_json().unwrap(), saved, "audit is read-only");
        }
    }
}

#[test]
fn rejected_action_does_not_advance_rng_or_create_a_false_attribution() {
    let mut game = campaign(Hero::GuanYu, 0);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: 999, target: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    assert_eq!(game.reconstruct_rng_draw_attribution().unwrap().len(), 3);
}

#[test]
fn missing_or_tampered_rng_and_action_journals_are_rejected() {
    let game = campaign(Hero::ZhugeLiang, 10);
    let mut missing_rng = game.clone();
    missing_rng.rng_history = None;
    assert!(missing_rng.reconstruct_rng_draw_attribution().is_err());

    let mut missing_actions = game.clone();
    missing_actions.action_history = None;
    assert!(missing_actions.reconstruct_rng_draw_attribution().is_err());

    let mut truncated = game.clone();
    truncated.rng_history.as_mut().unwrap().pop();
    assert!(truncated.reconstruct_rng_draw_attribution().unwrap_err()
        .contains("do not align"));

    let mut rewritten = game.clone();
    if let RngEvent::Committed { state, .. } =
        &mut rewritten.rng_history.as_mut().unwrap()[2]
    {
        *state = state.wrapping_add(1);
    }
    assert!(rewritten.reconstruct_rng_draw_attribution().unwrap_err()
        .contains("boundary differs"));

    let mut modified_snapshot = game.clone();
    modified_snapshot.gold += 1;
    assert!(modified_snapshot.reconstruct_rng_draw_attribution().unwrap_err()
        .contains("final complete Game"));

    let mut old = game.clone();
    old.action_history = None;
    old.rng_history = None;
    assert!(old.reconstruct_rng_draw_attribution().is_err());
}
