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

/// Reconstruct HP, maximum HP and gold without Game, seed or action history.
/// A self-consistent forged journal is not authenticated by this reducer.
pub fn project(events: &[ResourceEvent]) -> Result<(Resources, u64), String> {
    let Some(ResourceEvent::Started { state }) = events.first() else {
        return Err("resource journal missing initial authority state".into());
    };
    if !state.valid() {
        return Err("invalid initial resource state".into());
    }
    let mut state = *state;
    let mut revision = 0u64;
    for event in events.iter().skip(1) {
        let ResourceEvent::Changed {
            revision: next, hp_delta, max_hp_delta, gold_delta,
        } = event else {
            return Err("duplicate resource journal start".into());
        };
        if revision.checked_add(1) != Some(*next) {
            return Err("gap or duplicate resource journal revision".into());
        }
        let candidate = Resources {
            hp: advance(state.hp, *hp_delta)?,
            max_hp: advance(state.max_hp, *max_hp_delta)?,
            gold: advance(state.gold, *gold_delta)?,
        };
        if !candidate.valid() {
            return Err("invalid resource state after event".into());
        }
        state = candidate;
        revision = *next;
    }
    Ok((state, revision))
}
