//! Expanded third-floor ambush: telegraphed enemy cover and player counterplay.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, Stage, content::Hero, map};

#[test]
fn third_floor_ambush_cover_is_visible_and_can_be_broken_by_real_attack() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_603, Difficulty::Warlords, "ambush-247603".into(),
    );
    // Controlled map checkpoint; combat and counterplay use public player Actions.
    game.floor = 2;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    let before = game.to_json().unwrap();
    let preview = game.route_objective_hint(Node::Battle).expect("ambush must be telegraphed");
    assert!(preview.contains("伏击") && preview.contains("护甲"), "{preview}");
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).expect("enter ambush");
    assert_eq!(game.floor, 3);
    assert_eq!(game.stage, Stage::Combat);
    let objective = game.tactical_objective().expect("ambush combat objective");
    assert!(objective.contains("伏击") && objective.contains("护甲"), "{objective}");
    let shield = game.battle.as_ref().unwrap().foes[0].block;
    assert!(shield >= 8, "ambushers should begin behind an actual shield: {shield}");
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
    game.battle.as_mut().unwrap().hand.insert(0,
        three_kingdoms_spire_reborn::Card::new("strike"));
    game.apply(Action::Play { index: 1, target: 1 }).expect("break ambush shield");
    assert!(game.battle.as_ref().unwrap().foes[0].block < shield,
        "a real player attack must break ambush protection");
}
