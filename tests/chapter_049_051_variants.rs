//! Fixed Mao/Jiajing local witnesses, not full collation or natural map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(xs: &[String], flag: &str) -> usize {
    xs.iter().filter(|s| s.as_str() == flag).count()
}

#[test]
fn fixed_variants_and_existing_rewards() {
    let doc = include_str!("../docs/CHAPTERS_049_051_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in ["2568012", "2568013", "2568014", "2554748", "2554822",
                "2554761", "2554747", "2554720", "2554725"] {
        assert!(doc.contains(rev), "missing revision {rev}");
    }
    for anchor in ["与曹操连和", "归师勿掩", "庾公之斯", "杀了陈矫",
                   "右肋", "左肋", "原刻影印", "0回"] {
        assert!(doc.contains(anchor), "missing variant {anchor}");
    }
    assert_eq!(audit.lines().skip(1).filter(|l| !l.is_empty()).count(), 120);
    for ch in 49..=51 {
        assert!(audit.lines().any(|l| l.starts_with(&format!("{ch:03}\t"))
            && l.contains("CHAPTERS_049_051_JIAJING_VARIANTS.md")));
    }
    for (id, ch, flag, gold, heal, source) in [
        ("redcliff", 49, "redcliff_wind", 18, 5, "嘉靖097"),
        ("huarong", 50, "huarong_mercy", 16, 12, "嘉靖099"),
        ("nanjun_claim_ledger", 51, "nanjun_claim_record", 17, 8, "嘉靖102"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.flag, b.reward_gold, b.reward_heal),
                   (ch, flag, gold, heal));
        assert!(b.premise.contains(source), "{id} missing source boundary");
    }
}

#[test]
fn zhuge_redcliff_nanjun_chain_is_atomic_and_persistent() {
    let mut g = Game::new(Hero::ZhugeLiang, 49051);
    g.lore_flags.push("longzhong_plan".into());
    enter(&mut g, "redcliff");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g.lore_flags, "redcliff_wind"), 1);
    g.lore_flags.push("jiangdong_council_record".into());
    enter(&mut g, "nanjun_claim_ledger");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g.lore_flags, "nanjun_claim_record"), 1);
    let mut restored = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut restored, "nanjun_claim_ledger");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "nanjun_claim_record"), 1);
}

#[test]
fn huarong_refusal_and_mercy_are_hero_isolated() {
    let mut g = Game::new(Hero::GuanYu, 50051);
    g.lore_flags.push("baima_debt".into());
    enter(&mut g, "huarong");
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&g.lore_flags, "huarong_mercy"), 0);
    assert!(Game::from_json(&g.to_json().unwrap()).is_ok());
    let mut g2 = Game::new(Hero::GuanYu, 50052);
    g2.lore_flags.push("baima_debt".into());
    enter(&mut g2, "huarong");
    g2.apply(Action::Choose { index: 1 }).unwrap();
    let mut restored = Game::from_json(&g2.to_json().unwrap()).unwrap();
    enter(&mut restored, "huarong");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "huarong_mercy"), 1);
    assert_eq!(count(&restored.lore_flags, "nanjun_claim_record"), 0);
}
