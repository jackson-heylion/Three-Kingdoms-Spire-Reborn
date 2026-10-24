//! Deterministic authority events for persistent campaign resources.
//! This is a deliberately narrow event-sourcing slice: no action replay, RNG,
//! snapshot or combat event is consulted when reducing these three fields.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resources {
    pub hp: i32,
    pub max_hp: i32,
    pub gold: i32,
}

impl Resources {
    fn valid(self) -> bool {
        self.max_hp > 0 && (0..=self.max_hp).contains(&self.hp) && self.gold >= 0
    }
}

/// Every committed Game::apply contributes exactly one Changed event, including
/// zero-delta actions. Revision continuity detects omitted action boundaries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResourceEvent {
    Started { state: Resources },
    Changed {
        revision: u64,
        hp_delta: i64,
        max_hp_delta: i64,
        gold_delta: i64,
    },
}

fn advance(value: i32, delta: i64) -> Result<i32, String> {
    i32::try_from(i64::from(value).checked_add(delta)
        .ok_or("resource event arithmetic overflow")?)
        .map_err(|_| "resource event outside i32 range".into())
}

/// Portable checkpoint of the committed resource projection. It is not a
/// cryptographic signature and does not authenticate omitted history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceCheckpoint {
    pub version: u32,
    pub state: Resources,
    pub revision: u64,
}

pub const RESOURCE_CHECKPOINT_VERSION: u32 = 1;

/// Incremental, atomic reducer for authoritative persistent resource events.
/// A failed push leaves both the committed state and revision unchanged.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceProjector {
    state: Option<Resources>,
    revision: u64,
}

impl ResourceProjector {
    pub fn push(&mut self, event: &ResourceEvent) -> Result<(), String> {
        match event {
            ResourceEvent::Started { state } => {
                if self.state.is_some() {
                    return Err("duplicate resource journal start".into());
                }
                if !state.valid() {
                    return Err("invalid initial resource state".into());
                }
                self.state = Some(*state);
            }
            ResourceEvent::Changed {
                revision: next, hp_delta, max_hp_delta, gold_delta,
            } => {
                let current = self.state.ok_or("resource journal missing initial authority state")?;
                if self.revision.checked_add(1) != Some(*next) {
                    return Err("gap or duplicate resource journal revision".into());
                }
                let candidate = Resources {
                    hp: advance(current.hp, *hp_delta)?,
                    max_hp: advance(current.max_hp, *max_hp_delta)?,
                    gold: advance(current.gold, *gold_delta)?,
                };
                if !candidate.valid() {
                    return Err("invalid resource state after event".into());
                }
                self.state = Some(candidate);
                self.revision = *next;
            }
        }
        Ok(())
    }

    pub fn state(&self) -> Result<(Resources, u64), String> {
        Ok((self.state.ok_or("resource journal missing initial authority state")?, self.revision))
    }

    pub fn checkpoint(&self) -> Result<ResourceCheckpoint, String> {
        let (state, revision) = self.state()?;
        Ok(ResourceCheckpoint { version: RESOURCE_CHECKPOINT_VERSION, state, revision })
    }

    /// Resume at the exact next revision. The caller must trust the checkpoint
    /// provenance; a valid checkpoint cannot prove the discarded prefix.
    pub fn resume(checkpoint: ResourceCheckpoint) -> Result<Self, String> {
        if checkpoint.version != RESOURCE_CHECKPOINT_VERSION {
            return Err("unsupported resource checkpoint version".into());
        }
        if !checkpoint.state.valid() {
            return Err("invalid resource checkpoint state".into());
        }
        Ok(Self { state: Some(checkpoint.state), revision: checkpoint.revision })
    }
}

/// Reconstruct HP, maximum HP and gold without Game, seed or action history.
/// Batch and streaming reduction share identical continuity and state rules.
pub fn project(events: &[ResourceEvent]) -> Result<(Resources, u64), String> {
    let mut projector = ResourceProjector::default();
    for event in events {
        projector.push(event)?;
    }
    projector.state()
}

impl crate::Game {
    /// Reconstruct persistent resources solely from the committed resource
    /// journal, then compare with the current snapshot. This does not rebuild
    /// battle energy, cards, enemies, RNG or any other Game state.
    pub fn audit_resources(&self) -> Result<usize, String> {
        let history = self.resource_history.as_ref()
            .ok_or("legacy save has no authoritative resource journal")?;
        let (state, revision) = project(history)?;
        if state != (Resources {
            hp: self.hp, max_hp: self.max_hp, gold: self.gold,
        }) || revision != self.revision {
            return Err("resource journal differs from current game state".into());
        }
        Ok(history.len())
    }
}
