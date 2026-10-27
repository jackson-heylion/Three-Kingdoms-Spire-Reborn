//! Chapters 057–059: source evidence, atomic choices, V2 saves and cross-act once-only rewards.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn at(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 5759);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn has(g: &Game, f: &str) -> bool { g.lore_flags.iter().any(|s| s == f) }
fn awards(g: &Game, id: &str) -> usize { g.easter_unlocked.iter().filter(|s| s.as_str() == id).count() }

#[test]
fn pinned_full_readings_are_not_misreported_as_full_collation() {
    for (n, rev, doc) in [
        (57, "2568020", include_str!("../docs/CHAPTER_057_READING.md")),
        (58, "7909160", include_str!("../docs/CHAPTER_058_READING.md")),
        (59, "2568022", include_str!("../docs/CHAPTER_059_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={rev}")) && doc.contains("单底本"), "{n}");
    }
    let history = include_str!("../docs/CHAPTERS_057_059_HISTORY_SPOTCHECK.md");
    for rev in ["2387393", "2583384", "2583383", "2585429"] {
        assert!(history.contains(rev));
    }
    assert!(history.contains("裴注") && history.contains("局部"));
    for (id, chapter, pre) in [
        ("leiyang_review", 57, "phoenix_letter"),
        ("mateng_case", 57, "tongque_dispatch_record"),
        ("tongguan_retreat", 58, "mateng_case_record"),
        ("weinan_division", 59, "tongguan_retreat_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(pre));
    }
    assert_eq!(beat("fengchu").unwrap().chapter, 57);
}
#[test]
fn zhuge_leiyang_choice_is_atomic_mutually_exclusive_and_once_only() {
    let mut g = at(Hero::ZhugeLiang, "leiyang_review", &["longzhong_plan", "phoenix_letter"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "leiyang_case_record"));
    assert_eq!(awards(&g, "phoenix_office_reckoning"), 1);
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(), saved);
    enter(&mut g, "leiyang_review");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "phoenix_office_reckoning"), 1);
    let mut refused = at(Hero::ZhugeLiang, "leiyang_review", &["longzhong_plan", "phoenix_letter"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&refused, "leiyang_case_record"));
    assert_eq!(awards(&refused, "phoenix_office_reckoning"), 0);
}
#[test]
fn cao_mateng_tongguan_weinan_chain_survives_save_and_rewards_once() {
    let mut g = at(Hero::CaoCao, "mateng_case", &["guandu_grain", "tongque_dispatch_record"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "tongguan_retreat");
    g.apply(Action::Choose { index: 1 }).unwrap();
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(), saved);
    enter(&mut g, "weinan_division");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "weinan_division_record"));
    assert_eq!(awards(&g, "xuchang_weinan_reckoning"), 1);
    enter(&mut g, "weinan_division");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "xuchang_weinan_reckoning"), 1);
}
#[test]
fn refusing_mateng_record_never_unlocks_weinan_debt() {
    let mut g = at(Hero::CaoCao, "mateng_case", &["guandu_grain", "tongque_dispatch_record"]);
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&g, "mateng_case_record"));
    enter(&mut g, "tongguan_retreat");
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "weinan_division");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "xuchang_weinan_reckoning"), 0);
}
#[test]
fn legacy_liaodong_precedes_optional_mateng_but_both_remain_reachable() {
    let flags = vec!["guandu_grain".into(), "tongque_dispatch_record".into()];
    assert_eq!(next_beat(Hero::CaoCao, 2, &flags, &[]).map(|b| b.id), Some("liaodong"));
    assert_eq!(next_beat(Hero::CaoCao, 2, &flags, &["liaodong".into()])
        .map(|b| b.id), Some("mateng_case"));
}
