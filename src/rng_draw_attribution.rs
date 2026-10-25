//! Offline RNG draw attribution from the xorshift64* state transition.
//!
//! This counts consumed RNG transitions per committed action without changing
//! the save schema or gameplay RNG. It does NOT identify the individual call
//! sites, bound arguments, or supply a trusted event-sourcing log.
use crate::{Action, Game, rng_events::RngEvent};

/// Diagnostic upper bound, not a gameplay restriction. A legitimate action
/// exceeding this budget is reported as *unattributed*, never rejected by apply.
pub const MAX_ATTRIBUTED_DRAWS_PER_ACTION: u32 = 8192;

#[derive(Clone, Debug)]
pub struct RngDrawAttribution {
    pub revision: u64,
    pub action: Action,
    pub state_before: u64,
    pub state_after: u64,
    pub draws: u32,
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
            replay.apply(action.clone()).map_err(|error|
                format!("RNG attribution replay action {} failed: {error}", index + 1))?;
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
            attributed.push(RngDrawAttribution {
                revision: replay.revision,
                action: action.clone(),
                state_before: before,
                state_after: replay.rng_state,
                draws,
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
