//! Action-boundary authority for noncombat campaign state. No action replay.
//! Each successful revision is recorded; unchanged revisions carry no snapshot.
//! This covers map, route, narrative and inventory, but not full Game sourcing.
use crate::{Game, Stage, Node, Card, MAX_FLOOR, map, archetypes, challenges};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignState {
    pub floor: u8,
    pub stage: Stage,
    pub last_node: Option<Node>,
    pub route: Vec<Node>,
    pub act_map: Option<map::ActMap>,
    pub map_lane: Option<u8>,
    pub route_lanes: Vec<u8>,
    pub relics: Vec<String>,
    pub relic_history: Vec<String>,
    pub potions: u8,
    pub potion_bag: Vec<String>,
    pub shop_relic: Option<String>,
    pub boons: Vec<String>,
    pub boon_choices: Vec<String>,
    pub rewards: Vec<Card>,
    pub market: Vec<Card>,
    pub lore_flags: Vec<String>,
    pub lore_seen: Vec<String>,
    pub lore_current: Option<String>,
    pub easter_unlocked: Vec<String>,
    pub event_id: u8,
    pub victories: u32,
    pub variants: Option<[u8; 3]>,
    pub doctrine: Option<archetypes::Doctrine>,
    pub modifiers: Vec<challenges::ChallengeModifier>,
}

impl CampaignState {
    pub fn current(game: &Game) -> Self {
        Self {
            floor: game.floor, stage: game.stage, last_node: game.last_node,
            route: game.route.clone(), act_map: game.act_map.clone(),
            map_lane: game.map_lane, route_lanes: game.route_lanes.clone(),
            relics: game.relics.clone(), relic_history: game.relic_history.clone(),
            potions: game.potions, potion_bag: game.potion_bag.clone(),
            shop_relic: game.shop_relic.clone(), boons: game.boons.clone(),
            boon_choices: game.boon_choices.clone(), rewards: game.rewards.clone(),
            market: game.market.clone(), lore_flags: game.lore_flags.clone(),
            lore_seen: game.lore_seen.clone(), lore_current: game.lore_current.clone(),
            easter_unlocked: game.easter_unlocked.clone(), event_id: game.event_id,
            victories: game.victories, variants: game.variants,
            doctrine: game.doctrine, modifiers: game.modifiers.clone(),
        }
    }

    fn valid(&self) -> bool {
        self.floor <= MAX_FLOOR
            && self.map_lane.is_none_or(|lane| lane < map::LANES)
            && self.route_lanes.iter().all(|lane| *lane < map::LANES)
            && (self.route_lanes.is_empty() || self.route_lanes.len() == self.route.len())
            && self.act_map.as_ref().is_none_or(map::ActMap::is_valid)
            && (self.stage != Stage::Combat || self.floor > 0)
            && (self.lore_current.is_none() || self.stage == Stage::Story)
            && self.potions <= 3 && self.potion_bag.len() == usize::from(self.potions)
            && self.rewards.iter().chain(&self.market).all(|card| !card.temporary)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CampaignEvent {
    Started { state: CampaignState },
    Changed { revision: u64, state: Option<CampaignState> },
}

pub fn transition(before: &CampaignState, after: &CampaignState, revision: u64)
    -> Result<CampaignEvent, String>
{
    if !before.valid() || !after.valid() || after.floor < before.floor {
        return Err("invalid campaign state transition".into());
    }
    Ok(CampaignEvent::Changed {
        revision, state: (before != after).then(|| after.clone()),
    })
}

/// Invalid revisions do not mutate the last accepted campaign state.
#[derive(Clone, Debug, Default)]
pub struct CampaignProjector {
    state: Option<CampaignState>,
    revision: u64,
}

impl CampaignProjector {
    pub fn push(&mut self, event: &CampaignEvent) -> Result<(), String> {
        match event {
            CampaignEvent::Started { state } => {
                if self.state.is_some() || !state.valid() || state.floor != 0
                    || state.stage != Stage::Map
                {
                    return Err("invalid or duplicate campaign journal start".into());
                }
                self.state = Some(state.clone());
            }
            CampaignEvent::Changed { revision, state } => {
                let previous = self.state.as_ref().ok_or("campaign journal missing start")?;
                if self.revision.checked_add(1) != Some(*revision) {
                    return Err("gap or duplicate campaign journal revision".into());
                }
                if let Some(next) = state {
                    if !next.valid() || next.floor < previous.floor || next == previous {
                        return Err("invalid campaign journal change".into());
                    }
                    self.state = Some(next.clone());
                }
                self.revision = *revision;
            }
        }
        Ok(())
    }

    pub fn state(&self) -> Result<(CampaignState, u64), String> {
        Ok((self.state.clone().ok_or("campaign journal missing start")?, self.revision))
    }
}

pub fn project(events: &[CampaignEvent]) -> Result<(CampaignState, u64), String> {
    let mut projector = CampaignProjector::default();
    for event in events { projector.push(event)?; }
    projector.state()
}

impl Game {
    pub fn audit_campaign_authority(&self) -> Result<usize, String> {
        let history = self.campaign_history.as_ref()
            .ok_or("legacy save has no campaign authority journal")?;
        let (state, revision) = project(history)?;
        if revision != self.revision || state != CampaignState::current(self) {
            return Err("campaign authority differs from Game snapshot".into());
        }
        Ok(history.len())
    }
}
