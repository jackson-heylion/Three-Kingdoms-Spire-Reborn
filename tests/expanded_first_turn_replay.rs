//! Exercise an actual enemy turn through the public action seam, rather than
//! treating boss-entry checkpoints as proof of playable combat.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Stage, content::Hero};

#[test]
fn first_enemy_turn_replays_and_restores_for_every_hero_and_tier() {
    let heroes = [
        Hero::CaoCao, Hero::GuanYu, Hero::ZhugeLiang,
        Hero::SunShangxiang, Hero::LuBu,
    ];
    for (hero_index, hero) in heroes.into_iter().enumerate() {
        for tier in Difficulty::ALL {
            let mut game = Game::new_with_campaign_difficulty(
                hero, 247_000 + hero_index as u64, tier,
                format!("first-turn-{hero_index}-{}", tier.id()),
            );
            game.apply(Action::Choose { index: 1 }).expect("enter first battle");
            assert_eq!(game.stage, Stage::Combat);
            let before = game.revision;
            game.apply(Action::EndTurn).expect("resolve first enemy turn");
            assert_eq!(game.revision, before + 1, "enemy turn must commit once");
            game.audit_campaign_authority().expect("campaign journal");
            game.verify_action_replay().expect("first enemy turn replay");
            let saved = game.to_json().expect("serialize after enemy turn");
            let resumed = Game::from_json(&saved).expect("resume after enemy turn");
            assert_eq!(resumed.to_json().unwrap(), saved);
            resumed.verify_action_replay().expect("resumed enemy turn replay");
        }
    }
}
