//! Pinned Mao-edition independent reread, not a claim of complete JiaJing/history collation.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn at(hero: Hero, seed: u64, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, seed);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|f| (*f).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn has(g: &Game, flag: &str) -> bool { g.lore_flags.iter().any(|s| s == flag) }
fn egg_count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_full_read_witnesses_have_real_boundaries_and_120_unique_rows() {
    let doc = include_str!("../docs/CHAPTERS_055_057_JIAJING_VARIANTS.md");
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for marker in ["oldid=2568018", "oldid=2568019", "oldid=2568020",
                   "正文L119—152", "正文L119—157", "正文L119—217",
                   "荀攸献策", "周瑜尚未死", "庞统百余日积案",
                   "非全回异本史料校勘"] {
        assert!(doc.contains(marker) || ledger.contains(marker), "missing {marker}");
    }
    let rows: Vec<_> = ledger.lines().skip(1).filter(|s| !s.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    let unique: std::collections::HashSet<_> =
        rows.iter().map(|s| s.split('\t').next().unwrap()).collect();
    assert_eq!(unique.len(), 120);
    for chapter in 55..=57 {
        let prefix = format!("{chapter:03}\t");
        let row = rows.iter().find(|s| s.starts_with(&prefix)).unwrap();
        assert!(row.contains("2026-11-02独立固定单底本全文复核"));
        assert!(row.contains("非全回异本史料校勘"));
    }
    for (id, chapter, hero, prerequisite) in [
        ("nanxu_escape_witness", 55, Hero::SunShangxiang, "jiangdong_covenant"),
        ("tongque_dispatch_ledger", 56, Hero::CaoCao, "river_banquet_record"),
        ("jingzhou_false_passage", 56, Hero::ZhugeLiang, "jingzhou_pledge_copy_record"),
        ("leiyang_review", 57, Hero::ZhugeLiang, "phoenix_letter"),
        ("mateng_case", 57, Hero::CaoCao, "tongque_dispatch_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.hero, b.prerequisite), (chapter, Some(hero), Some(prerequisite)));
    }
    assert!(beat("nanxu_escape_witness").unwrap().premise.contains("持剑追令未撤"));
    assert!(beat("tongque_dispatch_ledger").unwrap().premise.contains("程昱独立献策"));
    assert!(beat("mateng_case").unwrap().premise.contains("荀攸建议"));
}

#[test]
fn sun_nanqu_escape_is_atomic_persistent_and_yiling_reward_once_only() {
    let mut g = at(Hero::SunShangxiang, 55057, "nanxu_escape_witness",
        &["jiangdong_covenant", "yiling_ashes"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "nanxu_escape_witness_record"));
    assert_eq!(egg_count(&g, "nanxu_yiling_witness"), 1);
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(), saved);
    enter(&mut g, "nanxu_escape_witness");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&g, "nanxu_yiling_witness"), 1);

    let mut refusal = at(Hero::SunShangxiang, 55058, "nanxu_escape_witness",
        &["jiangdong_covenant"]);
    refusal.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&refusal, "nanxu_escape_witness_record"));
    enter(&mut refusal, "yiling");
    refusal.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&refusal, "yiling_ashes"));
    assert_eq!(egg_count(&refusal, "nanxu_yiling_witness"), 0);
}

#[test]
fn cao_military_archive_chain_is_mutually_exclusive_and_reward_idempotent() {
    let mut g = at(Hero::CaoCao, 56057, "tongque_dispatch_ledger",
        &["river_banquet_record", "guandu_grain"]);
    for (id, flag) in [
        ("tongque_dispatch_ledger", "tongque_dispatch_record"),
        ("mateng_case", "mateng_case_record"),
        ("tongguan_retreat", "tongguan_retreat_record"),
        ("weinan_division", "weinan_division_record"),
    ] {
        enter(&mut g, id);
        let before = g.to_json().unwrap();
        assert!(g.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(g.to_json().unwrap(), before);
        g.apply(Action::Choose { index: 1 }).unwrap();
        assert!(has(&g, flag), "missing {flag}");
    }
    assert_eq!(egg_count(&g, "xuchang_weinan_reckoning"), 1);
    let mut g = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g, "weinan_division");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&g, "xuchang_weinan_reckoning"), 1);

    let mut refusal = at(Hero::CaoCao, 56058, "mateng_case",
        &["tongque_dispatch_record", "guandu_grain"]);
    refusal.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&refusal, "mateng_case_record"));
    let mut refusal = Game::from_json(&refusal.to_json().unwrap()).unwrap();
    enter(&mut refusal, "liaodong");
    refusal.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&refusal, "liaodong_letter"));
    assert_eq!(egg_count(&refusal, "xuchang_weinan_reckoning"), 0);
}

#[test]
fn zhuge_refusal_does_not_lock_independent_phoenix_office_case() {
    let mut g = at(Hero::ZhugeLiang, 57057, "jingzhou_false_passage",
        &["jingzhou_pledge_copy_record", "phoenix_letter", "longzhong_plan"]);
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&g, "jingzhou_false_passage_record"));
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    enter(&mut g, "leiyang_review");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "leiyang_case_record"));
    assert_eq!(egg_count(&g, "phoenix_office_reckoning"), 1);
    enter(&mut g, "leiyang_review");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&g, "phoenix_office_reckoning"), 1);
}
