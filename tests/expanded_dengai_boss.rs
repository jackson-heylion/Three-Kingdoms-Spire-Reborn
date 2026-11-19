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
