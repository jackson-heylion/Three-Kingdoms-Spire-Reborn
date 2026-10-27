//! 066–068: real optional scene choices, save compatibility, source and reward provenance.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
fn at(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 6668);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn has(g: &Game, f: &str) -> bool { g.lore_flags.iter().any(|s| s == f) }
fn count(g: &Game, id: &str) -> usize { g.easter_unlocked.iter().filter(|s| s.as_str() == id).count() }
#[test]
fn fixed_sources_and_unique_real_scenes() {
    for (n, rev, doc) in [
        (66, "2568029", include_str!("../docs/CHAPTER_066_READING.md")),
        (67, "2568030", include_str!("../docs/CHAPTER_067_READING.md")),
        (68, "2568031", include_str!("../docs/CHAPTER_068_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={rev}")) && doc.contains("单底本"), "{n}");
    }
    let h = include_str!("../docs/CHAPTERS_066_068_HISTORY_SPOTCHECK.md");
    for rev in ["2387393", "2086403", "2390639", "2603286"] { assert!(h.contains(rev)); }
    assert!(h.contains("裴注") && h.contains("0/120"));
    for (id, chapter, pre) in [
        ("lukou_terms_record", 66, "ma_chao_letter_record"),
        ("hanzhong_civil_ledger", 67, "zhangsong_envoy_record"),
        ("xiangshui_partition", 67, "chengdu_land_record"),
        ("xiaoyao_ferry_witness", 67, "hefei_casualty_record"),
        ("ruxu_rescue_roll", 68, "xiaoyao_ferry_record"),
        ("zuoci_court_record", 68, "hanzhong_civil_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(pre));
    }
}
#[test]
fn guan_lukou_is_atomic_mutually_exclusive_saveable_and_once_only() {
    let flags = ["jingzhou_handover_record", "ma_chao_letter_record"];
    let mut g = at(Hero::GuanYu, "lukou_terms_record", &flags);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "lukou_terms_record"));
    assert_eq!(count(&g, "jingzhou_lukou_debt"), 1);
    let save = g.to_json().unwrap();
    let mut restored = Game::from_json(&save).unwrap();
    assert_eq!(restored.to_json().unwrap(), save);
    enter(&mut restored, "lukou_terms_record");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "jingzhou_lukou_debt"), 1);
    let mut no = at(Hero::GuanYu, "lukou_terms_record", &flags);
    no.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&no, "lukou_terms_record"));
    assert_eq!(count(&no, "jingzhou_lukou_debt"), 0);
}
#[test]
fn sun_ferry_to_ruxu_payoff_is_once_only() {
    let mut g = at(Hero::SunShangxiang, "xiaoyao_ferry_witness", &["hefei_casualty_record"]);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "ruxu_rescue_roll");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "hefei_ruxu_debt"), 1);
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    enter(&mut g, "ruxu_rescue_roll");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "hefei_ruxu_debt"), 1);
    let mut no = at(Hero::SunShangxiang, "xiaoyao_ferry_witness", &["hefei_casualty_record"]);
    no.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut no, "ruxu_rescue_roll");
    no.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&no, "hefei_ruxu_debt"), 0);
}
#[test]
fn cao_and_zhuge_cross_chapter_ledger_rewards_require_all_flags() {
    let mut cao = at(Hero::CaoCao, "hanzhong_civil_ledger", &["zhangsong_envoy_record"]);
    cao.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut cao, "zuoci_court_record");
    cao.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&cao, "hanzhong_zuoci_debt"), 1);
    let mut zhuge = at(Hero::ZhugeLiang, "xiangshui_partition", &["yizhou_map_record", "chengdu_land_record"]);
    zhuge.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&zhuge, "xiangshui_yizhou_reckoning"), 1);
}
#[test]
fn old_story_priority_is_unchanged_and_new_scenes_remain_reachable() {
    for (hero, flags, old, new) in [
        (Hero::GuanYu, vec!["anxi_seal", "jingzhou_warning", "ma_chao_letter_record"], "jingzhou", "lukou_terms_record"),
        (Hero::ZhugeLiang, vec!["redcliff_wind", "chengdu_land_record"], "wuzhang", "xiangshui_partition"),
        (Hero::CaoCao, vec!["guandu_grain", "zhangsong_envoy_record"], "liaodong", "hanzhong_civil_ledger"),
        (Hero::SunShangxiang, vec!["jiangdong_covenant", "hefei_casualty_record"], "yiling", "xiaoyao_ferry_witness"),
    ] {
        let flags: Vec<String> = flags.into_iter().map(str::to_owned).collect();
        assert_eq!(next_beat(hero, 2, &flags, &[]).map(|b| b.id), Some(old));
        let seen: Vec<String> = BEATS.iter().filter(|b| b.id != new).map(|b| b.id.to_owned()).collect();
        assert_eq!(next_beat(hero, 2, &flags, &seen).map(|b| b.id), Some(new));
    }
}
