use three_kingdoms_spire_reborn::{
    Action, Card, Difficulty, Game, Intent, Node, Stage,
    combat::EventKind,
    content::{Hero, card_spec},
    map,
    progression::Profile,
};

fn enter_node(game: &mut Game, completed_floor: u8, node: Node) {
    game.floor = completed_floor;
    game.stage = Stage::Map;
    let act = completed_floor / game.campaign_rows();
    game.act_map = Some(if game.difficulty.is_some() {
        map::ActMap::generate_expanded(|_| 0, act)
    } else {
        map::ActMap::generate(|_| 0, act)
    });
    game.map_lane = None;
    game.route = vec![node];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).unwrap();
}

#[test]
fn legacy_v1_and_v2_twenty_one_floor_saves_load_play_and_reject_corruption() {
    // Construct genuine 21-floor baseline snapshots representing V1 and stripped early V2 saves.
    let mut base = Game::new(Hero::GuanYu, 2026);
    base.apply(Action::Choose { index: 1 }).unwrap();
    let v2_json = base.to_json().unwrap();
    let mut v1_val: serde_json::Value = serde_json::from_str(&v2_json).unwrap();
    let obj = v1_val.as_object_mut().unwrap();
    obj.insert("version".into(), serde_json::json!(1));
    for v2_only in [
        "action_history", "event_history", "resource_history", "battle_resource_history",
        "foe_board_history", "card_zone_history", "combat_effect_history",
        "battle_runtime_history", "campaign_history", "telemetry_history", "rng_history",
    ] {
        obj.remove(v2_only);
    }
    if let Some(battle) = obj.get_mut("battle").and_then(serde_json::Value::as_object_mut) {
        for field in ["played_this_turn", "last_turn_cards", "next_turn_energy_penalty"] {
            battle.remove(field);
        }
    }

    let mut v1_loaded = Game::from_json(&v1_val.to_string()).unwrap();
    assert_eq!(v1_loaded.campaign_max_floor(), 21);
    assert!(v1_loaded.difficulty.is_none());
    assert_eq!(v1_loaded.version, 2);
    assert_eq!(v1_loaded.defeat_diagnosis(), None);
    // V2 21-floor run with action_history replays deterministically after JSON restore.
    base.verify_action_replay().expect("legacy 21-floor V2 action_history must replay");
    let mut replay_a = Game::from_json(&v2_json).unwrap();
    let mut replay_b = Game::from_json(&replay_a.to_json().unwrap()).unwrap();
    replay_a.apply(Action::EndTurn).unwrap();
    replay_b.apply(Action::EndTurn).unwrap();
    assert_eq!(replay_a.to_json().unwrap(), replay_b.to_json().unwrap());
    // Still playable through combat -> end turn -> defeat/settle cleanly without expanded rules interference.
    v1_loaded.apply(Action::EndTurn).unwrap();
    v1_loaded.hp = 0;
    v1_loaded.stage = Stage::Lost;
    v1_loaded.battle = None;
    assert!(v1_loaded.defeat_diagnosis().unwrap().contains("旧制"));

    let mut profile = Profile::new();
    assert!(profile.record(&v1_loaded, "legacy-v1-compat".into()).unwrap());
    assert!(!profile.record(&v1_loaded, "legacy-v1-compat".into()).unwrap(),
        "legacy chronicle settlement must stay idempotent");

    // V2 snapshot without optional battle runtime fields still preserves 21-floor cap & fails closed on tampering.
    let mut v2_val: serde_json::Value = serde_json::from_str(&v2_json).unwrap();
    v2_val["battle"].as_object_mut().unwrap().remove("played_this_turn");
    let v2_loaded = Game::from_json(&v2_val.to_string()).unwrap();
    assert_eq!(v2_loaded.campaign_max_floor(), 21);
    assert!(v2_loaded.difficulty.is_none());

    let mut corrupt = v1_val.clone();
    corrupt["version"] = serde_json::json!(99);
    assert!(Game::from_json(&corrupt.to_string()).is_err());

    let mut bad_max_hp = v2_val;
    bad_max_hp["hp"] = serde_json::json!(99999);
    assert!(Game::from_json(&bad_max_hp.to_string()).is_err());
}

#[test]
fn invalid_actions_are_rejected_atomically_across_every_stage() {
    let assert_atomic_err = |g: &mut Game, action: Action| {
        let before = g.to_json().unwrap();
        let err = g.apply(action);
        assert!(err.is_err(), "expected error on stage {:?}", g.stage);
        assert_eq!(g.to_json().unwrap(), before, "rejected action mutated state on {:?}", g.stage);
    };

    // Map stage: out-of-range lane / non-map play action.
    let mut map_g = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "atomic-map".into());
    assert_atomic_err(&mut map_g, Action::Choose { index: 99 });
    assert_atomic_err(&mut map_g, Action::EndTurn);

    // Combat stage: invalid card slot, dead target, over-cost play, invalid assault outside siege.
    enter_node(&mut map_g, 0, Node::Battle);
    assert_atomic_err(&mut map_g, Action::Play { index: 99, target: 1 });
    assert_atomic_err(&mut map_g, Action::Play { index: 1, target: 9 });
    assert_atomic_err(&mut map_g, Action::AssaultGate);
    assert_atomic_err(&mut map_g, Action::UsePotionAt { slot: 3, target: 1 });
    map_g.battle.as_mut().unwrap().energy = 0;
    if let Some(pos) = map_g.battle.as_ref().unwrap().hand.iter().position(|c|
        card_spec(&c.id).is_some_and(|s| s.cost > 0))
    {
        assert_atomic_err(&mut map_g, Action::Play { index: pos + 1, target: 1 });
    }

    // Reward, Camp, Market, Story, Ancient, Won, Lost atomic checks.
    map_g.stage = Stage::Reward;
    map_g.battle = None;
    map_g.rewards = vec![Card::new("strike")];
    assert_atomic_err(&mut map_g, Action::Choose { index: 9 });

    map_g.stage = Stage::Camp;
    assert_atomic_err(&mut map_g, Action::Choose { index: 9 });

    map_g.stage = Stage::Market;
    map_g.gold = 0;
    map_g.market = vec![Card::new("strike")];
    assert_atomic_err(&mut map_g, Action::Choose { index: 1 });
    assert_atomic_err(&mut map_g, Action::Choose { index: 7 });

    map_g.stage = Stage::Story;
    map_g.lore_current = None;
    assert_atomic_err(&mut map_g, Action::Choose { index: 9 });
    assert_atomic_err(&mut map_g, Action::Choose { index: 5 });
    let mut council_g = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "atomic-council".into());
    council_g.gold = 0;
    enter_node(&mut council_g, 8, Node::Story);
    if council_g.story_fourth_option().is_some() {
        council_g.apply(Action::Choose { index: 4 }).unwrap();
        assert!(council_g.in_council());
        assert_atomic_err(&mut council_g, Action::Choose { index: 1 });
    }

    map_g.stage = Stage::Ancient;
    map_g.boon_choices = vec!["bingshu".into()];
    assert_atomic_err(&mut map_g, Action::Choose { index: 5 });

    map_g.stage = Stage::Won;
    assert_atomic_err(&mut map_g, Action::Choose { index: 1 });

    map_g.stage = Stage::Lost;
    assert_atomic_err(&mut map_g, Action::EndTurn);
}

#[test]
fn protracted_battle_turn_forty_supply_exhaustion_escalates_cleanly() {
    let mut stall = Game::new_with_campaign_difficulty(
        Hero::ZhugeLiang, 31, Difficulty::Warlords, "exhaustion-direct".into(),
    );
    enter_node(&mut stall, 7, Node::Boss);
    stall.battle.as_mut().unwrap().turn = 40;
    stall.battle.as_mut().unwrap().foes[0].intent = Intent::Guard(1);
    stall.hp = 10;
    stall.apply(Action::EndTurn).unwrap();
    assert_eq!(stall.hp, 9, "turn 40 should inflict 1 supply_exhaustion damage");

    stall.battle.as_mut().unwrap().turn = 44;
    stall.battle.as_mut().unwrap().foes[0].intent = Intent::Guard(1);
    stall.apply(Action::EndTurn).unwrap();
    assert_eq!(stall.hp, 7, "turn 44 should escalate to 2 supply_exhaustion damage");
    assert!(stall.battle.as_ref().unwrap().events.iter().any(|e|
        e.kind == EventKind::DamageReceived && e.source == "supply_exhaustion"));

    stall.hp = 2;
    stall.battle.as_mut().unwrap().turn = 48;
    stall.battle.as_mut().unwrap().foes[0].intent = Intent::Guard(1);
    stall.apply(Action::EndTurn).unwrap();
    assert_eq!(stall.stage, Stage::Lost);
    assert!(stall.defeat_diagnosis().unwrap().contains("粮尽"));
}

#[test]
fn all_seven_special_encounter_classes_verify_both_success_and_failure() {
    // 1. Breakout: formation escape success vs turn-4 timeout failure.
    let mut br_ok = Game::new_with_campaign_difficulty(Hero::GuanYu, 7, Difficulty::Warlords, "sp-br-ok".into());
    enter_node(&mut br_ok, 0, Node::Battle);
    {
        let b = br_ok.battle.as_mut().unwrap();
        b.played_this_turn = 3;
        b.block = 12;
        b.foes[0].intent = Intent::Guard(1);
    }
    br_ok.apply(Action::EndTurn).unwrap();
    assert_ne!(br_ok.stage, Stage::Lost);
    assert_ne!(br_ok.stage, Stage::Combat);

    let mut br_fail = Game::new_with_campaign_difficulty(Hero::GuanYu, 7, Difficulty::Warlords, "sp-br-fail".into());
    enter_node(&mut br_fail, 0, Node::Battle);
    for _ in 0..4 {
        if br_fail.stage != Stage::Combat { break; }
        br_fail.hp = br_fail.max_hp;
        br_fail.apply(Action::EndTurn).unwrap();
    }
    assert_eq!(br_fail.stage, Stage::Lost);

    // 2. Grain convoy: guard survival vs cargo depletion.
    let mut cv_ok = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "sp-cv-ok".into());
    enter_node(&mut cv_ok, 1, Node::Battle);
    {
        let b = cv_ok.battle.as_mut().unwrap();
        b.turn = 3;
        b.convoy_hp = Some(10);
        b.convoy_cover = Some(8);
        b.foes[0].intent = Intent::Guard(1);
    }
    cv_ok.apply(Action::EndTurn).unwrap();
    assert_ne!(cv_ok.stage, Stage::Combat);
    assert_ne!(cv_ok.stage, Stage::Lost);

    let mut cv_fail = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "sp-cv-fail".into());
    enter_node(&mut cv_fail, 1, Node::Battle);
    cv_fail.battle.as_mut().unwrap().convoy_hp = Some(1);
    cv_fail.battle.as_mut().unwrap().convoy_cover = Some(0);
    cv_fail.apply(Action::EndTurn).unwrap();
    assert_eq!(cv_fail.stage, Stage::Lost);

    // 3. Ambush counter: shatter armor via direct player attack -> weak vs defeat/no-bounty failure arm.
    let mut am_ok = Game::new_with_campaign_difficulty(Hero::LuBu, 7, Difficulty::Warlords, "sp-am-ok".into());
    enter_node(&mut am_ok, 2, Node::Battle);
    {
        let b = am_ok.battle.as_mut().unwrap();
        b.foes[0].block = 4;
        b.hand = vec![Card::new("strike")];
        b.energy = 3;
    }
    am_ok.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert!(am_ok.battle.as_ref().unwrap().events.iter().any(|e|
        e.kind == EventKind::PowerTriggered && e.source == "ambush_vanguard_reeling"));

    let mut am_fail = Game::new_with_campaign_difficulty(Hero::LuBu, 7, Difficulty::Warlords, "sp-am-fail".into());
    enter_node(&mut am_fail, 2, Node::Battle);
    am_fail.hp = 1;
    am_fail.battle.as_mut().unwrap().foes[0].intent = Intent::Attack(12);
    am_fail.apply(Action::EndTurn).unwrap();
    assert_eq!(am_fail.stage, Stage::Lost);
    assert!(!am_fail.event_history.as_ref().unwrap().iter().any(|r|
        r.event.source == "ambush_vanguard_reeling"));

    // 4. Siege: AssaultGate breaks gate vs turn-5 timeout.
    let mut sg_ok = Game::new_with_campaign_difficulty(Hero::LuBu, 7, Difficulty::Warlords, "sp-sg-ok".into());
    enter_node(&mut sg_ok, 3, Node::Battle);
    sg_ok.battle.as_mut().unwrap().siege_gate_hp = Some(4);
    sg_ok.battle.as_mut().unwrap().energy = 2;
    sg_ok.apply(Action::AssaultGate).unwrap();
    assert_ne!(sg_ok.stage, Stage::Combat);
    assert_ne!(sg_ok.stage, Stage::Lost);

    let mut sg_fail = Game::new_with_campaign_difficulty(Hero::LuBu, 7, Difficulty::Warlords, "sp-sg-fail".into());
    enter_node(&mut sg_fail, 3, Node::Battle);
    sg_fail.battle.as_mut().unwrap().turn = 5;
    sg_fail.apply(Action::EndTurn).unwrap();
    assert_eq!(sg_fail.stage, Stage::Lost);

    // 5. Naval crossing: cover survival at turn 4 vs sunk hull.
    let mut nv_ok = Game::new_with_campaign_difficulty(Hero::SunShangxiang, 7, Difficulty::Warlords, "sp-nv-ok".into());
    enter_node(&mut nv_ok, 18, Node::Battle);
    {
        let b = nv_ok.battle.as_mut().unwrap();
        b.turn = 4;
        b.convoy_hp = Some(10);
        b.convoy_cover = Some(8);
        b.foes.truncate(1);
        b.foes[0].intent = Intent::Guard(1);
    }
    nv_ok.apply(Action::EndTurn).unwrap();
    assert_ne!(nv_ok.stage, Stage::Combat);
    assert_ne!(nv_ok.stage, Stage::Lost);

    let mut nv_fail = Game::new_with_campaign_difficulty(Hero::SunShangxiang, 7, Difficulty::Warlords, "sp-nv-fail".into());
    enter_node(&mut nv_fail, 18, Node::Battle);
    nv_fail.battle.as_mut().unwrap().convoy_hp = Some(1);
    nv_fail.battle.as_mut().unwrap().convoy_cover = Some(0);
    nv_fail.apply(Action::EndTurn).unwrap();
    assert_eq!(nv_fail.stage, Stage::Lost);

    // 6. Jieting water defense: water council salvage success vs dry failure.
    let mut wt_ok = Game::new_with_campaign_difficulty(Hero::ZhugeLiang, 7, Difficulty::Warlords, "sp-wt-ok".into());
    wt_ok.gold = 100;
    enter_node(&mut wt_ok, 24, Node::Story);
    wt_ok.apply(Action::Choose { index: 4 }).unwrap();
    wt_ok.apply(Action::Choose { index: 1 }).unwrap();
    enter_node(&mut wt_ok, 25, Node::Battle);
    {
        let b = wt_ok.battle.as_mut().unwrap();
        b.convoy_hp = Some(8);
        b.convoy_cover = Some(2);
        b.foes.truncate(1);
        b.foes[0].intent = Intent::Guard(1);
    }
    wt_ok.apply(Action::EndTurn).unwrap();
    assert!(wt_ok.battle.as_ref().unwrap().events.iter().any(|e|
        e.source == "jieting_water_wells_repaired"));

    let mut wt_fail = Game::new_with_campaign_difficulty(Hero::ZhugeLiang, 7, Difficulty::Warlords, "sp-wt-fail".into());
    wt_fail.gold = 100;
    enter_node(&mut wt_fail, 24, Node::Story);
    wt_fail.apply(Action::Choose { index: 4 }).unwrap();
    wt_fail.apply(Action::Choose { index: 1 }).unwrap();
    enter_node(&mut wt_fail, 25, Node::Battle);
    wt_fail.battle.as_mut().unwrap().convoy_hp = Some(1);
    wt_fail.battle.as_mut().unwrap().convoy_cover = Some(0);
    wt_fail.apply(Action::EndTurn).unwrap();
    assert_eq!(wt_fail.stage, Stage::Lost);

    // 7. Boss multi-phase transition (Zhang Jiao Act 1): enters phase 2, advertises hint,
    //    and supports both boss defeat and boss victory settlement.
    let mut bs = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "sp-boss".into());
    bs.apply(Action::SetExpandedVariants { act1: 0, act2: 0, act3: 0, act4: 0, act5: 0 }).unwrap();
    enter_node(&mut bs, 7, Node::Boss);
    assert!(bs.phase_transition_hint(0).is_some());
    {
        let b = bs.battle.as_mut().unwrap();
        let half = (b.foes[0].max_hp / 2).max(1);
        b.foes[0].hp = half;
        b.foes[0].intent = Intent::Guard(1);
    }
    bs.apply(Action::EndTurn).unwrap();
    assert_eq!(bs.battle.as_ref().unwrap().foes[0].phase, 1);

    let mut bs_win = bs.clone();
    {
        let b = bs_win.battle.as_mut().unwrap();
        for f in &mut b.foes { f.hp = 1; }
        b.foes[1..].iter_mut().for_each(|f| f.hp = 0);
        b.hand = vec![Card::new("strike")];
        b.energy = 3;
    }
    bs_win.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert!(matches!(bs_win.stage, Stage::Reward | Stage::Ancient | Stage::Map | Stage::Won));

    bs.hp = 1;
    bs.battle.as_mut().unwrap().foes[0].intent = Intent::Attack(99);
    bs.apply(Action::EndTurn).unwrap();
    assert_eq!(bs.stage, Stage::Lost);
}
