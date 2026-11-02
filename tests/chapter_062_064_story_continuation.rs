//! Chapter 062–064 responsibility: opt-in, independently chosen archive chain.
//! These state-level tests do NOT establish natural reachability; the CLI test does.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn archive(hero: Hero, scene: &str, prerequisite: bool) -> Game {
    let mut game = Game::new(hero, 61068);
    game.floor = 15;
    game.stage = Stage::Story;
    game.lore_current = Some(scene.into());
    if prerequisite { game.lore_flags.push("leiyang_case_record".into()); }
    game
}
fn choose(game: &mut Game, index: usize) {
    game.apply(Action::Choose { index }).unwrap();
}
fn has(items: &[String], value: &str) -> bool {
    items.iter().any(|entry| entry == value)
}

#[test]
fn explicit_continuation_keeps_each_chapter_decision_and_v2_save() {
    let mut game = archive(Hero::ZhugeLiang, "yizhou_map_audit", true);
    let chain = [
        ("yizhou_map_audit", "yizhou_map_record", "fu_pass_review"),
        ("fu_pass_review", "fu_pass_record", "luofeng_mourning"),
        ("luofeng_mourning", "luofeng_mourning_record", "jinyan_bridge_review"),
    ];
    for (current, flag, next) in chain {
        assert_eq!(game.lore_current.as_deref(), Some(current));
        assert!(game.lore_hint().contains("4. 保留本卷责任并接续核审"));
        assert!(!has(&game.lore_flags, flag));
        choose(&mut game, 4);
        assert_eq!(game.stage, Stage::Story);
        assert!(has(&game.lore_flags, flag));
        assert!(has(&game.lore_seen, current));
        assert_eq!(game.lore_current.as_deref(), Some(next));
        let snapshot = game.to_json().unwrap();
        game = Game::from_json(&snapshot).unwrap();
        assert_eq!(game.to_json().unwrap(), snapshot);
    }
    assert!(!has(&game.lore_flags, "jinyan_bridge_record"));
    choose(&mut game, 1);
    assert_eq!(game.stage, Stage::Map);
    assert!(has(&game.lore_flags, "jinyan_bridge_record"));
    assert!(!has(&game.lore_flags, "chengdu_land_record"));
    assert_eq!(game.lore_seen, vec![
        "yizhou_map_audit", "fu_pass_review",
        "luofeng_mourning", "jinyan_bridge_review",
    ]);
}

#[test]
fn default_and_refusal_routes_still_exit_without_forged_payoffs() {
    let mut legacy = archive(Hero::ZhugeLiang, "yizhou_map_audit", true);
    choose(&mut legacy, 1);
    assert_eq!(legacy.stage, Stage::Map);
    assert!(has(&legacy.lore_flags, "yizhou_map_record"));
    assert!(!has(&legacy.lore_flags, "fu_pass_record"));

    let mut refusal = archive(Hero::ZhugeLiang, "yizhou_map_audit", true);
    choose(&mut refusal, 4);
    assert_eq!(refusal.lore_current.as_deref(), Some("fu_pass_review"));
    choose(&mut refusal, 2);
    assert_eq!(refusal.stage, Stage::Map);
    assert!(!has(&refusal.lore_flags, "fu_pass_record"));
    assert!(!has(&refusal.lore_flags, "luofeng_mourning_record"));
    let saved = refusal.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}

#[test]
fn unauthorized_continuation_is_rejected_without_state_or_reward_changes() {
    for (hero, scene, prerequisite) in [
        (Hero::CaoCao, "yizhou_map_audit", true),
        (Hero::ZhugeLiang, "yizhou_map_audit", false),
        (Hero::ZhugeLiang, "jiangwei", false),
        (Hero::ZhugeLiang, "jinyan_bridge_review", true),
    ] {
        let mut game = archive(hero, scene, prerequisite);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
    }
    let mut seen = archive(Hero::ZhugeLiang, "yizhou_map_audit", true);
    seen.lore_seen.push("fu_pass_review".into());
    let before = seen.to_json().unwrap();
    assert!(seen.apply(Action::Choose { index: 4 }).is_err());
    assert_eq!(seen.to_json().unwrap(), before);
}
