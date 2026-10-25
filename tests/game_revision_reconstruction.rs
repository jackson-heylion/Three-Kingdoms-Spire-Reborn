use three_kingdoms_spire_reborn::{Action, Game, RunMode, content::Hero};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

#[test]
fn ten_real_campaigns_reconstruct_every_full_game_boundary() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let mut game = Game::new_with_identity(
                hero, 20261025, ascension, RunMode::Custom, "replay-test".into(),
            );
            game.profile_start_receipt = true;
            let mut expected = vec![game.to_json().unwrap()];
            for action in [
                Action::SetDoctrine { id: "auto".into() },
                Action::Choose { index: 1 },
                Action::EndTurn,
            ] {
                game.apply(action).unwrap();
                expected.push(game.to_json().unwrap());
            }
            assert_eq!(game.revision, 3);
            assert_eq!(game.verify_action_replay().unwrap(), 3);
            let serialized = game.to_json().unwrap();
            let restored = Game::from_json(&serialized).unwrap();
            for (revision, snapshot) in expected.iter().enumerate() {
                let reconstructed = restored.reconstruct_game_at(revision as u64).unwrap();
                assert_eq!(
                    reconstructed.to_json().unwrap(), *snapshot,
                    "full Game differs for {} ascension {} revision {}",
                    hero.name(), ascension, revision,
                );
                assert_eq!(reconstructed.revision, revision as u64);
                assert_eq!(
                    reconstructed.action_history.as_ref().unwrap().len(), revision,
                );
            }
            assert_eq!(restored.to_json().unwrap(), serialized);
        }
    }
}

#[test]
fn reconstruction_fails_closed_for_missing_incomplete_or_forged_actions() {
    let mut game = Game::new(Hero::GuanYu, 10101);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let original = game.to_json().unwrap();

    assert!(game.reconstruct_game_at(2).is_err());
    assert_eq!(game.to_json().unwrap(), original);

    let mut legacy = game.clone();
    legacy.action_history = None;
    assert!(legacy.reconstruct_game_at(0).unwrap_err().contains("没有完整操作日志"));

    let mut truncated = game.clone();
    truncated.action_history.as_mut().unwrap().clear();
    assert!(truncated.reconstruct_game_at(0).unwrap_err().contains("不一致"));

    let mut forged = game.clone();
    forged.action_history.as_mut().unwrap()[0] = Action::Choose { index: 999 };
    assert!(forged.reconstruct_game_at(1).unwrap_err().contains("重放第1步失败"));
    assert_eq!(forged.revision, game.revision);

    let mut snapshot_tampered = game.clone();
    snapshot_tampered.gold += 1;
    // The historical reconstruction is independent of the mutable snapshot,
    // while full replay verification must reject the forged current state.
    assert_eq!(
        snapshot_tampered.reconstruct_game_at(1).unwrap().to_json().unwrap(),
        original,
    );
    assert!(snapshot_tampered.verify_action_replay().is_err());
}

#[test]
fn older_v2_journal_absence_is_not_fabricated_by_reconstruction() {
    let mut game = Game::new(Hero::ZhugeLiang, 123456);
    game.action_history = Some(Vec::new());
    game.event_history = None;
    // The synthetic legacy snapshot cannot retain a cursor-certified
    // telemetry stream while its complete event archive is absent.
    game.telemetry_history = None;
    game.rng_history = None;
    game.resource_history = None;
    game.battle_resource_history = None;
    game.foe_board_history = None;
    game.card_zone_history = None;
    game.combat_effect_history = None;
    game.apply(Action::SetDoctrine { id: "auto".into() }).unwrap();
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    let rebuilt = restored.reconstruct_game_at(1).unwrap();
    assert!(rebuilt.event_history.is_none());
    assert!(rebuilt.rng_history.is_none());
    assert!(rebuilt.resource_history.is_none());
    assert!(rebuilt.battle_resource_history.is_none());
    assert!(rebuilt.foe_board_history.is_none());
    assert!(rebuilt.card_zone_history.is_none());
    assert!(rebuilt.combat_effect_history.is_none());
    assert_eq!(rebuilt.to_json().unwrap(), restored.to_json().unwrap());
}
