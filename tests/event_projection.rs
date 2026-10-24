use three_kingdoms_spire_reborn::{
    Action, Game, RunMode,
    combat::{CombatEvent, EventKind},
    content::Hero,
    event_stack::{RunEvent, projection::{CampaignProjection, project}},
};

fn event(encounter: u64, floor: u8, seq: u64, kind: EventKind, amount: i32) -> RunEvent {
    RunEvent {
        encounter, floor,
        event: CombatEvent {
            seq, kind, source: "fixture".into(), amount, target: None,
        },
    }
}

#[test]
fn reconstructs_observable_facts_across_encounters_without_actions() {
    let archive = vec![
        event(1, 1, 1, EventKind::CardPlayed, 1),
        event(1, 1, 2, EventKind::DamageDealt, 9),
        event(1, 1, 3, EventKind::DamageReceived, 3),
        event(1, 1, 4, EventKind::EventCountered, 0),
        event(1, 1, 5, EventKind::FoeDefeated, 1),
        event(2, 2, 1, EventKind::FoeSummoned, 1),
        event(2, 2, 2, EventKind::EnemyTechnique, 1),
        event(2, 2, 3, EventKind::EventReplaced, 4),
        event(2, 2, 4, EventKind::DamageDealt, 5),
    ];
    let projection = project(&archive).unwrap();
    assert_eq!(projection.encounters.len(), 2);
    let first = &projection.encounters[0];
    assert_eq!((first.encounter, first.floor, first.first_seq, first.last_seq),
        (1, 1, 1, 5));
    assert_eq!((first.events, first.cards_played, first.foe_hp_damage,
        first.hero_hp_damage, first.foes_defeated, first.reactions_countered),
        (5, 1, 9, 3, 1, 1));
    let second = &projection.encounters[1];
    assert_eq!((second.events, second.foes_summoned, second.enemy_techniques,
        second.reactions_replaced, second.foe_hp_damage), (4, 1, 1, 1, 5));
    let encoded = serde_json::to_string(&projection).unwrap();
    let restored: CampaignProjection = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored, projection);
}

#[test]
fn rejects_missing_battle_events_and_negative_settled_damage() {
    let mut archive = vec![
        event(1, 1, 1, EventKind::CardPlayed, 1),
        event(1, 1, 2, EventKind::DamageDealt, 4),
    ];
    assert!(project(&archive).is_ok());
    archive[1].event.seq = 3;
    assert!(project(&archive).is_err());
    archive[1].event.seq = 2;
    archive[1].event.amount = -1;
    assert!(project(&archive).is_err());
    archive[1].event.kind = EventKind::DamageReceived;
    assert!(project(&archive).is_err());
    archive[1].event.kind = EventKind::DamageDealt;
    archive[1].event.amount = i32::MAX;
    assert_eq!(project(&archive).unwrap().encounters[0].foe_hp_damage,
        i32::MAX as u64);
}

#[test]
fn live_game_archive_projects_identically_after_v2_roundtrip() {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 20261024, 0, RunMode::Standard);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let before = project(game.event_history.as_deref().unwrap()).unwrap();
    assert!(!before.encounters.is_empty());
    let loaded = Game::from_json(&game.to_json().unwrap()).unwrap();
    let after = project(loaded.event_history.as_deref().unwrap()).unwrap();
    assert_eq!(before, after);
    // This projection requires no action log. It deliberately does NOT
    // certify the remaining Game state or replace verify_action_replay().
    let mut without_actions = loaded;
    without_actions.action_history = None;
    assert_eq!(project(without_actions.event_history.as_deref().unwrap()).unwrap(), before);
    assert!(project(&[]).unwrap().encounters.is_empty());
}
