//! Fixed-source spotchecks plus executable early-arc settlement.
use three_kingdoms_spire_reborn::{Action, Game, Node, Stage, content::Hero, lore};

fn scene(hero: Hero, id: &str, seed: u64) -> Game {
    let mut game = Game::new(hero, seed);
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
    game
}

#[test]
fn fixed_sources_keep_two_novels_and_history_separate() {
    let evidence = include_str!("../docs/CHAPTERS_001_003_VARIANT_HISTORY_SPOTCHECK.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2583915","8185068","2567965","3207988","2554643",
        "2554667","2554699","2554663","2554697","2583378","2583383","2452141"] {
        assert!(evidence.contains(oldid), "missing pinned source {oldid}");
    }
    for chapter in ["001","002","003"] {
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter}\\t")))
            .expect("chapter ledger row");
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("局部异文"));
        assert!(row.contains("未完成全回异文校勘"));
    }
    let taoyuan = lore::beat("taoyuan").unwrap();
    let anxi = lore::beat("anxi").unwrap();
    let redhare = lore::beat("redhare").unwrap();
    assert!(taoyuan.premise.contains("嘉靖本") && taoyuan.premise.contains("恩若兄弟"));
    assert!(anxi.premise.contains("刘备本人") && anxi.premise.contains("张飞"));
    assert!(redhare.premise.contains("不证赠马") && redhare.premise.contains("丁原"));
    assert_eq!(lore::next_beat(Hero::GuanYu, 0, &[], &[]).unwrap().id, "taoyuan");
    assert_eq!(lore::next_beat(Hero::LuBu, 0, &[], &[]).unwrap().id, "redhare");
}

#[test]
fn guan_acceptance_pays_once_and_v2_restore_keeps_source_debt() {
    let mut game = scene(Hero::GuanYu, "taoyuan", 1001);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "taoyuan_oath"));
    game.stage = Stage::Story;
    game.lore_current = Some("anxi".into());
    let gold = game.gold;
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.gold, gold + 25 + 13);
    assert!(game.lore_flags.iter().any(|f| f == "anxi_seal"));
    assert_eq!(game.easter_unlocked.iter().filter(|id| id.as_str()=="taoyuan_anxi_source_debt").count(),1);
    assert!(lore::newly_completed_eggs(game.hero, &game.lore_flags, &game.easter_unlocked)
        .iter().all(|egg| egg.id != "taoyuan_anxi_source_debt"));
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(restored.gold, game.gold);
    assert_eq!(restored.easter_unlocked, game.easter_unlocked);
}

#[test]
fn refusal_neutral_and_invalid_choice_are_atomic() {
    for index in [2,3] {
        let mut game = scene(Hero::GuanYu, "taoyuan", 1002 + index as u64);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.stage = Stage::Story;
        game.lore_current = Some("anxi".into());
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index }).unwrap();
        assert!(!game.lore_flags.iter().any(|f| f == "anxi_seal"));
        assert!(!game.easter_unlocked.iter().any(|id| id == "taoyuan_anxi_source_debt"));
    }
    for index in [2,3] {
        let mut game = scene(Hero::LuBu, "redhare", 1100 + index as u64);
        game.apply(Action::Choose { index }).unwrap();
        assert!(!game.lore_flags.iter().any(|f| f == "red_hare"));
    }
    let mut game = scene(Hero::LuBu, "redhare", 1104);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "red_hare"));
    assert!(game.deck.iter().any(|card| card.id == "blood_courage"));
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert!(restored.lore_flags.iter().any(|f| f == "red_hare"));
}

#[test]
fn natural_map_fixed_seeds_include_a_two_story_guan_route() {
    let mut reachable = Vec::new();
    for seed in 1..=512u64 {
        let mut game = Game::new(Hero::GuanYu, seed);
        let Some(first) = game.route.iter().position(|node| *node == Node::Story) else {continue};
        game.apply(Action::Choose { index: first + 1 }).unwrap();
        if game.lore_current.as_deref() != Some("taoyuan") {continue;}
        game.apply(Action::Choose { index: 1 }).unwrap();
        let Some(second) = game.route.iter().position(|node| *node == Node::Story) else {continue};
        game.apply(Action::Choose { index: second + 1 }).unwrap();
        if game.lore_current.as_deref() != Some("anxi") {continue;}
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert!(game.lore_flags.iter().any(|f| f == "anxi_seal"));
        assert!(game.easter_unlocked.iter().any(|id| id == "taoyuan_anxi_source_debt"));
        reachable.push(seed);
        if reachable.len() == 3 {break;}
    }
    assert!(!reachable.is_empty(), "no natural first-two-floor seed exposed the optional accepted chain");
}
