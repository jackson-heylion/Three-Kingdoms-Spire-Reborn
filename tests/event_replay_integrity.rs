//! Snapshot/event-source consistency regression.
//! Passing verify-events must mean more than matching archived event strings.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero,
    event_stack::{Listener, Response},
};

fn started(mode: RunMode) -> Game {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 20261020, 0, mode);
    game.apply(Action::Choose { index: 1 }).expect("start battle");
    assert!(game.battle.is_some());
    assert!(!game.event_history.as_ref().unwrap().is_empty());
    game
}

#[test]
fn faithful_custom_reaction_roundtrips_and_reconstructs() {
    let mut game = started(RunMode::Custom);
    game.apply(Action::RegisterReaction {
        listener: Listener::once("ward", "weak", Response::Cancel),
    }).expect("register unranked reaction");
    let snapshot = game.to_json().unwrap();
    let loaded = Game::from_json(&snapshot).unwrap();
    assert_eq!(loaded.reconstruct_event_history().unwrap(),
        loaded.event_history.clone().unwrap());
    assert_eq!(loaded.verify_action_replay().unwrap(), 2);
}

#[test]
fn tampering_with_unjournalled_resources_or_listener_state_fails_verification() {
    let game = started(RunMode::Standard);
    for mutate in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let mut altered = game.clone();
        if mutate.0 { altered.gold += 1; }
        if mutate.1 { altered.hp -= 1; }
        if mutate.2 { altered.battle.as_mut().unwrap().energy += 1; }
        // These are otherwise legal V2 game snapshots and still have a valid
        // ordered event archive: integrity needs a deterministic action replay.
        let loaded = Game::from_json(&altered.to_json().unwrap()).unwrap();
        assert!(loaded.reconstruct_event_history().is_err(),
            "a non-event state change was accepted: {mutate:?}");
    }

    let mut custom = started(RunMode::Custom);
    custom.apply(Action::RegisterReaction {
        listener: Listener::once("shield", "weak", Response::Cancel),
    }).unwrap();
    custom.battle.as_mut().unwrap().reactions.listeners[0].charges = 2;
    let loaded = Game::from_json(&custom.to_json().unwrap()).unwrap();
    assert!(loaded.reconstruct_event_history().is_err(),
        "listener mutation must not be authenticated by unchanged event history");
}

#[test]
fn corrupted_archive_or_retained_window_is_rejected_on_load() {
    let game = started(RunMode::Standard);
    let mut changed_archive = game.clone();
    changed_archive.event_history.as_mut().unwrap().last_mut()
        .unwrap().event.source.push_str(":forged");
    assert!(Game::from_json(&changed_archive.to_json().unwrap()).is_err(),
        "source mismatch between full archive and battle window");

    let mut changed_window = game.clone();
    changed_window.battle.as_mut().unwrap().events.last_mut().unwrap().amount += 1;
    assert!(Game::from_json(&changed_window.to_json().unwrap()).is_err(),
        "modified recent event values must not pass sequence-only validation");
}

#[test]
fn legacy_missing_archive_is_rebuilt_only_from_complete_actions() {
    let mut game = started(RunMode::Standard);
    let expected = game.event_history.take().unwrap();
    let saved = game.to_json().unwrap();
    let restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.reconstruct_event_history().unwrap(), expected);
    let mut tampered = restored.clone();
    tampered.gold += 3;
    assert!(tampered.reconstruct_event_history().is_err(),
        "even legacy opt-out must compare the remaining full state");
    tampered.action_history = None;
    assert!(tampered.reconstruct_event_history().is_err(),
        "missing original actions cannot certify a regenerated source");
}

#[test]
fn legacy_recent_event_window_rejects_saturated_sequence_duplicates() {
    // Legacy V2 saves may omit the full event archive. Their retained battle
    // window still has to be strictly increasing, even at u64::MAX.
    let mut game = started(RunMode::Standard);
    game.event_history = None;
    let battle = game.battle.as_mut().unwrap();
    let mut event = battle.events.last().unwrap().clone();
    event.seq = u64::MAX;
    battle.events = vec![event.clone(), event];
    battle.event_seq = u64::MAX;

    // saturating_add(MAX, 1) == MAX used to accept this forged duplicate.
    assert!(Game::from_json(&game.to_json().unwrap()).is_err());
}

#[test]
fn legacy_recent_event_window_accepts_strictly_increasing_sequences() {
    let mut game = started(RunMode::Standard);
    game.event_history = None;
    let battle = game.battle.as_mut().unwrap();
    let mut first = battle.events.last().unwrap().clone();
    first.seq = u64::MAX - 1;
    let mut second = first.clone();
    second.seq = u64::MAX;
    battle.events = vec![first, second];
    battle.event_seq = u64::MAX;
    assert!(Game::from_json(&game.to_json().unwrap()).is_ok());
}
