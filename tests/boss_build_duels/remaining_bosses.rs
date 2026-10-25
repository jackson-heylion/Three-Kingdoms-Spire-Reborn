//! Paired coverage for the four bosses absent from the original Simayi/Zuoci
//! duel fixture. Curated cards are injected; this is NOT natural acquisition,
//! boss balance certification, or evidence of human playability.
use super::*;

#[test]
fn remaining_four_bosses_have_seed_paired_three_policy_encounters() {
    const BOSSES: [&str; 4] = ["zhangjiao", "dongzhuo", "sunjian", "yuanshao"];
    const DIFFICULTIES: [u8; 2] = [0, 10];
    const POLICIES: [(&str, DuelPolicy); 3] = [
        ("first_playable", DuelPolicy::FirstPlayable),
        ("discard_aware", DuelPolicy::DiscardAware),
        ("guard_first", DuelPolicy::GuardFirst),
    ];
    assert_eq!(PLANS.len(), 15);
    let mut cells = Vec::new();
    let mut baseline_wins = 0_usize;
    let mut curated_wins = 0_usize;
    let mut baseline_only_wins = 0_usize;
    let mut curated_only_wins = 0_usize;
    let mut technique_events = 0_usize;
    let mut phase_observations = 0_usize;

    for (plan_index, plan) in PLANS.iter().enumerate() {
        for (boss_index, boss) in BOSSES.iter().enumerate() {
            for ascension in DIFFICULTIES {
                // Identical seed for all three policies and each baseline/plan
                // pair; disjoint from the earlier Simayi/Zuoci fixture seeds.
                let seed = 26_400_000_u64 + plan_index as u64 * 1_000
                    + boss_index as u64 * 100 + u64::from(ascension);
                for (policy_id, policy) in POLICIES {
                    let baseline = duel_with_policy(
                        encounter(plan, boss, ascension, seed, false), boss, policy,
                    );
                    let curated = duel_with_policy(
                        encounter(plan, boss, ascension, seed, true), boss, policy,
                    );
                    baseline_wins += usize::from(baseline.survived);
                    curated_wins += usize::from(curated.survived);
                    baseline_only_wins += usize::from(baseline.survived && !curated.survived);
                    curated_only_wins += usize::from(curated.survived && !baseline.survived);
                    technique_events += baseline.techniques + curated.techniques;
                    phase_observations += usize::from(baseline.phase_seen)
                        + usize::from(curated.phase_seen);
                    assert!(baseline.actions > 0 && curated.actions > 0);
                    cells.push(serde_json::json!({
                        "plan": plan.id,
                        "hero": format!("{:?}", plan.hero),
                        "boss": boss,
                        "ascension": ascension,
                        "seed": seed,
                        "policy": policy_id,
                        "baseline": duel_report(&baseline),
                        "curated": duel_report(&curated),
                        "baseline_discards": baseline.discard_actions,
                        "curated_discards": curated.discard_actions,
                    }));
                }
            }
        }
    }
    let pairs = 15 * BOSSES.len() * DIFFICULTIES.len() * POLICIES.len();
    assert_eq!(cells.len(), pairs);
    assert!(technique_events > 0, "no boss-specific technique was exercised");
    let report = serde_json::json!({
        "schema_version": 1,
        "fixture": "single_floor_one_boss_120hp",
        "interpretation": "synthetic ten-card decks, fixed single-floor boss, three deterministic heuristics; not natural drafting, full-campaign boss exposure or human win rate",
        "bosses": BOSSES,
        "policies": POLICIES.map(|(id, _)| id),
        "ascensions": DIFFICULTIES,
        "pairs": pairs,
        "baseline_wins": baseline_wins,
        "curated_wins": curated_wins,
        "baseline_only_wins": baseline_only_wins,
        "curated_only_wins": curated_only_wins,
        "technique_events": technique_events,
        "phase_observations": phase_observations,
        "cells": cells,
    });
    if let Ok(path) = std::env::var("TKS_REMAINING_BOSS_DUEL_REPORT") {
        let mut bytes = serde_json::to_vec_pretty(&report).expect("serialize boss matrix");
        bytes.push(b'\n');
        std::fs::write(path, bytes).expect("write boss matrix");
    }
    eprintln!(
        "FOUR_BOSS_MATRIX pairs={pairs} baseline_wins={baseline_wins} curated_wins={curated_wins} baseline_only={baseline_only_wins} curated_only={curated_only_wins} techniques={technique_events} phase_seen={phase_observations}"
    );
}
