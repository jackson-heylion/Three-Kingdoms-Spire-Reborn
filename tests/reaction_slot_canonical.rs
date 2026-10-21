//! Dynamic listeners must use the exact foe tokens emitted by the battle loop.
//! A parsed number with leading zero/sign would silently never fire.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero, combat::EventKind,
    event_stack::{Listener, Response, Stack},
};

#[test]
fn canonical_source_and_target_slots_cover_all_valid_positions() {
    for i in 0..4 {
        let slot = format!("foe:{i}");
        let mut status = Listener::once("target", "poison", Response::Cancel);
        status.target = Some(slot.clone());
        let mut dealt = Listener::once("dealt", "hero", Response::Reduce(1));
        dealt.kind = EventKind::DamageDealt;
        dealt.target = Some(slot.clone());
        let mut received = Listener::once("received", &slot, Response::Replace(4));
        received.kind = EventKind::DamageReceived;
        let mut stack = Stack::default();
        stack.install_custom(status).unwrap();
        stack.install_custom(dealt).unwrap();
        stack.install_custom(received).unwrap();
        assert!(stack.valid());
    }
}

#[test]
fn noncanonical_slots_cannot_register_as_unmatchable_rules() {
    let mut g = Game::new_with_mode(Hero::ZhugeLiang, 20261021, 0, RunMode::Custom);
    g.apply(Action::Choose { index: 1 }).unwrap();
    let baseline = g.to_json().unwrap();
    for slot in ["foe:00", "foe:01", "foe:03", "foe:+1", "foe:4", "foe:-1", "foe: 1"] {
        let mut status = Listener::once("bad", "weak", Response::Cancel);
        status.target = Some(slot.into());
        let mut damage = Listener::once("bad", "fire", Response::Reduce(2));
        damage.kind = EventKind::DamageDealt;
        damage.target = Some(slot.into());
        let mut incoming = Listener::once("bad", slot, Response::Cancel);
        incoming.kind = EventKind::DamageReceived;
        for listener in [status, damage, incoming] {
            assert!(g.apply(Action::RegisterReaction { listener }).is_err(),
                "accepted unmatchable slot {slot}");
            assert_eq!(g.to_json().unwrap(), baseline,
                "invalid registration changed RNG, journal or state");
        }
    }
    let canonical = Listener::once("real", "weak", Response::Cancel);
    g.apply(Action::RegisterReaction { listener: canonical }).unwrap();
    let saved = g.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
    assert_eq!(g.verify_action_replay().unwrap(), 2);
    assert_eq!(g.reconstruct_event_history().unwrap(), g.event_history.clone().unwrap());
}
