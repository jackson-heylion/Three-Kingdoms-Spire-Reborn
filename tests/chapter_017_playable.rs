//! Chapter 17: real Game::apply outcomes and cross-act flags, not full-run autoplay.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 17026);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|flag| (*flag).into()).collect();
    game
}

#[test]
fn chapter_17_requires_the_previous_choice_but_does_not_block_existing_routes() {
    assert_eq!(beat("seven_armies").unwrap().prerequisite, Some("halberd_shot"));
    assert_eq!(beat("shouchun_grain").unwrap().prerequisite, Some("puyang_supply"));
    assert_eq!(beat("seven_armies").unwrap().chapter, 17);
    assert_eq!(beat("shouchun_grain").unwrap().chapter, 17);

    let none: Vec<String> = Vec::new();
    assert!(next_beat(Hero::LuBu, 1, &none, &none).is_none());
    assert!(next_beat(Hero::CaoCao, 1, &none, &none).is_none());

    let lu_flags = vec!["phoenix_pavilion".into(), "halberd_shot".into()];
    assert_eq!(next_beat(Hero::LuBu, 1, &lu_flags, &none).unwrap().id, "halberd");
    assert_eq!(next_beat(Hero::LuBu, 1, &lu_flags, &["halberd".into()]).unwrap().id,
        "seven_armies");
    assert_eq!(next_beat(Hero::LuBu, 2, &["halberd_shot".into()], &none)
        .unwrap().id, "whitegate", "new chapter 17 flag must not gate White Gate");

    let cao_flags = vec!["puyang_supply".into(), "xu_capital".into()];
    assert_eq!(next_beat(Hero::CaoCao, 1, &cao_flags, &none).unwrap().id,
        "shouchun_grain");
    assert_eq!(next_beat(Hero::CaoCao, 1, &["xu_capital".into()], &none)
        .unwrap().id, "guandu", "missing chapter 17 must not gate Guandu");
}

#[test]
fn seven_armies_choice_commits_card_flag_and_single_white_gate_payoff() {
    let mut game = scene(Hero::LuBu, "seven_armies", &["halberd_shot"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before, "invalid choice is fully atomic");

    let original_gold = game.gold;
    let original_deck = game.deck.len();
    let message = game.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(message.contains("七路军书"));
    assert!(game.lore_flags.iter().any(|flag| flag == "seven_armies_account"));
    assert!(game.lore_seen.iter().any(|id| id == "seven_armies"));
    assert_eq!(game.deck.len(), original_deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "hold_line");
    assert_eq!(game.gold, original_gold + 14);
    assert!(matches!(game.stage, Stage::Map));

    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    restored.stage = Stage::Story;
    restored.lore_current = Some("whitegate".into());
    let payoff = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(payoff.contains("辕门军书后账"));
    assert_eq!(restored.easter_unlocked.iter().filter(|id| *id == "seven_armies_debt").count(), 1);
    restored.stage = Stage::Story;
    restored.lore_current = Some("whitegate".into());
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(restored.easter_unlocked.iter().filter(|id| *id == "seven_armies_debt").count(), 1);
}

#[test]
fn cashing_out_seven_armies_cannot_claim_the_later_responsibility() {
    let mut game = scene(Hero::LuBu, "seven_armies", &["halberd_shot"]);
    let before_deck = game.deck.len();
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|flag| flag == "seven_armies_account"));
    assert_eq!(game.deck.len(), before_deck);
    game.stage = Stage::Story;
    game.lore_current = Some("whitegate".into());
    let narration = game.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(!narration.contains("辕门军书后账"));
    assert!(!game.easter_unlocked.iter().any(|id| id == "seven_armies_debt"));
}

#[test]
fn shouchun_accountability_survives_save_and_reconciles_at_guandu() {
    let mut game = scene(Hero::CaoCao, "shouchun_grain",
        &["puyang_supply", "xuzhou_debt", "xu_capital"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    let original_deck = game.deck.len();
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|flag| flag == "shouchun_grain_account"));
    assert_eq!(game.deck.len(), original_deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "granary_support");
    let json = game.to_json().unwrap();
    let mut restored = Game::from_json(&json).unwrap();
    assert_eq!(restored.to_json().unwrap(), json);
    restored.stage = Stage::Story;
    restored.lore_current = Some("guandu".into());
    let message = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(message.contains("寿春粮秤旧案"));
    assert_eq!(restored.easter_unlocked.iter().filter(|id| *id == "shouchun_grain_debt").count(), 1);
    restored.stage = Stage::Story;
    restored.lore_current = Some("guandu".into());
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(restored.easter_unlocked.iter().filter(|id| *id == "shouchun_grain_debt").count(), 1);
}

#[test]
fn cashing_out_shouchun_leaves_original_guandu_unlocked() {
    let mut game = scene(Hero::CaoCao, "shouchun_grain",
        &["puyang_supply", "xu_capital"]);
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|flag| flag == "shouchun_grain_account"));
    assert_eq!(next_beat(Hero::CaoCao, 1, &game.lore_flags, &game.lore_seen)
        .unwrap().id, "guandu");
}
