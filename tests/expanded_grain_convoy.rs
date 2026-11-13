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
    let before_map = game.to_json().unwrap();
    let route_notice = game.route_objective_hint(Node::Battle).expect("preview before entering");
    assert!(route_notice.contains("粮道保卫") && route_notice.contains("防御"));
    assert_eq!(game.to_json().unwrap(), before_map, "route preview is read-only");
    assert!(game.route_objective_hint(Node::Market).is_none());
    game.apply(Action::Choose { index: 1 }).expect("enter second floor through Action");
    assert_eq!(game.floor, 2);
    assert_eq!(game.stage, Stage::Combat);
    let before = game.to_json().unwrap();
    let warning = game.tactical_objective().expect("grain convoy should be an actual objective");
    assert!(warning.contains("粮道保卫") && warning.contains("粮车") && warning.contains("防御"),
        "players need a convoy status, deadline/risk and counterplay: {warning}");
    assert_eq!(game.to_json().unwrap(), before, "preview must not change RNG or state");

    let mut legacy = Game::new(Hero::GuanYu, 247_520);
    legacy.apply(Action::Choose { index: 1 }).expect("legacy battle");
    assert!(legacy.tactical_objective().is_none(), "legacy does not inherit new objectives");
}


#[test]
fn convoy_cover_damage_and_canonical_defeat_survive_json() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 247_521, Difficulty::Warlords, "convoy-failure-247521".into(),
    );
    // Isolated second-floor checkpoint; the start and rounds still use public Action.
    game.floor = 1;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).unwrap();
    let before_gold = game.gold;
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
    assert_eq!(game.to_json().unwrap(), before, "bad action must be atomic");

    // A real defensive Action protects the convoy, even though this controlled
    // checkpoint ensures a defense card is in hand.
    game.battle.as_mut().unwrap().hand.insert(0,
        three_kingdoms_spire_reborn::Card::new("defend"));
    game.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert_eq!(game.battle.as_ref().unwrap().convoy_cover, Some(2));
    let current = game.to_json().unwrap();
    assert_eq!(Game::from_json(&current).unwrap().to_json().unwrap(), current);

    // Control the enemy's ordinary intent to isolate convoy-loss semantics.
    for _ in 0..4 {
        if game.stage != Stage::Combat { break; }
        game.battle.as_mut().unwrap().foes[0].intent =
            three_kingdoms_spire_reborn::Intent::Guard(0);
        game.apply(Action::EndTurn).unwrap();
    }
    assert_eq!(game.stage, Stage::Lost, "unprotected convoy must eventually be destroyed");
    assert_eq!(game.gold, before_gold, "failed defense never grants battle payout");
    assert!(game.log.last().unwrap().contains("粮道保卫失败"));
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}

#[test]
fn convoy_victory_uses_normal_action_and_reward_without_legacy_effect() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_522, Difficulty::Warlords, "convoy-win-247522".into(),
    );
    game.floor = 1;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).unwrap();
    let before_gold = game.gold;
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes.truncate(1);
        battle.foes[0].hp = 1; // Controlled success boundary, not a natural run.
        battle.foes[0].block = 0;
        battle.hand.insert(0, three_kingdoms_spire_reborn::Card::new("strike"));
    }
    game.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert_eq!(game.stage, Stage::Reward);
    assert!(game.gold > before_gold, "successful defense pays ordinary battle rewards");
    assert!(game.battle.is_none(), "no convoy persists into reward");
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}
