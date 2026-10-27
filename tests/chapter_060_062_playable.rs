//! 060–062: pinned provenance, mutually exclusive records, V2, atomic rejection and once-only cross-act payoffs.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};

fn at(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 6062);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn has(g: &Game, flag: &str) -> bool { g.lore_flags.iter().any(|s| s == flag) }
fn count(g: &Game, id: &str) -> usize { g.easter_unlocked.iter().filter(|s| s.as_str() == id).count() }

#[test]
fn pinned_full_texts_are_scoped_to_single_base_only() {
    for (chapter, oldid, doc) in [
        (60, "2568025", include_str!("../docs/CHAPTER_060_READING.md")),
        (61, "2568023", include_str!("../docs/CHAPTER_061_READING.md")),
        (62, "2568027", include_str!("../docs/CHAPTER_062_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")) && doc.contains("单底本"), "{chapter}");
    }
    let h = include_str!("../docs/CHAPTERS_060_062_HISTORY_SPOTCHECK.md");
    for rev in ["2583384", "2583383", "2274279"] { assert!(h.contains(rev)); }
    assert!(h.contains("裴注") && h.contains("局部") && h.contains("0/120"));
    for (id, chapter, pre) in [
        ("zhangsong_audience", 60, "weinan_division_record"),
        ("yizhou_map_audit", 60, "leiyang_case_record"),
        ("river_child_witness", 61, "nanxu_escape_witness_record"),
        ("fu_pass_review", 62, "yizhou_map_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(pre));
    }
}

#[test]
fn cao_envoy_ledger_preserves_old_liaodong_and_is_once_only() {
    let flags = ["guandu_grain", "mateng_case_record", "tongguan_retreat_record", "weinan_division_record"];
    let owned: Vec<String> = flags.iter().map(|s| (*s).into()).collect();
    assert_eq!(next_beat(Hero::CaoCao, 2, &owned, &[]).map(|b| b.id), Some("liaodong"));
    let all_other_scenes: Vec<String> = BEATS.iter().filter(|b| b.id != "zhangsong_audience")
        .map(|b| b.id.to_string()).collect();
    assert_eq!(next_beat(Hero::CaoCao, 2, &owned, &all_other_scenes).map(|b| b.id), Some("zhangsong_audience"));
    let mut g = at(Hero::CaoCao, "zhangsong_audience", &flags);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "zhangsong_envoy_record"));
    assert_eq!(count(&g, "xuchang_shu_envoy_debt"), 1);
    let saved = g.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "zhangsong_audience");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "xuchang_shu_envoy_debt"), 1);
    let mut refusal = at(Hero::CaoCao, "zhangsong_audience", &flags);
    refusal.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&refusal, "zhangsong_envoy_record"));
    assert_eq!(count(&refusal, "xuchang_shu_envoy_debt"), 0);
}

#[test]
fn sun_river_witness_cannot_steal_legacy_yiling_or_reverse_the_child() {
    let flags = ["jiangdong_covenant", "nanxu_escape_witness_record"];
    let owned: Vec<String> = flags.iter().map(|s| (*s).into()).collect();
    assert_eq!(next_beat(Hero::SunShangxiang, 2, &owned, &[]).map(|b| b.id), Some("yiling"));
    let all_other_scenes: Vec<String> = BEATS.iter().filter(|b| b.id != "river_child_witness")
        .map(|b| b.id.to_string()).collect();
    assert_eq!(next_beat(Hero::SunShangxiang, 2, &owned, &all_other_scenes).map(|b| b.id), Some("river_child_witness"));
    let mut g = at(Hero::SunShangxiang, "river_child_witness", &flags);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "river_child_witness_record"));
    assert_eq!(count(&g, "nanxu_river_witness"), 1);
    let mut refused = at(Hero::SunShangxiang, "river_child_witness", &flags);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&refused, "river_child_witness_record"));
    assert_eq!(count(&refused, "nanxu_river_witness"), 0);
}

#[test]
fn zhuge_map_to_fu_pass_is_postwar_report_and_once_only() {
    let flags = ["leiyang_case_record", "redcliff_wind"];
    let owned: Vec<String> = flags.iter().map(|s| (*s).into()).collect();
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &owned, &[]).map(|b| b.id), Some("wuzhang"));
    let mut g = at(Hero::ZhugeLiang, "yizhou_map_audit", &flags);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "yizhou_map_record"));
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    enter(&mut g, "fu_pass_review");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "yizhou_fu_pass_debt"), 1);
    enter(&mut g, "fu_pass_review");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "yizhou_fu_pass_debt"), 1);
    let mut refused = at(Hero::ZhugeLiang, "yizhou_map_audit", &flags);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&refused, "yizhou_map_record"));
    enter(&mut refused, "fu_pass_review");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "yizhou_fu_pass_debt"), 0);
}
