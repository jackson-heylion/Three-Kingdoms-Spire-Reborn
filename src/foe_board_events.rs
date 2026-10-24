//! Per-action enemy-board authority journal. This captures committed enemy
//! identity, health, status, phase and intent, not intermediate combat triggers.
//! It does not reconstruct the hero, card piles, listeners or RNG.
use serde::{Deserialize, Serialize};
use crate::{Foe, Game, MAX_FLOOR};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoeBoard {
    pub floor: u8,
    pub turn: u32,
    pub foes: Vec<Foe>,
}

impl FoeBoard {
    pub fn current(game: &Game) -> Option<Self> {
        game.battle.as_ref().map(|battle| Self {
            floor: game.floor,
            turn: battle.turn,
            foes: battle.foes.clone(),
        })
    }

    fn valid(&self) -> bool {
        (1..=MAX_FLOOR).contains(&self.floor)
            && self.turn > 0
            && (1..=4).contains(&self.foes.len())
            && self.foes.iter().all(|foe| {
                foe.max_hp > 0
                    && (0..=foe.max_hp).contains(&foe.hp)
                    && foe.block >= 0
                    && foe.poison >= 0
                    && foe.phase <= 1
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FoeBoardEvent {
    Started,
    Changed { revision: u64, change: FoeBoardChange },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FoeBoardChange {
    Idle,
    Enter { board: FoeBoard },
    Update { board: FoeBoard },
    Exit,
}

/// One record per successful action, including actions outside combat.
/// A board is a complete post-commit enemy state, not a delta or action replay.
pub fn transition(
    before: Option<FoeBoard>,
    after: Option<FoeBoard>,
    revision: u64,
) -> Result<FoeBoardEvent, String> {
    let change = match (before, after) {
        (None, None) => FoeBoardChange::Idle,
        (None, Some(board)) if board.valid() => FoeBoardChange::Enter { board },
        (Some(_), None) => FoeBoardChange::Exit,
        (Some(old), Some(board))
            if old.floor == board.floor && board.turn >= old.turn && board.valid() =>
        {
            FoeBoardChange::Update { board }
        }
        _ => return Err("invalid enemy board transition".into()),
    };
    Ok(FoeBoardEvent::Changed { revision, change })
}

/// A rejected event leaves the last committed board, floor and revision intact.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FoeBoardProjector {
    started: bool,
    board: Option<FoeBoard>,
    last_floor: Option<u8>,
    revision: u64,
}

impl FoeBoardProjector {
    pub fn push(&mut self, event: &FoeBoardEvent) -> Result<(), String> {
        match event {
            FoeBoardEvent::Started => {
                if self.started {
                    return Err("duplicate enemy board journal start".into());
                }
                self.started = true;
            }
            FoeBoardEvent::Changed { revision, change } => {
                if !self.started {
                    return Err("enemy board journal missing start".into());
                }
                if self.revision.checked_add(1) != Some(*revision) {
                    return Err("gap or duplicate enemy board revision".into());
                }
                let next = match change {
                    FoeBoardChange::Idle if self.board.is_none() => None,
                    FoeBoardChange::Enter { board }
                        if self.board.is_none()
                            && board.valid()
                            && self.last_floor.is_none_or(|floor| board.floor > floor) =>
                    {
                        Some(board.clone())
                    }
                    FoeBoardChange::Update { board }
                        if self.board.as_ref().is_some_and(|old| {
                            old.floor == board.floor && board.turn >= old.turn
                        }) && board.valid() =>
                    {
                        Some(board.clone())
                    }
                    FoeBoardChange::Exit if self.board.is_some() => None,
                    _ => return Err("invalid enemy board lifecycle".into()),
                };
                if let Some(board) = &next {
                    self.last_floor = Some(board.floor);
                }
                self.board = next;
                self.revision = *revision;
            }
        }
        Ok(())
    }

    pub fn state(&self) -> Result<(Option<FoeBoard>, u64), String> {
        if !self.started {
            return Err("enemy board journal missing start".into());
        }
        Ok((self.board.clone(), self.revision))
    }
}

pub fn project(events: &[FoeBoardEvent]) -> Result<(Option<FoeBoard>, u64), String> {
    let mut projector = FoeBoardProjector::default();
    for event in events {
        projector.push(event)?;
    }
    projector.state()
}

impl Game {
    /// Compare an independent event projection against the current enemy board.
    /// A legacy save with no journal cannot be certified by this method.
    pub fn audit_foe_board(&self) -> Result<usize, String> {
        let history = self.foe_board_history.as_ref()
            .ok_or("legacy save has no enemy board authority journal")?;
        let (board, revision) = project(history)?;
        if board != FoeBoard::current(self) || revision != self.revision {
            return Err("enemy board journal differs from current game state".into());
        }
        Ok(history.len())
    }
}
