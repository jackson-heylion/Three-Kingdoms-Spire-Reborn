//! P1④: independently read Jiajing 229–235 versus fixed Mao 115–117; partial evidence only.
use three_kingdoms_spire_reborn::lore::beat;

#[test]
fn three_chapters_have_distinct_novel_texts_without_full_collation_claim() {
    let evidence = include_str!("../docs/CHAPTERS_115_117_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for (chapter, oldid, jiajing, mao, id, flag, prerequisite, gold) in [
        (115, "2568078", "提兵二十万", "提兵三十萬",
         "taoyang_xiahouba_casualty", "taoyang_xiahouba_casualty_record", "tanshan_grain_fire_record", 22),
        (116, "2568079", "未知傅佥性命如何", "自刎而死",
         "yangping_gate_betrayal", "yangping_gate_betrayal_record", "tazhong_farming_dispatch_record", 25),
        (117, "2568080", "瞻中箭落马而死", "拔劍自刎而死",
         "mianzhu_family_casualties", "mianzhu_family_casualties_record", "jiange_detour_warning_record", 27),
    ] {
        assert!(evidence.contains(oldid), "chapter {chapter} fixed source");
        assert!(evidence.contains(jiajing), "chapter {chapter} Jiajing evidence");
        assert!(evidence.contains(mao), "chapter {chapter} Mao evidence");
        let row = audit.lines().find(|row| row.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("CHAPTERS_115_117_JIAJING_VARIANTS.md"));
        let scene = beat(id).unwrap();
        assert_eq!(scene.chapter, chapter);
        assert!(scene.premise.contains("嘉靖本"), "chapter {chapter} in-game provenance");
        assert_eq!(scene.flag, flag);
        assert_eq!(scene.prerequisite, Some(prerequisite));
        assert_eq!(scene.reward_gold, gold);
    }
    let relief = beat("wu_mianzhu_relief_dispatch").unwrap();
    assert_eq!(relief.prerequisite, Some("wu_relief_warning_record"));
    assert_eq!(relief.flag, "wu_mianzhu_relief_dispatch_record");
    assert!(relief.premise.contains("第235则"));
    assert!(relief.premise.contains("无吴援及时抵达"));
    assert!(evidence.contains("非全回校勘"));
    assert!(evidence.contains("累计全回校勘仍0/120"));
}
