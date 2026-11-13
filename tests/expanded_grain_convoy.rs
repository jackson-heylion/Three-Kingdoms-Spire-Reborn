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
    assert!(warning.contains("本回合预计劫粮4点"), "forecast must expose net convoy loss before committing: {warning}");
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
    assert!(game.tactical_objective().unwrap().contains("本回合预计劫粮2点"),
        "a played defense must reduce the visible incoming raid");
    let current = game.to_json().unwrap();
    assert_eq!(Game::from_json(&current).unwrap().to_json().unwrap(), current);
    // Already-maxed cargo cover must not emit a fictional extra +2 event.
    game.battle.as_mut().unwrap().convoy_cover = Some(8);
    game.battle.as_mut().unwrap().hand.insert(0,
        three_kingdoms_spire_reborn::Card::new("defend"));
    let audited = game.event_history.as_ref().unwrap().iter()
        .filter(|entry| entry.event.source == "convoy_cover").count();
    game.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert_eq!(game.battle.as_ref().unwrap().convoy_cover, Some(8));
    assert_eq!(game.event_history.as_ref().unwrap().iter()
        .filter(|entry| entry.event.source == "convoy_cover").count(), audited);

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
    assert!(game.gold > before_gold, "successful defense pays battle and cargo rewards");
    assert!(game.log.last().unwrap().contains("保住粮车12/12，额外获得24金"),
        "preserving cargo needs an explicit bonus: {:?}", game.log.last());
    assert!(game.battle.is_none(), "no convoy persists into reward");
    let before_invalid = game.to_json().unwrap();
    assert!(game.apply(Action::EndTurn).is_err(), "no replaying victory as a turn");
    assert_eq!(game.to_json().unwrap(), before_invalid);
    let rewarded_gold = game.gold;
    game.apply(Action::Choose { index: 0 }).expect("claim reward exactly once");
    assert_eq!(game.gold, rewarded_gold);
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}

#[test]
fn a_real_seed_can_reach_grain_convoy_without_mutating_initial_state() {
    // Entire trajectory before floor two is real player actions. No forced
    // floor, map, hand, enemy HP, lore flag, or scripted reward is injected.
    for seed in 247_310..247_510 {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::LuBu, seed, Difficulty::Warlords, format!("natural-convoy-{seed}"),
        );
        game.apply(Action::Choose { index: 1 }).expect("start natural breakout");
        for _ in 0..4 {
            while game.stage == Stage::Combat {
                let choice = game.battle.as_ref().and_then(|battle| {
                    battle.hand.iter().enumerate().find(|(_, card)| {
                        (card.id == "strike" || card.id == "wushuang")
                            && three_kingdoms_spire_reborn::content::card_spec(&card.id)
                                .is_some_and(|spec| spec.cost <= battle.energy)
                    }).map(|(index, _)| index + 1)
                });
                let Some(index) = choice else { break };
                game.apply(Action::Play { index, target: 1 }).expect("normal attack");
            }
            if game.stage != Stage::Combat { break; }
            game.apply(Action::EndTurn).expect("normal enemy turn");
        }
        if game.stage != Stage::Reward { continue; }
        game.apply(Action::Choose { index: 0 }).expect("decline card reward");
        if game.stage != Stage::Map { continue; }
        let Some(index) = game.route.iter().position(|node| *node == Node::Battle)
        else { continue };
        let notice = game.route_objective_hint(Node::Battle).expect("natural route preview");
        assert!(notice.contains("粮道保卫"));
        game.apply(Action::Choose { index: index + 1 }).expect("choose actual convoy lane");
        assert_eq!(game.floor, 2);
        assert_eq!(game.stage, Stage::Combat);
        assert!(game.tactical_objective().unwrap().contains("粮道保卫"));
        assert_eq!(game.battle.as_ref().unwrap().convoy_hp, Some(12));
        game.verify_action_replay().expect("natural path through real actions");
        let json = game.to_json().expect("natural convoy save");
        let restored = Game::from_json(&json).expect("natural convoy JSON restore");
        restored.verify_action_replay().expect("restored natural convoy action replay");
        assert_eq!(restored.to_json().unwrap(), json);
        // Continue the natural encounter through an actual enemy turn.
        // No enemy, map, hand, or RNG fixture is injected.
        let initial_cargo = game.battle.as_ref().unwrap().convoy_hp.unwrap();
        game.apply(Action::EndTurn).expect("natural convoy enemy turn");
        let remaining_cargo = game.battle.as_ref().unwrap().convoy_hp.unwrap();
        assert!(remaining_cargo < initial_cargo,
            "an unguarded natural enemy turn must raid the convoy: {initial_cargo} -> {remaining_cargo}");
        let after_raid = game.to_json().expect("save after natural convoy raid");
        let recovered = Game::from_json(&after_raid).expect("restore natural raid");
        assert_eq!(recovered.to_json().unwrap(), after_raid);
        game.verify_action_replay().expect("natural raid action replay");
        recovered.verify_action_replay().expect("restored natural raid replay");
        // Continue the same unmodified, naturally reached encounter to an
        // actual terminal outcome. This is not the controlled HP/intent fixture
        // used by the isolated convoy failure test above.
        let gold_before_terminal = game.gold;
        for _ in 0..20 {
            if game.stage != Stage::Combat {
                break;
            }
            game.apply(Action::EndTurn).expect("natural convoy enemy round");
        }
        assert_eq!(game.stage, Stage::Lost,
            "repeatedly abandoning a natural convoy must eventually end the encounter");
        assert!(game.log.iter().any(|line| line.contains("粮道保卫失败")),
            "natural defeat must be caused by losing the convoy objective, not merely the hero dying: {:?}", game.log);
        assert_eq!(game.gold, gold_before_terminal,
            "a naturally lost encounter must never grant convoy victory rewards");
        let terminal = game.to_json().expect("save natural terminal outcome");
        let restored_terminal = Game::from_json(&terminal).expect("restore natural terminal");
        assert_eq!(restored_terminal.to_json().unwrap(), terminal);
        game.verify_action_replay().expect("natural terminal action replay");
        restored_terminal.verify_action_replay().expect("restored terminal action replay");
        return;
    }
    panic!("no natural seeded floor-two convoy route found within 200 Lu Bu seeds");
}

#[test]
fn convoy_lethal_raid_is_warned_and_defense_changes_the_warning() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 247_523, Difficulty::Warlords, "convoy-lethal-247523".into(),
    );
    game.floor = 1;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 0));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).unwrap();
    // A low-cargo checkpoint isolates the actionable danger announcement.
    game.battle.as_mut().unwrap().convoy_hp = Some(4);
    let before = game.to_json().unwrap();
    let warning = game.tactical_objective().unwrap();
    assert!(warning.contains("本回合预计劫粮4点"));
    assert!(warning.contains("本回合粮车将被毁"), "{warning}");
    assert_eq!(game.to_json().unwrap(), before, "risk inspection is read-only");

    game.battle.as_mut().unwrap().hand.insert(0,
        three_kingdoms_spire_reborn::Card::new("defend"));
    game.apply(Action::Play { index: 1, target: 1 }).unwrap();
    let countered = game.tactical_objective().unwrap();
    assert!(countered.contains("本回合预计劫粮2点"), "{countered}");
    assert!(!countered.contains("本回合粮车将被毁"), "{countered}");
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}
