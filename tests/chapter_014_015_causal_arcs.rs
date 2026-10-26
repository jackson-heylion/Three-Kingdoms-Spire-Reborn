//! Chapters XIV–XV: preserve original causality, but let player choices
//! carry optional, cross-act consequences. This is not historical collation.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage,
    content::Hero,
    lore::{beat, next_beat, newly_completed_eggs},
};

#[test]
fn xudu_remains_accessible_without_new_flags_and_xuhuang_is_an_optional_payoff() {
    let xudu = beat("xudu").unwrap();
    let xuhuang = beat("xuhuang").unwrap();
    assert_eq!((xudu.chapter, xuhuang.chapter), (14, 14));
    assert_eq!(xudu.prerequisite, None, "legacy Xudu path must remain open");
    assert_eq!(xuhuang.prerequisite, Some("xu_capital"));
    assert_eq!(xuhuang.hero, Some(Hero::CaoCao));
    for detail in ["洛阳", "杨彪", "荀彧", "董昭", "天子", "粮运"] {
        assert!(xudu.premise.contains(detail), "missing Xudu cause: {detail}");
    }
    for detail in ["杨奉", "韩暹", "徐晃", "许褚", "满宠", "拒绝杀杨奉"] {
        assert!(xuhuang.premise.contains(detail), "missing Xu Huang cause: {detail}");
    }
    let seen = vec!["sevenstar".into(), "xudu".into()];
    assert_eq!(next_beat(Hero::CaoCao, 0, &["xu_capital".into()], &seen)
        .map(|b| b.id), Some("xuhuang"));
    assert_ne!(next_beat(Hero::CaoCao, 0, &[], &seen)
        .map(|b| b.id), Some("xuhuang"));
    assert_ne!(next_beat(Hero::LuBu, 0, &["xu_capital".into()], &[])
        .map(|b| b.id), Some("xuhuang"));
}

#[test]
fn shenting_preserves_the_pawned_seal_and_taishi_cis_return() {
    let old = beat("sun_father_letter").unwrap();
    let shenting = beat("shenting").unwrap();
    assert_eq!((old.chapter, shenting.chapter), (7, 15));
    assert_eq!(shenting.hero, Some(Hero::SunShangxiang));
    assert_eq!(shenting.prerequisite, Some("sun_father_dispatch"));
    for detail in ["孙策", "玉玺", "袁术", "周瑜", "太史慈", "神亭", "质借", "按约归来"] {
        assert!(shenting.premise.contains(detail), "missing Shenting cause: {detail}");
    }
    assert_eq!(next_beat(Hero::SunShangxiang, 0,
        &["sun_father_dispatch".into()], &["sun_father_letter".into()])
        .map(|b| b.id), Some("shenting"));
    assert_ne!(next_beat(Hero::SunShangxiang, 0, &[], &["sun_father_letter".into()])
        .map(|b| b.id), Some("shenting"));
}

#[test]
fn chapter_choices_are_exclusive_atomic_and_v2_save_compatible() {
    for (hero, id, flag, seed) in [
        (Hero::CaoCao, "xuhuang", "xuhuang_honor", 14014_u64),
        (Hero::SunShangxiang, "shenting", "taishi_trust", 15015_u64),
    ] {
        for (choice, keeps_vow) in [(1, true), (2, false), (3, false)] {
            let mut game = Game::new(hero, seed);
            game.stage = Stage::Story;
            game.lore_current = Some(id.into());
            let before = game.to_json().unwrap();
            assert!(game.apply(Action::Choose { index: 4 }).is_err());
            assert_eq!(game.to_json().unwrap(), before,
                "invalid choice must not advance RNG, revision or authority journals");
            game.apply(Action::Choose { index: choice }).unwrap();
            assert_eq!(game.lore_flags.iter().any(|known| known == flag), keeps_vow,
                "{id}: choice {choice}");
            assert!(game.lore_seen.iter().any(|known| known == id));
            let saved = game.to_json().unwrap();
            assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
        }
    }
}

#[test]
fn hidden_rewards_need_all_flags_are_hero_scoped_and_award_only_once() {
    for (hero, egg_id, flags) in [
        (Hero::CaoCao, "xuhuang_oath",
            ["xuhuang_honor", "guandu_grain", "liaodong_letter"]),
        (Hero::SunShangxiang, "taishi_return",
            ["sun_father_dispatch", "taishi_trust", "sun_yiling_relief"]),
    ] {
        let complete: Vec<String> = flags.iter().map(|s| (*s).into()).collect();
        assert!(newly_completed_eggs(hero, &complete, &[])
            .iter().any(|egg| egg.id == egg_id));
        for omitted in flags {
            let incomplete: Vec<_> = complete.iter()
                .filter(|flag| flag.as_str() != omitted).cloned().collect();
            assert!(!newly_completed_eggs(hero, &incomplete, &[])
                .iter().any(|egg| egg.id == egg_id), "unearned reward without {omitted}");
        }
        assert!(!newly_completed_eggs(hero, &complete, &[egg_id.into()])
            .iter().any(|egg| egg.id == egg_id));
        let other = if hero == Hero::CaoCao { Hero::LuBu } else { Hero::GuanYu };
        assert!(!newly_completed_eggs(other, &complete, &[])
            .iter().any(|egg| egg.id == egg_id));
    }
}
