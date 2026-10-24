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
    assert!(project(std::slice::from_ref(&changed)).is_err());
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

#[test]
fn resource_stream_matches_batch_and_committed_game() {
    use three_kingdoms_spire_reborn::resource_events::ResourceProjector;

    let mut game = Game::new(Hero::ZhugeLiang, 202610241832);
    game.apply(Action::SetVariants { act1: 0, act2: 0, act3: 0 }).unwrap();
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    let history = game.resource_history.as_ref().unwrap();
    let mut stream = ResourceProjector::default();
    assert!(stream.state().is_err());
    assert!(stream.checkpoint().is_err());
    for event in history {
        stream.push(event).unwrap();
    }
    assert_eq!(stream.state().unwrap(), project(history).unwrap());
    assert_eq!(stream.state().unwrap().1, game.revision);
    assert_eq!(stream.state().unwrap().0.hp, game.hp);
    assert_eq!(stream.state().unwrap().0.gold, game.gold);
}

#[test]
fn resource_stream_rejections_do_not_advance_state_or_cursor() {
    use three_kingdoms_spire_reborn::resource_events::ResourceProjector;

    let start = ResourceEvent::Started {
        state: Resources { hp: 60, max_hp: 70, gold: 90 },
    };
    let mut stream = ResourceProjector::default();
    assert!(stream.push(&ResourceEvent::Changed {
        revision: 1, hp_delta: 0, max_hp_delta: 0, gold_delta: 0,
    }).is_err());
    assert_eq!(stream, ResourceProjector::default());
    stream.push(&start).unwrap();
    let saved = stream.clone();
    assert!(stream.push(&start).is_err());
    assert_eq!(stream, saved);
    assert!(stream.push(&ResourceEvent::Changed {
        revision: 2, hp_delta: 0, max_hp_delta: 0, gold_delta: 0,
    }).is_err());
    assert_eq!(stream, saved);
    assert!(stream.push(&ResourceEvent::Changed {
        revision: 1, hp_delta: 100, max_hp_delta: 0, gold_delta: 0,
    }).is_err());
    assert_eq!(stream, saved);
    stream.push(&ResourceEvent::Changed {
        revision: 1, hp_delta: -10, max_hp_delta: 0, gold_delta: -5,
    }).unwrap();
    assert_eq!(stream.state().unwrap(),
        (Resources { hp: 50, max_hp: 70, gold: 85 }, 1));
}

#[test]
fn resource_checkpoint_json_resume_rejects_stale_or_invalid_state() {
    use three_kingdoms_spire_reborn::resource_events::{
        RESOURCE_CHECKPOINT_VERSION, ResourceCheckpoint, ResourceProjector,
    };

    let start = ResourceEvent::Started {
        state: Resources { hp: 60, max_hp: 70, gold: 90 },
    };
    let first = ResourceEvent::Changed {
        revision: 1, hp_delta: -10, max_hp_delta: 5, gold_delta: -40,
    };
    let next = ResourceEvent::Changed {
        revision: 2, hp_delta: 5, max_hp_delta: 0, gold_delta: 10,
    };
    let mut stream = ResourceProjector::default();
    stream.push(&start).unwrap();
    stream.push(&first).unwrap();
    let json = serde_json::to_string(&stream.checkpoint().unwrap()).unwrap();
    let checkpoint: ResourceCheckpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(checkpoint.version, RESOURCE_CHECKPOINT_VERSION);
    let mut restored = ResourceProjector::resume(checkpoint.clone()).unwrap();
    let before = restored.clone();
    assert!(restored.push(&first).is_err());
    assert_eq!(restored, before);
    restored.push(&next).unwrap();
    assert_eq!(restored.state().unwrap(), project(&[start, first, next]).unwrap());

    let mut invalid = checkpoint.clone();
    invalid.version += 1;
    assert!(ResourceProjector::resume(invalid).is_err());
    let mut invalid = checkpoint;
    invalid.state.gold = -1;
    assert!(ResourceProjector::resume(invalid).is_err());
    assert!(serde_json::from_str::<ResourceCheckpoint>(
        r#"{"version":1,"revision":1,"state":{"hp":50,"max_hp":75,"gold":50},"unknown":1}"#
    ).is_err());
}
