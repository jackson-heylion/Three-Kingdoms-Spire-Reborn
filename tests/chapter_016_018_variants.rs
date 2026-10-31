//! Fixed electronic Mao/Jiajing spot-checks and actual Game::apply regression, not full collation.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(items: &[String], id: &str) -> usize {
    items.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn pinned_editions_are_spotchecks_and_old_rewards_are_unchanged() {
    let doc = include_str!("../docs/CHAPTERS_016_018_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for revision in ["2567978", "2567979", "2567980", "2554678", "2554701", "7904568", "2554696", "2554666", "2554664"] {
        assert!(doc.contains(revision), "missing pinned revision {revision}");
    }
    for anchor in ["其功一也", "一百五十匹", "此張紘所作也", "十七萬", "三十萬", "史官云", "繡守襄城", "繡守穰城", "原刻影印", "0回"] {
        assert!(doc.contains(anchor), "missing actual witness anchor {anchor}");
    }
    assert_eq!(audit.lines().skip(1).filter(|x| !x.is_empty()).count(), 120);
    for ch in 16..=18 {
        let prefix = format!("{ch:03}\t");
        assert!(audit.lines().any(|row| row.starts_with(&prefix) && row.contains("CHAPTERS_016_018_JIAJING_VARIANTS.md")));
    }
    for (id, chapter, flag, gold, heal, anchor) in [
        ("halberd", 16, "halberd_shot", 35, 8, "三功"),
        ("seven_armies", 17, "seven_armies_account", 14, 5, "九端"),
        ("shouchun_grain", 17, "shouchun_grain_account", 12, 6, "十七万"),
        ("anzhong", 18, "anzhong_field_report", 16, 6, "穰城"),
        ("xiaopei_courier", 18, "xiaopei_courier_guard", 12, 7, "嘉靖第036"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.flag, b.reward_gold, b.reward_heal), (chapter, flag, gold, heal));
        assert!(b.premise.contains(anchor), "{id} missing source distinction");
    }
}

#[test]
fn lu_bu_three_chapter_choices_pay_once_at_white_gate_and_survive_v2() {
    let mut game = Game::new(Hero::LuBu, 16018);
    game.lore_flags.push("phoenix_pavilion".into());
    for (id, flag) in [
        ("halberd", "halberd_shot"),
        ("seven_armies", "seven_armies_account"),
        ("xiaopei_courier", "xiaopei_courier_guard"),
    ] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(count(&game.lore_flags, flag), 1);
    }
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut restored, "whitegate");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "seven_armies_debt"), 1);
    assert_eq!(count(&restored.easter_unlocked, "xiaopei_courier_reckoning"), 1);
    enter(&mut restored, "whitegate");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "seven_armies_debt"), 1);
    assert_eq!(count(&restored.easter_unlocked, "xiaopei_courier_reckoning"), 1);

    let mut refused = Game::new(Hero::LuBu, 16019);
    refused.lore_flags.push("halberd_shot".into());
    enter(&mut refused, "seven_armies");
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&refused.lore_flags, "seven_armies_account"), 0);
    assert_eq!(next_beat(Hero::LuBu, 2, &refused.lore_flags, &[]).unwrap().id, "whitegate");
    enter(&mut refused, "whitegate");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused.easter_unlocked, "seven_armies_debt"), 0);
}

#[test]
fn cao_grain_and_anzhong_reports_are_optional_and_paid_once_at_guandu() {
    let mut game = Game::new(Hero::CaoCao, 17018);
    game.lore_flags.extend(["puyang_supply", "xu_capital"].map(str::to_string));
    for (id, flag) in [
        ("shouchun_grain", "shouchun_grain_account"),
        ("anzhong", "anzhong_field_report"),
    ] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 0 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(count(&game.lore_flags, flag), 1);
    }
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut restored, "guandu");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "shouchun_grain_debt"), 1);
    assert_eq!(count(&restored.easter_unlocked, "anzhong_reckoning"), 1);
    enter(&mut restored, "guandu");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked, "shouchun_grain_debt"), 1);
    assert_eq!(count(&restored.easter_unlocked, "anzhong_reckoning"), 1);

    let mut refused = Game::new(Hero::CaoCao, 17019);
    refused.lore_flags.push("xu_capital".into());
    assert_eq!(next_beat(Hero::CaoCao, 1, &refused.lore_flags, &[]).unwrap().id, "guandu");
    enter(&mut refused, "guandu");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused.easter_unlocked, "anzhong_reckoning"), 0);
}
