use three_kingdoms_spire_reborn::{
    Action, Card, Difficulty, Game, Intent, Node, Stage,
    builds::PLANS,
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
fn expanded_economic_saturation_bounds_and_non_repeatable_nodes() {
    // 1. Combined passive victory recovery strictly saturates per act [16, 14, 12, 10, 8]
    //    when settled through real Action::Play combat victory, never exceeding the act cap.
    for (completed_floor, expected_cap) in [(0_u8, 16), (8, 14), (16, 12), (24, 10), (32, 8)] {
        let mut g = Game::new_with_campaign_difficulty(
            Hero::GuanYu,
            7,
            Difficulty::Warlords,
            format!("rec-{completed_floor}"),
        );
        g.relics.extend(["神医".into(), "百草".into()]);
        g.boons.push("renewal".into());
        enter_node(&mut g, completed_floor, Node::Battle);
        assert_eq!(g.victory_recovery_cap(), Some(expected_cap), "unexpected act cap at floor {}", g.floor);
        g.hp = 20;
        {
            let b = g.battle.as_mut().unwrap();
            for f in &mut b.foes { f.hp = 0; }
            b.foes[0].hp = 4;
            b.foes[0].block = 0;
            b.hand = vec![Card::new("strike")];
            b.energy = 3;
        }
        g.apply(Action::Play { index: 1, target: 1 }).unwrap();
        assert_eq!(g.hp, 20 + expected_cap, "victory recovery must equal exact act cap on floor {}", g.floor);
        assert!(matches!(g.stage, Stage::Reward | Stage::Map));
        if g.stage == Stage::Reward {
            let gold_before = g.gold;
            g.apply(Action::Choose { index: 0 }).unwrap();
            assert_eq!(g.stage, Stage::Map);
            assert!(g.rewards.is_empty(), "skipping reward must clear offered rewards");
            assert_eq!(g.gold, gold_before);
        }
    }

    // 2. Ancient boon herb_garden is strictly capped at 2 potions in expanded rules (max 3 bag slots),
    //    and cannot be re-offered by subsequent Act Boss -> Ancient rewards once owned.
    let mut anc = Game::new_with_campaign_difficulty(Hero::ZhugeLiang, 7, Difficulty::Warlords, "herb-cap".into());
    anc.potion_bag.clear();
    anc.potions = 0;
    anc.stage = Stage::Ancient;
    anc.boon_choices = vec!["herb_garden".into()];
    anc.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(anc.potion_bag.len(), 2, "expanded herb_garden grants 2 potions, not 3");
    assert_eq!(anc.stage, Stage::Map);
    assert!(anc.boons.iter().any(|b| b == "herb_garden"));
    // Defeat Act II boss and verify Ancient re-roll omits already-owned herb_garden.
    enter_node(&mut anc, 15, Node::Boss);
    {
        let b = anc.battle.as_mut().unwrap();
        for f in &mut b.foes { f.hp = 0; }
        b.foes[0].hp = 4;
        b.foes[0].block = 0;
        b.hand = vec![Card::new("strike")];
        b.energy = 3;
    }
    anc.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert_eq!(anc.stage, Stage::Reward);
    anc.apply(Action::Choose { index: 0 }).unwrap();
    assert_eq!(anc.stage, Stage::Ancient);
    assert!(!anc.boon_choices.iter().any(|b| b == "herb_garden"), "owned boon must never repeat");
    assert!(!anc.boon_choices.is_empty());
    anc.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(anc.stage, Stage::Map);

    // 3. Endless guard/heal stall battle resolves cleanly via Turn 40+ supply_exhaustion (never softlocks).
    let mut stall = Game::new_with_campaign_difficulty(Hero::LuBu, 11, Difficulty::Destiny, "loop-cap".into());
    enter_node(&mut stall, 15, Node::Boss);
    stall.hp = 12;
    {
        let b = stall.battle.as_mut().unwrap();
        b.turn = 40;
        b.foes[0].intent = Intent::Guard(10);
    }
    let mut rounds = 0;
    while stall.stage == Stage::Combat && rounds < 20 {
        // Simulating a pure block/heal stall: keep block capped but supply_exhaustion ignores block.
        stall.battle.as_mut().unwrap().block = 100;
        stall.battle.as_mut().unwrap().foes[0].intent = Intent::Guard(10);
        stall.apply(Action::EndTurn).unwrap();
        rounds += 1;
    }
    assert_eq!(stall.stage, Stage::Lost, "protracted infinite-stall guard loop must settle via exhaustion");
    assert!(stall.defeat_diagnosis().unwrap().contains("粮尽"));

    // 4. Story events and Market upgrades cannot be farmed twice on the same node.
    let mut mk = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "mkt-once".into());
    mk.gold = 500;
    enter_node(&mut mk, 4, Node::Market);
    mk.apply(Action::Choose { index: 7 }).unwrap();
    assert!(mk.market_upgraded);
    let snap = mk.to_json().unwrap();
    assert!(mk.apply(Action::Choose { index: 7 }).is_err());
    assert_eq!(mk.to_json().unwrap(), snap);
}

#[test]
fn build_catalog_covers_fifteen_distinct_blueprints_and_non_duplicate_core_accounting() {
    assert_eq!(PLANS.len(), 15);
    for plan in PLANS {
        assert!(plan.is_catalog_valid(), "invalid build plan: {}", plan.id);
        let mut dup_deck = vec![Card::new(plan.core[0]), Card::new(plan.core[0])];
        assert_eq!(plan.acquired(&dup_deck), 1, "duplicate starter/core must count once");
        for id in &plan.core[1..] {
            dup_deck.push(Card::new(id));
        }
        assert_eq!(plan.acquired(&dup_deck), plan.core.len());
    }
    // 5. Progressive Market card removal (75x3 -> 100 -> 125 -> 150 -> 175 cap), Camp drill saturation
    //    (2x5 + 1x3 -> +0 cap), and late-act high-gold payout trim remain isolated to expanded rules.
    let mut econ = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "econ-sat".into());
    econ.gold = 2_000;
    while econ.deck.len() < 12 { econ.deck.push(Card::new("strike")); }
    enter_node(&mut econ, 3, Node::Market);
    for expected_cost in [75_i32, 75, 75, 100, 125, 150, 175, 175] {
        assert_eq!(econ.card_removal_price(), expected_cost);
        let gold_before = econ.gold;
        let msgs = econ.apply(Action::Remove { index: 1 }).unwrap();
        assert!(msgs.iter().any(|m| m.contains(&format!("{expected_cost}金"))), "expected {expected_cost}金 in {msgs:?}");
        assert_eq!(econ.gold, gold_before - expected_cost);
    }
    let mut drills = Game::new_with_campaign_difficulty(Hero::GuanYu, 7, Difficulty::Warlords, "camp-sat".into());
    for expected_gain in [2_i32, 2, 2, 2, 2, 1, 1, 1, 0, 0] {
        assert_eq!(drills.camp_drill_hp_gain(), expected_gain);
        let max_before = drills.max_hp;
        enter_node(&mut drills, 2, Node::Camp);
        drills.apply(Action::Choose { index: 3 }).unwrap();
        assert_eq!(drills.max_hp, max_before + expected_gain);
    }
    let mut legacy = Game::new(Hero::GuanYu, 7);
    legacy.gold = 2_000;
    legacy.stage = Stage::Market;
    assert_eq!(legacy.card_removal_price(), 75);
    assert_eq!(legacy.camp_drill_hp_gain(), 2);
    let legacy_gold = legacy.gold;
    legacy.apply(Action::Remove { index: 1 }).unwrap();
    assert_eq!(legacy.gold, legacy_gold - 75);
    assert_eq!(legacy.card_removal_price(), 75, "legacy removal price never inflates");
    // Late-act (floor > 24, gold >= 900) halves normal battle payout deterministically.
    let mut rich = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "gold-trim".into());
    enter_node(&mut rich, 25, Node::Battle);
    rich.gold = 950;
    {
        let b = rich.battle.as_mut().unwrap();
        for f in &mut b.foes { f.hp = 0; }
        b.foes[0].hp = 4;
        b.foes[0].block = 0;
        b.hand = vec![Card::new("strike")];
        b.energy = 3;
    }
    rich.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert!(rich.gold - 950 <= 19, "late-act hoarded gold payout should saturate below half normal ceiling");

    // Negative boundaries for late-act gold saturation: <900 gold, floor<=24, or epoch-0 keep full payout.
    let mut under_gold = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "under-900".into());
    enter_node(&mut under_gold, 25, Node::Battle);
    under_gold.gold = 850;
    {
        let b = under_gold.battle.as_mut().unwrap();
        for f in &mut b.foes { f.hp = 0; }
        b.foes[0].hp = 4;
        b.foes[0].block = 0;
        b.hand = vec![Card::new("strike")];
        b.energy = 3;
    }
    under_gold.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert!(under_gold.gold - 850 >= 20, "gold < 900 must keep full battle payout");

    let mut early_rich = Game::new_with_campaign_difficulty(Hero::CaoCao, 7, Difficulty::Warlords, "early-950".into());
    enter_node(&mut early_rich, 18, Node::Battle);
    early_rich.gold = 950;
    {
        let b = early_rich.battle.as_mut().unwrap();
        for f in &mut b.foes { f.hp = 0; }
        b.foes[0].hp = 4;
        b.foes[0].block = 0;
        b.hand = vec![Card::new("strike")];
        b.energy = 3;
    }
    early_rich.apply(Action::Play { index: 1, target: 1 }).unwrap();
    assert!(early_rich.gold - 950 >= 20, "floor <= 24 must keep full battle payout");

    let mut st = Game::new_with_campaign_difficulty(Hero::SunShangxiang, 7, Difficulty::Warlords, "story-once".into());
    enter_node(&mut st, 1, Node::Story);
    st.lore_current = None;
    st.event_id = 3;
    let gold_before = st.gold;
    st.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(st.stage, Stage::Map);
    assert!(st.gold > gold_before);
    assert!(st.deck.iter().all(|c| card_spec(&c.id).is_some()));
    assert!(!st.deck.iter().any(|c| matches!(card_spec(&c.id).map(|s| s.effect), Some(Effect::Unplayable))));
    let snap = st.to_json().unwrap();
    assert!(st.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(st.to_json().unwrap(), snap);
}

#[test]
fn forty_floor_multi_act_economic_saturation_and_potion_caps_hold_under_replay() {
    // Drive a natural 40-floor expanded run and verify economic bounds across every traversed act:
    // potion bag <= 3, valid deck specs, positive gold progression, and full replay verification.
    let mut reached_forty = false;
    for seed in 247_510_u64..247_516 {
        let mut g = Game::new_with_campaign_difficulty(
            Hero::GuanYu, seed, Difficulty::Warlords, format!("eco-40-{seed}"),
        );
        let mut steps = 0_u32;
        while steps < 5_000 && !matches!(g.stage, Stage::Won | Stage::Lost) {
            assert!(g.potion_bag.len() <= 3, "potion bag exceeded 3 slots on floor {}", g.floor);
            assert!(usize::from(g.potions) == g.potion_bag.len());
            assert!(g.hp <= g.max_hp && g.gold >= 0);
            let act = match g.stage {
                Stage::Map => {
                    if g.hp * 2 < g.max_hp && let Some(pos) = g.route.iter().position(|n| *n == Node::Camp) {
                        Action::Choose { index: pos + 1 }
                    } else if let Some(pos) = g.route.iter().position(|n| *n == Node::Boss) {
                        Action::Choose { index: pos + 1 }
                    } else if let Some(pos) = g.route.iter().position(|n| *n == Node::Story) {
                        Action::Choose { index: pos + 1 }
                    } else {
                        Action::Choose { index: 1 }
                    }
                }
                Stage::Story => {
                    if g.in_council() { Action::Choose { index: 3 } }
                    else if g.can_open_expanded_council() {
                        Action::Choose { index: if g.story_fifth_option().is_some() { 5 } else { 4 } }
                    } else { Action::Choose { index: 1 } }
                }
                Stage::Camp => Action::Choose { index: if g.hp + 12 <= g.max_hp { 1 } else { 2 } },
                Stage::Market => Action::Choose { index: 0 },
                Stage::Reward => {
                    let pick = g.rewards.iter().enumerate()
                        .filter(|(_, c)| card_spec(&c.id).is_some_and(|s| !matches!(s.effect, Effect::Unplayable)))
                        .max_by_key(|(idx, c)| (i32::from(c.upgraded), -(*idx as i32)))
                        .map_or(0, |(idx, _)| idx + 1);
                    Action::Choose { index: if g.deck.len() < 24 { pick } else { 0 } }
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
                                && !matches!(s.effect, Effect::Unplayable | Effect::TutorAttack | Effect::CopyDiscard)))
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
            g.apply(act).unwrap();
            steps += 1;
        }
        assert!(matches!(g.stage, Stage::Won | Stage::Lost));
        g.audit_campaign_authority().unwrap();
        g.verify_action_replay().unwrap();
        if g.floor >= 32 {
            reached_forty = true;
            assert!(g.deck.len() <= 55);
            break;
        }
    }
    assert!(reached_forty, "expected deterministic natural run to traverse late-act economy bounds");
}
