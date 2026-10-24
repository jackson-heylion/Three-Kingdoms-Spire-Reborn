use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero,
    resource_events::{Resources, ResourceEvent, project},
};

#[test]
fn authoritative_resources_rebuild_without_actions_or_rng() {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 20261024, 0, RunMode::Standard);
    assert_eq!(game.audit_resources().unwrap(), 1);
    game.apply(Action::SetVariants { act1: 0, act2: 0, act3: 0 }).unwrap();
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.audit_resources().unwrap(), game.revision as usize + 1);
    game.verify_action_replay().unwrap();
    let loaded = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(loaded.audit_resources().unwrap(), game.audit_resources().unwrap());
    let mut without_actions = loaded.clone();
    without_actions.action_history = None;
    without_actions.event_history = None;
    assert_eq!(without_actions.audit_resources().unwrap(), game.revision as usize + 1);

    let intact = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
    assert_eq!(game.to_json().unwrap(), intact);
}

#[test]
fn rejects_gaps_duplicates_invalid_states_and_overflow() {
    let start = ResourceEvent::Started {
        state: Resources { hp: 60, max_hp: 70, gold: 90 },
    };
    let changed = ResourceEvent::Changed {
        revision: 1, hp_delta: -10, max_hp_delta: 5, gold_delta: -40,
    };
    assert_eq!(project(&[start.clone(), changed.clone()]).unwrap(),
        (Resources { hp: 50, max_hp: 75, gold: 50 }, 1));
    assert!(project(&[]).is_err());
    assert!(project(&[changed.clone()]).is_err());
    assert!(project(&[start.clone(), start.clone()]).is_err());
    assert!(project(&[start.clone(), changed.clone(), changed.clone()]).is_err());
    assert!(project(&[start.clone(), ResourceEvent::Changed {
        revision: 2, hp_delta: 0, max_hp_delta: 0, gold_delta: 0,
    }]).is_err());
    assert!(project(&[start.clone(), ResourceEvent::Changed {
        revision: 1, hp_delta: 100, max_hp_delta: 0, gold_delta: 0,
    }]).is_err());
    assert!(project(&[start.clone(), ResourceEvent::Changed {
        revision: 1, hp_delta: 0, max_hp_delta: 0, gold_delta: i64::MAX,
    }]).is_err());
    assert!(project(&[start, ResourceEvent::Changed {
        revision: 1, hp_delta: 0, max_hp_delta: -70, gold_delta: 0,
    }]).is_err());
}

#[test]
fn tampered_snapshot_is_detected_and_legacy_save_stays_unknown() {
    let mut game = Game::new(Hero::GuanYu, 1234);
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.audit_resources().unwrap();
    let mut forged = game.clone();
    forged.gold += 1;
    assert!(forged.audit_resources().is_err());
    let mut forged = game.clone();
    forged.resource_history.as_mut().unwrap().pop();
    assert!(forged.audit_resources().is_err());

    let mut old_json: serde_json::Value = serde_json::from_str(&game.to_json().unwrap()).unwrap();
    old_json.as_object_mut().unwrap().remove("resource_history");
    let mut legacy = Game::from_json(&old_json.to_string()).unwrap();
    assert!(legacy.audit_resources().is_err());
    legacy.verify_action_replay().unwrap();
    legacy.apply(Action::EndTurn).unwrap();
    assert!(legacy.resource_history.is_none());
    assert!(legacy.audit_resources().is_err());
}
