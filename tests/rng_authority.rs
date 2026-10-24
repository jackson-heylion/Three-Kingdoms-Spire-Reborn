use three_kingdoms_spire_reborn::{
    Action, Game, content::Hero,
    rng_events::{RngEvent, RngProjector, project},
};

#[test]
fn committed_rng_boundaries_round_trip_and_replay() {
    let mut game = Game::new(Hero::GuanYu, 1234);
    assert_eq!(game.audit_rng().unwrap(), 1);
    assert_eq!(game.project_authority_boundary(0).unwrap()["rng_state"], game.rng_state);
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.audit_rng().unwrap(), game.revision as usize + 1);
    assert_eq!(game.project_authority_boundary(game.revision).unwrap()["rng_state"], game.rng_state);
    game.verify_action_replay().unwrap();
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(restored.audit_authority_bundle().unwrap(), 3);
    let mut without_actions = restored.clone();
    without_actions.action_history = None;
    without_actions.event_history = None;
    assert_eq!(without_actions.project_authority_boundary(1).unwrap()["rng_state"],
        game.rng_history.as_ref().unwrap().get(1).map(|event| match event {
            RngEvent::Committed { state, .. } => *state,
            _ => unreachable!(),
        }).unwrap());
}

#[test]
fn rejected_action_preserves_rng_journal_and_state() {
    let mut game = Game::new(Hero::ZhugeLiang, 20261025);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    assert_eq!(game.audit_rng().unwrap(), 1);
}

#[test]
fn reducer_rejects_gaps_zero_and_duplicates_atomically() {
    let mut projector = RngProjector::default();
    projector.push(&RngEvent::Started { state: 1 }).unwrap();
    projector.push(&RngEvent::Committed { revision: 1, state: 42 }).unwrap();
    let committed = projector.clone();
    for bad in [
        RngEvent::Committed { revision: 1, state: 43 },
        RngEvent::Committed { revision: 3, state: 43 },
        RngEvent::Committed { revision: 2, state: 0 },
        RngEvent::Started { state: 1 },
    ] {
        assert!(projector.push(&bad).is_err());
        assert_eq!(projector, committed);
    }
    projector.push(&RngEvent::Committed { revision: 2, state: 42 }).unwrap();
    assert_eq!(projector.state().unwrap(), (42, 2));
    assert_eq!(project(&[
        RngEvent::Started { state: 1 },
        RngEvent::Committed { revision: 1, state: 42 },
        RngEvent::Committed { revision: 2, state: 42 },
    ]).unwrap(), (42, 2));
}

#[test]
fn legacy_save_stays_playable_but_is_not_certified() {
    let mut game = Game::new(Hero::CaoCao, 77);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&game.to_json().unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("rng_history");
    let old = Game::from_json(&json.to_string()).unwrap();
    assert!(old.audit_rng().unwrap_err().contains("legacy"));
    assert!(old.audit_authority_bundle().unwrap_err().contains("legacy"));
    assert!(old.verify_action_replay().is_ok());
    assert_eq!(old.rng_state, game.rng_state);
}
