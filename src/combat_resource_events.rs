//! Action-boundary authority journal for active combat energy, block and turn.
//! Unlike the bounded combat event ring, this journal retains every successful
//! action boundary. It is deliberately not a full battle or RNG reconstruction.
use serde::{Deserialize, Serialize};
use crate::{Game, MAX_FLOOR};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleResources {
    pub floor: u8,
    pub turn: u32,
    pub energy: i32,
    pub block: i32,
}

impl BattleResources {
    pub fn current(game: &Game) -> Option<Self> {
        game.battle.as_ref().map(|battle| Self {
            floor: game.floor,
            turn: battle.turn,
            energy: battle.energy,
            block: battle.block,
        })
    }

    fn valid(self) -> bool {
        (1..=MAX_FLOOR).contains(&self.floor)
            && self.turn > 0 && self.energy >= 0 && self.block >= 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BattleResourceEvent {
    Started,
    Changed { revision: u64, change: BattleResourceChange },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BattleResourceChange {
    Idle,
    Enter { state: BattleResources },
    Update { energy_delta: i64, block_delta: i64, turns_elapsed: u64 },
    Exit,
}

/// Capture the committed transition, not an intermediate damage or card event.
/// One Changed event is written for every successful Game::apply, including
/// out-of-combat actions. Revision continuity exposes missing action boundaries.
pub fn transition(
    before: Option<BattleResources>,
    after: Option<BattleResources>,
    revision: u64,
) -> Result<BattleResourceEvent, String> {
    let change = match (before, after) {
        (None, None) => BattleResourceChange::Idle,
        (None, Some(state)) if state.valid() => BattleResourceChange::Enter { state },
        (Some(_), None) => BattleResourceChange::Exit,
        (Some(old), Some(new)) if old.floor == new.floor && new.turn >= old.turn && new.valid() => {
            BattleResourceChange::Update {
                energy_delta: i64::from(new.energy) - i64::from(old.energy),
                block_delta: i64::from(new.block) - i64::from(old.block),
                turns_elapsed: u64::from(new.turn - old.turn),
            }
        }
        _ => return Err("invalid combat resource transition".into()),
    };
    Ok(BattleResourceEvent::Changed { revision, change })
}

fn advance(value: i32, delta: i64) -> Result<i32, String> {
    i32::try_from(i64::from(value).checked_add(delta)
        .ok_or("combat resource arithmetic overflow")?)
        .map_err(|_| "combat resource outside i32 range".into())
}

/// Atomic incremental projection. Rejected events never alter the committed
/// revision or state; neither the action log nor Game snapshot is consulted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BattleResourceProjector {
    started: bool,
    state: Option<BattleResources>,
    revision: u64,
}

impl BattleResourceProjector {
    pub fn push(&mut self, event: &BattleResourceEvent) -> Result<(), String> {
        match event {
            BattleResourceEvent::Started => {
                if self.started {
                    return Err("duplicate combat resource journal start".into());
                }
                self.started = true;
            }
            BattleResourceEvent::Changed { revision, change } => {
                if !self.started {
                    return Err("combat resource journal missing start".into());
                }
                if self.revision.checked_add(1) != Some(*revision) {
                    return Err("gap or duplicate combat resource revision".into());
                }
                let next = match change {
                    BattleResourceChange::Idle if self.state.is_none() => None,
                    BattleResourceChange::Enter { state }
                        if self.state.is_none() && state.valid() => Some(*state),
                    BattleResourceChange::Exit if self.state.is_some() => None,
                    BattleResourceChange::Update { energy_delta, block_delta, turns_elapsed } => {
                        let old = self.state.ok_or("combat resource update without active battle")?;
                        let turn = u32::try_from(u64::from(old.turn)
                            .checked_add(*turns_elapsed)
                            .ok_or("combat turn overflow")?)
                            .map_err(|_| "combat turn outside u32 range")?;
                        let candidate = BattleResources {
                            floor: old.floor,
                            turn,
                            energy: advance(old.energy, *energy_delta)?,
                            block: advance(old.block, *block_delta)?,
                        };
                        if !candidate.valid() {
                            return Err("invalid combat resource state after event".into());
                        }
                        Some(candidate)
                    }
                    _ => return Err("invalid combat resource lifecycle".into()),
                };
                self.state = next;
                self.revision = *revision;
            }
        }
        Ok(())
    }

    pub fn state(&self) -> Result<(Option<BattleResources>, u64), String> {
        if !self.started {
            return Err("combat resource journal missing start".into());
        }
        Ok((self.state, self.revision))
    }
}

pub fn project(events: &[BattleResourceEvent]) -> Result<(Option<BattleResources>, u64), String> {
    let mut projector = BattleResourceProjector::default();
    for event in events {
        projector.push(event)?;
    }
    projector.state()
}

impl Game {
    /// Explicit read-only audit. Legacy V2 snapshots remain playable but have
    /// no authority evidence to reconstruct; this does not authenticate edits
    /// to both journal and snapshot or reconstruct card piles, foes or RNG.
    pub fn audit_battle_resources(&self) -> Result<usize, String> {
        let history = self.battle_resource_history.as_ref()
            .ok_or("legacy save has no combat resource authority journal")?;
        let (state, revision) = project(history)?;
        if state != BattleResources::current(self) || revision != self.revision {
            return Err("combat resource journal differs from current game state".into());
        }
        Ok(history.len())
    }
}
