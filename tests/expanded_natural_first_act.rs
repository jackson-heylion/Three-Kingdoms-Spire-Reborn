//! First-act end-to-end acceptance: only Game::apply(Action) from a real new campaign.
//! This deliberately fails if no tested seed can reach the floor-eight boss.
//! It does not assert a 40-floor victory or count a checkpoint as natural reachability.
use three_kingdoms_spire_reborn::{
    Action, Difficulty, Game, Node, Stage, content::{Hero, card_spec},
};

fn advance_combat(game: &mut Game) -> Result<(), String> {
    for _ in 0..24 {
        if game.stage != Stage::Combat { return Ok(()); }
        if game.battle.as_ref().is_some_and(|b| b.siege_gate_hp.is_some() && b.energy >= 2) {
            game.apply(Action::AssaultGate)?;
            continue;
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
                // Continue the *same* natural action history through the boss.
                // Winning must expose a reward; losing must be an authoritative
                // terminal result. Neither is synthesized by a test fixture.
                let mut resolved = game;
                advance_combat(&mut resolved).expect("first boss must resolve via player actions");
                assert!(
                    matches!(resolved.stage, Stage::Reward | Stage::Lost),
                    "boss combat must end in reward or defeat, got {:?}",
                    resolved.stage
                );
                resolved.audit_campaign_authority().expect("boss resolution authority audit");
                resolved.verify_action_replay().expect("boss result action replay");
                let outcome = resolved.to_json().expect("boss outcome snapshot");
                let resumed = Game::from_json(&outcome).expect("restore boss result");
                assert_eq!(resumed.to_json().unwrap(), outcome);
                resumed.verify_action_replay().expect("restored boss result replay");
                if resolved.stage == Stage::Reward {
                    // Reward is an explicit player choice, not automatic settlement.
                    resolved.apply(Action::Choose { index: 0 }).expect("accept boss reward");
                    resolved.audit_campaign_authority().expect("post-reward authority audit");
                    resolved.verify_action_replay().expect("post-reward action replay");
                    let awarded = resolved.to_json().expect("reward snapshot");
                    assert_eq!(Game::from_json(&awarded).unwrap().to_json().unwrap(), awarded);
                }
                return;
            }
            Err(error) => failures.push(error),
        }
    }
    panic!("no seed naturally reached floor-eight boss: {}", failures.join(" | "));
}


/// Independent defeat path: the player reaches the real floor-eight boss,
/// then deliberately refuses to defend. No battle state, HP, RNG or map is
/// manufactured. This must remain distinct from the victory/reward path.
#[test]
fn natural_first_boss_defeat_is_terminal_atomic_and_replayable() {
    let mut failures = Vec::new();
    for seed in 247_510..247_526 {
        let Ok(mut game) = follow_first_act(seed) else {
            failures.push(format!("seed {seed}: could not enter boss"));
            continue;
        };
        let entrance = game.to_json().expect("natural boss entrance snapshot");
        let mut turns = 0;
        while game.stage == Stage::Combat && turns < 80 {
            game.apply(Action::EndTurn).expect("end turn is a legal player action");
            turns += 1;
        }
        if game.stage != Stage::Lost {
            failures.push(format!(
                "seed {seed}: refusing to defend ended at {:?} after {turns} turns",
                game.stage
            ));
            continue;
        }
        assert_ne!(game.to_json().unwrap(), entrance, "defeat must change game state");
        game.audit_campaign_authority().expect("natural defeat authority audit");
        game.verify_action_replay().expect("natural defeat action replay");
        let lost = game.to_json().expect("defeat snapshot");
        let restored = Game::from_json(&lost).expect("restore natural defeat");
        assert_eq!(restored.to_json().unwrap(), lost);
        restored.verify_action_replay().expect("restored defeat replay");

        // A terminal game cannot grant another reward or mutate its history.
        assert!(game.apply(Action::Choose { index: 0 }).is_err());
        assert_eq!(game.to_json().unwrap(), lost, "invalid post-defeat action must be atomic");
        return;
    }
    panic!(
        "no naturally entered first boss produced a replayable defeat: {}",
        failures.join(" | ")
    );
}
