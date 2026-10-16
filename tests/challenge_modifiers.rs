//! Regression coverage for opt-in custom difficulty and save/replay boundaries.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, Stage, Node, challenges::ChallengeModifier, content::Hero,
};

fn set(game: &mut Game, id: &str, enabled: bool) {
    game.apply(Action::SetModifier { id: id.into(), enabled }).unwrap();
}

#[test]
fn settings_are_sorted_idempotent_atomic_and_replayable() {
    let mut game = Game::new_with_mode(Hero::GuanYu, 771, 0, RunMode::Custom);
    let snapshot = game.to_json().unwrap();
    assert!(game.apply(Action::SetModifier { id: "typo".into(), enabled: true }).is_err());
    assert_eq!(game.to_json().unwrap(), snapshot);
    set(&mut game, "thin_supplies", true);
    set(&mut game, "no_rest", true);
    set(&mut game, "iron_legion", true);
    set(&mut game, "no_rest", true);
    assert_eq!(game.modifiers, ChallengeModifier::ALL);
    let json = game.to_json().unwrap();
    assert_eq!(Game::from_json(&json).unwrap().modifiers, ChallengeModifier::ALL);
    assert_eq!(game.verify_action_replay().unwrap(), 4);
    set(&mut game, "no_rest", false);
    assert_eq!(game.modifiers.len(), 2);
    assert!(game.verify_action_replay().is_ok());
}

#[test]
fn battle_modifiers_are_effective_not_just_labels() {
    let mut normal = Game::new_with_mode(Hero::CaoCao, 19, 0, RunMode::Custom);
    let mut harder = Game::new_with_mode(Hero::CaoCao, 19, 0, RunMode::Custom);
    set(&mut harder, "iron_legion", true);
    set(&mut harder, "thin_supplies", true);
    assert_eq!(normal.route[0], Node::Battle);
    assert_eq!(harder.route[0], Node::Battle);
    normal.apply(Action::Choose { index: 1 }).unwrap();
    harder.apply(Action::Choose { index: 1 }).unwrap();
    let n = normal.battle.as_ref().unwrap();
    let h = harder.battle.as_ref().unwrap();
    assert_eq!(n.foes.len(), h.foes.len());
    assert_eq!(h.energy + 1, n.energy);
    for (nf, hf) in n.foes.iter().zip(&h.foes) {
        assert_eq!(hf.block, nf.block + 8);
        assert_eq!(hf.hp, nf.hp);
    }
    let before = harder.to_json().unwrap();
    assert!(harder.apply(Action::SetModifier { id: "iron_legion".into(), enabled: false }).is_err());
    assert_eq!(harder.to_json().unwrap(), before);
    assert!(harder.verify_action_replay().is_ok());
}

#[test]
fn camp_restriction_refuses_without_spending_revision_or_hp() {
    let mut game = Game::new_with_mode(Hero::SunShangxiang, 27, 0, RunMode::Custom);
    set(&mut game, "no_rest", true);
    game.stage = Stage::Camp;
    game.hp -= 8;
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 1 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    assert!(game.apply(Action::Choose { index: 3 }).is_ok());
}

#[test]
fn ranked_and_daily_modes_cannot_enable_modifiers() {
    for mode in [RunMode::Standard, RunMode::Daily] {
        let mut game = Game::new_with_mode(Hero::LuBu, 28, 0, mode);
        let snapshot = game.to_json().unwrap();
        assert!(game.apply(Action::SetModifier { id: "no_rest".into(), enabled: true }).is_err());
        assert_eq!(game.to_json().unwrap(), snapshot);
        let mut raw: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
        raw["modifiers"] = serde_json::json!(["no_rest"]);
        assert!(Game::from_json(&raw.to_string()).is_err());
    }
}
