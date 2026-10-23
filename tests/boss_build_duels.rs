//! Fixed-encounter paired build regression. Synthetic decks are NOT naturally
//! drafted builds; these outcomes cannot establish mature build win rates.
use three_kingdoms_spire_reborn::{
    Action, Card, Game, Intent, Node, Stage,
    builds::{BuildPlan, PLANS},
    combat::EventKind,
    content::{Effect, card_spec},
};

#[derive(Debug)]
struct Duel {
    survived: bool,
    hp_damage: u64,
    phase_seen: bool,
    techniques: usize,
    actions: usize,
}

fn duel_report(outcome: &Duel) -> serde_json::Value {
    serde_json::json!({
        "won": outcome.survived,
        "hp_damage": outcome.hp_damage,
        "phase_seen": outcome.phase_seen,
        "techniques": outcome.techniques,
        "actions": outcome.actions
    })
}

fn encounter(plan: &BuildPlan, boss: &str, ascension: u8, seed: u64, curated: bool) -> Game {
    let mut game = Game::new_with_ascension(plan.hero, seed, ascension);
    if curated {
        assert!(plan.is_catalog_valid());
        for (id, slot) in plan.core.iter().chain(plan.support.iter())
            .copied().zip([0_usize, 1, 2, 3, 5, 6])
        {
            assert_eq!(game.deck[slot].id, if slot < 5 { "strike" } else { "defend" });
            game.deck[slot] = Card::new(id);
        }
    }
    let first_battle = game.route.iter().position(|node|
        matches!(node, Node::Battle | Node::Elite | Node::Boss))
        .expect("opening route has a combat node");
    game.apply(Action::Choose { index: first_battle + 1 }).expect("opening combat");
    assert_eq!(game.stage, Stage::Combat);
    // This fixture cannot be reconstructed from seed/actions, because it
    // injects a named boss on floor one and (optionally) curated cards.
    game.action_history = None;
    game.last_node = Some(Node::Boss);
    game.hp = game.max_hp;
    game.potions = 0;
    game.potion_bag.clear();
    let battle = game.battle.as_mut().expect("battle");
    battle.foes.truncate(1);
    let foe = &mut battle.foes[0];
    foe.id = boss.into();
    foe.hp = 120;
    foe.max_hp = 120;
    foe.block = 0;
    foe.strength = 0;
    foe.poison = 0;
    foe.weak = 0;
    foe.vulnerable = 0;
    foe.phase = 0;
    foe.intent = Intent::Attack(8);
    game
}

/// Reuse the first-playable policy semantics of tks-sim, with a hard cap on
/// zero-cost loops. This is a deterministic bot, not a human strategy.
fn next_action(game: &Game) -> Action {
    let battle = game.battle.as_ref().expect("combat");
    if battle.played_this_turn >= 36 {
        return Action::EndTurn;
    }
    let target = battle.foes.iter().position(|foe| foe.hp > 0)
        .map_or(1, |index| index + 1);
    for (index, card) in battle.hand.iter().enumerate() {
        let Some(spec) = card_spec(&card.id) else { continue };
        if matches!(spec.effect, Effect::Unplayable) { continue; }
        let cost = if spec.cost == -1 { battle.energy }
            else if card.enchantment.as_deref() == Some("迅捷") {
                (spec.cost - 1).max(0)
            } else { spec.cost };
        if cost <= battle.energy {
            return Action::Play { index: index + 1, target };
        }
    }
    Action::EndTurn
}

fn duel(mut game: Game, boss: &str) -> Duel {
    let mut hp_damage = 0_u64;
    let mut phase_seen = false;
    let mut techniques = 0_usize;
    for step in 0..800 {
        assert_eq!(game.stage, Stage::Combat, "duel ended before evaluation");
        let history_len = game.event_history.as_ref().expect("full audit stream").len();
        let action = next_action(&game);
        game.apply(action).unwrap_or_else(|error| panic!(
            "{boss} turn {:?} action {step}: {error}",
            game.battle.as_ref().map(|battle| battle.turn),
        ));
        let events = &game.event_history.as_ref().expect("full audit stream")[history_len..];
        for event in events {
            if event.event.kind == EventKind::DamageReceived
                && event.event.target.as_deref() == Some("hero")
            {
                hp_damage += event.event.amount.max(0) as u64;
            }
            if event.event.kind == EventKind::EnemyTechnique && event.event.source == boss {
                techniques += 1;
            }
        }
        phase_seen |= game.battle.as_ref().is_some_and(|battle|
            battle.foes.iter().any(|foe| foe.id == boss && foe.phase > 0));
        if game.stage != Stage::Combat {
            assert!(matches!(game.stage, Stage::Reward | Stage::Won | Stage::Lost));
            let saved = game.to_json().expect("serialize result");
            assert_eq!(Game::from_json(&saved).expect("restore result").to_json().unwrap(), saved);
            return Duel {
                survived: game.stage != Stage::Lost,
                hp_damage,
                phase_seen,
                techniques,
                actions: step + 1,
            };
        }
        if step % 31 == 0 {
            let saved = game.to_json().expect("serialize mid-fight");
            assert_eq!(Game::from_json(&saved).expect("restore mid-fight").to_json().unwrap(), saved);
        }
    }
    panic!("{boss} encounter exceeded 800 actions without finishing");
}

#[test]
fn all_fifteen_curated_plans_have_paired_real_boss_outcomes() {
    assert_eq!(PLANS.len(), 15);
    let mut comparisons = 0;
    let mut baseline_wins = 0;
    let mut curated_wins = 0;
    let mut observed_damage = 0_u64;
    let mut observed_phases = 0;
    let mut observed_techniques = 0;
    let mut total_actions = 0;
    let mut cells = Vec::new();
    for (plan_index, plan) in PLANS.iter().enumerate() {
        for boss in ["simayi", "zuoci"] {
            for ascension in [0_u8, 10] {
                for trial in 0..2_u64 {
                    let seed = 26_102_400_u64 + (plan_index as u64) * 1000
                        + u64::from(ascension) * 20 + trial;
                    let baseline = duel(encounter(plan, boss, ascension, seed, false), boss);
                    let curated = duel(encounter(plan, boss, ascension, seed, true), boss);
                    baseline_wins += usize::from(baseline.survived);
                    curated_wins += usize::from(curated.survived);
                    observed_damage += baseline.hp_damage + curated.hp_damage;
                    observed_phases += usize::from(baseline.phase_seen)
                        + usize::from(curated.phase_seen);
                    observed_techniques += baseline.techniques + curated.techniques;
                    total_actions += baseline.actions + curated.actions;
                    cells.push(serde_json::json!({
                        "plan": plan.id,
                        "hero": format!("{:?}", plan.hero),
                        "boss": boss,
                        "ascension": ascension,
                        "seed": seed,
                        "baseline": duel_report(&baseline),
                        "curated": duel_report(&curated)
                    }));
                    comparisons += 1;
                }
            }
        }
    }
    assert_eq!(comparisons, 15 * 2 * 2 * 2);
    assert!(total_actions >= comparisons * 2);
    assert_eq!(cells.len(), comparisons);
    if let Ok(path) = std::env::var("TKS_BOSS_DUEL_REPORT") {
        let report = serde_json::json!({
            "schema_version": 1,
            "fixture": "single_floor_one_boss_120hp",
            "policy": "first_playable",
            "pairs": comparisons,
            "baseline_wins": baseline_wins,
            "curated_wins": curated_wins,
            "gross_hp_damage": observed_damage,
            "phase_observations": observed_phases,
            "technique_events": observed_techniques,
            "total_actions": total_actions,
            "cells": cells
        });
        let mut bytes = serde_json::to_vec_pretty(&report).expect("serialize boss report");
        bytes.push(b'\n');
        std::fs::write(path, bytes).expect("write boss report");
    }
    eprintln!(
        "BOSS_DUEL_FIXTURE pairs={comparisons} baseline_wins={baseline_wins} \
         curated_wins={curated_wins} gross_hp_damage={observed_damage} \
         phase_observations={observed_phases} technique_events={observed_techniques} \
         total_actions={total_actions}"
    );
}


/// Exploratory follow-up for the negative sun_discard/Zuoci result in the
/// original four-seed slice. Use NEW seeds shared by all three Sun builds:
/// the baseline is identical for a given (ascension, seed), so it is run once.
/// This is 32 independent seed/ascension fixtures, not 96 independent baselines.
#[test]
#[ignore = "explicit CI focus: 32 independent seeds x three Sun plans"]
fn sun_zuoci_independent_seed_pairs() {
    let plans: Vec<_> = PLANS.iter()
        .filter(|plan| plan.hero == three_kingdoms_spire_reborn::content::Hero::SunShangxiang)
        .collect();
    assert_eq!(plans.len(), 3);
    let mut cells = Vec::new();
    for ascension in [0_u8, 10] {
        for trial in 0..16_u64 {
            // Disjoint from the original 26_102_400 + plan_index * 1000 seeds.
            let seed = 26_300_000 + u64::from(ascension) * 100 + trial;
            let baseline = duel(encounter(plans[0], "zuoci", ascension, seed, false), "zuoci");
            for plan in &plans {
                let curated = duel(encounter(plan, "zuoci", ascension, seed, true), "zuoci");
                cells.push(serde_json::json!({
                    "plan": plan.id,
                    "hero": format!("{:?}", plan.hero),
                    "boss": "zuoci",
                    "ascension": ascension,
                    "seed": seed,
                    "baseline": duel_report(&baseline),
                    "curated": duel_report(&curated),
                }));
            }
        }
    }
    assert_eq!(cells.len(), 3 * 2 * 16);
    let baseline_wins: usize = cells.iter()
        .filter(|cell| cell["baseline"]["won"] == true).count();
    let curated_wins: usize = cells.iter()
        .filter(|cell| cell["curated"]["won"] == true).count();
    let report = serde_json::json!({
        "schema_version": 1,
        "fixture": "single_floor_one_boss_120hp",
        "policy": "first_playable",
        "focus": "sun_zuoci_independent_seeds",
        "independent_fixtures": 32,
        "pairs": cells.len(),
        "baseline_wins": baseline_wins,
        "curated_wins": curated_wins,
        "cells": cells,
    });
    let path = std::env::var("TKS_SUN_ZUOCI_REPORT")
        .expect("set TKS_SUN_ZUOCI_REPORT for this explicitly ignored test");
    let mut bytes = serde_json::to_vec_pretty(&report).expect("serialize focus report");
    bytes.push(b'\n');
    std::fs::write(path, bytes).expect("write focus report");
    eprintln!(
        "SUN_ZUOCI_FOCUS fixtures=32 pairs=96 baseline_wins={baseline_wins} curated_wins={curated_wins}"
    );
}
