//! Committed combat power and dynamic-listener authority at action boundaries.
//! This restores ordered power stacks and registered reaction rules without
//! action replay. It does not replay intermediate triggers or reconstruct RNG.
use crate::{Game, MAX_FLOOR, combat::PowerStack, event_stack::Stack};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleEffects {
    pub floor: u8,
    pub turn: u32,
    pub powers: Vec<PowerStack>,
    pub reactions: Stack,
}

impl BattleEffects {
    pub fn current(game: &Game) -> Option<Self> {
        game.battle.as_ref().map(|battle| Self {
            floor: game.floor,
            turn: battle.turn,
            powers: battle.powers.clone(),
            reactions: battle.reactions.clone(),
        })
    }

    fn valid(&self) -> bool {
        (1..=MAX_FLOOR).contains(&self.floor)
            && self.turn > 0
            && self.powers.len() <= 8
            && self.powers.iter().enumerate().all(|(index, power)| {
                (1..=99).contains(&power.stacks)
                    && !self.powers[..index].iter().any(|old| old.id == power.id)
            })
            && self.reactions.valid()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CombatEffectEvent {
    Started,
    Changed { revision: u64, change: CombatEffectChange },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CombatEffectChange {
    Idle,
    Enter { state: BattleEffects },
    Update { state: BattleEffects },
    Unchanged,
    Exit,
}

/// Exactly one event for every successful action, including noncombat actions.
/// An unchanged combat effect state still consumes the action revision.
pub fn transition(
    before: Option<BattleEffects>,
    after: Option<BattleEffects>,
    revision: u64,
) -> Result<CombatEffectEvent, String> {
    let change = match (before, after) {
        (None, None) => CombatEffectChange::Idle,
        (None, Some(state)) if state.valid() => CombatEffectChange::Enter { state },
        (Some(_), None) => CombatEffectChange::Exit,
        (Some(old), Some(state))
            if old.floor == state.floor && state.turn >= old.turn && state.valid() =>
        {
            if old == state {
                CombatEffectChange::Unchanged
            } else {
                CombatEffectChange::Update { state }
            }
        }
        _ => return Err("invalid combat effect transition".into()),
    };
    Ok(CombatEffectEvent::Changed { revision, change })
}

/// Rejected records never advance the revision or change the committed state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CombatEffectProjector {
    started: bool,
    state: Option<BattleEffects>,
    last_floor: Option<u8>,
    revision: u64,
}

impl CombatEffectProjector {
    pub fn push(&mut self, event: &CombatEffectEvent) -> Result<(), String> {
        match event {
            CombatEffectEvent::Started => {
                if self.started {
                    return Err("duplicate combat effect journal start".into());
                }
                self.started = true;
            }
            CombatEffectEvent::Changed { revision, change } => {
                if !self.started {
                    return Err("combat effect journal missing start".into());
                }
                if self.revision.checked_add(1) != Some(*revision) {
                    return Err("gap or duplicate combat effect revision".into());
                }
                let next = match change {
                    CombatEffectChange::Idle if self.state.is_none() => None,
                    CombatEffectChange::Enter { state }
                        if self.state.is_none()
                            && state.valid()
                            && self.last_floor.is_none_or(|floor| state.floor > floor) =>
                    {
                        Some(state.clone())
                    }
                    CombatEffectChange::Update { state }
                        if self.state.as_ref().is_some_and(|old| {
                            old.floor == state.floor && state.turn >= old.turn
                        }) && state.valid() =>
                    {
                        Some(state.clone())
                    }
                    CombatEffectChange::Unchanged if self.state.is_some() => self.state.clone(),
                    CombatEffectChange::Exit if self.state.is_some() => None,
                    _ => return Err("invalid combat effect lifecycle".into()),
                };
                if let Some(state) = &next {
                    self.last_floor = Some(state.floor);
                }
                self.state = next;
                self.revision = *revision;
            }
        }
        Ok(())
    }

    pub fn state(&self) -> Result<(Option<BattleEffects>, u64), String> {
        if !self.started {
            return Err("combat effect journal missing start".into());
        }
        Ok((self.state.clone(), self.revision))
    }
}

pub fn project(events: &[CombatEffectEvent]) -> Result<(Option<BattleEffects>, u64), String> {
    let mut projector = CombatEffectProjector::default();
    for event in events {
        projector.push(event)?;
    }
    projector.state()
}

impl Game {
    /// Old V2 saves without this journal remain playable, but are not certified.
    pub fn audit_combat_effects(&self) -> Result<usize, String> {
        let history = self.combat_effect_history.as_ref()
            .ok_or("legacy save has no combat effect authority journal")?;
        let (state, revision) = project(history)?;
        if state != BattleEffects::current(self) || revision != self.revision {
            return Err("combat effect journal differs from current game state".into());
        }
        Ok(history.len())
    }
}
