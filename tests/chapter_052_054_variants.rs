//! Fixed Mao/Jiajing local variants; not full collation or natural-map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(xs: &[String], name: &str) -> usize {
    xs.iter().filter(|x| x.as_str() == name).count()
}

#[test]
fn fixed_witnesses_and_existing_reward_contracts() {
    let doc = include_str!("../docs/CHAPTERS_052_054_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in ["2568016", "2568015", "2568017", "2554821", "2554795",
                "2554824", "2554774", "2554789", "2554797"] {
        assert!(doc.contains(rev), "missing revision {rev}");
    }
    for phrase in ["刘延", "劉賢", "一百合", "一百餘合", "天子之阶",
                   "不世之功", "九族难保", "原刻影印", "0回"] {
        assert!(doc.contains(phrase), "missing witness {phrase}");
    }
    assert_eq!(audit.lines().skip(1).filter(|r| !r.is_empty()).count(), 120);
    for ch in 52..=54 {
        assert!(audit.lines().any(|r| r.starts_with(&format!("{ch:03}\t"))
            && r.contains("CHAPTERS_052_054_JIAJING_VARIANTS.md")));
    }
    for (id, ch, flag, gold, heal, anchor) in [
        ("jingxiang_lease_ledger", 52, "jingxiang_lease_record", 18, 8, "嘉靖103"),
        ("changsha_mercy", 53, "changsha_mercy_record", 16, 9, "嘉靖105"),
        ("hefei_memorial", 53, "hefei_casualty_record", 19, 7, "嘉靖106"),
        ("jingzhou_pledge_copy", 54, "jingzhou_pledge_copy_record", 17, 8, "嘉靖107"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.flag, b.reward_gold, b.reward_heal),
                   (ch, flag, gold, heal));
        assert!(b.premise.contains(anchor), "{id} missing source boundary");
    }
}

#[test]
fn zhuge_lease_to_pledge_is_atomic_and_persistent() {
    let mut g = Game::new(Hero::ZhugeLiang, 52054);
    g.lore_flags.push("jiangdong_council_record".into());
    enter(&mut g, "nanjun_claim_ledger");
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "jingxiang_lease_ledger");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "jingzhou_pledge_copy");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g.lore_flags, "jingxiang_lease_record"), 1);
    assert_eq!(count(&g.lore_flags, "jingzhou_pledge_copy_record"), 1);
    let mut restored = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut restored, "jingzhou_pledge_copy");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "jingzhou_pledge_copy_record"), 1);
}

#[test]
fn guan_mercy_and_sun_hefei_refusal_are_hero_isolated() {
    let mut guan = Game::new(Hero::GuanYu, 53054);
    guan.lore_flags.push("huarong_mercy".into());
    enter(&mut guan, "changsha_mercy");
    guan.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&guan.lore_flags, "changsha_mercy_record"), 1);
    assert!(Game::from_json(&guan.to_json().unwrap()).is_ok());

    let mut sun = Game::new(Hero::SunShangxiang, 53055);
    sun.lore_flags.push("jiangdong_muster_record".into());
    enter(&mut sun, "hefei_memorial");
    sun.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&sun.lore_flags, "hefei_casualty_record"), 0);
    assert_eq!(count(&sun.lore_flags, "changsha_mercy_record"), 0);
    assert!(Game::from_json(&sun.to_json().unwrap()).is_ok());
}
