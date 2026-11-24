//! Action-boundary authority for noncombat campaign state. No action replay.
//! Each successful revision is recorded; unchanged revisions carry no snapshot.
//! This covers map, route, narrative and inventory, but not full Game sourcing.
use crate::{Game, Stage, Node, Card, WuchaoPlan, ChibiPlan, JietingPlan, RunMode, MAX_FLOOR, EXPANDED_MAX_FLOOR, map, archetypes, challenges, difficulty::Difficulty, content::Hero};
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
            jieting_plan: game.jieting_plan,
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


/**
 * Stable committed decision candidates, including the three councils and
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
        "wuchao_coalition_raid" | "wuchao_coalition_fortified" | "wuchao_coalition_unchecked" =>
            Some(("wuchao", "guandu")),
        "wuchao_tyrant_raid" | "wuchao_tyrant_fortified" | "wuchao_tyrant_unchecked"
        | "wuchao_tyrant_supply_saved" => Some(("wuchao", "dongzhuo")),
        "wuchao_naval_raid_disruption" | "wuchao_naval_supply_screen"
        | "wuchao_naval_supply_shortage" | "wuchao_naval_raid_countered" =>
            Some(("wuchao", "naval")),
        "chibi_naval_fleet_screen" | "chibi_naval_fire_spread"
        | "chibi_naval_declined_cover" | "chibi_naval_fire_contained" =>
            Some(("chibi", "naval")),
        "chibi_fourth_act_fire" | "chibi_fourth_act_fleet" | "chibi_fourth_act_declined"
        | "chibi_luxun_firebreak" | "chibi_lumeng_counterattack" =>
            Some(("chibi", "fourth_act_boss")),
        "jieting_hill_abandons_wells" | "jieting_hill_suppression"
        | "jieting_hill_recaptured_wells" | "jieting_water_wells_repaired"
        | "jieting_retreat_water_salvaged" => Some(("jieting", "water")),
        "jieting_jiangwei_wells" | "jieting_jiangwei_hills" | "jieting_jiangwei_retreat" =>
            Some(("jieting", "jiangwei")),
        "jieting_yinping_wells" | "jieting_yinping_hills" | "jieting_yinping_retreat"
        | "jieting_late_water_settlement" | "jieting_yinping_water_screen"
        | "jieting_yinping_retreat_command" => Some(("jieting", "yinping")),
        "mist_arrows_naval_counter" => Some(("mist_arrows", "naval")),
        "empty_city_scheme_countered" => Some(("empty_city", "third_act_boss")),
        "jingzhou_vigilance_countered" => Some(("jingzhou_warning", "fourth_act_boss")),
        "fancheng_flood_counter" | "fancheng_flood_exposure" =>
            Some(("fancheng_flood_record", "fancheng_flood")),
        "wuzhang_morale_recovered" => Some(("star_lantern", "northern_expedition")),
        _ => None,
    }
}

impl Game {
    /// Offline diagnostic. Replays the committed actions and checks the exact
    /// event stream and final snapshot before associating a real positive
    /// battlefield effect with an earlier historical decision. No new save
    /// fields, RNG draws or journal mutations; old unverified saves cannot
    /// manufacture effect receipts. This deliberately covers only named sites.
    pub fn historical_effect_receipts(&self) -> Result<Vec<HistoricalEffectReceipt>, String> {
        if self.difficulty.is_none() { return Ok(Vec::new()); }
        let history = self.campaign_history.as_ref()
            .ok_or("missing campaign journal for historical effect receipts")?;
        let decisions = historical_decisions(history)?;
        let trace = self.reconstruct_action_event_trace()?;
        let mut seen = HashSet::new();
        let mut receipts = Vec::new();
        for action in trace {
            for record in action.events {
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
                    source: record.event.source,
                    amount: record.event.amount,
                });
            }
        }
        Ok(receipts)
    }

    /// Player-facing archive, derived from the same replay-verified receipts
    /// rather than a second game truth. It is a conservative covered subset,
    /// not a full SPEC density measurement or a historical factual claim.
    pub fn historical_archive_report(&self) -> Result<String, String> {
        if self.difficulty.is_none() {
            return Ok("旧制战役保留原演义纪事；新制战史档案不适用。".into());
        }
        let receipts = self.historical_effect_receipts()?;
        let history = self.campaign_history.as_ref().ok_or("缺少战役决策档案")?;
        let decisions = historical_decisions(history)?;
        let effective = receipts.iter().map(|r| r.decision_id.as_str())
            .collect::<HashSet<_>>().len();
        let mut lines = vec![format!(
            "战史档案（动作重放已核对）：已记录{}项战术选择，{}项选择有实际战场后果，共{}个去重后的战场兑现点。",
            decisions.len(), effective, receipts.len()
        ), "仅统计已覆盖的历史战术；不等于完整12–18次有效触动指标。小说章回为文学背景，战术数值与反制属于游戏原创推演。".into()];
        for decision in &decisions {
            let (chapter, title) = match decision.kind.as_str() {
                "wuchao" => (30, "官渡乌巢"),
                "chibi" => (49, "赤壁东风"),
                "jieting" => (95, "街亭军议"),
                flag => crate::lore::BEATS.iter().find(|b| b.flag == flag)
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
        if decisions.is_empty() { lines.push("尚无已覆盖的历史战术选择。".into()); }
        Ok(lines.join("\n"))
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
