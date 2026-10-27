//! P1 literature chapters 096–098: source pins, real choices and atomic/legacy checks.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 9698);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn reward_count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}
#[test]
fn pinned_full_reading_is_distinct_from_partial_history_and_missing_second_base() {
    for (chapter, oldid, doc) in [
        (96, "2568062", include_str!("../docs/CHAPTER_096_READING.md")),
        (97, "2568059", include_str!("../docs/CHAPTER_097_READING.md")),
        (98, "2568061", include_str!("../docs/CHAPTER_098_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_096_098_HISTORY_SPOTCHECK.md");
    for id in ["2115776", "2602668", "2274283", "2583383"] {
        assert!(history.contains(id), "{id}");
    }
    assert!(history.contains("裴松之") && history.contains("0/120"));
    let rows: Vec<_> = include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(99).skip(96) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    for (id, chapter, flag) in [
        ("masu_family_account", 96, "masu_family_record"),
        ("shiting_false_defection_audit", 96, "shiting_spy_record"),
        ("second_memorial_supply_review", 97, "second_memorial_supply_record"),
        ("chencang_siege_wounded_ledger", 97, "chencang_siege_record"),
        ("chencang_retreat_supply_audit", 98, "chencang_retreat_supply_record"),
        ("wu_han_enthronement_dispatch", 98, "wu_han_enthronement_record"),
    ] {
        let beat = beat(id).unwrap();
        assert_eq!(beat.chapter, chapter);
        assert_eq!(beat.flag, flag);
    }
}
#[test]
fn jieting_family_debt_is_atomic_exclusive_restorable_and_once_only() {
    let mut game = scene(Hero::ZhugeLiang, "masu_family_account", &["jieting_water_record"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&game, "jieting_masu_family_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "masu_family_account");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&restored, "jieting_masu_family_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "masu_family_account", &["jieting_water_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(reward_count(&refused, "jieting_masu_family_debt"), 0);
    assert!(!refused.lore_flags.iter().any(|f| f == "masu_family_record"));
}
#[test]
fn chencang_two_stage_reward_requires_both_accepted_records() {
    let mut game = scene(Hero::ZhugeLiang, "chencang_siege_wounded_ledger", &["redcliff_wind"]);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&game, "redcliff_chencang_retreat_debt"), 0);
    enter(&mut game, "chencang_retreat_supply_audit");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&game, "redcliff_chencang_retreat_debt"), 1);
    enter(&mut game, "chencang_retreat_supply_audit");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&game, "redcliff_chencang_retreat_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "chencang_siege_wounded_ledger", &["redcliff_wind"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "chencang_retreat_supply_audit");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&refused, "redcliff_chencang_retreat_debt"), 0);
}
#[test]
fn wu_posthumous_archives_and_legacy_routes_are_preserved() {
    let mut game = scene(Hero::SunShangxiang, "shiting_false_defection_audit", &["jiangdong_covenant"]);
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "wu_han_enthronement_dispatch");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "shiting_spy_record"));
    assert!(!game.lore_flags.iter().any(|f| f == "wu_han_enthronement_record"));
    let seen: Vec<String> = BEATS.iter()
        .filter(|b| b.id != "masu_family_account")
        .map(|b| b.id.to_owned()).collect();
    let flags = vec!["redcliff_wind".to_owned(), "jieting_water_record".to_owned()];
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &[]).map(|b| b.id), Some("wuzhang"));
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &seen).map(|b| b.id), Some("masu_family_account"));
}
