//! First-act end-to-end acceptance: only Game::apply(Action) from a real new campaign.
//! This deliberately fails if no tested seed can reach the floor-eight boss.
//! It does not assert a 40-floor victory or count a checkpoint as natural reachability.
use three_kingdoms_spire_reborn::{
    Action, Difficulty, Game, Node, Stage, content::{Hero, card_spec},
};

fn advance_combat(game: &mut Game) -> Result<(), String> {
    for _ in 0..24 {
        if game.stage != Stage::Combat { return Ok(()); }
        if game.battle.as_ref().is_some_and(|b| b.siege_gate_hp.is_some()) {
            if game.battle.as_ref().is_some_and(|b| b.energy >= 2) {
                game.apply(Action::AssaultGate)?;
                continue;
            }
        }
        loop {
            let choice = game.battle.as_ref().and_then(|b| {
                let target = b.foes.iter().position(|foe| foe.hp > 0)? + 1;
                b.hand.iter().enumerate().find(|(_, card)| {
                    (card.id == "strike" || card.id == "wushuang")
                        && card_spec(&card.id).is_some_and(|spec| spec.cost <= b.energy)
                }).map(|(i, _)| (i + 1, target))
            });
            let Some((index, target)) = choice else { break; };
            game.apply(Action::Play { index, target })?;
            if game.stage != Stage::Combat { break; }
        }
        if game.stage == Stage::Combat { game.apply(Action::EndTurn)?; }
    }
    if game.stage == Stage::Combat { return Err("combat exceeded 24 turns".into()); }
    Ok(())
}

fn follow_first_act(seed: u64) -> Result<Game, String> {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, seed, Difficulty::Warlords, format!("first-act-{seed}"),
    );
    game.apply(Action::Choose { index: 1 })?;
    let mut steps = 0;
    while game.floor < 8 && game.stage != Stage::Lost && steps < 300 {
        steps += 1;
        match game.stage {
            Stage::Combat => advance_combat(&mut game)?,
            Stage::Reward => { game.apply(Action::Choose { index: 0 })?; }
            Stage::Camp => { game.apply(Action::Choose { index: 1 })?; }
            Stage::Market => { game.apply(Action::Choose { index: 0 })?; }
            Stage::Story => { game.apply(Action::Choose { index: 3 })?; }
            Stage::Map => {
                // Prefer safe recovery/story nodes. The map and legal lane indices
                // come from the game, not a manufactured route.
                let preferred = [Node::Camp, Node::Story, Node::Market, Node::Battle, Node::Elite, Node::Boss];
                let index = preferred.iter().find_map(|kind|
                    game.route.iter().position(|node| node == kind)
                ).ok_or("no legal onward route")?;
                game.apply(Action::Choose { index: index + 1 })?;
            }
            Stage::Ancient | Stage::Won | Stage::Lost => break,
        }
    }
    if game.floor != 8 || game.stage != Stage::Combat {
        return Err(format!("seed {seed} ended at floor {} stage {:?} after {steps} actions", game.floor, game.stage));
    }
    if game.last_node != Some(Node::Boss) { return Err("floor 8 did not open a boss".into()); }
    Ok(game)
}

#[test]
fn natural_new_campaign_reaches_first_act_boss_with_replay_and_resume() {
    let mut failures = Vec::new();
    for seed in 247_510..247_526 {
        match follow_first_act(seed) {
            Ok(game) => {
                game.verify_action_replay().expect("all actions from actual new campaign replay");
                game.audit_campaign_authority().expect("canonical authority audit");
                let saved = game.to_json().expect("boss entrance snapshot");
                let restored = Game::from_json(&saved).expect("boss entrance restore");
                assert_eq!(restored.to_json().unwrap(), saved);
                restored.verify_action_replay().expect("restored first-boss trajectory");
                assert_eq!(game.floor, 8);
                return;
            }
            Err(error) => failures.push(error),
        }
    }
    panic!("no seed naturally reached floor-eight boss: {}", failures.join(" | "));
}
