use three_kingdoms_spire_reborn::{
    Action, Card, Game, content::Hero,
    card_zone_events::{
        BattleCardZones, BattleCardsChange, CardZoneEvent, CardZoneProjector, CardZones,
        project, transition,
    },
};

fn piles(floor: u8, turn: u32) -> BattleCardZones {
    BattleCardZones {
        floor, turn,
        draw: vec![Card::new("strike")],
        hand: vec![Card::new("defend")],
        discard: vec![],
        exhausted: vec![],
    }
}

#[test]
fn game_card_journal_reconstructs_deck_and_battle_piles_without_actions() {
    let mut game = Game::new(Hero::ZhugeLiang, 202610241931);
    assert_eq!(game.audit_card_zones().unwrap(), 1);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.battle.is_some());
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.audit_card_zones().unwrap(), game.revision as usize + 1);
    game.verify_action_replay().unwrap();

    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    let mut independent = restored.clone();
    independent.action_history = None;
    independent.event_history = None;
    independent.foe_board_history = None;
    independent.resource_history = None;
    independent.battle_resource_history = None;
    assert_eq!(independent.audit_card_zones().unwrap(), game.revision as usize + 1);

    let mut tampered = game.clone();
    tampered.battle.as_mut().unwrap().hand.clear();
    assert!(tampered.audit_card_zones().is_err());
    let mut truncated = game.clone();
    truncated.card_zone_history.as_mut().unwrap().pop();
    assert!(truncated.audit_card_zones().is_err());

    let original = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
    assert_eq!(game.to_json().unwrap(), original);
}

#[test]
fn card_stream_tracks_permanent_changes_and_ordered_zones() {
    let initial = CardZones { deck: vec![Card::new("strike")], battle: None };
    let mut first = initial.clone();
    first.battle = Some(piles(1, 1));
    let mut second = first.clone();
    second.deck.push(Card::new("defend"));
    let zones = second.battle.as_mut().unwrap();
    zones.hand.swap_remove(0);
    zones.discard.push(Card::new("defend"));
    zones.turn = 2;
    let events = vec![
        CardZoneEvent::Started { deck: initial.deck.clone() },
        transition(&initial, &first, 1).unwrap(),
        transition(&first, &second, 2).unwrap(),
        transition(&second, &second, 3).unwrap(),
    ];
    let mut stream = CardZoneProjector::default();
    for event in &events {
        stream.push(event).unwrap();
    }
    assert_eq!(stream.state().unwrap(), (second.clone(), 3));
    assert_eq!(project(&events).unwrap(), (second.clone(), 3));
    let json = serde_json::to_string(&events).unwrap();
    let decoded: Vec<CardZoneEvent> = serde_json::from_str(&json).unwrap();
    assert_eq!(project(&decoded).unwrap(), (second, 3));
}

#[test]
fn invalid_card_events_are_rejected_without_mutation() {
    let initial = CardZones { deck: vec![Card::new("strike")], battle: None };
    let first = CardZones { battle: Some(piles(1, 1)), ..initial.clone() };
    let mut stream = CardZoneProjector::default();
    assert!(stream.push(&transition(&initial, &first, 1).unwrap()).is_err());
    assert_eq!(stream, CardZoneProjector::default());
    stream.push(&CardZoneEvent::Started { deck: initial.deck.clone() }).unwrap();
    stream.push(&transition(&initial, &first, 1).unwrap()).unwrap();
    let prior = stream.clone();
    let mut temporary = Card::new("strike");
    temporary.temporary = true;
    let invalid = [
        CardZoneEvent::Started { deck: initial.deck.clone() },
        CardZoneEvent::Changed { revision: 3, permanent: None, battle: BattleCardsChange::Unchanged },
        CardZoneEvent::Changed { revision: 2, permanent: Some(vec![temporary]), battle: BattleCardsChange::Unchanged },
        CardZoneEvent::Changed { revision: 2, permanent: None, battle: BattleCardsChange::Idle },
        CardZoneEvent::Changed { revision: 2, permanent: None, battle: BattleCardsChange::Update { zones: piles(2, 1) } },
        CardZoneEvent::Changed { revision: 2, permanent: None, battle: BattleCardsChange::Update { zones: piles(1, 0) } },
    ];
    for event in &invalid {
        assert!(stream.push(event).is_err());
        assert_eq!(stream, prior);
    }
    stream.push(&transition(&first, &initial, 2).unwrap()).unwrap();
    let closed = stream.clone();
    assert!(stream.push(&CardZoneEvent::Changed {
        revision: 3, permanent: None, battle: BattleCardsChange::Enter { zones: piles(1, 1) },
    }).is_err());
    assert_eq!(stream, closed);
    stream.push(&CardZoneEvent::Changed {
        revision: 3, permanent: None, battle: BattleCardsChange::Enter { zones: piles(2, 1) },
    }).unwrap();
    assert_eq!(stream.state().unwrap().1, 3);
}

#[test]
fn legacy_v2_card_history_is_not_fabricated() {
    let mut game = Game::new(Hero::GuanYu, 1234);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&game.to_json().unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("card_zone_history");
    let mut legacy = Game::from_json(&json.to_string()).unwrap();
    assert!(legacy.audit_card_zones().is_err());
    legacy.verify_action_replay().unwrap();
    legacy.apply(Action::EndTurn).unwrap();
    assert!(legacy.card_zone_history.is_none());
    assert!(legacy.audit_card_zones().is_err());
}
