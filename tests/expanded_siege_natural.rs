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


/// Use only the real campaign action seam to reach the fourth-floor gate.
fn seeded_siege_entry(seed: u64) -> Option<Game> {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu,
        seed,
        Difficulty::Warlords,
        format!("natural-siege-outcomes-{seed}"),
    );
    game.apply(Action::Choose { index: 1 }).ok()?;
    for floor in 1..=4 {
        if floor > 1 {
            if game.stage != Stage::Map {
                return None;
            }
            let lane = game.route.iter().position(|node| *node == Node::Battle)?;
            game.apply(Action::Choose { index: lane + 1 }).ok()?;
        }
        if game.floor != floor || game.stage != Stage::Combat {
            return None;
        }
        if floor == 4 {
            return Some(game);
        }
        for _ in 0..12 {
            if game.stage != Stage::Combat {
                break;
            }
            loop {
                let next = game.battle.as_ref().and_then(|battle| {
                    let target = battle.foes.iter().position(|foe| foe.hp > 0)? + 1;
                    battle.hand.iter().enumerate().find(|(_, card)| {
                        (card.id == "strike" || card.id == "wushuang")
                            && card_spec(&card.id).is_some_and(|spec| spec.cost <= battle.energy)
                    }).map(|(index, _)| (index + 1, target))
                });
                let Some((index, target)) = next else {
                    break;
                };
                game.apply(Action::Play { index, target }).ok()?;
                if game.stage != Stage::Combat {
                    break;
                }
            }
            if game.stage == Stage::Combat {
                game.apply(Action::EndTurn).ok()?;
            }
        }
        if game.stage != Stage::Reward {
            return None;
        }
        game.apply(Action::Choose { index: 0 }).ok()?;
    }
    None
}

#[test]
fn natural_siege_breach_and_timeout_settle_once_and_replay() {
    let mut victory_seed = None;
    let mut timeout_seed = None;
    for seed in 247_510..247_710 {
        let Some(entry) = seeded_siege_entry(seed) else {
            continue;
        };
        if victory_seed.is_none() {
            let mut win = entry.clone();
            let gold_before = win.gold;
            for _ in 0..3 {
                if win.stage != Stage::Combat {
                    break;
                }
                win.apply(Action::AssaultGate).expect("breach gate with real action");
                if win.stage == Stage::Combat {
                    win.apply(Action::EndTurn).expect("defenders get a turn");
                }
            }
            if win.stage == Stage::Reward {
                assert!(win.gold > gold_before, "gate victory pays combat gold");
                assert!(win.log.iter().any(|line| line.contains("城门已破")));
                win.verify_action_replay().expect("victory action replay");
                let saved = win.to_json().unwrap();
                let restored = Game::from_json(&saved).unwrap();
                assert_eq!(restored.to_json().unwrap(), saved);
                restored.verify_action_replay().expect("restored victory replay");
                assert!(win.apply(Action::AssaultGate).is_err());
                assert_eq!(win.to_json().unwrap(), saved, "invalid action is atomic");
                win.apply(Action::Choose { index: 0 }).unwrap();
                assert_eq!(win.gold, gold_before + (restored.gold - gold_before));
                win.verify_action_replay().expect("reward claimed only once");
                victory_seed = Some(seed);
            }
        }
        if timeout_seed.is_none() {
            let mut lost = entry;
            let gold_before = lost.gold;
            for _ in 0..5 {
                if lost.stage != Stage::Combat {
                    break;
                }
                lost.apply(Action::EndTurn).expect("defenders act on real turn");
            }
            if lost.stage == Stage::Lost && lost.log.iter().any(|line| line.contains("攻城超时")) {
                assert_eq!(lost.gold, gold_before, "timeout cannot pay victory gold");
                assert!(lost.event_history.as_ref().is_some_and(|events| {
                    events.last().is_some_and(|event| event.event.source == "defeat")
                }));
                lost.verify_action_replay().expect("timed defeat replay");
                let saved = lost.to_json().unwrap();
                let restored = Game::from_json(&saved).unwrap();
                assert_eq!(restored.to_json().unwrap(), saved);
                restored.verify_action_replay().expect("restored timed defeat");
                assert!(lost.apply(Action::AssaultGate).is_err());
                assert_eq!(lost.to_json().unwrap(), saved, "defeat is terminal");
                timeout_seed = Some(seed);
            }
        }
        if victory_seed.is_some() && timeout_seed.is_some() {
            break;
        }
    }
    assert!(victory_seed.is_some(), "no reachable seed survives two defender turns to breach the gate");
    assert!(timeout_seed.is_some(), "no reachable seed survives long enough for the fifth-turn timeout");
}
