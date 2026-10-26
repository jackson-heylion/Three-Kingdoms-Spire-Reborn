//! Read-only projection from the complete settled combat-event archive.
//! No seed, action log, RNG or Game snapshot is consulted. These counters do
//! not reconstruct decks, HP, intent, listeners or any other authoritative state.
use super::{verify_record_coordinate, RunEvent};

/// Semantic validation of the complete, settled combat archive.
pub mod causality;
use crate::combat::EventKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignProjection {
    pub encounters: Vec<EncounterProjection>,
}

/// The terminal result is derived solely from BattleEnded, not from Game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleOutcome { Victory, Defeat }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutcomeTotals {
    pub victories: u64,
    pub defeats: u64,
    pub active_encounters: u64,
    pub cards_played: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterProjection {
    pub encounter: u64,
    pub floor: u8,
    pub first_seq: u64,
    pub last_seq: u64,
    pub events: u64,
    /// Settled damage to foes after their armor; not raw attempted damage.
    pub foe_hp_damage: u64,
    /// Settled damage to hero HP after block and troop interception.
    pub hero_hp_damage: u64,
    /// Only HP loss from actual foe attacks; excludes blood price and attrition.
    /// None means a legacy checkpoint did not track this split.
    #[serde(default)]
    pub foe_attack_hp_damage: Option<u64>,
    pub cards_played: u64,
    pub foes_defeated: u64,
    pub foes_summoned: u64,
    pub enemy_techniques: u64,
    pub reactions_countered: u64,
    pub reactions_replaced: u64,
    /// True only when a terminal BattleEnded record was observed.
    #[serde(default)]
    pub closed: bool,
    /// Old v1 checkpoints have no winner; do not infer one from closed=true.
    #[serde(default)]
    pub outcome: Option<BattleOutcome>,
}

impl EncounterProjection {
    fn new(record: &RunEvent) -> Self {
        Self {
            encounter: record.encounter,
            floor: record.floor,
            first_seq: record.event.seq,
            last_seq: record.event.seq,
            events: 0,
            foe_hp_damage: 0,
            hero_hp_damage: 0,
            foe_attack_hp_damage: Some(0),
            cards_played: 0,
            foes_defeated: 0,
            foes_summoned: 0,
            enemy_techniques: 0,
            reactions_countered: 0,
            reactions_replaced: 0,
            closed: false,
            outcome: None,
        }
    }
}

impl CampaignProjection {
    /// Reconstruct recorded terminal results without consulting Game or actions.
    /// An old checkpoint with closed=true but no outcome is not sufficient.
    pub fn outcome_totals(&self) -> Result<OutcomeTotals, String> {
        let mut totals = OutcomeTotals::default();
        for (index, encounter) in self.encounters.iter().enumerate() {
            add(&mut totals.cards_played, encounter.cards_played)?;
            match (encounter.closed, encounter.outcome) {
                (true, Some(BattleOutcome::Victory)) => add(&mut totals.victories, 1)?,
                (true, Some(BattleOutcome::Defeat)) => add(&mut totals.defeats, 1)?,
                (false, None) if index + 1 == self.encounters.len() => {
                    totals.active_encounters = 1;
                },
                (true, None) => return Err("sealed legacy checkpoint lacks battle outcome".into()),
                _ => return Err("invalid event projection outcome sequence".into()),
            }
        }
        Ok(totals)
    }

    /// Compare event-derived facts with lifetime telemetry. This is a read-only
    /// consistency check, not a signature or proof that either source is genuine.
    pub fn reconcile_telemetry(&self, counters: &crate::CombatCounters,
                               active_battle: bool) -> Result<(), String> {
        let totals = self.outcome_totals()?;
        // CombatCounters.hero_hp_damage tracks enemy attacks only. Other
        // DamageReceived events (blood price, supply exhaustion) still count
        // toward the broader event-derived hero_hp_damage statistic.
        let foe_damage = self.encounters.iter().try_fold(0u64, |total, encounter| {
            let damage = encounter.foe_attack_hp_damage.ok_or_else(||
                "legacy checkpoint lacks foe attack damage provenance".to_string())?;
            total.checked_add(damage).ok_or_else(||
                "event projection counter overflow".to_string())
        })?;
        if counters.battles_started != self.encounters.len() as u64
            || counters.battles_won != totals.victories
            || counters.battles_lost != totals.defeats
            || counters.cards_played != totals.cards_played
            || counters.hero_hp_damage != foe_damage
            || active_battle != (totals.active_encounters == 1) {
            return Err("event outcomes disagree with lifetime combat telemetry".into());
        }
        Ok(())
    }
}

fn add(counter: &mut u64, value: u64) -> Result<(), String> {
    *counter = counter.checked_add(value)
        .ok_or_else(|| "event projection counter overflow".to_string())?;
    Ok(())
}

fn settled_damage(record: &RunEvent) -> Result<u64, String> {
    u64::try_from(record.event.amount).map_err(|_| {
        format!("negative settled damage in encounter {} event {}",
            record.encounter, record.event.seq)
    })
}

/// Versioned progress checkpoint for resuming event-only audit projections.
/// It contains no authoritative game state or proof that the events are genuine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionCheckpoint {
    pub schema_version: u8,
    pub projection: CampaignProjection,
}

/// Incremental, read-only reducer for complete, ordered event archives.
/// The first event must be encounter 1 / sequence 1. A rejected event never
/// advances the cursor or changes the projection; callers may retry it.
/// The reducer retains only encounter summaries, not raw event payloads.
#[derive(Clone, Debug, Default)]
pub struct CampaignProjector {
    projection: CampaignProjection,
    last: Option<(u64, u8, u64)>,
}

impl CampaignProjector {
    pub fn new() -> Self { Self::default() }
    /// Export only the committed prefix. A checkpoint is not a substitute
    /// for the original journal or an authenticated Game save.
    pub fn checkpoint(&self) -> ProjectionCheckpoint {
        ProjectionCheckpoint {
            schema_version: 1,
            projection: self.projection.clone(),
        }
    }

    /// Resume at the exact next event coordinate after a serialized checkpoint.
    /// Reject structurally impossible or version-incompatible checkpoints.
    /// This validates shape, not event provenance or the reported totals.
    pub fn resume(checkpoint: ProjectionCheckpoint) -> Result<Self, String> {
        if checkpoint.schema_version != 1 {
            return Err("unsupported event projection checkpoint version".into());
        }
        let mut prior_floor = 0;
        for (index, encounter) in checkpoint.projection.encounters.iter().enumerate() {
            if encounter.encounter != index as u64 + 1
                || encounter.floor == 0 || encounter.floor > crate::MAX_FLOOR
                || encounter.floor < prior_floor
                || encounter.first_seq != 1 || encounter.last_seq == 0
                || encounter.events != encounter.last_seq
            {
                return Err("invalid event projection checkpoint coordinates".into());
            }
            // These categories represent disjoint EventKind variants. Their
            // combined occurrence count cannot exceed all archived events.
            let counted = [
                encounter.cards_played, encounter.foes_defeated,
                encounter.foes_summoned, encounter.enemy_techniques,
                encounter.reactions_countered, encounter.reactions_replaced,
            ].into_iter().try_fold(0u64, |total, count| total.checked_add(count))
                .ok_or_else(|| "event projection checkpoint counter overflow".to_string())?;
            if counted > encounter.events {
                return Err("invalid event projection checkpoint counters".into());
            }
            if encounter.outcome.is_some() && !encounter.closed {
                return Err("checkpoint outcome without battle end seal".into());
            }
            if encounter.foe_attack_hp_damage.is_some_and(|n| n > encounter.hero_hp_damage) {
                return Err("checkpoint foe attack damage exceeds total hero damage".into());
            }
            prior_floor = encounter.floor;
        }
        let last = checkpoint.projection.encounters.last()
            .map(|e| (e.encounter, e.floor, e.last_seq));
        Ok(Self { projection: checkpoint.projection, last })
    }


    pub fn projection(&self) -> &CampaignProjection { &self.projection }

    pub fn push(&mut self, record: &RunEvent) -> Result<(), String> {
        verify_record_coordinate(self.last, record)?;
        if self.projection.encounters.last().is_some_and(|last|
            last.encounter == record.encounter && last.closed) {
            return Err("events after encounter end seal".into());
        }
        let new_encounter = self.last.is_none_or(|(encounter, _, _)|
            encounter != record.encounter);
        // Stage the entire encounter update before committing. Even a failed
        // damage conversion or checked addition leaves this reducer unchanged.
        let mut current = if new_encounter {
            EncounterProjection::new(record)
        } else {
            self.projection.encounters.last()
                .ok_or_else(|| "missing encounter projection".to_string())?
                .clone()
        };
        current.last_seq = record.event.seq;
        add(&mut current.events, 1)?;
        match record.event.kind {
            EventKind::DamageDealt =>
                add(&mut current.foe_hp_damage, settled_damage(record)?)?,
            EventKind::DamageReceived => {
                let damage = settled_damage(record)?;
                add(&mut current.hero_hp_damage, damage)?;
                // Only the four canonical foe slots represent attacks counted
                // by CombatCounters.hero_hp_damage. Never infer this from a
                // generic DamageReceived event or an untrusted prefix alone.
                if matches!(record.event.source.as_str(),
                    "foe:0" | "foe:1" | "foe:2" | "foe:3")
                    && let Some(counter) = &mut current.foe_attack_hp_damage {
                        add(counter, damage)?;
                    }
            },
            EventKind::CardPlayed => add(&mut current.cards_played, 1)?,
            EventKind::FoeDefeated => add(&mut current.foes_defeated, 1)?,
            EventKind::FoeSummoned => add(&mut current.foes_summoned, 1)?,
            EventKind::EnemyTechnique => add(&mut current.enemy_techniques, 1)?,
            EventKind::EventCountered => add(&mut current.reactions_countered, 1)?,
            EventKind::EventReplaced => add(&mut current.reactions_replaced, 1)?,
            EventKind::BattleEnded => {
                let valid = record.event.target.is_none() && matches!(
                    (record.event.source.as_str(), record.event.amount),
                    ("victory", 1) | ("defeat", 0));
                if !valid { return Err("invalid encounter end seal".into()); }
                current.closed = true;
                current.outcome = Some(if record.event.source == "victory" {
                    BattleOutcome::Victory
                } else {
                    BattleOutcome::Defeat
                });
            },
            _ => {}
        }
        if new_encounter {
            self.projection.encounters.push(current);
        } else if let Some(last) = self.projection.encounters.last_mut() {
            *last = current;
        }
        self.last = Some((record.encounter, record.floor, record.event.seq));
        Ok(())
    }

    pub fn finish(self) -> CampaignProjection { self.projection }
}

/// Rebuild only explicitly journalled, settled combat facts from events.
/// The archive must start at encounter 1 / sequence 1 and contain no gaps.
/// Empty input yields an empty projection, not evidence of a complete campaign.
/// No comparison to a Game snapshot is made; a forged but well-formed archive
/// cannot be authenticated by this projection alone.
pub fn project(events: &[RunEvent]) -> Result<CampaignProjection, String> {
    let mut projector = CampaignProjector::new();
    for record in events {
        projector.push(record)?;
    }
    Ok(projector.finish())
}


// Strict, restartable journal consumer. Unlike the legacy projector, it
// refuses to cross a battle boundary without a terminal BattleEnded event.
/// A checkpoint may end mid-battle; all earlier encounters must be sealed.
#[derive(Clone, Debug, Default)]
pub struct SealedCampaignProjector {
    inner: CampaignProjector,
}

impl SealedCampaignProjector {
    pub fn new() -> Self { Self::default() }

    /// Resume a strict stream. Legacy checkpoints containing unfinished
    /// earlier encounters cannot be promoted into strict audit evidence.
    pub fn resume(checkpoint: ProjectionCheckpoint) -> Result<Self, String> {
        let inner = CampaignProjector::resume(checkpoint)?;
        let encounters = &inner.projection.encounters;
        if encounters.iter().any(|encounter| encounter.closed && encounter.outcome.is_none())
            || encounters[..encounters.len().saturating_sub(1)]
                .iter().any(|encounter| !encounter.closed) {
            return Err("sealed checkpoint lacks a terminal outcome or earlier seal".into());
        }
        Ok(Self { inner })
    }

    pub fn projection(&self) -> &CampaignProjection { self.inner.projection() }
    pub fn checkpoint(&self) -> ProjectionCheckpoint { self.inner.checkpoint() }

    /// Reject a missing end seal as soon as the next encounter arrives.
    /// A rejected event does not change the cursor or any accumulated totals.
    pub fn push(&mut self, record: &RunEvent) -> Result<(), String> {
        if self.inner.projection.encounters.last().is_some_and(|last|
            last.encounter != record.encounter && !last.closed) {
            return Err("unsealed encounter before next encounter".into());
        }
        self.inner.push(record)
    }

    /// A strict stream is complete only when nonempty and fully sealed.
    pub fn finish(self) -> Result<CampaignProjection, String> {
        let projection = self.inner.finish();
        if projection.encounters.is_empty()
            || projection.encounters.iter().any(|encounter|
                !encounter.closed || encounter.outcome.is_none()) {
            return Err("event archive contains an unsealed or outcome-less encounter".into());
        }
        Ok(projection)
    }
}

/// Require an explicit terminal event for every encounter. Unlike the legacy
/// `project` entry point, this rejects missing tails, including a truncated
/// last encounter. It does not authenticate a forged, self-consistent journal.
/// Old V2 archives without BattleEnded remain supported by `project` only.
pub fn project_sealed(events: &[RunEvent]) -> Result<CampaignProjection, String> {
    let mut projector = SealedCampaignProjector::new();
    for record in events {
        projector.push(record)?;
    }
    projector.finish()
}
