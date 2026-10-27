//! Chapter 054–056 fixed-text provenance, decisions, old-save routes and once-only rewards.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn at(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 5456);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn has(g: &Game, flag: &str) -> bool {
    g.lore_flags.iter().any(|f| f == flag)
}
fn awards(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|f| f.as_str() == id).count()
}

#[test]
fn three_fixed_readings_and_history_layer_are_pinned() {
    for (n, oldid, doc) in [
        (54, "2568017", include_str!("../docs/CHAPTER_054_READING.md")),
        (55, "2568018", include_str!("../docs/CHAPTER_055_READING.md")),
        (56, "2568019", include_str!("../docs/CHAPTER_056_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")) && doc.contains("单底本"), "{n}");
    }
    let h = include_str!("../docs/CHAPTERS_054_056_HISTORY_SPOTCHECK.md");
    for rev in ["2583378", "2387393", "2585429"] {
        assert!(h.contains(rev));
    }
    assert!(h.contains("不纳") && h.contains("冬"));
    for (id, chapter, pre) in [
        ("jingzhou_pledge_copy", 54, "jingxiang_lease_record"),
        ("nanxu_escape_witness", 55, "jiangdong_covenant"),
        ("tongque_dispatch_ledger", 56, "river_banquet_record"),
        ("jingzhou_false_passage", 56, "jingzhou_pledge_copy_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(pre));
    }
}
#[test]
fn zhuge_pledge_and_false_passage_are_atomic_persistent_and_once_only() {
    let mut g = at(Hero::ZhugeLiang, "jingzhou_pledge_copy",
        &["jingxiang_lease_record", "redcliff_wind"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "jingzhou_pledge_copy_record"));
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(), saved);
    enter(&mut g, "jingzhou_false_passage");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "jingzhou_pledge_reckoning"), 1);
    enter(&mut g, "jingzhou_false_passage");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "jingzhou_pledge_reckoning"), 1);
    let mut refused = at(Hero::ZhugeLiang, "jingzhou_pledge_copy",
        &["jingxiang_lease_record", "redcliff_wind"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&refused, "jingzhou_pledge_copy_record"));
    assert_eq!(awards(&refused, "jingzhou_pledge_reckoning"), 0);
}
#[test]
fn old_zhuge_wuzhang_route_has_priority_over_new_retrospective() {
    let flags = vec!["redcliff_wind".into(), "jingxiang_lease_record".into()];
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &[]).map(|b| b.id),
        Some("wuzhang"));
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &["wuzhang".into(), "nanjun_claim_ledger".into()])
        .map(|b| b.id), Some("jingzhou_pledge_copy"));
}
#[test]
fn sun_wedding_yiling_remain_available_before_optional_escape_ledger() {
    let flags = vec!["jiangdong_covenant".into()];
    assert_eq!(next_beat(Hero::SunShangxiang, 2, &flags, &[])
        .map(|b| b.id), Some("yiling"));
    assert_eq!(next_beat(Hero::SunShangxiang, 2, &flags, &["yiling".into()])
        .map(|b| b.id), Some("nanxu_escape_witness"));
    let mut g = at(Hero::SunShangxiang, "nanxu_escape_witness",
        &["jiangdong_covenant"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 4 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "yiling");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "nanxu_yiling_witness"), 1);
    let mut g = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g, "nanxu_escape_witness");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "nanxu_yiling_witness"), 1);
}
#[test]
fn cao_tongque_inquest_preserves_liaodong_and_reward_is_mutually_exclusive() {
    let flags = vec!["guandu_grain".into(), "river_banquet_record".into()];
    assert_eq!(next_beat(Hero::CaoCao, 2, &flags, &[])
        .map(|b| b.id), Some("liaodong"));
    let mut g = at(Hero::CaoCao, "tongque_dispatch_ledger",
        &["guandu_grain", "river_banquet_record"]);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "tongque_dispatch_record"));
    assert_eq!(awards(&g, "tongque_river_warning"), 1);
    let mut refused = at(Hero::CaoCao, "tongque_dispatch_ledger",
        &["guandu_grain", "river_banquet_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&refused, "tongque_dispatch_record"));
    assert_eq!(awards(&refused, "tongque_river_warning"), 0);
}
