use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero, event_stack::RevisionCheckpoint,
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

fn campaign(hero: Hero, ascension: u8, lane: usize) -> Game {
    let mut game = Game::new_with_identity(
        hero, 20261025, ascension, RunMode::Custom, "trusted-run".into(),
    );
    for action in [
        Action::SetDoctrine { id: "auto".into() },
        Action::Choose { index: lane },
        Action::EndTurn,
    ] {
        game.apply(action).unwrap();
    }
    game
}

#[test]
fn ten_campaigns_attest_historical_and_current_full_game_revisions() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let game = campaign(hero, ascension, 1);
            let original = game.to_json().unwrap();
            let restored = Game::from_json(&original).unwrap();
            for revision in [0, 1, 2, 3] {
                let checkpoint = game.checkpoint_at(revision).unwrap();
                assert_eq!(checkpoint.schema_version, 1);
                assert_eq!(checkpoint.revision, revision);
                assert_eq!(checkpoint.digest_sha256.len(), 64);
                let encoded = serde_json::to_string(&checkpoint).unwrap();
                let decoded: RevisionCheckpoint = serde_json::from_str(&encoded).unwrap();
                assert_eq!(decoded, checkpoint);
                restored.verify_checkpoint(&decoded).unwrap();
                assert_eq!(checkpoint, restored.checkpoint_at(revision).unwrap());
            }
            assert_eq!(restored.to_json().unwrap(), original, "audit must be read-only");
        }
    }
}

#[test]
fn trusted_checkpoint_detects_valid_but_rewritten_history() {
    let original = campaign(Hero::GuanYu, 0, 1);
    let trusted = original.checkpoint_at(original.revision).unwrap();
    let alternate = campaign(Hero::GuanYu, 0, 0);
    assert_eq!(alternate.verify_action_replay().unwrap(), 3);
    assert!(alternate.verify_checkpoint(&trusted).unwrap_err().contains("digest mismatch"));

    let different_seed = Game::new_with_identity(
        Hero::GuanYu, 20261026, 0, RunMode::Custom, "trusted-run".into(),
    );
    assert!(different_seed.verify_checkpoint(&trusted).unwrap_err().contains("different campaign"));
}

#[test]
fn checkpoint_export_rejects_forged_live_snapshot_and_truncated_journal() {
    let original = campaign(Hero::SunShangxiang, 10, 1);
    let trusted = original.checkpoint_at(2).unwrap();

    let mut edited = original.clone();
    edited.gold += 1;
    assert!(edited.checkpoint_at(2).unwrap_err().contains("checkpoint export refused"));
    assert!(edited.verify_checkpoint(&trusted).is_err());

    let mut truncated = original.clone();
    truncated.action_history.as_mut().unwrap().pop();
    assert!(truncated.checkpoint_at(2).is_err());

    let mut legacy = original.clone();
    legacy.action_history = None;
    assert!(legacy.checkpoint_at(0).is_err());
    assert!(original.checkpoint_at(4).is_err());
}

#[test]
fn invalid_checkpoint_schema_and_digest_are_rejected() {
    let game = campaign(Hero::ZhugeLiang, 0, 1);
    let checkpoint = game.checkpoint_at(3).unwrap();

    let mut bad_version = checkpoint.clone();
    bad_version.schema_version = 2;
    assert!(game.verify_checkpoint(&bad_version).unwrap_err().contains("unsupported"));

    let mut malformed = checkpoint.clone();
    malformed.digest_sha256 = "not-a-digest".into();
    assert!(game.verify_checkpoint(&malformed).unwrap_err().contains("invalid SHA-256"));

    let mut wrong = checkpoint.clone();
    wrong.digest_sha256 = "0".repeat(64);
    assert!(game.verify_checkpoint(&wrong).unwrap_err().contains("digest mismatch"));

    let mut future = checkpoint;
    future.revision += 1;
    assert!(game.verify_checkpoint(&future).is_err());
}
