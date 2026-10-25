//! Read-only, precisely interleaved RNG calls and emitted combat events.
//!
//! The cursor means "events already appended before this RNG call", not the
//! parent of the random draw. Only replayed action history can produce this
//! timeline; it cannot reconstruct a Game from events or prove authenticity.
use crate::{
    Action, Game,
    event_stack::RunEvent,
    rng_events::RngCall,
};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", content = "record", rename_all = "snake_case")]
pub enum TimelineStep {
    RngCall(RngCall),
    CombatEvent(RunEvent),
}

#[derive(Clone, Debug, Serialize)]
pub struct ActionRngEventTimeline {
    pub revision: u64,
    pub action: Action,
    pub steps: Vec<TimelineStep>,
}

impl Game {
    /// Join independently audited action RNG calls to the exact order in which
    /// settled combat events were appended. No event archive => no claimed
    /// interleaving; legacy saves are explicitly unsupported.
    pub fn reconstruct_action_rng_event_timeline(
        &self,
    ) -> Result<Vec<ActionRngEventTimeline>, String> {
        let archive = self.event_history.as_ref()
            .ok_or("RNG/event timeline requires a complete combat event archive")?;
        let attribution = self.reconstruct_rng_draw_attribution()?;
        let mut next_event = 0_usize;
        let mut timeline = Vec::with_capacity(attribution.len());

        for action in attribution {
            let start = action.event_start
                .ok_or("RNG/event timeline missing replayed event archive")?;
            let end = action.event_end
                .ok_or("RNG/event timeline missing replayed event boundary")?;
            if start != next_event || end < start || end > archive.len() {
                return Err(format!("RNG/event timeline has an event gap at revision {}", action.revision));
            }
            let mut steps = Vec::with_capacity(action.calls.len() + end - start);
            for call in action.calls {
                let before = call.event_cursor_before
                    .ok_or("RNG/event timeline missing a call's event cursor")?;
                if before < next_event || before > end {
                    return Err(format!("RNG/event timeline call order differs at revision {}", action.revision));
                }
                for event in &archive[next_event..before] {
                    steps.push(TimelineStep::CombatEvent(event.clone()));
                }
                next_event = before;
                steps.push(TimelineStep::RngCall(call));
            }
            for event in &archive[next_event..end] {
                steps.push(TimelineStep::CombatEvent(event.clone()));
            }
            next_event = end;
            timeline.push(ActionRngEventTimeline {
                revision: action.revision,
                action: action.action,
                steps,
            });
        }
        if next_event != archive.len() {
            return Err("RNG/event timeline has unattributed combat events".into());
        }
        Ok(timeline)
    }
}
