//! Chapter 18: two separate causal chains tested through real Game::apply.
//! The player's counterfactual choices are not claims about the novel or history.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 18026);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    game
}

fn count_egg(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn chapter_18_is_available_only_after_its_own_earlier_choices() {
    let cao = beat("anzhong").unwrap();
    assert_eq!((cao.chapter, cao.act, cao.prerequisite), (18, 1, Some("shouchun_grain_account")));
    let lu = beat("xiaopei_courier").unwrap();
    assert_eq!((lu.chapter, lu.act, lu.prerequisite), (18, 1, Some("seven_armies_account")));

    let cao_flags = vec!["shouchun_grain_account".into(), "xu_capital".into()];
    let cao_seen = vec!["shouchun_grain".into()];
    assert_eq!(next_beat(Hero::CaoCao, 1, &cao_flags, &cao_seen).unwrap().id, "anzhong");
    assert_eq!(next_beat(Hero::CaoCao, 1, &["xu_capital".into()], &cao_seen)
        .unwrap().id, "guandu", "old Guandu path must not require chapter 18");

    let lu_flags = vec!["halberd_shot".into(), "seven_armies_account".into()];
    let lu_seen = vec!["halberd".into(), "seven_armies".into()];
    assert_eq!(next_beat(Hero::LuBu, 1, &lu_flags, &lu_seen).unwrap().id, "xiaopei_courier");
    assert_eq!(next_beat(Hero::LuBu, 2, &["halberd_shot".into()], &[])
        .unwrap().id, "whitegate", "old White Gate path must not require chapter 18");
}

#[test]
fn cao_complete_report_is_atomic_persistent_and_paid_once_at_guandu() {
    let mut game = scene(Hero::CaoCao, "anzhong",
        &["puyang_supply", "shouchun_grain_account", "xu_capital"]);
    let original = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), original);
    let gold = game.gold;
    let deck = game.deck.len();
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|s| s == "anzhong_field_report"));
    assert!(game.lore_seen.iter().any(|s| s == "anzhong"));
    assert_eq!(game.deck.len(), deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "qimou");
    assert_eq!(game.gold, gold + 16);
    assert!(matches!(game.stage, Stage::Map));

    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    restored.stage = Stage::Story;
    restored.lore_current = Some("guandu".into());
    let outcome = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(outcome.contains("安众虚实军报"));
    assert_eq!(count_egg(&restored, "anzhong_reckoning"), 1);
    restored.stage = Stage::Story;
    restored.lore_current = Some("guandu".into());
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count_egg(&restored, "anzhong_reckoning"), 1);
}

#[test]
fn cao_selective_report_does_not_claim_the_later_payoff() {
    let mut game = scene(Hero::CaoCao, "anzhong",
        &["shouchun_grain_account", "xu_capital"]);
    let deck = game.deck.len();
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|s| s == "anzhong_field_report"));
    assert_eq!(game.deck.len(), deck);
    game.stage = Stage::Story;
    game.lore_current = Some("guandu".into());
    let outcome = game.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(!outcome.contains("安众虚实军报"));
    assert_eq!(count_egg(&game, "anzhong_reckoning"), 0);
}

#[test]
fn lu_courier_mercy_persists_and_is_paid_once_at_white_gate() {
    let mut game = scene(Hero::LuBu, "xiaopei_courier",
        &["halberd_shot", "seven_armies_account"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    let deck = game.deck.len();
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|s| s == "xiaopei_courier_guard"));
    assert_eq!(game.deck.len(), deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "hold_line");

    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    restored.stage = Stage::Story;
    restored.lore_current = Some("whitegate".into());
    let outcome = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(outcome.contains("小沛留使旧账"));
    assert_eq!(count_egg(&restored, "xiaopei_courier_reckoning"), 1);
    restored.stage = Stage::Story;
    restored.lore_current = Some("whitegate".into());
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count_egg(&restored, "xiaopei_courier_reckoning"), 1);
}

#[test]
fn lu_killing_the_courier_does_not_lock_white_gate_or_unlock_mercy() {
    let mut game = scene(Hero::LuBu, "xiaopei_courier",
        &["halberd_shot", "seven_armies_account"]);
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|s| s == "xiaopei_courier_guard"));
    game.stage = Stage::Story;
    game.lore_current = Some("whitegate".into());
    let outcome = game.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(!outcome.contains("小沛留使旧账"));
    assert_eq!(count_egg(&game, "xiaopei_courier_reckoning"), 0);
}
