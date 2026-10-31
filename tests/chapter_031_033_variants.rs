//! Fixed electronic-edition spot checks for chapters 031-033; not full collation or natural-map proof.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(g: &Game, flag: &str) -> usize {
    g.lore_flags.iter().filter(|s| s.as_str() == flag).count()
}

#[test]
fn edition_witnesses_and_existing_rewards_are_pinned() {
    let doc = include_str!("../docs/CHAPTERS_031_033_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2567992","2567994","2567995","2554645","2554695",
                  "2554693","2554708","2554694","2554706","2086290"] {
        assert!(doc.contains(oldid), "missing fixed revision {oldid}");
    }
    for anchor in ["民為邦本", "此是操買民心也", "嫡子輕而庶子重",
                   "後史官錄", "裴松之", "原刻影印", "局部史料核证"] {
        assert!(doc.contains(anchor), "missing source boundary {anchor}");
    }
    let rows: Vec<_> = audit.lines().skip(1).filter(|s| !s.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for ch in 31..=33 {
        assert!(rows.iter().any(|s| s.starts_with(&format!("{ch:03}\t"))
            && s.contains("CHAPTERS_031_033_JIAJING_VARIANTS.md")));
    }
    for (id, chapter, prerequisite, flag, gold, heal, anchor) in [
        ("cangting_memorial",31,"guandu_captured_letters","cangting_memorial_record",16,7,"民为邦本"),
        ("ye_flood_ledger",32,"cangting_memorial_record","ye_flood_record",17,8,"嫡子轻而庶子重"),
        ("northern_settlement",33,"liaodong_letter","northern_settlement_account",16,7,"甄皇后传"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter,b.prerequisite,b.flag,b.reward_gold,b.reward_heal),
                   (chapter,Some(prerequisite),flag,gold,heal));
        assert!(b.premise.contains(anchor), "{id} missing source distinction");
    }
}

#[test]
fn cangting_ye_liaodong_chain_is_atomic_optional_and_v2_persistent() {
    let mut g = Game::new(Hero::CaoCao, 31033);
    g.lore_flags.push("guandu_grain".into());
    g.lore_flags.push("guandu_captured_letters".into());
    for id in ["cangting_memorial","ye_flood_ledger","liaodong","northern_settlement"] {
        enter(&mut g,id);
        let before=g.to_json().unwrap();
        assert!(g.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(g.to_json().unwrap(),before);
        g.apply(Action::Choose { index: 1 }).unwrap();
    }
    for flag in ["cangting_memorial_record","ye_flood_record","liaodong_letter",
                 "northern_settlement_account"] {
        assert_eq!(count(&g,flag),1);
    }
    let saved=g.to_json().unwrap();
    let mut restored=Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(),saved);
    enter(&mut restored,"northern_settlement");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored,"northern_settlement_account"),1);
}

#[test]
fn refusals_do_not_lock_old_liaodong_or_forge_ye_responsibility() {
    let mut g=Game::new(Hero::CaoCao,31034);
    g.lore_flags.push("guandu_grain".into());
    g.lore_flags.push("guandu_captured_letters".into());
    enter(&mut g,"cangting_memorial");
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&g,"cangting_memorial_record"),0);
    enter(&mut g,"liaodong");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g,"liaodong_letter"),1);
    enter(&mut g,"northern_settlement");
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&g,"northern_settlement_account"),0);
    let mut restored=Game::from_json(&g.to_json().unwrap()).unwrap();
    restored.lore_flags.push("cangting_memorial_record".into());
    enter(&mut restored,"ye_flood_ledger");
    restored.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&restored,"ye_flood_record"),0);
}
