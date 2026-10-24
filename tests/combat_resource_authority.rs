use three_kingdoms_spire_reborn::{
    Action, Game, content::Hero,
    combat_resource_events::{
        BattleResourceChange, BattleResourceEvent, BattleResourceProjector,
        BattleResources, project, transition,
    },
};

#[test]
fn combat_resource_journal_matches_game_without_action_or_rng_replay() {
    let mut game = Game::new(Hero::ZhugeLiang, 202610241931);
    assert_eq!(game.audit_battle_resources().unwrap(), 1);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.battle.is_some());
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.audit_battle_resources().unwrap(), game.revision as usize + 1);
    game.verify_action_replay().unwrap();

    let loaded = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(loaded.audit_battle_resources().unwrap(), game.audit_battle_resources().unwrap());
    let mut without_actions = loaded.clone();
    without_actions.action_history = None;
    without_actions.event_history = None;
    without_actions.resource_history = None;
    assert_eq!(without_actions.audit_battle_resources().unwrap(), game.revision as usize + 1);

    let mut tampered = game.clone();
    tampered.battle.as_mut().unwrap().energy += 1;
    assert!(tampered.audit_battle_resources().is_err());
    let mut truncated = game.clone();
    truncated.battle_resource_history.as_mut().unwrap().pop();
    assert!(truncated.audit_battle_resources().is_err());

    let intact = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
    assert_eq!(game.to_json().unwrap(), intact);
}

#[test]
fn combat_resource_lifecycle_rebuilds_across_encounters() {
    let first = BattleResources { floor: 1, turn: 1, energy: 3, block: 4 };
    let later = BattleResources { floor: 1, turn: 2, energy: 2, block: 0 };
    let second = BattleResources { floor: 2, turn: 1, energy: 4, block: 6 };
    let history = vec![
        BattleResourceEvent::Started,
        transition(None, None, 1).unwrap(),
        transition(None, Some(first), 2).unwrap(),
        transition(Some(first), Some(later), 3).unwrap(),
        transition(Some(later), None, 4).unwrap(),
        transition(None, Some(second), 5).unwrap(),
    ];
    let mut stream = BattleResourceProjector::default();
    for event in &history {
        stream.push(event).unwrap();
    }
    assert_eq!(stream.state().unwrap(), (Some(second), 5));
    assert_eq!(project(&history).unwrap(), stream.state().unwrap());
    let json = serde_json::to_string(&history).unwrap();
    let decoded: Vec<BattleResourceEvent> = serde_json::from_str(&json).unwrap();
    assert_eq!(project(&decoded).unwrap(), (Some(second), 5));
}

#[test]
fn invalid_combat_transitions_fail_atomically() {
    let first = BattleResources { floor: 1, turn: 1, energy: 3, block: 4 };
    let mut stream = BattleResourceProjector::default();
    assert!(stream.push(&transition(None, None, 1).unwrap()).is_err());
    assert_eq!(stream, BattleResourceProjector::default());
    stream.push(&BattleResourceEvent::Started).unwrap();
    stream.push(&transition(None, Some(first), 1).unwrap()).unwrap();
    let before = stream.clone();
    let invalid = [
        BattleResourceEvent::Started,
        transition(Some(first), None, 3).unwrap(),
        BattleResourceEvent::Changed {
            revision: 2,
            change: BattleResourceChange::Update {
                energy_delta: i64::MAX, block_delta: 0, turns_elapsed: 0,
            },
        },
        BattleResourceEvent::Changed {
            revision: 2,
            change: BattleResourceChange::Idle,
        },
        BattleResourceEvent::Changed {
            revision: 2,
            change: BattleResourceChange::Enter { state: first },
        },
    ];
    for event in &invalid {
        assert!(stream.push(event).is_err());
        assert_eq!(stream, before);
    }
    assert!(transition(Some(first), Some(BattleResources { turn: 0, ..first }), 2).is_err());
    assert!(transition(Some(first), Some(BattleResources { floor: 2, ..first }), 2).is_err());
    stream.push(&transition(Some(first), None, 2).unwrap()).unwrap();
    assert_eq!(stream.state().unwrap(), (None, 2));
    assert!(stream.push(&transition(Some(first), None, 3).unwrap()).is_err());
}

#[test]
fn legacy_v2_remains_playable_without_fabricated_combat_history() {
    let mut game = Game::new(Hero::GuanYu, 1234);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&game.to_json().unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("battle_resource_history");
    let mut legacy = Game::from_json(&json.to_string()).unwrap();
    assert!(legacy.audit_battle_resources().is_err());
    legacy.verify_action_replay().unwrap();
    legacy.apply(Action::EndTurn).unwrap();
    assert!(legacy.battle_resource_history.is_none());
    assert!(legacy.audit_battle_resources().is_err());
}
