//! Pinned Mao/Jiajing local variants; forced Story tests are not natural-map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(items: &[String], target: &str) -> usize {
    items.iter().filter(|s| s.as_str() == target).count()
}

#[test]
fn pinned_sources_and_existing_chapter_beat_contracts() {
    let doc = include_str!("../docs/CHAPTERS_064_066_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for id in ["2568026", "2568028", "2568029", "2554734", "2554770",
               "2554827", "2554733", "2554794", "2558066"] {
        assert!(doc.contains(id), "missing fixed source revision {id}");
    }
    for anchor in ["三十餘處", "關隘四十五處", "必要害二十州而還",
                   "務要劉璋割二十州奉還主公", "諫議郎趙儼",
                   "腰斬於市", "左右或奏帝", "0/120", "原刻影印"] {
        assert!(doc.contains(anchor), "missing pinned local evidence {anchor}");
    }
    let rows: Vec<_> = audit.lines().skip(1).filter(|line| !line.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for ch in 64..=66 {
        assert!(rows.iter().any(|line| line.starts_with(&format!("{ch:03}\t"))
            && line.contains("CHAPTERS_064_066_JIAJING_VARIANTS.md")));
    }
    for (id, ch, prerequisite, flag, gold, heal, anchor) in [
        ("jinyan_bridge_review", 64, "luofeng_mourning_record", "jinyan_bridge_record", 20, 8, "三十余处"),
        ("chengdu_land_decree", 65, "jinyan_bridge_record", "chengdu_land_record", 21, 9, "嘉靖129"),
        ("ma_chao_letter", 65, "jingzhou_handover_record", "ma_chao_letter_record", 17, 7, "嘉靖130"),
        ("lukou_terms_record", 66, "ma_chao_letter_record", "lukou_terms_record", 19, 8, "嘉靖132"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.prerequisite, b.flag), (ch, Some(prerequisite), flag));
        assert_eq!((b.reward_gold, b.reward_heal), (gold, heal));
        assert!(b.premise.contains(anchor), "missing provenance in {id}");
    }
}

#[test]
fn zhuge_jinyan_to_chengdu_is_atomic_optional_and_save_compatible() {
    let mut game = Game::new(Hero::ZhugeLiang, 64065);
    game.lore_flags.push("luofeng_mourning_record".into());
    for id in ["jinyan_bridge_review", "chengdu_land_decree"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&game.lore_flags, "jinyan_bridge_record"), 1);
    assert_eq!(count(&game.lore_flags, "chengdu_land_record"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "chengdu_land_decree");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "chengdu_land_record"), 1);
}

#[test]
fn guanyu_lukou_refusal_does_not_forge_evidence() {
    let mut refused = Game::new(Hero::GuanYu, 65066);
    refused.lore_flags.push("jingzhou_handover_record".into());
    enter(&mut refused, "ma_chao_letter");
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&refused.lore_flags, "ma_chao_letter_record"), 0);

    let mut game = Game::new(Hero::GuanYu, 65067);
    game.lore_flags.push("jingzhou_handover_record".into());
    enter(&mut game, "ma_chao_letter");
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "lukou_terms_record");
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&game.lore_flags, "lukou_terms_record"), 0);
    let saved = game.to_json().unwrap();
    assert!(Game::from_json(&saved).is_ok());
}
