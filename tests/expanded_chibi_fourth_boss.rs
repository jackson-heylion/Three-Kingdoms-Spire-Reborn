//! Controlled act-IV boss entrance for Red Cliffs' persistent consequences.
//! The actual settlement, enemy intents and JSON replay use Game::apply.
//! This is not evidence of an unmodified 0→40 seed journey.
use three_kingdoms_spire_reborn::{
    Action, ChibiPlan, Difficulty, Game, Node, Stage, content::Hero, map,
};

fn act_four_commander(hero: Hero) -> Game {
    let mut game = Game::new_with_campaign_difficulty(
        hero, 1_u64 << 33, Difficulty::Warlords, "chibi-act-four-boss".into(),
    );
    game.floor = 31;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 3));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    game
}

#[test]
fn red_cliffs_three_outcomes_change_a_real_fourth_act_boss_and_survive_restore() {
    for hero in [Hero::ZhugeLiang, Hero::SunShangxiang] {
        for (choice, marker) in [
            (ChibiPlan::FireSettled, "chibi_fourth_act_fire"),
            (ChibiPlan::FleetSettled, "chibi_fourth_act_fleet"),
            (ChibiPlan::DeclineSettled, "chibi_fourth_act_declined"),
        ] {
            let mut game = act_four_commander(hero);
            game.chibi_plan = Some(choice);
            let preview = game.route_objective_hint(Node::Boss).unwrap();
            assert!(preview.contains("赤壁"), "{preview}");
            let result = game.apply(Action::Choose { index: 1 }).unwrap();
            assert_eq!(game.floor, 32);
            assert_eq!(game.stage, Stage::Combat);
            assert_eq!(game.chibi_plan, Some(choice));
            assert!(result.join("；").contains("赤壁"));
            assert!(game.tactical_objective().unwrap().contains("赤壁"));
            let b = game.battle.as_ref().unwrap();
            assert_eq!(b.foes[0].id, "lumeng");
            assert!(b.events.iter().any(|event| event.source == marker));
            match choice {
                ChibiPlan::FireSettled => assert!(b.foes[0].weak >= 2),
                ChibiPlan::FleetSettled => {
                    assert!(b.block >= 8);
                    assert!(b.energy >= 4);
                }
                ChibiPlan::DeclineSettled => {
                    assert!(b.foes[0].strength >= 1);
                    assert!(b.foes[0].block >= 10);
                }
                _ => unreachable!(),
            }
            let mut resumed = Game::from_json(&game.to_json().unwrap()).unwrap();
            let event = game.apply(Action::EndTurn).unwrap();
            assert_eq!(event, resumed.apply(Action::EndTurn).unwrap());
            assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
        }
    }
}
