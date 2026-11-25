use three_kingdoms_spire_reborn::{
    Action, Card, Difficulty, Game, Node, Stage, XiangfanPlan, XiapiPlan,
    content::Hero,
    lore, map,
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
fn market_expanded_defend_trim_and_training_upgrade_preserve_legacy() {
    let mut expanded = Game::new_with_campaign_difficulty(
        Hero::CaoCao, 11, Difficulty::Warlords, "mkt-exp".into(),
    );
    expanded.stage = Stage::Market;
    expanded.gold = 300;
    expanded.deck = vec![
        Card::new("strike"),
        Card::new("defend"),
        Card::new("cursed"),
        Card::new("quest_tulu"),
    ];
    assert_eq!(expanded.market_upgrade_price(), Some(110));

    // Step 1: option 6 removes cursed first.
    let msg = expanded.apply(Action::Choose { index: 6 }).unwrap().join("；");
    assert!(msg.contains("75金"), "{msg}");
    assert_eq!(expanded.gold, 225);
    assert!(!expanded.deck.iter().any(|c| c.id == "cursed"));

    // Step 2: option 6 removes strike next.
    expanded.apply(Action::Choose { index: 6 }).unwrap();
    assert_eq!(expanded.gold, 150);
    assert!(!expanded.deck.iter().any(|c| c.id == "strike"));

    // Cannot remove the last playable card ("defend" when only quest_tulu remains).
    let err = expanded.apply(Action::Choose { index: 6 }).unwrap_err();
    assert!(err.contains("保留一张可打出的牌"), "{err}");

    // Add another defend so removing a spare defend works cleanly.
    expanded.deck.push(Card::new("defend"));
    let msg = expanded.apply(Action::Choose { index: 6 }).unwrap().join("；");
    assert!(msg.contains("75金"), "{msg}");
    assert_eq!(expanded.gold, 75);

    // 75g < 110g fails training upgrade cleanly.
    let err = expanded.apply(Action::Choose { index: 7 }).unwrap_err();
    assert!(err.contains("110金"), "{err}");

    // With 240g, option 7 upgrades defend (never quest_tulu), deducts 110g, and caps at 1 per Market.
    expanded.gold = 240;
    let msg = expanded.apply(Action::Choose { index: 7 }).unwrap().join("；");
    assert!(msg.contains("110金整训升级"), "{msg}");
    assert_eq!(expanded.gold, 130);
    assert!(expanded.market_upgraded);
    assert!(expanded.deck.iter().any(|c| c.id == "defend" && c.upgraded));
    assert!(expanded.deck.iter().all(|c| c.id != "quest_tulu" || !c.upgraded));
    assert_eq!(expanded.apply(Action::Choose { index: 7 }).unwrap_err(), "本处军市整训名额已用完");

    // Legacy 21-floor retains original market options and prices.
    let mut legacy = Game::new(Hero::CaoCao, 11);
    legacy.stage = Stage::Market;
    legacy.gold = 300;
    legacy.deck = vec![Card::new("defend"), Card::new("defend")];
    assert_eq!(legacy.market_upgrade_price(), None);
    assert_eq!(legacy.apply(Action::Choose { index: 7 }).unwrap_err(), "军市选项不存在");
    assert!(legacy.apply(Action::Choose { index: 6 }).unwrap_err().contains("无可快捷移除卡牌"));
}

#[test]
fn next_expanded_beat_covers_all_five_acts_without_mutating_legacy_beats() {
    // LuBu has only 9 hero beats in BEATS (act 0..=2); legacy returns None in acts 3-4.
    assert!(lore::next_beat(Hero::LuBu, 3, &[], &[]).is_none());
    assert!(lore::next_beat(Hero::LuBu, 4, &[], &[]).is_none());
    assert!(lore::BEATS.iter().all(|b| b.act <= 2));

    // Realistic progression: two Act-0 Story nodes unlock Act-1..2 hero chain,
    // and Acts 4–5 keep supplying distinct unseen beats without running empty.
    let mut seen: Vec<String> = Vec::new();
    let mut flags: Vec<String> = Vec::new();
    for act in [0_u8, 0, 1, 2, 3, 4] {
        let beat = lore::next_expanded_beat(Hero::LuBu, act, &flags, &seen)
            .expect("expanded acts should supply distinct literary beats");
        assert!(!seen.contains(&beat.id.to_string()));
        seen.push(beat.id.to_string());
        flags.push(beat.flag.to_string());
    }
    assert_eq!(seen.len(), 6);

    // Even if Act 0 only visited one Story node ("redhare"), Acts 4-5 recover the missed
    // hero chain ahead of shared archival beats.
    let recovered = lore::next_expanded_beat(
        Hero::LuBu, 3, &["red_hare".into()], &["redhare".into()],
    ).expect("Acts 4-5 should recover missed LuBu Act-0 continuation");
    assert_eq!(recovered.id, "pavilion");

    // Exhausted Act-2 hero beats with a skipped Act-1 hero beat still prioritize hero beats first.
    let skipped_hero_beat = lore::next_expanded_beat(
        Hero::LuBu,
        3,
        &["red_hare".into(), "phoenix_pavilion".into()],
        &["redhare".into(), "pavilion".into(), "whitegate".into()],
    )
    .expect("should recover skipped LuBu hero beat before shared archival beats");
    assert_eq!(skipped_hero_beat.hero, Some(Hero::LuBu));
}

#[test]
fn xiapi_and_xiangfan_councils_commit_save_and_settle_across_battles_and_bosses() {
    // Xiapi council (LuBu, Act 3-4)
    let mut lubu = Game::new_with_campaign_difficulty(
        Hero::LuBu, 7, Difficulty::Warlords, "xiapi-test".into(),
    );
    lubu.apply(Action::SetExpandedVariants { act1: 0, act2: 0, act3: 0, act4: 0, act5: 0 }).unwrap();
    lubu.floor = 18;
    lubu.stage = Stage::Story;
    lubu.lore_current = None;
    lubu.event_id = 0;
    assert!(lubu.story_fourth_option().is_some());
    assert!(lubu.lore_hint().contains("下邳破围军议"));

    // Open council with 4 -> subchoice 1 (Split) costs 16g
    let gold_before = lubu.gold;
    lubu.apply(Action::Choose { index: 4 }).unwrap();
    assert_eq!(lubu.xiapi_plan, Some(XiapiPlan::Council));
    assert!(lubu.lore_hint().contains("分兵夹击突围"));
    lubu.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(lubu.gold, gold_before - 16);
    assert_eq!(lubu.xiapi_plan, Some(XiapiPlan::SplitPending));
    assert_eq!(lubu.stage, Stage::Map);

    // Save/load roundtrip preserves XiapiPlan and route preview
    let mut skirmish = Game::from_json(&lubu.to_json().unwrap()).unwrap();
    assert_eq!(skirmish.xiapi_plan, Some(XiapiPlan::SplitPending));
    let route_hint = skirmish.route_objective_hint(Node::Battle).unwrap();
    assert!(route_hint.contains("下邳军议"), "{route_hint}");

    // Enter skirmish on floor 20 -> resolves to SplitResolved + weak + fury + objective
    enter_node(&mut skirmish, 19, Node::Battle);
    assert_eq!(skirmish.xiapi_plan, Some(XiapiPlan::SplitResolved));
    {
        let b = skirmish.battle.as_ref().unwrap();
        assert!(b.foes[0].weak >= 2 && b.vulnerable >= 1 && b.fury >= 2);
    }
    assert!(skirmish.tactical_objective().unwrap().contains("下邳破围"));

    // Separate Act-3 boss checkpoint with SplitResolved -> settles cleanly to SplitSettled
    let mut lubu_boss = Game::new_with_campaign_difficulty(
        Hero::LuBu, 7, Difficulty::Warlords, "xiapi-boss".into(),
    );
    lubu_boss.apply(Action::SetExpandedVariants { act1: 0, act2: 0, act3: 0, act4: 0, act5: 0 }).unwrap();
    lubu_boss.xiapi_plan = Some(XiapiPlan::SplitResolved);
    enter_node(&mut lubu_boss, 23, Node::Boss);
    assert_eq!(lubu_boss.xiapi_plan, Some(XiapiPlan::SplitSettled));
    assert!(lubu_boss.tactical_objective().unwrap().contains("下邳决战"));

    // Entering Boss directly with ChargePending settles Fury (+4) without extra skirmish draw (+1).
    let mut charge_boss = Game::new_with_campaign_difficulty(
        Hero::LuBu, 7, Difficulty::Warlords, "xiapi-charge-boss".into(),
    );
    charge_boss.apply(Action::SetExpandedVariants { act1: 0, act2: 0, act3: 0, act4: 0, act5: 0 }).unwrap();
    charge_boss.xiapi_plan = Some(XiapiPlan::ChargePending);
    enter_node(&mut charge_boss, 23, Node::Boss);
    assert_eq!(charge_boss.xiapi_plan, Some(XiapiPlan::ChargeSettled));
    let cb = charge_boss.battle.as_ref().unwrap();
    assert_eq!(cb.hand.len(), 5, "ChargePending settled straight at Boss must not draw an extra skirmish card");
    assert!(cb.fury >= 4);

    // Xiangfan council (GuanYu, Act 4)
    let mut guanyu = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 7, Difficulty::Warlords, "xiangfan-test".into(),
    );
    guanyu.apply(Action::SetExpandedVariants { act1: 0, act2: 0, act3: 0, act4: 1, act5: 0 }).unwrap();
    guanyu.floor = 26;
    guanyu.stage = Stage::Story;
    guanyu.lore_current = None;
    guanyu.event_id = 0;
    assert!(guanyu.story_fourth_option().is_some());
    guanyu.apply(Action::Choose { index: 4 }).unwrap();
    assert_eq!(guanyu.xiangfan_plan, Some(XiangfanPlan::Council));
    guanyu.apply(Action::Choose { index: 2 }).unwrap(); // DikePending (-6 HP)
    assert_eq!(guanyu.xiangfan_plan, Some(XiangfanPlan::DikePending));
    assert!(guanyu.route_objective_hint(Node::Battle).unwrap().contains("筑堤固守"));

    enter_node(&mut guanyu, 26, Node::Battle);
    assert_eq!(guanyu.xiangfan_plan, Some(XiangfanPlan::DikeResolved));
    assert!(guanyu.battle.as_ref().unwrap().block >= 12);

    let mut guanyu_boss = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 7, Difficulty::Warlords, "xiangfan-boss".into(),
    );
    guanyu_boss.apply(Action::SetExpandedVariants { act1: 0, act2: 0, act3: 0, act4: 1, act5: 0 }).unwrap();
    guanyu_boss.xiangfan_plan = Some(XiangfanPlan::DikeResolved);
    guanyu_boss.lore_flags.push("jingzhou_warning".into());
    enter_node(&mut guanyu_boss, 31, Node::Boss);
    assert_eq!(guanyu_boss.xiangfan_plan, Some(XiangfanPlan::DikeSettled));
    let b32 = guanyu_boss.battle.as_ref().unwrap();
    assert!(b32.block >= 14 && b32.thorns >= 3);
    let obj32 = guanyu_boss.tactical_objective().unwrap();
    assert!(obj32.contains("荆州警戒") && obj32.contains("襄樊水防"), "{obj32}");

    // FloodResolved deducts up to 10 Boss HP and inflicts weak on floor 32 Boss.
    let mut flood_boss = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 7, Difficulty::Warlords, "xiangfan-flood-boss".into(),
    );
    flood_boss.apply(Action::SetExpandedVariants { act1: 0, act2: 0, act3: 0, act4: 1, act5: 0 }).unwrap();
    flood_boss.xiangfan_plan = Some(XiangfanPlan::FloodResolved);
    enter_node(&mut flood_boss, 31, Node::Boss);
    assert_eq!(flood_boss.xiangfan_plan, Some(XiangfanPlan::FloodSettled));
    let fb = flood_boss.battle.as_ref().unwrap();
    assert_eq!(fb.foes[0].hp, fb.foes[0].max_hp - 10);
    assert!(fb.foes[0].weak >= 2);
}
