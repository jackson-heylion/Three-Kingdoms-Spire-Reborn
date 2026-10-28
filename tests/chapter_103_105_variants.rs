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
    for token in ["皆要燒死", "杖馬岱四十", "禮義廉恥", "鐵矢長八寸",
        "王平、廖化、張嶷、張翼、吳懿", "馬岱、王平、廖化、張翼、張嶷",
        "逆從魏寇", "謀為不軌", "非逐字异文", "仍0/120"] {
        assert!(evidence.contains(token), "missing anchored distinction: {token}");
    }
}
