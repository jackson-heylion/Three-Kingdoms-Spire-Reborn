//! Chapter 19: counterfactual choices tested through real Game::apply.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 19026);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    game
}
fn count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn new_paths_are_optional_and_old_arcs_remain_reachable() {
    let lu = beat("xiapi_orders").unwrap();
    assert_eq!((lu.chapter, lu.act, lu.prerequisite), (19, 1, Some("xiaopei_courier_guard")));
    let cao = beat("xiapi_flood").unwrap();
    assert_eq!((cao.chapter, cao.act, cao.prerequisite), (19, 2, Some("anzhong_field_report")));
    assert_eq!(next_beat(Hero::LuBu, 1,
        &["halberd_shot".into(), "xiaopei_courier_guard".into()],
        &["halberd".into(), "seven_armies".into(), "xiaopei_courier".into()])
        .unwrap().id, "xiapi_orders");
    assert_eq!(next_beat(Hero::LuBu, 2, &["halberd_shot".into()], &[])
        .unwrap().id, "whitegate");
    assert_eq!(next_beat(Hero::CaoCao, 2, &["guandu_grain".into()], &[])
        .unwrap().id, "liaodong");
}

#[test]
fn lu_orders_survive_save_and_pay_once_at_white_gate() {
    let mut game = scene(Hero::LuBu, "xiapi_orders",
        &["halberd_shot", "xiaopei_courier_guard"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    let gold = game.gold;
    let deck = game.deck.len();
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|s| s == "xiapi_orders_account"));
    assert_eq!(game.deck.len(), deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "hold_line");
    assert_eq!(game.gold, gold + 18);
    assert!(matches!(game.stage, Stage::Map));
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    restored.stage = Stage::Story;
    restored.lore_current = Some("whitegate".into());
    let message = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(message.contains("下邳失令军簿"));
    assert_eq!(count(&restored, "xiapi_orders_debt"), 1);
    restored.stage = Stage::Story;
    restored.lore_current = Some("whitegate".into());
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "xiapi_orders_debt"), 1);
}

#[test]
fn lu_refuses_orders_but_white_gate_still_works() {
    let mut game = scene(Hero::LuBu, "xiapi_orders",
        &["halberd_shot", "xiaopei_courier_guard"]);
    let deck = game.deck.len();
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|s| s == "xiapi_orders_account"));
    assert_eq!(game.deck.len(), deck);
    game.stage = Stage::Story;
    game.lore_current = Some("whitegate".into());
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "xiapi_orders_debt"), 0);
}

#[test]
fn cao_water_report_is_atomic_and_pays_once_after_guandu() {
    let mut game = scene(Hero::CaoCao, "xiapi_flood",
        &["shouchun_grain_account", "anzhong_field_report", "guandu_grain"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    let deck = game.deck.len();
    let message = game.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(message.contains("决水安民军报"));
    assert!(game.lore_flags.iter().any(|s| s == "xiapi_flood_ledger"));
    assert_eq!(game.deck.len(), deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "granary_support");
    assert_eq!(count(&game, "xiapi_water_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    restored.stage = Stage::Story;
    restored.lore_current = Some("xiapi_flood".into());
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "xiapi_water_debt"), 1);
}

#[test]
fn cao_victory_only_choice_cannot_claim_flood_ledger() {
    let mut game = scene(Hero::CaoCao, "xiapi_flood",
        &["shouchun_grain_account", "anzhong_field_report", "guandu_grain"]);
    let deck = game.deck.len();
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(game.deck.len(), deck);
    assert!(!game.lore_flags.iter().any(|s| s == "xiapi_flood_ledger"));
    assert_eq!(count(&game, "xiapi_water_debt"), 0);
}
