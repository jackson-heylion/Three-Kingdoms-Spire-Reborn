//! Second-floor convoy defense through the public Game::apply(Action) seam.
//! This deliberately isolated entry first proves the missing objective;
//! natural 1->2 action reachability is a separate acceptance requirement.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, Stage, content::Hero, map};

#[test]
fn grain_convoy_is_announced_before_players_act() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 247_520, Difficulty::Warlords, "convoy-preview-247520".into(),
    );
    game.floor = 1;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).expect("enter second floor through Action");
    assert_eq!(game.floor, 2);
    assert_eq!(game.stage, Stage::Combat);
    let before = game.to_json().unwrap();
    let warning = game.tactical_objective().expect("grain convoy should be an actual objective");
    assert!(warning.contains("粮道保卫") && warning.contains("粮车") && warning.contains("防御"),
        "players need a convoy status, deadline/risk and counterplay: {warning}");
    assert_eq!(game.to_json().unwrap(), before, "preview must not change RNG or state");

    let mut legacy = Game::new(Hero::GuanYu, 247_520);
    legacy.floor = 1;
    legacy.route = vec![Node::Battle];
    legacy.route_lanes = vec![0];
    legacy.apply(Action::Choose { index: 1 }).expect("legacy battle");
    assert!(legacy.tactical_objective().is_none(), "legacy does not inherit new objectives");
}
