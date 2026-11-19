//! New-campaign act V alternate boss: Deng Ai's telegraphed Yinping infiltration.
//! A deliberately controlled floor-39 entrance, NOT a natural floor-0 victory.
use three_kingdoms_spire_reborn::{
    Action, BossMove, Difficulty, Game, Intent, Node, Stage, content::Hero, map,
};

fn fifth_act_alternate_boss() -> Game {
    // Seed bit 44 selects the alternate fifth-act commander. Starting at
    // floor 39 isolates the encounter without forging a full natural run.
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 1_u64 << 44, Difficulty::Warlords,
        "yinping-alternate".into(),
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
fn alternate_fifth_act_boss_exposes_a_replayable_casualty_free_supply_raid() {
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter alternate final boss");
    assert_eq!(game.floor, 40);
    assert_eq!(game.stage, Stage::Combat);
    let foe = &game.battle.as_ref().unwrap().foes[0];
    assert_eq!(foe.id, "dengai");
    assert!(matches!(foe.intent, Intent::Boss(BossMove::YinpingFlank(0))));
    let preview = foe.intent.label();
    assert!(preview.contains("阴平偷渡") && preview.contains("至少3张牌") && preview.contains("10金"), "{preview}");
    let before = game.to_json().expect("entrance save");
    let mut loaded = Game::from_json(&before).expect("restore visible intent");
    let original_gold = game.gold;
    let original = game.apply(Action::EndTurn).expect("unopposed enemy action");
    let resumed = loaded.apply(Action::EndTurn).expect("same action after restore");
    assert_eq!(original, resumed);
    assert_eq!(game.gold, original_gold - 10);
    assert!(original.join("；").contains("阴平偷渡得手"));
    assert!(game.battle.as_ref().unwrap().foes[0].block >= 6);
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
}

#[test]
fn three_real_play_actions_intercept_yinping_raid_and_preserve_resources() {
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter alternate final boss");
    // A normal starting draw contains at least four basic cards; no hand
    // injection or hero block mutation is permitted to earn this counter.
    for _ in 0..3 {
        let index = game.battle.as_ref().unwrap().hand.iter()
            .position(|card| card.id == "strike" || card.id == "defend")
            .expect("starter draw must contain a basic playable card") + 1;
        game.apply(Action::Play { index, target: 1 }).expect("actual legal card play");
    }
    assert_eq!(game.battle.as_ref().unwrap().played_this_turn, 3);
    let start_gold = game.gold;
    let snapshot = game.to_json().expect("save after three plays");
    let mut loaded = Game::from_json(&snapshot).expect("restore counterplay");
    let result = game.apply(Action::EndTurn).expect("counter infiltration");
    let replayed = loaded.apply(Action::EndTurn).expect("restored counter");
    assert_eq!(result, replayed);
    assert!(result.join("；").contains("阴平偷渡被截击"));
    assert_eq!(game.gold, start_gold);
    assert!(game.battle.as_ref().unwrap().foes[0].vulnerable >= 2);
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
}

#[test]
fn old_campaign_bosses_never_use_yinping_infiltration() {
    let mut old = Game::new(Hero::GuanYu, 1_u64 << 44);
    old.floor = 20;
    old.stage = Stage::Map;
    old.act_map = Some(map::ActMap::generate(|_| 0, 2));
    old.map_lane = None;
    old.route = vec![Node::Boss];
    old.route_lanes = vec![0];
    old.apply(Action::Choose { index: 1 }).expect("legacy final boss");
    assert_ne!(old.battle.as_ref().unwrap().foes[0].id, "dengai");
}
