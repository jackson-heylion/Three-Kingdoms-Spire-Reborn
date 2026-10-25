//! Independent action-boundary projection of run-wide combat statistics.
//! This does not reconstruct individual combat events or authenticate the save.
use crate::{CombatCounters, CombatTelemetry, Game, MAX_FLOOR, MAX_RECENT_TURN_METRICS};
use serde::{Deserialize, Serialize};

fn valid(state: &CombatTelemetry) -> bool {
    state.recent_turns.len() <= MAX_RECENT_TURN_METRICS
        && state.recent_turns.iter().all(|turn| {
            (1..=MAX_FLOOR).contains(&turn.floor)
                && turn.turn > 0 && turn.hero_hp_after >= 0 && turn.troop_hp_after >= 0
        })
        && state.totals.battles_won.saturating_add(state.totals.battles_lost)
            <= state.totals.battles_started
}
fn monotonic(old: &CombatCounters, next: &CombatCounters) -> bool {
    next.battles_started >= old.battles_started
        && next.battles_won >= old.battles_won
        && next.battles_lost >= old.battles_lost
        && next.turns_completed >= old.turns_completed
        && next.cards_played >= old.cards_played
        && next.hero_hp_damage >= old.hero_hp_damage
        && next.troop_attacks >= old.troop_attacks
        && next.troop_damage >= old.troop_damage
        && next.troop_absorbed >= old.troop_absorbed
        && next.command_procs >= old.command_procs
        && next.command_energy >= old.command_energy
        && next.command_block >= old.command_block
        && next.sun_discards >= old.sun_discards
        && next.sun_discard_block >= old.sun_discard_block
        && next.sun_combo_procs >= old.sun_combo_procs
        && next.sun_combo_block >= old.sun_combo_block
        && next.sun_combo_draws >= old.sun_combo_draws
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TelemetryEvent {
    Started {
        state: CombatTelemetry,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        archive_len: Option<u64>,
    },
    Changed {
        revision: u64,
        state: Option<CombatTelemetry>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        archive_len: Option<u64>,
    },
}
impl TelemetryEvent {
    /// A cursor is absent in pre-upgrade V2 journals, never inferred.
    pub fn archive_len(&self) -> Option<u64> {
        match self {
            Self::Started { archive_len, .. } | Self::Changed { archive_len, .. } => *archive_len,
        }
    }
}
pub fn transition(before: &CombatTelemetry, after: &CombatTelemetry, revision: u64)
    -> Result<TelemetryEvent, String>
{
    transition_with_archive(before, after, revision, None)
}
/// Persist the settled event-archive prefix at the same atomic action boundary.
pub fn transition_with_archive(before: &CombatTelemetry, after: &CombatTelemetry,
    revision: u64, archive_len: Option<u64>) -> Result<TelemetryEvent, String>
{
    if !valid(before) || !valid(after) || !monotonic(&before.totals, &after.totals) {
        return Err("invalid combat telemetry transition".into());
    }
    Ok(TelemetryEvent::Changed {
        revision, state: (before != after).then(|| after.clone()), archive_len,
    })
}
#[derive(Clone, Debug, Default)]
pub struct TelemetryProjector {
    state: Option<CombatTelemetry>,
    revision: u64,
    archive_len: Option<u64>,
}
impl TelemetryProjector {
    pub fn archive_cursor(&self) -> Option<u64> { self.archive_len }
    pub fn push(&mut self, event: &TelemetryEvent) -> Result<(), String> {
        match event {
            TelemetryEvent::Started { state, archive_len } => {
                if self.state.is_some() || *state != CombatTelemetry::default()
                    || archive_len.is_some_and(|n| n != 0)
                {
                    return Err("invalid or duplicate telemetry journal start".into());
                }
                self.state = Some(state.clone());
                self.archive_len = *archive_len;
            }
            TelemetryEvent::Changed { revision, state, archive_len } => {
                let previous = self.state.as_ref().ok_or("telemetry journal missing start")?;
                if self.revision.checked_add(1) != Some(*revision) {
                    return Err("gap or duplicate telemetry journal revision".into());
                }
                if self.archive_len.is_some() != archive_len.is_some()
                    || self.archive_len.zip(*archive_len).is_some_and(|(old, new)| new < old)
                {
                    return Err("telemetry archive cursor missing or regressed".into());
                }
                if let Some(next) = state {
                    if !valid(next) || !monotonic(&previous.totals, &next.totals)
                        || previous == next
                    {
                        return Err("invalid telemetry journal change".into());
                    }
                    self.state = Some(next.clone());
                }
                self.archive_len = *archive_len;
                self.revision = *revision;
            }
        }
        Ok(())
    }
    pub fn state(&self) -> Result<(CombatTelemetry, u64), String> {
        Ok((self.state.clone().ok_or("telemetry journal missing start")?, self.revision))
    }
}
pub fn project(events: &[TelemetryEvent]) -> Result<(CombatTelemetry, u64), String> {
    let mut projector = TelemetryProjector::default();
    for event in events { projector.push(event)?; }
    projector.state()
}
impl Game {
    pub fn audit_telemetry_authority(&self) -> Result<usize, String> {
        let history = self.telemetry_history.as_ref()
            .ok_or("legacy save has no telemetry authority journal")?;
        let (state, revision) = project(history)?;
        if state != self.telemetry || revision != self.revision {
            return Err("telemetry authority differs from Game snapshot".into());
        }
        Ok(history.len())
    }
}
