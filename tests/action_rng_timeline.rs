//! Full action -> RNG-call / combat-event ordering through actual Game::apply.
//! This tests emission order, not a causal parent relationship or event sourcing.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode,
    content::Hero,
    rng_events::TimelineStep,
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

fn campaign(hero: Hero, ascension: u8) -> Game {
    let mut game = Game::new_with_identity(
        hero, 20261025, ascension, RunMode::Custom, "rng-event-timeline".into(),
    );
    for action in [
        Action::SetDoctrine { id: "auto".into() },
        Action::Choose { index: 1 },
        Action::EndTurn,
    ] {
        game.apply(action).unwrap();
    }
    game
}

#[test]
fn ten_real_campaigns_preserve_exact_event_and_rng_emission_order() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let game = campaign(hero, ascension);
            let saved = game.to_json().unwrap();
            let restored = Game::from_json(&saved).unwrap();
            let timeline = restored.reconstruct_action_rng_event_timeline().unwrap();
            let draws = restored.reconstruct_rng_draw_attribution().unwrap();
            let archived = restored.event_history.as_ref().unwrap();
            assert_eq!(timeline.len(), 3);
            assert_eq!(timeline[0].revision, 1);
            assert_eq!(timeline[2].revision, 3);

            let mut event_cursor = 0_usize;
            let mut calls_seen = 0_usize;
            for (action, attribution) in timeline.iter().zip(draws.iter()) {
                assert_eq!(action.revision, attribution.revision);
                let mut action_calls = 0_usize;
                let mut action_events = 0_usize;
                assert_eq!(attribution.event_start, Some(event_cursor));
                for step in &action.steps {
                    match step {
                        TimelineStep::RngCall(call) => {
                            assert_eq!(call.event_cursor_before, Some(event_cursor));
                            assert!(call.call_site.contains(".rs:"));
                            calls_seen += 1;
                            action_calls += 1;
                        }
                        TimelineStep::CombatEvent(event) => {
                            assert_eq!(Some(event), archived.get(event_cursor));
                            event_cursor += 1;
                            action_events += 1;
                        }
                    }
                }
                assert_eq!(attribution.event_end, Some(event_cursor));
                assert_eq!(action_calls, attribution.calls.len());
                assert_eq!(action_events, attribution.event_end.unwrap()
                    - attribution.event_start.unwrap());
            }
            assert_eq!(event_cursor, archived.len(), "no event may be silently omitted");
            assert!(calls_seen > 0, "real first-floor battle must consume RNG");
            assert_eq!(restored.to_json().unwrap(), saved, "audit must not modify the save");
            let json = serde_json::to_string(&timeline).unwrap();
            assert!(json.contains("rng_call") && json.contains("combat_event"));
        }
    }
}

#[test]
fn legacy_and_tampered_journals_never_claim_complete_interleaving() {
    let game = campaign(Hero::GuanYu, 0);

    let mut old = game.clone();
    old.event_history = None;
    assert!(old.reconstruct_action_rng_event_timeline().unwrap_err()
        .contains("complete combat event archive"));

    let mut missing_rng = game.clone();
    missing_rng.rng_history = None;
    assert!(missing_rng.reconstruct_action_rng_event_timeline().is_err());

    let mut changed = game.clone();
    changed.event_history.as_mut().unwrap()[0].event.amount += 1;
    assert!(changed.reconstruct_action_rng_event_timeline().is_err());

    let mut truncated = game.clone();
    truncated.action_history.as_mut().unwrap().pop();
    assert!(truncated.reconstruct_action_rng_event_timeline().is_err());
}
