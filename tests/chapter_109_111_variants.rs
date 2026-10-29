//! P1④: immutable electronic Jiajing/Mao spot checks for chapters 109–111.
use three_kingdoms_spire_reborn::lore::beat;

#[test]
fn fixed_jiajing_109_111_variants_preserve_playable_contracts() {
    let evidence = include_str!("../docs/CHAPTERS_109_111_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for (chapter, mao, jiajing, id, flag, prerequisite, gold) in [
        (109, "2568073", "2558411", "tielong_spring_rescue", "tielong_spring_rescue_record", "qushan_crossbow_aftercare_record", 25),
        (110, "2568072", "2558397", "shouchun_false_edict_audit", "shouchun_false_edict_record", "caofang_deposition_record", 25),
        (111, "2568075", "7908286", "dangu_casualty_roll", "dangu_casualty_roll_record", "taoxi_didao_withdrawal_record", 24),
    ] {
        assert!(evidence.contains(mao), "fixed Mao oldid {chapter}");
        assert!(evidence.contains(jiajing), "fixed Jiajing oldid {chapter}");
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("CHAPTERS_109_111_JIAJING_VARIANTS.md"));
        let scene = beat(id).unwrap();
        assert_eq!(scene.chapter, chapter);
        assert!(scene.premise.contains("嘉靖本"));
        assert_eq!(scene.flag, flag);
        assert_eq!(scene.prerequisite, Some(prerequisite));
        assert_eq!(scene.reward_gold, gold);
    }
    for token in ["百餘蜀兵", "二百餘人", "字仲聞", "字仲恭", "熟麥千頃", "有麥可就食", "非全回逐字异文校勘", "仍0/120"] {
        assert!(evidence.contains(token), "missing anchored evidence: {token}");
    }
    assert_eq!(beat("caofang_deposition_receipt").unwrap().prerequisite, Some("gaoping_oath_court_record"));
    assert_eq!(beat("shouchun_city_captives").unwrap().prerequisite, Some("wei_huainan_summons_record"));
}


// Scripted Story entry checks actual Game::apply transitions. This is not a
// claim that a naturally generated three-act map reaches every archive.
fn staged(
    hero: three_kingdoms_spire_reborn::content::Hero,
    id: &str,
    flags: &[&str],
) -> three_kingdoms_spire_reborn::Game {
    use three_kingdoms_spire_reborn::{Game, Stage};
    let mut game = Game::new(hero, 109111);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|flag| (*flag).into()).collect();
    game
}

fn enter(game: &mut three_kingdoms_spire_reborn::Game, id: &str) {
    game.stage = three_kingdoms_spire_reborn::Stage::Story;
    game.lore_current = Some(id.into());
}

fn unlocked(game: &three_kingdoms_spire_reborn::Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|egg| egg.as_str() == id).count()
}

#[test]
fn ch109_112_wei_false_edict_chain_has_atomic_choices_and_once_only_debts() {
    use three_kingdoms_spire_reborn::{Action, Game};
    use three_kingdoms_spire_reborn::content::Hero;

    let mut game = staged(
        Hero::CaoCao,
        "caofang_deposition_receipt",
        &["gaoping_oath_court_record"],
    );
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before, "rejection must preserve full snapshot");

    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "caofang_deposition_record"));
    assert_eq!(unlocked(&game, "gaoping_shouchun_edict_debt"), 0);

    enter(&mut game, "shouchun_false_edict_audit");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&game, "gaoping_shouchun_edict_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);

    enter(&mut restored, "wei_huainan_summons_audit");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut restored, "shouchun_city_captives");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&restored, "huainan_shouchun_captives_debt"), 1);
    assert_eq!(unlocked(&restored, "gaoping_shouchun_edict_debt"), 1);
    assert!(three_kingdoms_spire_reborn::lore::newly_completed_eggs(
        restored.hero, &restored.lore_flags, &restored.easter_unlocked
    ).iter().all(|egg| egg.id != "gaoping_shouchun_edict_debt"
        && egg.id != "huainan_shouchun_captives_debt"));

    let mut declined = staged(
        Hero::CaoCao,
        "caofang_deposition_receipt",
        &["gaoping_oath_court_record"],
    );
    declined.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!declined.lore_flags.iter().any(|f| f == "caofang_deposition_record"));
    enter(&mut declined, "shouchun_false_edict_audit");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&declined, "gaoping_shouchun_edict_debt"), 0);
}

#[test]
fn ch109_111_shu_water_supply_and_dangu_losses_keep_choice_exclusivity() {
    use three_kingdoms_spire_reborn::{Action, Game};
    use three_kingdoms_spire_reborn::content::Hero;

    let mut game = staged(
        Hero::ZhugeLiang,
        "tielong_spring_rescue",
        &["qushan_crossbow_aftercare_record"],
    );
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "tielong_spring_rescue_record"));
    enter(&mut game, "taoxi_didao_withdrawal");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&game, "taoxi_dangu_casualty_debt"), 0);

    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    enter(&mut restored, "dangu_casualty_roll");
    let before = restored.to_json().unwrap();
    assert!(restored.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(restored.to_json().unwrap(), before);
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&restored, "taoxi_dangu_casualty_debt"), 1);
    assert!(restored.lore_flags.iter().any(|f| f == "dangu_casualty_roll_record"));
    assert_eq!(Game::from_json(&restored.to_json().unwrap()).unwrap().easter_unlocked,
        restored.easter_unlocked);

    let mut declined = staged(
        Hero::ZhugeLiang,
        "taoxi_didao_withdrawal",
        &["tielong_spring_rescue_record"],
    );
    declined.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut declined, "dangu_casualty_roll");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&declined, "taoxi_dangu_casualty_debt"), 0);
}
