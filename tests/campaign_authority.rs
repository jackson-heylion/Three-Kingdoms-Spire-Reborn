//! Independent map/route/lore/inventory projection, then seven-stream audit.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, Stage, campaign_authority::{self, CampaignEvent, CampaignState},
    content::Hero, event_stack::authority,
};
const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

#[test]
fn ten_campaigns_recover_noncombat_state_without_action_replay() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let mut game = Game::new_with_identity(
                hero, 20261025, ascension, RunMode::Custom, "campaign-authority".into());
            game.apply(Action::SetDoctrine { id: "auto".into() }).unwrap();
            game.apply(Action::Choose { index: 1 }).unwrap();
            game.apply(Action::EndTurn).unwrap();
            let raw = game.to_json().unwrap();
            let mut restored = Game::from_json(&raw).unwrap();
            let (state, revision) = campaign_authority::project(
                restored.campaign_history.as_ref().unwrap()).unwrap();
            assert_eq!(revision, restored.revision);
            assert_eq!(state, CampaignState::current(&restored));
            assert_eq!(state.stage, Stage::Combat);
            restored.action_history = None;
            restored.event_history = None;
            assert_eq!(restored.audit_campaign_authority().unwrap(), 4);
            assert_eq!(restored.audit_composite_authority().unwrap().campaign, state);
        }
    }
}

#[test]
fn forged_snapshot_and_separately_valid_campaign_floor_are_rejected() {
    let mut game = Game::new(Hero::GuanYu, 20261025);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let expected = campaign_authority::project(game.campaign_history.as_ref().unwrap()).unwrap().0;
    let mut snapshot = game.clone();
    snapshot.relics.push("forged".into());
    assert_eq!(campaign_authority::project(snapshot.campaign_history.as_ref().unwrap()).unwrap().0, expected);
    assert!(snapshot.audit_campaign_authority().is_err());

    let mut forged = game.clone();
    match forged.campaign_history.as_mut().unwrap().last_mut().unwrap() {
        CampaignEvent::Changed { state: Some(state), .. } => state.floor = 2,
        other => panic!("expected campaign state change: {other:?}"),
    }
    assert!(campaign_authority::project(forged.campaign_history.as_ref().unwrap()).is_ok());
    assert!(authority::project(&forged).unwrap_err().contains("cross-journal campaign"));
}

#[test]
fn failed_projection_is_atomic_and_legacy_absence_is_not_fabricated() {
    let mut game = Game::new(Hero::ZhugeLiang, 777);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let events = game.campaign_history.as_ref().unwrap();
    let mut projector = campaign_authority::CampaignProjector::default();
    projector.push(&events[0]).unwrap();
    let original = projector.state().unwrap();
    let invalid = CampaignEvent::Changed { revision: 2, state: None };
    assert!(projector.push(&invalid).is_err());
    assert_eq!(projector.state().unwrap(), original);
    projector.push(&events[1]).unwrap();
    assert_eq!(projector.state().unwrap().1, 1);

    let mut old = game.clone();
    old.campaign_history = None;
    let raw = old.to_json().unwrap();
    let loaded = Game::from_json(&raw).unwrap();
    assert!(loaded.campaign_history.is_none());
    assert!(loaded.audit_campaign_authority().unwrap_err().contains("legacy"));
    assert!(loaded.audit_composite_authority().is_err());
    assert!(loaded.verify_action_replay().is_ok());
}

#[test]
fn rejected_actions_preserve_campaign_journal_and_all_game_bytes() {
    let mut game = Game::new(Hero::LuBu, 22);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
}

#[test]
fn revision_zero_anchors_identity_for_all_heroes_and_run_modes() {
    for hero in HEROES {
        for mode in [RunMode::Standard, RunMode::Custom, RunMode::Daily] {
            let run_id = format!("identity-{hero:?}-{mode:?}");
            let mut game = Game::new_with_identity(hero, 20261025, 4, mode, run_id.clone());
            let initial = game.audit_composite_authority().unwrap();
            let identity = initial.campaign.identity.as_ref().unwrap();
            assert_eq!(identity.hero, hero);
            assert_eq!(identity.seed, 20261025);
            assert_eq!(identity.ascension, 4);
            assert_eq!(identity.mode, mode);
            assert_eq!(identity.run_id, run_id);
            game.apply(Action::Choose { index: 1 }).unwrap();
            let saved = game.to_json().unwrap();
            let restored = Game::from_json(&saved).unwrap();
            restored.audit_composite_authority().unwrap();
            restored.verify_action_replay().unwrap();

            let mut forged = restored.clone();
            forged.seed ^= 1;
            assert!(forged.audit_composite_authority().is_err());
            let mut forged = restored.clone();
            forged.hero = if hero == Hero::GuanYu { Hero::LuBu } else { Hero::GuanYu };
            assert!(forged.audit_composite_authority().is_err());
            let mut forged = restored.clone();
            forged.ascension = 5;
            assert!(forged.audit_composite_authority().is_err());
            let mut forged = restored.clone();
            forged.mode = if mode == RunMode::Custom { RunMode::Daily } else { RunMode::Custom };
            assert!(forged.audit_composite_authority().is_err());
            let mut forged = restored.clone();
            forged.run_id.push_str("-forged");
            assert!(forged.audit_composite_authority().is_err());
        }
    }
}

#[test]
fn identity_cannot_change_mid_campaign_and_legacy_saves_stay_playable() {
    let mut game = Game::new_with_identity(
        Hero::ZhugeLiang, 42, 0, RunMode::Custom, "stable-run".into());
    game.apply(Action::Choose { index: 1 }).unwrap();
    let mut tampered = game.clone();
    if let CampaignEvent::Changed { state: Some(state), .. } =
        &mut tampered.campaign_history.as_mut().unwrap()[1]
    {
        state.identity.as_mut().unwrap().run_id.push_str("-swapped");
    } else {
        panic!("expected campaign transition");
    }
    assert!(campaign_authority::project(tampered.campaign_history.as_ref().unwrap())
        .unwrap_err().contains("identity changed"));

    // A pre-upgrade V2 journal did not anchor identity. It remains loadable
    // and actionable, but cannot claim the new identity certification.
    let mut legacy = game.clone();
    for event in legacy.campaign_history.as_mut().unwrap() {
        match event {
            CampaignEvent::Started { state } => state.identity = None,
            CampaignEvent::Changed { state: Some(state), .. } => state.identity = None,
            _ => {}
        }
    }
    let raw = legacy.to_json().unwrap();
    let mut loaded = Game::from_json(&raw).unwrap();
    assert!(loaded.audit_composite_authority().is_err());
    loaded.apply(Action::EndTurn).unwrap();
    assert_eq!(Game::from_json(&loaded.to_json().unwrap()).unwrap().revision, 2);
}

#[test]
fn authoritative_transition_rejects_identity_swap_before_commit() {
    let game = Game::new_with_identity(
        Hero::GuanYu, 20261123, 0, RunMode::Custom, "identity-guard".into());
    let before = CampaignState::current(&game);
    let mut after = before.clone();
    after.identity.as_mut().unwrap().run_id.push_str("-forged");
    assert!(campaign_authority::transition(&before, &after, 1)
        .unwrap_err().contains("invalid campaign state transition"));

    let unchanged = campaign_authority::transition(&before, &before, 1).unwrap();
    assert!(matches!(unchanged, CampaignEvent::Changed { revision: 1, state: None }));

    // Pre-upgrade journals without an identity must not be rejected merely
    // because the current Game snapshot carries an identity.
    let mut legacy = before.clone();
    legacy.identity = None;
    assert!(campaign_authority::transition(&legacy, &before, 1).is_ok());
}

#[test]
fn legacy_identity_free_journal_remains_projectable_after_multiple_real_actions() {
    let mut game = Game::new_with_identity(
        Hero::GuanYu, 20261123, 0, RunMode::Standard, "legacy-resume".into());
    game.apply(Action::Choose { index: 1 }).unwrap();
    for event in game.campaign_history.as_mut().unwrap() {
        match event {
            CampaignEvent::Started { state } => state.identity = None,
            CampaignEvent::Changed { state: Some(state), .. } => state.identity = None,
            _ => {}
        }
    }
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert!(restored.audit_composite_authority().is_err());
    restored.apply(Action::EndTurn).unwrap();
    let (state, revision) =
        campaign_authority::project(restored.campaign_history.as_ref().unwrap()).unwrap();
    assert_eq!(revision, restored.revision);
    assert!(state.identity.is_none(), "legacy journal must not gain certified identity");
    let reloaded = Game::from_json(&restored.to_json().unwrap()).unwrap();
    assert_eq!(campaign_authority::project(reloaded.campaign_history.as_ref().unwrap()).unwrap().0, state);
    assert!(reloaded.audit_composite_authority().is_err());
    // This fixture strips identity from a modern journal after the actions ran.
    // It must remain playable/projectable, but must not pass certified replay:
    // the original action trace still records the unmodified modern journal.
    assert!(reloaded.verify_action_replay().is_err());
}


#[test]
fn historical_decision_ledger_is_conservative_and_replay_stable() {
    let mut legacy = Game::new(Hero::GuanYu, 20261124);
    legacy.apply(Action::Choose { index: 1 }).unwrap();
    let journal = legacy.campaign_history.as_ref().unwrap();
    assert!(campaign_authority::historical_decisions(journal).unwrap().is_empty());
    let saved = legacy.to_json().unwrap();
    let restored = Game::from_json(&saved).unwrap();
    assert_eq!(
        campaign_authority::historical_decisions(journal).unwrap(),
        campaign_authority::historical_decisions(restored.campaign_history.as_ref().unwrap()).unwrap()
    );
    let mut forged = journal.clone();
    forged.push(CampaignEvent::Changed { revision: 1, state: None });
    assert!(campaign_authority::historical_decisions(&forged).is_err());
}

#[test]
fn historical_decisions_count_first_tactical_choice_not_later_resolution() {
    use three_kingdoms_spire_reborn::{WuchaoPlan, difficulty::Difficulty};
    let game = Game::new(Hero::GuanYu, 20261124);
    let mut initial = CampaignState::current(&game);
    initial.identity.as_mut().unwrap().difficulty = Some(Difficulty::Warlords);
    let mut pending = initial.clone();
    pending.wuchao_plan = Some(WuchaoPlan::RaidPending);
    let mut resolved = pending.clone();
    resolved.wuchao_plan = Some(WuchaoPlan::RaidResolved);
    let mut alternate = resolved.clone();
    alternate.wuchao_plan = Some(WuchaoPlan::FortifyResolved);
    let events = vec![
        CampaignEvent::Started { state: initial.clone() },
        campaign_authority::transition(&initial, &pending, 1).unwrap(),
        campaign_authority::transition(&pending, &resolved, 2).unwrap(),
        campaign_authority::transition(&resolved, &alternate, 3).unwrap(),
    ];
    let decisions = campaign_authority::historical_decisions(&events).unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].revision, 1);
    assert_eq!(decisions[0].kind, "wuchao");
    assert_eq!(decisions[0].id, format!("{}:wuchao:1", game.run_id));
}

#[test]
fn historical_decisions_ignore_council_placeholder_until_player_commits() {
    use three_kingdoms_spire_reborn::{WuchaoPlan, difficulty::Difficulty};
    let game = Game::new(Hero::GuanYu, 20261124);
    let mut initial = CampaignState::current(&game);
    initial.identity.as_mut().unwrap().difficulty = Some(Difficulty::Warlords);
    let mut council = initial.clone();
    council.stage = Stage::Story;
    council.wuchao_plan = Some(WuchaoPlan::Council);
    let mut committed = council.clone();
    committed.wuchao_plan = Some(WuchaoPlan::RaidPending);
    let events = vec![
        CampaignEvent::Started { state: initial.clone() },
        campaign_authority::transition(&initial, &council, 1).unwrap(),
        campaign_authority::transition(&council, &committed, 2).unwrap(),
    ];
    let decisions = campaign_authority::historical_decisions(&events).unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].revision, 2);
}

#[test]
fn historical_decisions_do_not_misclassify_settlement_only_journal_changes() {
    use three_kingdoms_spire_reborn::{WuchaoPlan, ChibiPlan, JietingPlan, difficulty::Difficulty};
    let game = Game::new(Hero::ZhugeLiang, 20261124);
    let mut initial = CampaignState::current(&game);
    initial.identity.as_mut().unwrap().difficulty = Some(Difficulty::Warlords);
    initial.floor = 25;
    let mut settled = initial.clone();
    settled.wuchao_plan = Some(WuchaoPlan::RaidResolved);
    settled.chibi_plan = Some(ChibiPlan::FireSettled);
    settled.jieting_plan = Some(JietingPlan::WaterSettled);
    let events = vec![
        CampaignEvent::Started { state: {
            let mut start = initial.clone();
            start.floor = 0;
            start
        } },
        campaign_authority::transition(&{
            let mut start = initial.clone();
            start.floor = 0;
            start
        }, &initial, 1).unwrap(),
        campaign_authority::transition(&initial, &settled, 2).unwrap(),
    ];
    assert!(campaign_authority::historical_decisions(&events).unwrap().is_empty(),
        "a settlement without a committed pending choice cannot count as player interaction");
}
