//! Action-boundary authority for noncombat campaign state. No action replay.
//! Each successful revision is recorded; unchanged revisions carry no snapshot.
//! This covers map, route, narrative and inventory, but not full Game sourcing.
use crate::{Game, Stage, Node, Card, WuchaoPlan, ChibiPlan, JietingPlan, XiapiPlan, XiangfanPlan, RunMode, MAX_FLOOR, EXPANDED_MAX_FLOOR, map, archetypes, challenges, difficulty::Difficulty, content::Hero};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Immutable rules identity anchored at revision zero, independently of the
/// mutable Game snapshot. An absent identity means a pre-upgrade journal and
/// must never be reported as identity-certified.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignIdentity {
    pub hero: Hero,
    pub seed: u64,
    pub ascension: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,
    #[serde(default, skip_serializing_if = "crate::is_baseline_expanded_rules")]
    pub expanded_rules_revision: u8,
    pub mode: RunMode,
    pub run_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<CampaignIdentity>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wuchao_plan: Option<WuchaoPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chibi_plan: Option<ChibiPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jieting_plan: Option<JietingPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xiapi_plan: Option<XiapiPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xiangfan_plan: Option<XiangfanPlan>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub market_upgraded: bool,
    pub easter_unlocked: Vec<String>,
    pub event_id: u8,
    pub victories: u32,
    pub variants: Option<[u8; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doctrine: Option<archetypes::Doctrine>,
    pub modifiers: Vec<challenges::ChallengeModifier>,
}

impl CampaignState {
    pub fn current(game: &Game) -> Self {
        Self {
            identity: Some(CampaignIdentity {
                hero: game.hero, seed: game.seed, ascension: game.ascension,
                difficulty: game.difficulty, expanded_rules_revision: game.expanded_rules_revision,
                mode: game.mode, run_id: game.run_id.clone(),
            }),
            floor: game.floor, stage: game.stage, last_node: game.last_node,
            route: game.route.clone(), act_map: game.act_map.clone(),
            map_lane: game.map_lane, route_lanes: game.route_lanes.clone(),
            relics: game.relics.clone(), relic_history: game.relic_history.clone(),
            potions: game.potions, potion_bag: game.potion_bag.clone(),
            shop_relic: game.shop_relic.clone(), boons: game.boons.clone(),
            boon_choices: game.boon_choices.clone(), rewards: game.rewards.clone(),
            market: game.market.clone(), lore_flags: game.lore_flags.clone(),
            lore_seen: game.lore_seen.clone(), lore_current: game.lore_current.clone(),
            wuchao_plan: game.wuchao_plan, chibi_plan: game.chibi_plan,
            jieting_plan: game.jieting_plan, xiapi_plan: game.xiapi_plan,
            xiangfan_plan: game.xiangfan_plan, market_upgraded: game.market_upgraded,
            easter_unlocked: game.easter_unlocked.clone(), event_id: game.event_id,
            victories: game.victories, variants: game.variants,
            doctrine: game.doctrine, modifiers: game.modifiers.clone(),
        }
    }

    fn valid(&self) -> bool {
        self.floor <= self.identity.as_ref().map_or(MAX_FLOOR, |identity| {
            if identity.difficulty.is_some() { EXPANDED_MAX_FLOOR } else { MAX_FLOOR }
        })
            && self.identity.as_ref().is_none_or(|identity| {
                identity.ascension <= 10
                    && identity.expanded_rules_revision <= crate::EXPANDED_RULES_REVISION
                    && (identity.expanded_rules_revision == 0 || identity.difficulty.is_some())
                    && identity.difficulty.is_none_or(|_| identity.ascension == 0 && identity.mode == RunMode::Standard)
                    && (identity.mode == RunMode::Custom || self.modifiers.is_empty())
                    && self.doctrine.is_none_or(|doctrine| doctrine.hero() == identity.hero)
            })
            && self.map_lane.is_none_or(|lane| lane < map::LANES)
            && self.route_lanes.iter().all(|lane| *lane < map::LANES)
            && (self.route_lanes.is_empty() || self.route_lanes.len() == self.route.len())
            && self.act_map.as_ref().is_none_or(map::ActMap::is_valid)
            && (self.stage != Stage::Combat || self.floor > 0)
            && (self.lore_current.is_none() || self.stage == Stage::Story)
            && self.wuchao_plan.is_none_or(|plan| {
                self.identity.as_ref().is_some_and(|identity| identity.difficulty.is_some())
                    && (plan != WuchaoPlan::Council || self.stage == Stage::Story)
            })
            && self.chibi_plan.is_none_or(|plan| {
                self.identity.as_ref().is_some_and(|identity| identity.difficulty.is_some())
                    && (plan != ChibiPlan::Council || self.stage == Stage::Story)
            })
            && self.jieting_plan.is_none_or(|plan| {
                self.identity.as_ref().is_some_and(|identity| identity.difficulty.is_some()
                    && identity.hero == Hero::ZhugeLiang)
                    && self.floor >= 25
                    && (plan != JietingPlan::Council
                        || (self.stage == Stage::Story && (25..=31).contains(&self.floor)))
            })
            && self.xiapi_plan.is_none_or(|plan| {
                self.identity.as_ref().is_some_and(|identity| identity.difficulty.is_some()
                    && identity.expanded_rules_revision == crate::EXPANDED_RULES_REVISION
                    && identity.hero == Hero::LuBu)
                    && self.floor >= 17
                    && (plan != XiapiPlan::Council
                        || (self.stage == Stage::Story && (17..=31).contains(&self.floor)))
            })
            && self.xiangfan_plan.is_none_or(|plan| {
                self.identity.as_ref().is_some_and(|identity| identity.difficulty.is_some()
                    && identity.expanded_rules_revision == crate::EXPANDED_RULES_REVISION
                    && identity.hero == Hero::GuanYu)
                    && self.floor >= 25
                    && (plan != XiangfanPlan::Council
                        || (self.stage == Stage::Story && (25..=31).contains(&self.floor)))
            })
            && (!self.market_upgraded
                || (self.stage == Stage::Market
                    && self.identity.as_ref().is_some_and(|identity| identity.difficulty.is_some()
                        && identity.expanded_rules_revision == crate::EXPANDED_RULES_REVISION)))
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
    if !before.valid() || !after.valid() || after.floor < before.floor
        || before.identity.as_ref().is_some_and(|identity| after.identity.as_ref() != Some(identity))
    {
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
                    if next.identity != previous.identity {
                        return Err("campaign identity changed after start".into());
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


struct HistoricalAuditBundle {
    decisions: Vec<HistoricalDecision>,
    receipts: Vec<HistoricalEffectReceipt>,
    interactions: Vec<HistoricalInteraction>,
    untriggered: Vec<UntriggeredHistoryReason>,
}

/**
 * Stable committed decision candidates, including the councils and
 * selected literary decisions with named tactical consequences. A candidate
 * alone is not proof of an actual downstream effect. Narrative displays,
 * revisits and unchanged revisions never create another candidate.
 */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalDecision {
    pub id: String,
    pub revision: u64,
    pub floor: u8,
    pub kind: String,
}

fn tactical_lore_kind(flag: &str) -> Option<&'static str> {
    match flag {
        "mist_arrows" => Some("mist_arrows"),
        "empty_city" => Some("empty_city"),
        "jingzhou_warning" => Some("jingzhou_warning"),
        "fancheng_flood_record" => Some("fancheng_flood_record"),
        "star_lantern" => Some("star_lantern"),
        _ => None,
    }
}

pub fn historical_decisions(events: &[CampaignEvent]) -> Result<Vec<HistoricalDecision>, String> {
    let mut projector = CampaignProjector::default();
    let mut decisions = Vec::new();
    let mut recorded = HashSet::new();
    for event in events {
        let previous = projector.state.as_ref().cloned();
        projector.push(event)?;
        let Some(before) = previous else { continue };
        let Some(after) = projector.state.as_ref() else { continue };
        let revision = projector.revision;
        let Some(identity) = before.identity.as_ref().filter(|id| id.difficulty.is_some())
            else { continue };
        // A valid council decision is a Story -> Map commitment at the
        // same floor, by the correct hero and in the legal chapter window.
        // A direct None -> Pending or edited resolved plan is not evidence.
        let council_commit = before.stage == Stage::Story && after.stage == Stage::Map
            && before.floor == after.floor;
        let changes = [
            ("wuchao", council_commit && identity.hero == Hero::CaoCao
                && (9..=16).contains(&before.floor) && before.wuchao_plan == Some(WuchaoPlan::Council)
                && matches!(after.wuchao_plan, Some(WuchaoPlan::RaidPending | WuchaoPlan::FortifyPending | WuchaoPlan::DeclinePending))),
            ("chibi", council_commit && matches!(identity.hero, Hero::ZhugeLiang | Hero::SunShangxiang)
                && (17..=24).contains(&before.floor) && before.chibi_plan == Some(ChibiPlan::Council)
                && matches!(after.chibi_plan, Some(ChibiPlan::FirePending | ChibiPlan::FleetPending | ChibiPlan::DeclinePending))),
            ("jieting", council_commit && identity.hero == Hero::ZhugeLiang
                && (25..=30).contains(&before.floor) && before.jieting_plan == Some(JietingPlan::Council)
                && matches!(after.jieting_plan, Some(JietingPlan::WaterPending | JietingPlan::HillPending | JietingPlan::RetreatPending))),
            ("xiapi", council_commit && identity.hero == Hero::LuBu
                && (17..=31).contains(&before.floor) && before.xiapi_plan == Some(XiapiPlan::Council)
                && matches!(after.xiapi_plan, Some(XiapiPlan::SplitPending | XiapiPlan::DefensePending | XiapiPlan::ChargePending))),
            ("xiangfan", council_commit && identity.hero == Hero::GuanYu
                && (25..=31).contains(&before.floor) && before.xiangfan_plan == Some(XiangfanPlan::Council)
                && matches!(after.xiangfan_plan, Some(XiangfanPlan::FloodPending | XiangfanPlan::DikePending | XiangfanPlan::HoldPending))),
        ];
        for (kind, changed) in changes {
            if changed && recorded.insert(kind) {
                decisions.push(HistoricalDecision {
                    id: format!("{}:{}:{}", after.identity.as_ref().map_or("", |id| id.run_id.as_str()), kind, revision),
                    revision,
                    floor: after.floor,
                    kind: kind.to_owned(),
                });
            }
        }
        // Only a committed choice at the current literary scene can unlock
        // its tactical receipt. Merely opening the scene, reading a flag,
        // or carrying a flag in an initial snapshot cannot create a decision.
        if before.stage == Stage::Story
            && let Some(beat) = before.lore_current.as_deref().and_then(crate::lore::beat)
            && !before.lore_seen.iter().any(|id| id == beat.id)
            && after.lore_seen.iter().any(|id| id == beat.id)
            && !before.lore_flags.iter().any(|flag| flag == beat.flag)
            && after.lore_flags.iter().any(|flag| flag == beat.flag)
            && let Some(kind) = tactical_lore_kind(beat.flag)
            && recorded.insert(kind)
        {
            decisions.push(HistoricalDecision {
                id: format!("{}:{}:{}", after.identity.as_ref().map_or("", |id| id.run_id.as_str()), kind, revision),
                revision,
                floor: after.floor,
                kind: kind.to_owned(),
            });
        }
    }
    Ok(decisions)
}

/// First observed positive, replay-verified tactical effect per historical
/// decision and battlefield. This is evidence of an effect, not an additional
/// meaningful-interaction count: multiple effects of one decision may overlap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalEffectReceipt {
    pub decision_id: String,
    pub revision: u64,
    pub floor: u8,
    pub encounter: u64,
    pub event_seq: u64,
    pub battlefield: String,
    pub source: String,
    pub amount: i32,
}

/// A conservative countable player interaction: one committed scene/council
/// choice, with actual immediate changes or at least one verified downstream
/// battlefield effect. The categories overlap and never inflate the count.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalInteraction {
    pub key: String,
    pub revision: u64,
    pub floor: u8,
    pub choice: usize,
    pub hp_delta: i64,
    pub max_hp_delta: i64,
    pub gold_delta: i64,
    pub deck_changed: bool,
    pub battlefield_receipts: usize,
    pub cross_act_receipts: usize,
    /// Actual changes to a legal next-floor route, not merely a plan flag.
    #[serde(default)]
    pub map_receipts: usize,
    #[serde(default)]
    pub first_map_effect_floor: Option<u8>,
    /// For independent tactical interactions (choice == 0), the exact
    /// replay-verified positive effect, rather than an invented story choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle_source: Option<String>,
}

/// Explicit explanation when a historical scene or tactical decision is
/// missing, declined, or unexercised along a branch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UntriggeredHistoryReason {
    pub key: String,
    pub category: &'static str,
    pub floor: u8,
    pub reason: &'static str,
}

/// Replay-derived measurement for one expanded journey. A partial or lost
/// journey remains visible, but is not a completed forty-floor sample.
/// Counts describe the covered subset only; they do not certify SPEC density.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HistoricalDensityReport {
    pub schema_version: u8,
    pub hero: Hero,
    pub difficulty: Difficulty,
    pub seed: u64,
    pub floor: u8,
    pub stage: Stage,
    pub complete_campaign: bool,
    pub covered_lower_bound: usize,
    pub resource_or_deck: usize,
    pub map_effect: usize,
    pub battle_effect: usize,
    pub cross_act_effect: usize,
    pub interactions: Vec<HistoricalInteraction>,
    pub untriggered_reasons: Vec<UntriggeredHistoryReason>,
}

impl HistoricalDensityReport {
    fn from_verified(
        game: &Game,
        difficulty: Difficulty,
        interactions: Vec<HistoricalInteraction>,
        untriggered_reasons: Vec<UntriggeredHistoryReason>,
    ) -> Self {
        Self {
            schema_version: 2,
            hero: game.hero,
            difficulty,
            seed: game.seed,
            floor: game.floor,
            stage: game.stage,
            complete_campaign: game.stage == Stage::Won && game.floor == EXPANDED_MAX_FLOOR,
            covered_lower_bound: interactions.len(),
            resource_or_deck: interactions.iter().filter(|r|
                r.hp_delta != 0 || r.max_hp_delta != 0 || r.gold_delta != 0 || r.deck_changed).count(),
            map_effect: interactions.iter().filter(|r| r.map_receipts > 0).count(),
            battle_effect: interactions.iter().filter(|r| r.battlefield_receipts > 0).count(),
            cross_act_effect: interactions.iter().filter(|r| r.cross_act_receipts > 0).count(),
            interactions,
            untriggered_reasons,
        }
    }
}

/// Compare the committed map options with the original seeded topology.
/// A pending plan or a map preview alone is not an effect. Count only a
/// real change in a reachable lane's node, preserving the original graph.
fn jieting_route_changed(state: &CampaignState) -> bool {
    let next = state.floor.saturating_add(1);
    if state.stage != Stage::Map || !(26..=31).contains(&next)
        || !state.identity.as_ref().is_some_and(|identity|
            identity.difficulty.is_some() && identity.hero == Hero::ZhugeLiang)
        || !matches!(state.jieting_plan, Some(JietingPlan::WaterPending
            | JietingPlan::HillPending | JietingPlan::RetreatPending))
    { return false; }
    let Some(map) = state.act_map.as_ref() else { return false; };
    if map.act != (next - 1) / 8 { return false; }
    let seeded = map.routes(usize::from((next - 1) % 8), state.map_lane);
    if seeded.len() != state.route.len() || seeded.len() != state.route_lanes.len()
        || !seeded.iter().zip(&state.route_lanes).all(|((lane, _), actual)| lane == actual)
    { return false; }
    seeded.iter().zip(&state.route).any(|((_, original), actual)|
        original != actual && *actual == Node::Battle)
}

// A council can alter more than one subsequent route. Each floor is counted
// once, and all floors remain attached to the original decision rather than
// creating additional independent history interactions.
fn jieting_map_effect_floors(
    history: &[CampaignEvent], decision_revision: u64,
) -> Result<Vec<u8>, String> {
    let mut projector = CampaignProjector::default();
    let mut floors = Vec::new();
    for event in history {
        projector.push(event)?;
        if projector.revision < decision_revision { continue; }
        if let Some(state) = projector.state.as_ref()
            && jieting_route_changed(state)
        {
            let next = state.floor + 1;
            if !floors.contains(&next) { floors.push(next); }
        }
    }
    Ok(floors)
}

fn historical_effect_site(source: &str) -> Option<(&'static str, &'static str)> {
    match source {
        "wuchao_act_three_raid" | "wuchao_act_three_fortify" | "wuchao_act_three_decline" =>
            Some(("wuchao", "third_act_settlement")),
        "chibi_act_three_fire" | "chibi_act_three_fleet" | "chibi_act_three_decline" =>
            Some(("chibi", "third_act_settlement")),
        "chibi_act_four_debt" | "chibi_act_four_supply" | "chibi_act_four_setback" =>
            Some(("chibi", "fourth_act_settlement")),
        "jieting_act_four_water" | "jieting_act_four_hill" | "jieting_act_four_retreat" =>
            Some(("jieting", "fourth_act_settlement")),
        "jieting_act_five_water" | "jieting_act_five_debt" | "jieting_act_five_recovery" =>
            Some(("jieting", "fifth_act_settlement")),
        "xiapi_split_battle" | "xiapi_camp_defense" | "xiapi_lone_charge" =>
            Some(("xiapi", "field_skirmish")),
        "xiapi_split_boss" | "xiapi_camp_boss" | "xiapi_charge_boss" =>
            Some(("xiapi", "act_boss")),
        "xiangfan_flood_battle" | "xiangfan_dike_battle" | "xiangfan_hold_battle" =>
            Some(("xiangfan", "field_skirmish")),
        "xiangfan_flood_boss" | "xiangfan_dike_boss" | "xiangfan_hold_boss" =>
            Some(("xiangfan", "fourth_act_boss")),
        "wuchao_coalition_raid" | "wuchao_coalition_fortified" | "wuchao_coalition_unchecked" =>
            Some(("wuchao", "guandu")),
        "wuchao_tyrant_raid" | "wuchao_tyrant_fortified" | "wuchao_tyrant_unchecked"
            => Some(("wuchao", "dongzhuo")),
        "wuchao_tyrant_supply_saved" => Some(("wuchao", "dongzhuo_counter")),
        "wuchao_naval_raid_disruption" | "wuchao_naval_supply_screen"
        | "wuchao_naval_supply_shortage" => Some(("wuchao", "naval")),
        "wuchao_naval_raid_countered" => Some(("wuchao", "naval_counter")),
        "chibi_naval_fleet_screen" | "chibi_naval_fire_spread"
        | "chibi_naval_declined_cover" => Some(("chibi", "naval")),
        "chibi_naval_fire_contained" => Some(("chibi", "naval_counter")),
        "chibi_fourth_act_fire" | "chibi_fourth_act_fleet" | "chibi_fourth_act_declined"
            => Some(("chibi", "fourth_act_boss")),
        "chibi_luxun_firebreak" | "chibi_lumeng_counterattack" =>
            Some(("chibi", "fourth_act_boss_counter")),
        "jieting_hill_abandons_wells" => Some(("jieting", "water")),
        "jieting_hill_suppression" | "jieting_hill_recaptured_wells"
        | "jieting_water_wells_repaired" | "jieting_retreat_water_salvaged" =>
            Some(("jieting", "water_counter")),
        "jieting_jiangwei_wells" | "jieting_jiangwei_hills" | "jieting_jiangwei_retreat" =>
            Some(("jieting", "jiangwei")),
        "jieting_yinping_wells" | "jieting_yinping_hills" | "jieting_yinping_retreat"
        | "jieting_late_water_settlement" => Some(("jieting", "yinping")),
        "jieting_yinping_water_screen" | "jieting_yinping_retreat_command" =>
            Some(("jieting", "yinping_counter")),
        "mist_arrows_naval_counter" => Some(("mist_arrows", "naval")),
        "empty_city_scheme_countered" => Some(("empty_city", "third_act_boss")),
        "jingzhou_vigilance_countered" => Some(("jingzhou_warning", "fourth_act_boss")),
        "fancheng_flood_counter" | "fancheng_flood_exposure" =>
            Some(("fancheng_flood_record", "fancheng_flood")),
        "wuzhang_morale_recovered" => Some(("star_lantern", "northern_expedition")),
        _ => None,
    }
}

/// Explicitly historical battle counterplay that needs no literary flag.
/// Generic attacks, poison ticks, routine victories, and displays are not
/// historical interactions. Multiple effects of one site remain one record.
fn standalone_counter_site(floor: u8, kind: &crate::EventKind, source: &str)
    -> Option<(&'static str, &'static str)>
{
    use crate::EventKind;
    match (floor, kind, source) {
        (16, EventKind::PowerTriggered, "yuanshao_escort_severed") =>
            Some(("battle:guandu_escort", "wuchao")),
        (40, EventKind::DamageDealt, "yinping_interception")
        | (40, EventKind::PowerTriggered, "yinping_vanguard_broken" | "yinping_escort_gold_recovered") =>
            Some(("battle:yinping_counter", "jieting")),
        _ => None,
    }
}

fn standalone_battle_interactions(
    trace: &[crate::replay::ActionEventTrace], decisions: &[HistoricalDecision],
) -> Vec<HistoricalInteraction> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for action in trace {
        if !matches!(&action.action, crate::Action::Play { .. } | crate::Action::EndTurn
            | crate::Action::PlaySelected { .. })
        { continue; }
        for record in &action.events {
            if record.event.amount <= 0 { continue; }
            let Some((site, council)) = standalone_counter_site(
                record.floor, &record.event.kind, &record.event.source
            ) else { continue; };
            // Be conservative when a prior council already owns this
            // historical episode. Do not count the same site twice.
            if decisions.iter().any(|d| d.kind == council && d.revision < action.revision)
                || !seen.insert(site)
            { continue; }
            result.push(HistoricalInteraction {
                key: site.into(), revision: action.revision, floor: record.floor, choice: 0,
                hp_delta: 0, max_hp_delta: 0, gold_delta: 0, deck_changed: false,
                battlefield_receipts: 1, cross_act_receipts: 0,
                map_receipts: 0, first_map_effect_floor: None,
                battle_source: Some(record.event.source.clone()),
            });
        }
    }
    result
}

impl Game {
    /// Offline diagnostic. Replays the committed actions and checks the exact
    /// event stream and final snapshot before associating a real positive
    /// battlefield effect with an earlier historical decision. No new save
    /// fields, RNG draws or journal mutations; old unverified saves cannot
    /// manufacture effect receipts. This deliberately covers only named sites.
    pub fn historical_effect_receipts(&self) -> Result<Vec<HistoricalEffectReceipt>, String> {
        if self.difficulty.is_none() { return Ok(Vec::new()); }
        Ok(self.collect_historical_audit()?.receipts)
    }

    fn collect_historical_audit(&self) -> Result<HistoricalAuditBundle, String> {
        let trace = self.reconstruct_action_event_trace()?;
        let history = self.campaign_history.as_ref()
            .ok_or("missing campaign journal for historical effect receipts")?;
        let decisions = historical_decisions(history)?;
        let receipts = self.decision_effect_receipts(&decisions, &trace);
        let interactions = self.interactions_from_verified_receipts(&decisions, &receipts, &trace)?;
        let untriggered = self.collect_untriggered_reasons(&decisions, &receipts, history)?;
        Ok(HistoricalAuditBundle { decisions, receipts, interactions, untriggered })
    }

    fn decision_effect_receipts(
        &self, decisions: &[HistoricalDecision], trace: &[crate::replay::ActionEventTrace],
    ) -> Vec<HistoricalEffectReceipt> {
        let mut seen = HashSet::new();
        let mut receipts = Vec::new();
        for action in trace {
            for record in &action.events {
                if record.event.amount <= 0
                    || !matches!(record.event.kind, crate::EventKind::PowerTriggered
                        | crate::EventKind::EnemyTechnique | crate::EventKind::DamageDealt
                        | crate::EventKind::DamageReceived)
                { continue; }
                let Some((kind, battlefield)) = historical_effect_site(&record.event.source) else { continue };
                let Some(decision) = decisions.iter().find(|decision|
                    decision.kind == kind && decision.revision < action.revision) else { continue };
                if !seen.insert((kind, battlefield)) { continue; }
                receipts.push(HistoricalEffectReceipt {
                    decision_id: decision.id.clone(),
                    revision: action.revision,
                    floor: record.floor,
                    encounter: record.encounter,
                    event_seq: record.event.seq,
                    battlefield: battlefield.to_owned(),
                    source: record.event.source.clone(),
                    amount: record.event.amount,
                });
            }
        }
        receipts
    }

    fn collect_untriggered_reasons(
        &self,
        decisions: &[HistoricalDecision],
        receipts: &[HistoricalEffectReceipt],
        history: &[CampaignEvent],
    ) -> Result<Vec<UntriggeredHistoryReason>, String> {
        let mut reasons = Vec::new();
        let mut seen_keys = HashSet::new();
        let mut visited_battles = HashSet::new();
        let mut projector = CampaignProjector::default();
        for event in history {
            let previous = projector.state.as_ref().cloned();
            projector.push(event)?;
            let Some(before) = previous else { continue; };
            let Some(after) = projector.state.as_ref() else { continue; };
            if after.last_node == Some(Node::Battle) {
                visited_battles.insert(after.floor);
            }
            if before.stage == Stage::Story
                && let Some(beat) = before.lore_current.as_deref().and_then(crate::lore::beat)
                && !before.lore_seen.iter().any(|id| id == beat.id)
                && after.lore_seen.iter().any(|id| id == beat.id)
                && !after.lore_flags.iter().any(|flag| flag == beat.flag)
                && !decisions.iter().any(|d| d.kind == beat.flag)
                && seen_keys.insert(format!("scene:{}", beat.id))
            {
                reasons.push(UntriggeredHistoryReason {
                    key: format!("scene:{}", beat.id),
                    category: "declined_story",
                    floor: before.floor,
                    reason: "遇到文学事件但未采纳对应战术选项",
                });
            }
        }
        let councils = [
            ("wuchao", self.hero == Hero::CaoCao, 9_u8, 16_u8),
            ("chibi", matches!(self.hero, Hero::ZhugeLiang | Hero::SunShangxiang), 17, 24),
            ("jieting", self.hero == Hero::ZhugeLiang, 25, 30),
            ("xiapi", self.hero == Hero::LuBu, 17, 31),
            ("xiangfan", self.hero == Hero::GuanYu, 25, 31),
        ];
        for (kind, eligible, start_floor, end_floor) in councils {
            if !eligible || decisions.iter().any(|d| d.kind == kind) { continue; }
            if self.floor > end_floor
                || (matches!(self.stage, Stage::Won | Stage::Lost) && self.floor >= start_floor)
            {
                let key = format!("council:{kind}");
                if seen_keys.insert(key.clone()) {
                    reasons.push(UntriggeredHistoryReason {
                        key,
                        category: "missed_council",
                        floor: self.floor.min(end_floor),
                        reason: "经过军议章节窗口但未触发或完成该军议抉择",
                    });
                }
            }
        }
        for decision in decisions {
            if receipts.iter().any(|r| r.decision_id == decision.id) { continue; }
            if decision.kind == "jieting" && !jieting_map_effect_floors(history, decision.revision)?.is_empty() {
                continue;
            }
            let (category, reason) = match decision.kind.as_str() {
                "empty_city" => {
                    if self.floor >= 24 && self.act_variant(2) != 0 {
                        ("alternate_boss_branch", "第三幕选用替代关底左慈，未对阵司马懿")
                    } else if self.floor < 24 {
                        ("target_not_reached", "尚未推进至第24层司马懿关底")
                    } else {
                        ("counter_condition_unmet", "对阵司马懿时未满足出牌1–2张且保持10护甲条件")
                    }
                }
                "mist_arrows" => {
                    if self.floor < 19 {
                        ("target_not_reached", "尚未推进至第19层水陆阵战")
                    } else if !visited_battles.contains(&19) {
                        ("route_avoided_site", "行军路线未进入第19层水陆阵战节点")
                    } else {
                        ("counter_condition_unmet", "第19层水陆阵战偶数回合未满足出牌且至少4点掩护条件")
                    }
                }
                "fancheng_flood_record" => {
                    if self.floor < 27 {
                        ("target_not_reached", "尚未推进至第27层樊城水势战场")
                    } else if !visited_battles.contains(&27) {
                        ("route_avoided_site", "行军路线未进入第27层樊城野战节点")
                    } else {
                        ("counter_condition_unmet", "第27层偶数回合未触发或未满足8护甲水势反制条件")
                    }
                }
                "jingzhou_warning" => {
                    if self.floor >= 32 && self.act_variant(3) != 1 {
                        ("alternate_boss_branch", "第四幕选用替代关底陆逊，未对阵吕蒙")
                    } else if self.floor < 32 {
                        ("target_not_reached", "尚未推进至第32层吕蒙关底")
                    } else {
                        ("counter_condition_unmet", "对阵吕蒙时未满足出牌且至少8护甲条件")
                    }
                }
                "star_lantern" => {
                    if self.floor >= 40 && self.act_variant(4) != 0 {
                        ("alternate_boss_branch", "第五幕选用替代关底邓艾，未对阵姜维")
                    } else if self.floor < 40 {
                        ("target_not_reached", "尚未推进至第40层姜维关底")
                    } else {
                        ("counter_condition_unmet", "对阵姜维首回合未满足出牌至少2张并守住断粮")
                    }
                }
                kind => {
                    let min_site_floor = match kind {
                        "wuchao" => 16,
                        "chibi" => 19,
                        "jieting" => 26,
                        "xiapi" | "xiangfan" => decision.floor.saturating_add(1),
                        _ => decision.floor.saturating_add(1),
                    };
                    if self.floor < min_site_floor {
                        ("target_not_reached", "作出军议后尚未推进至对应的兑现战场或关底")
                    } else {
                        ("counter_condition_unmet", "已到达军议生效窗口但未触发已核验的正面兑现事件")
                    }
                }
            };
            reasons.push(UntriggeredHistoryReason {
                key: format!("decision:{}", decision.kind),
                category,
                floor: self.floor.max(decision.floor),
                reason,
            });
        }
        Ok(reasons)
    }

    /// A lower bound on meaningful history interactions, not a claim that
    /// all map/battle anchors are covered. The full replay performed by
    /// historical_effect_receipts verifies all the journals used below.
    pub fn historical_interactions(&self) -> Result<Vec<HistoricalInteraction>, String> {
        if self.difficulty.is_none() { return Ok(Vec::new()); }
        Ok(self.collect_historical_audit()?.interactions)
    }

    /// Structured, anonymous observation for offline aggregation. Fail closed
    /// on missing or inconsistent action evidence; never manufacture a zero
    /// density measurement for a legacy campaign.
    pub fn historical_density_report(&self) -> Result<HistoricalDensityReport, String> {
        let difficulty = self.difficulty.ok_or("旧制战役不适用新制历史触动统计")?;
        let audit = self.collect_historical_audit()?;
        Ok(HistoricalDensityReport::from_verified(self, difficulty, audit.interactions, audit.untriggered))
    }

    fn interactions_from_verified_receipts(
        &self,
        decisions: &[HistoricalDecision],
        receipts: &[HistoricalEffectReceipt],
        trace: &[crate::replay::ActionEventTrace],
    ) -> Result<Vec<HistoricalInteraction>, String> {
        use crate::{Action, resource_events::ResourceEvent, card_zone_events::CardZoneEvent};
        let history = self.campaign_history.as_ref().ok_or("missing campaign journal")?;
        let resources = self.resource_history.as_ref().ok_or("missing resource journal")?;
        let cards = self.card_zone_history.as_ref().ok_or("missing permanent card journal")?;
        let actions = self.action_history.as_ref().ok_or("missing action journal")?;
        let jieting_maps = if let Some(decision) = decisions.iter().find(|d| d.kind == "jieting") {
            jieting_map_effect_floors(history, decision.revision)?
        } else { Vec::new() };
        let mut projection = CampaignProjector::default();
        let mut seen = HashSet::new();
        let mut interactions = Vec::new();
        for event in history {
            let previous = projection.state.as_ref().cloned();
            projection.push(event)?;
            let Some(before) = previous else { continue };
            let Some(after) = projection.state.as_ref() else { continue };
            if before.stage != Stage::Story { continue; }
            let revision = projection.revision;
            let offset = usize::try_from(revision).map_err(|_| "revision outside platform")?;
            let Some(Action::Choose { index }) = offset.checked_sub(1)
                .and_then(|i| actions.get(i)) else { continue };
            let decision = decisions.iter().find(|d| d.revision == revision);
            // Council subchoices own the action, even when opening the
            // council did not clear a literary scene from the live snapshot.
            let council = decision.filter(|d| matches!(d.kind.as_str(),
                "wuchao" | "chibi" | "jieting" | "xiapi" | "xiangfan"));
            let key = if let Some(d) = council {
                format!("council:{}", d.kind)
            } else if let Some(beat) = before.lore_current.as_deref().and_then(crate::lore::beat) {
                if !matches!(*index, 1 | 2 | 4)
                    || before.lore_seen.iter().any(|id| id == beat.id)
                    || !after.lore_seen.iter().any(|id| id == beat.id)
                { continue; }
                format!("scene:{}", beat.id)
            } else { continue; };
            let Some(ResourceEvent::Changed {
                revision: resource_revision, hp_delta, max_hp_delta, gold_delta,
            }) = resources.get(offset) else { return Err("missing interaction resource boundary".into()); };
            let Some(CardZoneEvent::Changed {
                revision: card_revision, permanent, ..
            }) = cards.get(offset) else { return Err("missing interaction card boundary".into()); };
            if *resource_revision != revision || *card_revision != revision {
                return Err("interaction journal revision mismatch".into());
            }
            let related = receipts.iter().filter(|r|
                decision.is_some_and(|d| r.decision_id == d.id));
            let (battlefield_receipts, cross_act_receipts) =
                related.fold((0, 0), |(total, cross), r| {
                    let later_act = r.floor.saturating_sub(1) / self.campaign_rows()
                        > before.floor.saturating_sub(1) / self.campaign_rows();
                    (total + 1, cross + usize::from(later_act))
                });
            let deck_changed = permanent.is_some();
            let map_floors = if decision.is_some_and(|d| d.kind == "jieting") {
                jieting_maps.as_slice()
            } else { &[] };
            if *hp_delta == 0 && *max_hp_delta == 0 && *gold_delta == 0
                && !deck_changed && battlefield_receipts == 0 && map_floors.is_empty()
            { continue; }
            if !seen.insert(key.clone()) { continue; }
            interactions.push(HistoricalInteraction {
                key, revision, floor: before.floor, choice: *index,
                hp_delta: *hp_delta, max_hp_delta: *max_hp_delta, gold_delta: *gold_delta,
                deck_changed, battlefield_receipts, cross_act_receipts,
                map_receipts: map_floors.len(), first_map_effect_floor: map_floors.first().copied(),
                battle_source: None,
            });
        }
        interactions.extend(standalone_battle_interactions(trace, decisions));
        interactions.sort_by_key(|interaction| interaction.revision);
        Ok(interactions)
    }

    /// Player-facing archive, derived from the same replay-verified receipts
    /// rather than a second game truth. It is a conservative covered subset,
    /// not a full SPEC density measurement or a historical factual claim.
    pub fn historical_archive_report(&self) -> Result<String, String> {
        if self.difficulty.is_none() {
            return Ok("旧制战役保留原演义纪事；新制战史档案不适用。".into());
        }
        let audit = self.collect_historical_audit()?;
        let effective = audit.receipts.iter().map(|r| r.decision_id.as_str())
            .collect::<HashSet<_>>().len();
        let density = HistoricalDensityReport::from_verified(
            self,
            self.difficulty.expect("expanded campaign"),
            audit.interactions,
            audit.untriggered,
        );
        let interactions = &density.interactions;
        let decisions = &audit.decisions;
        let receipts = &audit.receipts;
        let (immediate, battle, cross_act, map) =
            (density.resource_or_deck, density.battle_effect, density.cross_act_effect, density.map_effect);
        let mut lines = vec![format!(
            "战史档案（动作重放已核对）：已记录{}项战术选择，{}项选择有实际战场后果，共{}个去重后的战场兑现点。",
            decisions.len(), effective, receipts.len()
        ), "仅统计已覆盖的历史选择及战术；不等于完整12–18次有效触动指标。小说章回为文学背景，战术数值与反制属于游戏原创推演。".into()];
        lines.push(format!("有效历史触动保守下界：{}次；即时资源/牌组变化{}次、地图变化{}次、战场兑现{}次、跨幕兑现{}次（类别重叠，不相加）。", interactions.len(), immediate, map, battle, cross_act));
        for interaction in interactions {
            if let Some(source) = interaction.battle_source.as_ref() {
                let (title, chapter) = if interaction.key == "battle:guandu_escort" {
                    ("官渡断援", 30)
                } else { ("阴平截击", 117) };
                lines.push(format!("  【{title}】第{}层／动作{}；独立战场反制：{source}；文学背景：毛评演义第{chapter:03}回 {}；游戏原创推演，不是故事选项。",
                    interaction.floor, interaction.revision, crate::lore::provenance_url(chapter)));
                continue;
            }
            lines.push(format!("  {} 第{}层／动作{}／选择{}；体力{:+}、上限{:+}、金币{:+}、牌组变化{}；战场兑现{}处",
                interaction.key, interaction.floor, interaction.revision, interaction.choice,
                interaction.hp_delta, interaction.max_hp_delta, interaction.gold_delta,
                if interaction.deck_changed { "有" } else { "无" }, interaction.battlefield_receipts));
            if let Some(beat) = interaction.key.strip_prefix("scene:").and_then(crate::lore::beat) {
                lines.push(format!("    【{}】文学背景：毛评演义第{:03}回 {}；游戏选择及资源变化为原创推演。",
                    beat.title, beat.chapter, crate::lore::provenance_url(beat.chapter)));
            }
            if let Some(floor) = interaction.first_map_effect_floor {
                lines.push(format!("    街亭水源责任实际改变{}层的可选路线，首次第{floor}层；同一军议不重复计数。",
                    interaction.map_receipts));
            }
        }
        for decision in decisions {
            let (chapter, title) = match decision.kind.as_str() {
                "wuchao" => (30, "官渡乌巢"),
                "chibi" => (49, "赤壁东风"),
                "jieting" => (95, "街亭军议"),
                "xiapi" => (19, "下邳破围"),
                "xiangfan" => (74, "襄樊水防"),
                flag => interactions.iter().find(|r| r.revision == decision.revision)
                    .and_then(|r| r.key.strip_prefix("scene:"))
                    .and_then(crate::lore::beat)
                    .or_else(|| crate::lore::BEATS.iter().find(|b| b.flag == flag))
                    .map_or((0, "历史战术"), |b| (b.chapter, b.title)),
            };
            lines.push(format!("【{title}】第{}层／动作{}；文学背景：毛评演义第{chapter:03}回 {}",
                decision.floor, decision.revision,
                if chapter == 0 { String::new() } else { crate::lore::provenance_url(chapter).to_string() }));
            let mut affected = false;
            for receipt in receipts.iter().filter(|r| r.decision_id == decision.id) {
                affected = true;
                lines.push(format!("  游戏推演兑现：第{}层／动作{}／战斗{}事件{}；{}（{}）",
                    receipt.floor, receipt.revision, receipt.encounter, receipt.event_seq,
                    receipt.source, receipt.amount));
            }
            if !affected { lines.push("  尚无已核验的实际战场兑现。".into()); }
        }
        for reason in &density.untriggered_reasons {
            lines.push(format!("  未触发原因核对：{} [{}]（第{}层）：{}",
                reason.key, reason.category, reason.floor, reason.reason));
        }
        if decisions.is_empty() { lines.push("尚无已覆盖的历史战术选择。".into()); }
        Ok(lines.join("\n"))
    }

    // Cheap, immutable revision-zero check; never scan the growing journal
    // on each action. This is not authentication of a deliberately rewritten
    // whole save. Legacy identity-free journals retain their old semantics.
    pub(crate) fn verify_rules_epoch_anchor(&self) -> Result<(), String> {
        if self.expanded_rules_revision > crate::EXPANDED_RULES_REVISION
            || (self.expanded_rules_revision != 0 && self.difficulty.is_none())
        {
            return Err("不支持的战役规则版本".into());
        }
        let anchor = match self.campaign_history.as_deref().and_then(|events| events.first()) {
            Some(CampaignEvent::Started { state }) => state.identity.as_ref(),
            _ => None,
        };
        match anchor {
            Some(identity) if identity.expanded_rules_revision != self.expanded_rules_revision
                || identity.difficulty != self.difficulty =>
                Err("战役起始记录与当前规则版本或难度不一致".into()),
            None if self.expanded_rules_revision != 0 =>
                Err("新制规则版本缺少战役起始身份记录".into()),
            _ => Ok(()),
        }
    }

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


#[cfg(test)]
mod tactical_lore_receipt_projection_tests {
    use super::*;

    // Projector fixture only: the full receipt API still requires successful
    // replay of the actual player actions and rejects manipulated journals.
    #[test]
    fn literary_choice_requires_both_new_seen_scene_and_new_flag() {
        let game = Game::new_with_campaign_difficulty(
            Hero::ZhugeLiang, 7, Difficulty::Warlords, "literary-receipt".into(),
        );
        let start = CampaignState::current(&game);
        let mut scene = start.clone();
        scene.floor = 17;
        scene.stage = Stage::Story;
        scene.lore_current = Some("empty_city".into());
        let mut declined = scene.clone();
        declined.stage = Stage::Map;
        declined.lore_current = None;
        declined.lore_seen.push("empty_city".into());
        let mut accepted = declined.clone();
        accepted.lore_flags.push("empty_city".into());
        let base = vec![
            CampaignEvent::Started { state: start.clone() },
            transition(&start, &scene, 1).unwrap(),
        ];
        assert!(historical_decisions(&base).unwrap().is_empty());

        let mut no_choice = base.clone();
        no_choice.push(transition(&scene, &declined, 2).unwrap());
        assert!(historical_decisions(&no_choice).unwrap().is_empty());

        let mut committed = base;
        committed.push(transition(&scene, &accepted, 2).unwrap());
        committed.push(transition(&accepted, &accepted, 3).unwrap());
        let candidates = historical_decisions(&committed).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, "empty_city");
        assert_eq!(candidates[0].revision, 2);
    }
}


#[cfg(test)]
mod rules_epoch_anchor_regression {
    use super::*;

    #[test]
    fn current_epoch_cannot_be_removed_from_snapshot_or_anchor() {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::CaoCao, 7, Difficulty::Warlords, "epoch-anchor".into(),
        );
        game.apply(crate::Action::Choose { index: 1 }).unwrap();
        let raw = game.to_json().unwrap();
        let mut edited: serde_json::Value = serde_json::from_str(&raw).unwrap();
        edited.as_object_mut().unwrap().remove("expanded_rules_revision");
        assert!(Game::from_json(&edited.to_string()).is_err());

        let mut changed = game.clone();
        changed.expanded_rules_revision = 0;
        let before = changed.to_json().unwrap();
        assert!(changed.apply(crate::Action::EndTurn).is_err());
        assert_eq!(changed.to_json().unwrap(), before);

        let mut missing = game.clone();
        missing.campaign_history = None;
        assert!(Game::from_json(&missing.to_json().unwrap()).is_err());
        let before = missing.to_json().unwrap();
        assert!(missing.apply(crate::Action::EndTurn).is_err());
        assert_eq!(missing.to_json().unwrap(), before);
        assert!(Game::from_json(&raw).unwrap().verify_action_replay().is_ok());
    }

    #[test]
    fn shipped_epoch_without_identity_remains_loadable() {
        let mut game = Game::new(Hero::CaoCao, 7);
        game.campaign_history = None;
        let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        restored.apply(crate::Action::Choose { index: 1 }).unwrap();
        assert!(restored.campaign_history.is_none());
    }
}


#[cfg(test)]
mod historical_interaction_boundary_regression {
    use super::*;

    #[test]
    fn ordinary_battle_and_read_only_archive_do_not_count_as_history() {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::CaoCao, 7, Difficulty::Warlords, "interaction-empty".into(),
        );
        game.apply(crate::Action::Choose { index: 1 }).unwrap();
        let raw = game.to_json().unwrap();
        assert!(game.historical_interactions().unwrap().is_empty());
        let report = game.historical_archive_report().unwrap();
        assert!(report.contains("有效历史触动保守下界：0次"));
        assert_eq!(game.to_json().unwrap(), raw);
        let restored = Game::from_json(&raw).unwrap();
        assert_eq!(restored.historical_interactions().unwrap(),
            game.historical_interactions().unwrap());
    }

    #[test]
    fn missing_immediate_resource_evidence_cannot_be_reported_as_zero() {
        let mut game = Game::new_with_campaign_difficulty(
            Hero::CaoCao, 7, Difficulty::Warlords, "interaction-missing".into(),
        );
        game.resource_history = None;
        assert!(game.historical_interactions().is_err());
    }
}


#[cfg(test)]
mod historical_map_effect_regression {
    use super::*;

    #[test]
    fn actual_jieting_overlay_counts_changed_reachable_lane_not_pending_flag() {
        // Projector fixture only; public archive additionally verifies the
        // complete action replay before using these receipts.
        let game = Game::new_with_campaign_difficulty(
            Hero::ZhugeLiang, 7, Difficulty::Warlords, "map-receipt".into(),
        );
        let start = CampaignState::current(&game);
        let mut scene = start.clone();
        scene.floor = 25;
        scene.stage = Stage::Story;
        scene.jieting_plan = Some(JietingPlan::Council);
        let mut map_state = scene.clone();
        map_state.stage = Stage::Map;
        map_state.jieting_plan = Some(JietingPlan::RetreatPending);
        let mut layout = map::ActMap::generate_expanded(|_| 0, 3);
        layout.rows[1] = [Node::Story, Node::Camp, Node::Market];
        map_state.route_lanes = vec![0, 1, 2];
        map_state.route = vec![Node::Story, Node::Camp, Node::Market];
        map_state.map_lane = None;
        map_state.act_map = Some(layout);
        assert!(!jieting_route_changed(&map_state), "pending plan alone is not an effect");
        map_state.route[0] = Node::Battle;
        assert!(jieting_route_changed(&map_state));
        let journal = vec![
            CampaignEvent::Started { state: start.clone() },
            transition(&start, &scene, 1).unwrap(),
            transition(&scene, &map_state, 2).unwrap(),
            transition(&map_state, &map_state, 3).unwrap(),
        ];
        assert_eq!(jieting_map_effect_floors(&journal, 2).unwrap(), vec![26]);
        map_state.route_lanes.swap(0, 1);
        assert!(!jieting_route_changed(&map_state), "unrelated lane permutation is not evidence");
    }
}


#[cfg(test)]
mod standalone_counter_projection_tests {
    use super::*;
    #[test]
    fn only_named_positive_historical_counter_sites_are_eligible() {
        use crate::EventKind;
        assert_eq!(standalone_counter_site(16, &EventKind::PowerTriggered,
            "yuanshao_escort_severed"), Some(("battle:guandu_escort", "wuchao")));
        assert_eq!(standalone_counter_site(40, &EventKind::DamageDealt,
            "yinping_interception"), Some(("battle:yinping_counter", "jieting")));
        assert_eq!(standalone_counter_site(40, &EventKind::PowerTriggered,
            "yinping_vanguard_broken"), Some(("battle:yinping_counter", "jieting")));
        assert_eq!(standalone_counter_site(15, &EventKind::PowerTriggered,
            "yuanshao_escort_severed"), None);
        assert_eq!(standalone_counter_site(40, &EventKind::DamageDealt, "hero"), None);
        assert_eq!(standalone_counter_site(40, &EventKind::EnemyTechnique,
            "yinping_interception"), None);
    }

    #[test]
    fn historical_entry_grants_do_not_mask_later_active_counter_receipts() {
        for (entry, counter, kind) in [
            ("wuchao_tyrant_fortified", "wuchao_tyrant_supply_saved", "wuchao"),
            ("wuchao_naval_raid_disruption", "wuchao_naval_raid_countered", "wuchao"),
            ("chibi_naval_fire_spread", "chibi_naval_fire_contained", "chibi"),
            ("chibi_fourth_act_fire", "chibi_luxun_firebreak", "chibi"),
            ("jieting_hill_abandons_wells", "jieting_hill_recaptured_wells", "jieting"),
            ("jieting_yinping_wells", "jieting_yinping_water_screen", "jieting"),
        ] {
            let first = historical_effect_site(entry).unwrap();
            let later = historical_effect_site(counter).unwrap();
            assert_eq!(first.0, kind);
            assert_eq!(later.0, kind);
            assert_ne!(first.1, later.1);
        }
        assert_eq!(historical_effect_site("chibi_luxun_firebreak"),
            historical_effect_site("chibi_lumeng_counterattack"));
    }
}

#[cfg(test)]
mod untriggered_density_reason_tests {
    use super::*;

    #[test]
    fn density_report_explains_declined_story_missed_council_and_alternate_boss_reasons() {
        // Real replayed Story decline -> Switch -> Verify density report.
        let mut decliner = Game::new_with_campaign_difficulty(
            Hero::ZhugeLiang, 7, Difficulty::Warlords, "untriggered-decline".into(),
        );
        let mut saw_declined_story = false;
        for _ in 0..80 {
            if matches!(decliner.stage, Stage::Won | Stage::Lost) { break; }
            let action = match decliner.stage {
                Stage::Map => {
                    let pick = decliner.route.iter().position(|n| *n == Node::Story)
                        .map_or(1, |i| i + 1);
                    crate::Action::Choose { index: pick }
                }
                Stage::Story => {
                    if decliner.lore_current.is_some() { saw_declined_story = true; }
                    crate::Action::Choose { index: 3 }
                }
                Stage::Reward => crate::Action::Choose { index: 0 },
                Stage::Camp => crate::Action::Choose { index: 1 },
                Stage::Market => crate::Action::Choose { index: 0 },
                Stage::Ancient => crate::Action::Choose { index: 1 },
                Stage::Combat => {
                    let b = decliner.battle.as_ref().unwrap();
                    if let Some(i) = b.hand.iter().position(|c|
                        crate::content::card_spec(&c.id).is_some_and(|s|
                            !matches!(s.effect, crate::content::Effect::Unplayable) && s.cost <= b.energy))
                    {
                        crate::Action::Play { index: i + 1, target: 1 }
                    } else {
                        crate::Action::EndTurn
                    }
                }
                Stage::Won | Stage::Lost => break,
            };
            decliner.apply(action).unwrap();
            if saw_declined_story && decliner.stage == Stage::Map { break; }
        }
        assert!(saw_declined_story, "ZhugeLiang seed 7 should encounter a Story scene");
        let declined_report = decliner.historical_density_report().unwrap();
        assert!(declined_report.untriggered_reasons.iter().any(|r| r.category == "declined_story"),
            "declined report: {:?}", declined_report.untriggered_reasons);

        // Unit verification for alternate boss branch, target not reached, and missed council.
        let mut branch_game = Game::new_with_campaign_difficulty(
            Hero::ZhugeLiang, 7, Difficulty::Warlords, "untriggered-branches".into(),
        );
        branch_game.apply(crate::Action::SetExpandedVariants {
            act1: 0, act2: 1, act3: 1, act4: 0, act5: 1,
        }).unwrap();
        branch_game.floor = 40;
        branch_game.stage = Stage::Lost;
        let decisions = vec![
            HistoricalDecision { id: "d1".into(), revision: 2, floor: 17, kind: "empty_city".into() },
            HistoricalDecision { id: "d2".into(), revision: 3, floor: 33, kind: "star_lantern".into() },
            HistoricalDecision { id: "d3".into(), revision: 4, floor: 10, kind: "mist_arrows".into() },
        ];
        let reasons = branch_game.collect_untriggered_reasons(&decisions, &[], &[]).unwrap();
        assert!(reasons.iter().any(|r| r.key == "council:chibi" && r.category == "missed_council"));
        assert!(reasons.iter().any(|r| r.key == "council:jieting" && r.category == "missed_council"));
        assert!(reasons.iter().any(|r| r.key == "decision:empty_city" && r.category == "alternate_boss_branch"));
        assert!(reasons.iter().any(|r| r.key == "decision:star_lantern" && r.category == "alternate_boss_branch"));
        assert!(reasons.iter().any(|r| r.key == "decision:mist_arrows" && r.category == "route_avoided_site"));

        // When history visited floor-19 Battle, mist_arrows explains counter_condition_unmet.
        let mut s0 = CampaignState::current(&branch_game);
        s0.floor = 0;
        s0.stage = Stage::Map;
        let mut s19 = s0.clone();
        s19.floor = 19;
        s19.stage = Stage::Combat;
        s19.last_node = Some(Node::Battle);
        let hist = vec![
            CampaignEvent::Started { state: s0 },
            CampaignEvent::Changed { revision: 1, state: Some(s19) },
        ];
        let visited_reasons = branch_game.collect_untriggered_reasons(&decisions, &[], &hist).unwrap();
        assert!(visited_reasons.iter().any(|r| r.key == "decision:mist_arrows" && r.category == "counter_condition_unmet"));
    }
}
