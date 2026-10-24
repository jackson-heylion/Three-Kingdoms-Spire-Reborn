use three_kingdoms_spire_reborn::{
    Action, Game, content::Hero,
    foe_board_events::{FoeBoardChange, FoeBoardEvent},
    card_zone_events::{BattleCardsChange, CardZoneEvent},
};

fn active_game() -> Game {
    let mut game = Game::new(Hero::ZhugeLiang, 202610242338);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.battle.is_some());
    game.apply(Action::EndTurn).unwrap();
    game
}

#[test]
fn all_four_authorities_agree_at_each_action_boundary() {
    let mut game = Game::new(Hero::GuanYu, 1234);
    assert_eq!(game.audit_authority_bundle().unwrap(), 1);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.audit_authority_bundle().unwrap(), 2);
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.audit_authority_bundle().unwrap(), game.revision as usize + 1);
    game.verify_action_replay().unwrap();

    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    let mut without_actions = restored.clone();
    without_actions.action_history = None;
    without_actions.event_history = None;
    assert_eq!(without_actions.audit_authority_bundle().unwrap(), 3);

    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    assert_eq!(game.audit_authority_bundle().unwrap(), 3);
}

#[test]
fn catches_intermediate_foe_turn_forgery_even_when_individual_final_audit_passes() {
    let game = active_game();
    assert_eq!(game.audit_authority_bundle().unwrap(), 3);
    let mut tampered = game.clone();
    match &mut tampered.foe_board_history.as_mut().unwrap()[1] {
        FoeBoardEvent::Changed { change: FoeBoardChange::Enter { board }, .. } => {
            board.turn += 1;
        }
        _ => panic!("expected first combat to enter the enemy board"),
    }
    // The final board is unchanged, and each enemy transition remains legal.
    assert!(tampered.audit_foe_board().is_ok());
    assert!(tampered.audit_authority_bundle().unwrap_err().contains("boundary 1"));
}

#[test]
fn catches_intermediate_card_turn_forgery_even_when_individual_final_audit_passes() {
    let game = active_game();
    let mut tampered = game.clone();
    match &mut tampered.card_zone_history.as_mut().unwrap()[1] {
        CardZoneEvent::Changed { battle: BattleCardsChange::Enter { zones }, .. } => {
            zones.turn += 1;
        }
        _ => panic!("expected first combat to enter card zones"),
    }
    assert!(tampered.audit_card_zones().is_ok());
    assert!(tampered.audit_authority_bundle().unwrap_err().contains("boundary 1"));
}

#[test]
fn refuses_missing_or_truncated_authority_without_fabricating_history() {
    let game = active_game();
    let mut truncated = game.clone();
    truncated.resource_history.as_mut().unwrap().pop();
    assert!(truncated.audit_authority_bundle().unwrap_err().contains("revision count"));

    let mut json: serde_json::Value = serde_json::from_str(&game.to_json().unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("foe_board_history");
    let legacy = Game::from_json(&json.to_string()).unwrap();
    assert!(legacy.audit_authority_bundle().unwrap_err().contains("legacy"));
}

#[test]
fn projects_committed_boundaries_without_actions_or_rng_replay() {
    let mut game = Game::new(Hero::GuanYu, 1234);
    let initial = game.project_authority_boundary(0).unwrap();
    assert_eq!(initial["revision"], 0);
    assert_eq!(initial["resources"]["hp"], game.hp);
    assert!(initial["combat"].is_null());
    assert_eq!(initial["cards"]["deck"].as_array().unwrap().len(), game.deck.len());

    game.apply(Action::Choose { index: 1 }).unwrap();
    let entered = game.project_authority_boundary(1).unwrap();
    assert_eq!(entered["combat"]["turn"], 1);
    assert_eq!(entered["foes"]["foes"].as_array().unwrap().len(),
        game.battle.as_ref().unwrap().foes.len());
    assert!(entered["effects"].is_object());

    game.apply(Action::EndTurn).unwrap();
    let latest = game.project_authority_boundary(game.revision).unwrap();
    assert_eq!(latest["resources"]["hp"], game.hp);
    assert_eq!(latest["combat"]["turn"], game.battle.as_ref().unwrap().turn);
    assert_eq!(latest["cards"]["battle"]["hand"].as_array().unwrap().len(),
        game.battle.as_ref().unwrap().hand.len());

    let mut without_replay = Game::from_json(&game.to_json().unwrap()).unwrap();
    without_replay.action_history = None;
    without_replay.event_history = None;
    assert_eq!(without_replay.project_authority_boundary(1).unwrap(), entered);
    assert_eq!(without_replay.project_authority_boundary(game.revision).unwrap(), latest);
}

#[test]
fn boundary_projection_refuses_out_of_range_and_corrupt_future() {
    let game = active_game();
    assert!(game.project_authority_boundary(u64::MAX).unwrap_err().contains("beyond"));
    let mut truncated = game.clone();
    truncated.combat_effect_history.as_mut().unwrap().pop();
    // Even revision zero must not be reported as certified if the tail is missing.
    assert!(truncated.project_authority_boundary(0).unwrap_err().contains("revision count"));
    let mut legacy = game.clone();
    legacy.foe_board_history = None;
    assert!(legacy.project_authority_boundary(0).unwrap_err().contains("legacy"));
}
