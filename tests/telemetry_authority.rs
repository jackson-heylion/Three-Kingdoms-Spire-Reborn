//! Eight-domain independent combat telemetry authority.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero,
    telemetry_authority::{self, TelemetryEvent, TelemetryProjector},
};
const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang, Hero::CaoCao, Hero::LuBu,
];
#[test]
fn ten_campaigns_project_metrics_without_actions_and_reject_forged_snapshot() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let mut game = Game::new_with_identity(
                hero, 20261025, ascension, RunMode::Standard, "telemetry".into());
            assert_eq!(game.audit_telemetry_authority().unwrap(), 1);
            game.apply(Action::Choose { index: 1 }).unwrap();
            game.apply(Action::EndTurn).unwrap();
            let mut loaded = Game::from_json(&game.to_json().unwrap()).unwrap();
            let (state, revision) = telemetry_authority::project(
                loaded.telemetry_history.as_ref().unwrap()).unwrap();
            assert_eq!(revision, loaded.revision);
            assert_eq!(state, loaded.telemetry);
            assert!(state.totals.battles_started > 0);
            loaded.action_history = None;
            loaded.event_history = None;
            assert_eq!(loaded.audit_composite_authority().unwrap().telemetry, state);
            let mut forged = loaded.clone();
            forged.telemetry.totals.battles_started += 1;
            assert!(forged.audit_composite_authority().is_err());
        }
    }
}
#[test]
fn projector_rejects_regression_atomically() {
    let mut game = Game::new(Hero::GuanYu, 10);
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    let events = game.telemetry_history.as_ref().unwrap();
    let mut projector = TelemetryProjector::default();
    projector.push(&events[0]).unwrap();
    projector.push(&events[1]).unwrap();
    let before = projector.state().unwrap();
    let mut bad = before.0.clone();
    bad.totals.battles_started = 0;
    assert!(projector.push(&TelemetryEvent::Changed { revision: 2, state: Some(bad) }).is_err());
    assert_eq!(projector.state().unwrap(), before);
    projector.push(&events[2]).unwrap();
    assert_eq!(projector.state().unwrap().1, 2);
}
#[test]
fn old_saves_and_rejected_actions_do_not_fabricate_metrics() {
    let mut game = Game::new(Hero::LuBu, 777);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    let mut truncated = game.clone();
    truncated.telemetry_history.as_mut().unwrap().pop();
    assert!(truncated.audit_composite_authority().is_err());
    let mut old = game.clone();
    old.telemetry_history = None;
    let mut loaded = Game::from_json(&old.to_json().unwrap()).unwrap();
    assert!(loaded.audit_composite_authority().is_err());
    loaded.apply(Action::EndTurn).unwrap();
    assert!(loaded.telemetry_history.is_none());
    assert!(Game::from_json(&loaded.to_json().unwrap()).is_ok());
}
