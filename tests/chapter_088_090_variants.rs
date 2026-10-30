//! Fixed JiaJing/Mao 088-090 local variants and real accountability transitions.
//! Scripted Story entry does not prove natural campaign reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
}
fn egg_count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}
fn flag_count(game: &Game, id: &str) -> usize {
    game.lore_flags.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn pinned_two_editions_and_four_existing_playable_contracts() {
    let evidence = include_str!("../docs/CHAPTERS_088_090_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for revision in [
        "2568052", "2646272", "2568053", "2554741", "2554779",
        "2554807", "2554782", "2554783", "2554737",
    ] {
        assert!(evidence.contains(revision), "missing fixed revision {revision}");
    }
    for anchor in ["五六百", "二千", "一千", "董荼奴", "董荼那", "司馬溫公", "赤帝施權柄", "赤帝司權柄", "二萬神兵", "三萬神兵", "不留兵"] {
        assert!(evidence.contains(anchor), "missing local variant {anchor}");
    }
    assert_eq!(audit.lines().skip(1).filter(|row| !row.is_empty()).count(), 120);
    for chapter in 88..=90 {
        let row = audit.lines().find(|row| row.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("CHAPTERS_088_090_JIAJING_VARIANTS.md"));
        assert!(row.contains("局部"));
    }
    for (id, chapter, pre, flag, gold, heal) in [
        ("lushui_ferry_care", 88, "nanzhong_first_release_record", "lushui_ferry_record", 24, 11),
        ("xier_spring_civilian_water", 89, "lushui_ferry_record", "xier_spring_record", 25, 12),
        ("nanzhong_vine_fire_account", 90, "xier_spring_record", "nanzhong_vine_fire_record", 26, 11),
        ("nanzhong_local_governance", 90, "nanzhong_vine_fire_record", "nanzhong_local_rule_record", 21, 10),
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
fn lushui_to_seven_captures_to_mourning_is_atomic_and_once_after_restore() {
    let mut game = Game::new(Hero::ZhugeLiang, 88090);
    for flag in ["redcliff_wind", "dengzhi_alliance_record", "nanzhong_yongchang_record", "nanzhong_first_release_record"] {
        game.lore_flags.push(flag.into());
    }
    for id in [
        "lushui_ferry_care", "xier_spring_civilian_water",
        "nanzhong_vine_fire_account", "nanzhong_local_governance",
        "lushui_memorial_rites",
    ] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before, "invalid choice mutated {id}");
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(egg_count(&game, "redcliff_nanzhong_water_debt"), 1);
    assert_eq!(egg_count(&game, "nanzhong_mourning_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut game = Game::from_json(&saved).unwrap();
    assert_eq!(game.to_json().unwrap(), saved);
    for id in ["xier_spring_civilian_water", "nanzhong_vine_fire_account", "lushui_memorial_rites"] {
        enter(&mut game, id);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    for flag in ["lushui_ferry_record", "xier_spring_record", "nanzhong_vine_fire_record", "nanzhong_local_rule_record", "lushui_memorial_record"] {
        assert_eq!(flag_count(&game, flag), 1, "{flag}");
    }
    assert_eq!(egg_count(&game, "redcliff_nanzhong_water_debt"), 1);
    assert_eq!(egg_count(&game, "nanzhong_mourning_debt"), 1);
}

#[test]
fn refusing_the_water_ledger_does_not_forge_five_captures_or_unlock_mourning() {
    let mut game = Game::new(Hero::ZhugeLiang, 89090);
    game.lore_flags.push("redcliff_wind".into());
    game.lore_flags.push("nanzhong_first_release_record".into());
    enter(&mut game, "lushui_ferry_care");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(flag_count(&game, "lushui_ferry_record"), 0);
    enter(&mut game, "xier_spring_civilian_water");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&game, "redcliff_nanzhong_water_debt"), 0);
    assert_eq!(egg_count(&game, "nanzhong_mourning_debt"), 0);
}
