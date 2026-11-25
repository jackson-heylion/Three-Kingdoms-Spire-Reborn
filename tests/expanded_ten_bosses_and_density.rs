use three_kingdoms_spire_reborn::{
    Action, Card, Difficulty, Game, Intent, Node, Stage,
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

#[test]
fn all_ten_expanded_bosses_settle_phase_two_victory_and_defeat_diagnostically() {
    let cases = [
        (0_u8, 0_u8, 8_u8, "zhangjiao"),
        (0, 1, 8, "sunjian"),
        (1, 0, 16, "dongzhuo"),
        (1, 1, 16, "yuanshao"),
        (2, 0, 24, "simayi"),
        (2, 1, 24, "zuoci"),
        (3, 0, 32, "luxun"),
        (3, 1, 32, "lumeng"),
        (4, 0, 40, "jiangwei"),
        (4, 1, 40, "dengai"),
    ];
    for (act, branch, floor, expected_id) in cases {
        let mut g = Game::new_with_campaign_difficulty(
            Hero::CaoCao, 19, Difficulty::Warlords, format!("boss-action-{expected_id}"),
        );
        g.apply(Action::SetExpandedVariants {
            act1: u8::from(act == 0 && branch == 1),
            act2: u8::from(act == 1 && branch == 1),
            act3: u8::from(act == 2 && branch == 1),
            act4: u8::from(act == 3 && branch == 1),
            act5: u8::from(act == 4 && branch == 1),
        }).unwrap();
        enter_node(&mut g, floor - 1, Node::Boss);
        assert_eq!(g.battle.as_ref().unwrap().foes[0].id, expected_id);

        // Drive boss down to Phase-2 threshold via real Action::Play strikes.
        {
            let b = g.battle.as_mut().unwrap();
            let half = (b.foes[0].max_hp / 2).max(1);
            b.foes[0].hp = half + 3;
            b.foes[0].block = 0;
            b.foes[0].intent = Intent::Guard(1);
            b.hand = vec![Card::new("strike")];
            b.energy = 3;
        }
        g.apply(Action::Play { index: 1, target: 1 }).unwrap();
        g.hp = g.max_hp;
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().foes[0].phase, 1, "{expected_id} failed Phase 2");

        // Clone into Defeat branch vs Victory branch and execute via Action only.
        let mut lose_branch = g.clone();
        lose_branch.hp = 1;
        lose_branch.battle.as_mut().unwrap().foes[0].intent = Intent::Attack(99);
        lose_branch.apply(Action::EndTurn).unwrap();
        assert_eq!(lose_branch.stage, Stage::Lost, "{expected_id} defeat branch");
        assert!(lose_branch.defeat_diagnosis().unwrap().contains("致命打击"));

        let mut win_branch = g;
        {
            let b = win_branch.battle.as_mut().unwrap();
            for f in &mut b.foes { f.hp = 0; }
            b.foes[0].hp = 4;
            b.foes[0].block = 0;
            b.hand = vec![Card::new("strike")];
            b.energy = 3;
        }
        win_branch.apply(Action::Play { index: 1, target: 1 }).unwrap();
        if floor == 40 {
            assert_eq!(win_branch.stage, Stage::Won, "{expected_id} should win campaign at floor 40");
        } else {
            assert_eq!(win_branch.stage, Stage::Reward, "{expected_id} win must enter Reward before continuing");
        }
    }
}

#[test]
fn late_act_shared_and_hero_echoes_fire_and_preserve_epoch_zero() {
    let cases: &[(Hero, &str, &str, &[u8])] = &[
        (Hero::GuanYu, "anxi_seal", "anxi_seal_integrity", &[23_u8]),
        (Hero::GuanYu, "wenjiu_oath", "wenjiu_vanguard_strike", &[23]),
        (Hero::GuanYu, "maicheng_supply", "maicheng_supply_discipline", &[31, 39]),
        (Hero::CaoCao, "liaodong_letter", "liaodong_foresight_strike", &[31, 39]),
        (Hero::SunShangxiang, "yiling_ashes", "yiling_ashes_counter", &[31, 39]),
        (Hero::SunShangxiang, "yiling_bridge", "yiling_bridge_rescue", &[39]),
        (Hero::ZhugeLiang, "successor_scroll", "successor_scroll_command", &[31]),
        (Hero::ZhugeLiang, "jieting_accountability", "jieting_accountability_guard", &[39]),
        (Hero::LuBu, "white_gate", "white_gate_reckoning", &[39]),
        (Hero::CaoCao, "seven_captures", "seven_captures_pacification", &[31]),
        (Hero::SunShangxiang, "wooden_ox", "wooden_ox_logistics_surge", &[39]),
    ];
    for &(hero, flag, source, prev_floors) in cases {
        for &prev_floor in prev_floors {
            let mut modern = Game::new_with_campaign_difficulty(
                hero, 13, Difficulty::Warlords, format!("r3-echo-{flag}-{prev_floor}"),
            );
            modern.lore_flags.push(flag.into());
            modern.floor = prev_floor;
            assert!(
                modern.route_objective_hint(Node::Boss).is_some_and(|s| s.contains("史事回响")),
                "{flag} missing map route hint before floor {}", prev_floor + 1
            );
            enter_node(&mut modern, prev_floor, Node::Boss);
            assert!(modern.tactical_objective().is_some_and(|s| s.contains("史事回响")));
            {
                let b = modern.battle.as_mut().unwrap();
                b.turn = 1;
                b.hand = vec![Card::new("defend"), Card::new("defend")];
                b.energy = 3;
                b.foes[0].hp = b.foes[0].max_hp.max(80);
            }
            // Activate via two real Action::Play calls (played == 2, block == 10 >= 8 >= 6).
            modern.apply(Action::Play { index: 1, target: 1 }).unwrap();
            modern.apply(Action::Play { index: 1, target: 1 }).unwrap();
            modern.apply(Action::EndTurn).unwrap();
            assert!(
                modern.battle.as_ref().unwrap().events.iter().any(|e|
                    e.kind == EventKind::PowerTriggered && e.source == source),
                "{flag} should emit {source} after floor {prev_floor}"
            );

            // Below-threshold negative test: only 1 card played (< 2) and 0 block must NOT emit.
            let mut under = Game::new_with_campaign_difficulty(
                hero, 13, Difficulty::Warlords, format!("r3-neg-{flag}-{prev_floor}"),
            );
            under.lore_flags.push(flag.into());
            enter_node(&mut under, prev_floor, Node::Boss);
            {
                let b = under.battle.as_mut().unwrap();
                b.turn = 1;
                b.hand = vec![Card::new("strike")];
                b.energy = 3;
                b.foes[0].hp = b.foes[0].max_hp.max(80);
            }
            under.apply(Action::Play { index: 1, target: 1 }).unwrap();
            under.apply(Action::EndTurn).unwrap();
            assert!(
                !under.battle.as_ref().unwrap().events.iter().any(|e| e.source == source),
                "{flag} must not emit {source} when played_this_turn < 2"
            );

            let base_epoch0 = Game::new_with_campaign_difficulty(
                hero, 13, Difficulty::Warlords, format!("r3-epoch0-{flag}-{prev_floor}"),
            );
            let mut raw: serde_json::Value = serde_json::from_str(&base_epoch0.to_json().unwrap()).unwrap();
            raw.as_object_mut().unwrap().remove("expanded_rules_revision");
            raw["campaign_history"][0]["state"]["identity"]
                .as_object_mut().unwrap()
                .remove("expanded_rules_revision");
            let mut epoch0 = Game::from_json(&raw.to_string()).unwrap();
            epoch0.lore_flags.push(flag.into());
            epoch0.floor = prev_floor;
            assert!(
                !epoch0.route_objective_hint(Node::Boss).is_some_and(|s| s.contains("史事回响")),
                "epoch-0 save must omit route echo hint for {flag}"
            );
            enter_node(&mut epoch0, prev_floor, Node::Boss);
            assert!(
                !epoch0.tactical_objective().is_some_and(|s| s.contains("史事回响")),
                "epoch-0 save must omit tactical echo hint for {flag}"
            );
            {
                let b = epoch0.battle.as_mut().unwrap();
                b.turn = 1;
                b.hand = vec![Card::new("defend"), Card::new("defend")];
                b.energy = 3;
                b.foes[0].hp = b.foes[0].max_hp.max(80);
            }
            epoch0.apply(Action::Play { index: 1, target: 1 }).unwrap();
            epoch0.apply(Action::Play { index: 1, target: 1 }).unwrap();
            epoch0.apply(Action::EndTurn).unwrap();
            assert!(
                !epoch0.battle.as_ref().unwrap().events.iter().any(|e| e.source == source),
                "epoch-0 save must suppress {source}"
            );
        }
    }

    // Wrong-hero guard: CaoCao holding hero-specific GuanYu flag "anxi_seal" must NOT fire.
    let mut wrong_hero = Game::new_with_campaign_difficulty(
        Hero::CaoCao, 13, Difficulty::Warlords, "r3-wrong-hero-anxi".into(),
    );
    wrong_hero.lore_flags.push("anxi_seal".into());
    enter_node(&mut wrong_hero, 23, Node::Boss);
    {
        let b = wrong_hero.battle.as_mut().unwrap();
        b.turn = 1;
        b.hand = vec![Card::new("defend"), Card::new("defend")];
        b.energy = 3;
    }
    wrong_hero.apply(Action::Play { index: 1, target: 1 }).unwrap();
    wrong_hero.apply(Action::Play { index: 1, target: 1 }).unwrap();
    wrong_hero.apply(Action::EndTurn).unwrap();
    assert!(
        !wrong_hero.battle.as_ref().unwrap().events.iter().any(|e| e.source == "anxi_seal_integrity"),
        "hero-specific echo must never fire for another hero"
    );
}

fn natural_story_policy_action(g: &Game) -> Action {
    match g.stage {
        Stage::Map => {
            if let Some(pos) = g.route.iter().position(|n| *n == Node::Story) {
                Action::Choose { index: pos + 1 }
            } else if let Some(pos) = g.route.iter().position(|n| *n == Node::Boss) {
                Action::Choose { index: pos + 1 }
            } else if g.hp + 15 <= g.max_hp && let Some(pos) = g.route.iter().position(|n| *n == Node::Camp) {
                Action::Choose { index: pos + 1 }
            } else {
                Action::Choose { index: 1 }
            }
        }
        Stage::Story => {
            if g.in_council() {
                Action::Choose { index: 3 }
            } else if g.can_open_expanded_council() {
                Action::Choose { index: if g.story_fifth_option().is_some() { 5 } else { 4 } }
            } else {
                Action::Choose { index: 1 }
            }
        }
        Stage::Camp => Action::Choose { index: if g.hp + 12 <= g.max_hp { 1 } else { 2 } },
        Stage::Market => Action::Choose { index: 0 },
        Stage::Reward => {
            let pick = g.rewards.iter().enumerate()
                .filter(|(_, c)| card_spec(&c.id).is_some_and(|s| !matches!(s.effect, Effect::Unplayable)))
                .max_by_key(|(idx, c)| (i32::from(c.upgraded), -(*idx as i32)))
                .map_or(0, |(idx, _)| idx + 1);
            Action::Choose { index: if g.deck.len() < 22 { pick } else { 0 } }
        }
        Stage::Ancient => Action::Choose { index: 1 },
        Stage::Combat => {
            let b = g.battle.as_ref().unwrap();
            let live = b.foes.iter().position(|f| f.hp > 0).map(|i| i + 1);
            if b.siege_gate_hp.is_some_and(|hp| hp > 0) && (live.is_none() || b.energy >= 2) {
                if b.energy >= 2 { Action::AssaultGate } else { Action::EndTurn }
            } else if let Some(target) = live {
                if let Some(pos) = b.hand.iter().position(|c| card_spec(&c.id).is_some_and(|s|
                    s.cost <= b.energy
                        && !matches!(s.effect, Effect::Unplayable | Effect::TutorAttack | Effect::CopyDiscard)
                        && !matches!(s.effect, Effect::BloodPrice(loss, _) if g.hp <= loss)))
                {
                    Action::Play { index: pos + 1, target }
                } else {
                    Action::EndTurn
                }
            } else {
                Action::EndTurn
            }
        }
        Stage::Won | Stage::Lost => Action::EndTurn,
    }
}

#[test]
fn pure_natural_action_runs_unlock_early_and_late_anchors_across_all_heroes() {
    // Drive purely via Game::apply(Action) with ZERO direct mutation of lore_flags/lore_current/floor.
    for (hero, expected_anchor) in [
        (Hero::GuanYu, "wenjiu_oath"),
        (Hero::ZhugeLiang, "longzhong_plan"),
        (Hero::CaoCao, "xu_capital"),
        (Hero::SunShangxiang, "jiangdong_covenant"),
        (Hero::LuBu, "phoenix_pavilion"),
    ] {
        let mut found = false;
        for seed in 247_510_u64..247_530 {
            let mut g = Game::new_with_campaign_difficulty(
                hero, seed, Difficulty::Warlords, format!("nat-chain-{hero:?}-{seed}"),
            );
            let mut steps = 0_u32;
            while steps < 2_400 && !matches!(g.stage, Stage::Won | Stage::Lost) && g.floor <= 24 {
                let act = natural_story_policy_action(&g);
                g.apply(act).unwrap();
                steps += 1;
                if g.lore_flags.iter().any(|f| f == expected_anchor) {
                    found = true;
                    break;
                }
            }
            if found {
                g.audit_campaign_authority().expect("natural action authority must audit cleanly");
                g.verify_action_replay().expect("natural action chain replay must succeed");
                break;
            }
        }
        assert!(found, "{hero:?} should naturally unlock strategic anchor {expected_anchor} via pure actions");
    }

    // Verify shared Act 3–4 tactical anchors (seven_captures / wooden_ox) become naturally reachable in multi-act runs
    // and populate valid historical_density_report() receipts without state mutation.
    let mut reached_shared_late = false;
    for seed in 247_510_u64..247_522 {
        let mut g = Game::new_with_campaign_difficulty(
            Hero::GuanYu, seed, Difficulty::Warlords, format!("nat-shared-late-{seed}"),
        );
        let mut steps = 0_u32;
        while steps < 4_500 && !matches!(g.stage, Stage::Won | Stage::Lost) {
            let act = natural_story_policy_action(&g);
            g.apply(act).unwrap();
            steps += 1;
        }
        if g.floor >= 32
            && g.lore_flags.iter().any(|f| matches!(f.as_str(), "seven_captures" | "wooden_ox" | "maicheng_supply"))
        {
            g.audit_campaign_authority().unwrap();
            g.verify_action_replay().unwrap();
            let density = g.historical_density_report().unwrap();
            assert!(density.covered_lower_bound >= 6, "multi-act natural story run should record rich historical coverage");
            reached_shared_late = true;
            break;
        }
    }
    assert!(reached_shared_late, "pure natural action trajectory must reach shared/late-act echoes cleanly");
}
