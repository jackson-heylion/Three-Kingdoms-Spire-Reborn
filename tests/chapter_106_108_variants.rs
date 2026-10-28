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
