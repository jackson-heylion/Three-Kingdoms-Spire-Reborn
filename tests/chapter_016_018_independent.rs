//! Independent fixed Mao 016-018 review and rejection regression; not complete collation.
use three_kingdoms_spire_reborn::{Action, Game, Stage, content::Hero, lore};

#[test]
fn independent_mao_016_018_pins_real_boundaries_without_claiming_full_variants() {
    let doc = include_str!("../docs/CHAPTERS_016_018_JIAJING_VARIANTS.md");
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for (chapter, oldid) in [("016", "2567978"), ("017", "2567979"), ("018", "2567980")] {
        let rows: Vec<_> = ledger.lines().filter(|row| row.starts_with(&format!("{chapter}\t"))).collect();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].contains(&format!("2026-11-01独立复核毛评固定oldid={oldid}")));
        assert!(rows[0].contains("非全回异本或史料校勘"));
        assert!(doc.contains(&format!("oldid={oldid}")));
    }
    for marker in ["L119—206", "L119—170", "L119—169", "王垕抗辩无罪",
        "尚未展开017七路战事", "018开篇才揭东南反伏击",
        "刘备尚未败走投曹", "嘉靖原刻影印", "正常战役完整跨幕多种子自然可达"] {
        assert!(doc.contains(marker), "missing evidence: {marker}");
    }
    assert!(lore::beat("halberd").unwrap().premise.contains("回末袁术军来取徐州"));
    assert!(lore::beat("seven_armies").unwrap().premise.contains("不能把王垕写成主动盗粮者"));
    assert!(lore::beat("xiaopei_courier").unwrap().premise.contains("刘备尚未败走投曹"));
    assert!(lore::beat("anzhong").unwrap().premise.contains("018开篇才揭东南伏兵"));
}

#[test]
fn courier_refusal_is_atomic_and_does_not_unlock_whitegate_reckoning_after_v2() {
    let mut game = Game::new(Hero::LuBu, 160181);
    game.lore_flags.extend(["halberd_shot", "seven_armies_account"].map(str::to_string));
    game.stage = Stage::Story;
    game.lore_current = Some("xiaopei_courier".into());
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 3 }).unwrap();
    assert!(!game.lore_flags.iter().any(|f| f == "xiaopei_courier_guard"));
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    restored.stage = Stage::Story;
    restored.lore_current = Some("whitegate".into());
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert!(!restored.easter_unlocked.iter().any(|e| e == "xiaopei_courier_reckoning"));
}
