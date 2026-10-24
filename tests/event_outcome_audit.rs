//! Read-only event-derived battle outcomes reconciled against lifetime counters.
//! This is not full Game event sourcing or proof of event authenticity.
use three_kingdoms_spire_reborn::{
    Action, CombatCounters, Game, RunMode,
    combat::{CombatEvent, EventKind},
    content::Hero,
    event_stack::{RunEvent, projection::{
        BattleOutcome, CampaignProjector, OutcomeTotals, ProjectionCheckpoint,
        project, project_sealed,
    }},
};

fn record(encounter: u64, floor: u8, seq: u64,
          kind: EventKind, source: &str, amount: i32) -> RunEvent {
    RunEvent {
        encounter, floor,
        event: CombatEvent {
            seq, kind, source: source.into(), amount, target: None,
        },
    }
}

#[test]
fn sealed_outcomes_reconcile_wins_losses_cards_and_active_battle() {
    let archive = [
        record(1, 1, 1, EventKind::CardPlayed, "strike", 1),
        record(1, 1, 2, EventKind::BattleEnded, "victory", 1),
        record(2, 2, 1, EventKind::EnemyTechnique, "simayi", 1),
        record(2, 2, 2, EventKind::BattleEnded, "defeat", 0),
    ];
    let projection = project_sealed(&archive).unwrap();
    assert_eq!(projection.encounters[0].outcome, Some(BattleOutcome::Victory));
    assert_eq!(projection.encounters[1].outcome, Some(BattleOutcome::Defeat));
    assert_eq!(projection.outcome_totals().unwrap(), OutcomeTotals {
        victories: 1, defeats: 1, active_encounters: 0, cards_played: 1,
    });

    let mut counters = CombatCounters {
        battles_started: 2, battles_won: 1, battles_lost: 1,
        cards_played: 1, ..Default::default()
    };
    projection.reconcile_telemetry(&counters, false).unwrap();
    assert!(projection.reconcile_telemetry(&counters, true).is_err());
    counters.battles_won = 2;
    assert!(projection.reconcile_telemetry(&counters, false).is_err());
    counters.battles_won = 1;
    counters.cards_played = 0;
    assert!(projection.reconcile_telemetry(&counters, false).is_err());
    counters.cards_played = 1;
    counters.battles_started = 3;
    assert!(projection.reconcile_telemetry(&counters, false).is_err());

    let unfinished = project(&archive[..3]).unwrap();
    assert_eq!(unfinished.outcome_totals().unwrap().active_encounters, 1);
    let partial = CombatCounters {
        battles_started: 2, battles_won: 1, cards_played: 1,
        ..Default::default()
    };
    unfinished.reconcile_telemetry(&partial, true).unwrap();
    assert!(unfinished.reconcile_telemetry(&partial, false).is_err());
}

#[test]
fn checkpoint_restores_outcome_but_rejects_unsealed_claim_and_legacy_unknown() {
    let mut projector = CampaignProjector::new();
    projector.push(&record(1, 1, 1, EventKind::CardPlayed, "strike", 1)).unwrap();
    projector.push(&record(1, 1, 2, EventKind::BattleEnded, "victory", 1)).unwrap();
    let serialized = serde_json::to_string(&projector.checkpoint()).unwrap();
    let checkpoint: ProjectionCheckpoint = serde_json::from_str(&serialized).unwrap();
    let resumed = CampaignProjector::resume(checkpoint.clone()).unwrap();
    assert_eq!(resumed.projection().outcome_totals().unwrap().victories, 1);

    let mut contradictory = checkpoint.clone();
    contradictory.projection.encounters[0].closed = false;
    assert!(CampaignProjector::resume(contradictory).is_err());

    // A previously serialized v1 checkpoint did not contain an outcome field.
    // Keep it readable but never invent a winner from the closed bit.
    let mut legacy = serde_json::to_value(checkpoint).unwrap();
    legacy["projection"]["encounters"][0].as_object_mut().unwrap().remove("outcome");
    let legacy: ProjectionCheckpoint = serde_json::from_value(legacy).unwrap();
    let resumed = CampaignProjector::resume(legacy).unwrap();
    assert!(resumed.projection().outcome_totals().is_err());
}

#[test]
fn live_archive_reconciles_with_real_game_telemetry_without_action_replay() {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 20261024, 0, RunMode::Standard);
    project(game.event_history.as_deref().unwrap()).unwrap()
        .reconcile_telemetry(&game.telemetry.totals, false).unwrap();
    game.apply(Action::Choose { index: 1 }).unwrap();
    let projection = project(game.event_history.as_deref().unwrap()).unwrap();
    projection.reconcile_telemetry(&game.telemetry.totals, game.battle.is_some()).unwrap();
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    project(restored.event_history.as_deref().unwrap()).unwrap()
        .reconcile_telemetry(&restored.telemetry.totals, restored.battle.is_some()).unwrap();
}
