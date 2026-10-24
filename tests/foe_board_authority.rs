use three_kingdoms_spire_reborn::{
    Action, Foe, Game, Intent, content::Hero,
    foe_board_events::{
        FoeBoard, FoeBoardChange, FoeBoardEvent, FoeBoardProjector, project, transition,
    },
};

fn board(floor: u8, turn: u32, hp: i32) -> FoeBoard {
    FoeBoard {
        floor,
        turn,
        foes: vec![Foe {
            id: "bandit".into(), hp, max_hp: 30, block: 0,
            strength: 0, poison: 0, vulnerable: 0, weak: 0,
            intent: Intent::Attack(6), phase: 0,
        }],
    }
}

#[test]
fn committed_enemy_board_rebuilds_without_actions_or_rng() {
    let mut game = Game::new(Hero::ZhugeLiang, 202610241931);
    assert_eq!(game.audit_foe_board().unwrap(), 1);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.battle.is_some());
    game.apply(Action::EndTurn).unwrap();
    assert_eq!(game.audit_foe_board().unwrap(), game.revision as usize + 1);
    game.verify_action_replay().unwrap();

    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(restored.audit_foe_board().unwrap(), game.audit_foe_board().unwrap());
    let mut without_actions = restored.clone();
    without_actions.action_history = None;
    without_actions.event_history = None;
    without_actions.resource_history = None;
    without_actions.battle_resource_history = None;
    assert_eq!(without_actions.audit_foe_board().unwrap(), game.revision as usize + 1);

    let mut tampered = game.clone();
    tampered.battle.as_mut().unwrap().foes[0].hp -= 1;
    assert!(tampered.audit_foe_board().is_err());
    let mut truncated = game.clone();
    truncated.foe_board_history.as_mut().unwrap().pop();
    assert!(truncated.audit_foe_board().is_err());

    let original = game.to_json().unwrap();
    assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
    assert_eq!(game.to_json().unwrap(), original);
}

#[test]
fn enemy_board_stream_reconstructs_intents_statuses_and_lifecycle() {
    let first = board(1, 1, 30);
    let mut second = board(1, 2, 21);
    second.foes[0].poison = 2;
    second.foes[0].block = 4;
    second.foes[0].intent = Intent::Guard(7);
    let next = board(2, 1, 30);
    let events = vec![
        FoeBoardEvent::Started,
        transition(None, None, 1).unwrap(),
        transition(None, Some(first.clone()), 2).unwrap(),
        transition(Some(first), Some(second.clone()), 3).unwrap(),
        transition(Some(second), None, 4).unwrap(),
        transition(None, Some(next.clone()), 5).unwrap(),
    ];
    let mut stream = FoeBoardProjector::default();
    for event in &events {
        stream.push(event).unwrap();
    }
    assert_eq!(stream.state().unwrap(), (Some(next.clone()), 5));
    assert_eq!(project(&events).unwrap(), (Some(next.clone()), 5));
    let json = serde_json::to_string(&events).unwrap();
    let decoded: Vec<FoeBoardEvent> = serde_json::from_str(&json).unwrap();
    assert_eq!(project(&decoded).unwrap(), (Some(next), 5));
}

#[test]
fn enemy_board_rejects_invalid_and_repeated_encounters_atomically() {
    let first = board(1, 1, 30);
    let mut stream = FoeBoardProjector::default();
    assert!(stream.push(&transition(None, None, 1).unwrap()).is_err());
    assert_eq!(stream, FoeBoardProjector::default());
    stream.push(&FoeBoardEvent::Started).unwrap();
    stream.push(&transition(None, Some(first.clone()), 1).unwrap()).unwrap();
    let prior = stream.clone();
    let invalid = [
        FoeBoardEvent::Started,
        transition(Some(first.clone()), None, 3).unwrap(),
        FoeBoardEvent::Changed {
            revision: 2, change: FoeBoardChange::Idle,
        },
        FoeBoardEvent::Changed {
            revision: 2, change: FoeBoardChange::Update { board: board(1, 0, 30) },
        },
        FoeBoardEvent::Changed {
            revision: 2, change: FoeBoardChange::Update { board: board(1, 2, -1) },
        },
        FoeBoardEvent::Changed {
            revision: 2, change: FoeBoardChange::Update {
                board: FoeBoard { foes: vec![], ..board(1, 2, 30) },
            },
        },
    ];
    for event in &invalid {
        assert!(stream.push(event).is_err());
        assert_eq!(stream, prior);
    }
    assert!(transition(Some(first.clone()), Some(board(2, 1, 30)), 2).is_err());
    stream.push(&transition(Some(first), None, 2).unwrap()).unwrap();
    let closed = stream.clone();
    assert!(stream.push(&transition(None, Some(board(1, 1, 30)), 3).unwrap()).is_err());
    assert_eq!(stream, closed);
    stream.push(&transition(None, Some(board(2, 1, 30)), 3).unwrap()).unwrap();
    assert_eq!(stream.state().unwrap().1, 3);
}

#[test]
fn old_v2_snapshots_do_not_fabricate_enemy_board_history() {
    let mut game = Game::new(Hero::GuanYu, 1234);
    game.apply(Action::Choose { index: 1 }).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&game.to_json().unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("foe_board_history");
    let mut legacy = Game::from_json(&json.to_string()).unwrap();
    assert!(legacy.audit_foe_board().is_err());
    legacy.verify_action_replay().unwrap();
    legacy.apply(Action::EndTurn).unwrap();
    assert!(legacy.foe_board_history.is_none());
    assert!(legacy.audit_foe_board().is_err());
}
