//! Fixed electronic JiaJing/Mao spotchecks for 082-084 and real, optional campaign consequences.
//! Scripted Story entry is not proof of natural seeded map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
}
fn count_flag(game: &Game, id: &str) -> usize {
    game.lore_flags.iter().filter(|s| s.as_str() == id).count()
}
fn count_egg(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn pinned_sources_are_local_only_and_original_story_contracts_are_stable() {
    let evidence = include_str!("../docs/CHAPTERS_082_084_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in [
        "2568044", "8730382", "2568047", "2554785", "2554740",
        "2554781", "2554831", "2554753", "2554786",
    ] {
        assert!(evidence.contains(rev), "missing fixed source {rev}");
    }
    for token in [
        "上下同心", "馮習令吳班領兵助戰", "吳班令馮習引兵助戰",
        "八千精兵", "十合", "二十餘合", "局部文字对校",
    ] {
        assert!(evidence.contains(token), "missing source distinction {token}");
    }
    assert_eq!(audit.lines().skip(1).filter(|r| !r.is_empty()).count(), 120);
    for chapter in 82..=84 {
        let row = audit.lines().find(|r| r.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("CHAPTERS_082_084_JIAJING_VARIANTS.md"));
        assert!(row.contains("局部"));
    }
    for (id, chapter, pre, flag, gold, heal) in [
        ("wu_wei_envoy_audit", 82, "langzhong_heads_record", "wu_wei_envoy_record", 20, 8),
        ("wei_wu_tribute_review", 82, "abdication_seal_record", "wei_wu_tribute_record", 21, 8),
        ("yiling_defense_memorial", 83, "wu_wei_envoy_record", "yiling_defense_record", 22, 10),
        ("yiling_camp_map_warning", 83, "eastward_counsel_record", "yiling_camp_map_record", 21, 9),
        ("yiling_fire_refuge_ledger", 84, "yiling_defense_record", "yiling_fire_refuge_record", 22, 10),
        ("fishbelly_eightfold_report", 84, "yiling_camp_map_record", "fishbelly_report_record", 22, 10),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(pre));
        assert_eq!(b.flag, flag);
        assert_eq!((b.reward_gold, b.reward_heal), (gold, heal));
    }
    assert!(beat("wu_wei_envoy_audit").unwrap().premise.contains("嘉靖163"));
    assert!(beat("yiling_defense_memorial").unwrap().premise.contains("嘉靖165"));
}

#[test]
fn sun_envoy_defense_and_fire_relief_reward_is_once_after_v2_restore() {
    let mut game = Game::new(Hero::SunShangxiang, 82084);
    for flag in ["jiangdong_covenant", "langzhong_heads_record"] {
        game.lore_flags.push(flag.into());
    }
    for id in ["wu_wei_envoy_audit", "yiling_defense_memorial", "yiling_fire_refuge_ledger"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before, "invalid choice mutated {id}");
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    for flag in ["wu_wei_envoy_record", "yiling_defense_record", "yiling_fire_refuge_record"] {
        assert_eq!(count_flag(&game, flag), 1);
    }
    assert_eq!(count_egg(&game, "yiling_envoy_fire_civilian_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut game = Game::from_json(&saved).unwrap();
    assert_eq!(game.to_json().unwrap(), saved);
    for id in ["wu_wei_envoy_audit", "yiling_defense_memorial", "yiling_fire_refuge_ledger"] {
        enter(&mut game, id);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count_egg(&game, "yiling_envoy_fire_civilian_debt"), 1);
    for flag in ["wu_wei_envoy_record", "yiling_defense_record", "yiling_fire_refuge_record"] {
        assert_eq!(count_flag(&game, flag), 1);
    }
}

#[test]
fn refusing_fire_relief_cannot_unlock_payoff_or_fake_the_flag() {
    let mut game = Game::new(Hero::SunShangxiang, 83084);
    for flag in ["jiangdong_covenant", "wu_wei_envoy_record", "yiling_defense_record"] {
        game.lore_flags.push(flag.into());
    }
    enter(&mut game, "yiling_fire_refuge_ledger");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count_flag(&game, "yiling_fire_refuge_record"), 0);
    assert_eq!(count_egg(&game, "yiling_envoy_fire_civilian_debt"), 0);
    assert_eq!(count_egg(&Game::from_json(&game.to_json().unwrap()).unwrap(), "yiling_envoy_fire_civilian_debt"), 0);
}
