//! Chapter IX ends in Wang Yun's death and Chang'an's fall, not stable peace.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage, content::Hero, lore::{beat, next_beat},
};

#[test]
fn ninth_chapter_is_between_pavilion_and_the_two_later_debts() {
    let pavilion = beat("pavilion").unwrap();
    let changan = beat("changan").unwrap();
    let halberd = beat("halberd").unwrap();
    let whitegate = beat("whitegate").unwrap();
    assert_eq!((pavilion.chapter, changan.chapter, halberd.chapter, whitegate.chapter),
        (8, 9, 16, 19));
    assert_eq!(changan.prerequisite, Some("phoenix_pavilion"));
    assert_eq!(halberd.prerequisite, Some("phoenix_pavilion"),
        "older saves must remain able to visit the later act without chapter IX");
    assert_eq!(whitegate.prerequisite, Some("halberd_shot"));
    for detail in ["李肃", "蔡邕", "王允", "贾诩", "长安", "失城"] {
        assert!(changan.premise.contains(detail), "missing IX causality: {detail}");
    }
    assert!(halberd.premise.contains("长安"));
    assert!(whitegate.premise.contains("长安"));
    let seen = vec!["redhare".into(), "pavilion".into()];
    assert_ne!(next_beat(Hero::LuBu, 0, &[], &seen).map(|b| b.id), Some("changan"));
    assert_eq!(next_beat(Hero::LuBu, 0, &["phoenix_pavilion".into()], &seen)
        .map(|b| b.id), Some("changan"));
    assert_eq!(next_beat(Hero::LuBu, 1, &["phoenix_pavilion".into()], &[])
        .map(|b| b.id), Some("halberd"));
}

#[test]
fn original_choices_are_mutually_exclusive_and_saveable() {
    for (choice, expected) in [(1, true), (2, false), (3, false)] {
        let mut game = Game::new(Hero::LuBu, 9009);
        game.stage = Stage::Story;
        game.lore_current = Some("changan".into());
        let old = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), old);
        game.apply(Action::Choose { index: choice }).unwrap();
        let flagged = game.lore_flags.iter().any(|f| f == "changan_account");
        assert_eq!(flagged, expected);
        assert!(game.lore_seen.iter().any(|id| id == "changan"));
        let snapshot = game.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(), snapshot);
    }
}
