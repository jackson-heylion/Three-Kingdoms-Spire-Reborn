//! Independent Mao 046-048 reread, not full dual-edition or historical collation.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(xs: &[String], flag: &str) -> usize {
    xs.iter().filter(|x| x.as_str() == flag).count()
}
#[test]
fn pinned_independent_reading_and_correct_arrow_deadline() {
    let doc = include_str!("../docs/CHAPTERS_046_048_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    let rows: Vec<_> = audit.lines().skip(1).filter(|l| !l.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for (ch, rev) in [("046", "2568009"), ("047", "2568010"), ("048", "2568011")] {
        let line = rows.iter().find(|l| l.starts_with(&format!("{ch}\t"))).unwrap();
        assert!(line.contains("2026-11-02独立复核"));
        assert!(line.contains(rev));
        assert!(line.contains("未完成异本对校"));
    }
    for marker in ["周瑜先问十日", "孔明主动缩成三日", "黄盖实受五十杖",
                   "曹操因未约日期先疑", "曹操自行下令", "程昱警火",
                   "回末身份未揭", "赤壁火攻未起"] {
        assert!(doc.contains(marker), "missing {marker}");
    }
    let b = beat("borrow_arrows").unwrap();
    assert!(b.premise.contains("周瑜先问十日造十万箭"));
    assert!(b.premise.contains("孔明自立三日军令状"));
    assert_eq!((b.chapter, b.flag, b.reward_gold), (46, "mist_arrows", 20));
}
#[test]
fn zhuge_refusal_keeps_legacy_redcliff_and_v2_atomic() {
    let mut g = Game::new(Hero::ZhugeLiang, 460481);
    g.lore_flags.push("longzhong_plan".into());
    enter(&mut g, "borrow_arrows");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&g.lore_flags, "mist_arrows"), 0);
    let mut saved = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut saved, "redcliff");
    saved.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&saved.lore_flags, "redcliff_wind"), 1);
    assert_eq!(count(&saved.lore_flags, "mist_arrows"), 0);
}
#[test]
fn cao_chain_refusal_cannot_mint_hidden_rewards_or_lock_liaodong() {
    let mut g = Game::new(Hero::CaoCao, 470481);
    g.lore_flags.push("guandu_grain".into());
    enter(&mut g, "jiangkou_inquest");
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "chain_ship_ledger");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&g.lore_flags, "chain_ship_risk_record"), 0);
    assert_eq!(count(&g.easter_unlocked, "river_spy_chain_debt"), 0);
    let mut saved = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut saved, "liaodong");
    saved.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&saved.lore_flags, "liaodong_letter"), 1);
    let mut yes = Game::new(Hero::CaoCao, 470482);
    yes.lore_flags.push("guandu_grain".into());
    enter(&mut yes, "jiangkou_inquest");
    yes.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut yes, "chain_ship_ledger");
    yes.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut yes, "river_banquet_inquest");
    yes.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&yes.easter_unlocked, "river_spy_chain_debt"), 1);
    assert_eq!(count(&yes.easter_unlocked, "redcliff_warning_ledger"), 1);
    let mut restored = Game::from_json(&yes.to_json().unwrap()).unwrap();
    enter(&mut restored, "river_banquet_inquest");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "redcliff_warning_ledger"), 1);
}
