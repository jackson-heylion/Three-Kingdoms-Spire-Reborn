//! Fixed dual-edition local witnesses and existing playable consequences; not full collation.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn flag_count(g: &Game, flag: &str) -> usize {
    g.lore_flags.iter().filter(|s| s.as_str() == flag).count()
}
fn egg_count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_witnesses_have_distinct_counts_years_and_no_reward_change() {
    let doc = include_str!("../docs/CHAPTERS_046_048_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in ["2568009", "2568010", "2568011", "2554793", "2554771",
                "2554721", "2554717", "2554776", "2554800"] {
        assert!(doc.contains(rev), "missing pinned revision {rev}");
    }
    for anchor in ["九万余枝", "十余万枝", "书略曰", "韩遂、马超",
                   "韩遂、马腾", "建安十二年", "建安十三年", "原刻影印"] {
        assert!(doc.contains(anchor), "missing local witness {anchor}");
    }
    assert_eq!(audit.lines().skip(1).filter(|l| !l.is_empty()).count(), 120);
    for ch in 46..=48 {
        let line = audit.lines().find(|l| l.starts_with(&format!("{ch:03}\t")))
            .expect("chapter missing from 120-row ledger");
        assert!(line.contains("CHAPTERS_046_048_JIAJING_VARIANTS.md"));
        assert!(line.contains("正文通读/未完成异本对校"));
    }
    for (id, ch, flag, gold, heal, edition) in [
        ("borrow_arrows", 46, "mist_arrows", 20, 0, "嘉靖091"),
        ("chain_ship_ledger", 47, "chain_ship_risk_record", 19, 8, "嘉靖093"),
        ("river_banquet_inquest", 48, "river_banquet_record", 18, 7, "嘉靖095"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.flag, b.reward_gold, b.reward_heal),
                   (ch, flag, gold, heal));
        assert!(b.premise.contains(edition), "{id} missing edition boundary");
    }
}

#[test]
fn zhuge_mist_choice_is_atomic_and_legacy_compatible() {
    let mut g = Game::new(Hero::ZhugeLiang, 46048);
    g.lore_flags.push("longzhong_plan".into());
    enter(&mut g, "borrow_arrows");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(flag_count(&g, "mist_arrows"), 1);
    let mut saved = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut saved, "borrow_arrows");
    saved.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(flag_count(&saved, "mist_arrows"), 1);
    let mut refusal = Game::new(Hero::ZhugeLiang, 46049);
    refusal.lore_flags.push("longzhong_plan".into());
    enter(&mut refusal, "borrow_arrows");
    refusal.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(flag_count(&refusal, "mist_arrows"), 0);
}

#[test]
fn cao_false_letter_to_banquet_pays_once_without_forcing_old_saves() {
    let mut g = Game::new(Hero::CaoCao, 46050);
    g.lore_flags.push("guandu_grain".into());
    enter(&mut g, "jiangkou_inquest");
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "chain_ship_ledger");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&g, "river_spy_chain_debt"), 1);
    enter(&mut g, "river_banquet_inquest");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&g, "redcliff_warning_ledger"), 1);
    let mut saved = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut saved, "river_banquet_inquest");
    saved.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&saved, "redcliff_warning_ledger"), 1);
    let mut old = Game::new(Hero::CaoCao, 46051);
    old.lore_flags.push("guandu_grain".into());
    enter(&mut old, "liaodong");
    old.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(flag_count(&old, "liaodong_letter"), 1);
}
