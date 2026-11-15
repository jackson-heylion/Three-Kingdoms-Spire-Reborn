//! Natural second-act vertical acceptance. No checkpoint injection or fabricated battle state.
//! This is intentionally strict: a first-act defeat cannot count as act-two progress.
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
fn natural_act_two_requires_first_boss_victory_and_replayable_progress() {
    // Keep failures explicit. A death before floor 9 is not a successful traversal.
    let mut failures = Vec::new();
    for seed in 247_510..247_526 {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::LuBu, seed, Difficulty::Warlords, format!("act-two-{seed}"),
        );
        game.apply(Action::Choose { index: 1 }).expect("new campaign choice");
        let mut saw_first_boss = false;
        let mut saw_second_boss = false;
        let mut failure = None;
        for _ in 0..500 {
            if game.stage == Stage::Lost || game.stage == Stage::Won { break; }
            if game.floor == 8 && game.stage == Stage::Combat && game.last_node == Some(Node::Boss) {
                saw_first_boss = true;
            }
            if game.floor == 16 && game.stage == Stage::Combat && game.last_node == Some(Node::Boss) {
                saw_second_boss = true;
                break;
            }
            if let Err(error) = advance(&mut game) {
                failure = Some(error);
                break;
            }
        }
        if saw_first_boss && saw_second_boss {
            game.audit_campaign_authority().expect("act-two authority audit");
            game.verify_action_replay().expect("act-two action replay");
            let json = game.to_json().expect("act-two snapshot");
            let restored = Game::from_json(&json).expect("act-two restore");
            assert_eq!(restored.to_json().unwrap(), json);
            restored.verify_action_replay().expect("restored act-two replay");
            assert_eq!(game.floor, 16);
            assert_eq!(game.last_node, Some(Node::Boss));
            return;
        }
        failures.push(format!(
            "seed {seed}: floor {} stage {:?}, first boss {saw_first_boss}, second boss {saw_second_boss}, error {:?}",
            game.floor, game.stage, failure
        ));
    }
    panic!("no natural first-boss victory continued to second boss: {}", failures.join(" | "));
}
