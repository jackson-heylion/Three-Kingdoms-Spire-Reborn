use three_kingdoms_spire_reborn::{
    Action, Game, RunMode,
    combat::{CombatEvent, EventKind},
    content::Hero,
    event_stack::{RunEvent, projection::{CampaignProjection, CampaignProjector, ProjectionCheckpoint, project}},
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

#[test]
fn incremental_projection_matches_batch_across_battle_boundaries() {
    let archive = [
        event(1, 1, 1, EventKind::CardPlayed, 1),
        event(1, 1, 2, EventKind::DamageDealt, 11),
        event(1, 1, 3, EventKind::DamageReceived, 2),
        event(2, 3, 1, EventKind::EnemyTechnique, 1),
        event(2, 3, 2, EventKind::FoeSummoned, 1),
        event(3, 3, 1, EventKind::EventCountered, 0),
    ];
    let mut incremental = CampaignProjector::new();
    assert_eq!(incremental.projection(), &project(&[]).unwrap());
    for record in &archive[..3] {
        incremental.push(record).unwrap();
    }
    assert_eq!(incremental.projection(), &project(&archive[..3]).unwrap());
    for record in &archive[3..] {
        incremental.push(record).unwrap();
    }
    assert_eq!(incremental.finish(), project(&archive).unwrap());
}

#[test]
fn rejected_incremental_events_leave_cursor_and_counters_unchanged() {
    let first = event(1, 1, 1, EventKind::CardPlayed, 1);
    let second = event(1, 1, 2, EventKind::DamageDealt, 7);
    let mut reducer = CampaignProjector::new();
    assert!(reducer.push(&event(2, 1, 1, EventKind::CardPlayed, 1)).is_err());
    assert!(reducer.projection().encounters.is_empty());
    reducer.push(&first).unwrap();
    let expected = reducer.projection().clone();
    for invalid in [
        event(1, 1, 1, EventKind::DamageDealt, 1), // duplicate
        event(1, 1, 3, EventKind::DamageDealt, 1), // gap
        event(1, 2, 2, EventKind::DamageDealt, 1), // floor mismatch
        event(3, 2, 1, EventKind::DamageDealt, 1), // skipped encounter
        event(1, 1, 2, EventKind::DamageDealt, -1), // negative settled damage
        event(1, 1, 2, EventKind::DamageReceived, -1),
    ] {
        assert!(reducer.push(&invalid).is_err());
        assert_eq!(reducer.projection(), &expected);
    }
    // A failed event does not consume sequence 2.
    reducer.push(&second).unwrap();
    assert_eq!(reducer.finish(), project(&[first, second]).unwrap());
}

#[test]
fn live_archive_streams_identically_without_action_log() {
    let mut game = Game::new_with_mode(Hero::SunShangxiang, 20261024, 0, RunMode::Standard);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let saved = Game::from_json(&game.to_json().unwrap()).unwrap();
    let archive = saved.event_history.as_ref().unwrap();
    let mut reducer = CampaignProjector::new();
    for record in archive {
        reducer.push(record).unwrap();
    }
    assert_eq!(reducer.finish(), project(archive).unwrap());
}

#[test]
fn serialized_checkpoint_resumes_at_next_event_across_encounters() {
    let archive = [
        event(1, 1, 1, EventKind::CardPlayed, 1),
        event(1, 1, 2, EventKind::DamageDealt, 8),
        event(2, 3, 1, EventKind::EnemyTechnique, 1),
        event(2, 3, 2, EventKind::FoeSummoned, 1),
        event(3, 3, 1, EventKind::EventCountered, 0),
    ];
    let mut partial = CampaignProjector::new();
    for record in &archive[..3] {
        partial.push(record).unwrap();
    }
    let json = serde_json::to_string(&partial.checkpoint()).unwrap();
    let checkpoint: ProjectionCheckpoint = serde_json::from_str(&json).unwrap();
    let mut resumed = CampaignProjector::resume(checkpoint).unwrap();
    assert_eq!(resumed.projection(), &project(&archive[..3]).unwrap());
    // Replayed old events and skipped coordinates must not advance the cursor.
    assert!(resumed.push(&archive[2]).is_err());
    assert!(resumed.push(&archive[4]).is_err());
    for record in &archive[3..] {
        resumed.push(record).unwrap();
    }
    assert_eq!(resumed.finish(), project(&archive).unwrap());
}

#[test]
fn rejects_malformed_or_incompatible_projection_checkpoints() {
    let mut reducer = CampaignProjector::new();
    reducer.push(&event(1, 1, 1, EventKind::CardPlayed, 1)).unwrap();
    reducer.push(&event(1, 1, 2, EventKind::DamageDealt, 3)).unwrap();
    let valid = reducer.checkpoint();
    let mut wrong_version = valid.clone();
    wrong_version.schema_version = 2;
    let mut skipped_encounter = valid.clone();
    skipped_encounter.projection.encounters[0].encounter = 2;
    let mut wrong_event_count = valid.clone();
    wrong_event_count.projection.encounters[0].events = 1;
    let mut impossible_kind_count = valid.clone();
    impossible_kind_count.projection.encounters[0].cards_played = 3;
    let mut invalid_floor = valid.clone();
    invalid_floor.projection.encounters[0].floor = 0;
    for invalid in [
        wrong_version, skipped_encounter, wrong_event_count,
        impossible_kind_count, invalid_floor,
    ] {
        assert!(CampaignProjector::resume(invalid).is_err());
    }
    // Failed resume attempts cannot mutate the source reducer.
    assert_eq!(reducer.checkpoint(), valid);
    let mut empty = CampaignProjector::resume(CampaignProjector::new().checkpoint()).unwrap();
    empty.push(&event(1, 1, 1, EventKind::CardPlayed, 1)).unwrap();
    assert_eq!(empty.projection().encounters[0].events, 1);
}
