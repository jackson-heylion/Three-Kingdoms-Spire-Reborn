//! Natural first-battle scenario, beginning from a public new-tier campaign.
//! Unlike a high-floor checkpoint, all actions here originate at the real seed.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Stage, content::Hero};

#[test]
fn first_new_tier_battle_requires_breakout_before_encirclement() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_301, Difficulty::Warlords, "breakout-247301".into(),
    );
    game.apply(Action::Choose { index: 1 }).expect("natural first battle");
    assert_eq!(game.stage, Stage::Combat);
    let before_gold = game.gold;
    // Doing nothing for four complete enemy turns must now lose the timed
    // breakout objective, not grant a normal victory or reward.
    for turn in 1..=4 {
        assert_eq!(game.stage, Stage::Combat, "must have an opportunity on turn {turn}");
        game.apply(Action::EndTurn).expect("end turn through public action");
    }
    assert_eq!(game.stage, Stage::Lost, "encirclement must have a defeat outcome");
    assert_eq!(game.gold, before_gold, "a failed breakout pays no victory gold");
    assert!(game.log.last().is_some_and(|entry| entry.contains("突围失败")),
        "failure must explain the objective, not silently count as a normal battle");
    game.verify_action_replay().expect("timed failure action replay");
    let snapshot = game.to_json().expect("save timed defeat");
    let restored = Game::from_json(&snapshot).expect("restore timed defeat");
    assert_eq!(restored.to_json().unwrap(), snapshot);
    restored.verify_action_replay().expect("restored timed failure action replay");
}
