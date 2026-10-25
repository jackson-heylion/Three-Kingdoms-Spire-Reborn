use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, Stage, content::Hero, event_stack::RunEvent,
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang, Hero::CaoCao, Hero::LuBu,
];

fn campaign(hero: Hero, ascension: u8) -> Game {
    let mut game = Game::new_with_identity(
        hero, 20261025, ascension, RunMode::Custom, "causal-test".into(),
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

#[test]
fn ten_campaigns_attribute_every_archived_event_to_one_real_action() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let game = campaign(hero, ascension);
            let original = game.to_json().unwrap();
            let restored = Game::from_json(&original).unwrap();
            let trace = restored.reconstruct_action_event_trace().unwrap();
            assert_eq!(trace.len(), 3, "{} ascension {ascension}", hero.name());
            assert_eq!(trace.iter().map(|t| t.revision).collect::<Vec<_>>(), vec![1, 2, 3]);
            assert_eq!(trace[0].stage_before, Stage::Map);
            assert_eq!(trace[1].stage_after, Stage::Combat);
            assert_eq!(trace[2].stage_before, Stage::Combat);
            assert!(trace.iter().any(|t| !t.events.is_empty()));
            let flattened: Vec<RunEvent> = trace.iter()
                .flat_map(|t| t.events.iter().cloned()).collect();
            assert_eq!(flattened, restored.event_history.as_ref().unwrap().clone());
            for pair in trace.windows(2) {
                assert_eq!(pair[0].rng_after, pair[1].rng_before);
                assert_eq!(pair[0].stage_after, pair[1].stage_before);
            }
            assert_eq!(trace.last().unwrap().rng_after, restored.rng_state);
            assert_eq!(restored.to_json().unwrap(), original, "trace must be read-only");
        }
    }
}

#[test]
fn causal_trace_rejects_event_edits_and_unattributed_archive_tail() {
    let game = campaign(Hero::GuanYu, 0);
    let mut modified = game.clone();
    modified.event_history.as_mut().unwrap()[0].event.amount += 1;
    assert!(modified.reconstruct_action_event_trace().unwrap_err()
        .contains("事件序列与存档档案不一致"));

    let mut missing = game.clone();
    missing.event_history.as_mut().unwrap().pop();
    assert!(missing.reconstruct_action_event_trace().is_err());

    let mut altered_state = game.clone();
    altered_state.gold += 1;
    assert!(altered_state.reconstruct_action_event_trace().unwrap_err()
        .contains("最终完整 Game"));
}

#[test]
fn causal_trace_never_fabricates_old_archives_or_truncated_actions() {
    let game = campaign(Hero::ZhugeLiang, 10);
    let mut old = game.clone();
    old.event_history = None;
    assert!(old.reconstruct_action_event_trace().unwrap_err()
        .contains("缺少完整战斗事件档案"));

    let mut truncated = game.clone();
    truncated.action_history.as_mut().unwrap().pop();
    assert!(truncated.reconstruct_action_event_trace().is_err());

    let mut no_actions = Game::new(Hero::LuBu, 12);
    no_actions.action_history = None;
    assert!(no_actions.reconstruct_action_event_trace().is_err());
}

#[test]
fn dynamic_listener_lifecycle_is_replayable_without_inventing_combat_events() {
    use three_kingdoms_spire_reborn::event_stack::{Listener, Response};

    let mut game = Game::new_with_identity(
        Hero::GuanYu, 811, 0, RunMode::Custom, "listener-causality".into(),
    );
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::RegisterReaction {
        listener: Listener::once("causal-ward", "weak", Response::Reduce(1)),
    }).unwrap();
    assert_eq!(game.battle.as_ref().unwrap().reactions.listeners.len(), 1);
    game.apply(Action::UnregisterReaction { id: "causal-ward".into() }).unwrap();
    assert!(game.battle.as_ref().unwrap().reactions.listeners.is_empty());
    game.apply(Action::EndTurn).unwrap();

    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    let trace = restored.reconstruct_action_event_trace().unwrap();
    assert_eq!(trace.len(), 4);
    assert!(trace[1].events.is_empty(), "register must not forge a settled combat event");
    assert!(trace[2].events.is_empty(), "unregister must not forge a settled combat event");
    assert_eq!(trace[1].stage_after, Stage::Combat);
    assert_eq!(trace[2].rng_before, trace[2].rng_after);
    assert_eq!(trace[3].rng_after, restored.rng_state);
}
