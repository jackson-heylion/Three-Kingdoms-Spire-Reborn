//! Natural siege entry: no artificial floor, route, deck, HP or lore injection.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, Stage, content::{Hero, card_spec}};

#[test]
fn natural_fourth_floor_siege_is_telegraphed_and_replayable() {
    for seed in 247_510..247_710 {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::LuBu, seed, Difficulty::Warlords, format!("natural-siege-{seed}"),
        );
        game.apply(Action::Choose { index: 1 }).expect("start expanded campaign");
        let mut reached = true;
        for floor in 1..=4 {
            if floor > 1 {
                if game.stage != Stage::Map { reached = false; break; }
                let Some(lane) = game.route.iter().position(|node| *node == Node::Battle)
                    else { reached = false; break };
                if floor == 4 {
                    let snapshot = game.to_json().unwrap();
                    let hint = game.route_objective_hint(Node::Battle).expect("siege warning");
                    assert!(hint.contains("攻城") && hint.contains("第5回合")
                        && hint.contains("2军令"), "{hint}");
                    assert_eq!(game.to_json().unwrap(), snapshot, "preview must be read-only");
                }
                game.apply(Action::Choose { index: lane + 1 }).expect("choose natural battle");
            }
            if game.floor != floor || game.stage != Stage::Combat {
                reached = false; break;
            }
            if floor == 4 { break; }
            for _ in 0..12 {
                if game.stage != Stage::Combat { break; }
                loop {
                    let attack = game.battle.as_ref().and_then(|battle| {
                        let target = battle.foes.iter().position(|foe| foe.hp > 0)? + 1;
                        battle.hand.iter().enumerate().find(|(_, card)| {
                            (card.id == "strike" || card.id == "wushuang")
                                && card_spec(&card.id).is_some_and(|spec| spec.cost <= battle.energy)
                        }).map(|(index, _)| (index + 1, target))
                    });
                    let Some((index, target)) = attack else { break };
                    game.apply(Action::Play { index, target }).expect("play natural attack");
                    if game.stage != Stage::Combat { break; }
                }
                if game.stage == Stage::Combat {
                    game.apply(Action::EndTurn).expect("resolve natural enemy turn");
                }
            }
            if game.stage != Stage::Reward { reached = false; break; }
            game.apply(Action::Choose { index: 0 }).expect("decline reward");
        }
        if !reached || game.stage != Stage::Combat { continue; }
        assert_eq!(game.floor, 4);
        assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(12));
        assert!(game.tactical_objective().unwrap().contains("12/12"));
        game.verify_action_replay().expect("natural route replay");
        let saved = game.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
        game.apply(Action::AssaultGate).expect("natural siege action");
        assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(8));
        let before_invalid = game.to_json().unwrap();
        assert!(game.apply(Action::AssaultGate).is_err(), "second assault must need more energy");
        assert_eq!(game.to_json().unwrap(), before_invalid, "failed assault is atomic");
        game.verify_action_replay().expect("natural siege action replay");
        let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        restored.verify_action_replay().expect("restored natural siege replay");
        return;
    }
    panic!("no naturally reachable fourth-floor siege in 200 Lu Bu seeds");
}
