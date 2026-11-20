//! New-campaign special battlefield acceptance for the third-act fleet and
//! the defender-first siege option. Floor checkpoints are deliberate fixtures,
//! NOT a fabricated natural 0→40 victory or historical choice. All consequential
//! actions are committed through Game::apply and compared across JSON restore.
use three_kingdoms_spire_reborn::{
    Action, Difficulty, Game, Node, Stage, Card, content::Hero, map,
};

fn isolated_battle(floor: u8) -> Game {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 1_u64 << 44, Difficulty::Warlords,
        format!("special-battle-floor-{floor}"),
    );
    game.floor = floor - 1;
    game.stage = Stage::Map;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, usize::from((floor - 1) / 8)));
    game.map_lane = None;
    game.route = vec![Node::Battle];
    game.route_lanes = vec![0];
    game
}

#[test]
fn river_fleet_entry_previews_tide_and_paid_attacks_board_raiders() {
    let mut game = isolated_battle(19);
    assert!(game.route_objective_hint(Node::Battle).unwrap().contains("水陆阵战"));
    game.apply(Action::Choose { index: 1 }).expect("enter real third-act battle");
    assert_eq!(game.floor, 19);
    assert_eq!(game.stage, Stage::Combat);
    assert_eq!(game.battle.as_ref().unwrap().convoy_hp, Some(12));
    assert_eq!(game.battle.as_ref().unwrap().convoy_cover, Some(0));
    let objective = game.tactical_objective().unwrap();
    assert!(objective.contains("水陆阵战") && objective.contains("逆流"));
    // Controlled card and foe-block checkpoint. The attack, boarding and
    // event, however, come from a legal player action with real damage.
    {
        let b = game.battle.as_mut().unwrap();
        b.foes[0].block = 0;
        b.energy = b.energy.max(1);
        b.hand.push(Card::new("strike"));
    }
    let index = game.battle.as_ref().unwrap().hand.len();
    let before = game.to_json().expect("save before attack");
    let mut loaded = Game::from_json(&before).expect("restore before attack");
    let feedback = game.apply(Action::Play { index, target: 1 })
        .expect("attack raider with real card");
    assert_eq!(feedback, loaded.apply(Action::Play { index, target: 1 }).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert!(feedback.join("；").contains("舰队掩护+2"));
    let b = game.battle.as_ref().unwrap();
    assert_eq!(b.convoy_cover, Some(2));
    assert!(b.events.iter().any(|event|
        event.source == "naval_boarding_cover" && event.amount == 2));
    let save = game.to_json().unwrap();
    let mut resumed = Game::from_json(&save).unwrap();
    let turn = game.apply(Action::EndTurn).expect("resolve actual fleet raid");
    assert_eq!(turn, resumed.apply(Action::EndTurn).unwrap());
    assert_eq!(game.to_json().unwrap(), resumed.to_json().unwrap());
}

#[test]
fn fleet_guard_protects_the_hull_and_loss_seals_the_battle() {
    let mut game = isolated_battle(19);
    game.apply(Action::Choose { index: 1 }).expect("enter fleet defense");
    let mut unguarded = Game::from_json(&game.to_json().unwrap()).unwrap();
    {
        let b = game.battle.as_mut().unwrap();
        b.hand.push(Card::new("defend"));
    }
    let card = game.battle.as_ref().unwrap().hand.len();
    game.apply(Action::Play { index: card, target: 1 })
        .expect("pay for real fleet guard");
    assert_eq!(game.battle.as_ref().unwrap().convoy_cover, Some(4));
    assert!(game.battle.as_ref().unwrap().events.iter().any(|event|
        event.source == "naval_guard_cover" && event.amount == 4));
    // Same encounter/RNG but one legal guard action should preserve at least
    // as much hull as ending the turn without protection.
    game.apply(Action::EndTurn).expect("guarded raid");
    unguarded.apply(Action::EndTurn).expect("unguarded raid");
    if game.stage == Stage::Combat && unguarded.stage == Stage::Combat {
        assert!(game.battle.as_ref().unwrap().convoy_hp
            >= unguarded.battle.as_ref().unwrap().convoy_hp);
    }
    // Separate explicitly controlled near-sinking checkpoint. The lethal
    // raid and defeat still execute through EndTurn, never injected directly.
    let mut sinking = isolated_battle(19);
    sinking.apply(Action::Choose { index: 1 }).unwrap();
    sinking.battle.as_mut().unwrap().convoy_hp = Some(1);
    let saved = sinking.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    let loss = sinking.apply(Action::EndTurn).expect("fleet sinks to real enemy raid");
    assert_eq!(loss, restored.apply(Action::EndTurn).unwrap());
    assert_eq!(sinking.to_json().unwrap(), restored.to_json().unwrap());
    assert!(loss.join("；").contains("战船覆没"));
    assert_eq!(sinking.stage, Stage::Lost);
    assert!(sinking.battle.is_none());
}

#[test]
fn defeating_siege_defenders_before_assault_improves_real_breach() {
    let mut game = isolated_battle(4);
    assert!(game.route_objective_hint(Node::Battle).unwrap().contains("守军"));
    game.apply(Action::Choose { index: 1 }).expect("enter real siege");
    assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(12));
    // Only the final defender is retained as a 1-HP controlled tactical
    // checkpoint. Clearing it still requires a paid real attack card.
    {
        let b = game.battle.as_mut().unwrap();
        for foe in &mut b.foes { foe.hp = 0; foe.block = 0; }
        b.foes[0].hp = 1;
        b.energy = 3;
        b.hand.push(Card::new("strike"));
    }
    let start = game.to_json().unwrap();
    let mut rushing = Game::from_json(&start).unwrap();
    let rush = rushing.apply(Action::AssaultGate).expect("rush gate with guards alive");
    assert!(rush.join("；").contains("攻城破门4点"));
    assert_eq!(rushing.battle.as_ref().unwrap().siege_gate_hp, Some(8));

    let card = game.battle.as_ref().unwrap().hand.len();
    game.apply(Action::Play { index: card, target: 1 })
        .expect("defeat last guard with paid strike");
    assert_eq!(game.stage, Stage::Combat,
        "the siege still requires an actual gate breach");
    assert!(game.tactical_objective().unwrap().contains("当前破门6点"));
    let before_breach = game.to_json().unwrap();
    let mut loaded = Game::from_json(&before_breach).unwrap();
    let breach = game.apply(Action::AssaultGate).expect("breach without defenders");
    assert_eq!(breach, loaded.apply(Action::AssaultGate).unwrap());
    assert_eq!(game.to_json().unwrap(), loaded.to_json().unwrap());
    assert!(breach.join("；").contains("攻城破门6点"));
    assert_eq!(game.battle.as_ref().unwrap().siege_gate_hp, Some(6));
}
