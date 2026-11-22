//! New-campaign act V alternate boss: Deng Ai's telegraphed Yinping infiltration.
//! A deliberately controlled floor-39 entrance, NOT a natural floor-0 victory.
use three_kingdoms_spire_reborn::{
    Action, BossMove, Difficulty, Game, Intent, Node, Stage, content::Hero, map,
};

fn fifth_act_alternate_boss() -> Game {
    // Seed bit 44 selects the alternate fifth-act commander. Starting at
    // floor 39 isolates the encounter without forging a full natural run.
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 1_u64 << 44, Difficulty::Warlords,
        "yinping-alternate".into(),
    );
    game.floor = 39;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    game
}

#[test]
fn alternate_fifth_act_boss_exposes_a_replayable_casualty_free_supply_raid() {
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter alternate final boss");
    assert_eq!(game.floor, 40);
    assert_eq!(game.stage, Stage::Combat);
    let foe = &game.battle.as_ref().unwrap().foes[0];
    assert_eq!(foe.id, "dengai");
    assert!(matches!(foe.intent, Intent::Boss(BossMove::YinpingFlank(0))));
    let preview = foe.intent.label();
    assert!(preview.contains("阴平偷渡") && preview.contains("至少3张牌") && preview.contains("10金"), "{preview}");
    let before = game.to_json().expect("entrance save");
    let mut loaded = Game::from_json(&before).expect("restore visible intent");
    let original_gold = game.gold;
    let original = game.apply(Action::EndTurn).expect("unopposed enemy action");
    let resumed = loaded.apply(Action::EndTurn).expect("same action after restore");
    assert_eq!(original, resumed);
    assert_eq!(game.gold, original_gold - 10);
    assert!(original.join("；").contains("阴平偷渡得手"));
    assert!(game.battle.as_ref().unwrap().foes[0].block >= 6);
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
}

#[test]
fn three_real_play_actions_intercept_yinping_raid_and_preserve_resources() {
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter alternate final boss");
    // A normal starting draw contains at least four basic cards; no hand
    // injection or hero block mutation is permitted to earn this counter.
    for _ in 0..3 {
        let index = game.battle.as_ref().unwrap().hand.iter()
            .position(|card| card.id == "strike" || card.id == "defend")
            .expect("starter draw must contain a basic playable card") + 1;
        game.apply(Action::Play { index, target: 1 }).expect("actual legal card play");
    }
    assert_eq!(game.battle.as_ref().unwrap().played_this_turn, 3);
    let start_gold = game.gold;
    let hp_before_counter = game.battle.as_ref().unwrap().foes[0].hp;
    let snapshot = game.to_json().expect("save after three plays");
    let mut loaded = Game::from_json(&snapshot).expect("restore counterplay");
    let result = game.apply(Action::EndTurn).expect("counter infiltration");
    let replayed = loaded.apply(Action::EndTurn).expect("restored counter");
    assert_eq!(result, replayed);
    assert!(result.join("；").contains("阴平偷渡被截击"));
    assert!(result.join("；").contains("反击造成4点无视护甲伤害"));
    assert_eq!(game.battle.as_ref().unwrap().foes[0].hp, hp_before_counter - 4);
    assert!(game.battle.as_ref().unwrap().events.iter().any(|event|
        event.source == "yinping_interception" && event.amount == 4
            && event.kind == three_kingdoms_spire_reborn::combat::EventKind::DamageDealt));
    assert_eq!(game.gold, start_gold);
    assert!(game.battle.as_ref().unwrap().foes[0].vulnerable >= 2);
    assert_eq!(game.battle.as_ref().unwrap().foes[0].weak, 0,
        "card-count interception must not receive the defense-only reward");
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
}


#[test]
fn four_paid_cards_expose_a_real_summoned_raider_for_followup_attack() {
    // Controlled phase-two setup only; the escort itself is summoned by
    // the real boss action, and the player must pay for four legal cards.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter fifth-act boss");
    {
        let b = game.battle.as_mut().unwrap();
        b.foes[0].phase = 1;
        b.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("boss summons escort");
    assert_eq!(game.battle.as_ref().unwrap().foes[1].id, "assassin");
    {
        let b = game.battle.as_mut().unwrap();
        b.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
        b.energy = 4;
        // Explicit controlled armor checkpoint: verify the new offensive
        // shield-break independently of the naturally summoned foe's HP.
        // This is not presented as a natural floor-zero journey.
        b.foes[1].block = 6;
        for _ in 0..4 {
            b.hand.push(three_kingdoms_spire_reborn::Card::new("defend"));
        }
    }
    for card_number in 1..=4 {
        let index = game.battle.as_ref().unwrap().hand.iter()
            .position(|card| card.id == "defend").unwrap() + 1;
        game.apply(Action::Play { index, target: 1 })
            .expect("paid defensive action");
        if card_number < 4 {
            assert_eq!(game.battle.as_ref().unwrap().foes[1].block, 6,
                "escort cover must remain until the fourth paid card");
        }
    }
    assert_eq!(game.battle.as_ref().unwrap().played_this_turn, 4);
    assert_eq!(game.battle.as_ref().unwrap().foes[1].block, 0,
        "the fourth card must break cover immediately, while another attack is possible");
    let before = game.to_json().unwrap();
    let mut restored = Game::from_json(&before).unwrap();
    let report = game.apply(Action::EndTurn).unwrap();
    assert_eq!(report, restored.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    assert!(report.join("；").contains("连环出牌识破山道援军"), "{report:?}");
    let battle = game.battle.as_ref().unwrap();
    assert!(battle.foes[1].vulnerable > 0);
    assert_eq!(battle.foes[1].block, 0,
        "four-card interception must strip the escort's six-point shield");
    assert!(report.join("；").contains("击破现有护甲0"), "{report:?}");
    // The six armor was broken by the fourth real card BEFORE EndTurn,
    // not falsely attributed to an enemy-turn reset. The event survives
    // the save/resume boundary above.
    assert!(battle.events.iter().any(|e|
        e.source == "yinping_raider_armor_broken" && e.amount == 6
            && e.kind == three_kingdoms_spire_reborn::combat::EventKind::PowerTriggered));
    assert!(battle.events.iter().any(|e|
        e.source == "yinping_raider_exposed"
            && e.kind == three_kingdoms_spire_reborn::combat::EventKind::PowerTriggered));
    assert_eq!(battle.foes[1].weak, 0,
        "four-card offense must not inherit defense-only suppression");
}

#[test]
fn second_phase_unchecked_yinping_calls_one_telegraphed_raider_after_save_restore() {
    // This is an isolated phase-two tactical fixture, NOT evidence that a
    // natural 0-to-40 campaign can reach the second phase or defeat it.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter fifth-act boss");
    {
        let b = game.battle.as_mut().expect("controlled encounter");
        b.foes[0].phase = 1;
        b.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    let preview = game.battle.as_ref().unwrap().foes[0].intent.label();
    assert!(preview.contains("至少4张牌") && preview.contains("护甲达到12"));
    assert!(preview.contains("首次失守还会召集援兵"), "{preview}");

    let entry = game.to_json().expect("phase-two tactical save");
    let mut resumed = Game::from_json(&entry).expect("resume phase-two tactical fixture");
    let gold_before = game.gold;
    let messages = game.apply(Action::EndTurn).expect("let the telegraphed raid land");
    let resumed_messages = resumed.apply(Action::EndTurn).expect("replay raid after resume");
    assert_eq!(messages, resumed_messages);
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
    assert_eq!(game.gold, gold_before - 15);
    assert!(messages.join("；").contains("山道援兵已出现（下回合才行动）"));
    let battle = game.battle.as_ref().expect("raider cannot act before its first preview");
    assert_eq!(battle.foes.len(), 2);
    assert_eq!(battle.foes[1].id, "assassin");
    assert!(battle.foes[1].hp > 0);
    let summons: Vec<_> = battle.events.iter().filter(|e|
        e.kind == three_kingdoms_spire_reborn::combat::EventKind::FoeSummoned
            && e.source == "assassin").collect();
    assert_eq!(summons.len(), 1, "summon must be a single authoritative event");
    // The player's next decision sees the new foe's real, already-rolled
    // intent. Neither the CLI nor the test invents the next enemy action.
    assert!(!battle.foes[1].intent.label().is_empty());
    let restored = Game::from_json(&game.to_json().unwrap()).expect("summoned foe survives save");
    assert_eq!(restored.battle.unwrap().foes[1].id, "assassin");
}


#[test]
fn player_attack_on_summoned_vanguard_cuts_yinping_supply_route_once() {
    // Controlled tactical fixture ONLY. Do not count the manually shortened
    // raider HP or supplied hand card as natural floor-zero campaign evidence.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter Deng Ai boss");
    game.battle.as_mut().unwrap().foes[0].phase = 1;
    game.battle.as_mut().unwrap().foes[0].intent =
        Intent::Boss(BossMove::YinpingFlank(1));
    game.apply(Action::EndTurn).expect("raid summons one vanguard");
    let boss_before = game.battle.as_ref().unwrap().foes[0].hp;
    {
        let battle = game.battle.as_mut().unwrap();
        assert_eq!(battle.foes[1].id, "assassin");
        battle.foes[1].hp = 1;
        battle.foes[1].block = 0;
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    }
    let entry = game.to_json().expect("controlled raider state");
    let mut restored = Game::from_json(&entry).expect("resume controlled tactical state");
    let card_index = game.battle.as_ref().unwrap().hand.len();
    let result = game.apply(Action::Play { index: card_index, target: 2 })
        .expect("player targets real spawned raider");
    let replayed = restored.apply(Action::Play { index: card_index, target: 2 })
        .expect("same choice after save restore");
    assert_eq!(result, replayed);
    assert!(result.join("；").contains("夺回阴平山道，邓艾遭反击6点无视护甲伤害"), "{result:?}");
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.foes[1].hp, 0);
    assert_eq!(battle.foes[0].hp, boss_before - 6);
    assert_eq!(battle.events.iter().filter(|event|
        event.source == "yinping_supply_cut" && event.amount == 6
            && event.kind == three_kingdoms_spire_reborn::combat::EventKind::DamageDealt
    ).count(), 1);
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    // An invalid target must be exercised with a card that actually
    // requires a target. A guard or utility card ignores the target number.
    game.battle.as_mut().unwrap().hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    let dead_target_attack = game.battle.as_ref().unwrap().hand.len();
    let settled = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: dead_target_attack, target: 2 }).is_err(),
        "defeated raider must reject a real targeted attack");
    assert_eq!(game.to_json().unwrap(), settled, "invalid action must be atomic");
}

#[test]
fn guarded_second_phase_interception_pins_visible_raider_and_replays() {
    // Controlled phase-two tactical fixture, never a floor-zero victory.
    // The assassin is summoned by the real EndTurn action, not injected.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).unwrap();
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[0].phase = 1;
        battle.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("real failed infiltration summons raider");
    assert_eq!(game.battle.as_ref().unwrap().foes[1].id, "assassin");
    // The preceding failed raid legitimately costs one energy on the next
    // turn. This controlled tactical case restores the third energy solely
    // to validate the FULL 12-armor interception; the separate ten-armor
    // case below keeps the actual two-energy penalty and its consequences.
    assert_eq!(game.battle.as_ref().unwrap().energy, 2);
    game.battle.as_mut().unwrap().energy = 3;
    // Three legal defensive plays reach 12 armor but not four-card threshold.
    // The controlled hand fixture isolates the new player-choice branch.
    for _ in 0..3 {
        let battle = game.battle.as_mut().unwrap();
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("defend"));
        let index = battle.hand.len();
        game.apply(Action::Play { index, target: 1 }).unwrap();
    }
    assert_eq!(game.battle.as_ref().unwrap().played_this_turn, 3);
    assert!(game.battle.as_ref().unwrap().block >= 12);
    // The boss's next intent is deliberately fixed to the same second-phase
    // raid, not a random new battle or forged enemy.
    game.battle.as_mut().unwrap().foes[0].intent =
        Intent::Boss(BossMove::YinpingFlank(1));
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    let output = game.apply(Action::EndTurn).unwrap();
    let replayed = restored.apply(Action::EndTurn).unwrap();
    assert_eq!(output, replayed);
    assert!(output.join("；").contains("山道援军也被压制"), "{output:?}");
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.foes[1].weak, 1);
    assert_eq!(battle.events.iter().filter(|event|
        event.source == "yinping_raider_pinned"
    ).count(), 1);
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
}

#[test]
fn yinping_raider_telegraphs_escort_cover_and_direct_kill_prevents_support() {
    // Controlled late-game boss phase, NOT a natural floor-zero campaign.
    // The raider itself is created by a genuine EndTurn failure, and its
    // intent is rolled by the production engine, not assigned by this test.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter Deng Ai encounter");
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[0].phase = 1;
        battle.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("raider arrives through actual boss action");
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.turn, 2);
    assert_eq!(battle.foes[1].id, "assassin");
    let support = match battle.foes[1].intent {
        Intent::Assist(armor) => armor,
        ref unexpected => panic!("living raider needs a visible, actionable support intent: {unexpected:?}"),
    };
    assert!(support >= 6);
    assert!(battle.foes[1].intent.label().contains("协同护阵"));

    let entry = game.to_json().expect("save with announced support");
    let mut loaded = Game::from_json(&entry).expect("restore announced support");
    assert_eq!(loaded.battle.as_ref().unwrap().foes[1].intent,
        Intent::Assist(support));
    let report = game.apply(Action::EndTurn).expect("raider defends commander");
    assert_eq!(report, loaded.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert!(report.join("；").contains("协同护阵") || report.join("；").contains("布阵"),
        "{report:?}");
    let battle = game.battle.as_ref().expect("active boss encounter");
    assert!(battle.foes[0].block >= support,
        "raider's support must alter authoritative commander armor");
    assert!(battle.events.iter().any(|e|
        e.kind == three_kingdoms_spire_reborn::combat::EventKind::EnemyTechnique
            && e.source == "assassin" && e.amount == support
    ), "support must be in the combat event journal");

    // If a player selects the announced raider as their attack target,
    // defeating it before EndTurn prevents the otherwise scheduled support.
    // HP/hand staging is explicit and only verifies this local tactic.
    let mut cut = Game::from_json(&entry).expect("branch before raider support");
    {
        let battle = cut.battle.as_mut().unwrap();
        battle.foes[1].hp = 1;
        battle.foes[1].block = 0;
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    }
    let card = cut.battle.as_ref().unwrap().hand.len();
    cut.apply(Action::Play { index: card, target: 2 })
        .expect("player kills telegraphed supporting raider");
    assert_eq!(cut.battle.as_ref().unwrap().foes[1].hp, 0);
    assert!(cut.battle.as_ref().unwrap().events.iter().any(|e|
        e.source == "yinping_supply_cut"));
    let before_support = cut.battle.as_ref().unwrap().events.iter()
        .filter(|e| e.source == "assassin"
            && e.kind == three_kingdoms_spire_reborn::combat::EventKind::EnemyTechnique)
        .count();
    let saved = cut.to_json().expect("save after direct cut");
    let mut restored = Game::from_json(&saved).expect("resume counterplay");
    let report = cut.apply(Action::EndTurn).expect("skip defeated raider intent");
    assert_eq!(report, restored.apply(Action::EndTurn).unwrap());
    assert_eq!(cut.to_json().unwrap(), restored.to_json().unwrap());
    let after_support = cut.battle.as_ref().unwrap().events.iter()
        .filter(|e| e.source == "assassin"
            && e.kind == three_kingdoms_spire_reborn::combat::EventKind::EnemyTechnique)
        .count();
    assert_eq!(after_support, before_support, "dead raider cannot support Deng Ai");
}

#[test]
fn player_guard_disrupts_raider_cover_during_the_same_enemy_turn() {
    // Controlled phase-two scenario, not a full natural campaign.
    // The enemy's Assist intent must first be generated by the real
    // failure-and-summon action rather than manually invented.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter Deng Ai");
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[0].phase = 1;
        battle.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("generate actual vanguard and support intent");
    let support = match game.battle.as_ref().unwrap().foes[1].intent {
        Intent::Assist(n) => n,
        ref other => panic!("expected announced raider support: {other:?}"),
    };
    assert_eq!(game.battle.as_ref().unwrap().energy, 2);
    // Repeat a phase-two infiltration on that naturally generated support
    // turn to isolate simultaneous boss-raid and raider-assist settlement.
    game.battle.as_mut().unwrap().foes[0].intent =
        Intent::Boss(BossMove::YinpingFlank(1));
    for _ in 0..2 {
        let battle = game.battle.as_mut().unwrap();
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("defend"));
        let index = battle.hand.len();
        game.apply(Action::Play { index, target: 1 })
            .expect("two real paid defense cards");
    }
    let ready = game.battle.as_ref().unwrap();
    assert_eq!(ready.block, 10);
    assert_eq!(ready.played_this_turn, 2);
    assert_eq!(ready.energy, 0);
    let save = game.to_json().expect("persist announced support and paid defense");
    let mut resumed = Game::from_json(&save).expect("restore pending simultaneous actions");
    let messages = game.apply(Action::EndTurn).expect("player partial screen");
    assert_eq!(messages, resumed.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
    let report = messages.join("；");
    assert!(report.contains("阴平偷渡得手"), "{report}");
    assert!(report.contains("援军遭虚弱牵制，护阵减弱"), "{report}");
    let battle = game.battle.as_ref().unwrap();
    let reduced_support = support * 3 / 4;
    assert_eq!(battle.foes[0].block, 9 + reduced_support,
        "only reduced support may reinforce the nine armor from the raid");
    assert_eq!(battle.foes[1].weak, 1,
        "weakened raider retains one stack after end-turn decay");
    assert!(battle.events.iter().any(|e|
        e.kind == three_kingdoms_spire_reborn::combat::EventKind::EnemyTechnique
            && e.source == "assassin" && e.amount == reduced_support
    ), "journal must report actual post-weak support, never the previewed full value");
}

#[test]
fn surviving_raider_escorts_extra_gold_unless_defeated_or_guarded() {
    // Tactical phase-two staging only: the vanguard itself must still
    // appear from the authoritative failed Yinping raid. No fake foe,
    // history receipt, RNG, or terminal result is injected.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).unwrap();
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[0].phase = 1;
        battle.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("real raid summons visible vanguard");
    assert_eq!(game.battle.as_ref().unwrap().foes[1].id, "assassin");
    assert!(game.battle.as_ref().unwrap().foes[1].hp > 0);
    // The raider is left alive and no defense is played. This is a genuine
    // tactical opportunity cost, not a hidden extra penalty on first summon.
    game.battle.as_mut().unwrap().foes[0].intent =
        Intent::Boss(BossMove::YinpingFlank(1));
    let before = game.gold;
    assert!(before >= 20, "the fixture must expose the full escort levy");
    let saved = game.to_json().expect("save prior to escorted raid");
    let mut restored = Game::from_json(&saved).expect("restore before raid");
    let messages = game.apply(Action::EndTurn).expect("unopposed escorted raid");
    assert_eq!(messages, restored.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    assert_eq!(game.gold, before - 20, "15 base and 5 escorted gold");
    assert!(messages.join("；").contains("山道援军协同多掠金5，负重暴露破绽（易伤）"), "{messages:?}");
    let after = game.battle.as_ref().expect("surviving foe encounter");
    assert_eq!(after.foes[1].vulnerable, 1,
        "the exposed escort remains vulnerable after turn-end status decay");
    assert!(after.events.iter().any(|e|
        e.kind == three_kingdoms_spire_reborn::combat::EventKind::StatusApplied
            && e.source == "vulnerable" && e.amount == 2
    ), "exposure is an authoritative combat status event");
    assert_eq!(after.events.iter().filter(|e|
        e.kind == three_kingdoms_spire_reborn::combat::EventKind::EnemyTechnique
            && e.source == "yinping_raider_escort_levy"
            && e.amount == 5
    ).count(), 1, "escort levy must have one stable journal event");

    // The exposed escort offers a genuine next-turn risk/reward decision:
    // directly defeating it recovers only the five gold logged as stolen,
    // while poison or passive death cannot trigger the supply-cut hook.
    let mut counter = Game::from_json(&game.to_json().unwrap())
        .expect("restore exposed raider");
    {
        let battle = counter.battle.as_mut().unwrap();
        battle.foes[1].hp = 1;
        battle.foes[1].block = 0;
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    }
    let before_recovery = counter.gold;
    let card = counter.battle.as_ref().unwrap().hand.len();
    counter.apply(Action::Play { index: card, target: 2 })
        .expect("player directly defeats exposed escort");
    assert_eq!(counter.gold, before_recovery + 5);
    assert!(counter.battle.as_ref().unwrap().events.iter().any(|e|
        e.kind == three_kingdoms_spire_reborn::combat::EventKind::PowerTriggered
            && e.source == "yinping_escort_gold_recovered" && e.amount == 5
    ));
    let saved = counter.to_json().expect("persist recovered supply");
    let restored = Game::from_json(&saved).expect("resume recovered supply");
    assert_eq!(counter.to_json().unwrap(), restored.to_json().unwrap());
}

#[test]
fn escorted_supplies_remain_recoverable_after_eighty_logged_events() {
    // Long-battle tactical fixture, not a natural floor-zero journey.
    // The levy and vanguard are emitted by genuine EndTurn boss actions.
    // A patient player must not lose the earned recovery merely because
    // the recent 80-event UI window no longer displays the original raid.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter Deng Ai");
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[0].phase = 1;
        battle.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("real boss summons escort");
    game.battle.as_mut().unwrap().foes[0].intent =
        Intent::Boss(BossMove::YinpingFlank(1));
    game.apply(Action::EndTurn).expect("real escort steals supplies");
    assert!(game.battle.as_ref().unwrap().events.iter().any(|event|
        event.source == "yinping_raider_escort_levy" && event.amount == 5));
    // Controlled hand/energy replenishment exercises repeated PAID actions
    // and their normal event emission, without injecting synthetic events,
    // history flags, RNG outcomes, enemies, or a false campaign conclusion.
    for _ in 0..80 {
        if !game.battle.as_ref().unwrap().events.iter().any(|event|
            event.source == "yinping_raider_escort_levy") {
            break;
        }
        let battle = game.battle.as_mut().unwrap();
        battle.energy = battle.energy.max(1);
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("defend"));
        let index = battle.hand.len();
        game.apply(Action::Play { index, target: 1 })
            .expect("legal defensive card emits ordinary battle events");
    }
    assert!(!game.battle.as_ref().unwrap().events.iter().any(|event|
        event.source == "yinping_raider_escort_levy"),
        "the bounded UI journal must truly have evicted the levy");
    assert!(game.event_history.as_ref().is_some_and(|history|
        history.iter().any(|record|
            record.floor == 40 && record.event.source == "yinping_raider_escort_levy"
                && record.event.amount == 5)),
        "the durable encounter archive retains the actual levy");
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[1].hp = 1;
        battle.foes[1].block = 0;
        // This setup preserves the exposure already earned from the actual
        // escorted levy; it does not manufacture a historical consequence.
        assert!(battle.foes[1].vulnerable > 0);
        battle.energy = battle.energy.max(1);
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    }
    let before = game.gold;
    let index = game.battle.as_ref().unwrap().hand.len();
    let save = game.to_json().expect("persist after window eviction");
    let mut loaded = Game::from_json(&save).expect("restore long-fight archive");
    let feedback = game.apply(Action::Play { index, target: 2 })
        .expect("legally kill loot-carrying raider after long fight");
    assert_eq!(feedback, loaded.apply(Action::Play { index, target: 2 }).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert_eq!(game.gold, before + 5,
        "archived escorted levy, not the recent UI window, pays recovery");
    assert!(game.battle.as_ref().unwrap().events.iter().any(|event|
        event.source == "yinping_escort_gold_recovered" && event.amount == 5));
}

#[test]
fn exposed_escort_loot_can_be_recovered_after_commander_falls() {
    // A controlled phase-two encounter: real EndTurn summons the escort
    // and records the gold levy. The commander is then staged as defeated
    // to isolate the player's remaining tactical choice, not as a natural
    // floor-zero campaign completion.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).unwrap();
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[0].phase = 1;
        battle.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("summon actual escort");
    game.battle.as_mut().unwrap().foes[0].intent =
        Intent::Boss(BossMove::YinpingFlank(1));
    game.apply(Action::EndTurn).expect("record actual escorted levy");
    {
        let battle = game.battle.as_mut().unwrap();
        assert_eq!(battle.foes[1].vulnerable, 1);
        battle.foes[0].hp = 0;
        battle.foes[1].hp = 1;
        battle.foes[1].block = 0;
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    }
    let before = game.gold;
    let save = game.to_json().expect("save pending direct counterplay");
    let mut resumed = Game::from_json(&save).expect("restore pending direct counterplay");
    // Killing the final foe also settles the boss victory and awards its
    // normal gold. Compare identical legal final strikes with and without
    // the exposure status to isolate the *additional* five-gold recovery.
    // Comparing against before+5 accidentally counts the victory payout
    // as a regression in the escort-recovery rule.
    let mut unexposed = Game::from_json(&save).expect("comparison final strike");
    unexposed.battle.as_mut().unwrap().foes[1].vulnerable = 0;
    let card = game.battle.as_ref().unwrap().hand.len();
    let result = game.apply(Action::Play { index: card, target: 2 })
        .expect("directly defeat exposed escort");
    assert_eq!(result, resumed.apply(Action::Play { index: card, target: 2 }).unwrap());
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
    unexposed.apply(Action::Play { index: card, target: 2 })
        .expect("same legal final strike without exposed loot");
    assert!(game.gold > before, "final boss victory still pays its normal reward");
    assert_eq!(game.gold, unexposed.gold + 5,
        "exposure recovers exactly five gold on top of identical victory rewards");
    // Defeating the final foe immediately settles combat and clears the
    // battle object. The five-gold differential is the observable, persisted
    // outcome; battle.events is only inspectable before final settlement.
    assert!(game.battle.is_none(), "last foe death settles the encounter");
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap(),
        "the recovered gold and final reward survive serialized replay");
}

#[test]
fn ten_armor_after_a_failed_raid_screens_existing_vanguard_without_free_interception() {
    // Controlled second-phase opponent and cards; energy stays on the real
    // two-point penalty from the first failed raid. This is not 0→40 evidence.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).unwrap();
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[0].phase = 1;
        battle.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("failed raid summons visible raider");
    assert_eq!(game.battle.as_ref().unwrap().energy, 2);
    assert_eq!(game.battle.as_ref().unwrap().foes[1].id, "assassin");
    for _ in 0..2 {
        let battle = game.battle.as_mut().unwrap();
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("defend"));
        let index = battle.hand.len();
        game.apply(Action::Play { index, target: 1 }).expect("legal paid defense");
    }
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.played_this_turn, 2);
    assert_eq!(battle.block, 10);
    assert_eq!(battle.energy, 0);
    game.battle.as_mut().unwrap().foes[0].intent =
        Intent::Boss(BossMove::YinpingFlank(1));
    let gold_before = game.gold;
    let boss_hp = game.battle.as_ref().unwrap().foes[0].hp;
    let save = game.to_json().expect("save real paid defense actions");
    let mut resumed = Game::from_json(&save).expect("restore penalized combat");
    let messages = game.apply(Action::EndTurn).expect("partial defensive raid response");
    assert_eq!(messages, resumed.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
    let report = messages.join("；");
    assert!(report.contains("护阵牵制已有山道援军，攻击虚弱"), "{report}");
    assert!(report.contains("阴平偷渡得手"), "boss must still succeed");
    assert_eq!(game.gold, gold_before - gold_before.min(15),
        "ten-armor screen stops the five-gold escort levy, not the base loss");
    assert!(game.battle.as_ref().unwrap().events.iter().all(|event|
        event.source != "yinping_raider_escort_levy"));
    let after = game.battle.as_ref().unwrap();
    assert_eq!(after.foes[0].hp, boss_hp, "partial screen cannot claim intercept damage");
    assert_eq!(after.foes[1].weak, 1, "screen persists past turn-end decay");
    assert_eq!(after.events.iter().filter(|event|
        event.source == "yinping_raider_screened" &&
            event.kind == three_kingdoms_spire_reborn::combat::EventKind::PowerTriggered
    ).count(), 1, "one screen must create one replayable event");
}

#[test]
fn defense_only_yinping_interception_weakens_next_attack_and_replays_after_restore() {
    // A controlled floor-40 encounter proves the alternative, defense-focused
    // player action path. This is not natural floor-zero campaign evidence.
    let mut game = fifth_act_alternate_boss();
    game.apply(Action::Choose { index: 1 }).expect("enter alternate final boss");
    let boss_hp = game.battle.as_ref().unwrap().foes[0].hp;
    let gold = game.gold;
    // Supply the two cards required to exercise the guard-specific branch.
    // Both are actually played via Game::apply, not granted as free block.
    for _ in 0..2 {
        game.battle.as_mut().unwrap().hand.push(
            three_kingdoms_spire_reborn::Card::new("defend"),
        );
        let index = game.battle.as_ref().unwrap().hand.len();
        game.apply(Action::Play { index, target: 1 })
            .expect("real defensive card play");
    }
    let before = game.battle.as_ref().unwrap();
    assert!(before.block >= 10);
    assert_eq!(before.played_this_turn, 2, "card-count path must remain unavailable");
    let save = game.to_json().expect("save actual defense actions");
    let mut resumed = Game::from_json(&save).expect("restore defensive position");
    let result = game.apply(Action::EndTurn).expect("seal the pass");
    let from_save = resumed.apply(Action::EndTurn).expect("replay the same turn");
    assert_eq!(result, from_save);
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
    assert!(result.join("；").contains("坚守扰乱偷渡军，下回合邓艾攻击虚弱"), "{result:?}");
    let boss = &game.battle.as_ref().unwrap().foes[0];
    assert_eq!(boss.hp, boss_hp - 4);
    assert_eq!(boss.weak, 1, "weak must survive the current EndTurn tick");
    assert!(boss.vulnerable >= 2);
    assert_eq!(game.gold, gold);
    assert!(game.battle.as_ref().unwrap().events.iter().any(|event|
        event.kind == three_kingdoms_spire_reborn::combat::EventKind::StatusApplied
            && event.source == "weak" && event.amount == 2
    ));
}


#[test]
fn jieting_three_decisions_have_distinct_replayable_fifth_act_boss_consequences() {
    use three_kingdoms_spire_reborn::JietingPlan;
    // Controlled floor-39 staging. The independent natural Jieting test
    // proves the earlier council can be chosen from floor zero; this
    // tactical matrix only proves what its three settled outcomes do
    // after entering Deng Ai through an ordinary Action::Choose.
    // The forecast uses the same act_variant(4) resolver as boss spawning,
    // even when no explicit variant was selected at floor zero.
    for (variant, expected_alternate) in [(1_u8, true), (0_u8, false)] {
        let mut seeded = Game::new_with_campaign_difficulty(
            Hero::ZhugeLiang, 1_u64 << 44, Difficulty::Warlords,
            format!("selected-jieting-yinping-{variant}"),
        );
        seeded.apply(Action::SetVariants { act1: 0, act2: 0, act3: variant })
            .expect("select fifth-act commander through legal campaign setup");
        seeded.floor = 39;
        seeded.jieting_plan = Some(JietingPlan::WaterSettled);
        let saved = seeded.to_json().unwrap();
        let forecast = seeded.route_objective_hint(Node::Boss)
            .expect("both fifth-act commanders must preview settled Jieting consequences");
        assert_eq!(forecast.contains("邓艾·阴平"), expected_alternate,
            "forecast must name the selected commander: {forecast}");
        assert_eq!(forecast.contains("姜维·北伐"), !expected_alternate,
            "the other commander must have its own historical forecast: {forecast}");
        assert_eq!(seeded.to_json().unwrap(), saved, "seed forecast must be read-only");
    }
    let cases = [
        (JietingPlan::WaterSettled, "街亭守水", "jieting_yinping_wells"),
        (JietingPlan::HillSettled, "街亭据山", "jieting_yinping_hills"),
        (JietingPlan::RetreatSettled, "街亭撤退", "jieting_yinping_retreat"),
    ];
    for (plan, warning, event_source) in cases {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::ZhugeLiang, 1_u64 << 44, Difficulty::Warlords,
            format!("jieting-yinping-{event_source}"),
        );
        game.apply(Action::SetVariants { act1: 0, act2: 0, act3: 1 })
            .expect("legal variant choice at floor zero");
        game.floor = 39;
        game.stage = Stage::Map;
        game.jieting_plan = Some(plan);
        game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
        game.map_lane = None;
        game.route = vec![Node::Boss];
        game.route_lanes = vec![0];
        let pre = game.to_json().unwrap();
        let hint = game.route_objective_hint(Node::Boss).expect("read-only boss warning");
        assert!(hint.contains(warning), "{hint}");
        assert_eq!(game.to_json().unwrap(), pre, "route preview must not alter state");
        let feedback = game.apply(Action::Choose { index: 1 })
            .expect("naturally valid map action enters alternate boss");
        assert_eq!(game.floor, 40);
        assert_eq!(game.stage, Stage::Combat);
        assert!(feedback.join("；").contains(warning), "{feedback:?}");
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].id, "dengai");
        assert_eq!(game.jieting_plan, Some(plan), "historical receipt remains stable");
        assert!(game.tactical_objective().unwrap().contains(warning));
        assert_eq!(battle.events.iter().filter(|event|
            event.kind == three_kingdoms_spire_reborn::combat::EventKind::PowerTriggered
                && event.source == event_source
        ).count(), 1, "only one cross-act history consequence");
        match plan {
            JietingPlan::WaterSettled => {
                assert!(battle.block >= 10);
                assert_eq!(battle.foes[0].weak, 2);
                assert_eq!(battle.foes[0].strength, 0);
            }
            JietingPlan::HillSettled => {
                assert_eq!(battle.strength, 1);
                assert_eq!(battle.foes[0].strength, 2);
                assert!(battle.foes[0].block >= 10);
            }
            JietingPlan::RetreatSettled => {
                assert_eq!(battle.energy, 4);
                assert!(battle.foes[0].block >= 6);
            }
            _ => unreachable!(),
        }
        let saved = game.to_json().unwrap();
        let mut restored = Game::from_json(&saved).expect("save retains real boss and history effect");
        assert_eq!(restored.to_json().unwrap(), saved);
        let first = game.apply(Action::EndTurn).expect("end real first boss turn");
        let repeated = restored.apply(Action::EndTurn).expect("same historical boss after resume");
        assert_eq!(first, repeated);
        assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
        if plan == JietingPlan::WaterSettled {
            assert_eq!(game.battle.as_ref().unwrap().foes[0].weak, 1,
                "water decision should weaken the next enemy attack, not only the first signature");
        }
    }
}

#[test]
fn jieting_water_screen_recovers_only_on_actual_player_vanguard_kill_and_replays() {
    use three_kingdoms_spire_reborn::JietingPlan;
    // Isolated tactical fixture, not a natural floor-zero journey.
    let mut game = Game::new_with_campaign_difficulty(
        Hero::ZhugeLiang, 1_u64 << 44, Difficulty::Warlords,
        "water-yinping-recapture".into(),
    );
    game.floor = 39;
    game.stage = Stage::Map;
    game.jieting_plan = Some(JietingPlan::WaterSettled);
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    assert!(game.route_objective_hint(Node::Boss).unwrap().contains("夺回4点护甲"));
    game.apply(Action::Choose { index: 1 }).expect("enter alternate commander");
    assert!(game.tactical_objective().unwrap().contains("夺回4点护甲"));
    {
        let b = game.battle.as_mut().unwrap();
        b.foes[0].phase = 1;
        b.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("allow telegraphed vanguard to arrive");
    {
        let b = game.battle.as_mut().unwrap();
        assert_eq!(b.foes[1].id, "assassin");
        b.foes[1].hp = 1;
        b.foes[1].block = 0;
        b.hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    }
    let card_index = game.battle.as_ref().unwrap().hand.len();
    let before_block = game.battle.as_ref().unwrap().block;
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    let feedback = game.apply(Action::Play { index: card_index, target: 2 }).unwrap();
    let resumed_feedback = restored.apply(Action::Play { index: card_index, target: 2 }).unwrap();
    assert_eq!(feedback, resumed_feedback);
    assert!(feedback.join("；").contains("街亭守水旧部夺回掩护，护甲+4"));
    let b = game.battle.as_ref().unwrap();
    assert_eq!(b.block, before_block + 4);
    assert_eq!(b.foes[1].hp, 0);
    assert_eq!(b.events.iter().filter(|e|
        e.source == "jieting_yinping_water_screen" && e.amount == 4
    ).count(), 1);
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
}

#[test]
fn jieting_hill_risk_unlocks_eight_damage_when_player_breaks_yinping_vanguard() {
    use three_kingdoms_spire_reborn::JietingPlan;
    // Controlled tactical late-game fixture. Earlier Jieting decision
    // naturally reachable only in the separate floor-zero acceptance driver.
    let mut game = Game::new_with_campaign_difficulty(
        Hero::ZhugeLiang, 1_u64 << 44, Difficulty::Warlords,
        "jieting-yinping-flank-counter".into(),
    );
    game.floor = 39;
    game.stage = Stage::Map;
    game.jieting_plan = Some(JietingPlan::HillSettled);
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).expect("enter alternate fifth boss");
    {
        let b = game.battle.as_mut().unwrap();
        b.foes[0].phase = 1;
        b.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("summon mountain raider after an unblocked raid");
    {
        let b = game.battle.as_mut().unwrap();
        assert_eq!(b.foes[1].id, "assassin");
        b.foes[1].hp = 1;
        b.foes[1].block = 0;
        b.hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    }
    let before_boss = game.battle.as_ref().unwrap().foes[0].hp;
    let card_index = game.battle.as_ref().unwrap().hand.len();
    let snapshot = game.to_json().expect("save before actual vanguard attack");
    let mut restored = Game::from_json(&snapshot).expect("restore tactical target");
    let first = game.apply(Action::Play { index: card_index, target: 2 })
        .expect("strike real mountain raider");
    let second = restored.apply(Action::Play { index: card_index, target: 2 })
        .expect("replay the identical attack");
    assert_eq!(first, second);
    assert!(first.join("；").contains("邓艾遭反击8点无视护甲伤害"), "{first:?}");
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.foes[0].hp, before_boss - 8);
    assert_eq!(battle.foes[1].hp, 0);
    assert_eq!(battle.events.iter().filter(|event|
        event.source == "yinping_supply_cut"
            && event.kind == three_kingdoms_spire_reborn::combat::EventKind::DamageDealt
            && event.amount == 8
    ).count(), 1);
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
}

#[test]
fn jieting_retreat_reserve_rewards_actual_yinping_interception_after_restore() {
    use three_kingdoms_spire_reborn::JietingPlan;
    // Tactical fixture at the real boss entrance. The separate natural
    // floor-zero driver proves how the RetreatSettled flag is earned.
    let mut game = Game::new_with_campaign_difficulty(
        Hero::ZhugeLiang, 1_u64 << 44, Difficulty::Warlords,
        "retreat-yinping-interception".into(),
    );
    game.floor = 39;
    game.stage = Stage::Map;
    game.jieting_plan = Some(JietingPlan::RetreatSettled);
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    assert!(game.route_objective_hint(Node::Boss).unwrap()
        .contains("成功截击阴平偷渡额外反击2点"));
    game.apply(Action::Choose { index: 1 }).expect("enter real boss via legal route");
    assert_eq!(game.battle.as_ref().unwrap().energy, 4);
    assert!(game.tactical_objective().unwrap().contains("额外反击2点"));
    // A controlled pre-committed defensive position isolates the actual
    // boss interception, not a claim of a natural full-campaign win.
    game.battle.as_mut().unwrap().block = 12;
    let hp = game.battle.as_ref().unwrap().foes[0].hp;
    let saved = game.to_json().expect("save before real turn action");
    let mut resumed = Game::from_json(&saved).expect("restore before interception");
    let actual = game.apply(Action::EndTurn).expect("guarded interception");
    let replay = resumed.apply(Action::EndTurn).expect("identical restored interception");
    assert_eq!(actual, replay);
    assert!(actual.join("；").contains("反击造成6点无视护甲伤害"));
    assert_eq!(game.battle.as_ref().unwrap().foes[0].hp, hp - 6);
    assert_eq!(game.battle.as_ref().unwrap().events.iter().filter(|event|
        event.source == "yinping_interception"
            && event.kind == three_kingdoms_spire_reborn::combat::EventKind::DamageDealt
            && event.amount == 6
    ).count(), 1);
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
}

#[test]
fn jieting_retreat_recovers_command_only_after_direct_yinping_vanguard_break() {
    use three_kingdoms_spire_reborn::JietingPlan;
    // Controlled fifth-act tactical fixture, not a natural floor-zero win.
    let mut game = Game::new_with_campaign_difficulty(
        Hero::ZhugeLiang, 1_u64 << 44, Difficulty::Warlords,
        "retreat-yinping-command".into(),
    );
    game.floor = 39;
    game.stage = Stage::Map;
    game.jieting_plan = Some(JietingPlan::RetreatSettled);
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    assert!(game.route_objective_hint(Node::Boss).unwrap().contains("回收1点军令"));
    game.apply(Action::Choose { index: 1 }).expect("enter fifth-act alternate commander");
    assert!(game.tactical_objective().unwrap().contains("回收1点军令"));
    {
        let battle = game.battle.as_mut().unwrap();
        battle.foes[0].phase = 1;
        battle.foes[0].intent = Intent::Boss(BossMove::YinpingFlank(1));
    }
    game.apply(Action::EndTurn).expect("summon telegraphed mountain raider");
    {
        let battle = game.battle.as_mut().unwrap();
        assert_eq!(battle.foes[1].id, "assassin");
        battle.foes[1].hp = 1;
        battle.foes[1].block = 0;
        battle.energy = 1;
        battle.hand.push(three_kingdoms_spire_reborn::Card::new("strike"));
    }
    let index = game.battle.as_ref().unwrap().hand.len();
    let saved = game.to_json().expect("save before real action");
    let mut restored = Game::from_json(&saved).expect("restore before action");
    let feedback = game.apply(Action::Play { index, target: 2 }).expect("defeat real raider");
    let replay_feedback = restored.apply(Action::Play { index, target: 2 })
        .expect("identical restored action");
    assert_eq!(feedback, replay_feedback);
    assert!(feedback.join("；").contains("街亭撤退旧部截断山道，回收军令+1"));
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.energy, 1, "one command spent and one recaptured");
    assert_eq!(battle.foes[1].hp, 0);
    assert_eq!(battle.events.iter().filter(|event|
        event.source == "jieting_yinping_retreat_command" && event.amount == 1
    ).count(), 1);
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
}

#[test]
fn old_campaign_bosses_never_use_yinping_infiltration() {
    let mut old = Game::new(Hero::GuanYu, 1_u64 << 44);
    old.floor = 20;
    old.stage = Stage::Map;
    old.act_map = Some(map::ActMap::generate(|_| 0, 2));
    old.map_lane = None;
    old.route = vec![Node::Boss];
    old.route_lanes = vec![0];
    old.apply(Action::Choose { index: 1 }).expect("legacy final boss");
    assert_ne!(old.battle.as_ref().unwrap().foes[0].id, "dengai");
}
