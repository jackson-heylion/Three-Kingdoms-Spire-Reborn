//! Chapters 22–24: real, optional choices; source events and counterfactual player actions differ.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 22024);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|flag| (*flag).into()).collect();
    game
}
fn unlocked(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|flag| flag.as_str() == id).count()
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}

#[test]
fn pinned_chapters_are_distinct_and_new_beats_remain_optional() {
    let prisoner = beat("wangzhong_prisoners").unwrap();
    let household = beat("xiapi_household").unwrap();
    let doctor = beat("jiping_testimony").unwrap();
    let palace = beat("palace_ledger").unwrap();
    assert_eq!((prisoner.chapter, prisoner.act, prisoner.prerequisite),
        (22, 1, Some("wenjiu_oath")));
    assert_eq!((household.chapter, household.act, household.prerequisite),
        (24, 1, Some("wangzhong_release_record")));
    assert_eq!((doctor.chapter, doctor.act, doctor.prerequisite),
        (23, 1, Some("xu_capital")));
    assert_eq!((palace.chapter, palace.act, palace.prerequisite),
        (24, 1, Some("jiping_case_record")));
    assert!(prisoner.premise.contains("王忠") && prisoner.premise.contains("刘岱"));
    assert!(doctor.premise.contains("秦庆童") && doctor.premise.contains("吉平"));
    assert!(palace.premise.contains("董贵妃") && household.premise.contains("张辽"));
    assert_eq!(next_beat(Hero::GuanYu, 1, &["wenjiu_oath".into()],
        &["xutian_guard".into(), "xuzhou_gate".into()])
        .map(|b| b.id), Some("wangzhong_prisoners"));
    assert_eq!(next_beat(Hero::CaoCao, 1, &["xu_capital".into()],
        &["guandu".into()]).map(|b| b.id), Some("jiping_testimony"));
    assert_eq!(next_beat(Hero::GuanYu, 2, &["anxi_seal".into()], &[])
        .map(|b| b.id), Some("jingzhou"), "old saves cannot be forced through new events");
    assert_eq!(next_beat(Hero::CaoCao, 1, &["xu_capital".into()], &[])
        .map(|b| b.id), Some("jiping_testimony"));
    assert_eq!(beat("guandu").unwrap().prerequisite, Some("xu_capital"));
}

#[test]
fn prisoner_release_and_household_protection_pay_once_at_jingzhou() {
    let mut game = scene(Hero::GuanYu, "wangzhong_prisoners",
        &["anxi_seal", "wenjiu_oath"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), before, "rejected choice must be atomic");
    let deck = game.deck.len();
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "wangzhong_release_record"));
    assert_eq!(game.deck.len(), deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "hold_line");
    enter(&mut game, "xiapi_household");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "xiapi_household_ledger"));
    assert_eq!(unlocked(&game, "xiapi_household_debt"), 0);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "jingzhou");
    let report = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(report.contains("放俘守眷旧约"));
    assert_eq!(unlocked(&restored, "xiapi_household_debt"), 1);
    enter(&mut restored, "jingzhou");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&restored, "xiapi_household_debt"), 1);
}

#[test]
fn rejecting_the_prisoner_or_household_record_never_softlocks_old_story() {
    let mut first = scene(Hero::GuanYu, "wangzhong_prisoners",
        &["anxi_seal", "wenjiu_oath"]);
    first.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!first.lore_flags.iter().any(|f| f == "wangzhong_release_record"));
    assert_ne!(next_beat(Hero::GuanYu, 1, &first.lore_flags, &first.lore_seen)
        .map(|b| b.id), Some("xiapi_household"));
    assert_eq!(next_beat(Hero::GuanYu, 2, &first.lore_flags, &first.lore_seen)
        .map(|b| b.id), Some("jingzhou"));
    let mut second = scene(Hero::GuanYu, "xiapi_household",
        &["anxi_seal", "wenjiu_oath", "wangzhong_release_record"]);
    second.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!second.lore_flags.iter().any(|f| f == "xiapi_household_ledger"));
    enter(&mut second, "jingzhou");
    second.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&second, "xiapi_household_debt"), 0);
}

#[test]
fn jiping_evidence_and_palace_accounting_pay_once_after_guandu() {
    let mut game = scene(Hero::CaoCao, "jiping_testimony", &["xu_capital"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    let deck = game.deck.len();
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "jiping_case_record"));
    assert_eq!(game.deck.len(), deck + 1);
    assert_eq!(game.deck.last().unwrap().id, "qimou");
    enter(&mut game, "palace_ledger");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "palace_purge_record"));
    assert_eq!(unlocked(&game, "jiping_palace_debt"), 0);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "guandu");
    let report = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(report.contains("衣带诏与宫门旧账"));
    assert_eq!(unlocked(&restored, "jiping_palace_debt"), 1);
    enter(&mut restored, "guandu");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&restored, "jiping_palace_debt"), 1);
}

#[test]
fn refusing_the_jiping_or_palace_record_preserves_old_guandu() {
    let mut first = scene(Hero::CaoCao, "jiping_testimony", &["xu_capital"]);
    first.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!first.lore_flags.iter().any(|f| f == "jiping_case_record"));
    assert_ne!(next_beat(Hero::CaoCao, 1, &first.lore_flags, &first.lore_seen)
        .map(|b| b.id), Some("palace_ledger"));
    assert_eq!(next_beat(Hero::CaoCao, 1, &first.lore_flags, &first.lore_seen)
        .map(|b| b.id), Some("guandu"));
    let mut second = scene(Hero::CaoCao, "palace_ledger",
        &["xu_capital", "jiping_case_record"]);
    second.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!second.lore_flags.iter().any(|f| f == "palace_purge_record"));
    enter(&mut second, "guandu");
    second.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&second, "jiping_palace_debt"), 0);
}
