//! P1④: real, distinct Jiajing-vs-Mao passages, not a claim of complete collation.
use three_kingdoms_spire_reborn::lore::beat;

#[test]
fn three_late_chapters_keep_two_novel_sources_and_partial_status_separate() {
    let evidence = include_str!("../docs/CHAPTERS_118_120_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for (chapter, mao_oldid, jiajing_phrase, mao_phrase, id, flag, prerequisite) in [
        (118, "2568082", "金银二千斤", "金銀三千斤",
         "chengdu_surrender_household", "chengdu_surrender_household_record", "mianzhu_family_casualties_record"),
        (119, "2568081", "乃吾儿之天下也", "乃吾兄之天下也",
         "wei_jin_abdication_court", "wei_jin_abdication_record", "zhonghui_false_edict_record"),
        (120, "2568085", "四州四十三郡", "四州八十三郡",
         "wu_stonecity_surrender_roll", "wu_stonecity_surrender_record", "sunhao_lukang_memorial_record"),
    ] {
        assert!(evidence.contains(mao_oldid));
        assert!(evidence.contains(jiajing_phrase), "chapter {chapter} Jiajing quote");
        assert!(evidence.contains(mao_phrase), "chapter {chapter} Mao quote");
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter:03}"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("嘉靖本第二小说文本局部对校"));
        let scene = beat(id).unwrap();
        assert_eq!(scene.chapter, chapter);
        assert!(scene.premise.contains("嘉靖本"));
        assert_eq!(scene.flag, flag);
        assert_eq!(scene.prerequisite, Some(prerequisite));
    }
    assert!(evidence.contains("未") && evidence.contains("影印"));
    assert!(evidence.contains("累计仍0/120"));
}
