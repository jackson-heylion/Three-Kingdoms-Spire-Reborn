//! Chapters 030–032: pinned provenance and optional Cao Cao responsibility ledger.
//! Choices do not change the novel; old Guandu/Liaodong routes remain available.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn scene(id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(Hero::CaoCao, 30032);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|f| (*f).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn flag(game: &Game, id: &str) -> bool {
    game.lore_flags.iter().any(|f| f == id)
}
fn awards(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|f| f.as_str() == id).count()
}

#[test]
fn guandu_cangting_and_ye_have_distinct_pinned_chapters_and_prerequisites() {
    let guandu = beat("guandu_letters").unwrap();
    let cangting = beat("cangting_memorial").unwrap();
    let ye = beat("ye_flood_ledger").unwrap();
    assert_eq!((guandu.chapter, guandu.act, guandu.prerequisite),
        (30, 1, Some("guandu_grain")));
    assert_eq!((cangting.chapter, cangting.act, cangting.prerequisite),
        (31, 1, Some("guandu_captured_letters")));
    assert_eq!((ye.chapter, ye.act, ye.prerequisite),
        (32, 2, Some("cangting_memorial_record")));
    assert!(guandu.premise.contains("乌巢") && guandu.premise.contains("沮授"));
    assert!(cangting.premise.contains("田丰") && cangting.premise.contains("汝南"));
    assert!(ye.premise.contains("冯礼") && ye.premise.contains("审荣"));
    assert_eq!(beat("guandu").unwrap().prerequisite, Some("xu_capital"));
    assert_eq!(beat("liaodong").unwrap().prerequisite, Some("guandu_grain"));
    assert_eq!(next_beat(Hero::CaoCao, 1, &["guandu_grain".into()], &[])
        .map(|b| b.id), Some("guandu_letters"));
    assert_eq!(next_beat(Hero::CaoCao, 1,
        &["guandu_grain".into(), "guandu_captured_letters".into()],
        &["guandu_letters".into()]).map(|b| b.id), Some("cangting_memorial"));
    assert_eq!(next_beat(Hero::CaoCao, 2,
        &["guandu_grain".into(), "cangting_memorial_record".into()], &[])
        .map(|b| b.id), Some("ye_flood_ledger"));
}

#[test]
fn accepted_records_survive_save_and_pay_once_after_liaodong() {
    let mut game = scene("guandu_letters", &["guandu_grain"]);
    let original = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), original, "invalid choice must be atomic");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(flag(&game, "guandu_captured_letters"));
    enter(&mut game, "cangting_memorial");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(flag(&game, "cangting_memorial_record"));
    let snapshot = game.to_json().unwrap();
    let mut game = Game::from_json(&snapshot).unwrap();
    assert_eq!(game.to_json().unwrap(), snapshot);
    enter(&mut game, "ye_flood_ledger");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(flag(&game, "ye_flood_record"));
    assert_eq!(awards(&game, "guandu_ye_civilians"), 0);
    enter(&mut game, "liaodong");
    let result = game.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(result.contains("官渡焚书"), "cross-act reward must be visible");
    assert_eq!(awards(&game, "guandu_ye_civilians"), 1);
    enter(&mut game, "liaodong");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&game, "guandu_ye_civilians"), 1);
}

#[test]
fn refusing_any_new_record_never_locks_legacy_liaodong() {
    for refused in ["guandu_letters", "cangting_memorial", "ye_flood_ledger"] {
        let prerequisites: &[&str] = match refused {
            "guandu_letters" => &["guandu_grain"],
            "cangting_memorial" => &["guandu_grain", "guandu_captured_letters"],
            _ => &["guandu_grain", "guandu_captured_letters", "cangting_memorial_record"],
        };
        let mut game = scene(refused, prerequisites);
        let original = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 0 }).is_err());
        assert_eq!(game.to_json().unwrap(), original);
        game.apply(Action::Choose { index: 2 }).unwrap();
        let omitted = beat(refused).unwrap().flag;
        assert!(!flag(&game, omitted));
        enter(&mut game, "liaodong");
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert!(flag(&game, "liaodong_letter"));
        assert_eq!(awards(&game, "guandu_ye_civilians"), 0);
    }
    let mut legacy = scene("liaodong", &["guandu_grain"]);
    legacy.apply(Action::Choose { index: 1 }).unwrap();
    assert!(flag(&legacy, "liaodong_letter"));
    assert_eq!(awards(&legacy, "guandu_ye_civilians"), 0);
    assert_ne!(next_beat(Hero::GuanYu, 1, &["guandu_grain".into()], &[])
        .map(|b| b.id), Some("guandu_letters"));
}
