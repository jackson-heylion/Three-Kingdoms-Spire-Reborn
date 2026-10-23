//! Reserved card listeners cannot be forged in custom games or V2 snapshots.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, Stage,
    content::Hero,
    combat::EventKind,
    event_stack::{Listener, Response, Stack},
};

fn wards() -> [Listener; 4] {
    [
        Listener::once("hold_line:weak", "weak", Response::Cancel),
        Listener::once("kongcheng:vulnerable", "vulnerable", Response::Reduce(1)),
        Listener::once("qiaoying:weak", "weak", Response::Reduce(1)),
        Listener::once("zhongyi:weak", "weak", Response::Replace(1)),
    ]
}

#[test]
fn all_four_wards_fit_beyond_custom_limit_and_roundtrip() {
    let mut stack = Stack::default();
    for i in 0..16 {
        stack.install_custom(Listener::once(
            &format!("custom-{i}"), "weak", Response::Reduce(1),
        )).unwrap();
    }
    for ward in wards() { stack.install(ward).unwrap(); }
    assert!(stack.valid());
    assert_eq!(stack.listeners.len(), 20);
    let json = serde_json::to_string(&stack).unwrap();
    let loaded: Stack = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded, stack);
}

#[test]
fn forged_builtin_effects_are_rejected_atomically() {
    let real = wards()[0].clone();
    let mut stack = Stack::default();
    stack.install(real.clone()).unwrap();
    let original = stack.clone();
    let mut wrong_source = real.clone();
    wrong_source.source = "vulnerable".into();
    let mut wrong_kind = real.clone();
    wrong_kind.kind = EventKind::DamageReceived;
    let mut wrong_target = real.clone();
    wrong_target.target = Some("foe:0".into());
    let mut wrong_effect = real.clone();
    wrong_effect.effect = Response::Replace(9999);
    let mut wrong_priority = real.clone();
    wrong_priority.priority = -20;
    let mut wrong_charges = real.clone();
    wrong_charges.charges = 2;
    for forged in [wrong_source, wrong_kind, wrong_target,
        wrong_effect, wrong_priority, wrong_charges]
    {
        assert!(!(Stack { listeners: vec![forged.clone()] }).valid());
        assert_eq!(stack.install(forged), Err("invalid reaction"));
        assert_eq!(stack, original);
    }
    for ward in wards().into_iter().skip(1) {
        let mut forged = ward;
        forged.effect = Response::Cancel;
        assert!(!(Stack { listeners: vec![forged] }).valid());
    }
}

#[test]
fn tampered_v2_snapshot_cannot_claim_a_reserved_ward() {
    let mut game = Game::new_with_mode(Hero::GuanYu, 20261021, 0, RunMode::Custom);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.stage, Stage::Combat);
    let mut forged = wards()[0].clone();
    forged.effect = Response::Replace(9999);
    game.battle.as_mut().unwrap().reactions.listeners.push(forged);
    let raw = game.to_json().unwrap();
    assert!(Game::from_json(&raw).is_err());
}
