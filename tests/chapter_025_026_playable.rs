//! Chapters 25–26: pinned single-text evidence and optional choices through Huarong.
//! Player choices record counterfactual responsibility; they never rewrite the novel.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn scene(id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(Hero::GuanYu, 25026);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|flag| (*flag).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn earned(game: &Game, id: &str) -> bool {
    game.lore_flags.iter().any(|flag| flag == id)
}
fn count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|flag| flag.as_str() == id).count()
}

#[test]
fn source_chapters_and_legacy_huarong_remain_distinct() {
    let terms = beat("baima_terms").unwrap();
    let departure = beat("seal_return").unwrap();
    assert_eq!((terms.chapter, terms.act, terms.prerequisite),
        (25, 1, Some("wenjiu_oath")));
    assert_eq!((departure.chapter, departure.act, departure.prerequisite),
        (26, 1, Some("three_terms_record")));
    assert!(terms.premise.contains("三约") && terms.premise.contains("颜良"));
    assert!(departure.premise.contains("文丑") && departure.premise.contains("陈震"));
    assert!(departure.premise.contains("蔡阳") && departure.premise.contains("孙乾"));
    assert_eq!(beat("baima").unwrap().chapter, 25);
    assert_eq!(beat("huarong").unwrap().prerequisite, Some("baima_debt"));
    assert_eq!(next_beat(Hero::GuanYu, 1, &["baima_debt".into()], &[])
        .map(|beat| beat.id), Some("huarong"),
        "a legacy save must not need chapter 25 or 26 to reach Huarong");
    assert_eq!(next_beat(Hero::GuanYu, 1, &["baima_debt".into(), "wenjiu_oath".into()],
        &["xutian_guard".into(), "xuzhou_gate".into(),
          "wangzhong_prisoners".into(), "xiapi_household".into()])
        .map(|beat| beat.id), Some("baima_terms"));
}

#[test]
fn three_terms_and_seal_return_unlock_once_after_huarong() {
    let mut game = scene("baima_terms", &["baima_debt", "wenjiu_oath"]);
    let original = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), original,
        "rejected action must preserve RNG, revision, flags and deck");
    let before_deck = game.deck.len();
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(earned(&game, "three_terms_record"));
    assert_eq!(game.deck.len(), before_deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "hold_line");
    enter(&mut game, "seal_return");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(earned(&game, "seal_return_record"));
    assert_eq!(count(&game, "three_terms_huarong"), 0);
    let snapshot = game.to_json().unwrap();
    let mut restored = Game::from_json(&snapshot).unwrap();
    assert_eq!(restored.to_json().unwrap(), snapshot);
    enter(&mut restored, "huarong");
    let output = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(output.contains("三约挂印"));
    assert_eq!(count(&restored, "three_terms_huarong"), 1);
    enter(&mut restored, "huarong");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "three_terms_huarong"), 1);
}

#[test]
fn declining_either_new_account_keeps_the_old_debt_and_avoids_false_rewards() {
    let mut first = scene("baima_terms", &["baima_debt", "wenjiu_oath"]);
    first.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!earned(&first, "three_terms_record"));
    assert_ne!(next_beat(Hero::GuanYu, 1, &first.lore_flags, &first.lore_seen)
        .map(|beat| beat.id), Some("seal_return"));
    enter(&mut first, "huarong");
    first.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&first, "sword_debt"), 1);
    assert_eq!(count(&first, "three_terms_huarong"), 0);

    let mut second = scene("seal_return",
        &["baima_debt", "wenjiu_oath", "three_terms_record"]);
    let snapshot = second.to_json().unwrap();
    assert!(second.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(second.to_json().unwrap(), snapshot);
    second.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!earned(&second, "seal_return_record"));
    enter(&mut second, "huarong");
    second.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&second, "sword_debt"), 1);
    assert_eq!(count(&second, "three_terms_huarong"), 0);
}

#[test]
fn chapter_26_choice_is_never_forced_on_legacy_or_other_heroes() {
    assert_ne!(next_beat(Hero::GuanYu, 1, &["wenjiu_oath".into()], &[])
        .map(|beat| beat.id), Some("seal_return"));
    assert_ne!(next_beat(Hero::CaoCao, 1,
        &["three_terms_record".into(), "baima_debt".into()], &[])
        .map(|beat| beat.id), Some("seal_return"));
    let mut old = scene("huarong", &["baima_debt"]);
    old.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&old, "sword_debt"), 1);
    assert_eq!(count(&old, "three_terms_huarong"), 0);
}
