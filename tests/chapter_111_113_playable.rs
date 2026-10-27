//! P1④ 111–113: pinned full readings, successor archives, legacy safety and once-only debts.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 111113);
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
fn three_readings_are_pinned_and_history_is_not_marked_complete() {
    for (chapter, oldid, doc) in [
        (111, "2568075", include_str!("../docs/CHAPTER_111_READING.md")),
        (112, "2568074", include_str!("../docs/CHAPTER_112_READING.md")),
        (113, "2568076", include_str!("../docs/CHAPTER_113_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_111_113_HISTORY_SPOTCHECK.md");
    assert!(history.contains("2589674") && history.contains("裴注") && history.contains("0/120"));
    let rows: Vec<_> = include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(114).skip(111) {
        assert!(row.starts_with(&format!("{chapter:03}\t")), "{chapter}");
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    for (id, chapter) in [
        ("dangu_casualty_roll", 111),
        ("zhugedan_wu_hostage_receipt", 111),
        ("wei_huainan_summons_audit", 111),
        ("shouchun_city_captives", 112),
        ("shouchun_wu_relief", 112),
        ("qiaozhou_war_counsel", 112),
        ("sunxiu_court_review", 113),
        ("qishan_retreat_courier", 113),
    ] { assert_eq!(beat(id).unwrap().chapter, chapter); }
}

#[test]
fn dangu_debt_is_atomic_mutually_exclusive_and_persistent() {
    let mut game = scene(Hero::ZhugeLiang, "dangu_casualty_roll", &["taoxi_didao_withdrawal_record"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "taoxi_dangu_casualty_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    enter(&mut restored, "dangu_casualty_roll");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "taoxi_dangu_casualty_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "dangu_casualty_roll", &["taoxi_didao_withdrawal_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!refused.lore_flags.iter().any(|s| s == "dangu_casualty_roll_record"));
    assert_eq!(count(&refused, "taoxi_dangu_casualty_debt"), 0);
}

#[test]
fn huainan_and_wu_succession_rewards_follow_only_accepted_choices() {
    let mut wei = scene(Hero::CaoCao, "wei_huainan_summons_audit", &["shouchun_false_edict_record"]);
    wei.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut wei, "shouchun_city_captives");
    wei.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&wei, "huainan_shouchun_captives_debt"), 1);
    let mut wu = scene(Hero::SunShangxiang, "zhugedan_wu_hostage_receipt", &["dongxing_snow_bridge_record"]);
    for id in ["zhugedan_wu_hostage_receipt", "shouchun_wu_relief", "sunxiu_court_review"] {
        enter(&mut wu, id);
        wu.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&wu, "dongxing_sunxiu_court_debt"), 1);
    let saved = wu.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}

#[test]
fn old_main_story_wins_and_late_archives_are_independently_selectable() {
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &["redcliff_wind".into()], &[]).map(|b| b.id), Some("wuzhang"));
    assert_eq!(next_beat(Hero::CaoCao, 2, &["guandu_grain".into()], &[]).map(|b| b.id), Some("liaodong"));
    assert_eq!(next_beat(Hero::SunShangxiang, 2, &["jiangdong_covenant".into()], &[]).map(|b| b.id), Some("yiling"));
    for (hero, id, prerequisite) in [
        (Hero::ZhugeLiang, "dangu_casualty_roll", "taoxi_didao_withdrawal_record"),
        (Hero::CaoCao, "wei_huainan_summons_audit", "shouchun_false_edict_record"),
        (Hero::SunShangxiang, "zhugedan_wu_hostage_receipt", "dongxing_snow_bridge_record"),
    ] {
        let seen: Vec<String> = BEATS.iter().filter(|b| b.id != id).map(|b| b.id.into()).collect();
        assert_eq!(next_beat(hero, 2, &[prerequisite.into()], &seen).map(|b| b.id), Some(id));
    }
    for seed in 111113..111121 {
        let json = Game::new(Hero::SunShangxiang, seed).to_json().unwrap();
        assert!(Game::from_json(&json).is_ok());
    }
}
