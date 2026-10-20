//! Explicit unregistration of dynamic combat listeners is replayable,
 //! persisted, and safe on failed commands. Existing saves without this action
 //! variant continue to deserialize without any migration.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, Stage, content::Hero,
    event_stack::{Listener, Response},
};

fn setup(mode:RunMode)->Game {
    let mut g=Game::new_with_mode(Hero::GuanYu,20261020,0,mode);
    g.apply(Action::Choose {index:1}).unwrap();
    assert_eq!(g.stage,Stage::Combat);
    g
}

#[test]
fn unregistration_is_atomic_replayable_and_persistent() {
    let mut g=setup(RunMode::Custom);
    g.apply(Action::RegisterReaction {
        listener:Listener::once("ward","weak",Response::Cancel),
    }).unwrap();
    let registered=g.to_json().unwrap();
    assert_eq!(g.battle.as_ref().unwrap().reactions.listeners.len(),1);
    for id in ["absent", "", &"x".repeat(65)] {
        assert!(g.apply(Action::UnregisterReaction {id:id.into()}).is_err());
        assert_eq!(g.to_json().unwrap(),registered,
            "rejected removal must not change RNG, history or snapshot");
    }

    g.apply(Action::UnregisterReaction {id:"ward".into()}).unwrap();
    assert!(g.battle.as_ref().unwrap().reactions.listeners.is_empty());
    assert_eq!(g.action_history.as_ref().unwrap().len(),3);
    let raw=g.to_json().unwrap();
    let resumed=Game::from_json(&raw).unwrap();
    assert_eq!(resumed.to_json().unwrap(),raw);
    assert_eq!(resumed.verify_action_replay().unwrap(),3);
    assert_eq!(resumed.reconstruct_event_history().unwrap(),
        resumed.event_history.clone().unwrap());

    // Same stable ID can now be safely reinstalled after removal.
    g.apply(Action::RegisterReaction {
        listener:Listener::once("ward","vulnerable",Response::Replace(1)),
    }).unwrap();
    assert_eq!(g.verify_action_replay().unwrap(),4);
    assert_eq!(g.battle.as_ref().unwrap().reactions.listeners[0].source,"vulnerable");
}

#[test]
fn ranked_runs_cannot_unregister_a_listener_or_mutate_journal() {
    let mut ranked=setup(RunMode::Standard);
    let initial=ranked.to_json().unwrap();
    assert!(ranked.apply(Action::UnregisterReaction{id:"hold_line:weak".into()}).is_err());
    assert_eq!(ranked.to_json().unwrap(),initial);
}
