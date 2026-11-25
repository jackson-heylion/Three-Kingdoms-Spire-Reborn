use three_kingdoms_spire_reborn::{
    Action, Difficulty, Game, Node, Stage,
    combat::EventKind,
    content::{Effect, Hero, card_spec},
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

/// Drive an expanded campaign cleanly using ONLY legal `Game::apply` actions so
/// `verify_action_replay()` and `historical_density_report()` stay 100% authentic.
fn play_to_floor_eight_echo(hero: Hero, seed: u64) -> (Game, Vec<String>) {
    let mut game = Game::new_with_campaign_difficulty(
        hero, seed, Difficulty::Warlords, format!("echo-{hero:?}-{seed}"),
    );
    let mut observed_receipts = Vec::new();
    let mut steps = 0_u32;
    while steps < 900 && !matches!(game.stage, Stage::Won | Stage::Lost) {
        if game.floor == 8 && game.last_node == Some(Node::Boss) && game.stage == Stage::Combat {
            let b = game.battle.as_ref().expect("floor 8 boss battle");
            if b.turn == 1 {
                // Play a block card first if available, then play any affordable cards to reach >=2 plays & >=6 block.
                let mut safety = 0;
                while safety < 10 && game.stage == Stage::Combat {
                    safety += 1;
                    let b = game.battle.as_ref().unwrap();
                    let need_block = hero != Hero::LuBu && b.block < 6;
                    let idx = if need_block {
                        b.hand.iter().position(|c| {
                            card_spec(&c.id).is_some_and(|s| {
                                s.cost <= b.energy
                                    && matches!(s.effect, Effect::Guard(_) | Effect::GuardEnergy(_) | Effect::XGuard(_))
                            })
                        }).or_else(|| {
                            b.hand.iter().position(|c| {
                                card_spec(&c.id).is_some_and(|s| {
                                    s.cost <= b.energy && !matches!(s.effect, Effect::Unplayable)
                                })
                            })
                        })
                    } else {
                        b.hand.iter().position(|c| {
                            card_spec(&c.id).is_some_and(|s| {
                                s.cost <= b.energy && !matches!(s.effect, Effect::Unplayable)
                            })
                        })
                    };
                    let Some(pos) = idx else { break; };
                    let target = b.foes.iter().position(|f| f.hp > 0).unwrap_or(0) + 1;
                    game.apply(Action::Play { index: pos + 1, target }).unwrap();
                }
                if game.stage == Stage::Combat {
                    game.apply(Action::EndTurn).unwrap();
                }
                break;
            }
        }
        let action = match game.stage {
            Stage::Map => {
                // Prefer Story early so the hero acquires Act-0 lore_flags naturally.
                if let Some(pos) = game.route.iter().position(|n| *n == Node::Story) {
                    Action::Choose { index: pos + 1 }
                } else if let Some(pos) = game.route.iter().position(|n| *n == Node::Boss) {
                    Action::Choose { index: pos + 1 }
                } else if let Some(pos) = game.route.iter().position(|n| matches!(n, Node::Camp | Node::Market)) {
                    Action::Choose { index: pos + 1 }
                } else {
                    Action::Choose { index: 1 }
                }
            }
            Stage::Story => Action::Choose { index: 1 },
            Stage::Camp => Action::Choose { index: if game.hp + 12 <= game.max_hp { 1 } else { 2 } },
            Stage::Market => Action::Choose { index: 0 },
            Stage::Reward => Action::Choose { index: usize::from(!game.rewards.is_empty()) },
            Stage::Ancient => Action::Choose { index: 1 },
            Stage::Combat => {
                let b = game.battle.as_ref().unwrap();
                let living_target = b.foes.iter().position(|f| f.hp > 0).map(|i| i + 1);
                if b.siege_gate_hp.is_some_and(|hp| hp > 0)
                    && (living_target.is_none() || b.energy >= 2)
                {
                    if b.energy >= 2 { Action::AssaultGate } else { Action::EndTurn }
                } else if let Some(target) = living_target {
                    let usable = |s: three_kingdoms_spire_reborn::content::CardSpec| {
                        s.cost <= b.energy
                            && !matches!(s.effect, Effect::Unplayable | Effect::TutorAttack | Effect::CopyDiscard)
                    };
                    if game.floor == 1 && b.turn >= 2 && b.block < 12
                        && let Some(pos) = b.hand.iter().position(|c| {
                            card_spec(&c.id).is_some_and(|s| usable(s)
                                && matches!(s.effect, Effect::Guard(_) | Effect::GuardEnergy(_) | Effect::XGuard(_)))
                        })
                    {
                        Action::Play { index: pos + 1, target }
                    } else if let Some(pos) = b.hand.iter().position(|c|
                        card_spec(&c.id).is_some_and(usable))
                    {
                        Action::Play { index: pos + 1, target }
                    } else {
                        Action::EndTurn
                    }
                } else {
                    Action::EndTurn
                }
            }
            Stage::Won | Stage::Lost => break,
        };
        game.apply(action).unwrap();
        steps += 1;
    }
    if let Ok(receipts) = game.historical_effect_receipts() {
        for r in receipts {
            observed_receipts.push(r.source);
        }
    }
    (game, observed_receipts)
}

#[test]
fn all_five_heroes_trigger_replay_verified_first_act_boss_echoes_and_keep_epoch_zero_invariance() {
    let expected = [
        (Hero::GuanYu, "taoyuan_oath", "taoyuan_oath_wall"),
        (Hero::ZhugeLiang, "longzhong_plan", "longzhong_plan_foresight"),
        (Hero::SunShangxiang, "sun_father_dispatch", "sun_father_vanguard_echo"),
        (Hero::CaoCao, "xu_capital", "xu_capital_court_command"),
        (Hero::LuBu, "red_hare", "red_hare_charge_impact"),
    ];
    for (hero, flag, source) in expected {
        // Test deterministic direct echo trigger AND verify epoch-0 save omits it.
        let mut modern = Game::new_with_campaign_difficulty(
            hero, 17, Difficulty::Warlords, format!("modern-echo-{hero:?}"),
        );
        modern.lore_flags.push(flag.into());
        modern.floor = 7;
        assert!(
            modern.route_objective_hint(Node::Boss).is_some_and(|s| s.contains("史事回响")),
            "{hero:?} floor 7 route hint should preview Act-1 Boss echo"
        );
        enter_node(&mut modern, 7, Node::Boss);
        assert!(
            modern.tactical_objective().is_some_and(|s| s.contains("史事回响")),
            "{hero:?} turn 1 should show echo objective"
        );
        {
            let b = modern.battle.as_mut().unwrap();
            b.turn = 1;
            b.played_this_turn = 2;
            b.block = 8;
        }
        let boss_hp_before = modern.battle.as_ref().unwrap().foes[0].hp;
        modern.apply(Action::EndTurn).unwrap();
        let b = modern.battle.as_ref().unwrap();
        assert!(
            b.events.iter().any(|e| e.kind == EventKind::PowerTriggered && e.source == source),
            "{hero:?} should emit PowerTriggered echo {source}"
        );
        assert!(
            b.foes[0].hp < boss_hp_before || b.foes[0].weak > 0 || b.foes[0].vulnerable > 0,
            "{hero:?} echo should affect boss on Turn 1"
        );

        let base_epoch0 = Game::new_with_campaign_difficulty(
            hero, 17, Difficulty::Warlords, format!("epoch0-echo-{hero:?}"),
        );
        let mut raw: serde_json::Value = serde_json::from_str(&base_epoch0.to_json().unwrap()).unwrap();
        raw.as_object_mut().unwrap().remove("expanded_rules_revision");
        raw["campaign_history"][0]["state"]["identity"]
            .as_object_mut().unwrap()
            .remove("expanded_rules_revision");
        let mut epoch0 = Game::from_json(&raw.to_string()).unwrap();
        epoch0.lore_flags.push(flag.into());
        enter_node(&mut epoch0, 7, Node::Boss);
        {
            let b = epoch0.battle.as_mut().unwrap();
            b.turn = 1;
            b.played_this_turn = 2;
            b.block = 8;
        }
        epoch0.apply(Action::EndTurn).unwrap();
        assert!(
            !epoch0.battle.as_ref().unwrap().events.iter().any(|e| e.source == source),
            "epoch-0 campaign must ignore expanded combat echoes for {hero:?}"
        );
    }

    // Also run authentic full-action Act-0 -> Floor-8 campaigns across all 5 heroes and verify replay + receipts!
    let all_heroes = [Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang, Hero::CaoCao, Hero::LuBu];
    let mut receipt_heroes = 0_usize;
    for hero in all_heroes {
        let mut hit = false;
        for seed in [7_u64, 17, 29, 43, 89, 101] {
            let (game, receipts) = play_to_floor_eight_echo(hero, seed);
            game.verify_action_replay().expect("action replay must succeed");
            if !receipts.is_empty() {
                let density = game.historical_density_report().unwrap();
                assert!(density.covered_lower_bound >= 1);
                hit = true;
                break;
            }
        }
        if hit { receipt_heroes += 1; }
    }
    assert_eq!(receipt_heroes, all_heroes.len(), "every hero should reach and replay-verify a natural Floor 8 echo receipt");
}

#[test]
fn late_act_hero_echoes_and_route_hints_cover_guanyu_cao_lubu_and_sun() {
    // SunShangxiang Act-3 crossing jiangdong_covenant advertises +2 cover at floor 18 (not at boss 23).
    let mut sun = Game::new_with_campaign_difficulty(Hero::SunShangxiang, 9, Difficulty::Warlords, "sun-cov".into());
    sun.lore_flags.push("jiangdong_covenant".into());
    sun.floor = 18;
    let sun18 = sun.route_objective_hint(Node::Battle).unwrap();
    assert!(sun18.contains("江东盟约") && sun18.contains("2掩护"), "sun18: {sun18}");
    sun.floor = 23;
    let sun23 = sun.route_objective_hint(Node::Boss).unwrap();
    assert!(sun23.contains("江东盟约") && !sun23.contains("4掩护"), "sun23: {sun23}");
    enter_node(&mut sun, 18, Node::Battle);
    {
        let b = sun.battle.as_mut().unwrap();
        b.played_this_turn = 2;
        b.convoy_cover = Some(4);
    }
    let hp_before = sun.battle.as_ref().unwrap().foes[0].hp;
    sun.apply(Action::EndTurn).unwrap();
    let b = sun.battle.as_ref().unwrap();
    assert!(b.events.iter().any(|e| e.source == "jiangdong_covenant_volleys"));
    assert!(b.foes[0].hp < hp_before);

    // LuBu Act-3/4 halberd_shot MUST NOT trigger passively with 0 played cards (even at Fury 6),
    // and triggers cleanly once >= 1 card is played.
    let mut lubu = Game::new_with_campaign_difficulty(Hero::LuBu, 9, Difficulty::Warlords, "lubu-halb".into());
    lubu.lore_flags.push("halberd_shot".into());
    lubu.floor = 23;
    assert!(lubu.route_objective_hint(Node::Boss).unwrap().contains("辕门射戟"));
    enter_node(&mut lubu, 23, Node::Boss);
    lubu.battle.as_mut().unwrap().fury = 6;
    lubu.battle.as_mut().unwrap().played_this_turn = 0;
    lubu.apply(Action::EndTurn).unwrap();
    assert!(
        !lubu.battle.as_ref().unwrap().events.iter().any(|e| e.source == "halberd_shot_deterrence"),
        "halberd_shot must not fire at 0 cards played"
    );
    lubu.battle.as_mut().unwrap().turn = 1;
    lubu.battle.as_mut().unwrap().played_this_turn = 1;
    lubu.battle.as_mut().unwrap().fury = 6;
    lubu.apply(Action::EndTurn).unwrap();
    assert!(lubu.battle.as_ref().unwrap().events.iter().any(|e| e.source == "halberd_shot_deterrence"));

    // CaoCao sevenstar_vow vs guandu_grain show truthful separate hints and weak >= 2 survives vs Yuan Shao Turn 1.
    let mut cao_only_knife = Game::new_with_campaign_difficulty(Hero::CaoCao, 9, Difficulty::Warlords, "cao-knife".into());
    cao_only_knife.lore_flags.push("sevenstar_vow".into());
    cao_only_knife.floor = 15;
    let knife_hint = cao_only_knife.route_objective_hint(Node::Boss).unwrap();
    assert!(knife_hint.contains("穿透敌首领5点") && !knife_hint.contains("虚弱"), "knife_hint: {knife_hint}");

    let mut cao = Game::new_with_campaign_difficulty(Hero::CaoCao, 9, Difficulty::Warlords, "cao-guandu".into());
    cao.apply(Action::SetExpandedVariants { act1: 0, act2: 1, act3: 0, act4: 0, act5: 0 }).unwrap();
    cao.lore_flags.extend(["sevenstar_vow".into(), "guandu_grain".into()]);
    cao.floor = 15;
    let combined = cao.route_objective_hint(Node::Boss).unwrap();
    assert!(combined.contains("七星宝刀") && combined.contains("官渡筹粮"));
    enter_node(&mut cao, 15, Node::Boss);
    cao.battle.as_mut().unwrap().played_this_turn = 2;
    cao.apply(Action::EndTurn).unwrap();
    let cao_b = cao.battle.as_ref().unwrap();
    assert!(cao_b.events.iter().any(|e| e.source == "sevenstar_assassination_strike"));
    assert!(cao_b.events.iter().any(|e| e.source == "guandu_grain_reserve_receipt"));
    assert!(cao_b.foes[0].weak >= 1, "weak=2 must leave >= 1 weak after Turn 1 vs Yuan Shao");

    // LuBu Act-2 phoenix_pavilion.
    let mut lubu2 = Game::new_with_campaign_difficulty(Hero::LuBu, 9, Difficulty::Warlords, "lubu-phx".into());
    lubu2.lore_flags.push("phoenix_pavilion".into());
    enter_node(&mut lubu2, 15, Node::Boss);
    lubu2.battle.as_mut().unwrap().played_this_turn = 2;
    lubu2.apply(Action::EndTurn).unwrap();
    assert!(lubu2.battle.as_ref().unwrap().events.iter().any(|e| e.source == "phoenix_pavilion_intrigue"));
    assert!(lubu2.battle.as_ref().unwrap().foes[0].weak >= 1);

    // GuanYu Act-2 huarong_mercy + SunShangxiang Act-4 river_divide.
    let mut guan = Game::new_with_campaign_difficulty(Hero::GuanYu, 9, Difficulty::Warlords, "guan-huarong".into());
    guan.lore_flags.push("huarong_mercy".into());
    enter_node(&mut guan, 15, Node::Boss);
    {
        let b = guan.battle.as_mut().unwrap();
        b.played_this_turn = 2;
        b.block = 8;
    }
    guan.apply(Action::EndTurn).unwrap();
    assert!(guan.battle.as_ref().unwrap().events.iter().any(|e| e.source == "huarong_mercy_release"));

    let mut sun4 = Game::new_with_campaign_difficulty(Hero::SunShangxiang, 9, Difficulty::Warlords, "sun-river".into());
    sun4.lore_flags.push("river_divide".into());
    enter_node(&mut sun4, 31, Node::Boss);
    {
        let b = sun4.battle.as_mut().unwrap();
        b.played_this_turn = 2;
        b.block = 8;
    }
    sun4.apply(Action::EndTurn).unwrap();
    assert!(sun4.battle.as_ref().unwrap().events.iter().any(|e| e.source == "river_divide_interception"));
}

#[test]
fn defeat_diagnosis_distinguishes_timeout_convoy_exhaustion_and_lethal_combat() {
    // 1. Breakout timeout (floor 1, hp > 0).
    let mut breakout = Game::new_with_campaign_difficulty(Hero::GuanYu, 7, Difficulty::Warlords, "diag-breakout".into());
    enter_node(&mut breakout, 0, Node::Battle);
    for _ in 0..4 {
        if breakout.stage != Stage::Combat { break; }
        breakout.hp = breakout.max_hp;
        breakout.apply(Action::EndTurn).unwrap();
    }
    assert_eq!(breakout.stage, Stage::Lost);
    assert!(breakout.defeat_diagnosis().unwrap().contains("突围超时"));

    // 2. Convoy failure (floor 2, hp > 0).
    let mut convoy = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "diag-convoy".into());
    enter_node(&mut convoy, 1, Node::Battle);
    for _ in 0..5 {
        if convoy.stage != Stage::Combat { break; }
        convoy.hp = convoy.max_hp;
        convoy.apply(Action::EndTurn).unwrap();
    }
    assert_eq!(convoy.stage, Stage::Lost);
    assert!(convoy.defeat_diagnosis().unwrap().contains("粮道失守"));

    // 3. Siege timeout (floor 4, hp > 0).
    let mut siege = Game::new_with_campaign_difficulty(Hero::LuBu, 7, Difficulty::Warlords, "diag-siege".into());
    enter_node(&mut siege, 3, Node::Battle);
    for _ in 0..5 {
        if siege.stage != Stage::Combat { break; }
        siege.hp = siege.max_hp;
        siege.apply(Action::EndTurn).unwrap();
    }
    assert_eq!(siege.stage, Stage::Lost);
    assert!(siege.defeat_diagnosis().unwrap().contains("攻城超时"));

    // 4. Supply exhaustion on Turn >= 40.
    let mut fatigue = Game::new_with_campaign_difficulty(Hero::ZhugeLiang, 7, Difficulty::Warlords, "diag-fatigue".into());
    enter_node(&mut fatigue, 7, Node::Boss);
    fatigue.hp = 1;
    fatigue.battle.as_mut().unwrap().turn = 40;
    fatigue.battle.as_mut().unwrap().foes[0].intent = three_kingdoms_spire_reborn::Intent::Guard(1);
    fatigue.apply(Action::EndTurn).unwrap();
    assert_eq!(fatigue.stage, Stage::Lost);
    assert!(fatigue.defeat_diagnosis().unwrap().contains("粮尽"));

    // 5. Lethal foe attack (hp == 0).
    let mut lethal = Game::new_with_campaign_difficulty(Hero::SunShangxiang, 7, Difficulty::Warlords, "diag-hit".into());
    enter_node(&mut lethal, 7, Node::Boss);
    lethal.hp = 1;
    lethal.battle.as_mut().unwrap().foes[0].intent = three_kingdoms_spire_reborn::Intent::Attack(20);
    lethal.apply(Action::EndTurn).unwrap();
    assert_eq!(lethal.stage, Stage::Lost);
    assert!(lethal.defeat_diagnosis().unwrap().contains("致命打击"));
    // Non-Lost stage has no defeat diagnosis.
    let active = Game::new_with_campaign_difficulty(Hero::GuanYu, 7, Difficulty::Warlords, "diag-active".into());
    assert_eq!(active.defeat_diagnosis(), None);

    // 6. Water depletion (floor 26 ZhugeLiang), Naval hull sunk (floor 19), Wildfire (burning_camp),
    //    Fancheng flood exposure (floor 27 GuanYu), and Story death after earlier damaged battle!
    let mut water = Game::new_with_campaign_difficulty(Hero::ZhugeLiang, 7, Difficulty::Warlords, "diag-water".into());
    water.gold = 100;
    enter_node(&mut water, 24, Node::Story);
    water.apply(Action::Choose { index: 4 }).unwrap();
    water.apply(Action::Choose { index: 1 }).unwrap();
    enter_node(&mut water, 25, Node::Battle);
    water.battle.as_mut().unwrap().convoy_hp = Some(1);
    water.battle.as_mut().unwrap().convoy_cover = Some(0);
    water.apply(Action::EndTurn).unwrap();
    assert!(water.defeat_diagnosis().unwrap().contains("街亭水源失守"));

    let mut hull = Game::new_with_campaign_difficulty(Hero::SunShangxiang, 7, Difficulty::Warlords, "diag-naval".into());
    enter_node(&mut hull, 18, Node::Battle);
    hull.battle.as_mut().unwrap().convoy_hp = Some(1);
    hull.battle.as_mut().unwrap().convoy_cover = Some(0);
    hull.apply(Action::EndTurn).unwrap();
    assert!(hull.defeat_diagnosis().unwrap().contains("战船耐久耗尽"));

    let mut burn = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "diag-burn".into());
    enter_node(&mut burn, 7, Node::Boss);
    burn.hp = 2;
    burn.battle.as_mut().unwrap().wildfire = 2;
    burn.apply(Action::EndTurn).unwrap();
    assert!(burn.defeat_diagnosis().unwrap().contains("焚营火势"));

    let mut story_death = Game::new_with_campaign_difficulty(Hero::GuanYu, 7, Difficulty::Warlords, "diag-story".into());
    enter_node(&mut story_death, 0, Node::Battle);
    story_death.apply(Action::EndTurn).unwrap(); // takes genuine enemy damage in event_history
    for f in &mut story_death.battle.as_mut().unwrap().foes { f.hp = 1; }
    let pos = story_death.battle.as_ref().unwrap().hand.iter().position(|c|
        card_spec(&c.id).is_some_and(|s| matches!(s.effect, Effect::Hit(_) | Effect::HitDraw(_, _) | Effect::HitTwice(_))))
        .unwrap();
    story_death.apply(Action::Play { index: pos + 1, target: 1 }).unwrap();
    if story_death.stage == Stage::Reward {
        story_death.apply(Action::Choose { index: 0 }).unwrap();
    }
    enter_node(&mut story_death, 1, Node::Story);
    story_death.hp = 1;
    if story_death.lore_current.is_some() {
        story_death.apply(Action::Choose { index: 2 }).unwrap();
    } else {
        story_death.event_id = 1;
        story_death.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(story_death.stage, Stage::Lost);
    let story_diag = story_death.defeat_diagnosis().unwrap();
    assert!(story_diag.contains("驿道抉择消耗体力至归零"), "story_diag: {story_diag}");
    assert!(!story_diag.contains("致命打击"), "must not confuse earlier won battle damage: {story_diag}");

    // 7. Fancheng flood exposure diagnosis (floor 27 GuanYu) + late-acquired echo beat attribution.
    let mut flood = Game::new_with_campaign_difficulty(Hero::GuanYu, 7, Difficulty::Warlords, "diag-flood".into());
    flood.lore_flags.push("fancheng_flood_record".into());
    enter_node(&mut flood, 26, Node::Battle);
    flood.hp = 2;
    {
        let b = flood.battle.as_mut().unwrap();
        b.turn = 2;
        b.block = 0;
        b.foes.truncate(1);
        b.foes[0].intent = three_kingdoms_spire_reborn::Intent::Guard(1);
    }
    flood.apply(Action::EndTurn).unwrap();
    assert_eq!(flood.stage, Stage::Lost);
    assert!(flood.defeat_diagnosis().unwrap().contains("樊城水势"));
}
