//! Fixed electronic-witness spotchecks for 037–039 and existing playable consequences.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(values: &[String], key: &str) -> usize {
    values.iter().filter(|v| v.as_str() == key).count()
}
#[test]
fn pinned_second_witnesses_and_legacy_ids() {
    let evidence = include_str!("../docs/CHAPTERS_037_039_JIAJING_VARIANTS.md");
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2567998", "4256828", "2567999", "2608779", "2554644", "2554713", "2554715", "2606312", "2554746"] {
        assert!(evidence.contains(oldid), "missing fixed revision {oldid}");
    }
    for anchor in ["徐庶", "東郭", "一千五百", "原刻", "史料"] {
        assert!(evidence.contains(anchor), "missing local witness anchor {anchor}");
    }
    assert_eq!(ledger.lines().skip(1).filter(|row| !row.is_empty()).count(), 120);
    for chapter in 37..=39 {
        assert!(ledger.lines().any(|row| row.starts_with(&format!("{chapter:03}\t"))
            && row.contains("CHAPTERS_037_039_JIAJING_VARIANTS.md")));
    }
    for (id, chapter, prerequisite, flag, anchor) in [
        ("xushu_departure", 37, "fancheng_counsel_record", "xushu_departure_record", "嘉靖073"),
        ("longzhong_conditions", 38, "longzhong_visit_record", "longzhong_conditions_record", "嘉靖075"),
        ("bowang_orders", 39, "longzhong_conditions_record", "bowang_orders_record", "一千五百"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.prerequisite, b.flag), (chapter, Some(prerequisite), flag));
        assert!(b.premise.contains(anchor), "missing source distinction in {id}");
    }
}
#[test]
fn zhuge_chain_unlocks_only_after_redcliff_and_restores_once() {
    let mut game = Game::new(Hero::ZhugeLiang, 37039);
    for id in ["longzhong", "longzhong_letter", "longzhong_conditions", "bowang_orders"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    for flag in ["longzhong_plan", "longzhong_visit_record", "longzhong_conditions_record", "bowang_orders_record"] {
        assert_eq!(count(&game.lore_flags, flag), 1);
    }
    assert_eq!(count(&game.easter_unlocked, "longzhong_strategy_debt"), 0);
    enter(&mut game, "redcliff");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game.easter_unlocked, "longzhong_strategy_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "redcliff");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "longzhong_strategy_debt"), 1);
}
#[test]
fn guan_refusal_and_sun_pardon_are_isolated() {
    let mut guan = Game::new(Hero::GuanYu, 37040);
    guan.lore_flags.push("fancheng_counsel_record".into());
    enter(&mut guan, "xushu_departure");
    guan.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&guan.lore_flags, "xushu_departure_record"), 0);
    let restored = Game::from_json(&guan.to_json().unwrap()).unwrap();
    assert_eq!(count(&restored.lore_flags, "xushu_departure_record"), 0);
    let mut sun = Game::new(Hero::SunShangxiang, 37041);
    sun.lore_flags.push("sun_succession_account".into());
    for id in ["jiangdong_hostage", "jiangxia_pardon"] {
        enter(&mut sun, id);
        sun.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&sun.lore_flags, "jiangxia_pardon_record"), 1);
    let mut restored = Game::from_json(&sun.to_json().unwrap()).unwrap();
    enter(&mut restored, "jiangxia_pardon");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "jiangxia_pardon_record"), 1);
}

#[test]
fn bowang_is_optional_for_existing_redcliff_conclusion() {
    // This conclusion predates the chapter-039 optional witness. Requiring the
    // new Bowang flag would silently lock out existing V2 campaigns.
    let mut game = Game::new(Hero::ZhugeLiang, 37042);
    for id in ["longzhong", "longzhong_letter", "longzhong_conditions"] {
        enter(&mut game, id);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    enter(&mut game, "bowang_orders");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&game.lore_flags, "bowang_orders_record"), 0);
    enter(&mut game, "redcliff");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game.easter_unlocked, "longzhong_strategy_debt"), 1);
    let snapshot = game.to_json().unwrap();
    let mut restored = Game::from_json(&snapshot).unwrap();
    enter(&mut restored, "redcliff");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "longzhong_strategy_debt"), 1);

    // A missing required Longzhong witness must still block the conclusion.
    let mut incomplete = Game::new(Hero::ZhugeLiang, 37043);
    enter(&mut incomplete, "longzhong");
    incomplete.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut incomplete, "redcliff");
    incomplete.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&incomplete.easter_unlocked, "longzhong_strategy_debt"), 0);
}
