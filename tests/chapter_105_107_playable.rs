//! P1④ 105–107: pinned full readings, optional archival choices, old-save safety.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 105107);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_chapter_evidence_and_history_layers_are_explicit() {
    for (chapter, oldid, doc) in [
        (105, "2602415", include_str!("../docs/CHAPTER_105_READING.md")),
        (106, "7909163", include_str!("../docs/CHAPTER_106_READING.md")),
        (107, "2568071", include_str!("../docs/CHAPTER_107_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_105_107_HISTORY_SPOTCHECK.md");
    for rev in ["2583390", "2086403", "2274283", "12068919"] {
        assert!(history.contains(rev));
    }
    assert!(history.contains("裴注") && history.contains("0/120"));
    let rows: Vec<_> = include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(108).skip(105) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    for (id, chapter) in [
        ("weiyan_double_memorial_audit", 105), ("zongyu_wu_border_receipt", 105),
        ("wei_palace_labor_register", 105), ("liaodong_rain_siege_ledger", 106),
        ("wei_regency_sickbed_register", 106), ("gaoping_oath_court_review", 107),
        ("qushan_water_rescue_dispatch", 107),
    ] {
        assert_eq!(beat(id).unwrap().chapter, chapter);
    }
}

#[test]
fn wei_yan_double_memorial_and_qushan_debt_are_atomic_persistent_and_once_only() {
    let mut g = scene(Hero::ZhugeLiang, "weiyan_double_memorial_audit", &["wuzhang_withdrawal_record"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "wuzhang_qushan_supply_debt"), 0);
    let saved = g.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "qushan_water_rescue_dispatch");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "wuzhang_qushan_supply_debt"), 1);
    enter(&mut restored, "qushan_water_rescue_dispatch");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "wuzhang_qushan_supply_debt"), 1);
    let mut declined = scene(Hero::ZhugeLiang, "weiyan_double_memorial_audit", &["wuzhang_withdrawal_record"]);
    declined.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut declined, "qushan_water_rescue_dispatch");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&declined, "wuzhang_qushan_supply_debt"), 0);
}

#[test]
fn palace_liaodong_regency_and_gaoping_are_four_distinct_accepted_responsibilities() {
    let mut g = scene(Hero::CaoCao, "wei_palace_labor_register", &["guandu_grain"]);
    for id in ["wei_palace_labor_register", "liaodong_rain_siege_ledger",
        "wei_regency_sickbed_register", "gaoping_oath_court_review"] {
        enter(&mut g, id);
        g.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&g, "wei_palace_gaoping_oath_debt"), 1);
    let mut declined = scene(Hero::CaoCao, "wei_palace_labor_register", &["guandu_grain"]);
    declined.apply(Action::Choose { index: 2 }).unwrap();
    for id in ["liaodong_rain_siege_ledger", "wei_regency_sickbed_register", "gaoping_oath_court_review"] {
        enter(&mut declined, id);
        declined.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&declined, "wei_palace_gaoping_oath_debt"), 0);
}

#[test]
fn old_routes_and_posthumous_archives_remain_optional() {
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &["redcliff_wind".into()], &[])
        .map(|b| b.id), Some("wuzhang"));
    assert_eq!(next_beat(Hero::CaoCao, 2, &["guandu_grain".into()], &[])
        .map(|b| b.id), Some("liaodong"));
    assert_eq!(next_beat(Hero::SunShangxiang, 2, &["jiangdong_covenant".into()], &[])
        .map(|b| b.id), Some("yiling"));
    let mut wu = scene(Hero::SunShangxiang, "zongyu_wu_border_receipt", &[]);
    wu.apply(Action::Choose { index: 1 }).unwrap();
    assert!(wu.lore_flags.iter().any(|f| f == "zongyu_border_reply_record"));
    let all_seen: Vec<String> = BEATS.iter().filter(|b| b.id != "weiyan_double_memorial_audit")
        .map(|b| b.id.to_owned()).collect();
    assert_eq!(next_beat(Hero::ZhugeLiang, 2,
        &["wuzhang_withdrawal_record".into()], &all_seen).map(|b| b.id),
        Some("weiyan_double_memorial_audit"));
    for seed in 105100..105108 {
        let saved = Game::new(Hero::ZhugeLiang, seed).to_json().unwrap();
        assert!(Game::from_json(&saved).is_ok());
    }
}
