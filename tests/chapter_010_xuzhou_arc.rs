//! Cao Cao's chapter X debt is an optional, source-grounded player branch,
//! not a claim that the novel historically lets him avoid the civilian toll.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage,
    content::Hero,
    lore::{beat, next_beat, newly_completed_eggs},
};

#[test]
fn xuzhou_story_is_gated_by_chapter_four_but_never_blocks_old_xudu_or_guandu() {
    let old = beat("sevenstar").unwrap();
    let debt = beat("xuzhou").unwrap();
    let capital = beat("xudu").unwrap();
    let guandu = beat("guandu").unwrap();
    assert_eq!((old.chapter, debt.chapter, capital.chapter, guandu.chapter),
        (4, 10, 14, 30));
    assert_eq!(debt.prerequisite, Some("sevenstar_vow"));
    assert_eq!(capital.prerequisite, None,
        "existing chapter XIV is not gated behind the new story");
    assert_eq!(guandu.prerequisite, Some("xu_capital"),
        "existing cross-act path must remain playable from older saves");
    for detail in ["张闓", "陶谦", "陈宫", "曹嵩", "青州兵", "荀彧", "徐州"] {
        assert!(debt.premise.contains(detail), "missing chapter X cause: {detail}");
    }
    for detail in ["徐州", "军纪"] {
        assert!(capital.premise.contains(detail));
    }
    let seen = ["sevenstar".to_string()].to_vec();
    assert_ne!(next_beat(Hero::CaoCao, 0, &[], &seen).map(|b| b.id),
        Some("xuzhou"));
    assert_eq!(next_beat(Hero::CaoCao, 0, &["sevenstar_vow".into()], &seen)
        .map(|b| b.id), Some("xuzhou"));
    assert_eq!(next_beat(Hero::CaoCao, 0, &[], &[])
        .map(|b| b.id), Some("sevenstar"));
    assert_eq!(next_beat(Hero::CaoCao, 0, &[], &seen)
        .map(|b| b.id), Some("xudu"));
}

#[test]
fn chapter_ten_player_choice_is_mutually_exclusive_atomically_saved() {
    for (choice, flagged) in [(1, true), (2, false), (3, false)] {
        let mut g = Game::new(Hero::CaoCao, 9010);
        g.stage = Stage::Story;
        g.lore_current = Some("xuzhou".into());
        let before = g.to_json().unwrap();
        assert!(g.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(g.to_json().unwrap(), before);
        g.apply(Action::Choose { index: choice }).unwrap();
        assert_eq!(g.lore_flags.iter().any(|f| f == "xuzhou_debt"), flagged);
        assert!(g.lore_seen.iter().any(|id| id == "xuzhou"));
        let json = g.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(), json);
    }
}

#[test]
fn cross_act_civilian_ledger_is_not_awarded_for_money_choices_or_missing_link() {
    let complete: Vec<String> = ["sevenstar_vow","xuzhou_debt",
        "xu_capital","guandu_grain"].into_iter().map(str::to_owned).collect();
    let reward = newly_completed_eggs(Hero::CaoCao, &complete, &[]);
    assert!(reward.iter().any(|egg| egg.id == "xuzhou_ledger"));
    for lost in ["sevenstar_vow", "xuzhou_debt", "xu_capital", "guandu_grain"] {
        let incomplete = complete.iter().filter(|flag| flag.as_str()!=lost)
            .cloned().collect::<Vec<_>>();
        assert!(!newly_completed_eggs(Hero::CaoCao, &incomplete, &[])
            .iter().any(|egg| egg.id == "xuzhou_ledger"),
            "unearned reward missing prerequisite {lost}");
    }
    assert!(!newly_completed_eggs(Hero::CaoCao, &complete,
        &["xuzhou_ledger".into()]).iter().any(|egg| egg.id == "xuzhou_ledger"));
    assert!(!newly_completed_eggs(Hero::GuanYu, &complete, &[])
        .iter().any(|egg| egg.id == "xuzhou_ledger"));
}
