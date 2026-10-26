//! Fixed single-base provenance and optional 033–035 cross-act responsibility choices.
//! Historical/novel outcomes are immutable; the player's choices affect only game ledgers.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 33035);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn has_flag(game: &Game, flag: &str) -> bool {
    game.lore_flags.iter().any(|f| f == flag)
}
fn awarded(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|f| f.as_str() == id).count()
}

#[test]
fn chapters_have_distinct_fixed_sources_and_optional_prerequisites() {
    let northern = beat("northern_settlement").unwrap();
    let tanxi = beat("tanxi_warning").unwrap();
    let xinye = beat("xinye_counsel").unwrap();
    assert_eq!((northern.chapter, northern.act, northern.prerequisite), (33, 2, Some("liaodong_letter")));
    assert_eq!((tanxi.chapter, tanxi.act, tanxi.prerequisite), (34, 1, Some("ancient_reunion")));
    assert_eq!((xinye.chapter, xinye.act, xinye.prerequisite), (35, 2, Some("tanxi_warning_record")));
    assert!(northern.premise.contains("公孙康") && northern.premise.contains("崔琰"));
    assert!(tanxi.premise.contains("伊籍") && tanxi.premise.contains("赵云"));
    assert!(xinye.premise.contains("单福") && xinye.premise.contains("李典"));
    for (document, oldid) in [
        (include_str!("../docs/CHAPTER_033_READING.md"), "2567995"),
        (include_str!("../docs/CHAPTER_034_READING.md"), "2567997"),
        (include_str!("../docs/CHAPTER_035_READING.md"), "2568001"),
    ] {
        assert!(document.contains(oldid));
        assert!(document.contains("单底本"));
    }
    assert_eq!(beat("liaodong").unwrap().prerequisite, Some("guandu_grain"));
    assert_eq!(beat("jingzhou").unwrap().prerequisite, Some("anxi_seal"));
    assert_eq!(next_beat(Hero::GuanYu, 2, &["anxi_seal".into()], &[]).map(|b| b.id), Some("jingzhou"));
}

#[test]
fn northern_aftermath_choice_is_atomic_persistent_and_awarded_once() {
    let mut game = scene(Hero::CaoCao, "northern_settlement", &["ye_flood_record", "liaodong_letter"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    let output = game.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(has_flag(&game, "northern_settlement_account"));
    assert!(output.contains("南皮民伤"));
    assert_eq!(awarded(&game, "northern_aftermath_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut game = Game::from_json(&saved).unwrap();
    assert_eq!(game.to_json().unwrap(), saved);
    enter(&mut game, "northern_settlement");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awarded(&game, "northern_aftermath_debt"), 1);
}

#[test]
fn tanxi_to_xinye_to_jingzhou_requires_every_responsibility_choice() {
    let mut game = scene(Hero::GuanYu, "tanxi_warning", &["ancient_reunion", "anxi_seal"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has_flag(&game, "tanxi_warning_record"));
    let saved = game.to_json().unwrap();
    let mut game = Game::from_json(&saved).unwrap();
    enter(&mut game, "xinye_counsel");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has_flag(&game, "xinye_counsel_record"));
    assert_eq!(awarded(&game, "jingxiang_warning_debt"), 0);
    enter(&mut game, "jingzhou");
    let output = game.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(output.contains("檀溪脱险"));
    assert_eq!(awarded(&game, "jingxiang_warning_debt"), 1);
    enter(&mut game, "jingzhou");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awarded(&game, "jingxiang_warning_debt"), 1);
}

#[test]
fn refusal_never_forces_old_saves_to_complete_new_chapters() {
    for (hero, id, flags, omitted, legacy, legacy_flag) in [
        (Hero::CaoCao, "northern_settlement", &["liaodong_letter", "ye_flood_record"][..], "northern_settlement_account", "liaodong", "liaodong_letter"),
        (Hero::GuanYu, "tanxi_warning", &["ancient_reunion", "anxi_seal"][..], "tanxi_warning_record", "jingzhou", "jingzhou_warning"),
        (Hero::GuanYu, "xinye_counsel", &["tanxi_warning_record", "anxi_seal"][..], "xinye_counsel_record", "jingzhou", "jingzhou_warning"),
    ] {
        let mut game = scene(hero, id, flags);
        game.apply(Action::Choose { index: 2 }).unwrap();
        assert!(!has_flag(&game, omitted));
        enter(&mut game, legacy);
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert!(has_flag(&game, legacy_flag));
        assert_eq!(awarded(&game, if hero == Hero::CaoCao { "northern_aftermath_debt" } else { "jingxiang_warning_debt" }), 0);
    }
    let mut old_cao = scene(Hero::CaoCao, "liaodong", &["guandu_grain"]);
    old_cao.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has_flag(&old_cao, "liaodong_letter"));
    let mut old_guan = scene(Hero::GuanYu, "jingzhou", &["anxi_seal"]);
    old_guan.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has_flag(&old_guan, "jingzhou_warning"));
}
