//! New-campaign act IV alternate boss: Lü Meng's visible, block-counterable infiltration.
//! Controlled boss entrance; this is not proof of a natural 1..32 journey.
use three_kingdoms_spire_reborn::{
    Action, BossMove, Difficulty, Game, Intent, Node, Stage, content::Hero, map,
};

fn fourth_act_boss(seed: u64) -> Game {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, seed, Difficulty::Warlords, format!("white-clothes-{seed}"),
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
fn act_four_alternate_boss_forecasts_white_clothes_and_punishes_unprotected_player() {
    // Bit 33 forces act IV's natural seeded alternate branch without injecting a boss.
    let mut game = fourth_act_boss(1u64 << 33);
    game.apply(Action::Choose { index: 1 }).expect("enter act IV boss");
    assert_eq!(game.floor, 32);
    assert_eq!(game.stage, Stage::Combat);
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.foes[0].id, "lumeng");
    assert!(matches!(battle.foes[0].intent, Intent::Boss(BossMove::WhiteClothes)));
    let intent = battle.foes[0].intent.label();
    assert!(intent.contains("白衣渡江") && intent.contains("8") && intent.contains("军令"), "{intent}");
    assert!(battle.block < 8, "starting relic armor is not enough to foil the raid");
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::AssaultGate).is_err());
    assert_eq!(game.to_json().unwrap(), before, "unrelated action is atomic");
    game.apply(Action::EndTurn).expect("enemy completes visible maneuver");
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.energy, 2, "infiltration cuts next-turn energy by one");
    let snapshot = game.to_json().unwrap();
    assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(), snapshot);
    assert!(game.log.iter().any(|entry| entry.contains("白衣渡江")));
}

#[test]
fn a_real_defend_action_exposes_lu_meng_instead_of_losing_next_turn_energy() {
    // The block must be produced by an actual Play action; no hand/block mutation.
    let mut defended = None;
    for offset in 0..32 {
        let mut game = fourth_act_boss((1u64 << 33) + offset);
        game.apply(Action::Choose { index: 1 }).unwrap();
        let index = game.battle.as_ref().and_then(|battle| {
            battle.hand.iter().position(|card| card.id == "defend")
        });
        if let Some(index) = index {
            defended = Some((game, index + 1));
            break;
        }
    }
    let (mut game, index) = defended.expect("at least one seeded draw must offer a defense card");
    assert!(matches!(game.battle.as_ref().unwrap().foes[0].intent,
        Intent::Boss(BossMove::WhiteClothes)));
    game.apply(Action::Play { index, target: 1 }).expect("real player defense");
    assert!(game.battle.as_ref().unwrap().block >= 8);
    game.apply(Action::EndTurn).expect("resolve the telegraphed raid");
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.energy, 3, "defensive counterplay prevents next-turn penalty");
    assert!(battle.foes[0].vulnerable > 0, "exposed infiltrators are vulnerable");
    assert!(game.log.iter().any(|entry| entry.contains("识破")));
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}

#[test]
fn original_three_act_boss_routes_do_not_gain_the_new_infiltrator() {
    let mut legacy = Game::new(Hero::GuanYu, 1u64 << 33);
    legacy.floor = 20;
    legacy.stage = Stage::Map;
    legacy.act_map = Some(map::ActMap::generate(|_| 0, 2));
    legacy.map_lane = None;
    legacy.route = vec![Node::Boss];
    legacy.route_lanes = vec![0];
    legacy.apply(Action::Choose { index: 1 }).unwrap();
    assert_ne!(legacy.battle.as_ref().unwrap().foes[0].id, "lumeng");
}
