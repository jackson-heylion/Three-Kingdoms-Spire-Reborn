//! Fourth-floor siege: an explicit gate-breach victory objective.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, Stage, content::Hero, map};

fn siege_checkpoint() -> Game {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_704, Difficulty::Warlords, "siege-gate".into(),
    );
    game.floor = 3;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game
}

#[test]
fn siege_gate_is_visible_and_player_can_win_without_killing_defenders() {
    let mut game = siege_checkpoint();
    let before = game.to_json().unwrap();
    let preview = game.route_objective_hint(Node::Battle).expect("siege warning");
    assert!(preview.contains("攻城") && preview.contains("第5回合") && preview.contains("2军令"), "{preview}");
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.floor, 4);
    assert_eq!(game.stage, Stage::Combat);
    assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(12));
    assert!(game.tactical_objective().unwrap().contains("12/12"));
    let before_invalid = game.to_json().unwrap();
    assert!(game.apply(Action::AssaultGate).is_ok());
    assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(8));
    assert_eq!(game.battle.as_ref().unwrap().energy, 1);
    let before_exhausted = game.to_json().unwrap();
    assert!(game.apply(Action::AssaultGate).is_err(), "one turn must not breach the entire gate");
    assert_eq!(game.to_json().unwrap(), before_exhausted, "failed assault is atomic");
    assert_ne!(game.to_json().unwrap(), before_invalid);
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.stage, Stage::Combat);
    game.apply(Action::AssaultGate).unwrap();
    assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(4));
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.stage, Stage::Combat);
    game.apply(Action::AssaultGate).unwrap();
    assert_eq!(game.stage, Stage::Reward);
    assert!(game.battle.is_none());
    assert!(game.log.iter().any(|line| line.contains("城门已破")));
    let victory = game.to_json().unwrap();
    assert!(game.apply(Action::AssaultGate).is_err());
    assert_eq!(game.to_json().unwrap(), victory, "invalid assault is atomic");
}

#[test]
fn siege_assault_requires_energy_and_does_not_affect_ordinary_battles() {
    let mut game = siege_checkpoint();
    assert!(game.apply(Action::AssaultGate).is_err());
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.battle.as_mut().unwrap().energy = 0;
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::AssaultGate).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    let mut ordinary = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_705, Difficulty::Warlords, "ordinary".into(),
    );
    ordinary.apply(Action::Choose { index: 1 }).unwrap();
    let before = ordinary.to_json().unwrap();
    assert!(ordinary.apply(Action::AssaultGate).is_err());
    assert_eq!(ordinary.to_json().unwrap(), before);
}
