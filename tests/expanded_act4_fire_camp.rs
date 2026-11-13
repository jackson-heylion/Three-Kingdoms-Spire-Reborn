//! Isolated fourth-act boss behavior through the public Game::apply(Action) seam.
//! This is a controlled floor-31 checkpoint, NOT natural 1-to-32 reachability proof.
use three_kingdoms_spire_reborn::{Action, Difficulty, Game, Node, Stage, content::Hero, map};

#[test]
fn fourth_act_boss_previews_distinct_fire_camp_mechanic() {
    let mut game = Game::new_with_campaign_difficulty(
        Hero::GuanYu, 247_411, Difficulty::Warlords, "act-four-fire-247411".into(),
    );
    game.floor = 31;
    game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 3));
    game.map_lane = None;
    game.route = vec![Node::Boss];
    game.route_lanes = vec![0];
    game.apply(Action::Choose { index: 1 }).expect("fourth-act boss via public action");
    assert_eq!(game.floor, 32);
    assert_eq!(game.stage, Stage::Combat);
    let enemy = &game.battle.as_ref().unwrap().foes[0];
    assert_eq!(enemy.id, "luxun", "fourth act must not silently reuse act-III Boss");
    assert!(enemy.intent.label().contains("焚营"), "fire tactic must be previewed");
    Game::from_json(&game.to_json().unwrap()).expect("new Boss intent round-trip");
}

#[test]
fn fourth_act_fire_waits_for_counterplay_and_survives_json_resume() {
    let mut countered = None;
    for seed in 247_411..247_443 {
        // Isolated entry fixture only. Both combat actions below are real
        // Game::apply transactions; no fabricated deck, HP or lore flags.
        let mut game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, seed, Difficulty::Warlords, format!("fire-counter-{seed}"),
        );
        game.floor = 31;
        game.act_map = Some(map::ActMap::generate_expanded(|_| 0, 3));
        game.map_lane = None;
        game.route = vec![Node::Boss];
        game.route_lanes = vec![0];
        game.apply(Action::Choose { index: 1 }).unwrap();
        let first_hp = game.hp;
        game.apply(Action::EndTurn).expect("Lu Xun's telegraphed first fire tactic");
        assert_eq!(game.hp, first_hp, "newly ignited flames cannot burn immediately");
        assert_eq!(game.battle.as_ref().unwrap().wildfire, 2);

        let guard = game.battle.as_ref().unwrap().hand.iter()
            .position(|card| card.id == "defend").map(|index| index + 1);
        let Some(guard) = guard else { continue };
        let before = game.to_json().unwrap();
        let restored = Game::from_json(&before).expect("fire stacks survive restore");
        assert_eq!(restored.battle.as_ref().unwrap().wildfire, 2);
        game.apply(Action::Play { index: guard, target: 1 })
            .expect("player chooses a normal defense card to extinguish fire");
        assert_eq!(game.battle.as_ref().unwrap().wildfire, 1);
        assert!(game.event_history.as_ref().unwrap().iter().any(|entry|
            entry.event.source == "fire_extinguished"));
        game.apply(Action::EndTurn).expect("remaining fire burns before next enemy action");
        assert_eq!(game.battle.as_ref().unwrap().wildfire, 0);
        assert!(game.event_history.as_ref().unwrap().iter().any(|entry|
            entry.event.source == "burning_camp" && entry.event.amount == 2));
        let saved = game.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
        countered = Some(seed);
        break;
    }
    assert!(countered.is_some(), "normal seeded defense cards must offer counterplay");
}
