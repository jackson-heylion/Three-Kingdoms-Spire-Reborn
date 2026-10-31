//! Pinned dual-edition local variants only; forced Story is not natural-map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(items: &[String], id: &str) -> usize {
    items.iter().filter(|x| x.as_str() == id).count()
}

#[test]
fn fixed_sources_and_playable_beat_contracts_are_pinned() {
    let doc = include_str!("../docs/CHAPTERS_022_024_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in ["2567985", "2567986", "2567987", "2554687", "2554670",
                "2554689", "2554692", "2554705", "2554649"] {
        assert!(doc.contains(rev), "missing fixed source {rev}");
    }
    for anchor in ["三十萬", "十萬", "情志既動", "李典", "樂進", "數十騎",
                   "十數騎", "原刻影印", "0/120"] {
        assert!(doc.contains(anchor), "missing evidence {anchor}");
    }
    let rows: Vec<_> = audit.lines().skip(1).filter(|s| !s.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for ch in 22..=24 {
        assert!(rows.iter().any(|r| r.starts_with(&format!("{ch:03}\t"))
            && r.contains("CHAPTERS_022_024_JIAJING_VARIANTS.md")));
    }
    for (id, ch, flag, gold, heal, anchor) in [
        ("wangzhong_prisoners", 22, "wangzhong_release_record", 17, 6, "嘉靖043"),
        ("jiping_testimony", 23, "jiping_case_record", 14, 5, "嘉靖045"),
        ("xiapi_household", 24, "xiapi_household_ledger", 14, 8, "嘉靖048"),
        ("palace_ledger", 24, "palace_purge_record", 16, 5, "嘉靖047"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.flag, b.reward_gold, b.reward_heal),
                   (ch, flag, gold, heal));
        assert!(b.premise.contains(anchor), "{id} lacks variant provenance");
    }
}

#[test]
fn guan_release_and_household_are_atomic_optional_and_save_compatible() {
    let mut game = Game::new(Hero::GuanYu, 22024);
    game.lore_flags.push("wenjiu_oath".into());
    for id in ["wangzhong_prisoners", "xiapi_household"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&game.lore_flags, "wangzhong_release_record"), 1);
    assert_eq!(count(&game.lore_flags, "xiapi_household_ledger"), 1);
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut restored, "xiapi_household");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "xiapi_household_ledger"), 1);
}

#[test]
fn cao_refusal_cannot_forge_purge_and_legacy_guandu_stays_available() {
    let mut game = Game::new(Hero::CaoCao, 23024);
    game.lore_flags.push("xuhuang_honor".into());
    enter(&mut game, "jiping_testimony");
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game.lore_flags, "jiping_case_record"), 1);
    enter(&mut game, "palace_ledger");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&game.lore_flags, "palace_purge_record"), 0);
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    restored.lore_flags.push("xu_capital".into());
    enter(&mut restored, "guandu");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "guandu_grain"), 1);
}
