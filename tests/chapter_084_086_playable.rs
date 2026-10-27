//! Fixed 084–086 source pins, original-game accountability, atomicity and V2 saves.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 8486);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).to_owned()).collect();
    g
}
fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn count(g: &Game, id: &str) -> usize { g.easter_unlocked.iter().filter(|s| s.as_str() == id).count() }

#[test]
fn fixed_chapter_provenance_and_historiography_are_explicit() {
    for (n, rev, body) in [
        (84, "2568047", include_str!("../docs/CHAPTER_084_READING.md")),
        (85, "2568048", include_str!("../docs/CHAPTER_085_READING.md")),
        (86, "2568049", include_str!("../docs/CHAPTER_086_READING.md")),
    ] {
        assert!(body.contains(&format!("oldid={rev}")), "{n}");
        assert!(body.contains("单底本") && body.contains("回末"), "{n}");
    }
    let history = include_str!("../docs/CHAPTERS_084_086_HISTORY_SPOTCHECK.md");
    for rev in ["2131313", "2583378", "2115776", "2602668", "2390639"] {
        assert!(history.contains(rev));
    }
    assert!(history.contains("0/120") && history.contains("李严为副"));
    for (id, ch, required) in [
        ("yiling_fire_refuge_ledger", 84, "yiling_defense_record"),
        ("fishbelly_eightfold_report", 84, "yiling_camp_map_record"),
        ("yongan_regency_register", 85, "fishbelly_report_record"),
        ("wei_three_fronts_review", 85, "wei_wu_tribute_record"),
        ("dengzhi_alliance_dispatch", 86, "yongan_regency_record"),
        ("guangling_river_defense", 86, "yiling_fire_refuge_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, ch);
        assert_eq!(b.prerequisite, Some(required));
        assert_eq!(b.act, 2);
    }
    assert!(beat("guangling_river_defense").unwrap().premise.contains("已死"));
    assert!(beat("wei_three_fronts_review").unwrap().premise.contains("病卒"));
}

#[test]
fn zhuge_fire_warning_regency_envoy_chain_is_atomic_and_idempotent() {
    let mut g = scene(Hero::ZhugeLiang, "fishbelly_eightfold_report", &["redcliff_wind", "yiling_camp_map_record"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "yongan_regency_register"); g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "dengzhi_alliance_dispatch"); g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "redcliff_yongan_envoy_debt"), 1);
    let saved = g.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "dengzhi_alliance_dispatch");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "redcliff_yongan_envoy_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "fishbelly_eightfold_report", &["redcliff_wind", "yiling_camp_map_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "yongan_regency_register"); refused.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut refused, "dengzhi_alliance_dispatch"); refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "redcliff_yongan_envoy_debt"), 0);
}

#[test]
fn sun_archive_and_wei_archive_pay_only_after_complete_chains() {
    let mut sun = scene(Hero::SunShangxiang, "yiling_fire_refuge_ledger", &["jiangdong_covenant", "yiling_defense_record"]);
    sun.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut sun, "guangling_river_defense"); sun.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&sun, "yiling_guangling_civilian_debt"), 1);
    let mut cao = scene(Hero::CaoCao, "wei_three_fronts_review", &["guandu_grain", "wei_wu_tribute_record"]);
    cao.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&cao, "guandu_wei_three_fronts_debt"), 1);
    let mut refused = scene(Hero::SunShangxiang, "yiling_fire_refuge_ledger", &["jiangdong_covenant", "yiling_defense_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "guangling_river_defense"); refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "yiling_guangling_civilian_debt"), 0);
}

#[test]
fn legacy_act_two_routes_win_over_optional_archival_scenes() {
    for (hero, flags, old, new) in [
        (Hero::SunShangxiang, vec!["jiangdong_covenant", "yiling_defense_record"], "yiling", "yiling_fire_refuge_ledger"),
        (Hero::ZhugeLiang, vec!["redcliff_wind", "yiling_camp_map_record"], "wuzhang", "fishbelly_eightfold_report"),
        (Hero::CaoCao, vec!["guandu_grain", "wei_wu_tribute_record"], "liaodong", "wei_three_fronts_review"),
    ] {
        let flags: Vec<String> = flags.into_iter().map(str::to_owned).collect();
        assert_eq!(next_beat(hero, 2, &flags, &[]).map(|b| b.id), Some(old));
        let seen: Vec<String> = BEATS.iter().filter(|b| b.id != new).map(|b| b.id.to_owned()).collect();
        assert_eq!(next_beat(hero, 2, &flags, &seen).map(|b| b.id), Some(new));
    }
}
