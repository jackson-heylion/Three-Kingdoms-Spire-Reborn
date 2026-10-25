//! Action-boundary authority for every active Battle field not already held
//! by resources, foe board, card zones, or combat effects. This captures the
//! bounded event window and combat scalars without replaying actions.
use crate::{Game, MAX_FLOOR, combat::{CombatEvent, MAX_RECENT_EVENTS}};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleRuntime {
    pub floor: u8,
    pub turn: u32,
    pub strength: i32,
    pub weak: u8,
    pub vulnerable: u8,
    pub momentum: i32,
    pub scheme: i32,
    pub combo: i32,
    pub command: i32,
    pub fury: i32,
    pub thorns: i32,
    pub played: u32,
    pub played_this_turn: u8,
    pub last_turn_cards: u8,
    pub next_turn_energy_penalty: u8,
    pub troops_hp: i32,
    pub troops_attack: i32,
    pub discard_tokens: i32,
    pub thought_used: bool,
    pub event_seq: u64,
    pub events: Vec<CombatEvent>,
}

impl BattleRuntime {
    pub fn current(game: &Game) -> Option<Self> {
        game.battle.as_ref().map(|b| Self {
            floor: game.floor, turn: b.turn, strength: b.strength,
            weak: b.weak, vulnerable: b.vulnerable,
            momentum: b.momentum, scheme: b.scheme, combo: b.combo,
            command: b.command, fury: b.fury, thorns: b.thorns,
            played: b.played, played_this_turn: b.played_this_turn,
            last_turn_cards: b.last_turn_cards,
            next_turn_energy_penalty: b.next_turn_energy_penalty,
            troops_hp: b.troops_hp, troops_attack: b.troops_attack,
            discard_tokens: b.discard_tokens, thought_used: b.thought_used,
            event_seq: b.event_seq, events: b.events.clone(),
        })
    }

    fn valid(&self) -> bool {
        (1..=MAX_FLOOR).contains(&self.floor) && self.turn > 0
            && self.next_turn_energy_penalty <= 2
            && [self.momentum, self.scheme, self.combo, self.command,
                self.fury, self.thorns, self.troops_hp, self.troops_attack,
                self.discard_tokens].iter().all(|value| *value >= 0)
            && self.events.len() <= MAX_RECENT_EVENTS
            && self.events.iter().all(|event| event.seq > 0 && event.seq <= self.event_seq)
            && self.events.windows(2).all(|pair|
                pair[0].seq.checked_add(1) == Some(pair[1].seq))
            && self.events.last().is_none_or(|event| event.seq == self.event_seq)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BattleRuntimeEvent {
    Started,
    Changed { revision: u64, change: BattleRuntimeChange },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BattleRuntimeChange {
    Idle,
    Enter { state: BattleRuntime },
    Update { state: BattleRuntime },
    Unchanged,
    Exit,
}

pub fn transition(
    before: Option<BattleRuntime>, after: Option<BattleRuntime>, revision: u64,
) -> Result<BattleRuntimeEvent, String> {
    let change = match (before, after) {
        (None, None) => BattleRuntimeChange::Idle,
        (None, Some(state)) if state.valid() => BattleRuntimeChange::Enter { state },
        (Some(_), None) => BattleRuntimeChange::Exit,
        (Some(old), Some(state))
            if old.floor == state.floor && state.turn >= old.turn && state.valid() =>
        {
            if old == state { BattleRuntimeChange::Unchanged }
            else { BattleRuntimeChange::Update { state } }
        }
        _ => return Err("invalid battle runtime transition".into()),
    };
    Ok(BattleRuntimeEvent::Changed { revision, change })
}

#[derive(Clone, Debug, Default)]
pub struct BattleRuntimeProjector {
    started: bool,
    state: Option<BattleRuntime>,
    last_floor: Option<u8>,
    revision: u64,
}

impl BattleRuntimeProjector {
    pub fn push(&mut self, event: &BattleRuntimeEvent) -> Result<(), String> {
        match event {
            BattleRuntimeEvent::Started => {
                if self.started { return Err("duplicate battle runtime start".into()); }
                self.started = true;
            }
            BattleRuntimeEvent::Changed { revision, change } => {
                if !self.started { return Err("battle runtime missing start".into()); }
                if self.revision.checked_add(1) != Some(*revision) {
                    return Err("battle runtime revision gap".into());
                }
                let next = match change {
                    BattleRuntimeChange::Idle if self.state.is_none() => None,
                    BattleRuntimeChange::Enter { state }
                        if self.state.is_none() && state.valid()
                            && self.last_floor.is_none_or(|floor| state.floor > floor) =>
                    {
                        Some(state.clone())
                    }
                    BattleRuntimeChange::Update { state }
                        if self.state.as_ref().is_some_and(|old|
                            old.floor == state.floor && state.turn >= old.turn)
                            && state.valid() =>
                    {
                        Some(state.clone())
                    }
                    BattleRuntimeChange::Unchanged if self.state.is_some() =>
                        self.state.clone(),
                    BattleRuntimeChange::Exit if self.state.is_some() => None,
                    _ => return Err("invalid battle runtime lifecycle".into()),
                };
                if let Some(state) = &next { self.last_floor = Some(state.floor); }
                self.state = next;
                self.revision = *revision;
            }
        }
        Ok(())
    }

    pub fn state(&self) -> Result<(Option<BattleRuntime>, u64), String> {
        if !self.started { return Err("battle runtime missing start".into()); }
        Ok((self.state.clone(), self.revision))
    }
}

pub fn project(events: &[BattleRuntimeEvent]) -> Result<(Option<BattleRuntime>, u64), String> {
    let mut projector = BattleRuntimeProjector::default();
    for event in events { projector.push(event)?; }
    projector.state()
}

impl Game {
    pub fn audit_battle_runtime_authority(&self) -> Result<usize, String> {
        let history = self.battle_runtime_history.as_ref()
            .ok_or("legacy save has no battle runtime journal")?;
        let (state, revision) = project(history)?;
        if state != BattleRuntime::current(self) || revision != self.revision {
            return Err("battle runtime journal differs from Game snapshot".into());
        }
        Ok(history.len())
    }
}
