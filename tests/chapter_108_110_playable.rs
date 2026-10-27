//! P1④ 108–110: pinned full readings, successor choices, atomicity and old-save safety.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 108110);
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
fn three_pinned_full_readings_and_partial_history_are_explicit() {
    for (chapter, oldid, doc) in [
        (108, "2568069", include_str!("../docs/CHAPTER_108_READING.md")),
        (109, "2568073", include_str!("../docs/CHAPTER_109_READING.md")),
        (110, "2568072", include_str!("../docs/CHAPTER_110_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_108_110_HISTORY_SPOTCHECK.md");
    for oldid in ["2085142", "2263487", "8746774", "12068919", "2386518"] {
        assert!(history.contains(oldid), "{oldid}");
    }
    assert!(history.contains("裴注") && history.contains("0/120"));
    let rows: Vec<_> = include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(111).skip(108) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    for (id, chapter) in [
        ("qushan_crossbow_aftercare", 108),
        ("dongxing_snow_bridge_ledger", 108),
        ("xincheng_siege_sickness", 108),
        ("sun_jun_banquet_inquest", 108),
        ("tielong_spring_rescue", 109),
        ("caofang_deposition_receipt", 109),
        ("shouchun_false_edict_audit", 110),
        ("taoxi_didao_withdrawal", 110),
    ] {
        assert_eq!(beat(id).unwrap().chapter, chapter);
    }
}

#[test]
fn dongxing_rescue_is_atomic_persistent_mutually_exclusive_and_once_only() {
    let mut game = scene(Hero::SunShangxiang, "dongxing_snow_bridge_ledger", &["yiling_ashes"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "yiling_dongxing_snow_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "dongxing_snow_bridge_ledger");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "yiling_dongxing_snow_debt"), 1);
    let mut declined = scene(Hero::SunShangxiang, "dongxing_snow_bridge_ledger", &["yiling_ashes"]);
    declined.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!declined.lore_flags.iter().any(|s| s == "dongxing_snow_bridge_record"));
    assert_eq!(count(&declined, "yiling_dongxing_snow_debt"), 0);
}

#[test]
fn shu_water_and_wei_edict_debts_require_accepted_responsibilities() {
    let mut shu = scene(Hero::ZhugeLiang, "qushan_crossbow_aftercare", &["wuzhang_withdrawal_record"]);
    shu.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut shu, "tielong_spring_rescue");
    shu.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&shu, "wuzhang_tielong_water_debt"), 1);
    enter(&mut shu, "taoxi_didao_withdrawal");
    shu.apply(Action::Choose { index: 1 }).unwrap();
    assert!(shu.lore_flags.iter().any(|s| s == "taoxi_didao_withdrawal_record"));
    let mut wei = scene(Hero::CaoCao, "caofang_deposition_receipt", &["gaoping_oath_court_record"]);
    wei.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut wei, "shouchun_false_edict_audit");
    wei.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&wei, "gaoping_shouchun_edict_debt"), 1);
    let mut refused = scene(Hero::CaoCao, "caofang_deposition_receipt", &["gaoping_oath_court_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "shouchun_false_edict_audit");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "gaoping_shouchun_edict_debt"), 0);
}

#[test]
fn legacy_routes_precede_optional_archives_and_late_scenes_are_selectable() {
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &["redcliff_wind".into()], &[])
        .map(|b| b.id), Some("wuzhang"));
    assert_eq!(next_beat(Hero::CaoCao, 2, &["guandu_grain".into()], &[])
        .map(|b| b.id), Some("liaodong"));
    assert_eq!(next_beat(Hero::SunShangxiang, 2, &["jiangdong_covenant".into()], &[])
        .map(|b| b.id), Some("yiling"));
    for (hero, id, prereq) in [
        (Hero::SunShangxiang, "dongxing_snow_bridge_ledger", "yiling_ashes"),
        (Hero::ZhugeLiang, "qushan_crossbow_aftercare", "qushan_water_rescue_record"),
        (Hero::CaoCao, "caofang_deposition_receipt", "gaoping_oath_court_record"),
    ] {
        let seen: Vec<String> = BEATS.iter().filter(|b| b.id != id).map(|b| b.id.into()).collect();
        assert_eq!(next_beat(hero, 2, &[prereq.into()], &seen).map(|b| b.id), Some(id));
    }
    for seed in 108110..108118 {
        let saved = Game::new(Hero::SunShangxiang, seed).to_json().unwrap();
        assert!(Game::from_json(&saved).is_ok());
    }
}
