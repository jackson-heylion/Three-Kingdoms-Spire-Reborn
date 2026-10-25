//! RNG-state snapshots at committed action boundaries.
//! This projects state, not draw counts, causal validity, or tamper authenticity.
#[path = "rng_draw_attribution.rs"]
mod draw_attribution;
pub use draw_attribution::{MAX_ATTRIBUTED_DRAWS_PER_ACTION, RngCall, RngDrawAttribution};

use crate::Game;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RngEvent {
    Started { state: u64 },
    Committed { revision: u64, state: u64 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RngProjector {
    state: Option<u64>,
    revision: u64,
}

impl RngProjector {
    /// A rejected event does not advance the committed state or revision.
    pub fn push(&mut self, event: &RngEvent) -> Result<(), String> {
        match event {
            RngEvent::Started { state } => {
                if self.state.is_some() {
                    return Err("duplicate RNG journal start".into());
                }
                if *state == 0 {
                    return Err("invalid initial RNG state".into());
                }
                self.state = Some(*state);
            }
            RngEvent::Committed { revision, state } => {
                if self.state.is_none() {
                    return Err("RNG journal missing initial state".into());
                }
                if self.revision.checked_add(1) != Some(*revision) {
                    return Err("gap or duplicate RNG journal revision".into());
                }
                if *state == 0 {
                    return Err("invalid committed RNG state".into());
                }
                self.state = Some(*state);
                self.revision = *revision;
            }
        }
        Ok(())
    }

    pub fn state(&self) -> Result<(u64, u64), String> {
        Ok((self.state.ok_or("RNG journal missing initial state")?, self.revision))
    }
}

pub fn project(events: &[RngEvent]) -> Result<(u64, u64), String> {
    let mut projector = RngProjector::default();
    for event in events {
        projector.push(event)?;
    }
    projector.state()
}

impl Game {
    /// Verify committed RNG state and revision, not individual random draws.
    pub fn audit_rng(&self) -> Result<usize, String> {
        let history = self.rng_history.as_ref()
            .ok_or("legacy save has no RNG authority journal")?;
        let (state, revision) = project(history)?;
        if state != self.rng_state || revision != self.revision {
            return Err("RNG journal differs from current game state".into());
        }
        Ok(history.len())
    }
}
