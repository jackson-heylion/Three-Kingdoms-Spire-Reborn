use three_kingdoms_spire_reborn::{
    Action, Card, Difficulty, Game, Node, Stage,
    builds::PLANS,
    campaign_authority,
    combat::EventKind,
    content::{Effect, Hero, card_spec},
    lore, map,
};

fn generate_expanded_for(seed: u64, act: u8) -> map::ActMap {
    let mut s = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(u64::from(act) + 1);
    map::ActMap::generate_expanded(
        |bound| {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (s as usize) % bound
        },
        act,
    )
}

#[test]
fn expanded_map_topology_is_valid_deterministic_across_250_seeds_and_all_five_acts() {
    for seed in 1_u64..=250 {
        for act in 0_u8..5 {
            let m1 = generate_expanded_for(seed, act);
            let m2 = generate_expanded_for(seed, act);
            assert!(m1.is_valid(), "seed {seed} act {act} expanded map invalid");
            assert_eq!(
                serde_json::to_string(&m1).unwrap(),
                serde_json::to_string(&m2).unwrap()
            );
            assert_eq!(m1.rows.len(), 8);
            assert!(m1.rows[7].iter().all(|n| *n == Node::Boss));
            assert!(!m1.routes(0, None).is_empty());
            if act >= 1 {
                assert!(
                    m1.rows[2..=4].iter().any(|row| row.contains(&Node::Story)),
                    "seed {seed} act {act} missing mid-act Story guarantee"
                );
            }
        }
    }
}

#[test]
fn late_echo_counterfactual_decline_vs_vow_and_reload_dedup() {
    // Verify genuine counterfactual contrast AND actual downstream Boss receipt emission vs omission
    // plus mid-boss JSON reload deduplication across all late-act / shared echoes.
    for (hero, beat_id, flag, source, prev_floor) in [
        (
            Hero::GuanYu,
            "maicheng",
            "maicheng_supply",
            "maicheng_supply_discipline",
            31_u8,
        ),
        (
            Hero::CaoCao,
            "liaodong",
            "liaodong_letter",
            "liaodong_foresight_strike",
            31,
        ),
        (
            Hero::SunShangxiang,
            "yiling_bridge",
            "yiling_bridge",
            "yiling_bridge_rescue",
            39,
        ),
        (
            Hero::ZhugeLiang,
            "jieting_debt",
            "jieting_accountability",
            "jieting_accountability_guard",
            39,
        ),
        (
            Hero::LuBu,
            "whitegate",
            "white_gate",
            "white_gate_reckoning",
            39,
        ),
        (
            Hero::CaoCao,
            "south_wilds",
            "seven_captures",
            "seven_captures_pacification",
            31,
        ),
        (
            Hero::SunShangxiang,
            "wooden_ox",
            "wooden_ox",
            "wooden_ox_logistics_surge",
            39,
        ),
    ] {
        let mut vow_run = Game::new_with_campaign_difficulty(
            hero,
            17,
            Difficulty::Warlords,
            format!("cf-vow-{beat_id}"),
        );
        vow_run.floor = 17;
        vow_run.stage = Stage::Map;
        vow_run.route = vec![Node::Story];
        vow_run.route_lanes = vec![0];
        vow_run.apply(Action::Choose { index: 1 }).unwrap();
        vow_run.lore_current = Some(beat_id.into());
        let mut bargain_run = vow_run.clone();

        vow_run.apply(Action::Choose { index: 1 }).unwrap();
        bargain_run.apply(Action::Choose { index: 2 }).unwrap();

        assert!(
            vow_run.lore_flags.iter().any(|f| f == flag),
            "{beat_id} vow must plant {flag}"
        );
        assert!(
            !bargain_run.lore_flags.iter().any(|f| f == flag),
            "{beat_id} bargain must not plant {flag}"
        );

        let mut saved_vow = Game::from_json(&vow_run.to_json().unwrap()).unwrap();
        let mut saved_bargain = Game::from_json(&bargain_run.to_json().unwrap()).unwrap();
        assert_eq!(
            saved_vow.lore_flags.iter().filter(|f| *f == flag).count(),
            1
        );
        // On non-boss floor 17 -> floor 18, route_objective_hint(Boss) must not fire prematurely.
        assert!(saved_vow.route_objective_hint(Node::Boss).is_none());

        for branch in [&mut saved_vow, &mut saved_bargain] {
            branch.floor = prev_floor;
            branch.stage = Stage::Map;
            branch.route = vec![Node::Boss];
            branch.route_lanes = vec![0];
            branch.apply(Action::Choose { index: 1 }).unwrap();
            {
                let b = branch.battle.as_mut().unwrap();
                b.turn = 1;
                b.hand = vec![Card::new("defend"), Card::new("defend")];
                b.energy = 3;
                b.foes[0].hp = b.foes[0].max_hp.max(90);
            }
            branch
                .apply(Action::Play {
                    index: 1,
                    target: 1,
                })
                .unwrap();
            branch
                .apply(Action::Play {
                    index: 1,
                    target: 1,
                })
                .unwrap();
            branch.apply(Action::EndTurn).unwrap();
        }
        assert_eq!(
            saved_vow
                .battle
                .as_ref()
                .unwrap()
                .events
                .iter()
                .filter(|e| e.kind == EventKind::PowerTriggered && e.source == source)
                .count(),
            1,
            "{beat_id} vow must emit exactly one receipt on Turn 1 Boss"
        );
        assert_eq!(
            saved_bargain
                .battle
                .as_ref()
                .unwrap()
                .events
                .iter()
                .filter(|e| e.source == source)
                .count(),
            0,
            "{beat_id} counterfactual bargain must emit zero receipts on Boss"
        );
        // Save/reload mid-boss and pass Turn 2: receipt count must remain 1 (no duplicate settlement).
        let mut reloaded_vow = Game::from_json(&saved_vow.to_json().unwrap()).unwrap();
        reloaded_vow.apply(Action::EndTurn).unwrap();
        assert_eq!(
            reloaded_vow
                .battle
                .as_ref()
                .unwrap()
                .events
                .iter()
                .filter(|e| e.source == source)
                .count(),
            1,
            "{beat_id} must not re-emit receipt on Turn 2 after save/reload"
        );
    }
}

#[test]
fn natural_pure_action_runs_unlock_xiapi_and_xiangfan_councils() {
    for (hero, seed_range) in [
        (Hero::LuBu, 247_510_u64..247_522),
        (Hero::GuanYu, 247_510_u64..247_522),
    ] {
        let mut settled = false;
        for seed in seed_range {
            let mut g = Game::new_with_campaign_difficulty(
                hero,
                seed,
                Difficulty::Warlords,
                format!("nat-council-{hero:?}-{seed}"),
            );
            let mut steps = 0_u32;
            while steps < 4_000 && !matches!(g.stage, Stage::Won | Stage::Lost) {
                let act = match g.stage {
                    Stage::Map => {
                        if let Some(pos) = g.route.iter().position(|n| *n == Node::Story) {
                            Action::Choose { index: pos + 1 }
                        } else if let Some(pos) = g.route.iter().position(|n| *n == Node::Boss) {
                            Action::Choose { index: pos + 1 }
                        } else if g.hp * 2 < g.max_hp
                            && let Some(pos) = g.route.iter().position(|n| *n == Node::Camp)
                        {
                            Action::Choose { index: pos + 1 }
                        } else {
                            Action::Choose { index: 1 }
                        }
                    }
                    Stage::Story => {
                        if g.in_council() {
                            Action::Choose { index: 3 }
                        } else if g.can_open_expanded_council() {
                            Action::Choose {
                                index: if g.story_fifth_option().is_some() {
                                    5
                                } else {
                                    4
                                },
                            }
                        } else {
                            Action::Choose { index: 1 }
                        }
                    }
                    Stage::Camp => Action::Choose {
                        index: if g.hp + 12 <= g.max_hp { 1 } else { 2 },
                    },
                    Stage::Market => Action::Choose { index: 0 },
                    Stage::Reward => Action::Choose {
                        index: if g.deck.len() < 18 { 1 } else { 0 },
                    },
                    Stage::Ancient => Action::Choose { index: 1 },
                    Stage::Combat => {
                        let b = g.battle.as_ref().unwrap();
                        let live = b.foes.iter().position(|f| f.hp > 0).map(|i| i + 1);
                        if b.siege_gate_hp.is_some_and(|hp| hp > 0)
                            && (live.is_none() || b.energy >= 2)
                        {
                            if b.energy >= 2 {
                                Action::AssaultGate
                            } else {
                                Action::EndTurn
                            }
                        } else if let Some(target) = live {
                            if let Some(pos) = b.hand.iter().position(|c| {
                                card_spec(&c.id).is_some_and(|s| {
                                    s.cost <= b.energy
                                        && !matches!(s.effect, Effect::Unplayable | Effect::TutorAttack | Effect::CopyDiscard)
                                        && !matches!(s.effect, Effect::BloodPrice(loss, _) if g.hp <= loss)
                                })
                            }) {
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
                g.apply(act).unwrap();
                steps += 1;
            }
            let council_done = match hero {
                Hero::LuBu => g.xiapi_plan.is_some(),
                Hero::GuanYu => g.xiangfan_plan.is_some(),
                _ => false,
            };
            if council_done {
                g.audit_campaign_authority().unwrap();
                g.verify_action_replay().unwrap();
                settled = true;
                break;
            }
        }
        assert!(
            settled,
            "{hero:?} should naturally open and commit its late-act council"
        );
    }
}

#[test]
fn spec_gate_invariants_tiers_plans_beats_and_epoch0_boundary() {
    assert_eq!(Difficulty::ALL.len(), 3);
    assert_eq!(
        Difficulty::ALL.iter().map(|d| d.id()).collect::<Vec<_>>(),
        ["warlords", "conqueror", "destiny"]
    );
    assert!(Difficulty::parse("legend").is_none());

    assert_eq!(PLANS.len(), 15);
    for plan in PLANS {
        assert!(plan.is_catalog_valid());
        for &cid in plan.core.iter().chain(plan.support.iter()) {
            let spec = card_spec(cid).expect("plan card must exist");
            assert!(!matches!(spec.effect, Effect::Unplayable));
        }
    }

    for id in [
        "taoyuan",
        "baima",
        "anxi",
        "wenjiu",
        "huarong",
        "jingzhou",
        "maicheng",
        "longzhong",
        "borrow_arrows",
        "redcliff",
        "wuzhang",
        "jiangwei",
        "jieting_debt",
        "sun_father_letter",
        "jiangdong_wedding",
        "river",
        "yiling",
        "yiling_bridge",
        "sevenstar",
        "xudu",
        "guandu",
        "liaodong",
        "redhare",
        "pavilion",
        "halberd",
        "whitegate",
        "empty_city",
        "south_wilds",
        "wooden_ox",
    ] {
        assert!(lore::beat(id).is_some(), "missing canonical beat {id}");
    }

    // Legacy 21-floor AND epoch-0 expanded saves must reject forged card_removals / camp_drills.
    for raw_json in [Game::new(Hero::GuanYu, 9).to_json().unwrap(), {
        let base = Game::new_with_campaign_difficulty(
            Hero::GuanYu,
            9,
            Difficulty::Warlords,
            "epoch0-econ".into(),
        );
        let mut val: serde_json::Value = serde_json::from_str(&base.to_json().unwrap()).unwrap();
        val.as_object_mut()
            .unwrap()
            .remove("expanded_rules_revision");
        val.to_string()
    }] {
        let mut forged: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        forged["card_removals"] = serde_json::json!(2);
        assert!(
            Game::from_json(&forged.to_string()).is_err(),
            "forged card_removals must fail"
        );
        forged.as_object_mut().unwrap().remove("card_removals");
        forged["camp_drills"] = serde_json::json!(1);
        assert!(
            Game::from_json(&forged.to_string()).is_err(),
            "forged camp_drills must fail"
        );
    }

    // Verify 255 card_removals saturation transitions cleanly through CampaignProjector without overflow.
    let mut sat = Game::new_with_campaign_difficulty(
        Hero::CaoCao,
        11,
        Difficulty::Warlords,
        "sat-255".into(),
    );
    sat.gold = 2_000;
    sat.card_removals = u8::MAX;
    while sat.deck.len() < 8 {
        sat.deck.push(Card::new("strike"));
    }
    sat.stage = Stage::Map;
    sat.route = vec![Node::Market];
    sat.route_lanes = vec![0];
    sat.apply(Action::Choose { index: 1 }).unwrap();
    let before_state = campaign_authority::CampaignState::current(&sat);
    sat.apply(Action::Remove { index: 1 }).unwrap();
    assert_eq!(sat.card_removals, u8::MAX);
    assert_eq!(sat.card_removal_price(), 175);
    let after_state = campaign_authority::CampaignState::current(&sat);
    let ev = campaign_authority::transition(&before_state, &after_state, sat.revision).unwrap();
    assert!(matches!(
        ev,
        campaign_authority::CampaignEvent::Changed { .. }
    ));
}
