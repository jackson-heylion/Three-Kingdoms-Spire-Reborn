//! Offline RNG draw attribution from the xorshift64* state transition.
//!
//! This counts consumed RNG transitions and captures their ordering relative
//! to emitted combat events, without changing the save schema or gameplay RNG.
//! It does not establish parent-child causality or trusted event sourcing.
use crate::{Action, Game, rng_events::RngEvent};

/// Diagnostic upper bound, not a gameplay restriction. A legitimate action
/// exceeding this budget is reported as *unattributed*, never rejected by apply.
pub const MAX_ATTRIBUTED_DRAWS_PER_ACTION: u32 = 8192;

/// A single actual call to Game::random while replaying one committed action.
/// upper<=1 returns zero without advancing RNG; such calls remain visible.
#[derive(Clone, Debug, serde::Serialize)]
pub struct RngCall {
    pub call_site: String,
    pub upper: usize,
    pub value: usize,
    pub state_before: u64,
    pub state_after: u64,
    /// Number of archived combat events already emitted when random() ran.
    /// None for old saves without an event archive; NOT a parent-event ID.
    pub event_cursor_before: Option<usize>,
    /// Exact synchronous observer currently dispatching when this draw ran.
    /// None for ordinary action effects; never inferred from event adjacency.
    pub dispatch_parent: Option<crate::event_parent::EventCoordinate>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RngDrawAttribution {
    pub revision: u64,
    pub action: Action,
    pub state_before: u64,
    pub state_after: u64,
    pub draws: u32,
    /// Exclusive event archive slice emitted by this action; absent on old saves.
    pub event_start: Option<usize>,
    pub event_end: Option<usize>,
    /// Every random() invocation, including zero-consumption bounds.
    pub calls: Vec<RngCall>,
}

/// Exact transition used by Game::random when upper > 1.
/// Game::random(0 or 1) consumes zero transitions.
fn advance(state: u64) -> u64 {
    let mut x = state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    x
}

fn infer_draws(before: u64, after: u64) -> Result<u32, String> {
    if before == 0 || after == 0 {
        return Err("invalid zero RNG state in draw attribution".into());
    }
    let mut state = before;
    for draws in 0..=MAX_ATTRIBUTED_DRAWS_PER_ACTION {
        if state == after {
            return Ok(draws);
        }
        state = advance(state);
    }
    Err(format!(
        "RNG transition exceeds {} draws for one action; attribution unavailable",
        MAX_ATTRIBUTED_DRAWS_PER_ACTION,
    ))
}

impl Game {
    /// Replay successful actions, compare each committed RNG journal boundary,
    /// and infer the number of xorshift transitions for each action.
    ///
    /// Unlike merely comparing the final RNG state, this identifies which
    /// action consumed each bounded group of draws. Old saves missing either
    /// journal are explicitly unsupported. The final *whole Game* must match.
    /// This is read-only and does not mutate the live save or RNG.
    pub fn reconstruct_rng_draw_attribution(&self) -> Result<Vec<RngDrawAttribution>, String> {
        let journal = self.rng_history.as_ref()
            .ok_or("RNG draw attribution requires a complete RNG journal")?;
        let actions = self.action_history.as_ref()
            .ok_or("RNG draw attribution requires a complete action journal")?;
        if journal.len() != actions.len().checked_add(1)
            .ok_or("RNG attribution journal length overflow")?
        {
            return Err("RNG journal and action count do not align".into());
        }
        let mut replay = self.reconstruct_game_at(0)?;
        if replay.rng_history.as_ref().and_then(|events| events.first()) != journal.first()
            || !matches!(journal.first(), Some(RngEvent::Started { .. }))
        {
            return Err("initial RNG journal state differs from reconstructed Game".into());
        }
        let mut attributed = Vec::with_capacity(actions.len());
        for (index, action) in actions.iter().enumerate() {
            let before = replay.rng_state;
            let event_start = replay.event_history.as_ref().map(Vec::len);
            replay.rng_trace = Some(Vec::new());
            replay.apply(action.clone()).map_err(|error|
                format!("RNG attribution replay action {} failed: {error}", index + 1))?;
            let calls = replay.rng_trace.take()
                .ok_or("RNG call-site instrumentation was lost during action replay")?;
            let event_end = replay.event_history.as_ref().map(Vec::len);
            if event_start.is_some() != event_end.is_some() {
                return Err("event archive availability changed during RNG replay".into());
            }
            if let (Some(start), Some(end)) = (event_start, event_end) {
                let archive = replay.event_history.as_ref().unwrap();
                let source = self.event_history.as_ref().unwrap();
                if end < start || source.get(start..end) != archive.get(start..end) {
                    return Err(format!("event archive diverged at RNG revision {}", index + 1));
                }
            }
            let committed = replay.rng_history.as_ref()
                .and_then(|events| events.get(index + 1));
            if committed != journal.get(index + 1) {
                return Err(format!(
                    "RNG journal boundary differs from action replay at revision {}",
                    index + 1,
                ));
            }
            if !matches!(committed, Some(RngEvent::Committed { revision, state })
                if *revision == replay.revision && *state == replay.rng_state)
            {
                return Err("replayed RNG journal has an invalid commit boundary".into());
            }
            let draws = infer_draws(before, replay.rng_state).map_err(|error|
                format!("RNG attribution revision {}: {error}", index + 1))?;
            // Independently check each instrumented call against the RNG
            // transition, bound, returned value and committed journal state.
            if calls.len() > MAX_ATTRIBUTED_DRAWS_PER_ACTION as usize {
                return Err(format!("RNG attribution revision {}: call-site trace exceeds diagnostic budget", index + 1));
            }
            let mut cursor = before;
            let mut consumed = 0_u32;
            let mut previous_event_cursor = event_start;
            for call in &calls {
                if let (Some(start), Some(end), Some(position), Some(previous)) =
                    (event_start, event_end, call.event_cursor_before, previous_event_cursor)
                {
                    if position < start || position < previous || position > end {
                        return Err(format!("RNG/event interleaving cursor diverged at revision {}", index + 1));
                    }
                } else if call.event_cursor_before != event_start {
                    return Err(format!("RNG/event archive availability diverged at revision {}", index + 1));
                }
                // A parent is only valid when the RNG call happened inside
                // an observer dispatch whose event was emitted in this action.
                // In particular, a prior action's event or merely the latest
                // event in the archive cannot be claimed as a parent.
                if let Some(parent) = call.dispatch_parent {
                    let (Some(start), Some(position), Some(archive)) =
                        (event_start, call.event_cursor_before, replay.event_history.as_ref())
                    else {
                        return Err(format!("RNG dispatch parent has no event archive at revision {}", index + 1));
                    };
                    if !archive.get(start..position).is_some_and(|events|
                        events.iter().any(|event| event.encounter == parent.encounter
                            && event.event.seq == parent.seq))
                    {
                        return Err(format!("RNG dispatch parent is not a prior same-action event at revision {}", index + 1));
                    }
                }
                previous_event_cursor = call.event_cursor_before;
                if call.state_before != cursor || call.call_site.is_empty() {
                    return Err(format!("RNG call-site trace diverged at revision {}", index + 1));
                }
                let expected = if call.upper <= 1 {
                    0
                } else {
                    cursor = advance(cursor);
                    consumed += 1;
                    (cursor.wrapping_mul(0x2545F4914F6CDD1D) % call.upper as u64) as usize
                };
                if call.state_after != cursor || call.value != expected {
                    return Err(format!("RNG call-site result diverged at revision {}", index + 1));
                }
            }
            if consumed != draws || cursor != replay.rng_state {
                return Err(format!("RNG call-site count differs from committed boundary at revision {}", index + 1));
            }
            attributed.push(RngDrawAttribution {
                revision: replay.revision,
                action: action.clone(),
                state_before: before,
                state_after: replay.rng_state,
                draws,
                event_start,
                event_end,
                calls,
            });
        }
        if replay.to_json()? != self.to_json()? {
            return Err("RNG attribution replay differs from final complete Game".into());
        }
        Ok(attributed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xorshift_transitions_are_counted_without_advancing_for_no_draw() {
        let mut state = 123456789_u64;
        assert_eq!(infer_draws(state, state).unwrap(), 0);
        for expected in 1..=8 {
            state = advance(state);
            assert_eq!(infer_draws(123456789, state).unwrap(), expected);
        }
        assert!(infer_draws(0, state).is_err());
    }

    #[test]
    fn exceeding_diagnostic_budget_reports_unknown_not_a_false_count() {
        let mut state = 1_u64;
        for _ in 0..=MAX_ATTRIBUTED_DRAWS_PER_ACTION {
            state = advance(state);
        }
        assert!(infer_draws(1, state).unwrap_err().contains("attribution unavailable"));
    }
}
