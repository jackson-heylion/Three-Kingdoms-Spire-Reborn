//! Natural fifth-act full-victory acceptance. All four earlier bosses must be won to reach floor 40. No checkpoint injection or fabricated battle state.
//! This is intentionally strict: defeats at earlier bosses cannot count as fifth-act progress.
use three_kingdoms_spire_reborn::{
    Action, Difficulty, Game, Node, Stage, WuchaoPlan, content::{Effect, Hero, card_spec}, progression::Profile,
};

fn fight(game: &mut Game) -> Result<(), String> {
    for _ in 0..32 {
        if game.stage != Stage::Combat { return Ok(()); }
        if game.battle.as_ref().is_some_and(|b| b.siege_gate_hp.is_some() && b.energy >= 2) {
            game.apply(Action::AssaultGate)?;
            continue;
        }
        // This is a legal, deterministic player policy, not a combat shortcut.
        // The earlier probe discarded every defensive card, making long-run
        // survival an artifact of the test driver rather than the game rules.
        let mut guarded = false;
        loop {
            let playable = game.battle.as_ref().and_then(|b| {
                // Focus the lowest-HP living enemy to remove an incoming intent sooner.
                // Earlier policies always attacked the first living foe, even when
                // another attacker could be eliminated immediately.
                let target = b.foes.iter().enumerate()
                    .filter(|(_, foe)| foe.hp > 0)
                    .min_by_key(|(_, foe)| foe.hp)
                    .map(|(index, _)| index + 1)?;
                let defend = !guarded && b.block < 8 && b.foes.iter().any(|foe|
                    foe.hp > 0 && matches!(foe.intent, three_kingdoms_spire_reborn::Intent::Attack(_)
                        | three_kingdoms_spire_reborn::Intent::MultiAttack(_, _))
                );
                // Use the cards actually acquired during the 40-floor campaign.
                // The former driver ignored every reward card except wushuang,
                // so its late-game survival did not measure the available build.
                // Exclude cards needing a separate selection UI or costing HP;
                // they require an explicit player policy rather than blind use.
                let choice = b.hand.iter().enumerate().filter_map(|(index, card)| {
                    let spec = card_spec(&card.id)?;
                    if spec.cost > b.energy && spec.cost >= 0 { return None; }
                    let priority = match spec.effect {
                        Effect::Unplayable | Effect::TutorAttack | Effect::CopyDiscard
                        | Effect::BloodPrice(_, _) => return None,
                        Effect::Heal(_) if game.hp >= game.max_hp => return None,
                        Effect::Guard(_) | Effect::GuardEnergy(_) | Effect::XGuard(_)
                            if !defend => return None,
                        Effect::Draw(_) | Effect::DrawEnergy(_, _)
                        | Effect::Energy(_) | Effect::GainMomentum(_)
                        | Effect::Fury(_) | Effect::Strength(_) | Effect::Power(_, _) => 0,
                        Effect::Heal(_) => 1,
                        Effect::Hit(_) | Effect::HitDraw(_, _) | Effect::HitTwice(_)
                        | Effect::All(_) | Effect::FuryStrike(_) | Effect::XHit(_)
                        | Effect::Pierce(_) | Effect::HitWeak(_, _)
                        | Effect::HitVulnerable(_, _) | Effect::Leech(_)
                        | Effect::HitPoison(_, _) => 2,
                        Effect::Guard(_) | Effect::GuardEnergy(_) | Effect::XGuard(_) => 3,
                        _ => 4,
                    };
                    Some((priority, index, card))
                }).min_by_key(|(priority, index, _)| (*priority, *index));
                choice.map(|(_, index, card)| (index + 1, target, matches!(card_spec(&card.id).map(|s| s.effect), Some(Effect::Guard(_) | Effect::GuardEnergy(_) | Effect::XGuard(_)))))
            });
            let Some((index, target, is_guard)) = playable else { break; };
            game.apply(Action::Play { index, target })?;
            guarded |= is_guard;
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
        // A real campaign must acquire offered cards. Choosing 0 explicitly
        // skips every reward and leaves the build at starter strength.
        Stage::Reward => {
            let index = if game.rewards.is_empty() { 0 } else { 1 };
            game.apply(Action::Choose { index }).map(|_| ())
        },
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
        let mut floor_trace = Vec::new();
        let mut observed_floor = 0;
        for _ in 0..900 {
            if game.floor != observed_floor {
                observed_floor = game.floor;
                floor_trace.push(format!("{}:{:?}:hp{}", game.floor, game.stage, game.hp));
            }
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
            // treating merely entering its combat as completing the campaign.
            let boss_entry = game.to_json().expect("final-boss entry snapshot");
            let boss_entry_restored = Game::from_json(&boss_entry).expect("final-boss entry restore");
            assert_eq!(boss_entry_restored.to_json().unwrap(), boss_entry);
            boss_entry_restored.verify_action_replay().expect("final-boss entry replay");
            if let Err(error) = fight(&mut game) {
                failures.push(format!("seed {seed}: floor-40 boss fight failed: {error}"));
                continue;
            }
            if game.stage == Stage::Reward
                && let Err(error) = game.apply(Action::Choose { index: 0 }) {
                    failures.push(format!("seed {seed}: final reward choice failed: {error}"));
                    continue;
            }
            if game.stage != Stage::Won {
                failures.push(format!("seed {seed}: final boss did not produce campaign victory: {:?}", game.stage));
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
            let mut chronicle = Profile::new();
            assert!(chronicle.record(&game, game.run_id.clone()).expect("record victory"));
            let recorded = chronicle.to_json().expect("serialize winning chronicle");
            let mut restored_chronicle = Profile::from_json(&recorded).expect("restore winning chronicle");
            assert!(!restored_chronicle.record(&restored, game.run_id.clone()).expect("idempotent victory"));
            assert_eq!(restored_chronicle.wins, 1);
            assert_eq!(restored_chronicle.losses, 0);
            assert_eq!(restored_chronicle.records.len(), 1);
            assert!(restored_chronicle.ascensions.is_empty(), "new tier must not unlock old ascension");
            assert!(restored_chronicle.difficulty_clears
                .get(game.hero.name()).is_some_and(|tiers| tiers.contains(&Difficulty::Warlords)));
            assert_eq!(restored_chronicle.to_json().unwrap(), recorded);

            // Re-run the exact winning seed from a fresh campaign, with no
            // checkpoint injection. Both runs must produce identical final
            // authority snapshots, not merely the same terminal stage.
            let mut repeated = Game::new_with_campaign_difficulty(
                Hero::LuBu, seed, Difficulty::Warlords, format!("act-five-{seed}"),
            );
            repeated.apply(Action::Choose { index: 1 }).expect("repeat opening choice");
            for _ in 0..900 {
                if repeated.stage == Stage::Won || repeated.stage == Stage::Lost { break; }
                if repeated.floor == 40 && repeated.stage == Stage::Combat
                    && repeated.last_node == Some(Node::Boss)
                {
                    fight(&mut repeated).expect("repeat final boss");
                    if repeated.stage == Stage::Reward {
                        repeated.apply(Action::Choose { index: 0 })
                            .expect("repeat final reward choice");
                    }
                    break;
                }
                advance(&mut repeated).expect("repeat natural player action");
            }
            assert_eq!(repeated.stage, Stage::Won, "winning seed {seed} did not reproduce");
            assert_eq!(repeated.to_json().unwrap(), json, "winning seed {seed} diverged");
            repeated.verify_action_replay().expect("repeat winning action replay");

            // A settled campaign is terminal: illegal attempts must not
            // re-enter rewards or mutate RNG, records, or serialized state.
            let terminal_snapshot = game.to_json().unwrap();
            assert!(game.apply(Action::Choose { index: 1 }).is_err());
            assert_eq!(game.to_json().unwrap(), terminal_snapshot);
            assert!(game.apply(Action::EndTurn).is_err());
            assert_eq!(game.to_json().unwrap(), terminal_snapshot);
            return;
        }
        failures.push(format!(
            "seed {seed}: floor {} stage {:?}, first boss {saw_first_boss}, second boss {saw_second_boss}, third boss {saw_third_boss}, fourth boss {saw_fourth_boss}, fifth boss {saw_fifth_boss}, error {:?}, trace {:?}",
            game.floor, game.stage, failure, floor_trace
        ));
    }
    panic!("no natural full victory through bosses 8, 16, 24, 32 and 40: {}", failures.join(" | "));
}

/// Passive player actions produce a real defeat without injecting a checkpoint.
#[test]
fn natural_expanded_defeat_is_recorded_once_without_unlocking_a_clear() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::LuBu, 247_510, Difficulty::Warlords, "natural-defeat-247510".into(),
    );
    game.apply(Action::Choose { index: 1 }).expect("opening choice");
    for _ in 0..900 {
        if game.stage == Stage::Lost { break; }
        match game.stage {
            Stage::Combat => { game.apply(Action::EndTurn).expect("legal passive turn"); }
            Stage::Map => {
                let index = game.route.iter().position(|node| matches!(node, Node::Battle | Node::Elite | Node::Boss))
                    .or_else(|| (!game.route.is_empty()).then_some(0))
                    .expect("legal onward route");
                game.apply(Action::Choose { index: index + 1 }).expect("legal route");
            }
            Stage::Won => panic!("passive defeat policy unexpectedly won"),
            _ => advance(&mut game).expect("legal noncombat choice"),
        }
    }
    assert_eq!(game.stage, Stage::Lost, "passive natural campaign must settle as defeat");
    game.audit_campaign_authority().expect("defeat authority audit");
    game.verify_action_replay().expect("defeat replay");
    let snapshot = game.to_json().expect("defeat snapshot");
    let restored = Game::from_json(&snapshot).expect("defeat restore");
    assert_eq!(restored.to_json().unwrap(), snapshot);
    restored.verify_action_replay().expect("restored defeat replay");
    let mut profile = Profile::new();
    assert!(profile.record(&game, game.run_id.clone()).expect("record defeat"));
    let persisted = profile.to_json().expect("persist chronicle");
    let mut reloaded = Profile::from_json(&persisted).expect("restore chronicle");
    assert!(!reloaded.record(&restored, game.run_id.clone()).expect("duplicate defeat"));
    assert_eq!(reloaded.wins, 0);
    assert_eq!(reloaded.losses, 1);
    assert_eq!(reloaded.records.len(), 1);
    assert!(reloaded.difficulty_clears.is_empty());
    assert!(reloaded.ascensions.is_empty());
    assert_eq!(reloaded.to_json().unwrap(), persisted);
    assert!(game.apply(Action::EndTurn).is_err());
    assert_eq!(game.to_json().unwrap(), snapshot);
}


// Natural player choices, not injected lore flags, must carry the Wuchao plan
// from an act-II Story node to the first actual battle in act III.
#[test]
fn natural_cao_cao_wuchao_raid_changes_a_later_battle_once() {
    let mut attempts = Vec::new();
    for seed in 247_510..247_526 {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::CaoCao, seed, Difficulty::Warlords, format!("wuchao-{seed}"),
        );
        game.apply(Action::Choose { index: 1 }).expect("natural opening");
        let mut planned = false;
        for _ in 0..900 {
            if game.stage == Stage::Story && game.floor >= 9 && game.floor <= 16
                && game.lore_hint().contains("4. 召开乌巢军议")
                && game.gold >= 20
            {
                game.apply(Action::Choose { index: 4 }).expect("enter council");
                assert_eq!(game.wuchao_plan, Some(WuchaoPlan::Council));
                let before_invalid = game.to_json().unwrap();
                assert!(game.apply(Action::Choose { index: 99 }).is_err());
                assert_eq!(game.to_json().unwrap(), before_invalid);
                let before_gold = game.gold;
                game.apply(Action::Choose { index: 1 }).expect("fund raid");
                assert_eq!(game.gold, before_gold - 20);
                assert_eq!(game.wuchao_plan, Some(WuchaoPlan::RaidPending));
                planned = true;
            } else if planned && game.stage == Stage::Combat && game.floor >= 17 {
                assert_eq!(game.wuchao_plan, Some(WuchaoPlan::RaidResolved));
                let battle = game.battle.as_ref().unwrap();
                assert_eq!(battle.vulnerable, 1);
                assert_eq!(battle.foes[0].block, 0);
                assert!(battle.foes[0].hp < battle.foes[0].max_hp);
                game.audit_campaign_authority().expect("wuchao campaign audit");
                game.verify_action_replay().expect("wuchao natural replay");
                let saved = game.to_json().expect("cross-act save");
                let loaded = Game::from_json(&saved).expect("cross-act restore");
                assert_eq!(loaded.to_json().unwrap(), saved);
                loaded.verify_action_replay().expect("restored wuchao replay");
                return;
            } else if game.stage == Stage::Lost || game.stage == Stage::Won {
                break;
            } else if let Err(error) = advance(&mut game) {
                attempts.push(format!("seed {seed}: floor {} {:?}: {error}", game.floor, game.stage));
                break;
            }
        }
        attempts.push(format!("seed {seed}: floor {} {:?}, planned={planned}", game.floor, game.stage));
    }
    panic!("no natural Cao Cao Wuchao council -> act-III battle: {}", attempts.join(" | "));
}
