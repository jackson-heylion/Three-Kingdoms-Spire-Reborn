//! Chapter XII: complete-source Puyang fire-gate narrative and cross-act grain consequences.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage, content::Hero,
    lore::{beat, next_beat, newly_completed_eggs},
};

#[test]
fn puyang_is_optional_payoff_of_xuzhou_and_does_not_gate_older_stories() {
    let xuzhou = beat("xuzhou").unwrap();
    let puyang = beat("puyang").unwrap();
    let xudu = beat("xudu").unwrap();
    let guandu = beat("guandu").unwrap();
    assert_eq!((xuzhou.chapter, puyang.chapter, xudu.chapter, guandu.chapter),
        (10, 12, 14, 30));
    assert_eq!(puyang.prerequisite, Some("xuzhou_debt"));
    assert_eq!(xudu.prerequisite, None);
    assert_eq!(guandu.prerequisite, Some("xu_capital"));
    for detail in ["陈宫", "田氏", "吕布", "典韦", "荀彧", "蝗灾", "兖州", "徐州"] {
        assert!(puyang.premise.contains(detail), "missing chapter XII cause: {detail}");
    }
    let seen = vec!["sevenstar".into(), "xuzhou".into()];
    assert_eq!(next_beat(Hero::CaoCao, 0, &["xuzhou_debt".into()], &seen)
        .map(|beat| beat.id), Some("puyang"));
    assert_eq!(next_beat(Hero::CaoCao, 0, &[], &seen)
        .map(|beat| beat.id), Some("xudu"),
        "legacy campaigns must not need the optional Puyang flag");
    assert_ne!(next_beat(Hero::LuBu, 0, &["xuzhou_debt".into()], &[])
        .map(|beat| beat.id), Some("puyang"));
}

#[test]
fn puyang_player_choices_commit_exclusive_flags_and_roundtrip() {
    for (choice, keeps_supply) in [(1, true), (2, false), (3, false)] {
        let mut game = Game::new(Hero::CaoCao, 12012);
        game.stage = Stage::Story;
        game.lore_current = Some("puyang".into());
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: choice }).unwrap();
        assert_eq!(game.lore_flags.iter().any(|flag| flag == "puyang_supply"),
            keeps_supply);
        assert!(game.lore_seen.iter().any(|id| id == "puyang"));
        let json = game.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(), json);
    }
}

#[test]
fn grain_payoff_requires_four_independent_choices_and_is_idempotent() {
    let complete: Vec<String> = ["xuzhou_debt", "puyang_supply", "xu_capital",
        "guandu_grain"].into_iter().map(str::to_owned).collect();
    let awarded = newly_completed_eggs(Hero::CaoCao, &complete, &[]);
    assert!(awarded.iter().any(|egg| egg.id == "yan_base"));
    for lost in ["xuzhou_debt", "puyang_supply", "xu_capital", "guandu_grain"] {
        let incomplete: Vec<_> = complete.iter().filter(|flag| flag.as_str() != lost)
            .cloned().collect();
        assert!(!newly_completed_eggs(Hero::CaoCao, &incomplete, &[])
            .iter().any(|egg| egg.id == "yan_base"),
            "unearned grain reward without {lost}");
    }
    assert!(!newly_completed_eggs(Hero::CaoCao, &complete, &["yan_base".into()])
        .iter().any(|egg| egg.id == "yan_base"));
    assert!(!newly_completed_eggs(Hero::GuanYu, &complete, &[])
        .iter().any(|egg| egg.id == "yan_base"));
}
