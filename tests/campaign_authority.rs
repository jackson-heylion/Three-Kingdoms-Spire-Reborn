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
