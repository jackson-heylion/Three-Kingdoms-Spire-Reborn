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

/// Incremental grammar for complete, committed card, turn and potion actions.
/// A rejected record does not change the cursor or any pending action. The
/// state holds only the current card identity and encounter coordinates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActionLifecycleProjector {
    previous: Option<(u64, u8, u64)>,
    encounter: u64,
    closed: bool,
    pending_card: Option<String>,
    pending_turn_start: bool,
}

impl ActionLifecycleProjector {
    pub fn new() -> Self { Self::default() }

    /// Stage the small state machine so a malformed record is never committed.
    pub fn push(&mut self, record: &RunEvent) -> Result<(), String> {
        let mut candidate = self.clone();
        candidate.apply(record)?;
        *self = candidate;
        Ok(())
    }

    fn apply(&mut self, record: &RunEvent) -> Result<(), String> {
        verify_record_coordinate(self.previous, record)?;
        let event = &record.event;
        if record.encounter != self.encounter {
            if self.encounter != 0
                && (!self.closed || self.pending_card.is_some() || self.pending_turn_start)
            {
                return Err(format!("encounter {}: unfinished action at boundary", self.encounter));
            }
            self.encounter = record.encounter;
            self.closed = false;
            self.pending_card = None;
            self.pending_turn_start = false;
        }
        if self.closed {
            return Err(format!("encounter {}: event after battle end", self.encounter));
        }
        match event.kind {
            EventKind::CardPlayed => {
                if self.pending_card.is_some() || self.pending_turn_start
                    || event.source.is_empty() || event.amount < 0 || event.target.is_some()
                {
                    return Err(format!("encounter {}: invalid or nested card play", self.encounter));
                }
                self.pending_card = Some(event.source.clone());
            }
            EventKind::CardResolved => {
                if self.pending_card.as_deref() != Some(event.source.as_str())
                    || event.amount != 0 || event.target.is_some()
                {
                    return Err(format!("encounter {}: orphan or mismatched card resolution", self.encounter));
                }
                self.pending_card = None;
            }
            EventKind::TurnEnded => {
                if self.pending_card.is_some() || self.pending_turn_start
                    || event.source != "hero" || event.amount != 0 || event.target.is_some()
                {
                    return Err(format!("encounter {}: invalid turn end", self.encounter));
                }
                self.pending_turn_start = true;
            }
            EventKind::TurnStarted => {
                if self.pending_card.is_some() || !self.pending_turn_start
                    || event.source != "hero" || event.amount != 0 || event.target.is_some()
                {
                    return Err(format!("encounter {}: turn start without end", self.encounter));
                }
                self.pending_turn_start = false;
            }
            EventKind::PotionUsed => {
                if self.pending_card.is_some() || self.pending_turn_start || event.amount != 1
                    || !valid_potion(&event.source, event.target.as_deref())
                {
                    return Err(format!("encounter {}: invalid potion use", self.encounter));
                }
            }
            EventKind::BattleEnded => {
                if self.pending_card.is_some() {
                    return Err(format!("encounter {}: battle sealed during card play", self.encounter));
                }
                // Lethal poison, an enemy attack or attrition can seal the
                // encounter after TurnEnded without a subsequent TurnStarted.
                self.pending_turn_start = false;
                self.closed = true;
            }
            _ => {}
        }
        self.previous = Some((record.encounter, record.floor, event.seq));
        Ok(())
    }

    /// An action revision must not end with a played-but-unresolved card or
    /// a half-completed enemy turn. This is checked after every persisted cursor.
    pub fn committed(&self) -> Result<(), String> {
        if self.pending_card.is_some() || self.pending_turn_start {
            return Err(format!("encounter {}: incomplete committed action", self.encounter));
        }
        Ok(())
    }
}

/// Validate a complete event archive with the same incremental grammar used
/// at action revision boundaries. This is event-only audit, not Game replay.
pub fn validate(archive: &[RunEvent]) -> Result<(), String> {
    let mut projector = ActionLifecycleProjector::new();
    for record in archive {
        projector.push(record)?;
    }
    projector.committed()
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
    fn committed_revision_cannot_split_a_card_or_enemy_turn() {
        let records = complete();
        let mut projector = ActionLifecycleProjector::new();
        for record in &records[..2] { projector.push(record).unwrap(); }
        assert!(projector.committed().unwrap_err().contains("incomplete committed action"));
        let before = projector.clone();
        let mut forged = records[2].clone();
        forged.event.kind = EventKind::CardResolved;
        forged.event.source = "other_card".into();
        assert!(projector.push(&forged).is_err());
        assert_eq!(projector, before, "rejected event must not move cursor");
        for record in &records[2..4] { projector.push(record).unwrap(); }
        projector.committed().unwrap();
        projector.push(&records[4]).unwrap();
        assert!(projector.committed().is_err(), "TurnEnded is not a complete revision");
        for record in &records[5..7] { projector.push(record).unwrap(); }
        projector.committed().unwrap();
        for record in &records[7..] { projector.push(record).unwrap(); }
        projector.committed().unwrap();
    }

    #[test]
    fn forged_action_cursor_cannot_borrow_card_resolution_from_next_revision() {
        let mut game = Game::new(Hero::GuanYu, 11);
        game.apply(Action::Choose { index: 1 }).unwrap();
        let index = game.battle.as_ref().unwrap().hand.iter()
            .position(|card| matches!(card.id.as_str(), "strike" | "defend"))
            .unwrap() + 1;
        game.apply(Action::Play { index, target: 1 }).unwrap();
        let play_revision = usize::try_from(game.revision).unwrap();
        let split_at = game.event_history.as_ref().unwrap().iter()
            .rposition(|record| record.event.kind == EventKind::CardResolved)
            .unwrap();
        game.apply(Action::EndTurn).unwrap();
        assert!(game.audit_event_archive_boundaries().unwrap().is_some());

        // The full event archive is still valid. Forge only the persisted
        // revision cursor to pretend that CardResolved arrived next action.
        let mut forged = game.clone();
        match &mut forged.telemetry_history.as_mut().unwrap()[play_revision] {
            crate::telemetry_authority::TelemetryEvent::Changed { archive_len, .. } => {
                *archive_len = Some(split_at as u64);
            }
            _ => panic!("expected committed telemetry revision"),
        }
        validate(forged.event_history.as_deref().unwrap()).unwrap();
        let error = forged.audit_event_archive_boundaries().unwrap_err();
        assert!(error.contains("incomplete committed action"), "{error}");
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
                assert!(loaded.audit_event_archive_boundaries().unwrap().is_some());
            }
        }
    }
}
