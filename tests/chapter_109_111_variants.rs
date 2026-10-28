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
