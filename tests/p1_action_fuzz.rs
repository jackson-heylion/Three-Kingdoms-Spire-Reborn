//! Deterministic invalid/valid action fuzzing across all five heroes.
//! No external RNG dependency: each failure must preserve the exact V2 JSON,
//! and the committed action history must reconstruct the final state.

use three_kingdoms_spire_reborn::{Action, Game, Stage, content::Hero};

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn randomized_action(game: &Game, rng: &mut u64) -> Action {
    let roll = next(rng);
    let index = (roll as usize % 12) + 1;
    match game.stage {
        Stage::Combat => {
            let hand = game.battle.as_ref().map_or(0, |b| b.hand.len());
            // Some genuinely playable hand positions mixed with invalid ones.
            let card = if hand > 0 && roll % 4 != 0 {
                (next(rng) as usize % hand) + 1
            } else { index + hand };
            let target = (next(rng) as usize % 6) + 1;
            match roll % 9 {
                0 | 1 | 2 => Action::Play { index: card, target },
                3 | 4 => Action::EndTurn,
                5 => Action::Discard { index: card },
                6 => Action::UsePotionAt { slot: (next(rng) as usize % 4) + 1, target },
                7 => Action::PlaySelected { index: card, target,
                    selection: (next(rng) as usize % 8) + 1 },
                _ => Action::UsePotion,
            }
        }
        Stage::Map if game.floor == 0 && roll % 8 == 0 =>
            Action::SetDoctrine { id: "auto".to_string() },
        Stage::Map | Stage::Reward | Stage::Camp | Stage::Market |
        Stage::Story | Stage::Ancient => {
            // Include 0 and impossible menu entries to verify rejection is atomic.
            Action::Choose { index: roll as usize % 9 }
        }
        Stage::Won | Stage::Lost => Action::EndTurn,
    }
}

#[test]
fn mixed_action_fuzz_is_transactional_replayable_and_restorable() {
    let heroes = [Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
        Hero::CaoCao, Hero::LuBu];
    let mut successes = 0_u32;
    let mut failures = 0_u32;

    for (hero_index, hero) in heroes.into_iter().enumerate() {
        for (scenario, ascension) in [0, 5, 10].into_iter().enumerate() {
            let seed = 94_000 + (hero_index * 17 + scenario) as u64;
            let mut game = Game::new_with_ascension(hero, seed, ascension);
            let mut rng = seed ^ 0x9e37_79b9_7f4a_7c15;
            for step in 0..160 {
                if matches!(game.stage, Stage::Won | Stage::Lost) { break; }
                let action = randomized_action(&game, &mut rng);
                let before = game.to_json().unwrap();
                let prior_revision = game.revision;
                match game.apply(action) {
                    Ok(_) => {
                        successes += 1;
                        assert_eq!(game.revision, prior_revision + 1,
                            "successful action must commit exactly once: hero {hero_index}, step {step}");
                        game.validate().unwrap();
                        if step % 17 == 0 {
                            let saved = game.to_json().unwrap();
                            assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved,
                                "V2 save must survive after any committed action");
                        }
                    }
                    Err(_) => {
                        failures += 1;
                        assert_eq!(game.to_json().unwrap(), before,
                            "rejected action mutated game or RNG: hero {hero_index}, step {step}");
                        assert_eq!(game.revision, prior_revision);
                    }
                }
            }
            game.verify_action_replay().unwrap_or_else(|e| {
                panic!("hero {hero_index}, scenario {scenario}, seed {seed} failed replay: {e}");
            });
            let snapshot = game.to_json().unwrap();
            let restored = Game::from_json(&snapshot).unwrap();
            assert_eq!(restored.to_json().unwrap(), snapshot);
            restored.verify_action_replay().unwrap();
        }
    }

    assert!(successes >= 100, "fuzzer failed to exercise enough accepted actions: {successes}");
    assert!(failures >= 100, "fuzzer failed to exercise enough rejected actions: {failures}");
}
