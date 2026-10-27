//! Fixed 087–089 source pins, five novel captures, historical layers and atomic gameplay.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 8789);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).to_owned()).collect();
    g
}
fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_editions_full_chapter_evidence_and_history_layers_are_explicit() {
    for (chapter, rev, body) in [
        (87, "2568051", include_str!("../docs/CHAPTER_087_READING.md")),
        (88, "2568052", include_str!("../docs/CHAPTER_088_READING.md")),
        (89, "2646272", include_str!("../docs/CHAPTER_089_READING.md")),
    ] {
        assert!(body.contains(&format!("oldid={rev}")), "{chapter}");
        assert!(body.contains("单底本") && body.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_087_089_HISTORY_SPOTCHECK.md");
    assert!(history.contains("2115776") && history.contains("2085132"));
    assert!(history.contains("裴松之") && history.contains("0/120"));
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    let rows: Vec<&str> = ledger.lines().collect();
    assert_eq!(rows.len(), 121);
    for chapter in 84..=89 {
        let row = rows[chapter];
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
    }
    for (id, chapter, required) in [
        ("nanzhong_yongchang_memorial", 87, "dengzhi_alliance_record"),
        ("nanzhong_first_release_ledger", 87, "nanzhong_yongchang_record"),
        ("wu_nanzhong_border_archive", 87, "guangling_defense_record"),
        ("lushui_ferry_care", 88, "nanzhong_first_release_record"),
        ("xier_spring_civilian_water", 89, "lushui_ferry_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(required));
        assert_eq!(b.act, 2);
    }
    assert!(beat("wu_nanzhong_border_archive").unwrap().premise.contains("已于小说084死"));
    assert!(beat("xier_spring_civilian_water").unwrap().premise.contains("第五擒"));
}

#[test]
fn zhuge_yongchang_lushui_xier_chain_is_atomic_and_once_only() {
    let mut g = scene(Hero::ZhugeLiang, "nanzhong_yongchang_memorial",
        &["redcliff_wind", "dengzhi_alliance_record"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    for (id, flag) in [
        ("nanzhong_yongchang_memorial", "nanzhong_yongchang_record"),
        ("nanzhong_first_release_ledger", "nanzhong_first_release_record"),
        ("lushui_ferry_care", "lushui_ferry_record"),
        ("xier_spring_civilian_water", "xier_spring_record"),
    ] {
        enter(&mut g, id);
        g.apply(Action::Choose { index: 1 }).unwrap();
        assert!(g.lore_flags.iter().any(|s| s == flag));
    }
    assert_eq!(count(&g, "redcliff_nanzhong_water_debt"), 1);
    let saved = g.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "xier_spring_civilian_water");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "redcliff_nanzhong_water_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "lushui_ferry_care",
        &["redcliff_wind", "dengzhi_alliance_record",
          "nanzhong_yongchang_record", "nanzhong_first_release_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "xier_spring_civilian_water");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "redcliff_nanzhong_water_debt"), 0);
}

#[test]
fn sun_posthumous_archive_is_mutually_exclusive_and_idempotent() {
    let mut sun = scene(Hero::SunShangxiang, "wu_nanzhong_border_archive",
        &["jiangdong_covenant", "guangling_defense_record"]);
    sun.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&sun, "jiangdong_nanzhong_envoy_debt"), 1);
    let mut refused = scene(Hero::SunShangxiang, "wu_nanzhong_border_archive",
        &["jiangdong_covenant", "guangling_defense_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&refused, "jiangdong_nanzhong_envoy_debt"), 0);
    assert!(!refused.lore_flags.iter().any(|s| s == "wu_nanzhong_border_record"));
}

#[test]
fn optional_southward_scenes_do_not_steal_legacy_act_two_routes() {
    for (hero, flags, old, new) in [
        (Hero::ZhugeLiang, vec!["redcliff_wind", "dengzhi_alliance_record"], "wuzhang", "nanzhong_yongchang_memorial"),
        (Hero::SunShangxiang, vec!["jiangdong_covenant", "guangling_defense_record"], "yiling", "wu_nanzhong_border_archive"),
    ] {
        let flags: Vec<String> = flags.into_iter().map(str::to_owned).collect();
        assert_eq!(next_beat(hero, 2, &flags, &[]).map(|b| b.id), Some(old));
        let seen: Vec<String> = BEATS.iter().filter(|b| b.id != new)
            .map(|b| b.id.to_owned()).collect();
        assert_eq!(next_beat(hero, 2, &flags, &seen).map(|b| b.id), Some(new));
    }
}
