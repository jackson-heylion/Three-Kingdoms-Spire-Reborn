//! Isolated fourth-act boss behavior through the public Game::apply(Action) seam.
//! This is a controlled floor-31 checkpoint, NOT natural 1-to-32 reachability proof.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, Stage, content::Hero, map};

#[test]
fn fourth_act_boss_previews_distinct_fire_camp_mechanic() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 247_411, Difficulty::Warlords, "act-four-fire-247411".into(),
    );
    game.floor = 31;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 3));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).expect("fourth-act boss via public action");
    assert_eq!(game.floor, 32);
    assert_eq!(game.stage, Stage::Combat);
    let enemy = &game.battle.as_ref().unwrap().foes[0];
    assert_eq!(enemy.id, "luxun", "fourth act must not silently reuse act-III Boss");
    assert!(enemy.intent.label().contains("焚营"), "fire tactic must be previewed");
    Game::from_json(&game.to_json().unwrap()).expect("new Boss intent round-trip");
}
