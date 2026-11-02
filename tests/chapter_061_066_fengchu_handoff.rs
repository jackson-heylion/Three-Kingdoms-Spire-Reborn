//! Chapter 057 handoff is a gameplay bridge toward 060–066, not proof of map reachability.
//! These integration tests inject Story; the real CLI seed proof remains separate.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter_fengchu(game: &mut Game) {
    game.stage = Stage::Story;
    game.lore_current = Some("fengchu".into());
    game.lore_flags.push("redcliff_wind".into());
}

fn count(items: &[String], id: &str) -> usize {
    items.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn accepted_fengchu_letter_opens_distinct_leiyang_choice_without_extra_map_visit() {
    let mut game = Game::new(Hero::ZhugeLiang, 61066);
    enter_fengchu(&mut game);
    game.apply(Action::Choose { index: 1 }).unwrap();

    assert_eq!(game.stage, Stage::Story);
    assert_eq!(game.lore_current.as_deref(), Some("leiyang_review"));
    assert_eq!(count(&game.lore_flags, "phoenix_letter"), 1);
    assert_eq!(count(&game.lore_seen, "fengchu"), 1);
    assert_eq!(count(&game.lore_seen, "leiyang_review"), 0);

    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    assert!(restored.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(restored.to_json().unwrap(), saved);

    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(restored.stage, Stage::Map);
    assert_eq!(restored.lore_current, None);
    assert_eq!(count(&restored.lore_flags, "leiyang_case_record"), 1);
    assert_eq!(count(&restored.lore_seen, "leiyang_review"), 1);
    assert!(Game::from_json(&restored.to_json().unwrap()).is_ok());
}

#[test]
fn refusing_letter_or_existing_v2_leiyang_history_never_forces_followup() {
    let mut refused = Game::new(Hero::ZhugeLiang, 61067);
    enter_fengchu(&mut refused);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(refused.stage, Stage::Map);
    assert!(refused.lore_current.is_none());
    assert_eq!(count(&refused.lore_flags, "phoenix_letter"), 0);
    assert_eq!(count(&refused.lore_flags, "leiyang_case_record"), 0);

    let mut old_save = Game::new(Hero::ZhugeLiang, 61068);
    enter_fengchu(&mut old_save);
    old_save.lore_seen.push("leiyang_review".into());
    let json = old_save.to_json().unwrap();
    let mut restored = Game::from_json(&json).unwrap();
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(restored.stage, Stage::Map);
    assert!(restored.lore_current.is_none());
    assert_eq!(count(&restored.lore_seen, "leiyang_review"), 1);
}
