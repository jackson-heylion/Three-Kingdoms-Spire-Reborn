//! Independent fixed Mao 052-054 reading; not full Jiajing or historical collation.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(g: &Game, flag: &str) -> usize {
    g.lore_flags.iter().filter(|x| x.as_str() == flag).count()
}

#[test]
fn fixed_witnesses_have_full_read_boundaries_without_claiming_full_collation() {
    let doc = include_str!("../docs/CHAPTERS_052_054_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    assert_eq!(audit.lines().skip(1).filter(|s| !s.is_empty()).count(), 120);
    for (chapter, oldid, boundary) in [
        ("052", "2568016", "正文L119—156"),
        ("053", "2568015", "正文L119—177"),
        ("054", "2568017", "正文L119—190"),
    ] {
        let matches: Vec<_> = audit.lines().filter(|s| s.starts_with(&format!("{chapter}\t"))).collect();
        assert_eq!(matches.len(), 1, "unique chapter {chapter}");
        assert!(matches[0].contains(&format!("2026-11-02独立固定毛评oldid={oldid}")));
        assert!(doc.contains(&format!("oldid={oldid}")));
        assert!(doc.contains(boundary));
        assert!(matches[0].contains("非全回对校"));
    }
    for anchor in [
        "正文未明确点名放箭者", "赵云刺死邢道荣", "张飞生擒刘贤",
        "李典先擒内应", "吕范提议、孙权批准、贾华布置",
        "第055回", "不等于正常地图多种子自然可达",
    ] {
        assert!(doc.contains(anchor), "missing causality evidence: {anchor}");
    }
    for (id, ch, flag, pre) in [
        ("jingxiang_lease_ledger", 52, "jingxiang_lease_record", "nanjun_claim_record"),
        ("changsha_mercy", 53, "changsha_mercy_record", "huarong_mercy"),
        ("hefei_memorial", 53, "hefei_casualty_record", "jiangdong_muster_record"),
        ("jingzhou_pledge_copy", 54, "jingzhou_pledge_copy_record", "jingxiang_lease_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.flag, b.prerequisite), (ch, flag, Some(pre)));
    }
    assert!(beat("changsha_mercy").unwrap().premise.contains("正文未明写放箭者"));
    assert!(beat("hefei_memorial").unwrap().premise.contains("李典擒戈定"));
    assert!(beat("jingzhou_pledge_copy").unwrap().premise.contains("吕范提议、孙权准许、贾华布置"));
}

#[test]
fn zhuge_lease_then_pledge_is_atomic_persistent_and_idempotent() {
    let mut g = Game::new(Hero::ZhugeLiang, 52054);
    g.lore_flags.push("nanjun_claim_record".into());
    enter(&mut g, "jingxiang_lease_ledger");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "jingxiang_lease_record"), 1);
    let mut restored = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut restored, "jingzhou_pledge_copy");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "jingzhou_pledge_copy_record"), 1);
    let mut restored = Game::from_json(&restored.to_json().unwrap()).unwrap();
    enter(&mut restored, "jingzhou_pledge_copy");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "jingzhou_pledge_copy_record"), 1);
}

#[test]
fn refusing_lease_does_not_forge_pledge_or_break_old_save() {
    let mut g = Game::new(Hero::ZhugeLiang, 52055);
    g.lore_flags.push("nanjun_claim_record".into());
    enter(&mut g, "jingxiang_lease_ledger");
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&g, "jingxiang_lease_record"), 0);
    assert_eq!(count(&Game::from_json(&g.to_json().unwrap()).unwrap(), "jingzhou_pledge_copy_record"), 0);
    // Existing saves may already hold the predecessor flag; no new scene is mandatory.
    let mut old = Game::new(Hero::ZhugeLiang, 52056);
    old.lore_flags.push("jingxiang_lease_record".into());
    enter(&mut old, "jingzhou_pledge_copy");
    old.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&old, "jingzhou_pledge_copy_record"), 1);
}

#[test]
fn guan_changsha_and_sun_hefei_choices_are_separate_and_refusable() {
    let mut guan = Game::new(Hero::GuanYu, 53054);
    guan.lore_flags.push("huarong_mercy".into());
    enter(&mut guan, "changsha_mercy");
    let before = guan.to_json().unwrap();
    assert!(guan.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(guan.to_json().unwrap(), before);
    guan.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&guan, "changsha_mercy_record"), 1);
    assert_eq!(count(&guan, "hefei_casualty_record"), 0);
    let mut sun = Game::new(Hero::SunShangxiang, 53055);
    sun.lore_flags.push("jiangdong_muster_record".into());
    enter(&mut sun, "hefei_memorial");
    sun.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&sun, "hefei_casualty_record"), 0);
    assert_eq!(count(&Game::from_json(&sun.to_json().unwrap()).unwrap(), "hefei_casualty_record"), 0);
}
