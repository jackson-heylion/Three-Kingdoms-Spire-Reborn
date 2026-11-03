//! Regression: ascension 1 must affect *actual* combat rather than only the UI.
//! Both runs use the same hero, map, seed and choice.
use three_kingdoms_spire_reborn::{Action, Game, Intent, Node, content::Hero};

fn first_encounter(hero: Hero, seed: u64, rank: u8) -> Game {
    let mut game = Game::new_with_ascension(hero, seed, rank);
    let choice = game.route.iter().position(|node| *node == Node::Battle)
        .expect("seed must expose a regular encounter");
    game.apply(Action::Choose { index: choice + 1 })
        .expect("enter first encounter");
    game
}

#[test]
fn rank_one_has_stronger_foes_than_rank_zero_for_every_hero() {
    for hero in [Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang, Hero::CaoCao, Hero::LuBu] {
        let normal = first_encounter(hero, 19, 0);
        let harder = first_encounter(hero, 19, 1);
        let a = normal.battle.as_ref().unwrap();
        let b = harder.battle.as_ref().unwrap();
        assert_eq!(a.foes.len(), b.foes.len());
        for (low, high) in a.foes.iter().zip(&b.foes) {
            assert_eq!(low.id, high.id, "same seed must choose same enemy");
            assert_eq!(high.hp, low.hp + 1, "rank 1 must increase HP");
            assert_eq!(high.max_hp, low.max_hp + 1);
        }
        assert_eq!(normal.ascension, 0);
        assert_eq!(harder.ascension, 1);
        assert_eq!(Game::from_json(&harder.to_json().unwrap()).unwrap().ascension, 1);
        assert!(normal.verify_action_replay().is_ok());
        assert!(harder.verify_action_replay().is_ok());
    }
}

#[test]
fn rank_one_also_increases_telegraphed_attacks_and_guards() {
    let mut attack = false;
    let mut guard = false;
    for seed in 1..=300 {
        let low = Game::new_with_ascension(Hero::GuanYu, seed, 0);
        let high = Game::new_with_ascension(Hero::GuanYu, seed, 1);
        let Some(choice) = low.route.iter().position(|n| *n == Node::Battle) else { continue; };
        let mut low = low;
        let mut high = high;
        low.apply(Action::Choose { index: choice + 1 }).unwrap();
        high.apply(Action::Choose { index: choice + 1 }).unwrap();
        let lhs = &low.battle.as_ref().unwrap().foes[0].intent;
        let rhs = &high.battle.as_ref().unwrap().foes[0].intent;
        match (lhs, rhs) {
            (Intent::Attack(a), Intent::Attack(b)) => {
                assert_eq!(*b, *a + 1, "rank 1 attack differs from rank 0");
                attack = true;
            }
            (Intent::Guard(a), Intent::Guard(b)) => {
                assert_eq!(*b, *a + 1, "rank 1 guard differs from rank 0");
                guard = true;
            }
            _ => {}
        }
        if attack && guard { break; }
    }
    assert!(attack, "must exercise a real rank-1 attack intent");
    assert!(guard, "must exercise a real rank-1 guard intent");
}

#[test]
fn difficulty_scaling_remains_monotone_across_all_ranks() {
    let baseline = first_encounter(Hero::CaoCao, 19, 0);
    let hp0 = baseline.battle.as_ref().unwrap().foes[0].max_hp;
    let mut last = hp0;
    for rank in 1..=10 {
        let game = first_encounter(Hero::CaoCao, 19, rank);
        let hp = game.battle.as_ref().unwrap().foes[0].max_hp;
        assert_eq!(hp, hp0 + (i32::from(rank) + 1) / 2);
        assert!(hp >= last);
        last = hp;
    }
}
