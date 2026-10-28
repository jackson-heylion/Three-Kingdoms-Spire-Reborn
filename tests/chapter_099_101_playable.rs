//! Chapter 099–101: pinned source evidence, atomic choices, mutually exclusive flags, V2 and rewards.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 99101);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}
#[test]
fn pinned_full_reading_and_partial_history_are_distinct() {
    for (chapter, oldid, doc) in [
        (99, "2568060", include_str!("../docs/CHAPTER_099_READING.md")),
        (100, "2568063", include_str!("../docs/CHAPTER_100_READING.md")),
        (101, "2568067", include_str!("../docs/CHAPTER_101_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_099_101_HISTORY_SPOTCHECK.md");
    for oldid in ["2115776", "2390639", "2583390"] {
        assert!(history.contains(oldid), "{oldid}");
    }
    assert!(history.contains("裴松之") && history.contains("0/120"));
    let rows: Vec<_> = include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(102).skip(99) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    for (id, chapter) in [
        ("wudu_yinping_civil_register", 99),
        ("qishan_rain_rations", 99),
        ("jigu_defiance_casualties", 100),
        ("wei_caozhen_death_archive", 100),
        ("lucheng_wheat_household", 101),
        ("liyan_dispatch_evidence", 101),
    ] {
        assert_eq!(beat(id).unwrap().chapter, chapter);
    }
}
#[test]
fn two_county_to_wheat_award_is_once_only_after_acceptance_and_v2_restore() {
    let mut game = scene(Hero::ZhugeLiang, "wudu_yinping_civil_register", &["redcliff_wind"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "wudu_lucheng_civil_debt"), 0);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "lucheng_wheat_household");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "wudu_lucheng_civil_debt"), 1);
    enter(&mut restored, "lucheng_wheat_household");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "wudu_lucheng_civil_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "wudu_yinping_civil_register", &["redcliff_wind"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "lucheng_wheat_household");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "wudu_lucheng_civil_debt"), 0);
    assert!(!refused.lore_flags.iter().any(|f| f == "wudu_yinping_civil_record"));
}
#[test]
fn jigu_loss_and_liyan_two_reports_require_both_records() {
    let mut game = scene(Hero::ZhugeLiang, "jigu_defiance_casualties", &["redcliff_wind"]);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "jigu_liyan_dispatch_debt"), 0);
    enter(&mut game, "liyan_dispatch_evidence");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "jigu_liyan_dispatch_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "jigu_defiance_casualties", &["redcliff_wind"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "liyan_dispatch_evidence");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "jigu_liyan_dispatch_debt"), 0);
}
#[test]
fn posthumous_wei_archive_and_legacy_wuzhang_route_remain_available() {
    let mut game = scene(Hero::CaoCao, "wei_caozhen_death_archive", &["guandu_grain"]);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "wei_caozhen_archive_record"));
    let flags = vec!["redcliff_wind".to_owned()];
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &[]).map(|b| b.id), Some("wuzhang"));
    let seen: Vec<String> = BEATS.iter().filter(|b| b.id != "wudu_yinping_civil_register")
        .map(|b| b.id.to_owned()).collect();
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &seen).map(|b| b.id), Some("wudu_yinping_civil_register"));
}
