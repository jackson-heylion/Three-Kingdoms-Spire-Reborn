//! Controlled encounter checkpoints for the new-campaign Yuan Shao
//! coalition and Cao Cao's act-II Wuchao responsibility. These DO NOT
//! establish a natural 0→40 route. All effects are applied via Game::apply.
use three_kingdoms_spire_reborn::{
    Action, BossMove, Card, Difficulty, Game, Intent, Node, Stage, WuchaoPlan,
    content::Hero, map,
};

fn guandu_boss() -> Game {
    // Bit 11 chooses the alternate act-II commander. No forced enemy state
    // is used for the actual encounter initialization.
    let mut game = Game::new_with_campaign_difficulty(
        Hero::CaoCao, 1_u64 << 11, Difficulty::Warlords, "guandu-coalition".into(),
    );
    game.floor = 15;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 1));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    game
}

#[test]
fn new_guandu_boss_deploys_visible_officer_while_old_council_is_optional() {
    let mut game = guandu_boss();
    assert!(game.route_objective_hint(Node::Boss).unwrap().contains("亲卫"));
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.floor, 16);
    assert_eq!(game.stage, Stage::Combat);
    let battle = game.battle.as_ref().unwrap();
    assert_eq!(battle.foes[0].id, "yuanshao");
    assert!(matches!(battle.foes[0].intent, Intent::Boss(BossMove::GrandAlliance(_))));
    assert_eq!(battle.foes[1].id, "officer");
    assert!(game.tactical_objective().unwrap().contains("下回合军令"));
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    let result = game.apply(Action::EndTurn).unwrap();
    assert_eq!(result, restored.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    assert!(result.join("；").contains("封锁粮道"));
    assert_eq!(game.battle.as_ref().unwrap().energy, 2);
}

#[test]
fn direct_player_hit_breaks_coalition_and_disables_second_phase_recruitment() {
    let mut game = guandu_boss();
    game.apply(Action::Choose { index: 1 }).unwrap();
    {
        // Controlled weakened vanguard, genuinely defeated by paid strike.
        let b = game.battle.as_mut().unwrap();
        b.foes[1].hp = 1;
        b.foes[1].block = 0;
        b.foes[0].block = 8;
        b.energy = 3;
        b.hand.push(Card::new("strike"));
    }
    let index = game.battle.as_ref().unwrap().hand.len();
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    let result = game.apply(Action::Play { index, target: 2 }).unwrap();
    assert_eq!(result, restored.apply(Action::Play { index, target: 2 }).unwrap());
    assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    assert!(result.join("；").contains("截断袁绍盟军粮道"));
    let b = game.battle.as_ref().unwrap();
    assert_eq!(b.foes[0].block, 2);
    assert!(b.foes[0].vulnerable >= 3);
    assert!(b.events.iter().any(|e| e.source == "yuanshao_escort_severed"));
    // A separate controlled half-health transition confirms the lasting
    // strategy: a killed officer cannot be replaced for free at phase two.
    game.battle.as_mut().unwrap().foes[0].hp =
        game.battle.as_ref().unwrap().foes[0].max_hp / 2;
    let mut resumed = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(game.apply(Action::EndTurn).unwrap(), resumed.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
    assert_eq!(game.battle.as_ref().unwrap().foes.iter()
        .filter(|foe| foe.id == "officer").count(), 1);
    assert!(game.battle.as_ref().unwrap().foes[0].phase > 0);
}

#[test]
fn three_wuchao_choices_change_guan_du_boss_without_spending_later_debt() {
    for (plan, marker) in [
        (WuchaoPlan::RaidPending, "wuchao_coalition_raid"),
        (WuchaoPlan::FortifyPending, "wuchao_coalition_fortified"),
        (WuchaoPlan::DeclinePending, "wuchao_coalition_unchecked"),
    ] {
        let mut game = guandu_boss();
        game.wuchao_plan = Some(plan);
        assert!(game.route_objective_hint(Node::Boss).unwrap().contains("乌巢"));
        let result = game.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(game.wuchao_plan, Some(plan), "act-III payoff stays pending");
        let b = game.battle.as_ref().unwrap();
        assert!(b.events.iter().any(|e| e.source == marker), "{plan:?}");
        match plan {
            WuchaoPlan::RaidPending => {
                assert!(b.foes[1].weak >= 2);
                assert!(b.foes[1].hp < b.foes[1].max_hp);
            }
            WuchaoPlan::FortifyPending => assert!(b.block >= 10),
            WuchaoPlan::DeclinePending => {
                assert!(b.foes[1].strength >= 2);
                assert!(b.foes[1].block >= 8);
            }
            _ => unreachable!(),
        }
        assert!(result.join("；").contains("乌巢"));
        let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        assert_eq!(game.apply(Action::EndTurn).unwrap(), restored.apply(Action::EndTurn).unwrap());
        assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    }
}
