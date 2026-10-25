//! Chapter XIII: the previously read Xiaopei guest-host debt becomes playable.
//! A hypothetical seizure of Xuzhou must not masquerade as novel canon.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage, content::Hero,
    lore::{beat, next_beat, newly_completed_eggs},
};

#[test]
fn xiaopei_requires_changan_but_never_gates_the_old_halberd_or_whitegate() {
    let changan = beat("changan").unwrap();
    let xiaopei = beat("xiaopei").unwrap();
    let halberd = beat("halberd").unwrap();
    let whitegate = beat("whitegate").unwrap();
    assert_eq!((changan.chapter, xiaopei.chapter, halberd.chapter, whitegate.chapter),
        (9, 13, 16, 19));
    assert_eq!(xiaopei.hero, Some(Hero::LuBu));
    assert_eq!(xiaopei.prerequisite, Some("changan_account"));
    assert_eq!(halberd.prerequisite, Some("phoenix_pavilion"));
    assert_eq!(whitegate.prerequisite, Some("halberd_shot"));
    for detail in ["定陶", "袁绍", "陈宫", "刘备", "关张", "张飞", "州印", "小沛", "军粮"] {
        assert!(xiaopei.premise.contains(detail), "missing chapter XIII cause: {detail}");
    }
    let seen = vec!["redhare".into(), "pavilion".into(), "changan".into()];
    let flags = vec!["red_hare".into(), "phoenix_pavilion".into(), "changan_account".into()];
    assert_eq!(next_beat(Hero::LuBu, 0, &flags, &seen).map(|b| b.id),
        Some("xiaopei"), "completed Changan debt must make Xiaopei selectable");
    assert_ne!(next_beat(Hero::LuBu, 0, &[], &seen).map(|b| b.id),
        Some("xiaopei"), "no unearned refuge without the Changan choice");
    assert_ne!(next_beat(Hero::CaoCao, 0, &flags, &[]).map(|b| b.id),
        Some("xiaopei"), "another hero must not claim Lu Bu's guest-host choice");
}

#[test]
fn xiaopei_choices_are_mutually_exclusive_atomic_and_saveable() {
    for (choice, preserves_guest_oath) in [(1, true), (2, false), (3, false)] {
        let mut game = Game::new(Hero::LuBu, 13013);
        game.stage = Stage::Story;
        game.lore_current = Some("xiaopei".into());
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before,
            "invalid choice must not advance RNG, journals, or game state");
        game.apply(Action::Choose { index: choice }).unwrap();
        assert_eq!(game.lore_flags.iter().any(|flag| flag == "xiaopei_refuge"),
            preserves_guest_oath);
        assert!(game.lore_seen.iter().any(|id| id == "xiaopei"));
        let saved = game.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
    }
}

#[test]
fn xiaopei_hidden_payoff_needs_four_distinct_choices_and_awards_once() {
    let complete: Vec<String> = [
        "changan_account", "xiaopei_refuge", "halberd_shot", "white_gate",
    ].into_iter().map(str::to_owned).collect();
    assert!(newly_completed_eggs(Hero::LuBu, &complete, &[])
        .iter().any(|egg| egg.id == "xiaopei_guest_oath"));
    for absent in &complete {
        let incomplete: Vec<String> = complete.iter()
            .filter(|flag| *flag != absent).cloned().collect();
        assert!(!newly_completed_eggs(Hero::LuBu, &incomplete, &[])
            .iter().any(|egg| egg.id == "xiaopei_guest_oath"),
            "must not award without {absent}");
    }
    assert!(!newly_completed_eggs(Hero::LuBu, &complete,
        &["xiaopei_guest_oath".into()]).iter()
        .any(|egg| egg.id == "xiaopei_guest_oath"));
    assert!(!newly_completed_eggs(Hero::GuanYu, &complete, &[])
        .iter().any(|egg| egg.id == "xiaopei_guest_oath"));
}
