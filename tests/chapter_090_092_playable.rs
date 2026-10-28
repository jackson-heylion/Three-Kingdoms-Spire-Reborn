//! Fixed 090–092 source pins, novel/history separation and playable accountability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(Hero::ZhugeLiang, 9092);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|f| (*f).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|f| f.as_str() == id).count()
}

#[test]
fn pinned_full_reading_and_partial_history_are_distinct() {
    for (chapter, oldid, source) in [
        (90, "2568053", include_str!("../docs/CHAPTER_090_READING.md")),
        (91, "7909162", include_str!("../docs/CHAPTER_091_READING.md")),
        (92, "2568054", include_str!("../docs/CHAPTER_092_READING.md")),
    ] {
        assert!(source.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(source.contains("单底本全文通读") && source.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_090_092_HISTORY_SPOTCHECK.md");
    assert!(history.contains("2115776") && history.contains("12068919"));
    assert!(history.contains("裴松之") && history.contains("0/120"));
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    let rows: Vec<&str> = ledger.lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(93).skip(90) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
    }
    for (id, chapter, prerequisite) in [
        ("nanzhong_vine_fire_account", 90, "xier_spring_record"),
        ("nanzhong_local_governance", 90, "nanzhong_vine_fire_record"),
        ("lushui_memorial_rites", 91, "nanzhong_local_rule_record"),
        ("chushi_memorial_supply", 91, "lushui_memorial_record"),
        ("ziwu_route_risk_hearing", 92, "chushi_supply_record"),
        ("tianshui_false_dispatch_review", 92, "ziwu_risk_record"),
    ] {
        let beat = beat(id).unwrap();
        assert_eq!(beat.chapter, chapter);
        assert_eq!(beat.prerequisite, Some(prerequisite));
    }
    assert!(beat("tianshui_false_dispatch_review").unwrap().premise.contains("尚未归蜀"));
}

#[test]
fn six_optional_scenes_are_atomic_mutually_exclusive_and_restorable() {
    let mut game = scene("nanzhong_vine_fire_account", &["redcliff_wind", "xier_spring_record"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    for (id, flag) in [
        ("nanzhong_vine_fire_account", "nanzhong_vine_fire_record"),
        ("nanzhong_local_governance", "nanzhong_local_rule_record"),
        ("lushui_memorial_rites", "lushui_memorial_record"),
        ("chushi_memorial_supply", "chushi_supply_record"),
        ("ziwu_route_risk_hearing", "ziwu_risk_record"),
        ("tianshui_false_dispatch_review", "tianshui_false_dispatch_record"),
    ] {
        enter(&mut game, id);
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert!(game.lore_flags.iter().any(|f| f == flag));
    }
    assert_eq!(count(&game, "nanzhong_mourning_debt"), 1);
    assert_eq!(count(&game, "redcliff_northern_route_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "tianshui_false_dispatch_review");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "nanzhong_mourning_debt"), 1);
    assert_eq!(count(&restored, "redcliff_northern_route_debt"), 1);
}

#[test]
fn refusal_does_not_create_prerequisite_or_reward() {
    let mut game = scene("nanzhong_vine_fire_account", &["redcliff_wind", "xier_spring_record"]);
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|f| f == "nanzhong_vine_fire_record"));
    enter(&mut game, "lushui_memorial_rites");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "nanzhong_mourning_debt"), 0);
}

#[test]
fn new_northern_accountability_does_not_steal_old_wuzhang_route() {
    let flags: Vec<String> = ["redcliff_wind", "xier_spring_record"]
        .into_iter().map(str::to_owned).collect();
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &[])
        .map(|b| b.id), Some("wuzhang"));
    let seen: Vec<String> = BEATS.iter().filter(|b| b.id != "nanzhong_vine_fire_account")
        .map(|b| b.id.to_owned()).collect();
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &seen)
        .map(|b| b.id), Some("nanzhong_vine_fire_account"));
}
