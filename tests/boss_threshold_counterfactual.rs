//! Isolated boss *intent* counterfactuals. These are synthetic combat
//! fixtures, not natural-draft win rates or complete boss matchup evidence.
use three_kingdoms_spire_reborn::{
    Action, BossMove, Game, Hero, Intent, Node, Stage,
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
