//! Event-only grammar for complete, committed card, turn and potion actions.
//! This validates observable lifecycle ordering without replaying Actions,
//! consulting Game snapshots, or claiming cryptographic provenance.
use crate::{
    combat::EventKind,
    event_stack::{RunEvent, verify_record_coordinate},
};

fn foe_slot(target: Option<&str>) -> bool {
    matches!(target, Some("foe:0" | "foe:1" | "foe:2" | "foe:3"))
}

fn valid_potion(source: &str, target: Option<&str>) -> bool {
    match source {
        "fire" | "poison" => foe_slot(target),
        "weak" => target == Some("all_foes"),
        "healing" | "shield" | "energy" | "strength" | "insight"
        | "cleanse" | "blood" => target.is_none(),
        _ => false,
    }
}

/// A successful play has exactly one matching resolution, with any effects
/// between them. A turn-end either starts the next turn or seals the battle.
/// Potion use is recorded before its effect with its actual target contract.
///
/// This is an immutable, linear-time audit of settled records, not a new
/// gameplay dispatch mechanism. Earlier encounters must end before the next
/// begins; the final encounter may still be active.
pub fn validate(archive: &[RunEvent]) -> Result<(), String> {
    let mut previous = None;
    let mut encounter = 0;
    let mut closed = false;
    let mut pending_card: Option<&str> = None;
    let mut pending_turn_start = false;
    for record in archive {
        verify_record_coordinate(previous, record)?;
        let event = &record.event;
        if record.encounter != encounter {
            if encounter != 0 && (!closed || pending_card.is_some() || pending_turn_start) {
                return Err(format!("encounter {encounter}: unfinished action at boundary"));
            }
            encounter = record.encounter;
            closed = false;
            pending_card = None;
            pending_turn_start = false;
        }
        if closed {
            return Err(format!("encounter {encounter}: event after battle end"));
        }
        match event.kind {
            EventKind::CardPlayed => {
                if pending_card.is_some() || pending_turn_start || event.source.is_empty()
                    || event.amount < 0 || event.target.is_some()
                {
                    return Err(format!("encounter {encounter}: invalid or nested card play"));
                }
                pending_card = Some(&event.source);
            }
            EventKind::CardResolved => {
                if pending_card != Some(event.source.as_str())
                    || event.amount != 0 || event.target.is_some()
                {
                    return Err(format!("encounter {encounter}: orphan or mismatched card resolution"));
                }
                pending_card = None;
            }
            EventKind::TurnEnded => {
                if pending_card.is_some() || pending_turn_start
                    || event.source != "hero" || event.amount != 0 || event.target.is_some()
                {
                    return Err(format!("encounter {encounter}: invalid turn end"));
                }
                pending_turn_start = true;
            }
            EventKind::TurnStarted => {
                if pending_card.is_some() || !pending_turn_start
                    || event.source != "hero" || event.amount != 0 || event.target.is_some()
                {
                    return Err(format!("encounter {encounter}: turn start without end"));
                }
                pending_turn_start = false;
            }
            EventKind::PotionUsed => {
                if pending_card.is_some() || pending_turn_start || event.amount != 1
                    || !valid_potion(&event.source, event.target.as_deref())
                {
                    return Err(format!("encounter {encounter}: invalid potion use"));
                }
            }
            EventKind::BattleEnded => {
                if pending_card.is_some() {
                    return Err(format!("encounter {encounter}: battle sealed during card play"));
                }
                // A lethal poison, enemy attack or attrition may end a battle
                // after TurnEnded without a subsequent TurnStarted.
                pending_turn_start = false;
                closed = true;
            }
            _ => {}
        }
        previous = Some((record.encounter, record.floor, event.seq));
    }
    if pending_card.is_some() || pending_turn_start {
        return Err(format!("encounter {encounter}: incomplete committed action"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Game, combat::CombatEvent, content::Hero};

    fn entry(seq: u64, kind: EventKind, source: &str, amount: i32,
             target: Option<&str>) -> RunEvent {
        RunEvent {
            encounter: 1, floor: 1,
            event: CombatEvent {
                seq, kind, source: source.into(), amount,
                target: target.map(str::to_owned),
            },
        }
    }

    fn complete() -> Vec<RunEvent> {
        vec![
            entry(1, EventKind::CardDrawn, "strike", 1, Some("hero")),
            entry(2, EventKind::CardPlayed, "strike", 1, None),
            entry(3, EventKind::DamageDealt, "hero", 4, Some("foe:0")),
            entry(4, EventKind::CardResolved, "strike", 0, None),
            entry(5, EventKind::TurnEnded, "hero", 0, None),
            entry(6, EventKind::DamageReceived, "foe:0", 1, Some("hero")),
            entry(7, EventKind::TurnStarted, "hero", 0, None),
            entry(8, EventKind::PotionUsed, "fire", 1, Some("foe:0")),
            entry(9, EventKind::DamageDealt, "fire", 4, Some("foe:0")),
            entry(10, EventKind::BattleEnded, "victory", 1, None),
        ]
    }

    #[test]
    fn accepts_complete_action_chains_and_lethal_end_turn() {
        validate(&complete()).unwrap();
        let mut lethal = complete();
        lethal.truncate(6);
        lethal.push(entry(7, EventKind::BattleEnded, "defeat", 0, None));
        validate(&lethal).unwrap();
        let mut active = complete();
        active.truncate(9);
        validate(&active).unwrap();
    }

    #[test]
    fn rejects_card_orphans_mismatches_nested_and_unfinished() {
        let mut events = complete();
        events[3].event.source = "defend".into();
        assert!(validate(&events).is_err());
        let mut events = complete();
        events[3].event.kind = EventKind::CardPlayed;
        assert!(validate(&events).is_err());
        let mut events = complete();
        events[1].event.kind = EventKind::CardResolved;
        assert!(validate(&events).is_err());
        let mut events = complete();
        events.truncate(3);
        assert!(validate(&events).is_err());
        let mut events = complete();
        events[1].event.target = Some("hero".into());
        assert!(validate(&events).is_err());
    }

    #[test]
    fn rejects_turn_sequence_forgery_and_incomplete_turn() {
        let mut events = complete();
        events[4].event.kind = EventKind::TurnStarted;
        assert!(validate(&events).is_err());
        let mut events = complete();
        events[6].event.kind = EventKind::TurnEnded;
        assert!(validate(&events).is_err());
        let mut events = complete();
        events.truncate(6);
        assert!(validate(&events).is_err());
        let mut events = complete();
        events[4].event.source = "foe:0".into();
        assert!(validate(&events).is_err());
    }

    #[test]
    fn rejects_potion_source_target_and_quantity_forgery() {
        for (source, target, amount) in [
            ("fire", None, 1), ("weak", Some("foe:0"), 1),
            ("healing", Some("hero"), 1), ("unknown", None, 1),
            ("fire", Some("foe:0"), 2),
        ] {
            let mut events = complete();
            events[7].event.source = source.into();
            events[7].event.target = target.map(str::to_owned);
            events[7].event.amount = amount;
            assert!(validate(&events).is_err());
        }
    }

    #[test]
    fn checks_real_five_hero_first_turns_and_v2_roundtrip() {
        for hero in [Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
                     Hero::CaoCao, Hero::LuBu] {
            for seed in [11, 20261026] {
                let mut game = Game::new(hero, seed);
                game.apply(Action::Choose { index: 1 }).unwrap();
                // All starter decks have nine Strike/Defend cards. Exercise
                // a real card resolution and the starting healing potion,
                // not just synthetic event fixtures.
                let hand_index = game.battle.as_ref().unwrap().hand.iter()
                    .position(|card| matches!(card.id.as_str(), "strike" | "defend"))
                    .unwrap() + 1;
                game.apply(Action::Play { index: hand_index, target: 1 }).unwrap();
                game.apply(Action::UsePotionAt { slot: 1, target: 1 }).unwrap();
                game.apply(Action::EndTurn).unwrap();
                let archive = game.event_history.as_deref().unwrap();
                validate(archive).unwrap();
                let loaded = Game::from_json(&game.to_json().unwrap()).unwrap();
                validate(loaded.event_history.as_deref().unwrap()).unwrap();
                loaded.audit_event_archive_authority().unwrap();
            }
        }
    }
}
