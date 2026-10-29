//! Fixed Jiajing/Mao electronic variant anchors for chapters 103–105.
use three_kingdoms_spire_reborn::lore::beat;

#[test]
fn fixed_ch103_105_variants_keep_playable_contracts() {
    let evidence = include_str!("../docs/CHAPTERS_103_105_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for (chapter, mao, jiajing, id, flag, prerequisite, gold, heal) in [
        (103, "2568064", "2557852", "shangfang_fire_account", "shangfang_fire_account_record", "weinan_transport_record", 23, 11),
        (104, "2568066", "2557860", "wuzhang_succession_register", "wuzhang_succession_record", "redcliff_wind", 25, 12),
        (104, "2568066", "2557863", "wuzhang_withdrawal_evidence", "wuzhang_withdrawal_record", "shangfang_fire_account_record", 25, 10),
        (105, "2602415", "2557864", "weiyan_double_memorial_audit", "weiyan_double_memorial_record", "wuzhang_withdrawal_record", 24, 11),
        (105, "2602415", "2557868", "zongyu_wu_border_receipt", "zongyu_border_reply_record", "jiangdong_covenant", 22, 9),
    ] {
        assert!(evidence.contains(mao), "missing Mao oldid {chapter}");
        assert!(evidence.contains(jiajing), "missing Jiajing oldid {chapter}");
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("CHAPTERS_103_105_JIAJING_VARIANTS.md"));
        let scene = beat(id).unwrap();
        assert_eq!(scene.chapter, chapter);
        assert!(scene.premise.contains("嘉靖本"), "{id}");
        assert!(scene.premise.contains("毛评本"), "{id}");
        assert_eq!(scene.flag, flag);
        assert_eq!(scene.prerequisite, Some(prerequisite));
        assert_eq!(scene.reward_gold, gold);
        assert_eq!(scene.reward_heal, heal);
    }
    let fire = beat("shangfang_fire_account").unwrap();
    for name in ["夏侯霸", "夏侯威", "夏侯惠", "夏侯和", "杖背四十"] {
        assert!(fire.premise.contains(name), "missing playable edition distinction: {name}");
    }
    for token in ["皆要燒死", "杖背四十", "夏侯霸、夏侯威", "夏侯惠、夏侯和", "禮義廉恥", "鐵矢長八寸",
        "王平、廖化、張嶷、張翼、吳懿", "馬岱、王平、廖化、張翼、張嶷",
        "逆從魏寇", "謀為不軌", "非逐字异文", "仍0/120"] {
        assert!(evidence.contains(token), "missing anchored distinction: {token}");
    }
}


// These are real Game::apply story transitions with scripted entry conditions.
// They do not claim a naturally generated seven-floor route reaches the chain.
fn staged(hero: three_kingdoms_spire_reborn::content::Hero, id: &str, flags: &[&str]) -> three_kingdoms_spire_reborn::Game {
    use three_kingdoms_spire_reborn::{Game, Stage};
    let mut game = Game::new(hero, 103105);
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
fn variant_shangfang_wuzhang_chain_keeps_atomic_choice_v2_and_once_only_payoff() {
    use three_kingdoms_spire_reborn::{Action, Game};
    use three_kingdoms_spire_reborn::content::Hero;

    let prerequisites = ["weinan_transport_record", "weibei_bridge_losses_record"];
    let mut game = staged(Hero::ZhugeLiang, "shangfang_fire_account", &prerequisites);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before, "rejected choice must not change RNG or flags");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|flag| flag == "shangfang_fire_account_record"));
    assert_eq!(unlocked(&game, "weibei_shangfang_withdrawal_debt"), 0);

    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "wuzhang_withdrawal_evidence");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&restored, "weibei_shangfang_withdrawal_debt"), 1);
    enter(&mut restored, "wuzhang_withdrawal_evidence");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&restored, "weibei_shangfang_withdrawal_debt"), 1);

    let mut declined = staged(Hero::ZhugeLiang, "shangfang_fire_account", &prerequisites);
    declined.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!declined.lore_flags.iter().any(|flag| flag == "shangfang_fire_account_record"));
    enter(&mut declined, "wuzhang_withdrawal_evidence");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&declined, "weibei_shangfang_withdrawal_debt"), 0);
}

#[test]
fn variant_ch104_105_succession_and_double_memorial_remain_distinct_choices() {
    use three_kingdoms_spire_reborn::{Action, Game};
    use three_kingdoms_spire_reborn::content::Hero;

    let mut game = staged(
        Hero::ZhugeLiang,
        "wuzhang_succession_register",
        &["redcliff_wind", "weinan_transport_record"],
    );
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&game, "redcliff_wuzhang_transport_debt"), 1);
    // The withdrawal's actual prerequisite is accepting the Shangfang fire ledger.
    enter(&mut game, "shangfang_fire_account");
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "wuzhang_withdrawal_evidence");
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "weiyan_double_memorial_audit");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|flag| flag == "weiyan_double_memorial_record"));
    assert_eq!(unlocked(&game, "wuzhang_qushan_supply_debt"), 0);
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);

    let mut declined = staged(Hero::ZhugeLiang, "weiyan_double_memorial_audit", &["wuzhang_withdrawal_record"]);
    declined.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!declined.lore_flags.iter().any(|flag| flag == "weiyan_double_memorial_record"));

    let mut wu = staged(Hero::SunShangxiang, "zongyu_wu_border_receipt", &["jiangdong_covenant"]);
    let before = wu.to_json().unwrap();
    assert!(wu.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(wu.to_json().unwrap(), before);
    wu.apply(Action::Choose { index: 1 }).unwrap();
    assert!(wu.lore_flags.iter().any(|flag| flag == "zongyu_border_reply_record"));
    assert_eq!(Game::from_json(&wu.to_json().unwrap()).unwrap().easter_unlocked, wu.easter_unlocked);
}
