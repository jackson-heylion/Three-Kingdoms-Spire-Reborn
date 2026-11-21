//! The main fifth-act Jiang Wei commander now honors three genuinely
//! committed Jieting plans. This is an explicit, controlled floor checkpoint,
//! NOT a claim of naturally navigating the whole 40-floor campaign.
use three_kingdoms_spire_reborn::{
    Action, Difficulty, Game, JietingPlan, Node, Stage, content::Hero, map,
};

fn jiangwei_checkpoint() -> Game {
    // Seed 0 selects act-V's primary commander; the opponent is not
    // injected. Route, turns, encounter intents and rewards use Game::apply.
    let mut game = Game::new_with_campaign_difficulty(
        Hero::ZhugeLiang, 0, Difficulty::Warlords, "jieting-jiangwei".into(),
    );
    game.floor = 39;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    game
}

#[test]
fn all_three_jieting_choices_change_jiangwei_and_replay_identically() {
    for (plan, marker) in [
        (JietingPlan::WaterSettled, "jieting_jiangwei_wells"),
        (JietingPlan::HillSettled, "jieting_jiangwei_hills"),
        (JietingPlan::RetreatSettled, "jieting_jiangwei_retreat"),
    ] {
        let mut game = jiangwei_checkpoint();
        game.jieting_plan = Some(plan);
        assert!(game.route_objective_hint(Node::Boss).unwrap().contains("街亭"));
        let feedback = game.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(game.floor, 40);
        assert_eq!(game.stage, Stage::Combat);
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].id, "jiangwei");
        assert!(battle.events.iter().any(|event| event.source == marker));
        assert!(game.tactical_objective().unwrap().contains("北伐史诗关底"));
        assert!(feedback.join("；").contains("街亭"));
        match plan {
            JietingPlan::WaterSettled => {
                assert!(battle.block >= 8);
                assert!(battle.foes[0].weak >= 2);
            }
            JietingPlan::HillSettled => {
                assert!(battle.foes[0].strength >= 2);
                assert!(battle.foes[0].block >= 8);
                assert!(battle.strength >= 1);
            }
            JietingPlan::RetreatSettled => {
                assert!(battle.block >= 4);
                assert!(battle.energy >= 4);
                assert!(battle.foes[0].block >= 6);
            }
            _ => unreachable!(),
        }
        let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        let result = game.apply(Action::EndTurn).unwrap();
        assert_eq!(result, restored.apply(Action::EndTurn).unwrap());
        assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    }
}
