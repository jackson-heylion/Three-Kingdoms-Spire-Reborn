use three_kingdoms_spire_reborn::{
    Action, Game, RunMode, content::Hero, event_parent::EventCoordinate,
};
use std::collections::HashSet;

const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

fn campaign(hero: Hero, ascension: u8) -> Game {
    let mut game = Game::new_with_identity(
        hero, 20261025, ascension, RunMode::Custom, "event-parent-test".into(),
    );
    for action in [
        Action::SetDoctrine { id: "auto".into() },
        Action::Choose { index: 1 },
        Action::EndTurn,
    ] {
        game.apply(action).unwrap();
    }
    game
}

#[test]
fn ten_campaigns_reconstruct_exact_dispatch_parent_edges_after_restart() {
    for hero in HEROES {
        for ascension in [0, 10] {
            let game = campaign(hero, ascension);
            let saved = game.to_json().unwrap();
            let restarted = Game::from_json(&saved).unwrap();
            let trace = restarted.reconstruct_event_dispatch_parents().unwrap();
            let archive = restarted.event_history.as_ref().unwrap();
            assert!(!archive.is_empty(), "{hero:?}/{ascension}");
            assert_eq!(trace.len(), archive.len(), "{hero:?}/{ascension}");
            let mut emitted = HashSet::new();
            for (link, original) in trace.iter().zip(archive) {
                assert_eq!(&link.event, original);
                let coordinate = EventCoordinate {
                    encounter: original.encounter, seq: original.event.seq,
                };
                if let Some(parent) = link.parent {
                    assert_eq!(parent.encounter, coordinate.encounter);
                    assert!(parent.seq < coordinate.seq);
                    assert!(emitted.contains(&parent), "parent must precede child");
                }
                assert!(emitted.insert(coordinate), "duplicate coordinate");
            }
            assert_eq!(restarted.to_json().unwrap(), saved, "audit must not write to save");
        }
    }
}

#[test]
fn missing_or_rewritten_authority_is_not_promoted_to_parent_evidence() {
    let game = campaign(Hero::ZhugeLiang, 10);
    let mut edited = game.clone();
    edited.event_history.as_mut().unwrap()[0].event.amount += 1;
    assert!(edited.reconstruct_event_dispatch_parents().is_err());

    let mut missing_archive = game.clone();
    missing_archive.event_history = None;
    assert!(missing_archive.reconstruct_event_dispatch_parents().unwrap_err()
        .contains("complete combat event archive"));

    let mut missing_actions = game.clone();
    missing_actions.action_history = None;
    assert!(missing_actions.reconstruct_event_dispatch_parents().is_err());

    let mut truncated = game.clone();
    truncated.action_history.as_mut().unwrap().pop();
    assert!(truncated.reconstruct_event_dispatch_parents().is_err());

    let mut forged_snapshot = game.clone();
    forged_snapshot.gold += 1;
    assert!(forged_snapshot.reconstruct_event_dispatch_parents().is_err());
}
