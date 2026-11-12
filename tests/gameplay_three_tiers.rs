//! Public-seam regression for the opt-in gameplay-expansion difficulty identity.
//! Legacy 21-floor campaigns must retain their original deterministic rule set.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, Stage, map, content::Hero};

fn first_battle(mut game: Game) -> Game {
    assert_eq!(game.route.first(), Some(&Node::Battle));
    game.apply(Action::Choose { index: 1 }).expect("first ordinary battle");
    game
}

#[test]
fn three_new_difficulties_are_real_playable_and_replayable() {
    assert_eq!(Difficulty::ALL.len(), 3);
    assert!(Difficulty::parse("unknown").is_none());
    for (i, difficulty) in Difficulty::ALL.iter().copied().enumerate() {
        assert_eq!(Difficulty::parse(difficulty.id()), Some(difficulty));
        let game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 50_100 + i as u64, difficulty, format!("new-difficulty-{i}"),
        );
        assert_eq!(game.difficulty, Some(difficulty));
        assert_eq!(game.ascension, 0);
        assert_eq!(Game::from_json(&game.to_json().unwrap()).unwrap().difficulty, Some(difficulty));
        let game = first_battle(game);
        assert!(game.battle.is_some());
        game.audit_campaign_authority().expect("campaign identity journal");
        game.verify_action_replay().expect("seed and action replay");
    }
}

#[test]
fn new_tiers_impose_increasing_first_encounter_pressure() {
    let legacy = first_battle(Game::new_with_ascension(Hero::CaoCao, 419, 0));
    let warlords = first_battle(Game::new_with_campaign_difficulty(
        Hero::CaoCao, 419, Difficulty::Warlords, "warlords".into(),
    ));
    let conqueror = first_battle(Game::new_with_campaign_difficulty(
        Hero::CaoCao, 419, Difficulty::Conqueror, "conqueror".into(),
    ));
    let destiny = first_battle(Game::new_with_campaign_difficulty(
        Hero::CaoCao, 419, Difficulty::Destiny, "destiny".into(),
    ));
    let hp = |game: &Game| game.battle.as_ref().unwrap().foes[0].max_hp;
    assert!(hp(&legacy) < hp(&warlords));
    assert!(hp(&warlords) < hp(&conqueror));
    assert!(hp(&conqueror) < hp(&destiny));
    for game in [&warlords, &conqueror, &destiny] {
        game.verify_action_replay().expect("difficulty is part of replay identity");
    }
}

#[test]
fn old_ascension_save_does_not_gain_new_difficulty() {
    let old = first_battle(Game::new_with_ascension(Hero::ZhugeLiang, 87, 6));
    assert!(old.difficulty.is_none());
    let serialized = old.to_json().unwrap();
    let restored = Game::from_json(&serialized).unwrap();
    assert_eq!(restored.ascension, 6);
    assert_eq!(restored.difficulty, None);
    assert_eq!(restored.to_json().unwrap(), serialized);
    restored.verify_action_replay().unwrap();
}

#[test]
fn new_tier_event_archive_reconstructs_after_action_and_json_resume() {
    for tier in Difficulty::ALL {
        let game = first_battle(Game::new_with_campaign_difficulty(
            Hero::CaoCao, 1945, tier, format!("event-replay-{}", tier.id()),
        ));
        let saved = game.to_json().unwrap();
        let restored = Game::from_json(&saved).expect("new tier save can resume");
        let events = restored.reconstruct_event_history()
            .expect("event archive must replay with the selected campaign difficulty");
        assert_eq!(events.len(), restored.event_history.as_ref().unwrap().len());
        assert_eq!(restored.to_json().unwrap(), saved, "event verification is read-only");
    }
}

#[test]
fn new_tier_replay_rejects_difficulty_identity_tampering_without_mutating_snapshot() {
    let mut game = first_battle(Game::new_with_campaign_difficulty(
        Hero::CaoCao, 1945, Difficulty::Warlords, "tamper-tier".into(),
    ));
    game.verify_action_replay().expect("original tier must replay");
    let original = game.to_json().unwrap();

    // The same actions and seed cannot authenticate a different ruleset.
    // This also covers tampering that leaves the battle-event journal intact.
    game.difficulty = Some(Difficulty::Destiny);
    let tampered = game.to_json().unwrap();
    assert_ne!(tampered, original);
    assert!(game.verify_action_replay().is_err());
    assert!(game.reconstruct_event_history().is_err());
    assert_eq!(game.to_json().unwrap(), tampered, "verification must be read-only");

    let restored = Game::from_json(&original).expect("untampered save loads");
    restored.verify_action_replay().expect("untampered save still replays");
}

#[test]
fn all_five_heroes_retain_tier_identity_across_json_and_event_replay() {
    // This cross-product catches hero-specific setup or combat initialization
    // paths accidentally rebuilding a three-tier run as legacy ascension.
    let heroes = [
        Hero::CaoCao, Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang, Hero::LuBu,
    ];
    for (hero_index, hero) in heroes.into_iter().enumerate() {
        for tier in Difficulty::ALL {
            let game = first_battle(Game::new_with_campaign_difficulty(
                hero, 70_000 + hero_index as u64, tier,
                format!("hero-{hero_index}-tier-{}", tier.id()),
            ));
            let before = game.to_json().expect("serialize campaign");
            let resumed = Game::from_json(&before).expect("resume campaign");
            assert_eq!(resumed.difficulty, Some(tier));
            assert_eq!(resumed.ascension, 0);
            resumed.audit_campaign_authority().expect("authoritative identity");
            resumed.verify_action_replay().expect("actions replay");
            resumed.reconstruct_event_history().expect("events replay");
            assert_eq!(resumed.to_json().unwrap(), before, "verification must be read-only");
        }
    }
}

#[test]
fn new_campaign_uses_five_eight_row_acts_without_changing_legacy() {
    let expanded = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 81234, Difficulty::Warlords, "five-acts".into(),
    );
    assert_eq!(expanded.act_name(), "乱世起兵");
    assert_eq!(expanded.campaign_rows(), 8);
    assert_eq!(expanded.campaign_max_floor(), 40);
    let map = expanded.act_map.as_ref().expect("new campaign map");
    assert_eq!(map.rows.len(), 8);
    assert_eq!(map.act, 0);
    assert!(map.rows[7].iter().all(|node| *node == Node::Boss));
    let mut started = expanded;
    started.apply(Action::Choose { index: 1 }).expect("first map action");
    assert_eq!(started.floor, 1);
    started.verify_action_replay().expect("expanded first action replay");
    Game::from_json(&started.to_json().unwrap()).expect("expanded map save");
    let legacy = Game::new(Hero::GuanYu, 81234);
    assert_eq!(legacy.act_name(), "黄巾乱世");
    assert_eq!(legacy.campaign_rows(), 7);
    assert_eq!(legacy.campaign_max_floor(), 21);
    assert_eq!(legacy.act_map.unwrap().rows.len(), 7);
}

#[test]
fn new_campaign_can_commit_floor_twenty_two_beyond_legacy_authority_limit() {
    // Isolated checkpoint fixture: proves the public Action commit boundary,
    // not that a player naturally traversed the previous 21 floors.
    let mut expanded = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 82_221, Difficulty::Warlords, "act-three-boundary".into(),
    );
    expanded.floor = 21;
    expanded.act_map = Some(map::ActMap::generate_expanded(|_| 0, 2));
    expanded.map_lane = None;
    expanded.route = vec![Node::Camp];
    expanded.route_lanes = vec![0];
    assert!(expanded.act_map.as_ref().unwrap().is_valid());
    assert_eq!(expanded.campaign_max_floor(), 40);
    let revision = expanded.revision;
    expanded.apply(Action::Choose { index: 1 }).expect("expanded campaign must cross floor 21");
    assert_eq!(expanded.floor, 22);
    assert_eq!(expanded.stage, Stage::Camp);
    assert_eq!(expanded.revision, revision + 1);
    Game::from_json(&expanded.to_json().unwrap()).expect("floor 22 save");

    // A legacy checkpoint must still reject a 22nd floor transactionally.
    let mut legacy = Game::new(Hero::GuanYu, 82_221);
    legacy.floor = 21;
    legacy.route = vec![Node::Camp];
    legacy.route_lanes = vec![0];
    let unchanged = legacy.to_json().unwrap();
    assert!(legacy.apply(Action::Choose { index: 1 }).is_err());
    assert_eq!(legacy.to_json().unwrap(), unchanged);
}

#[test]
fn later_acts_use_distinct_encounter_rosters_via_public_action() {
    // Checkpoint fixtures isolate the two late-act roster rules; these are
    // not evidence of natural traversal or late-act economic balance.
    for (floor, expected) in [
        (24, ["officer", "wolf", "crossbowman", "raider", "cuirass"]),
        (32, ["shaman", "witch", "panther", "firelancer", "tomb_guard"]),
    ] {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 812_345, Difficulty::Warlords, format!("roster-{floor}"),
        );
        game.floor = floor;
        game.act_map = Some(map::ActMap::generate_expanded(|_| 0, floor / 8));
        game.map_lane = None;
        game.route = vec![Node::Battle];
        game.route_lanes = vec![0];
        game.apply(Action::Choose { index: 1 }).expect("late-act encounter");
        assert_eq!(game.floor, floor + 1);
        let battle = game.battle.as_ref().expect("battle");
        assert!(battle.foes.iter().all(|foe| expected.contains(&foe.id.as_str())),
            "act {} roster: {:?}", floor / 8 + 1, battle.foes.iter().map(|f| &f.id).collect::<Vec<_>>());
        Game::from_json(&game.to_json().unwrap()).expect("late-act encounter save");
    }
}

#[test]
fn expanded_maps_keep_every_reachable_lane_connected_to_the_boss() {
    // Structural seed sweep only: not evidence of natural combat completion.
    // Each reachable lane must have a legal continuation on every row.
    for act in 0..map::EXPANDED_ACTS {
        for seed in 0_u64..64 {
            let mut rng = seed.wrapping_add(u64::from(act) * 1_000_003);
            let graph = map::ActMap::generate_expanded(
                |bound| {
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    ((rng >> 32) as usize) % bound
                },
                act,
            );
            assert!(graph.is_valid(), "act {act} seed {seed} invalid graph");
            assert_eq!(graph.rows.len(), map::EXPANDED_ROWS);
            assert!(graph.rows[map::EXPANDED_ROWS - 1].iter().all(|node| *node == Node::Boss));
            let mut reachable = graph.routes(0, None).into_iter().map(|(lane, _)| lane).collect::<Vec<_>>();
            assert!(!reachable.is_empty(), "act {act} seed {seed} has no entry");
            for row in 1..map::EXPANDED_ROWS {
                let mut next = Vec::new();
                for lane in reachable {
                    let choices = graph.routes(row, Some(lane));
                    assert!(!choices.is_empty(),
                        "act {act} seed {seed} row {row} lane {lane} dead end");
                    next.extend(choices.into_iter().map(|(to, _)| to));
                }
                next.sort_unstable();
                next.dedup();
                reachable = next;
            }
            assert!(!reachable.is_empty(), "act {act} seed {seed} cannot reach boss");
        }
    }
}

#[test]
fn expanded_campaign_rejects_invalid_map_choice_without_consuming_authority() {
    // Real first-floor public action, not a fabricated high-floor checkpoint.
    // A rejected lane must not advance the seeded map, revision, or replay log.
    for tier in Difficulty::ALL {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 91_247, tier, format!("atomic-map-{}", tier.id()),
        );
        let before = game.to_json().expect("serialize before invalid action");
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before, "invalid lane must be atomic");

        game.apply(Action::Choose { index: 1 }).expect("valid first-floor action");
        assert_eq!(game.floor, 1);
        game.verify_action_replay().expect("rejected action cannot enter replay");
        let saved = game.to_json().unwrap();
        let restored = Game::from_json(&saved).expect("expanded save after rejection");
        assert_eq!(restored.to_json().unwrap(), saved);
        restored.verify_action_replay().expect("expanded resumed replay");
    }
}

#[test]
fn fifth_act_boss_entrance_commits_floor_forty_and_restores() {
    // Isolated checkpoint fixture, not proof of a naturally completed 40-floor run.
    // The final boss must use the same public action, combat journals and save
    // boundary as earlier encounters, rather than failing at the old floor cap.
    for tier in Difficulty::ALL {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 940_247, tier, format!("final-boss-{}", tier.id()),
        );
        game.floor = 39;
        game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
        game.map_lane = None;
        game.route = vec![Node::Boss];
        game.route_lanes = vec![0];
        let revision = game.revision;

        game.apply(Action::Choose { index: 1 }).expect("enter floor 40 boss via public action");
        assert_eq!(game.floor, 40);
        assert_eq!(game.revision, revision + 1);
        assert_eq!(game.stage, Stage::Combat);
        assert_eq!(game.last_node, Some(Node::Boss));
        assert!(game.battle.as_ref().is_some_and(|battle| !battle.foes.is_empty()));
        let saved = game.to_json().expect("serialize floor 40 boss");
        let restored = Game::from_json(&saved).expect("restore floor 40 boss");
        assert_eq!(restored.to_json().unwrap(), saved);
    }
}

#[test]
fn all_five_act_boss_entrances_commit_at_eight_floor_intervals() {
    // Isolated checkpoints verify each boss entrance via the public action.
    // They are not evidence of natural full-campaign traversal or victory.
    for tier in Difficulty::ALL {
        for act in 0_u8..map::EXPANDED_ACTS {
            let mut game = Game::new_with_campaign_difficulty(
                Hero::GuanYu, 950_247 + u64::from(act), tier,
                format!("act-{act}-boss-{}", tier.id()),
            );
            let floor_before = (act + 1) * 8 - 1;
            game.floor = floor_before;
            game.act_map = Some(map::ActMap::generate_expanded(|_| 0, act));
            game.map_lane = None;
            game.route = vec![Node::Boss];
            game.route_lanes = vec![0];
            let revision = game.revision;

            game.apply(Action::Choose { index: 1 })
                .expect("boss entrance must commit through public action");
            assert_eq!(game.floor, floor_before + 1);
            assert_eq!(game.revision, revision + 1);
            assert_eq!(game.stage, Stage::Combat);
            assert_eq!(game.last_node, Some(Node::Boss));
            assert!(game.battle.as_ref().is_some_and(|battle| !battle.foes.is_empty()));
            let saved = game.to_json().expect("serialize boss entrance");
            assert_eq!(
                Game::from_json(&saved).expect("restore boss entrance").to_json().unwrap(),
                saved,
                "act {act} tier {} boss entrance must survive restore", tier.id()
            );
        }
    }
}

#[test]
fn expanded_campaign_rejects_floor_forty_one_without_mutating_authority() {
    // Isolated boundary fixture: verifies a rejected public action cannot
    // create a 41st floor or consume RNG/revision/replay state. It is not
    // evidence of a naturally completed campaign.
    for tier in Difficulty::ALL {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 960_247, tier, format!("floor-cap-{}", tier.id()),
        );
        game.floor = 40;
        game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 4));
        game.map_lane = None;
        game.route = vec![Node::Camp];
        game.route_lanes = vec![0];

        let before = game.to_json().expect("serialize floor 40 checkpoint");
        assert!(
            game.apply(Action::Choose { index: 1 }).is_err(),
            "tier {} must reject floor 41",
            tier.id()
        );
        assert_eq!(
            game.to_json().unwrap(),
            before,
            "rejected floor 41 must not mutate tier {}",
            tier.id()
        );
    }
}
