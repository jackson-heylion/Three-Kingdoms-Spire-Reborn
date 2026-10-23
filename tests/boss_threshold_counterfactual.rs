//! Isolated boss *intent* counterfactuals. These are synthetic combat
//! fixtures, not natural-draft win rates or complete boss matchup evidence.
use three_kingdoms_spire_reborn::{
    Action, BossMove, Game, Intent, Node, Stage,
    content::Hero,
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

fn boss_fixture(hero: Hero, ascension: u8, boss: &str) -> Game {
    let mut game = Game::new_with_ascension(hero, 20261024, ascension);
    game.apply(Action::Choose { index: 1 }).expect("enter first combat");
    assert_eq!(game.stage, Stage::Combat);
    // The original map battle is replaced by a controlled one-foe encounter.
    // Such a synthetic setup cannot be verified by initial-seed action replay.
    game.action_history = None;
    game.event_history = None;
    game.last_node = Some(Node::Boss);
    let battle = game.battle.as_mut().expect("combat fixture");
    battle.foes.truncate(1);
    let foe = &mut battle.foes[0];
    foe.id = boss.into();
    foe.hp = 300;
    foe.max_hp = 300;
    foe.block = 0;
    foe.strength = 0;
    foe.poison = 0;
    foe.vulnerable = 0;
    foe.weak = 0;
    foe.phase = 0;
    // Avoid a first-turn signature or any HP loss before the intervention.
    foe.intent = Intent::Guard(0);
    battle.turn = 2;
    battle.played_this_turn = 0;
    battle.last_turn_cards = 0;
    battle.events.clear();
    battle.event_seq = 0;
    game
}

#[test]
fn named_boss_card_spam_thresholds_are_isolated_across_heroes_and_difficulties() {
    for hero in HEROES {
        for ascension in [0_u8, 10_u8] {
            for (boss, threshold) in [("simayi", 4_u8), ("zuoci", 5_u8)] {
                let mut below = boss_fixture(hero, ascension, boss);
                below.battle.as_mut().unwrap().played_this_turn = threshold - 1;
                let mut at = below.clone();
                // Exactly one treatment variable differs: cards played this turn.
                at.battle.as_mut().unwrap().played_this_turn = threshold;
                assert_eq!(below.rng_state, at.rng_state);
                assert_eq!(below.hp, at.hp);
                assert_eq!(below.battle.as_ref().unwrap().foes[0].hp,
                           at.battle.as_ref().unwrap().foes[0].hp);

                below.apply(Action::EndTurn).expect("below threshold");
                at.apply(Action::EndTurn).expect("at threshold");
                let low = below.battle.as_ref().expect("below battle persists");
                let high = at.battle.as_ref().expect("at battle persists");
                assert_eq!(low.turn, 3);
                assert_eq!(high.turn, 3);
                assert_eq!(low.last_turn_cards, threshold - 1);
                assert_eq!(high.last_turn_cards, threshold);
                assert!(!matches!(low.foes[0].intent, Intent::Boss(_)),
                    "{boss} unexpectedly used signature below threshold");
                match boss {
                    "simayi" => assert!(matches!(
                        high.foes[0].intent, Intent::Boss(BossMove::HiddenScheme(_)))),
                    "zuoci" => assert!(matches!(
                        high.foes[0].intent, Intent::Boss(BossMove::Mirage(_)))),
                    _ => unreachable!(),
                }
                // The threshold changes the next telegraphed intent, not RNG,
                // health or the enemy's identity at this comparison point.
                assert_eq!(below.rng_state, at.rng_state);
                assert_eq!(below.hp, at.hp);
                assert_eq!(low.foes[0].hp, high.foes[0].hp);
                assert_eq!(low.foes[0].id, high.foes[0].id);
                for game in [&below, &at] {
                    let snapshot = game.to_json().expect("serialize fixture");
                    assert_eq!(Game::from_json(&snapshot).expect("restore fixture")
                        .to_json().unwrap(), snapshot);
                }
            }
        }
    }
}


/// The first turn only telegraphs the response. The following real enemy
/// turn must settle its gameplay consequences, including after V2 restoration.
fn curse_count(game: &Game) -> usize {
    let b = game.battle.as_ref().expect("active combat");
    b.draw.iter().chain(b.hand.iter()).chain(b.discard.iter())
        .chain(b.exhausted.iter()).filter(|card| card.id == "cursed").count()
}

#[test]
fn threshold_counters_settle_real_resource_effects_and_replay_after_restore() {
    for hero in HEROES {
        for ascension in [0_u8, 10_u8] {
            for (boss, threshold) in [("simayi", 4_u8), ("zuoci", 5_u8)] {
                let mut below = boss_fixture(hero, ascension, boss);
                // Keep Zuo Ci injured, but above his phase transition, so
                // Mirage's actual healing is measurable and isolated.
                if boss == "zuoci" {
                    below.battle.as_mut().unwrap().foes[0].hp = 240;
                }
                below.battle.as_mut().unwrap().played_this_turn = threshold - 1;
                let mut at = below.clone();
                at.battle.as_mut().unwrap().played_this_turn = threshold;

                // Turn 2: both enemies execute the same harmless Guard(0).
                // Only the turn-3 telegraph changes with the card count.
                below.apply(Action::EndTurn).expect("below: telegraph");
                at.apply(Action::EndTurn).expect("at: telegraph");
                assert_eq!(below.rng_state, at.rng_state);
                let signature = match &at.battle.as_ref().unwrap().foes[0].intent {
                    Intent::Boss(move_id) => move_id.clone(),
                    other => panic!("{boss} failed to telegraph counter: {other:?}"),
                };
                assert!(!matches!(
                    below.battle.as_ref().unwrap().foes[0].intent,
                    Intent::Boss(_)
                ));

                // Snapshot at the decision boundary; no event-sourcing claim
                // is made for these synthetic, history-free fixtures.
                let saved = at.to_json().expect("save telegraphed encounter");
                let mut restored = Game::from_json(&saved).expect("restore encounter");
                let curses_before = curse_count(&at);
                let hp_before = at.battle.as_ref().unwrap().foes[0].hp;
                below.apply(Action::EndTurn).expect("below: execute");
                at.apply(Action::EndTurn).expect("at: execute");
                restored.apply(Action::EndTurn).expect("restored: execute");
                assert_eq!(at.to_json().unwrap(), restored.to_json().unwrap(),
                    "{boss} execution diverged after V2 restore");
                assert_eq!(at.stage, Stage::Combat);
                assert_eq!(below.stage, Stage::Combat);
                let low = below.battle.as_ref().unwrap();
                let high = at.battle.as_ref().unwrap();
                assert!(high.events.iter().any(|event|
                    event.kind == three_kingdoms_spire_reborn::combat::EventKind::EnemyTechnique
                        && event.source == boss),
                    "{boss} signature was not journalled");
                match signature {
                    BossMove::HiddenScheme(penalty) if boss == "simayi" => {
                        assert!(penalty > 0);
                        assert_eq!(high.energy, low.energy - i32::from(penalty),
                            "Sima Yi must actually reduce next-turn energy");
                        assert_eq!(curse_count(&at), curses_before + 1,
                            "Sima Yi must add exactly one curse below the cap");
                        assert_eq!(curse_count(&below), curses_before);
                    }
                    BossMove::Mirage(amount) if boss == "zuoci" => {
                        assert!(amount > 0);
                        assert!(hp_before < high.foes[0].max_hp);
                        assert_eq!(high.foes[0].hp - low.foes[0].hp,
                            amount.min(high.foes[0].max_hp - low.foes[0].hp),
                            "Zuo Ci must heal the actual missing HP");
                        assert!(high.foes[0].block >= amount / 2,
                            "Zuo Ci must gain the telegraphed shield");
                    }
                    other => panic!("unexpected boss signature {boss}: {other:?}"),
                }
            }
        }
    }
}
