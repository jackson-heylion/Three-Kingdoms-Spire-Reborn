use std::collections::BTreeSet;
use three_kingdoms_spire_reborn::{
    Action, BossMove, Card, Difficulty, Foe, Game, Intent, Node, Stage,
    combat::EventKind,
    content::Hero,
    map,
};

fn enter_node(game: &mut Game, completed_floor: u8, node: Node) {
    game.floor = completed_floor;
    game.stage = Stage::Map;
    let act = completed_floor / 8;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, act));
    game.map_lane = None;
    game.route = vec![node];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).unwrap();
}

#[test]
fn all_ten_expanded_bosses_and_five_elites_scale_and_trigger_second_phase() {
    let cases = [
        (0_u8, 0_u8, 8_u8, "zhangjiao", true),
        (0, 1, 8, "sunjian", false),
        (1, 0, 16, "dongzhuo", false),
        (1, 1, 16, "yuanshao", true),
        (2, 0, 24, "simayi", false),
        (2, 1, 24, "zuoci", true),
        (3, 0, 32, "luxun", false),
        (3, 1, 32, "lumeng", false),
        (4, 0, 40, "jiangwei", false),
        (4, 1, 40, "dengai", false),
    ];
    for (act, branch, floor, expected_id, summons_add) in cases {
        let mut hps = Vec::new();
        for difficulty in Difficulty::ALL {
            let mut game = Game::new_with_campaign_difficulty(
                Hero::CaoCao, 7, difficulty, format!("boss-{expected_id}-{difficulty:?}"),
            );
            game.apply(Action::SetExpandedVariants {
                act1: u8::from(act == 0 && branch == 1),
                act2: u8::from(act == 1 && branch == 1),
                act3: u8::from(act == 2 && branch == 1),
                act4: u8::from(act == 3 && branch == 1),
                act5: u8::from(act == 4 && branch == 1),
            }).unwrap();
            enter_node(&mut game, floor - 1, Node::Boss);

            assert!(game.phase_transition_hint(0).is_some(), "{expected_id} should advertise phase 2");
            let max_hp = {
                let b = game.battle.as_mut().expect("boss battle");
                assert_eq!(b.foes[0].id, expected_id, "floor {floor} variant {branch}");
                let hp = b.foes[0].max_hp;
                b.foes[0].hp = (hp / 2).max(1);
                b.foes[0].intent = Intent::Guard(1);
                hp
            };
            hps.push(max_hp);
            game.hp = game.max_hp;
            game.apply(Action::EndTurn).unwrap();

            let b = game.battle.as_ref().expect("post-phase2 battle");
            assert_eq!(b.foes[0].phase, 1, "{expected_id} should enter phase 2 on {difficulty:?}");
            if summons_add && expected_id != "yuanshao" {
                assert!(b.foes.len() >= 2, "{expected_id} should summon reinforcement in phase 2");
                assert!(b.events.iter().any(|e| e.kind == EventKind::FoeSummoned),
                    "{expected_id} should emit FoeSummoned");
            }
        }
        assert!(hps[0] < hps[1] && hps[1] < hps[2],
            "{expected_id} HP should scale strictly across difficulties: {hps:?}");
    }

    // All 5 elite archetypes advertise and enter phase 2 cleanly.
    for elite_id in ["elite", "war_elephant", "royal_guard", "tiger_general", "sacred_beast"] {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::LuBu, 13, Difficulty::Warlords, format!("elite-{elite_id}"),
        );
        enter_node(&mut game, 11, Node::Elite);
        let foe_hp = game.battle.as_ref().unwrap().foes[0].max_hp;
        {
            let b = game.battle.as_mut().unwrap();
            b.foes = vec![Foe {
                id: elite_id.into(),
                hp: (foe_hp / 2).max(1),
                max_hp: foe_hp,
                block: 0,
                strength: 0,
                poison: 0,
                vulnerable: 0,
                weak: 0,
                intent: Intent::Guard(1),
                phase: 0,
            }];
        }
        assert!(game.phase_transition_hint(0).is_some(), "{elite_id} should advertise phase 2");
        game.apply(Action::EndTurn).unwrap();
        assert_eq!(game.battle.as_ref().unwrap().foes[0].phase, 1, "{elite_id} phase 2 failed");
    }
}

#[test]
fn zuoci_star_omen_activates_naturally_for_expanded_history() {
    let mut expanded = Game::new_with_campaign_difficulty(
        Hero::CaoCao, 7, Difficulty::Warlords, "zuoci-omen".into(),
    );
    expanded.apply(Action::SetExpandedVariants {
        act1: 0, act2: 0, act3: 1, act4: 0, act5: 0,
    }).unwrap();
    enter_node(&mut expanded, 23, Node::Boss);
    assert_eq!(expanded.battle.as_ref().unwrap().foes[0].id, "zuoci");

    // Advance turns until BossMove::StarOmen rolls naturally (turn > 1 with < 5 cards played).
    let mut saw_star_omen = false;
    for _ in 0..8 {
        if let Intent::Boss(BossMove::StarOmen(0)) = expanded.battle.as_ref().unwrap().foes[0].intent {
            saw_star_omen = true;
            break;
        }
        expanded.hp = expanded.max_hp;
        expanded.apply(Action::EndTurn).unwrap();
    }
    assert!(saw_star_omen, "Act-3 Zuoci should naturally roll StarOmen on later turns");

    // Failing the 9-block threshold adds armor + curse capped at 4 across piles.
    expanded.battle.as_mut().unwrap().block = 0;
    expanded.apply(Action::EndTurn).unwrap();
    {
        let b = expanded.battle.as_ref().unwrap();
        assert!(b.foes[0].block >= 8, "failed StarOmen should fortify Zuoci");
        let curse_count = b.draw.iter().chain(&b.hand).chain(&b.discard)
            .filter(|c| c.id == "cursed").count();
        assert!((1..=4).contains(&curse_count), "unexpected curse count: {curse_count}");
    }

    // Meeting 9 block counters StarOmen and leaves Zuoci vulnerable.
    {
        let b = expanded.battle.as_mut().unwrap();
        b.foes[0].intent = Intent::Boss(BossMove::StarOmen(0));
        b.foes[0].vulnerable = 0;
        b.block = 9;
    }
    expanded.apply(Action::EndTurn).unwrap();
    assert!(expanded.battle.as_ref().unwrap().foes[0].vulnerable >= 1,
        "9+ block should counter StarOmen(0) and inflict vulnerable");

    // Four existing curses prevent adding a fifth curse.
    {
        let b = expanded.battle.as_mut().unwrap();
        b.foes[0].intent = Intent::Boss(BossMove::StarOmen(0));
        b.block = 0;
        for _ in 0..4 { b.discard.push(Card::new("cursed")); }
    }
    let before_curses = {
        let b = expanded.battle.as_ref().unwrap();
        b.draw.iter().chain(&b.hand).chain(&b.discard).filter(|c| c.id == "cursed").count()
    };
    expanded.apply(Action::EndTurn).unwrap();
    let after_curses = {
        let b = expanded.battle.as_ref().unwrap();
        b.draw.iter().chain(&b.hand).chain(&b.discard).filter(|c| c.id == "cursed").count()
    };
    assert_eq!(after_curses, before_curses, "active curse pool cap of 4 must hold");
}

#[test]
fn act_four_foe_pools_distinguish_luxun_fire_vanguard_and_lumeng_defense_lines() {
    let collect_foes = |variant: u8| {
        let mut ids = BTreeSet::new();
        for seed in 1..=20_u64 {
            let mut g = Game::new_with_campaign_difficulty(
                Hero::SunShangxiang, seed, Difficulty::Warlords, format!("pool-{variant}-{seed}"),
            );
            g.apply(Action::SetExpandedVariants {
                act1: 0, act2: 0, act3: 0, act4: variant, act5: 0,
            }).unwrap();
            enter_node(&mut g, 25, Node::Battle);
            for foe in &g.battle.unwrap().foes {
                ids.insert(foe.id.clone());
            }
        }
        ids
    };

    let luxun_pool = collect_foes(0);
    let lumeng_pool = collect_foes(1);
    assert!(luxun_pool.contains("pyromancer"), "Luxun fire pool should include pyromancer: {luxun_pool:?}");
    assert!(lumeng_pool.contains("iron_guard") || lumeng_pool.contains("tactician"),
        "Lumeng defense pool should include iron_guard/tactician: {lumeng_pool:?}");
    assert_ne!(luxun_pool, lumeng_pool, "Act-4 pools should vary by commander branch");
}

#[test]
fn legacy_twenty_one_floor_compatibility_and_forged_late_plan_rejection() {
    let legacy = Game::new(Hero::GuanYu, 7);
    assert!(legacy.difficulty.is_none());
    assert_eq!(legacy.campaign_max_floor(), 21);
    assert_eq!(legacy.market_upgrade_price(), None);
    assert_eq!(legacy.xiapi_plan, None);
    assert_eq!(legacy.xiangfan_plan, None);
    assert!(legacy.historical_density_report().is_err());
    assert_eq!(legacy.historical_archive_report().unwrap(), "旧制战役保留原演义纪事；新制战史档案不适用。");

    // Legacy & mismatched-hero snapshots carrying Xiapi/Xiangfan/market_upgraded must be rejected.
    let mut forged_xiapi: serde_json::Value = serde_json::from_str(&legacy.to_json().unwrap()).unwrap();
    forged_xiapi["xiapi_plan"] = serde_json::json!("SplitPending");
    assert!(Game::from_json(&forged_xiapi.to_string()).is_err());

    let mut forged_market: serde_json::Value = serde_json::from_str(&legacy.to_json().unwrap()).unwrap();
    forged_market["market_upgraded"] = serde_json::json!(true);
    assert!(Game::from_json(&forged_market.to_string()).is_err());

    let guanyu = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 7, Difficulty::Warlords, "forged-xiangfan".into(),
    );
    let mut early_xiangfan: serde_json::Value = serde_json::from_str(&guanyu.to_json().unwrap()).unwrap();
    early_xiangfan["xiangfan_plan"] = serde_json::json!("Council");
    assert!(Game::from_json(&early_xiangfan.to_string()).is_err());
}
