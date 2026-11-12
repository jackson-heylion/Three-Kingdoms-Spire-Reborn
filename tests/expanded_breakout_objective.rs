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

#[test]
fn breakout_objective_is_visible_read_only_and_legacy_free() {
    for tier in Difficulty::ALL {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 247_302, tier, format!("breakout-notice-{}", tier.id()),
        );
        game.apply(Action::Choose { index: 1 }).expect("enter genuine first battle");
        let before = game.to_json().unwrap();
        let notice = game.tactical_objective().expect("visible new-tier objective");
        assert!(notice.contains("突围") && notice.contains("第4回合"), "{notice}");
        assert_eq!(game.to_json().unwrap(), before, "preview must not consume RNG or revision");
        game.verify_action_replay().expect("notice is read-only for replay");
    }
    let mut legacy = Game::new(Hero::GuanYu, 247_302);
    legacy.apply(Action::Choose { index: 1 }).unwrap();
    assert!(legacy.tactical_objective().is_none(), "21-floor original rules unchanged");
}

#[test]
fn breakout_can_be_won_early_by_normal_player_card_actions() {
    // Real seeded games only: no hand injection, saved checkpoint, or forged flags.
    let mut winning_game = None;
    for seed in 247_310..247_342 {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::LuBu, seed, Difficulty::Warlords, format!("breakout-win-{seed}"),
        );
        game.apply(Action::Choose { index: 1 }).unwrap();
        for _ in 0..4 {
            while game.stage == Stage::Combat {
                let choice = game.battle.as_ref().and_then(|battle| {
                    battle.hand.iter().enumerate().find(|(_, card)| {
                        (card.id == "strike" || card.id == "wushuang")
                            && three_kingdoms_spire_reborn::content::card_spec(&card.id)
                                .is_some_and(|spec| spec.cost <= battle.energy)
                    }).map(|(index, _)| index + 1)
                });
                let Some(index) = choice else { break; };
                game.apply(Action::Play { index, target: 1 }).expect("play natural attack");
            }
            if game.stage != Stage::Combat { break; }
            game.apply(Action::EndTurn).expect("normal combat turn");
        }
        if game.stage == Stage::Reward {
            winning_game = Some(game);
            break;
        }
    }
    let game = winning_game.expect("at least one normal seed should break out within four turns");
    assert!(game.gold > 90, "winning breakout must pay ordinary combat rewards");
    game.verify_action_replay().expect("successful action replay");
    let snapshot = game.to_json().unwrap();
    assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(), snapshot);
}
