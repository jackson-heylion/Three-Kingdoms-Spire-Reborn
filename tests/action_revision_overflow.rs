use three_kingdoms_spire_reborn::{Action, Game, content::Hero};

#[test]
fn exhausted_revision_rejects_action_without_mutating_any_journal() {
    let mut game = Game::new(Hero::GuanYu, 20261025);
    game.revision = u64::MAX;
    let before = game.to_json().unwrap();
    let error = game.apply(Action::SetDoctrine { id: "auto".into() }).unwrap_err();
    assert!(error.contains("revision overflow"), "{error}");
    assert_eq!(game.to_json().unwrap(), before);
}

#[test]
fn legacy_save_with_exhausted_revision_is_rejected_atomically() {
    let mut game = Game::new(Hero::CaoCao, 20261025);
    game.revision = u64::MAX;
    game.action_history = None;
    game.event_history = None;
    game.rng_history = None;
    game.resource_history = None;
    game.battle_resource_history = None;
    game.foe_board_history = None;
    game.card_zone_history = None;
    game.combat_effect_history = None;
    let serialized = game.to_json().unwrap();
    let mut restored = Game::from_json(&serialized).unwrap();
    let error = restored.apply(Action::SetDoctrine { id: "auto".into() }).unwrap_err();
    assert!(error.contains("revision overflow"), "{error}");
    assert_eq!(restored.to_json().unwrap(), serialized);
}

#[test]
fn normal_commits_keep_strictly_increasing_revisions() {
    let mut game = Game::new(Hero::ZhugeLiang, 20261025);
    game.apply(Action::SetDoctrine { id: "auto".into() }).unwrap();
    assert_eq!(game.revision, 1);
    game.apply(Action::SetDoctrine { id: "auto".into() }).unwrap();
    assert_eq!(game.revision, 2);
    assert_eq!(game.audit_authority_bundle().unwrap(), 3);
}
