use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero,
    combat::{PowerId, PowerStack},
    combat_effect_events::{
        BattleEffects, CombatEffectChange, CombatEffectEvent, CombatEffectProjector, project,
    },
    event_stack::{Listener, Response, Stack},
};

fn active_custom() -> Game {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 202610250034, 0, RunMode::Custom);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.battle.is_some());
    game
}

#[test]
fn action_boundaries_rebuild_registered_and_removed_listeners() {
    let mut game = active_custom();
    assert_eq!(game.audit_combat_effects().unwrap(), 2);
    let listener = Listener::once("audit-weak", "weak", Response::Reduce(1));
    game.apply(Action::RegisterReaction { listener: listener.clone() }).unwrap();
    let (effects, revision) = project(game.combat_effect_history.as_ref().unwrap()).unwrap();
    assert_eq!(revision, game.revision);
    assert_eq!(effects.unwrap().reactions.listeners, vec![listener]);
    assert_eq!(game.audit_authority_bundle().unwrap(), 3);

    game.apply(Action::UnregisterReaction { id: "audit-weak".into() }).unwrap();
    assert_eq!(game.audit_combat_effects().unwrap(), 4);
    assert_eq!(game.audit_authority_bundle().unwrap(), 4);
    game.verify_action_replay().unwrap();
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(restored.audit_authority_bundle().unwrap(), 4);
    restored.verify_action_replay().unwrap();
}

#[test]
fn power_stacks_are_projected_and_invalid_events_are_atomic() {
    let empty = BattleEffects {
        floor: 1, turn: 1, powers: vec![], reactions: Stack::default(),
    };
    let powered = BattleEffects {
        powers: vec![PowerStack { id: PowerId::OathGuard, stacks: 2 }],
        ..empty.clone()
    };
    let mut projector = CombatEffectProjector::default();
    projector.push(&CombatEffectEvent::Started).unwrap();
    projector.push(&CombatEffectEvent::Changed {
        revision: 1, change: CombatEffectChange::Enter { state: empty },
    }).unwrap();
    projector.push(&CombatEffectEvent::Changed {
        revision: 2, change: CombatEffectChange::Update { state: powered.clone() },
    }).unwrap();
    let committed = projector.clone();

    let invalid = BattleEffects {
        powers: vec![
            PowerStack { id: PowerId::OathGuard, stacks: 1 },
            PowerStack { id: PowerId::OathGuard, stacks: 3 },
        ],
        ..powered.clone()
    };
    assert!(projector.push(&CombatEffectEvent::Changed {
        revision: 3, change: CombatEffectChange::Update { state: invalid },
    }).is_err());
    assert_eq!(projector, committed);

    let mut invalid_listener = powered.clone();
    invalid_listener.reactions.listeners.push(Listener {
        charges: 0, ..Listener::once("invalid", "weak", Response::Cancel)
    });
    assert!(projector.push(&CombatEffectEvent::Changed {
        revision: 3, change: CombatEffectChange::Update { state: invalid_listener },
    }).is_err());
    assert_eq!(projector, committed);
    projector.push(&CombatEffectEvent::Changed {
        revision: 3, change: CombatEffectChange::Unchanged,
    }).unwrap();
    assert_eq!(projector.state().unwrap(), (Some(powered), 3));
}

#[test]
fn intermediate_effect_turn_forgery_is_caught_by_cross_journal_audit() {
    let mut game = active_custom();
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.audit_authority_bundle().unwrap(), 3);
    let mut tampered = game.clone();
    match &mut tampered.combat_effect_history.as_mut().unwrap()[1] {
        CombatEffectEvent::Changed {
            change: CombatEffectChange::Enter { state }, ..
        } => state.turn += 1,
        _ => panic!("expected first combat effect entry"),
    }
    // The isolated effect journal still has a legal, unchanged final state.
    assert!(tampered.audit_combat_effects().is_ok());
    assert!(tampered.audit_authority_bundle().unwrap_err().contains("boundary 1"));
}

#[test]
fn missing_legacy_journal_does_not_get_fabricated() {
    let game = active_custom();
    let mut value: serde_json::Value = serde_json::from_str(&game.to_json().unwrap()).unwrap();
    value.as_object_mut().unwrap().remove("combat_effect_history");
    let legacy = Game::from_json(&value.to_string()).unwrap();
    assert!(legacy.combat_effect_history.is_none());
    assert!(legacy.audit_combat_effects().unwrap_err().contains("legacy"));
    assert!(legacy.audit_authority_bundle().unwrap_err().contains("legacy"));
    // Action replay must preserve absence rather than invent a missing history.
    legacy.verify_action_replay().unwrap();
    legacy.reconstruct_event_history().unwrap();
}

#[test]
fn rejected_action_keeps_all_five_authorities_unchanged() {
    let mut game = active_custom();
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::UnregisterReaction { id: "absent".into() }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    assert_eq!(game.audit_authority_bundle().unwrap(), 2);
}
