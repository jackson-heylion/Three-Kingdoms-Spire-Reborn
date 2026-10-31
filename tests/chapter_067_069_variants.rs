//! Pinned Mao/Jiajing local variants, not full collation or natural-map reachability.
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
fn fixed_revision_variants_and_existing_rewards() {
    let doc = include_str!("../docs/CHAPTERS_067_069_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for revision in [
        "2568030", "2568031", "7907113", "2558051", "2558050",
        "2558049", "2554744", "2554763", "2554790",
    ] {
        assert!(doc.contains(revision), "missing revision {revision}");
    }
    for anchor in [
        "孫權再三勸止", "自與甘甯、凌统二人和解", "甲兵百餘圍住",
        "曹操親引甲兵五百人圍住", "腰斬於市", "皆斬於市",
        "0/120", "原刻影印",
    ] {
        assert!(doc.contains(anchor), "missing source anchor {anchor}");
    }
    let rows: Vec<_> = audit.lines().skip(1).filter(|row| !row.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for chapter in 67..=69 {
        assert!(rows.iter().any(|row| {
            row.starts_with(&format!("{chapter:03}\t"))
                && row.contains("CHAPTERS_067_069_JIAJING_VARIANTS.md")
        }));
    }
    for (id, chapter, prerequisite, flag, gold, heal, provenance) in [
        ("xiaoyao_ferry_witness", 67, "hefei_casualty_record", "xiaoyao_ferry_record", 19, 9, "嘉靖134"),
        ("ruxu_rescue_roll", 68, "xiaoyao_ferry_record", "ruxu_rescue_record", 21, 9, "嘉靖134"),
        ("zuoci_court_record", 68, "hanzhong_civil_record", "zuoci_court_record", 20, 8, "甲兵百余"),
        ("xudu_fire_inquest", 69, "zuoci_court_record", "xudu_fire_record", 20, 8, "腰斩于市"),
    ] {
        let record = beat(id).unwrap();
        assert_eq!((record.chapter, record.prerequisite, record.flag), (chapter, Some(prerequisite), flag));
        assert_eq!((record.reward_gold, record.reward_heal), (gold, heal));
        assert!(record.premise.contains(provenance), "{id}");
    }
}

#[test]
fn sun_bridge_and_ruxu_records_restore_without_double_reward() {
    let mut game = Game::new(Hero::SunShangxiang, 67069);
    game.lore_flags.push("hefei_casualty_record".into());
    for id in ["xiaoyao_ferry_witness", "ruxu_rescue_roll"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&game.easter_unlocked, "hefei_ruxu_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "ruxu_rescue_roll");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "hefei_ruxu_debt"), 1);
    assert_eq!(count(&restored.lore_flags, "ruxu_rescue_record"), 1);
}

#[test]
fn cao_cross_chapter_evidence_is_optional_and_atomic() {
    let mut game = Game::new(Hero::CaoCao, 67070);
    game.lore_flags.push("zhangsong_envoy_record".into());
    for id in ["hanzhong_civil_ledger", "zuoci_court_record", "xudu_fire_inquest"] {
        enter(&mut game, id);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&game.easter_unlocked, "hanzhong_zuoci_debt"), 1);
    assert_eq!(count(&game.lore_flags, "xudu_fire_record"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    enter(&mut restored, "xudu_fire_inquest");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "hanzhong_zuoci_debt"), 1);
    assert_eq!(count(&restored.lore_flags, "xudu_fire_record"), 1);

    let mut refused = Game::new(Hero::CaoCao, 67071);
    refused.lore_flags.push("zuoci_court_record".into());
    enter(&mut refused, "xudu_fire_inquest");
    let before = refused.to_json().unwrap();
    assert!(refused.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(refused.to_json().unwrap(), before);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&refused.lore_flags, "xudu_fire_record"), 0);
    assert_eq!(count(&refused.easter_unlocked, "xudu_hanshui_debt"), 0);
}
