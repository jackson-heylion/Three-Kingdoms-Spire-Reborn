//! Replay-derived, exact synchronous parent edges between combat events.
//!
//! Parent means that an event was emitted while another event's observer
//! dispatch was on the call stack. It does NOT mean that consecutive events
//! share a parent, or that a parent was inferred from timestamps or source IDs.
//! These edges are diagnostic, never stored in V2 saves or trusted on their own.
use crate::{Game, event_stack::RunEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventCoordinate {
    pub encounter: u64,
    pub seq: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventParentRecord {
    pub event: RunEvent,
    /// None means no synchronous dispatch parent was observed.
    pub parent: Option<EventCoordinate>,
}

fn validate_edges(trace: &[EventParentRecord]) -> Result<(), String> {
    let mut seen = HashSet::with_capacity(trace.len());
    for entry in trace {
        let id = EventCoordinate {
            encounter: entry.event.encounter,
            seq: entry.event.event.seq,
        };
        if let Some(parent) = entry.parent
            && (parent.encounter != id.encounter
                || parent.seq >= id.seq
                || !seen.contains(&parent))
        {
            return Err("invalid synchronous event parent coordinate".into());
        }
        if !seen.insert(id) {
            return Err("duplicate synchronous event coordinate".into());
        }
    }
    Ok(())
}

impl Game {
    /// Replay committed actions with temporary observer-stack instrumentation.
    /// Every emitted event must match the persisted archive in the same order.
    /// The final entire Game must also match; missing legacy journals are
    /// rejected, rather than inventing event parents after the fact.
    ///
    /// Only nested synchronous observer emissions get a parent edge. Ordinary
    /// sequential card effects, RNG draws and unrelated action effects remain
    /// roots: there is no speculative "nearest event" parent attribution.
    pub fn reconstruct_event_dispatch_parents(&self) -> Result<Vec<EventParentRecord>, String> {
        let source = self.event_history.as_ref()
            .ok_or("event parent tracing requires a complete combat event archive")?;
        let actions = self.action_history.as_ref()
            .ok_or("event parent tracing requires a complete action journal")?;
        let mut replay = self.reconstruct_game_at(0)?;
        if replay.event_history.as_deref() != Some(&source[..0]) {
            return Err("event parent tracing initial event archive differs".into());
        }
        replay.event_parent_trace = Some(Vec::new());
        for (index, action) in actions.iter().enumerate() {
            replay.apply(action.clone()).map_err(|error|
                format!("event parent replay revision {} failed: {error}", index + 1))?;
            let emitted = replay.event_history.as_ref()
                .ok_or("event parent replay lost event archive")?;
            if source.get(..emitted.len()) != Some(emitted.as_slice()) {
                return Err(format!("event parent archive differs at revision {}", index + 1));
            }
        }
        if replay.to_json()? != self.to_json()? {
            return Err("event parent replay differs from final complete Game".into());
        }
        let trace = replay.event_parent_trace.take()
            .ok_or("event parent instrumentation lost during replay")?;
        if trace.len() != source.len()
            || !trace.iter().zip(source).all(|(link, event)| &link.event == event)
        {
            return Err("event parent trace differs from complete archived events".into());
        }
        validate_edges(&trace)?;
        Ok(trace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, combat::{EventKind, PowerId, PowerStack}, content::Hero};

    #[test]
    fn nested_observer_has_exact_parent_and_sequential_root_has_none() {
        let mut game = Game::new(Hero::ZhugeLiang, 20261025);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.battle.as_mut().unwrap().powers.push(PowerStack {
            id: PowerId::WisdomOnDraw, stacks: 1,
        });
        game.event_parent_trace = Some(Vec::new());
        game.combat_event(EventKind::CardDrawn, "strike", 1, None);
        game.combat_event(EventKind::CardDrawn, "defend", 1, None);
        let trace = game.event_parent_trace.take().unwrap();
        assert_eq!(trace.len(), 4);
        assert_eq!(trace[0].event.event.kind, EventKind::CardDrawn);
        assert_eq!(trace[1].event.event.kind, EventKind::PowerTriggered);
        assert_eq!(trace[2].event.event.kind, EventKind::CardDrawn);
        assert_eq!(trace[3].event.event.kind, EventKind::PowerTriggered);
        assert_eq!(trace[0].parent, None);
        assert_eq!(trace[1].parent, Some(EventCoordinate {
            encounter: trace[0].event.encounter, seq: trace[0].event.event.seq,
        }));
        assert_eq!(trace[2].parent, None);
        assert_eq!(trace[3].parent, Some(EventCoordinate {
            encounter: trace[2].event.encounter, seq: trace[2].event.event.seq,
        }));
        validate_edges(&trace).unwrap();
        // The full replay method also checks the entire original event archive.
    }

    #[test]
    fn forged_or_future_parent_edges_are_rejected() {
        let mk = |seq, parent| EventParentRecord {
            event: RunEvent {
                encounter: 1, floor: 1,
                event: crate::combat::CombatEvent {
                    seq, kind: EventKind::CardDrawn, source: "test".into(),
                    amount: 1, target: None,
                },
            },
            parent,
        };
        let root = mk(1, None);
        let child = mk(2, Some(EventCoordinate { encounter: 1, seq: 1 }));
        assert!(validate_edges(&[root.clone(), child]).is_ok());
        assert!(validate_edges(&[root.clone(), mk(2, Some(EventCoordinate {
            encounter: 1, seq: 2,
        }))]).is_err());
        assert!(validate_edges(&[root.clone(), mk(2, Some(EventCoordinate {
            encounter: 2, seq: 1,
        }))]).is_err());
        assert!(validate_edges(&[root.clone(), root]).is_err());
    }
}
