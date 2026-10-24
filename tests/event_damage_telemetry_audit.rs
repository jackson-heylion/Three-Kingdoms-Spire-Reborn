//! Foe-attack HP damage has a narrower provenance than all DamageReceived.
use three_kingdoms_spire_reborn::{
    Action, CombatCounters, Game, Intent, RunMode,
    combat::{CombatEvent, EventKind},
    content::Hero,
    event_stack::{RunEvent, projection::{CampaignProjector, ProjectionCheckpoint, project}},
};

fn record(seq: u64, kind: EventKind, source: &str, amount: i32) -> RunEvent {
    RunEvent {
        encounter: 1, floor: 1,
        event: CombatEvent { seq, kind, source: source.into(), amount,
            target: Some("hero".into()) },
    }
}

#[test]
fn reconciles_only_foe_attack_damage_not_blood_price_or_attrition() {
    let mut end = record(5, EventKind::BattleEnded, "defeat", 0);
    end.event.target = None;
    let archive = [
        record(1, EventKind::DamageReceived, "foe:0", 4),
        record(2, EventKind::DamageReceived, "blood_price", 3),
        record(3, EventKind::DamageReceived, "supply_exhaustion", 2),
        record(4, EventKind::DamageReceived, "foe:1", 0),
        end,
    ];
    let projection = project(&archive).unwrap();
    assert_eq!(projection.encounters[0].hero_hp_damage, 9);
    assert_eq!(projection.encounters[0].foe_attack_hp_damage, Some(4));
    let mut counters = CombatCounters {
        battles_started: 1, battles_lost: 1, hero_hp_damage: 4,
        ..Default::default()
    };
    projection.reconcile_telemetry(&counters, false).unwrap();
    counters.hero_hp_damage = 9;
    assert!(projection.reconcile_telemetry(&counters, false).is_err());
}

#[test]
fn legacy_checkpoint_keeps_unknown_damage_provenance_unknown() {
    let mut projector = CampaignProjector::new();
    projector.push(&record(1, EventKind::DamageReceived, "foe:0", 5)).unwrap();
    let checkpoint = projector.checkpoint();
    let mut legacy = serde_json::to_value(&checkpoint).unwrap();
    legacy["projection"]["encounters"][0].as_object_mut().unwrap()
        .remove("foe_attack_hp_damage");
    let legacy: ProjectionCheckpoint = serde_json::from_value(legacy).unwrap();
    let mut resumed = CampaignProjector::resume(legacy).unwrap();
    assert_eq!(resumed.projection().encounters[0].foe_attack_hp_damage, None);
    resumed.push(&record(2, EventKind::DamageReceived, "foe:1", 2)).unwrap();
    assert_eq!(resumed.projection().encounters[0].foe_attack_hp_damage, None);
    let counters = CombatCounters { battles_started: 1, hero_hp_damage: 7,
        ..Default::default() };
    assert!(resumed.projection().reconcile_telemetry(&counters, true).is_err());

    let mut forged = checkpoint;
    forged.projection.encounters[0].foe_attack_hp_damage = Some(6);
    assert!(CampaignProjector::resume(forged).is_err());
}

#[test]
fn live_enemy_attack_reconciles_after_v2_roundtrip() {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 20261024, 0, RunMode::Standard);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let battle = game.battle.as_mut().unwrap();
    battle.block = 0;
    battle.troops_hp = 0;
    for foe in &mut battle.foes {
        foe.intent = Intent::Attack(4);
    }
    game.apply(Action::EndTurn).unwrap();
    let projection = project(game.event_history.as_deref().unwrap()).unwrap();
    projection.reconcile_telemetry(&game.telemetry.totals, game.battle.is_some()).unwrap();
    let loaded = Game::from_json(&game.to_json().unwrap()).unwrap();
    project(loaded.event_history.as_deref().unwrap()).unwrap()
        .reconcile_telemetry(&loaded.telemetry.totals, loaded.battle.is_some()).unwrap();
}
