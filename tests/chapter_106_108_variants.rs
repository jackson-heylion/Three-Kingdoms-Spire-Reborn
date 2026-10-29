//! P1④: immutable electronic Jiajing/Mao spot checks for chapters 106–108.
use three_kingdoms_spire_reborn::lore::beat;

#[test]
fn fixed_jiajing_106_108_variants_preserve_existing_playable_contracts() {
    let evidence = include_str!("../docs/CHAPTERS_106_108_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for (chapter, mao, jiajing, id, flag, prerequisite, gold) in [
        (106, "7909163", "2558415", "liaodong_rain_siege_ledger", "liaodong_rain_siege_record", "guandu_grain", 25),
        (107, "2568071", "2558416", "gaoping_oath_court_review", "gaoping_oath_court_record", "wei_regency_sickbed_record", 26),
        (108, "2568069", "2626693", "qushan_crossbow_aftercare", "qushan_crossbow_aftercare_record", "qushan_water_rescue_record", 22),
        (108, "2568069", "2626693", "dongxing_snow_bridge_ledger", "dongxing_snow_bridge_record", "yiling_ashes", 24),
    ] {
        assert!(evidence.contains(mao), "fixed Mao revision for {chapter}");
        assert!(evidence.contains(jiajing), "fixed Jiajing revision for {chapter}");
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("CHAPTERS_106_108_JIAJING_VARIANTS.md"));
        let scene = beat(id).unwrap();
        assert_eq!(scene.chapter, chapter);
        assert!(scene.premise.contains("嘉靖本"));
        assert!(scene.premise.contains("毛评本"));
        assert_eq!(scene.flag, flag);
        assert_eq!(scene.prerequisite, Some(prerequisite));
        assert_eq!(scene.reward_gold, gold);
    }
    for token in ["司馬陳珪", "司馬陳羣", "皆宰馬為食", "皆宰牛馬為食",
        "畢軌", "畢範", "十數騎", "數十騎", "一弩發十箭", "一弩發十矢",
        "勿生他意", "善理國政", "非逐字全回异文校勘", "仍0/120"] {
        assert!(evidence.contains(token), "missing anchored source text: {token}");
    }
    assert_eq!(beat("wei_regency_sickbed_register").unwrap().prerequisite, Some("liaodong_rain_siege_record"));
    assert_eq!(beat("xincheng_siege_sickness").unwrap().prerequisite, Some("dongxing_snow_bridge_record"));
}


// These tests enter optional archives through scripted Story stages. They
// exercise real Game::apply, not natural-map discovery or human play.
fn staged_archive(
    hero: three_kingdoms_spire_reborn::content::Hero,
    id: &str,
    flags: &[&str],
) -> three_kingdoms_spire_reborn::Game {
    use three_kingdoms_spire_reborn::{Game, Stage};
    let mut game = Game::new(hero, 106108);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|flag| (*flag).into()).collect();
    game
}

fn enter_archive(game: &mut three_kingdoms_spire_reborn::Game, id: &str) {
    game.stage = three_kingdoms_spire_reborn::Stage::Story;
    game.lore_current = Some(id.into());
}

fn egg_count(game: &three_kingdoms_spire_reborn::Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|egg| egg.as_str() == id).count()
}

#[test]
fn ch106_107_wei_liaodong_gaoping_debt_is_atomic_once_only_and_restorable() {
    use three_kingdoms_spire_reborn::{Action, Game};
    use three_kingdoms_spire_reborn::content::Hero;

    let mut game = staged_archive(Hero::CaoCao, "wei_palace_labor_register", &["guandu_grain"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before, "invalid choice must roll back everything");

    game.apply(Action::Choose { index: 1 }).unwrap();
    enter_archive(&mut game, "liaodong_rain_siege_ledger");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "liaodong_rain_siege_record"));
    assert_eq!(egg_count(&game, "wei_palace_gaoping_oath_debt"), 0);

    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter_archive(&mut restored, "wei_regency_sickbed_register");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    enter_archive(&mut restored, "gaoping_oath_court_review");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&restored, "wei_palace_gaoping_oath_debt"), 1);
    // Re-entering the optional archive after a V2 reload must not pay the same egg twice.
    let settled = restored.to_json().unwrap();
    let mut replayed = Game::from_json(&settled).unwrap();
    enter_archive(&mut replayed, "gaoping_oath_court_review");
    replayed.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&replayed, "wei_palace_gaoping_oath_debt"), 1);
    assert!(three_kingdoms_spire_reborn::lore::newly_completed_eggs(
        restored.hero, &restored.lore_flags, &restored.easter_unlocked
    ).iter().all(|egg| egg.id != "wei_palace_gaoping_oath_debt"));

    let mut declined = staged_archive(Hero::CaoCao, "wei_palace_labor_register", &["guandu_grain"]);
    declined.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!declined.lore_flags.iter().any(|f| f == "wei_palace_labor_record"));
    enter_archive(&mut declined, "liaodong_rain_siege_ledger");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    enter_archive(&mut declined, "wei_regency_sickbed_register");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    enter_archive(&mut declined, "gaoping_oath_court_review");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&declined, "wei_palace_gaoping_oath_debt"), 0);
}

#[test]
fn ch107_108_shu_water_chain_and_wu_dongxing_debt_keep_exclusive_rewards() {
    use three_kingdoms_spire_reborn::{Action, Game};
    use three_kingdoms_spire_reborn::content::Hero;

    let mut shu = staged_archive(
        Hero::ZhugeLiang, "qushan_water_rescue_dispatch",
        &["wuzhang_withdrawal_record", "weiyan_double_memorial_record"],
    );
    shu.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&shu, "wuzhang_qushan_supply_debt"), 1);
    enter_archive(&mut shu, "qushan_crossbow_aftercare");
    let before = shu.to_json().unwrap();
    assert!(shu.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(shu.to_json().unwrap(), before);
    shu.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&shu, "wuzhang_qushan_supply_debt"), 1);

    let mut wu = staged_archive(Hero::SunShangxiang, "dongxing_snow_bridge_ledger", &["yiling_ashes"]);
    wu.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&wu, "yiling_dongxing_snow_debt"), 1);
    let saved = wu.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    enter_archive(&mut restored, "xincheng_siege_sickness");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    enter_archive(&mut restored, "sun_jun_banquet_inquest");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&restored, "yiling_dongxing_snow_debt"), 1);
    // A restored campaign may revisit the same optional archive, but its hidden payoff is once-only.
    enter_archive(&mut restored, "dongxing_snow_bridge_ledger");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&restored, "yiling_dongxing_snow_debt"), 1);

    let mut declined = staged_archive(Hero::SunShangxiang, "dongxing_snow_bridge_ledger", &["yiling_ashes"]);
    declined.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!declined.lore_flags.iter().any(|f| f == "dongxing_snow_bridge_record"));
    assert_eq!(egg_count(&declined, "yiling_dongxing_snow_debt"), 0);
}
