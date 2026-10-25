//! Committed event cursors bind every action revision to an event-only prefix.
//! This tests transient forgeries that final-state cross-source audits miss.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero,
    telemetry_authority::TelemetryEvent,
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

fn campaign(hero: Hero, ascension: u8) -> Game {
    let mut game = Game::new_with_identity(
        hero, 20261026, ascension, RunMode::Custom, "event-cursors".into(),
    );
    game.apply(Action::SetDoctrine { id: "auto".into() }).unwrap();
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    game
}

#[test]
fn ten_campaigns_reconcile_every_committed_event_prefix_and_restart() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let game = campaign(hero, ascension);
            let loaded = Game::from_json(&game.to_json().unwrap()).unwrap();
            assert_eq!(loaded.audit_event_archive_boundaries().unwrap(),
                Some(usize::try_from(loaded.revision).unwrap() + 1));
            loaded.audit_event_archive_authority().unwrap();
            assert_eq!(loaded.verify_action_replay().unwrap(), 3);
            let before = loaded.to_json().unwrap();
            assert_eq!(loaded.to_json().unwrap(), before);
        }
    }
}

#[test]
fn transient_cursor_forgery_is_rejected_even_when_final_archive_matches() {
    let game = campaign(Hero::CaoCao, 10);
    let mut early = game.clone();
    let combat_cursor = early.telemetry_history.as_ref().unwrap()[2].archive_len().unwrap();
    // Revision 1 is still the map: moving future battle events into that
    // prefix must fail despite a fully legitimate final state.
    if let TelemetryEvent::Changed { archive_len, .. } =
        &mut early.telemetry_history.as_mut().unwrap()[1]
    {
        *archive_len = Some(combat_cursor);
    }
    assert!(early.audit_composite_authority().is_ok());
    let error = early.audit_event_archive_authority().unwrap_err();
    assert!(error.contains("event revision 1"), "{error}");

    let mut late = game.clone();
    if let TelemetryEvent::Changed { archive_len, .. } =
        &mut late.telemetry_history.as_mut().unwrap()[2]
    {
        *archive_len = Some(0);
    }
    assert!(late.audit_composite_authority().is_ok());
    let error = late.audit_event_archive_authority().unwrap_err();
    assert!(error.contains("cursor") || error.contains("event revision 2"), "{error}");

    let mut missing = game.clone();
    if let TelemetryEvent::Changed { archive_len, .. } =
        &mut missing.telemetry_history.as_mut().unwrap()[2]
    {
        *archive_len = None;
    }
    assert!(missing.audit_event_archive_boundaries().is_err());
}

#[test]
fn old_v2_cursors_remain_absent_during_resume_and_action_replay() {
    let mut legacy = campaign(Hero::GuanYu, 0);
    for event in legacy.telemetry_history.as_mut().unwrap() {
        match event {
            TelemetryEvent::Started { archive_len, .. }
            | TelemetryEvent::Changed { archive_len, .. } => *archive_len = None,
        }
    }
    let mut restored = Game::from_json(&legacy.to_json().unwrap()).unwrap();
    assert_eq!(restored.audit_event_archive_boundaries().unwrap(), None);
    restored.audit_event_archive_authority().unwrap();
    restored.verify_action_replay().unwrap();
    restored.apply(Action::EndTurn).unwrap();
    assert!(restored.telemetry_history.as_ref().unwrap().iter()
        .all(|event| event.archive_len().is_none()));
    restored.verify_action_replay().unwrap();
    assert_eq!(restored.audit_event_archive_boundaries().unwrap(), None);
}
