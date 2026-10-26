//! Full active Battle can be rebuilt without executing actions or trusting the
//! mutable battle snapshot. Legacy V2 journals remain readable, not certified.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero,
    battle_runtime_events::{self, BattleRuntimeChange, BattleRuntimeEvent},
    event_stack::authority,
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

fn campaign(hero: Hero, ascension: u8) -> Game {
    let mut game = Game::new_with_identity(
        hero, 20261026, ascension, RunMode::Custom, "runtime-authority".into(),
    );
    game.apply(Action::SetDoctrine { id: "auto".into() }).unwrap();
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    game
}

#[test]
fn all_heroes_rebuild_full_battle_from_nine_journals_across_save_restart() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let game = campaign(hero, ascension);
            let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
            let view = authority::project(&restored).unwrap();
            let rebuilt = view.rebuild_battle().unwrap();
            assert_eq!(
                serde_json::to_value(&rebuilt).unwrap(),
                serde_json::to_value(&restored.battle).unwrap(),
                "all battle fields must come from independent journal state"
            );
            restored.audit_battle_runtime_authority().unwrap();
            restored.audit_event_archive_authority().unwrap();
            restored.verify_action_replay().unwrap();
        }
    }
}

#[test]
fn runtime_snapshot_tampering_is_detected_and_independent_projection_is_unchanged() {
    let game = campaign(Hero::LuBu, 10);
    let view = authority::project(&game).unwrap();
    let mut forged = game.clone();
    forged.battle.as_mut().unwrap().fury += 1;
    forged.validate().unwrap();
    assert_eq!(authority::project(&forged).unwrap(), view);
    assert!(forged.audit_composite_authority().unwrap_err().contains("snapshot"));
    assert_ne!(
        serde_json::to_value(view.rebuild_battle().unwrap()).unwrap(),
        serde_json::to_value(&forged.battle).unwrap()
    );
}

#[test]
fn intermediate_runtime_floor_turn_forgery_fails_cross_journal_and_retry_is_atomic() {
    let game = campaign(Hero::GuanYu, 0);
    let mut forged = game.clone();
    if let BattleRuntimeEvent::Changed {
        change: BattleRuntimeChange::Enter { state }, ..
    } = &mut forged.battle_runtime_history.as_mut().unwrap()[2] {
        state.turn += 1;
    } else { panic!("expected battle runtime Enter at revision 2"); }
    assert!(battle_runtime_events::project(
        forged.battle_runtime_history.as_ref().unwrap()).is_ok());
    assert!(authority::project(&forged).unwrap_err().contains("cross-journal"));

    let mut projector = authority::CompositeAuthorityProjector::new();
    for index in 0..2 { projector.push(step(&game, index)).unwrap(); }
    let before = projector.view().unwrap();
    assert!(projector.push(step(&forged, 2)).is_err());
    assert_eq!(projector.view().unwrap(), before);
    projector.push(step(&game, 2)).unwrap();
    assert_eq!(projector.view().unwrap().revision, 2);
}

#[test]
fn forged_intermediate_event_window_cannot_hide_behind_valid_final_nine_domain_state() {
    let game = campaign(Hero::ZhugeLiang, 0);
    assert!(game.audit_event_archive_authority().is_ok());

    for alteration in ["rewrite_payload", "truncate_retained_window"] {
        let mut forged = game.clone();
        // SetDoctrine is revision 1; Choose enters combat at revision 2;
        // EndTurn is revision 3 and retains the original final state.
        let event = &mut forged.battle_runtime_history.as_mut().unwrap()[2];
        let state = match event {
            BattleRuntimeEvent::Changed {
                change: BattleRuntimeChange::Enter { state }, ..
            } => state,
            _ => panic!("expected active combat entry at revision 2"),
        };
        assert!(!state.events.is_empty(), "encounter should emit opening events");
        match alteration {
            "rewrite_payload" => state.events[0].source.push_str(":forged"),
            "truncate_retained_window" => { state.events.remove(0); }
            _ => unreachable!(),
        }

        // Individual runtime history and the final nine-domain projection
        // remain valid. Only prefix-to-runtime event evidence detects this
        // transient rewrite (which was repaired by the later revision).
        assert!(battle_runtime_events::project(
            forged.battle_runtime_history.as_ref().unwrap()).is_ok());
        assert!(forged.audit_composite_authority().is_ok());
        forged.validate().unwrap();
        let error = forged.audit_event_archive_authority().unwrap_err();
        assert!(error.contains("event revision 2: battle runtime window differs"),
            "{alteration}: {error}");

        // The rejection must survive save/restart without rewriting the log.
        let restored = Game::from_json(&forged.to_json().unwrap()).unwrap();
        assert!(restored.audit_event_archive_authority().is_err());
    }
}

#[test]
fn legacy_v2_without_runtime_history_can_resume_but_cannot_claim_complete_recovery() {
    let mut game = campaign(Hero::CaoCao, 0);
    game.battle_runtime_history = None;
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert!(restored.audit_composite_authority().unwrap_err().contains("battle runtime"));
    restored.apply(Action::EndTurn).unwrap();
    assert!(restored.battle_runtime_history.is_none());
    restored.verify_action_replay().unwrap();
}

#[test]
fn cursor_free_legacy_save_without_ninth_domain_stays_uncertified_not_rejected() {
    use three_kingdoms_spire_reborn::telemetry_authority::TelemetryEvent;

    let mut legacy = campaign(Hero::CaoCao, 0);
    legacy.battle_runtime_history = None;
    for entry in legacy.telemetry_history.as_mut().unwrap() {
        match entry {
            TelemetryEvent::Started { archive_len, .. }
            | TelemetryEvent::Changed { archive_len, .. } => *archive_len = None,
        }
    }
    let restored = Game::from_json(&legacy.to_json().unwrap()).unwrap();
    assert_eq!(restored.audit_event_archive_boundaries().unwrap(), None);
    assert!(restored.audit_composite_authority().is_err());
}

fn step(game: &Game, index: usize) -> authority::AuthorityStep<'_> {
    authority::AuthorityStep {
        resources: &game.resource_history.as_ref().unwrap()[index],
        rng: &game.rng_history.as_ref().unwrap()[index],
        combat: &game.battle_resource_history.as_ref().unwrap()[index],
        foes: &game.foe_board_history.as_ref().unwrap()[index],
        cards: &game.card_zone_history.as_ref().unwrap()[index],
        effects: &game.combat_effect_history.as_ref().unwrap()[index],
        runtime: &game.battle_runtime_history.as_ref().unwrap()[index],
        campaign: &game.campaign_history.as_ref().unwrap()[index],
        telemetry: &game.telemetry_history.as_ref().unwrap()[index],
    }
}
