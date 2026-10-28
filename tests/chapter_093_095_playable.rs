//! Chapters 093–095: pinned full text, distinct history layers and actual story choices.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 9395);
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
fn three_pinned_full_readings_are_distinct_from_partial_history_and_old_spotcheck() {
    for (chapter, oldid, doc) in [
        (93, "2568056", include_str!("../docs/CHAPTER_093_READING.md")),
        (94, "2568057", include_str!("../docs/CHAPTER_094_READING.md")),
        (95, "2568058", include_str!("../docs/CHAPTER_095_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_093_095_HISTORY_SPOTCHECK.md");
    for id in ["12068919", "2331498", "2390639", "2115776"] {
        assert!(history.contains(id), "{id}");
    }
    assert!(history.contains("裴注") && history.contains("0/120"));
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    let rows: Vec<_> = ledger.lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(96).skip(93) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    assert!(rows[95].contains("片段抽检升级"));
    for (id, chapter, flag) in [
        ("tianshui_recruitment_receipt", 93, "tianshui_recruit_record"),
        ("wanglang_death_archive", 93, "wanglang_archive_record"),
        ("qiang_captive_return", 94, "qiang_captive_record"),
        ("xincheng_secret_dispatch", 94, "xincheng_secret_record"),
        ("jieting_water_order", 95, "jieting_water_record"),
        ("xicheng_civilian_retreat", 95, "xicheng_civilian_record"),
    ] {
        let beat = beat(id).unwrap();
        assert_eq!(beat.chapter, chapter);
        assert_eq!(beat.flag, flag);
    }
}

#[test]
fn zhuge_choices_are_atomic_exclusive_restorable_and_hidden_rewards_once_only() {
    let mut game = scene(Hero::ZhugeLiang, "tianshui_recruitment_receipt",
        &["redcliff_wind", "tianshui_false_dispatch_record"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&game, "tianshui_truth_debt"), 1);
    enter(&mut game, "qiang_captive_return");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|f| f == "qiang_captive_record"));
    enter(&mut game, "jieting_water_order");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&game, "redcliff_jieting_supply_debt"), 1);
    enter(&mut game, "xicheng_civilian_retreat");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "xicheng_civilian_record"));
    let snapshot = game.to_json().unwrap();
    let mut restored = Game::from_json(&snapshot).unwrap();
    assert_eq!(restored.to_json().unwrap(), snapshot);
    enter(&mut restored, "jieting_water_order");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&restored, "tianshui_truth_debt"), 1);
    assert_eq!(reward_count(&restored, "redcliff_jieting_supply_debt"), 1);
}

#[test]
fn refusing_jieting_evidence_never_unlocks_the_retreat_chain() {
    let mut game = scene(Hero::ZhugeLiang, "jieting_water_order", &["redcliff_wind"]);
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|f| f == "jieting_water_record"));
    assert_eq!(reward_count(&game, "redcliff_jieting_supply_debt"), 0);
    enter(&mut game, "xicheng_civilian_retreat");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&game, "redcliff_jieting_supply_debt"), 0);
}

#[test]
fn cao_archives_are_posthumous_optional_and_old_story_routes_win() {
    let mut game = scene(Hero::CaoCao, "wanglang_death_archive", &["guandu_grain"]);
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "xincheng_secret_dispatch");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "wanglang_archive_record"));
    assert!(!game.lore_flags.iter().any(|f| f == "xincheng_secret_record"));
    let flags = vec!["redcliff_wind".to_owned()];
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &[]).map(|b| b.id), Some("wuzhang"));
    let seen: Vec<String> = BEATS.iter()
        .filter(|b| b.id != "jieting_water_order")
        .map(|b| b.id.to_owned()).collect();
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &seen).map(|b| b.id),
        Some("jieting_water_order"));
}
