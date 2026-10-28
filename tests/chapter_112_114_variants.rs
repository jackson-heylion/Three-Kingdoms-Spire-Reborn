//! P1④: source-specific partial Jiajing/Mao differences in chapters 112–114.
use three_kingdoms_spire_reborn::lore::beat;

#[test]
fn jiajing_112_114_variants_are_partial_and_keep_gameplay_contracts() {
    let evidence = include_str!("../docs/CHAPTERS_112_114_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for (chapter, oldid, jiajing, mao, id, flag, prerequisite, gold) in [
        (112, "2568074", "胡奋", "胡遵",
         "shouchun_city_captives", "shouchun_city_captives_record", "wei_huainan_summons_record", 25),
        (112, "2568074", "伏愚子", "或問",
         "qiaozhou_war_counsel", "qiaozhou_war_counsel_record", "dangu_casualty_roll_record", 24),
        (113, "2568076", "魏朝新见废曹芳", "亂賊誣伊尹",
         "sunxiu_court_review", "sunxiu_court_review_record", "shouchun_wu_relief_record", 24),
        (114, "2568077", "曹髦之首", "早晚圖之",
         "wei_caomao_palace_evidence", "caomao_palace_evidence_record", "caofang_deposition_record", 24),
    ] {
        assert!(evidence.contains(oldid), "fixed Mao source {chapter}");
        assert!(evidence.contains(jiajing), "Jiajing source {chapter}");
        assert!(evidence.contains(mao), "Mao source {chapter}");
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("CHAPTERS_112_114_JIAJING_VARIANTS.md"));
        let scene = beat(id).unwrap();
        assert_eq!(scene.chapter, chapter);
        assert!(scene.premise.contains("嘉靖本"));
        assert_eq!(scene.flag, flag);
        assert_eq!(scene.prerequisite, Some(prerequisite));
        assert_eq!(scene.reward_gold, gold);
    }
    let after = beat("qishan_retreat_courier").unwrap();
    assert_eq!(after.prerequisite, Some("qiaozhou_war_counsel_record"));
    let family = beat("wei_wangjing_family_relief").unwrap();
    assert_eq!(family.prerequisite, Some("caomao_palace_evidence_record"));
    assert!(evidence.contains("非全回校勘"));
    assert!(evidence.contains("累计全回校勘仍0/120"));
}
