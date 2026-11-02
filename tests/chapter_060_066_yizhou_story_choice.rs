//! Chapter 060-066: opt-in Yizhou responsibility at an actual Story node.
//! These state-level tests prove atomicity and compatibility, NOT natural reachability.
//! Real CLI evidence lives in tests/lore_natural_cli_e2e.py.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn at_story(hero: Hero, id: &str, leiyang: bool) -> Game {
    let mut game = Game::new(hero, 61068);
    game.stage = Stage::Story;
    game.floor = 15;
    game.lore_current = Some(id.into());
    if leiyang {
        game.lore_flags.push("leiyang_case_record".into());
    }
    game
}
fn contains(items: &[String], id: &str) -> bool {
    items.iter().any(|item| item == id)
}

#[test]
fn alternative_requires_a_real_third_act_scene_and_leiyang_responsibility() {
    for (hero, scene, leiyang) in [
        (Hero::ZhugeLiang, "jiangwei", false),
        (Hero::CaoCao, "jiangwei", true),
        (Hero::ZhugeLiang, "fengchu", true),
        (Hero::ZhugeLiang, "yizhou_map_audit", true),
    ] {
        let mut game = at_story(hero, scene, leiyang);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
    }
    let mut seen = at_story(Hero::ZhugeLiang, "jiangwei", true);
    seen.lore_seen.push("yizhou_map_audit".into());
    let before = seen.to_json().unwrap();
    assert!(seen.apply(Action::Choose { index: 4 }).is_err());
    assert_eq!(seen.to_json().unwrap(), before);
}

#[test]
fn switching_is_explicit_reward_free_and_v2_persistent_until_decision() {
    let mut game = at_story(Hero::ZhugeLiang, "jiangwei", true);
    assert!(game.lore_hint().contains("4. 改查西川图"));
    let gold = game.gold;
    let hp = game.hp;
    let deck = game.deck.clone();
    let flags = game.lore_flags.clone();
    let seen = game.lore_seen.clone();
    game.apply(Action::Choose { index: 4 }).unwrap();
    assert_eq!(game.stage, Stage::Story);
    assert_eq!(game.lore_current.as_deref(), Some("yizhou_map_audit"));
    assert_eq!(game.gold, gold);
    assert_eq!(game.hp, hp);
    assert_eq!(game.deck, deck);
    assert_eq!(game.lore_flags, flags);
    assert_eq!(game.lore_seen, seen);
    let saved = game.to_json().unwrap();
    let mut resumed = Game::from_json(&saved).unwrap();
    assert_eq!(resumed.to_json().unwrap(), saved);
    let before = resumed.to_json().unwrap();
    assert!(resumed.apply(Action::Choose { index: 4 }).is_err());
    assert_eq!(resumed.to_json().unwrap(), before);
    resumed.apply(Action::Choose { index: 1 }).unwrap();
    assert!(contains(&resumed.lore_flags, "yizhou_map_record"));
    assert!(contains(&resumed.lore_seen, "yizhou_map_audit"));
    assert!(!contains(&resumed.lore_seen, "jiangwei"));
    assert_eq!(resumed.stage, Stage::Map);
}

#[test]
fn declining_archive_does_not_forge_a_flag_and_legacy_default_still_works() {
    let mut refusal = at_story(Hero::ZhugeLiang, "jiangwei", true);
    refusal.apply(Action::Choose { index: 4 }).unwrap();
    refusal.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!contains(&refusal.lore_flags, "yizhou_map_record"));
    assert!(contains(&refusal.lore_seen, "yizhou_map_audit"));
    assert!(!contains(&refusal.lore_seen, "jiangwei"));

    // An existing V2 scene that never opts in keeps its old Jiang Wei path.
    let mut legacy = at_story(Hero::ZhugeLiang, "jiangwei", true);
    let saved = legacy.to_json().unwrap();
    legacy = Game::from_json(&saved).unwrap();
    legacy.apply(Action::Choose { index: 1 }).unwrap();
    assert!(contains(&legacy.lore_seen, "jiangwei"));
    assert!(!contains(&legacy.lore_seen, "yizhou_map_audit"));
    assert!(!contains(&legacy.lore_flags, "yizhou_map_record"));
}
