//! Six committed event journals form a coherent partial authority view.
//! No action replay, RNG stepping or snapshot is needed for the projection.
use three_kingdoms_spire_reborn::{
    Action, Game, RunMode,
    content::Hero,
    event_stack::authority,
    combat_resource_events::{self, BattleResourceChange, BattleResourceEvent},
    card_zone_events::{self, BattleCardsChange, CardZoneEvent},
};

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

fn campaign(hero: Hero, ascension: u8) -> Game {
    let mut game = Game::new_with_identity(
        hero, 20261025, ascension, RunMode::Custom, "composite-authority".into(),
    );
    game.apply(Action::SetDoctrine { id: "auto".into() }).unwrap();
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    game
}

#[test]
fn ten_real_campaigns_have_coherent_authority_after_save_restart() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let game = campaign(hero, ascension);
            let saved = game.to_json().unwrap();
            let restored = Game::from_json(&saved).unwrap();
            let view = authority::project(&restored).unwrap();
            assert_eq!(view.revision, restored.revision);
            assert!(view.combat.is_some());
            assert_eq!(view.cards.deck, restored.deck);
            view.verify_snapshot(&restored).unwrap();
            assert_eq!(restored.audit_composite_authority().unwrap(), view);
            assert_eq!(restored.to_json().unwrap(), saved, "audit changed persisted state");
        }
    }
}

#[test]
fn independent_legal_journals_with_conflicting_combat_turns_are_rejected() {
    let mut game = Game::new(Hero::GuanYu, 20261025);
    game.apply(Action::Choose { index: 1 }).unwrap();
    // This is a perfectly legal standalone combat-resource Enter record;
    // the card, foe and effect streams still correctly say turn 1.
    let history = game.battle_resource_history.as_mut().unwrap();
    let last = history.last_mut().unwrap();
    if let BattleResourceEvent::Changed {
        change: BattleResourceChange::Enter { state }, ..
    } = last {
        state.turn += 1;
    } else {
        panic!("expected combat Enter after first map choice");
    }
    assert!(combat_resource_events::project(history).is_ok(),
        "test must mutate to a separately valid journal");
    let error = authority::project(&game).unwrap_err();
    assert!(error.contains("cross-journal combat floor/turn mismatch"), "{error}");
}

#[test]
fn revision_gaps_and_missing_legacy_journals_never_pass_as_complete() {
    let game = campaign(Hero::ZhugeLiang, 10);
    let mut missing = game.clone();
    missing.card_zone_history = None;
    assert!(authority::project(&missing).unwrap_err().contains("missing card zone"));

    let mut shortened = game.clone();
    shortened.rng_history.as_mut().unwrap().pop();
    assert!(authority::project(&shortened).unwrap_err().contains("record count mismatch"));

    let mut corrupt = game.clone();
    corrupt.foe_board_history.as_mut().unwrap().remove(1);
    assert!(authority::project(&corrupt).is_err());
}

#[test]
fn snapshot_tampering_is_detected_without_mutating_journal_projection() {
    let game = campaign(Hero::CaoCao, 0);
    let expected = authority::project(&game).unwrap();
    let mut forged = game.clone();
    forged.gold += 1;
    assert_eq!(authority::project(&forged).unwrap(), expected,
        "event projection must not consult mutable snapshot fields");
    assert!(expected.verify_snapshot(&forged).unwrap_err().contains("snapshot"));
    assert!(forged.audit_composite_authority().is_err());
}

#[test]
fn revision_zero_has_six_valid_idle_projections() {
    let game = Game::new(Hero::LuBu, 99);
    let view = game.audit_composite_authority().unwrap();
    assert_eq!(view.revision, 0);
    assert!(view.combat.is_none() && view.foes.is_none()
        && view.cards.battle.is_none() && view.effects.is_none());
}

fn step(game: &Game, index: usize) -> authority::AuthorityStep<'_> {
    authority::AuthorityStep {
        resources: &game.resource_history.as_ref().unwrap()[index],
        rng: &game.rng_history.as_ref().unwrap()[index],
        combat: &game.battle_resource_history.as_ref().unwrap()[index],
        foes: &game.foe_board_history.as_ref().unwrap()[index],
        cards: &game.card_zone_history.as_ref().unwrap()[index],
        effects: &game.combat_effect_history.as_ref().unwrap()[index],
    }
}

#[test]
fn transient_cross_stream_conflict_cannot_hide_behind_consistent_final_state() {
    let mut game = Game::new(Hero::GuanYu, 20261025);
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.apply(Action::EndTurn).unwrap();
    let expected = authority::project(&game).unwrap();

    let mut tampered = game.clone();
    // The first battle journal says turn 2 too early, but the second update
    // catches up: final snapshot and individual card-zone projection agree.
    if let CardZoneEvent::Changed {
        battle: BattleCardsChange::Enter { zones }, ..
    } = &mut tampered.card_zone_history.as_mut().unwrap()[1] {
        zones.turn += 1;
    } else {
        panic!("expected card-zone Enter after map choice");
    }
    assert_eq!(card_zone_events::project(
        tampered.card_zone_history.as_ref().unwrap()).unwrap().0,
        expected.cards, "independent final card-zone state must remain valid");
    assert!(authority::project(&tampered).unwrap_err().contains("cross-journal"));

    // Atomicity: the failed revision leaves all six streams at revision zero.
    let mut projector = authority::CompositeAuthorityProjector::new();
    projector.push(step(&game, 0)).unwrap();
    let before = projector.view().unwrap();
    assert!(projector.push(step(&tampered, 1)).is_err());
    assert_eq!(projector.view().unwrap(), before);
    projector.push(step(&game, 1)).unwrap();
    assert_eq!(projector.view().unwrap().revision, 1);
}
