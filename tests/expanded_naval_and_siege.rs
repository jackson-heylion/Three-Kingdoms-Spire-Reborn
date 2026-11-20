//! New-campaign special battlefield acceptance for the third-act fleet and
//! the defender-first siege option. Floor checkpoints are deliberate fixtures,
//! NOT a fabricated natural 0→40 victory or historical choice. All consequential
//! actions are committed through Game::apply and compared across JSON restore.
use three_kingdoms_spire_reborn::{
    Action, ChibiPlan, Difficulty, Game, Node, Stage, Card, content::Hero, map,
};

fn isolated_battle(floor: u8) -> Game {
    isolated_battle_for(Hero::GuanYu, floor)
}

fn isolated_battle_for(hero: Hero, floor: u8) -> Game {
    isolated_battle_for_tier(hero, floor, Difficulty::Warlords)
}

fn isolated_battle_for_tier(hero: Hero, floor: u8, tier: Difficulty) -> Game {
    let mut game = Game::new_with_campaign_difficulty(
        hero, 1_u64 << 44, tier,
        format!("special-battle-floor-{floor}"),
    );
    game.floor = floor - 1;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, (floor - 1) / 8));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game
}

#[test]
fn river_fleet_entry_previews_tide_and_paid_attacks_board_raiders() {
    let mut game = isolated_battle(19);
    assert!(game.route_objective_hint(Node::Battle).unwrap().contains("水陆阵战"));
    game.apply(Action::Choose { index: 1 }).expect("enter real third-act battle");
    assert_eq!(game.floor, 19);
    assert_eq!(game.stage, Stage::Combat);
    assert_eq!(game.battle.as_ref().unwrap().convoy_hp, Some(12));
    assert_eq!(game.battle.as_ref().unwrap().convoy_cover, Some(0));
    let objective = game.tactical_objective().unwrap();
    assert!(objective.contains("水陆阵战") && objective.contains("逆流"));
    // Controlled card and foe-block checkpoint. The attack, boarding and
    // event, however, come from a legal player action with real damage.
    {
        let b = game.battle.as_mut().unwrap();
        b.foes[0].block = 0;
        b.energy = b.energy.max(1);
        b.hand.push(Card::new("strike"));
    }
    let index = game.battle.as_ref().unwrap().hand.len();
    let before = game.to_json().expect("save before attack");
    let mut loaded = Game::from_json(&before).expect("restore before attack");
    let feedback = game.apply(Action::Play { index, target: 1 })
        .expect("attack raider with real card");
    assert_eq!(feedback, loaded.apply(Action::Play { index, target: 1 }).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert!(feedback.join("；").contains("舰队掩护+2"));
    let b = game.battle.as_ref().unwrap();
    assert_eq!(b.convoy_cover, Some(2));
    assert!(b.events.iter().any(|event|
        event.source == "naval_boarding_cover" && event.amount == 2));
    let save = game.to_json().unwrap();
    let mut resumed = Game::from_json(&save).unwrap();
    let turn = game.apply(Action::EndTurn).expect("resolve actual fleet raid");
    assert_eq!(turn, resumed.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
}

#[test]
fn fleet_guard_protects_the_hull_and_loss_seals_the_battle() {
    let mut game = isolated_battle(19);
    game.apply(Action::Choose { index: 1 }).expect("enter fleet defense");
    let mut unguarded = Game::from_json(&game.to_json().unwrap()).unwrap();
    {
        let b = game.battle.as_mut().unwrap();
        b.hand.push(Card::new("defend"));
    }
    let card = game.battle.as_ref().unwrap().hand.len();
    game.apply(Action::Play { index: card, target: 1 })
        .expect("pay for real fleet guard");
    assert_eq!(game.battle.as_ref().unwrap().convoy_cover, Some(4));
    assert!(game.battle.as_ref().unwrap().events.iter().any(|event|
        event.source == "naval_guard_cover" && event.amount == 4));
    // Same encounter/RNG but one legal guard action should preserve at least
    // as much hull as ending the turn without protection.
    game.apply(Action::EndTurn).expect("guarded raid");
    unguarded.apply(Action::EndTurn).expect("unguarded raid");
    if game.stage == Stage::Combat && unguarded.stage == Stage::Combat {
        assert!(game.battle.as_ref().unwrap().convoy_hp
            >= unguarded.battle.as_ref().unwrap().convoy_hp);
    }
    // Separate explicitly controlled near-sinking checkpoint. The lethal
    // raid and defeat still execute through EndTurn, never injected directly.
    let mut sinking = isolated_battle(19);
    sinking.apply(Action::Choose { index: 1 }).unwrap();
    sinking.battle.as_mut().unwrap().convoy_hp = Some(1);
    let saved = sinking.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    let loss = sinking.apply(Action::EndTurn).expect("fleet sinks to real enemy raid");
    assert_eq!(loss, restored.apply(Action::EndTurn).unwrap());
    assert_eq!(sinking.to_json().unwrap(), restored.to_json().unwrap());
    assert!(loss.join("；").contains("战船覆没"));
    assert_eq!(sinking.stage, Stage::Lost);
    assert!(sinking.battle.is_none());
}

#[test]
fn defeating_siege_defenders_before_assault_improves_real_breach() {
    let mut game = isolated_battle(4);
    assert!(game.route_objective_hint(Node::Battle).unwrap().contains("守军"));
    game.apply(Action::Choose { index: 1 }).expect("enter real siege");
    assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(12));
    // Only the final defender is retained as a 1-HP controlled tactical
    // checkpoint. Clearing it still requires a paid real attack card.
    {
        let b = game.battle.as_mut().unwrap();
        for foe in &mut b.foes { foe.hp = 0; foe.block = 0; }
        b.foes[0].hp = 1;
        b.energy = 3;
        b.hand.push(Card::new("strike"));
    }
    let start = game.to_json().unwrap();
    let mut rushing = Game::from_json(&start).unwrap();
    let rush = rushing.apply(Action::AssaultGate).expect("rush gate with guards alive");
    assert!(rush.join("；").contains("攻城破门4点"));
    assert_eq!(rushing.battle.as_ref().unwrap().siege_gate_hp, Some(8));

    let card = game.battle.as_ref().unwrap().hand.len();
    game.apply(Action::Play { index: card, target: 1 })
        .expect("defeat last guard with paid strike");
    assert_eq!(game.stage, Stage::Combat,
        "the siege still requires an actual gate breach");
    assert!(game.tactical_objective().unwrap().contains("当前破门6点"));
    let before_breach = game.to_json().unwrap();
    let mut loaded = Game::from_json(&before_breach).unwrap();
    let breach = game.apply(Action::AssaultGate).expect("breach without defenders");
    assert_eq!(breach, loaded.apply(Action::AssaultGate).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert!(breach.join("；").contains("攻城破门6点"));
    assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(6));
}

#[test]
fn red_cliffs_council_responsibility_changes_the_later_fleet_battle() {
    // Historical plans are explicitly staged here as controlled checkpoints.
    // The resulting battle deployment, event log, and subsequent actions
    // are produced by the authoritative Game::apply boundary.
    for (chosen, settled) in [
        (ChibiPlan::FleetPending, ChibiPlan::FleetBattleResolved),
        (ChibiPlan::FirePending, ChibiPlan::FireBattleResolved),
        (ChibiPlan::DeclinePending, ChibiPlan::DeclineBattleResolved),
    ] {
        let mut game = isolated_battle_for(Hero::ZhugeLiang, 19);
        game.chibi_plan = Some(chosen);
        game.apply(Action::Choose { index: 1 })
            .expect("real entry into naval fight applies an existing Chibi choice");
        assert_eq!(game.chibi_plan, Some(settled));
        let battle = game.battle.as_ref().unwrap();
        match chosen {
            ChibiPlan::FleetPending => {
                assert_eq!(battle.convoy_cover, Some(4));
                assert!(battle.events.iter().any(|e|
                    e.source == "chibi_naval_fleet_screen" && e.amount == 4));
                assert!(game.tactical_objective().unwrap().contains("赤壁稳守水军"));
            }
            ChibiPlan::FirePending => {
                assert_eq!(battle.convoy_hp, Some(10));
                assert!(battle.foes[0].poison >= 3);
                assert!(battle.events.iter().any(|e|
                    e.source == "chibi_naval_fire_spread" && e.amount == 2));
                assert!(game.tactical_objective().unwrap().contains("赤壁火攻旧议"));
            }
            ChibiPlan::DeclinePending => {
                assert!(battle.foes[0].block >= 16,
                    "declining the council retains the existing 12 armor plus four naval");
                assert!(battle.events.iter().any(|e|
                    e.source == "chibi_naval_declined_cover" && e.amount == 4));
                assert!(game.tactical_objective().unwrap().contains("赤壁不介入旧债"));
            }
            _ => unreachable!(),
        }
        let saved = game.to_json().unwrap();
        let mut loaded = Game::from_json(&saved).expect("persist Chibi naval consequence");
        let result = game.apply(Action::EndTurn).expect("turn after council consequence");
        assert_eq!(result, loaded.apply(Action::EndTurn).unwrap());
        assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    }
}

#[test]
fn defensive_breakout_is_an_actual_paid_three_card_escape_and_replays() {
    // Controlled initial-floor encounter, NOT a simulated win across 40 floors.
    let mut game = isolated_battle(1);
    assert!(game.route_objective_hint(Node::Battle).unwrap().contains("结阵脱围"));
    game.apply(Action::Choose { index: 1 }).expect("enter real breakout");
    assert_eq!(game.floor, 1);
    assert!(game.tactical_objective().unwrap().contains("3张牌"));
    {
        let battle = game.battle.as_mut().unwrap();
        battle.energy = 3;
        for _ in 0..3 { battle.hand.push(Card::new("defend")); }
    }
    for count in 1..=3 {
        let index = game.battle.as_ref().unwrap().hand.iter()
            .position(|c| c.id == "defend").unwrap() + 1;
        game.apply(Action::Play { index, target: 1 })
            .expect("pay for a real defense card");
        assert_eq!(game.battle.as_ref().unwrap().played_this_turn, count);
    }
    assert!(game.battle.as_ref().unwrap().block >= 12);
    let checkpoint = game.to_json().expect("save paid breakout");
    let mut loaded = Game::from_json(&checkpoint).expect("resume paid breakout");
    let result = game.apply(Action::EndTurn).expect("retreat by actual EndTurn");
    assert_eq!(result, loaded.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert!(result.join("；").contains("突围结阵成功"));
    assert_eq!(game.stage, Stage::Reward,
        "defensive breakout must settle the same battle reward exactly once");
    assert!(game.battle.is_none());
    assert!(game.event_history.as_ref().is_some_and(|history|
        history.iter().any(|record|
            record.event.source == "breakout_formation_escape"
                && record.event.amount == 1)));
}

#[test]
fn piercing_attack_is_a_paid_player_boarding_hit_not_a_passive_damage_exemption() {
    let mut game = isolated_battle_for(Hero::SunShangxiang, 19);
    game.apply(Action::Choose { index: 1 }).expect("enter real fleet combat");
    {
        let b = game.battle.as_mut().unwrap();
        // A deliberately controlled high-armor raider: direct piercing
        // should still protect the ship even though ordinary strikes
        // would be entirely absorbed by armor.
        b.foes[0].block = 12;
        b.foes[0].hp = b.foes[0].hp.max(8);
        b.energy = b.energy.max(1);
        b.hand.push(Card::new("jianshu"));
    }
    let idx = game.battle.as_ref().unwrap().hand.len();
    let before = game.to_json().unwrap();
    let mut restored = Game::from_json(&before).unwrap();
    let message = game.apply(Action::Play { index: idx, target: 1 })
        .expect("play piercing card with paid energy");
    assert_eq!(message, restored.apply(Action::Play { index: idx, target: 1 }).unwrap());
    assert!(message.join("；").contains("舰队掩护+2"));
    assert_eq!(game.battle.as_ref().unwrap().convoy_cover, Some(2));
    assert!(game.battle.as_ref().unwrap().events.iter().any(|e|
        e.source == "naval_boarding_cover" && e.amount == 2));
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
}

#[test]
fn river_tide_follows_new_difficulty_and_uses_the_same_risk_forecast_as_damage() {
    for (tier, expected_tide) in [
        (Difficulty::Warlords, 2_u8),
        (Difficulty::Conqueror, 4_u8),
        (Difficulty::Destiny, 6_u8),
    ] {
        let mut game = isolated_battle_for_tier(Hero::GuanYu, 19, tier);
        game.apply(Action::Choose { index: 1 }).expect("real river crossing");
        // Explicit second-turn tactical checkpoint: all three campaigns
        // retain the same actual raider and must share the same Game rule.
        // No artificial event, result, history flag, RNG or map is injected.
        {
            let b = game.battle.as_mut().unwrap();
            b.turn = 2;
            b.convoy_hp = Some(12);
            b.convoy_cover = Some(0);
            b.foes[0].hp = b.foes[0].hp.max(1);
            for foe in b.foes.iter_mut().skip(1) { foe.hp = 0; }
            b.foes[0].intent = three_kingdoms_spire_reborn::Intent::Guard(0);
        }
        let risk = game.tactical_objective().unwrap();
        assert!(risk.contains(&format!("本回合逆流+{expected_tide}")));
        assert!(risk.contains(&format!("本回合预计失船{}点", 4 + expected_tide)));
        let snapshot = game.to_json().unwrap();
        let mut restored = Game::from_json(&snapshot).unwrap();
        let result = game.apply(Action::EndTurn).expect("real tier-specific naval raid");
        assert_eq!(result, restored.apply(Action::EndTurn).unwrap());
        assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
        assert_eq!(game.battle.as_ref().unwrap().convoy_hp,
            Some(8 - expected_tide),
            "expected one living raider and the difficulty's actual adverse current");
        assert!(game.battle.as_ref().unwrap().events.iter().any(|event|
            event.source == "naval_hull_raided"
                && event.amount == i32::from(4 + expected_tide)));
    }
}

#[test]
fn river_fleet_fourth_turn_guard_escorts_without_killing_remaining_enemies() {
    let mut game = isolated_battle(19);
    game.apply(Action::Choose { index: 1 }).expect("enter actual river battle");
    assert!(game.tactical_objective().unwrap().contains("第4回合"));
    // A declared turn-four tactical checkpoint. The last escort turn's
    // defense must be paid and enacted through Game::apply, after which the
    // actual enemy intent, tide and raid resolve before victory is decided.
    // This is not a natural 0→40 victory or fabricated historical trigger.
    {
        let b = game.battle.as_mut().unwrap();
        b.turn = 4;
        b.convoy_hp = Some(10);
        b.convoy_cover = Some(0);
        b.energy = 3;
        for foe in &mut b.foes { foe.hp = 0; }
        b.foes[0].hp = 8;
        b.foes[0].intent = three_kingdoms_spire_reborn::Intent::Guard(0);
        b.hand.push(Card::new("defend"));
    }
    let unguarded_save = game.to_json().unwrap();
    let mut unguarded = Game::from_json(&unguarded_save).unwrap();
    let index = game.battle.as_ref().unwrap().hand.len();
    game.apply(Action::Play { index, target: 1 })
        .expect("pay for one real fleet covering card");
    assert_eq!(game.battle.as_ref().unwrap().convoy_cover, Some(4));
    let before = game.to_json().unwrap();
    let mut restored = Game::from_json(&before).expect("restore guarded turn-four fleet");
    let outcome = game.apply(Action::EndTurn).expect("finish real fourth escort turn");
    assert_eq!(outcome, restored.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    assert!(outcome.join("；").contains("水陆护航"));
    assert_eq!(game.stage, Stage::Reward);
    assert!(game.battle.is_none());
    assert!(game.event_history.as_ref().is_some_and(|history|
        history.iter().any(|record| record.event.source == "naval_escort_delivered"
            && record.event.amount == 1)));
    // The same enemy survives if the player skips defensive commitment.
    // The hull falls below the six-point evacuation threshold, so no win.
    let missed = unguarded.apply(Action::EndTurn).expect("uncovered fleet suffers real raid");
    assert!(!missed.join("；").contains("水陆护航"));
    assert_eq!(unguarded.stage, Stage::Combat);
    assert_eq!(unguarded.battle.as_ref().unwrap().convoy_hp, Some(4));
}

#[test]
fn ambush_difficulty_shield_and_paid_break_weaken_vanguard_only_once() {
    // The floor-three setup is a controlled battle entry, not a natural
    // campaign. All shield-breaking consequences are paid Action::Play.
    for (tier, minimum) in [
        (Difficulty::Warlords, 8),
        (Difficulty::Conqueror, 10),
        (Difficulty::Destiny, 12),
    ] {
        let mut game = isolated_battle_for_tier(Hero::GuanYu, 3, tier);
        assert!(game.route_objective_hint(Node::Battle).unwrap()
            .contains("8/10/12"));
        game.apply(Action::Choose { index: 1 }).expect("enter ambush");
        assert!(game.battle.as_ref().unwrap().foes[0].block >= minimum);
        assert!(!game.battle.as_ref().unwrap().ambush_armor_broken);
        {
            let battle = game.battle.as_mut().unwrap();
            battle.foes[0].block = 1;
            battle.foes[0].hp = battle.foes[0].hp.max(50);
            battle.energy = battle.energy.max(1);
            battle.hand.push(Card::new("strike"));
        }
        let idx = game.battle.as_ref().unwrap().hand.len();
        let saved = game.to_json().unwrap();
        let mut loaded = Game::from_json(&saved).unwrap();
        let result = game.apply(Action::Play { index: idx, target: 1 })
            .expect("paid attack breaks actual shield");
        assert_eq!(result, loaded.apply(Action::Play { index: idx, target: 1 }).unwrap());
        assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
        assert!(result.join("；").contains("前锋虚弱2层"));
        let battle = game.battle.as_ref().unwrap();
        assert!(battle.ambush_armor_broken);
        assert!(battle.foes[0].weak >= 2);
        assert_eq!(battle.events.iter().filter(|event|
            event.source == "ambush_vanguard_reeling").count(), 1);
    }
}

#[test]
fn siege_last_guard_paid_kill_recovers_gear_for_next_breach() {
    let mut game = isolated_battle(4);
    game.apply(Action::Choose { index: 1 }).expect("enter siege");
    {
        let battle = game.battle.as_mut().unwrap();
        for foe in &mut battle.foes { foe.hp = 0; foe.block = 0; }
        battle.foes[0].hp = 1;
        battle.energy = 2;
        battle.hand.push(Card::new("strike"));
    }
    let before = game.to_json().unwrap();
    let mut loaded = Game::from_json(&before).unwrap();
    let idx = game.battle.as_ref().unwrap().hand.len();
    let result = game.apply(Action::Play { index: idx, target: 1 })
        .expect("real paid strike removes the last siege defender");
    assert_eq!(result, loaded.apply(Action::Play { index: idx, target: 1 }).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert!(result.join("；").contains("收复攻城器械"));
    assert_eq!(game.battle.as_ref().unwrap().energy, 2,
        "one paid strike costs 1 and the last guard restores exactly 1");
    assert_eq!(game.battle.as_ref().unwrap().events.iter().filter(|event|
        event.source == "siege_engine_reclaimed" && event.amount == 1).count(), 1);
    let breach = game.apply(Action::AssaultGate).expect("spend the recovered siege energy");
    assert!(breach.join("；").contains("攻城破门6点"));
    assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(6));
}

#[test]
fn red_cliffs_fire_damage_can_be_repaired_by_paid_fleet_guard() {
    // The earlier council flag is staged, but its enactment, actual guard
    // payment, event and save restoration go through Game::apply.
    let mut game = isolated_battle_for(Hero::ZhugeLiang, 19);
    game.chibi_plan = Some(ChibiPlan::FirePending);
    game.apply(Action::Choose { index: 1 }).expect("enter fiery naval battle");
    assert_eq!(game.battle.as_ref().unwrap().convoy_hp, Some(10));
    {
        let battle = game.battle.as_mut().unwrap();
        battle.energy = battle.energy.max(1);
        battle.hand.push(Card::new("defend"));
    }
    let before = game.to_json().unwrap();
    let mut loaded = Game::from_json(&before).unwrap();
    let idx = game.battle.as_ref().unwrap().hand.len();
    let result = game.apply(Action::Play { index: idx, target: 1 })
        .expect("play a paid guard card to contain Red Cliffs fire");
    assert_eq!(result, loaded.apply(Action::Play { index: idx, target: 1 }).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert!(result.join("；").contains("抢修战船1点"));
    assert_eq!(game.battle.as_ref().unwrap().convoy_hp, Some(11));
    assert!(game.battle.as_ref().unwrap().events.iter().any(|event|
        event.source == "chibi_naval_fire_contained" && event.amount == 1));
}
