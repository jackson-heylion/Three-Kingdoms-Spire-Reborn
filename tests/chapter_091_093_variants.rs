//! 091-093 fixed JiaJing/Mao textual spotchecks and real story state transitions.
//! Scripted Story entry is not proof of natural campaign reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
}
fn flag_count(game: &Game, id: &str) -> usize {
    game.lore_flags.iter().filter(|flag| flag.as_str() == id).count()
}
fn egg_count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|egg| egg.as_str() == id).count()
}
#[test]
fn fixed_dual_edition_evidence_and_six_playable_contracts() {
    let evidence = include_str!("../docs/CHAPTERS_091_093_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for revision in [
        "7909162", "2568054", "2568056", "2554832", "2554808",
        "2554787", "2554757", "2554809", "2555860",
    ] {
        assert!(evidence.contains(revision), "missing fixed revision {revision}");
    }
    for token in ["數百", "千餘", "事物紀源", "马超", "五千", "三千", "王朗"] {
        assert!(evidence.contains(token), "missing anchored variant {token}");
    }
    assert_eq!(audit.lines().skip(1).filter(|row| !row.is_empty()).count(), 120);
    for chapter in 91..=93 {
        let prefix = format!("{chapter:03}\t");
        let row = audit.lines().find(|row| row.starts_with(&prefix)).unwrap();
        assert!(row.contains("CHAPTERS_091_093_JIAJING_VARIANTS.md"));
        assert!(row.contains("局部"));
    }
    for (id, chapter, pre, flag, gold, heal) in [
        ("lushui_memorial_rites", 91, "nanzhong_local_rule_record", "lushui_memorial_record", 24, 12),
        ("chushi_memorial_supply", 91, "lushui_memorial_record", "chushi_supply_record", 24, 10),
        ("ziwu_route_risk_hearing", 92, "chushi_supply_record", "ziwu_risk_record", 25, 10),
        ("tianshui_false_dispatch_review", 92, "ziwu_risk_record", "tianshui_false_dispatch_record", 25, 11),
        ("tianshui_recruitment_receipt", 93, "tianshui_false_dispatch_record", "tianshui_recruit_record", 24, 10),
        ("wanglang_death_archive", 93, "guandu_grain", "wanglang_archive_record", 20, 7),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(pre));
        assert_eq!(b.flag, flag);
        assert_eq!((b.reward_gold, b.reward_heal), (gold, heal));
        assert!(b.premise.contains("嘉靖") && b.premise.contains("毛评"), "{id}");
    }
}
#[test]
fn nanzhong_to_tianshui_three_conclusions_are_once_after_v2_restore() {
    let mut game = Game::new(Hero::ZhugeLiang, 91093);
    for flag in ["xier_spring_record", "nanzhong_vine_fire_record", "nanzhong_local_rule_record", "redcliff_wind"] {
        game.lore_flags.push(flag.into());
    }
    for id in ["lushui_memorial_rites", "chushi_memorial_supply", "ziwu_route_risk_hearing",
        "tianshui_false_dispatch_review", "tianshui_recruitment_receipt"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before, "invalid choice mutated {id}");
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    for egg in ["nanzhong_mourning_debt", "redcliff_northern_route_debt", "tianshui_truth_debt"] {
        assert_eq!(egg_count(&game, egg), 1, "{egg}");
    }
    let snapshot = game.to_json().unwrap();
    let mut game = Game::from_json(&snapshot).unwrap();
    assert_eq!(game.to_json().unwrap(), snapshot);
    for id in ["lushui_memorial_rites", "ziwu_route_risk_hearing", "tianshui_recruitment_receipt"] {
        enter(&mut game, id);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    for egg in ["nanzhong_mourning_debt", "redcliff_northern_route_debt", "tianshui_truth_debt"] {
        assert_eq!(egg_count(&game, egg), 1, "{egg}");
    }
    for flag in ["lushui_memorial_record", "chushi_supply_record", "ziwu_risk_record",
        "tianshui_false_dispatch_record", "tianshui_recruit_record"] {
        assert_eq!(flag_count(&game, flag), 1, "{flag}");
    }
}
#[test]
fn profit_choice_cannot_forge_surrender_and_wei_archives_remain_optional() {
    let mut shu = Game::new(Hero::ZhugeLiang, 93093);
    shu.lore_flags.push("chushi_supply_record".into());
    enter(&mut shu, "ziwu_route_risk_hearing");
    shu.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut shu, "tianshui_false_dispatch_review");
    shu.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(flag_count(&shu, "tianshui_false_dispatch_record"), 0);
    enter(&mut shu, "tianshui_recruitment_receipt");
    shu.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&shu, "tianshui_truth_debt"), 0);
    let mut wei = Game::new(Hero::CaoCao, 93094);
    wei.lore_flags.push("guandu_grain".into());
    enter(&mut wei, "wanglang_death_archive");
    let before = wei.to_json().unwrap();
    assert!(wei.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(wei.to_json().unwrap(), before);
    wei.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(flag_count(&wei, "wanglang_archive_record"), 1);
    assert_eq!(Game::from_json(&wei.to_json().unwrap()).unwrap().to_json().unwrap(), wei.to_json().unwrap());
}
