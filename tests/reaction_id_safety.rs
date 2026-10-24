//! Reject terminal controls before custom reaction IDs reach audit output.
//! Failed registration is atomic; forged V2 snapshots cannot bypass validation.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero,
    event_stack::{Listener, Response},
};

fn custom_battle() -> Game {
    let mut game = Game::new_with_mode(Hero::ZhugeLiang, 20261024, 0, RunMode::Custom);
    game.apply(Action::Choose { index: 1 }).expect("start battle");
    assert!(game.battle.is_some());
    game
}

#[test]
fn control_character_ids_are_rejected_without_changing_game_state() {
    let mut game = custom_battle();
    let before = game.to_json().unwrap();
    for id in ["line\nbreak", "escape\u{1b}[31m", "carriage\rreturn", "nul\u{0}byte"] {
        let result = game.apply(Action::RegisterReaction {
            listener: Listener::once(id, "weak", Response::Cancel),
        });
        assert!(result.is_err(), "accepted unsafe reaction ID: {id:?}");
        assert_eq!(game.to_json().unwrap(), before, "rejected action mutated state");
    }
}

#[test]
fn forged_save_with_control_character_id_is_rejected() {
    let mut game = custom_battle();
    game.battle.as_mut().unwrap().reactions.listeners.push(
        Listener::once("fake\u{1b}[2J", "weak", Response::Cancel),
    );
    assert!(Game::from_json(&game.to_json().unwrap()).is_err());
}

#[test]
fn ordinary_unicode_id_still_roundtrips_and_replays() {
    let mut game = custom_battle();
    game.apply(Action::RegisterReaction {
        listener: Listener::once("守势-甲", "weak", Response::Cancel),
    }).unwrap();
    let loaded = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(loaded.reconstruct_event_history().unwrap(),
        loaded.event_history.clone().unwrap());
}
