//! Expanded third-floor ambush: telegraphed enemy cover and player counterplay.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, Stage, content::Hero, map};

#[test]
fn third_floor_ambush_cover_is_visible_and_can_be_broken_by_real_attack() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_603, Difficulty::Warlords, "ambush-247603".into(),
    );
    // Controlled map checkpoint; combat and counterplay use public player Actions.
    game.floor = 2;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    let before = game.to_json().unwrap();
    let preview = game.route_objective_hint(Node::Battle).expect("ambush must be telegraphed");
    assert!(preview.contains("伏击") && preview.contains("护甲"), "{preview}");
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).expect("enter ambush");
    assert_eq!(game.floor, 3);
    assert_eq!(game.stage, Stage::Combat);
    let objective = game.tactical_objective().expect("ambush combat objective");
    assert!(objective.contains("伏击") && objective.contains("护甲"), "{objective}");
    let shield = game.battle.as_ref().unwrap().foes[0].block;
    assert!(shield >= 8, "ambushers should begin behind an actual shield: {shield}");
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
    game.battle.as_mut().unwrap().hand.insert(0,
        three_kingdoms_spire_reborn::Card::new("strike"));
    game.apply(Action::Play { index: 1, target: 1 }).expect("break ambush shield");
    assert!(game.battle.as_ref().unwrap().foes[0].block < shield,
        "a real player attack must break ambush protection");
}

#[test]
fn natural_third_floor_ambush_can_be_reached_and_countered_with_player_actions() {
    // No mutations to floor, map, route, hand, enemy HP, block, RNG or lore.
    // Start with real campaign actions, clear the first two encounters, then
    // choose a naturally generated third-floor battle lane.
    for seed in 247_310..247_510 {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::LuBu, seed, Difficulty::Warlords, format!("natural-ambush-{seed}"),
        );
        game.apply(Action::Choose { index: 1 }).expect("enter natural breakout");
        let mut valid = true;
        for target_floor in 1..=3 {
            if target_floor > 1 {
                if game.stage != Stage::Map { valid = false; break; }
                let Some(lane) = game.route.iter().position(|node| *node == Node::Battle)
                    else { valid = false; break; };
                if target_floor == 3 {
                    let before = game.to_json().unwrap();
                    let preview = game.route_objective_hint(Node::Battle)
                        .expect("third-floor ambush must be telegraphed");
                    assert!(preview.contains("伏击") && preview.contains("护甲"), "{preview}");
                    assert_eq!(game.to_json().unwrap(), before, "preview is read-only");
                }
                game.apply(Action::Choose { index: lane + 1 }).expect("natural battle lane");
            }
            if game.floor != target_floor || game.stage != Stage::Combat {
                valid = false; break;
            }
            if target_floor == 3 { break; }
            for _ in 0..12 {
                if game.stage != Stage::Combat { break; }
                loop {
                    let next = game.battle.as_ref().and_then(|battle| {
                        let target = battle.foes.iter().position(|foe| foe.hp > 0)? + 1;
                        battle.hand.iter().enumerate().find(|(_, card)| {
                            (card.id == "strike" || card.id == "wushuang")
                                && three_kingdoms_spire_reborn::content::card_spec(&card.id)
                                    .is_some_and(|spec| spec.cost <= battle.energy)
                        }).map(|(index, _)| (index + 1, target))
                    });
                    let Some((index, target)) = next else { break };
                    game.apply(Action::Play { index, target }).expect("natural attack");
                    if game.stage != Stage::Combat { break; }
                }
                if game.stage == Stage::Combat {
                    game.apply(Action::EndTurn).expect("natural enemy turn");
                }
            }
            if game.stage != Stage::Reward { valid = false; break; }
            game.apply(Action::Choose { index: 0 }).expect("decline card reward");
        }
        if !valid || game.floor != 3 || game.stage != Stage::Combat { continue; }
        let shield = game.battle.as_ref().unwrap().foes[0].block;
        assert!(shield >= 8, "natural ambush must start with vanguard armor");
        let notice = game.tactical_objective().expect("ambush objective");
        assert!(notice.contains("伏击") && notice.contains("护甲"), "{notice}");
        let json = game.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(), json);
        game.verify_action_replay().expect("natural route replay");
        let mut attacked = false;
        for _ in 0..5 {
            if game.stage != Stage::Combat { break; }
            let next = game.battle.as_ref().and_then(|battle| {
                battle.hand.iter().enumerate().find(|(_, card)| {
                    (card.id == "strike" || card.id == "wushuang")
                        && three_kingdoms_spire_reborn::content::card_spec(&card.id)
                            .is_some_and(|spec| spec.cost <= battle.energy)
                }).map(|(index, _)| index + 1)
            });
            if let Some(index) = next {
                game.apply(Action::Play { index, target: 1 }).expect("real break-armor attack");
                attacked = true;
                break;
            }
            game.apply(Action::EndTurn).expect("draw natural attack");
        }
        if !attacked { continue; }
        assert!(game.battle.as_ref().unwrap().foes[0].block < shield,
            "natural player attack must actually break ambush armor");
        game.verify_action_replay().expect("natural break-armor replay");
        let after = game.to_json().unwrap();
        let restored = Game::from_json(&after).unwrap();
        assert_eq!(restored.to_json().unwrap(), after);
        restored.verify_action_replay().expect("restored break-armor replay");
        return;
    }
    panic!("no natural third-floor ambush counterplay found in 200 Lu Bu seeds");
}

#[test]
fn ambush_victory_pays_a_single_counterattack_bounty() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_604, Difficulty::Warlords, "ambush-bounty".into(),
    );
    // Controlled third-floor success boundary. The player still delivers
    // the finishing blow through Game::apply; natural access is tested above.
    game.floor = 2;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).unwrap();
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes.truncate(1);
        battle.foes[0].hp = 1;
        battle.foes[0].block = 1;
        battle.hand.insert(0, three_kingdoms_spire_reborn::Card::new("strike"));
    }
    game.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert_eq!(game.stage, Stage::Reward);
    assert!(game.log.iter().any(|line| line.contains("伏击反制成功") && line.contains("16金")),
        "the ambush must pay an explicit, player-visible bounty: {:?}", game.log);
    let paid = game.gold;
    let saved = game.to_json().unwrap();
    let restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    game.apply(Action::Choose { index: 0 }).unwrap();
    assert_eq!(game.gold, paid, "reward selection cannot duplicate ambush bounty");
    let after = game.to_json().unwrap();
    assert!(game.apply(Action::EndTurn).is_err());
    assert_eq!(game.to_json().unwrap(), after, "invalid post-victory action is atomic");
}

#[test]
fn ambush_zero_armor_without_a_player_break_cannot_claim_bounty() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_605, Difficulty::Warlords, "ambush-no-free-bounty".into(),
    );
    game.floor = 2;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).unwrap();
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes.truncate(1);
        battle.foes[0].hp = 1;
        battle.foes[0].block = 0;
        battle.hand.insert(0, three_kingdoms_spire_reborn::Card::new("strike"));
    }
    game.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert_eq!(game.stage, Stage::Reward);
    assert!(!game.log.iter().any(|line| line.contains("伏击反制成功")),
        "an unearned zero-block snapshot must not pay the bounty");
    let json = game.to_json().unwrap();
    assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(), json);
}
