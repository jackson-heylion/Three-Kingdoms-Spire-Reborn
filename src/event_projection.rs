//! Read-only projection from the complete settled combat-event archive.
//! No seed, action log, RNG or Game snapshot is consulted. These counters do
//! not reconstruct decks, HP, intent, listeners or any other authoritative state.
use super::{verify_record, RunEvent};
use crate::combat::EventKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignProjection {
    pub encounters: Vec<EncounterProjection>,
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
    pub cards_played: u64,
    pub foes_defeated: u64,
    pub foes_summoned: u64,
    pub enemy_techniques: u64,
    pub reactions_countered: u64,
    pub reactions_replaced: u64,
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
            cards_played: 0,
            foes_defeated: 0,
            foes_summoned: 0,
            enemy_techniques: 0,
            reactions_countered: 0,
            reactions_replaced: 0,
        }
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

/// Rebuild only explicitly journalled, settled combat facts from events.
/// The archive must start at encounter 1 / sequence 1 and contain no gaps.
/// Empty input yields an empty projection, not evidence of a complete campaign.
/// No comparison to a Game snapshot is made; a forged but well-formed archive
/// cannot be authenticated by this projection alone.
pub fn project(events: &[RunEvent]) -> Result<CampaignProjection, String> {
    verify_record(events)?;
    let mut output = CampaignProjection::default();
    for record in events {
        if output.encounters.last()
            .is_none_or(|last| last.encounter != record.encounter)
        {
            output.encounters.push(EncounterProjection::new(record));
        }
        let current = output.encounters.last_mut()
            .ok_or_else(|| "missing encounter projection".to_string())?;
        current.last_seq = record.event.seq;
        add(&mut current.events, 1)?;
        match record.event.kind {
            EventKind::DamageDealt =>
                add(&mut current.foe_hp_damage, settled_damage(record)?)?,
            EventKind::DamageReceived =>
                add(&mut current.hero_hp_damage, settled_damage(record)?)?,
            EventKind::CardPlayed => add(&mut current.cards_played, 1)?,
            EventKind::FoeDefeated => add(&mut current.foes_defeated, 1)?,
            EventKind::FoeSummoned => add(&mut current.foes_summoned, 1)?,
            EventKind::EnemyTechnique => add(&mut current.enemy_techniques, 1)?,
            EventKind::EventCountered => add(&mut current.reactions_countered, 1)?,
            EventKind::EventReplaced => add(&mut current.reactions_replaced, 1)?,
            _ => {}
        }
    }
    Ok(output)
}
