//! Fifteen naturally played defeat paths across the five heroes and three new
//! difficulties. This is defeat/settlement coverage, NOT 1-to-40 victory coverage.
//! No battle state, deck, RNG, lore flags, or checkpoint is injected.
use three_kingdoms_spire_reborn::{
    Action, Difficulty, Game, Node, Stage,
    content::Hero, progression::Profile,
};

#[test]
fn all_five_heroes_and_three_tiers_can_naturally_lose_and_settle_once() {
    let heroes = [
        Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
        Hero::CaoCao, Hero::LuBu,
    ];
    let tiers = [Difficulty::Warlords, Difficulty::Conqueror, Difficulty::Destiny];

    for (hero_index, hero) in heroes.into_iter().enumerate() {
        for (tier_index, tier) in tiers.into_iter().enumerate() {
            let seed = 247_600 + (hero_index * 3 + tier_index) as u64;
            let run_id = format!("natural-loss-{hero_index}-{tier_index}-{seed}");
            let mut game = Game::new_with_campaign_difficulty(hero, seed, tier, run_id.clone());
            game.apply(Action::Choose { index: 1 }).expect("legal campaign opening");

            for _ in 0..900 {
                if matches!(game.stage, Stage::Lost | Stage::Won) { break; }
                let action = match game.stage {
                    Stage::Combat => Action::EndTurn,
                    Stage::Map => {
                        let index = game.route.iter()
                            .position(|node| matches!(node, Node::Battle | Node::Elite | Node::Boss))
                            .or_else(|| (!game.route.is_empty()).then_some(0))
                            .expect("natural map must have a legal onward route");
                        Action::Choose { index: index + 1 }
                    }
                    Stage::Reward | Stage::Market => Action::Choose { index: 0 },
                    Stage::Camp | Stage::Ancient => Action::Choose { index: 1 },
                    Stage::Story => Action::Choose { index: 3 },
                    other => panic!("{hero:?} {tier:?}: unexpected stage {other:?}"),
                };
                game.apply(action).unwrap_or_else(|error| {
                    panic!("{hero:?} {tier:?} seed {seed}: legal player action failed: {error}")
                });
            }

            assert_eq!(game.stage, Stage::Lost,
                "{hero:?} {tier:?} seed {seed}: passive natural player did not lose");
            game.audit_campaign_authority().expect("defeat authority");
            game.verify_action_replay().expect("defeat replay");
            let snapshot = game.to_json().expect("serialize defeat");
            let restored = Game::from_json(&snapshot).expect("restore defeat");
            assert_eq!(restored.to_json().unwrap(), snapshot);
            restored.verify_action_replay().expect("restored defeat replay");

            let mut profile = Profile::new();
            assert!(profile.record(&game, run_id.clone()).expect("record defeat"));
            let persisted = profile.to_json().expect("persist defeat");
            let mut reloaded = Profile::from_json(&persisted).expect("reload profile");
            assert!(!reloaded.record(&restored, run_id).expect("duplicate defeat"));
            assert_eq!((reloaded.wins, reloaded.losses, reloaded.records.len()), (0, 1, 1));
            assert!(reloaded.difficulty_clears.is_empty(), "defeat must not unlock new tier");
            assert!(reloaded.ascensions.is_empty(), "new tier must not affect old ascensions");
            assert_eq!(reloaded.to_json().unwrap(), persisted);

            let mut terminal = restored;
            assert!(terminal.apply(Action::EndTurn).is_err());
            assert_eq!(terminal.to_json().unwrap(), snapshot,
                "rejected terminal action must not change state or RNG");
        }
    }
}
