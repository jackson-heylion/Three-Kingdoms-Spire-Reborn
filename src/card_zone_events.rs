//! Committed card-zone authority at successful action boundaries.
//! This restores permanent deck and ordered battle piles without action replay.
//! It does not restore RNG, powers, listeners, or intermediate card triggers.
use crate::{Card, Game, MAX_FLOOR};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleCardZones {
    pub floor: u8,
    pub turn: u32,
    pub draw: Vec<Card>,
    pub hand: Vec<Card>,
    pub discard: Vec<Card>,
    pub exhausted: Vec<Card>,
}

impl BattleCardZones {
    fn valid(&self) -> bool {
        (1..=MAX_FLOOR).contains(&self.floor) && self.turn > 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardZones {
    pub deck: Vec<Card>,
    pub battle: Option<BattleCardZones>,
}

impl CardZones {
    pub fn current(game: &Game) -> Self {
        Self {
            deck: game.deck.clone(),
            battle: game.battle.as_ref().map(|battle| BattleCardZones {
                floor: game.floor,
                turn: battle.turn,
                draw: battle.draw.clone(),
                hand: battle.hand.clone(),
                discard: battle.discard.clone(),
                exhausted: battle.exhausted.clone(),
            }),
        }
    }

    fn valid(&self) -> bool {
        self.deck.iter().all(|card| !card.temporary)
            && self.battle.as_ref().is_none_or(BattleCardZones::valid)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CardZoneEvent {
    Started { deck: Vec<Card> },
    Changed {
        revision: u64,
        /// None means the permanent deck was unchanged by this action.
        permanent: Option<Vec<Card>>,
        battle: BattleCardsChange,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BattleCardsChange {
    Idle,
    Enter { zones: BattleCardZones },
    Update { zones: BattleCardZones },
    Unchanged,
    Exit,
}

/// Store only permanent deck changes, but retain ordered battle piles after
/// each action that changes them. Every successful action still gets a revision.
pub fn transition(
    before: &CardZones,
    after: &CardZones,
    revision: u64,
) -> Result<CardZoneEvent, String> {
    if !before.valid() || !after.valid() {
        return Err("invalid card zone state".into());
    }
    let battle = match (&before.battle, &after.battle) {
        (None, None) => BattleCardsChange::Idle,
        (None, Some(zones)) => BattleCardsChange::Enter { zones: zones.clone() },
        (Some(_), None) => BattleCardsChange::Exit,
        (Some(old), Some(zones))
            if old.floor == zones.floor && zones.turn >= old.turn =>
        {
            if old == zones {
                BattleCardsChange::Unchanged
            } else {
                BattleCardsChange::Update { zones: zones.clone() }
            }
        }
        _ => return Err("invalid card battle transition".into()),
    };
    Ok(CardZoneEvent::Changed {
        revision,
        permanent: (before.deck != after.deck).then(|| after.deck.clone()),
        battle,
    })
}

/// Invalid events leave the last committed state and revision untouched.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardZoneProjector {
    started: bool,
    deck: Vec<Card>,
    battle: Option<BattleCardZones>,
    last_floor: Option<u8>,
    revision: u64,
}

impl CardZoneProjector {
    pub fn push(&mut self, event: &CardZoneEvent) -> Result<(), String> {
        match event {
            CardZoneEvent::Started { deck } => {
                if self.started || deck.iter().any(|card| card.temporary) {
                    return Err("invalid or duplicate card journal start".into());
                }
                self.deck = deck.clone();
                self.started = true;
            }
            CardZoneEvent::Changed { revision, permanent, battle } => {
                if !self.started {
                    return Err("card journal missing start".into());
                }
                if self.revision.checked_add(1) != Some(*revision) {
                    return Err("gap or duplicate card journal revision".into());
                }
                let next_deck = permanent.as_ref().unwrap_or(&self.deck);
                if next_deck.iter().any(|card| card.temporary) {
                    return Err("temporary card in permanent deck".into());
                }
                let next_battle = match battle {
                    BattleCardsChange::Idle if self.battle.is_none() => None,
                    BattleCardsChange::Enter { zones }
                        if self.battle.is_none()
                            && zones.valid()
                            && self.last_floor.is_none_or(|floor| zones.floor > floor) =>
                    {
                        Some(zones.clone())
                    }
                    BattleCardsChange::Update { zones }
                        if self.battle.as_ref().is_some_and(|old| {
                            old.floor == zones.floor && zones.turn >= old.turn
                        }) && zones.valid() =>
                    {
                        Some(zones.clone())
                    }
                    BattleCardsChange::Unchanged if self.battle.is_some() => self.battle.clone(),
                    BattleCardsChange::Exit if self.battle.is_some() => None,
                    _ => return Err("invalid card battle lifecycle".into()),
                };
                if let Some(zones) = &next_battle {
                    self.last_floor = Some(zones.floor);
                }
                self.deck = next_deck.clone();
                self.battle = next_battle;
                self.revision = *revision;
            }
        }
        Ok(())
    }

    pub fn state(&self) -> Result<(CardZones, u64), String> {
        if !self.started {
            return Err("card journal missing start".into());
        }
        Ok((CardZones { deck: self.deck.clone(), battle: self.battle.clone() }, self.revision))
    }
}

pub fn project(events: &[CardZoneEvent]) -> Result<(CardZones, u64), String> {
    let mut projector = CardZoneProjector::default();
    for event in events {
        projector.push(event)?;
    }
    projector.state()
}

impl Game {
    /// Legacy saves lack the journal; this audit cannot reconstruct their past.
    pub fn audit_card_zones(&self) -> Result<usize, String> {
        let events = self.card_zone_history.as_ref()
            .ok_or("legacy save has no card zone authority journal")?;
        let (state, revision) = project(events)?;
        if state != CardZones::current(self) || revision != self.revision {
            return Err("card journal differs from current game state".into());
        }
        Ok(events.len())
    }
}
