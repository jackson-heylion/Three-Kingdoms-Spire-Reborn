//! Public-seam regression for the opt-in gameplay-expansion difficulty identity.
//! Legacy 21-floor campaigns must retain their original deterministic rule set.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, content::Hero};

fn first_battle(mut game: Game) -> Game {
    assert_eq!(game.route.first(), Some(&Node::Battle));
    game.apply(Action::Choose { index: 1 }).expect("first ordinary battle");
    game
}

#[test]
fn three_new_difficulties_are_real_playable_and_replayable() {
    assert_eq!(Difficulty::ALL.len(), 3);
    assert!(Difficulty::parse("unknown").is_none());
    for (i, difficulty) in Difficulty::ALL.iter().copied().enumerate() {
        assert_eq!(Difficulty::parse(difficulty.id()), Some(difficulty));
        let game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 50_100 + i as u64, difficulty, format!("new-difficulty-{i}"),
        );
        assert_eq!(game.difficulty, Some(difficulty));
        assert_eq!(game.ascension, 0);
        assert_eq!(Game::from_json(&game.to_json().unwrap()).unwrap().difficulty, Some(difficulty));
        let game = first_battle(game);
        assert!(game.battle.is_some());
        game.audit_campaign_authority().expect("campaign identity journal");
        game.verify_action_replay().expect("seed and action replay");
    }
}

#[test]
fn new_tiers_impose_increasing_first_encounter_pressure() {
    let legacy = first_battle(Game::new_with_ascension(Hero::CaoCao, 419, 0));
    let warlords = first_battle(Game::new_with_campaign_difficulty(
        Hero::CaoCao, 419, Difficulty::Warlords, "warlords".into(),
    ));
    let conqueror = first_battle(Game::new_with_campaign_difficulty(
        Hero::CaoCao, 419, Difficulty::Conqueror, "conqueror".into(),
    ));
    let destiny = first_battle(Game::new_with_campaign_difficulty(
        Hero::CaoCao, 419, Difficulty::Destiny, "destiny".into(),
    ));
    let hp = |game: &Game| game.battle.as_ref().unwrap().foes[0].max_hp;
    assert!(hp(&legacy) < hp(&warlords));
    assert!(hp(&warlords) < hp(&conqueror));
    assert!(hp(&conqueror) < hp(&destiny));
    for game in [&warlords, &conqueror, &destiny] {
        game.verify_action_replay().expect("difficulty is part of replay identity");
    }
}

#[test]
fn old_ascension_save_does_not_gain_new_difficulty() {
    let old = first_battle(Game::new_with_ascension(Hero::ZhugeLiang, 87, 6));
    assert!(old.difficulty.is_none());
    let serialized = old.to_json().unwrap();
    let restored = Game::from_json(&serialized).unwrap();
    assert_eq!(restored.ascension, 6);
    assert_eq!(restored.difficulty, None);
    assert_eq!(restored.to_json().unwrap(), serialized);
    restored.verify_action_replay().unwrap();
}

#[test]
fn new_tier_event_archive_reconstructs_after_action_and_json_resume() {
    for tier in Difficulty::ALL {
        let game = first_battle(Game::new_with_campaign_difficulty(
            Hero::CaoCao, 1945, tier, format!("event-replay-{}", tier.id()),
        ));
        let saved = game.to_json().unwrap();
        let restored = Game::from_json(&saved).expect("new tier save can resume");
        let events = restored.reconstruct_event_history()
            .expect("event archive must replay with the selected campaign difficulty");
        assert_eq!(events.len(), restored.event_history.as_ref().unwrap().len());
        assert_eq!(restored.to_json().unwrap(), saved, "event verification is read-only");
    }
}
