//! Chapter IV: pinned textual provenance is distinct from optional player fiction.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage, content::Hero, lore::{beat, next_beat},
};

#[test]
fn chapter_four_is_an_optional_cao_choice_before_the_court_and_guandu_arcs() {
    let first = next_beat(Hero::CaoCao, 0, &[], &[]).unwrap();
    assert_eq!(first.id, "sevenstar");
    assert_eq!(first.chapter, 4);
    assert_eq!(first.flag, "sevenstar_vow");
    assert_eq!(first.prerequisite, None);
    assert_eq!(beat(first.id).unwrap().reward_card, "jianxiong");

    // Skipping the early choice must NOT prevent the later Cao storyline.
    let skipped = vec!["sevenstar".to_owned()];
    let second = next_beat(Hero::CaoCao, 0, &[], &skipped).unwrap();
    assert_eq!(second.id, "xudu");
    let flags = vec!["xu_capital".to_owned()];
    assert_eq!(next_beat(Hero::CaoCao, 1, &flags, &[]).unwrap().id, "guandu");

    assert_ne!(next_beat(Hero::LuBu, 0, &[], &[]).unwrap().id, "sevenstar");
}

#[test]
fn early_accountability_only_grants_a_bonus_after_guandu_payoff() {
    // Explicit narrative-state fixtures: the real acquisition path is choose 1
    // in a story encounter; this test isolates the opening combat calculation.
    fn opening_command(flags: &[&str]) -> (i32, i32) {
        let mut g = Game::new(Hero::CaoCao, 951);
        g.lore_flags = flags.iter().map(|s| (*s).to_owned()).collect();
        g.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(g.stage, Stage::Combat);
        let b = g.battle.as_ref().unwrap();
        let result = (b.command, b.block);
        let json = g.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(), json);
        result
    }

    let without_vow = opening_command(&["xu_capital", "guandu_grain"]);
    let with_vow = opening_command(&["sevenstar_vow", "xu_capital", "guandu_grain"]);
    assert_eq!(with_vow.0, without_vow.0 + 1);
    assert_eq!(with_vow.1, without_vow.1);
    let early_alone = opening_command(&["sevenstar_vow"]);
    let baseline = opening_command(&[]);
    assert_eq!(early_alone, baseline, "early choice alone cannot frontload a reward");
}
