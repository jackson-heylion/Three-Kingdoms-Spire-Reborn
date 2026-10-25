//! Regression for boss reward -> ancient boon selection with valid battle telemetry.
//! Kept outside the large lib.rs test module to avoid expanding the engine file.
use crate::{Action, Game, Hero, Node, Stage};

#[test]
fn boss_reward_flows_into_ancient_choice_without_losing_save() {
    let mut game = Game::new(Hero::GuanYu, 444);
    game.floor = 7;
    game.last_node = Some(Node::Boss);
    game.stage = Stage::Combat;
    // Starting a real battle records battles_started. The previous fixture
    // invoked win_battle with no battle, creating an impossible 0-start/1-win
    // telemetry transition rejected by the eighth authority journal.
    game.start_battle(Node::Boss);
    assert_eq!(game.telemetry.totals.battles_started, 1);
    let _ = game.win_battle();
    assert_eq!(game.telemetry.totals.battles_won, 1);
    assert_eq!(game.stage, Stage::Reward);
    game.apply(Action::Choose { index: 0 }).unwrap();
    assert_eq!(game.stage, Stage::Ancient);
    assert_eq!(game.boon_choices.len(), 3);
    let snap = game.to_json().unwrap();
    let mut restored = Game::from_json(&snap).unwrap();
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(restored.stage, Stage::Map);
    assert_eq!(restored.boons.len(), 1);
}
