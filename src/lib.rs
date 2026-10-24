//! UI-free deterministic game engine. Safe to reuse from Swift/UniFFI and wasm.
//! Every authoritative mutation goes through Game::apply(Action).
pub mod archetypes;
pub mod builds;
pub mod combat;
pub mod event_stack;
pub mod resource_events;
pub mod rng_events;
pub mod combat_resource_events;
pub mod foe_board_events;
pub mod card_zone_events;
pub mod combat_effect_events;
mod action_commit;
mod replay;
pub mod challenges;
pub mod content;
pub mod items;
pub mod lore;
pub mod map;
pub mod progression;
pub mod persistence;

use combat::{CombatEvent, EventKind, MAX_RECENT_EVENTS, PowerId, PowerStack};
use challenges::ChallengeModifier;
use content::{Effect, Hero, REWARD_POOL, NEUTRAL_POOL, hero_pool, card_spec, enemy_name, enemy_hp};
use serde::{Deserialize, Serialize};

pub const SAVE_VERSION: u32 = 2;
pub const MAX_FLOOR: u8 = 21;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunMode {
    #[default] Standard,
    Daily,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub upgraded: bool,
    #[serde(default)] pub enchantment: Option<String>,
    #[serde(default)] pub quest_progress: u8,
    #[serde(default)] pub temporary: bool,
}
impl Card {
    pub fn new(id: &str) -> Self {
        Self { id: id.into(), upgraded: false, enchantment: None, quest_progress: 0, temporary: false }
    }
    pub fn name(&self) -> String {
        let base = card_spec(&self.id).map_or("未知牌", |s| s.name);
        let upgraded = if self.upgraded { "+" } else { "" };
        let enchant = self.enchantment.as_ref().map_or(String::new(), |kind| format!("·{kind}"));
        let temporary = if self.temporary { "·临时" } else { "" };
        format!("{base}{upgraded}{enchant}{temporary}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage { Map, Combat, Reward, Camp, Market, Story, Ancient, Won, Lost }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Node { Battle, Elite, Camp, Market, Story, Boss }
impl Node {
    pub fn name(self) -> &'static str {
        match self {
            Self::Battle => "遭遇战", Self::Elite => "精英战", Self::Camp => "营寨",
            Self::Market => "军市", Self::Story => "奇遇", Self::Boss => "关底统帅",
        }
    }
}

/// Boss signatures are serialized as intent data, not inferred only at execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BossMove {
    Thunder(i32),      // 张角：雷咒，实际伤血后虚弱
    TigerRush(i32),    // 孙坚：多段虎袭
    TyrantLevy(i32),   // 董卓：进攻兼军资劫掠
    GrandAlliance(i32),// 袁绍：在场友军列阵护卫
    HiddenScheme(u8),  // 司马懿：削减下回合军令
    Mirage(i32),       // 左慈：回复并立盾
}
impl BossMove {
    pub fn label(&self) -> String {
        match self {
            Self::Thunder(n) => format!("雷咒 {n}伤害，伤血施加虚弱"),
            Self::TigerRush(n) => format!("虎袭 {n}×2（进阶3段）"),
            Self::TyrantLevy(n) => format!("暴政 {n}伤害，并掠夺金钱"),
            Self::GrandAlliance(n) => format!("盟军列阵 全军护甲+{n}"),
            Self::HiddenScheme(n) => format!("暗度陈仓 下回合军令-{n}"),
            Self::Mirage(n) => format!("幻身遁甲 回复{n}并获得护甲"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    Attack(i32),
    MultiAttack(i32, u8),
    Hex(u8, u8),
    Guard(i32),
    Rally(i32),
    Assist(i32),
    Boss(BossMove),
}
impl Intent {
    pub fn label(&self) -> String {
        match self {
            Self::Attack(n) => format!("进攻 {n}"),
            Self::MultiAttack(n, hits) => format!("连击 {n}×{hits}"),
            Self::Hex(weak, vuln) => format!("诡计 虚弱{weak}/易伤{vuln}"),
            Self::Guard(n) => format!("防御 {n}"),
            Self::Rally(n) => format!("聚势 +{n}"),
            Self::Assist(n) => format!("协同护阵 友军护甲+{n}"),
            Self::Boss(action) => action.label(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Foe {
    pub id: String,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub strength: i32,
    pub poison: i32,
    pub vulnerable: u8,
    #[serde(default)]
    pub weak: u8,
    pub intent: Intent,
    #[serde(default)] pub phase: u8,
}
impl Foe {
    pub fn name(&self) -> &'static str { enemy_name(&self.id) }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Battle {
    pub foes: Vec<Foe>,
    pub draw: Vec<Card>,
    pub hand: Vec<Card>,
    pub discard: Vec<Card>,
    pub exhausted: Vec<Card>,
    pub energy: i32,
    pub block: i32,
    pub strength: i32,
    pub weak: u8,
    #[serde(default)] pub vulnerable: u8,
    #[serde(default)] pub momentum: i32,
    #[serde(default)] pub scheme: i32,
    #[serde(default)] pub combo: i32,
    #[serde(default)] pub command: i32,
    #[serde(default)] pub fury: i32,
    #[serde(default)] pub thorns: i32,
    #[serde(default)] pub played: u32,
    #[serde(default)] pub played_this_turn: u8,
    #[serde(default)] pub last_turn_cards: u8,
    #[serde(default)] pub next_turn_energy_penalty: u8,
    #[serde(default)] pub troops_hp: i32,
    #[serde(default)] pub troops_attack: i32,
    #[serde(default)] pub discard_tokens: i32,
    #[serde(default)] pub powers: Vec<PowerStack>,
    #[serde(default)] pub events: Vec<CombatEvent>,
    #[serde(default)] pub event_seq: u64,
    #[serde(default, skip_serializing_if = "event_stack::Stack::is_empty")] pub reactions: event_stack::Stack,
    #[serde(default)] pub thought_used: bool,
    pub turn: u32,
}

impl Battle {
    /// Poll events after a *current-battle* sequence number. Never mutates RNG or state.
    /// Gaps from the 80-event ring are reported, not silently hidden.
    pub fn events_after(&self, cursor: u64, limit: usize) -> combat::EventPage<'_> {
        let latest = self.event_seq;
        let oldest_available = self.events.first()
            .map_or(latest.saturating_add(1), |event| event.seq);
        if cursor > latest {
            return combat::EventPage {
                entries: &[], oldest_available, latest, missed: 0,
                next_cursor: latest, has_more: false, reset: true,
            };
        }
        let start = self.events.partition_point(|event| event.seq <= cursor);
        let end = start.saturating_add(limit.clamp(1, MAX_RECENT_EVENTS)).min(self.events.len());
        let entries = &self.events[start..end];
        let missed = oldest_available.saturating_sub(cursor.saturating_add(1));
        let next_cursor = entries.last().map_or(latest.max(cursor), |event| event.seq);
        combat::EventPage {
            entries, oldest_available, latest, missed, next_cursor,
            has_more: end < self.events.len(), reset: false,
        }
    }
}

/// Lifetime-of-run counters. Fields are monotonic and old V2 saves default to zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CombatCounters {
    pub battles_started: u64,
    pub battles_won: u64,
    pub battles_lost: u64,
    pub turns_completed: u64,
    pub cards_played: u64,
    pub hero_hp_damage: u64,
    pub troop_attacks: u64,
    pub troop_damage: u64,
    pub troop_absorbed: u64,
    pub command_procs: u64,
    pub command_energy: u64,
    pub command_block: u64,
    pub sun_discards: u64,
    pub sun_discard_block: u64,
    pub sun_combo_procs: u64,
    pub sun_combo_block: u64,
    pub sun_combo_draws: u64,
}

impl CombatCounters {
    pub fn accumulate(&mut self, other: &Self) {
        self.battles_started = self.battles_started.saturating_add(other.battles_started);
        self.battles_won = self.battles_won.saturating_add(other.battles_won);
        self.battles_lost = self.battles_lost.saturating_add(other.battles_lost);
        self.turns_completed = self.turns_completed.saturating_add(other.turns_completed);
        self.cards_played = self.cards_played.saturating_add(other.cards_played);
        self.hero_hp_damage = self.hero_hp_damage.saturating_add(other.hero_hp_damage);
        self.troop_attacks = self.troop_attacks.saturating_add(other.troop_attacks);
        self.troop_damage = self.troop_damage.saturating_add(other.troop_damage);
        self.troop_absorbed = self.troop_absorbed.saturating_add(other.troop_absorbed);
        self.command_procs = self.command_procs.saturating_add(other.command_procs);
        self.command_energy = self.command_energy.saturating_add(other.command_energy);
        self.command_block = self.command_block.saturating_add(other.command_block);
        self.sun_discards = self.sun_discards.saturating_add(other.sun_discards);
        self.sun_discard_block = self.sun_discard_block.saturating_add(other.sun_discard_block);
        self.sun_combo_procs = self.sun_combo_procs.saturating_add(other.sun_combo_procs);
        self.sun_combo_block = self.sun_combo_block.saturating_add(other.sun_combo_block);
        self.sun_combo_draws = self.sun_combo_draws.saturating_add(other.sun_combo_draws);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnTelemetry {
    pub floor: u8,
    pub turn: u32,
    pub completed: bool,
    pub hero_hp_after: i32,
    pub troop_hp_after: i32,
    pub counters: CombatCounters,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CombatTelemetry {
    pub totals: CombatCounters,
    pub current_turn: CombatCounters,
    /// Bounded diagnostic trace, not a full event-sourcing replay.
    pub recent_turns: Vec<TurnTelemetry>,
}
pub const MAX_RECENT_TURN_METRICS: usize = 24;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub version: u32,
    pub revision: u64,
    pub seed: u64,
    pub rng_state: u64,
    /// Action-boundary RNG snapshots; absent in old V2 saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rng_history: Option<Vec<rng_events::RngEvent>>,
    pub hero: Hero,
    pub hp: i32,
    pub max_hp: i32,
    pub gold: i32,
    pub floor: u8,
    pub stage: Stage,
    pub last_node: Option<Node>,
    pub route: Vec<Node>,
    pub deck: Vec<Card>,
    pub relics: Vec<String>,
    pub potions: u8,
    pub battle: Option<Battle>,
    /// Complete authoritative successful-action journal for new campaigns.
    /// Old V2 saves lack a journal and remain playable with None.
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub action_history: Option<Vec<Action>>,
    /// Full ordered combat event stream for new campaigns; None for legacy saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_history: Option<Vec<event_stack::RunEvent>>,
    /// Versioned, action-boundary authority journal for persistent HP/max HP/gold.
    /// Old V2 saves omit it; missing history is never fabricated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_history: Option<Vec<resource_events::ResourceEvent>>,
    /// Per-action combat energy/block/turn transitions; absent in legacy V2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle_resource_history: Option<Vec<combat_resource_events::BattleResourceEvent>>,
    /// Full committed enemy board at each action boundary; absent in legacy V2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foe_board_history: Option<Vec<foe_board_events::FoeBoardEvent>>,
    /// Ordered card zones at successful action boundaries; absent in old V2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_zone_history: Option<Vec<card_zone_events::CardZoneEvent>>,
    /// Action-boundary power stacks and dynamic listeners; absent in old V2 saves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_effect_history: Option<Vec<combat_effect_events::CombatEffectEvent>>, 
    /// Custom-only stable difficulty switches; omitted in old V2 saves.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ChallengeModifier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doctrine: Option<archetypes::Doctrine>,
    #[serde(default)] pub telemetry: CombatTelemetry,
    pub rewards: Vec<Card>,
    pub market: Vec<Card>,
    pub log: Vec<String>,
    #[serde(default)] pub ascension: u8,
    #[serde(default)] pub event_id: u8,
    #[serde(default)] pub potion_bag: Vec<String>,
    #[serde(default)] pub shop_relic: Option<String>,
    #[serde(default)] pub relic_history: Vec<String>,
    #[serde(default)] pub victories: u32,
    #[serde(default)] pub boons: Vec<String>,
    #[serde(default)] pub boon_choices: Vec<String>,
    #[serde(default)] pub variants: Option<[u8; 3]>,
    #[serde(default)] pub mode: RunMode,
    #[serde(default)] pub run_id: String,
    /// Opt-in receipt for CLI campaigns whose separate chronicle start can be
    /// recovered after save.json committed but chronicle.json did not.
    /// Older V2 snapshots default to false and are never re-counted on load.
    #[serde(default)] pub profile_start_receipt: bool,
    #[serde(default)] pub act_map: Option<map::ActMap>,
    #[serde(default)] pub map_lane: Option<u8>,
    #[serde(default)] pub route_lanes: Vec<u8>,
    #[serde(default)] pub lore_flags: Vec<String>,
    #[serde(default)] pub lore_seen: Vec<String>,
    #[serde(default)] pub lore_current: Option<String>,
    #[serde(default)] pub easter_unlocked: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Choose { index: usize },
    Play { index: usize, target: usize },
    PlaySelected { index: usize, target: usize, selection: usize },
    EndTurn,
    UsePotion,
    UsePotionAt { slot: usize, target: usize },
    Upgrade { index: usize },
    Remove { index: usize },
    Enchant { index: usize, kind: String },
    Discard { index: usize },
    SetVariants { act1: u8, act2: u8, act3: u8 },
    SetDoctrine { id: String },
    SetModifier { id: String, enabled: bool },
    /// Explicit, replayable modding action. Only available in unranked custom battles.
    RegisterReaction { listener: event_stack::Listener },
    /// Remove a listener by its stable ID. Unranked custom battles only.
    UnregisterReaction { id: String },
}

impl Game {
    pub fn new(hero: Hero, seed: u64) -> Self { Self::new_with_ascension(hero, seed, 0) }
    pub fn new_with_mode(hero: Hero, seed: u64, ascension: u8, mode: RunMode) -> Self {
        Self::new_with_identity(hero, seed, ascension, mode, String::new())
    }
    pub fn new_with_identity(hero: Hero, seed: u64, ascension: u8, mode: RunMode, run_id: String) -> Self {
        let mut game = Self::new_with_ascension(hero,seed,ascension);
        game.mode = mode;
        game.run_id = run_id;
        game
    }
    pub fn new_with_ascension(hero: Hero, seed: u64, ascension: u8) -> Self {
        let mut deck = vec![];
        for _ in 0..5 { deck.push(Card::new("strike")); }
        for _ in 0..4 { deck.push(Card::new("defend")); }
        deck.push(Card::new(hero.signature()));
        let max_hp = hero.max_hp();
        let mut game = Self {
            version: SAVE_VERSION, revision: 0, seed, rng_state: seed.max(1),
            rng_history: Some(vec![rng_events::RngEvent::Started { state: seed.max(1) }]),
            hero, hp: max_hp, max_hp, gold: 90, floor: 0, stage: Stage::Map,
            last_node: None, route: vec![], deck: deck.clone(), relics: vec!["军旗".into()],
            potions: 1, battle: None, action_history: Some(vec![]), event_history: Some(vec![]),
            resource_history: Some(vec![resource_events::ResourceEvent::Started {
                state: resource_events::Resources { hp: max_hp, max_hp, gold: 90 },
            }]), battle_resource_history: Some(vec![combat_resource_events::BattleResourceEvent::Started]),
            foe_board_history: Some(vec![foe_board_events::FoeBoardEvent::Started]),
            card_zone_history: Some(vec![card_zone_events::CardZoneEvent::Started { deck }]),
            combat_effect_history: Some(vec![combat_effect_events::CombatEffectEvent::Started]),
            modifiers: vec![], doctrine: None, telemetry: CombatTelemetry::default(), rewards: vec![], market: vec![],
            log: vec![format!("{}率部出征，征途由此开始。", hero.name())],
            ascension: ascension.min(10), event_id: 0,
            potion_bag: vec!["healing".into()], shop_relic: None,
            relic_history: vec!["军旗".into()], victories: 0,
            boons: vec![], boon_choices: vec![], variants: None,
            mode: RunMode::Standard, run_id: String::new(), profile_start_receipt: false,
            act_map: None, map_lane: None, route_lanes: Vec::new(),
            lore_flags: vec![], lore_seen: vec![], lore_current: None,
            easter_unlocked: Vec::new(),
        };
        game.refresh_route();
        game
    }
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        let mut g: Self = serde_json::from_str(raw).map_err(|e| format!("存档解析失败: {e}"))?;
        if g.version == 1 {
            g.version = SAVE_VERSION;
            if g.potion_bag.is_empty() {
                for _ in 0..g.potions { g.potion_bag.push("healing".into()); }
            }
            if g.relic_history.is_empty() { g.relic_history = g.relics.clone(); }
        }
        if g.version != SAVE_VERSION {
            return Err(format!("存档版本 {} 不兼容当前版本 {}", g.version, SAVE_VERSION));
        }
        g.validate()?;
        Ok(g)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.hp < 0 || self.hp > self.max_hp || self.max_hp <= 0 || self.gold < 0 || self.floor > MAX_FLOOR
            || self.ascension > 10 || self.potions > 3 || self.potion_bag.len() > 3
            || usize::from(self.potions) != self.potion_bag.len()
            || self.potion_bag.iter().any(|id| !items::POTIONS.iter().any(|(valid,_)| valid == id))
        {
            return Err("存档属性无效".into());
        }
        if self.action_history.as_ref().is_some_and(|actions|actions.len()>50_000) {
            return Err("操作日志长度超限".into());
        }
        if self.rng_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("rng journal exceeds action limit".into());
        }
        if self.resource_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("resource journal exceeds action limit".into());
        }
        if self.battle_resource_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("combat resource journal exceeds action limit".into());
        }
        if self.foe_board_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("enemy board journal exceeds action limit".into());
        }
        if self.card_zone_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("card zone journal exceeds action limit".into());
        }
        if self.combat_effect_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("combat effect journal exceeds action limit".into());
        }
        if self.profile_start_receipt && self.run_id.is_empty() {
            return Err("可恢复战功凭证缺少唯一战役ID".into());
        }
        if (self.stage == Stage::Combat) != self.battle.is_some() {
            return Err("存档战斗阶段与战斗快照不一致".into());
        }
        if (self.mode != RunMode::Custom && !self.modifiers.is_empty())
            || self.modifiers.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err("挑战修饰符仅适用于自由模式，并且不能重复或乱序".into());
        }
        if self.doctrine.is_some_and(|focus| focus.hero() != self.hero) {
            return Err("当前战法与武将不匹配".into());
        }
        if self.variants.is_some_and(|settings| settings.iter().any(|n| *n > 1)) {
            return Err("章节变体必须为0或1".into());
        }
        if self.map_lane.is_some_and(|lane| lane >= map::LANES)
            || self.route_lanes.iter().any(|lane| *lane >= map::LANES)
            || self.act_map.as_ref().is_some_and(|graph| !graph.is_valid())
        { return Err("非法战役地图状态".into()); }
        if !self.route_lanes.is_empty() && self.route_lanes.len() != self.route.len() {
            return Err("地图显示路线与可达车道数量不一致".into());
        }
        if self.easter_unlocked.iter().any(|id| !lore::EASTER_EGGS.iter().any(|egg| egg.id == id))
        { return Err("未知演义彩蛋".into()); }
        if self.lore_flags.iter().any(|flag| !lore::BEATS.iter().any(|b| b.flag == flag))
            || self.lore_seen.iter().any(|id| lore::beat(id).is_none())
            || self.lore_current.as_deref().is_some_and(|id| lore::beat(id).is_none())
        { return Err("无效人物羁绊与剧情存档".into()); }
        if self.lore_current.is_some() && self.stage != Stage::Story {
            return Err("仅奇遇场景可拥有当前演义剧情".into());
        }
        for boon in self.boons.iter().chain(self.boon_choices.iter()) {
            if !items::BOONS.iter().any(|(id,_)| id == boon) { return Err(format!("未知赐福: {boon}")); }
        }
        for c in self.deck.iter().chain(self.rewards.iter()).chain(self.market.iter()) {
            if c.temporary { return Err("永久卡牌不能携带临时标志".into()); }
            if card_spec(&c.id).is_none() { return Err(format!("未知卡牌: {}", c.id)); }
            if c.enchantment.as_deref().is_some_and(|id|
                !matches!(id, "迅捷" | "锐锋" | "护身" | "明悟")) {
                return Err(format!("未知附魔: {:?}", c.enchantment));
            }
        }
        if self.telemetry.recent_turns.len() > MAX_RECENT_TURN_METRICS
            || self.telemetry.recent_turns.iter().any(|entry|
                entry.floor > MAX_FLOOR || entry.hero_hp_after < 0 || entry.troop_hp_after < 0)
        { return Err("战斗统计记录无效".into()); }
        if let Some(history)=&self.event_history {
            if history.len()>1_000_000 {return Err("event history exceeds size limit".into());}
            event_stack::verify_record(history)?;
        }
        if let Some(b) = &self.battle {
            if let Some(archive)=&self.event_history {
                // A complete archive must agree byte-for-byte with any retained
                // current-battle audit window; sequence numbers alone are insufficient.
                if !b.events.is_empty() {
                    let start=archive.len().checked_sub(b.events.len())
                        .ok_or("event archive shorter than battle window")?;
                    if !archive[start..].iter().zip(&b.events).all(|(entry,event)|
                        entry.encounter==self.telemetry.totals.battles_started
                        && entry.floor==self.floor && entry.event==*event) {
                        return Err("battle window differs from event archive".into());
                    }
                }
            }
            if !b.reactions.valid() {return Err("invalid listener stack".into());}
            if self.event_history.as_ref().is_some_and(|history|
                b.event_seq>0 && history.last().is_none_or(|e|
                    e.encounter!=self.telemetry.totals.battles_started
                    || e.event.seq!=b.event_seq))
            {return Err("current battle and event archive disagree".into());}
            for c in b.draw.iter().chain(b.hand.iter()).chain(b.discard.iter()).chain(b.exhausted.iter()) {
                if card_spec(&c.id).is_none() { return Err(format!("未知战斗卡牌: {}", c.id)); }
            }
            if b.energy < 0 || b.block < 0 || b.troops_hp < 0 || b.troops_attack < 0
                || b.discard_tokens < 0 || b.foes.is_empty() || b.foes.len() > 4
                || b.next_turn_energy_penalty > 2
                || b.powers.len() > 8 || b.powers.iter().any(|power| power.stacks <= 0 || power.stacks > 99)
                || b.powers.iter().enumerate().any(|(idx,p)| b.powers[..idx].iter().any(|prior| prior.id == p.id))
                || b.events.len() > MAX_RECENT_EVENTS
                || b.events.iter().any(|event| event.seq == 0 || event.seq > b.event_seq)
                || b.events.windows(2).any(|pair| pair[0].seq.checked_add(1) != Some(pair[1].seq))
                || b.events.last().is_some_and(|event| event.seq != b.event_seq)
                || b.momentum < 0 || b.scheme < 0 || b.combo < 0 || b.command < 0 || b.fury < 0
                || b.thorns < 0 || b.foes.iter().any(|f| f.max_hp <= 0 || f.hp < 0 || f.hp > f.max_hp || f.block < 0 || f.poison < 0 || f.phase > 1)
            { return Err("无效战斗数据".into()); }
        }
        Ok(())
    }
    fn random(&mut self, upper: usize) -> usize {
        if upper <= 1 { return 0; }
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) % upper as u64) as usize
    }
    fn shuffle(&mut self, cards: &mut [Card]) {
        for i in (1..cards.len()).rev() {
            let j = self.random(i + 1);
            cards.swap(i, j);
        }
    }
    fn refresh_route(&mut self) {
        let next = self.floor + 1;
        if next > MAX_FLOOR {
            self.route.clear();
            self.route_lanes.clear();
            return;
        }
        let act = self.floor / 7;
        if self.act_map.as_ref().is_none_or(|map| map.act != act) {
            let generated = map::ActMap::generate(|bound| self.random(bound),act);
            self.act_map = Some(generated);
            self.map_lane = None;
        }
        let row = usize::from((next - 1) % 7);
        let choices = self.act_map.as_ref().expect("generated").routes(row,self.map_lane);
        self.route_lanes = choices.iter().map(|(lane,_)| *lane).collect();
        self.route = choices.iter().map(|(_,node)| *node).collect();
    }
    pub fn effective_doctrine(&self) -> Option<archetypes::Doctrine> {
        self.doctrine.or_else(|| archetypes::inferred(self.hero,&self.deck))
    }
    pub fn doctrine_status(&self) -> String {
        let options=archetypes::Doctrine::for_hero(self.hero);
        let selection=self.doctrine.map_or("自动识别".to_owned(),|d|d.label().to_owned());
        let effective=self.effective_doctrine().map_or("尚未形成".to_owned(),
            |d|format!("{}（{}）",d.label(),d.id()));
        format!("战法偏好：{selection}；当前流派：{effective}。可选：{}（{}）、{}（{}）、auto；仅在地图或营寨可切换。",
            options[0].label(),options[0].id(),options[1].label(),options[1].id())
    }
    fn pick_reward_cards(&mut self) -> Vec<Card> {
        let own = hero_pool(self.hero);
        let mut choices: Vec<Card> = vec![];
        while choices.len() < 3 {
            let roll = self.random(100);
            let id = if roll < 62 {
                own[self.random(own.len())]
            } else if roll < 85 {
                NEUTRAL_POOL[self.random(NEUTRAL_POOL.len())]
            } else {
                REWARD_POOL[self.random(REWARD_POOL.len())]
            };
            let card = Card::new(id);
            if !choices.iter().any(|c| c.id == card.id) { choices.push(card); }
        }
        // Draft guidance is optional: at most ONE of three randomly generated
        // rewards is exchanged, with a 70% roll. Two slots remain untouched.
        // Without an explicit preference or two distinct owned synergies this
        // method consumes exactly the legacy RNG sequence.
        if let Some(focus)=self.effective_doctrine()
            && !choices.iter().any(|card|focus.contains(&card.id))
            && self.random(100)<70
        {
            let candidates=focus.members().iter().copied()
                .filter(|id| !choices.iter().any(|c|c.id==*id)
                    && !self.deck.iter().any(|c|c.id==*id))
                .collect::<Vec<_>>();
            if !candidates.is_empty() {
                let pick=candidates[self.random(candidates.len())];
                let replaced=self.random(choices.len());
                choices[replaced]=Card::new(pick);
            }
        }
        choices
    }

    pub fn act_name(&self) -> &'static str {
        let act = self.floor.saturating_sub(1) / 7;
        let branch = self.act_variant(act);
        match (act.min(2), branch) {
            (0, 0) => "黄巾乱世", (0, _) => "江东烽烟",
            (1, 0) => "关中风云", (1, _) => "群雄会盟",
            (2, 0) => "冢虎之谋", _ => "奇门斗法",
        }
    }
    fn act_variant(&self, act: u8) -> u64 {
        self.variants.map_or(
            self.seed.rotate_right(act as u32 * 11) & 1,
            |v| u64::from(v[usize::from(act.min(2))]),
        )
    }
    fn generate_foes(&mut self, node: Node) -> Vec<Foe> {
        let act = self.floor.saturating_sub(1) / 7;
        let branch = self.act_variant(act);
        let ids: Vec<&str> = match node {
            Node::Boss => vec![match (act, branch) {
                (0,0) => "zhangjiao", (0,_) => "sunjian",
                (1,0) => "dongzhuo", (1,_) => "yuanshao",
                (2,0) => "simayi", _ => "zuoci",
            }],
            Node::Elite => {
                let elites = ["elite", "war_elephant", "royal_guard", "tiger_general", "sacred_beast"];
                vec![elites[self.random(elites.len())]]
            }
            _ => {
                let band = match (act, branch) {
                    (0,0) => ["bandit", "archer", "yellow_turban", "pyromancer", "spearman"],
                    (0,_) => ["archer", "cuirass", "sling", "raider", "crossbowman"],
                    (1,0) => ["officer", "raider", "wolf", "horse_archer", "cuirass"],
                    (1,_) => ["officer", "crossbowman", "tactician", "assassin", "iron_guard"],
                    (2,0) => ["shaman", "tomb_guard", "tactician", "witch", "panther"],
                    _ => ["shaman", "firelancer", "assassin", "witch", "iron_guard"],
                };
                let first = band[self.random(band.len())];
                if self.floor > 2 && self.random(100) < 40 {
                    let second = band[self.random(band.len())];
                    vec![first, second]
                } else { vec![first] }
            }
        };
        ids.into_iter().map(|id| {
            let hp = enemy_hp(id);
            Foe {
                id: id.to_string(), hp, max_hp: hp, block: 0, strength: 0,
                poison: 0, vulnerable: 0, weak: 0, intent: Intent::Attack(6), phase: 0,
            }
        }).collect()
    }
    fn roll_intents(&mut self) {
        let count = self.battle.as_ref().map_or(0, |b| b.foes.len());
        for i in 0..count {
            // Consume the same seed-driven roll for every enemy, including scripted
            // bosses, so RNG state remains reproducible through a saved turn.
            let roll = self.random(100);
            let b = self.battle.as_mut().expect("battle");
            let turn = b.turn;
            let last_turn_cards = b.last_turn_cards;
            let living = b.foes.iter().filter(|foe|foe.hp > 0).count();
            let foe = &mut b.foes[i];
            if foe.hp == 0 { continue; }
            let act = (self.floor.saturating_sub(1) / 7) as i32;
            let attack = 6 + act * 3 + foe.strength + self.ascension as i32 / 3
                + match foe.id.as_str() {
                    "dongzhuo" => 7, "simayi" => 6, "zuoci" => 5, "tiger_general" => 5,
                    "archer" | "assassin" => 1, _ => 0,
                };
            let spellcaster = matches!(foe.id.as_str(),
                "shaman" | "pyromancer" | "tactician" | "witch"
                | "zhangjiao" | "simayi" | "zuoci");
            let swift = matches!(foe.id.as_str(),
                "archer" | "crossbowman" | "horse_archer" | "assassin" | "panther");
            // Every boss has a predictable signature on the first turn and each
            // third turn. The two late-act schemers additionally counter card
            // spam on the next telegraphed turn.
            let signature = turn % 3 == 1
                || (foe.id == "simayi" && last_turn_cards >= 4)
                || (foe.id == "zuoci" && last_turn_cards >= 5);
            let second = i32::from(foe.phase > 0);
            let boss_move = if signature {
                match foe.id.as_str() {
                    "zhangjiao" => Some(BossMove::Thunder((attack-2).max(3)+second*2)),
                    "sunjian" => Some(BossMove::TigerRush((attack/2).max(3)+second)),
                    "dongzhuo" => Some(BossMove::TyrantLevy((attack-3).max(3)+second*2)),
                    "yuanshao" => Some(BossMove::GrandAlliance(7+act*2+second*4)),
                    "simayi" => Some(BossMove::HiddenScheme(1+foe.phase.min(1))),
                    "zuoci" => Some(BossMove::Mirage(7+act*2+second*3)),
                    _ => None,
                }
            } else { None };
            foe.intent = if let Some(action) = boss_move {
                Intent::Boss(action)
            } else if foe.phase > 0 && foe.id == "tiger_general" && turn.is_multiple_of(2) {
                Intent::MultiAttack((attack / 2).max(4), 3)
            } else if foe.phase > 0 && foe.id == "war_elephant" && turn.is_multiple_of(3) {
                Intent::Guard(12 + act * 3)
            } else if foe.phase > 0 && foe.id == "sacred_beast" && turn.is_multiple_of(3) {
                Intent::Hex(2, 1)
            } else if foe.phase > 0 && foe.id == "royal_guard" && living > 1 && turn.is_multiple_of(2) {
                Intent::Assist(8 + act * 2)
            } else if foe.phase > 0 && foe.id == "elite" && turn.is_multiple_of(3) {
                Intent::Rally(3)
            } else if living > 1
                && matches!(foe.id.as_str(),"officer"|"tactician"|"royal_guard")
                && turn % 3 == 2
            {
                Intent::Assist(4+act*2)
            } else if foe.phase > 0 && matches!(foe.id.as_str(), "sunjian" | "dongzhuo") {
                Intent::MultiAttack((attack / 2).max(3), 2)
            } else if spellcaster && (roll < 26 || turn.is_multiple_of(4)) {
                Intent::Hex(1 + u8::from(act > 0), u8::from(act > 0))
            } else if swift && roll < 45 {
                Intent::MultiAttack((attack / 2).max(2), 2 + u8::from(act > 0))
            } else if roll < 68 {
                Intent::Attack(attack)
            } else if roll < 86 {
                Intent::Guard(6 + act * 4 + self.ascension as i32 / 2)
            } else {
                Intent::Rally(2 + i32::from(act > 0))
            };
        }
    }

    fn draw_cards(&mut self, count: usize) {
        for _ in 0..count {
            if self.battle.as_ref().expect("battle").hand.len() >= 10 { return; }
            if self.battle.as_ref().expect("battle").draw.is_empty() {
                let mut discard = std::mem::take(&mut self.battle.as_mut().expect("battle").discard);
                if discard.is_empty() { return; }
                self.shuffle(&mut discard);
                self.battle.as_mut().expect("battle").draw = discard;
            }
            let drawn = {
                let b = self.battle.as_mut().expect("battle");
                let card = b.draw.pop();
                if let Some(card) = &card { b.hand.push(card.clone()); }
                card
            };
            if let Some(card) = drawn {
                self.combat_event(EventKind::CardDrawn,&card.id,1,Some("hero".into()));
            }
        }
    }
    fn start_battle(&mut self, node: Node) {
        let mut foes = self.generate_foes(node);
        for foe in &mut foes {
            let extra = (self.ascension as i32 / 2) * (1 + (self.floor as i32 / 7));
            foe.hp += extra;
            foe.max_hp += extra;
            if self.modifiers.contains(&ChallengeModifier::IronLegion) {
                foe.block = foe.block.saturating_add(8);
            }
        }
        let mut draw = self.deck.clone();
        self.shuffle(&mut draw);
        self.telemetry.current_turn = CombatCounters::default();
        self.record_metric(|c| c.battles_started = c.battles_started.saturating_add(1));
        self.battle = Some(Battle {
            foes, draw, hand: vec![], discard: vec![], exhausted: vec![],
            energy: 3 - i32::from(self.modifiers.contains(&ChallengeModifier::ThinSupplies)), block: 0, strength: if self.hero == Hero::GuanYu { 1 } else { 0 },
            weak: 0, vulnerable: 0, momentum: 0, scheme: 0,
            combo: 0, command: 0, fury: 0, thorns: 0, played: 0,
            played_this_turn: 0, last_turn_cards: 0, next_turn_energy_penalty: 0,
            troops_hp: if self.hero == Hero::CaoCao { 12 } else { 0 },
            troops_attack: if self.hero == Hero::CaoCao { 2 } else { 0 },
            discard_tokens: 0,
            powers: vec![], events: vec![], event_seq: 0, reactions: event_stack::Stack::default(), thought_used: false,
            turn: 1,
        });
        self.roll_intents();
        let extra_draw = usize::from(self.relics.iter().any(|r| r == "虎翼"));
        let boon_draw = usize::from(self.boons.iter().any(|b| b == "strategist")) * 2;
        self.draw_cards(5 + extra_draw + boon_draw);
        if let Some(b) = &mut self.battle {
            if self.relics.iter().any(|r| r == "铁骑") { b.block += 8; }
            if self.boons.iter().any(|id| id == "veteran") { b.block += 6; }
            if self.boons.iter().any(|id| id == "war_drum") { b.energy += 1; }
            if self.boons.iter().any(|id| id == "blood_oath") { b.strength += 2; }
            if self.boons.iter().any(|id| id == "formation") { b.command += 2; }
            if self.relics.iter().any(|r| r == "璇玑") { b.energy += 1; }
            if self.relics.iter().any(|r| r == "军旗") { b.block += 4; }
            if self.relics.iter().any(|r| r == "虎符") { b.strength += 1; }
            if self.relics.iter().any(|r| r == "兵书") { b.scheme += 1; }
            // Relationship rewards live in the same battle snapshot, no separate UI logic.
            if self.hero == Hero::GuanYu && self.lore_flags.iter().any(|f| f == "taoyuan_oath") {
                b.block += 6;
            }
            if self.hero == Hero::GuanYu && self.lore_flags.iter().any(|f| f == "huarong_mercy") {
                b.strength += 1;
            }
            // Chapter V's voluntary accounting for the coalition's supplies
            // pays off only if the chapter LXXV Jingzhou vigilance vow was
            // also kept. It never gates the original main story.
            if self.hero == Hero::GuanYu
                && self.lore_flags.iter().any(|f| f == "wenjiu_oath")
                && self.lore_flags.iter().any(|f| f == "jingzhou_warning") {
                b.block += 3;
            }
            if self.hero == Hero::ZhugeLiang && self.lore_flags.iter().any(|f| f == "longzhong_plan") {
                b.scheme += 2;
            }
            if self.hero == Hero::ZhugeLiang && self.lore_flags.iter().any(|f| f == "star_lantern") {
                b.energy += 1;
            }
            if self.hero == Hero::SunShangxiang && self.lore_flags.iter().any(|f| f == "jiangdong_covenant") {
                b.combo += 2;
            }
            if self.hero == Hero::SunShangxiang && self.lore_flags.iter().any(|f| f == "river_divide") {
                b.thorns += 3;
            }
            if self.hero == Hero::CaoCao && self.lore_flags.iter().any(|f| f == "xu_capital") {
                b.command += 2;
            }
            if self.hero == Hero::CaoCao && self.lore_flags.iter().any(|f| f == "guandu_grain") {
                b.block += 7;
                // Chapter 4's acknowledgement of Chen Gong's warning pays off
                // only after the chapter 14 court arc and chapter 30 Guandu arc.
                // This is an alternate player choice, not the novel's history.
                if self.lore_flags.iter().any(|f| f == "sevenstar_vow") {
                    b.command += 1;
                }
            }
            if self.hero == Hero::LuBu && self.lore_flags.iter().any(|f| f == "red_hare") {
                b.fury += 3;
            }
            if self.hero == Hero::LuBu && self.lore_flags.iter().any(|f| f == "phoenix_pavilion") {
                b.strength += 1;
            }
        }
        self.stage = Stage::Combat;
    }
    fn finish_scene(&mut self) {
        self.stage = Stage::Map;
        self.last_node = None;
        self.rewards.clear();
        self.market.clear();
        self.shop_relic = None;
        self.boon_choices.clear();
        self.lore_current = None;
        self.refresh_route();
    }
    fn choose_route(&mut self, index: usize) -> Result<String, String> {
        let node = *self.route.get(index.checked_sub(1).ok_or("路线编号从1开始")?)
            .ok_or("无效路线编号")?;
        if let Some(lane) = self.route_lanes.get(index - 1) {
            self.map_lane = Some(*lane);
        }
        self.floor += 1;
        self.last_node = Some(node);
        let ground=self.act_map.as_ref().and_then(|map|
            self.map_lane.map(|lane|map.terrain_at(usize::from((self.floor-1)%7),lane)))
            .unwrap_or(map::Terrain::Road);
        let mut msg = format!("第{}层：{}（{}）", self.floor, node.name(),ground.label());
        match ground {
            map::Terrain::Supply => {
                self.gold=self.gold.saturating_add(12);
                msg.push_str("；收集辎重，获得12金");
            }
            map::Terrain::Hazard => {
                let lost=(self.hp-1).clamp(0,3);
                self.hp-=lost;
                msg.push_str(&format!("；穿越险地，消耗{lost}体力"));
            }
            map::Terrain::Road|map::Terrain::Closed => {}
        }
        match node {
            Node::Battle | Node::Elite | Node::Boss => self.start_battle(node),
            Node::Camp => self.stage = Stage::Camp,
            Node::Market => {
                self.market = self.pick_reward_cards();
                self.shop_relic = self.select_relic();
                self.stage = Stage::Market;
            }
            Node::Story => {
                for card in &mut self.deck {
                    if card.id == "quest_tulu" {
                        card.quest_progress = card.quest_progress.saturating_add(1);
                        if card.quest_progress >= 3 {
                            card.id = "quest_resolved".into();
                            card.quest_progress = 0;
                        }
                    }
                }
                self.event_id = self.random(12) as u8;
                // The literary ledger is deterministic for any saved seed and visited flags.
                // Original generic chance events remain when no eligible authored beat exists.
                let act = (self.floor.saturating_sub(1) / 7).min(2);
                self.lore_current = lore::next_beat(self.hero, act, &self.lore_flags, &self.lore_seen)
                    .map(|beat| beat.id.to_owned());
                self.stage = Stage::Story;
            },
        }
        Ok(msg)
    }
    fn target_index(&self, requested: usize) -> Result<usize, String> {
        let battle = self.battle.as_ref().ok_or("当前不在战斗中")?;
        let idx = requested.checked_sub(1).ok_or("目标编号从1开始")?;
        if battle.foes.get(idx).is_none_or(|f| f.hp <= 0) {
            Err("无效目标：敌军不存在或已阵亡".into())
        } else { Ok(idx) }
    }


    fn record_metric(&mut self, action: impl Fn(&mut CombatCounters)) {
        action(&mut self.telemetry.totals);
        action(&mut self.telemetry.current_turn);
    }

    fn capture_turn_metrics(&mut self, completed: bool) {
        let Some(b) = &self.battle else { return };
        let turn=b.turn;
        let troop_hp=b.troops_hp;
        if completed {
            self.record_metric(|c| c.turns_completed = c.turns_completed.saturating_add(1));
        }
        let entry=TurnTelemetry {
            floor: self.floor, turn, completed,
            hero_hp_after: self.hp, troop_hp_after: troop_hp,
            counters: std::mem::take(&mut self.telemetry.current_turn),
        };
        self.telemetry.recent_turns.push(entry);
        if self.telemetry.recent_turns.len()>MAX_RECENT_TURN_METRICS {
            self.telemetry.recent_turns.remove(0);
        }
    }

    fn combat_event(&mut self, kind: EventKind, source: &str, amount: i32, target: Option<String>) {
        // One synchronous, bounded trigger pipeline. All observers see the original
        // event before any subsequent PowerTriggered or CardDrawn events.
        let Some(b) = &mut self.battle else { return };
        b.event_seq = b.event_seq.saturating_add(1);
        let event = CombatEvent {
            seq: b.event_seq, kind, source: source.to_owned(), amount, target,
        };
        if let Some(history)=self.event_history.as_mut() {
            history.push(event_stack::RunEvent{
                encounter:self.telemetry.totals.battles_started,
                floor:self.floor,event:event.clone()});
        }
        b.events.push(event.clone());
        if b.events.len() > MAX_RECENT_EVENTS {
            let count = b.events.len() - MAX_RECENT_EVENTS;
            b.events.drain(..count);
        }
        self.dispatch_combat_event(&event);
    }

    fn attack_effect(effect: Effect) -> bool {
        matches!(effect,
            Effect::Hit(_) | Effect::HitDraw(_,_) | Effect::HitTwice(_)
            | Effect::All(_) | Effect::HitWeak(_,_) | Effect::HitVulnerable(_,_)
            | Effect::MomentumStrike(_) | Effect::SchemeStrike(_) | Effect::ComboStrike(_)
            | Effect::CommandStrike(_) | Effect::FuryStrike(_) | Effect::BloodPrice(_,_)
            | Effect::Leech(_) | Effect::HitPoison(_,_) | Effect::Pierce(_)
            | Effect::DiscardStrike(_) | Effect::XHit(_)
        )
    }

    fn dispatch_combat_event(&mut self, event: &CombatEvent) {
        if event.kind==EventKind::CardResolved {
            use event_stack::{Listener,Response};
            let reaction=match event.source.as_str() {
                "hold_line"=>Some(Listener::once("hold_line:weak","weak",Response::Cancel)),
                "kongcheng"=>Some(Listener::once("kongcheng:vulnerable","vulnerable",Response::Reduce(1))),
                "qiaoying"=>Some(Listener::once("qiaoying:weak","weak",Response::Reduce(1))),
                "zhongyi"=>Some(Listener::once("zhongyi:weak","weak",Response::Replace(1))),
                _=>None,
            };
            if let Some(rule)=reaction {
                self.battle.as_mut().expect("battle").reactions.install(rule)
                    .expect("valid builtin listener");
            }
        }
        match event.kind {
            EventKind::CardResolved => {
                let attack = card_spec(&event.source).is_some_and(|card| Self::attack_effect(card.effect));
                if attack {
                    let guard = self.power_stacks(PowerId::OathGuard) * 3;
                    if guard > 0 {
                        self.battle.as_mut().expect("battle").block += guard;
                        self.combat_event(EventKind::PowerTriggered, "oath_guard", guard, None);
                    }
                } else {
                    let stacks = self.power_stacks(PowerId::ThoughtSpring);
                    let unused = self.battle.as_ref().is_some_and(|b| !b.thought_used);
                    if stacks > 0 && unused {
                        let b = self.battle.as_mut().expect("battle");
                        b.energy += stacks;
                        b.thought_used = true;
                        self.combat_event(EventKind::PowerTriggered, "thought_spring", stacks, None);
                        self.draw_cards(1);
                    }
                }
            }
            EventKind::CardDiscarded => {
                let amount = self.power_stacks(PowerId::ArrowsOnDiscard) * 4;
                if amount > 0 {
                    let target = self.battle.as_ref().expect("battle").foes.iter()
                        .position(|foe| foe.hp > 0);
                    if let Some(idx) = target {
                        self.combat_event(EventKind::PowerTriggered,"arrows_on_discard",
                            amount,Some(format!("foe:{idx}")));
                        self.damage_enemy_fixed(idx, amount, "arrows_on_discard");
                    }
                }
            }
            EventKind::TurnEnded => self.trigger_granary(),
            EventKind::DamageReceived if event.amount > 0 => {
                let stacks = self.power_stacks(PowerId::BloodCourage);
                let fury = stacks * 2;
                if fury > 0 {
                    self.battle.as_mut().expect("battle").fury += fury;
                    self.combat_event(EventKind::PowerTriggered, "blood_courage", fury, None);
                }
            }
            EventKind::CardDrawn => {
                let amount = self.power_stacks(PowerId::WisdomOnDraw);
                if amount > 0 {
                    self.battle.as_mut().expect("battle").block += amount;
                    self.combat_event(EventKind::PowerTriggered,"wisdom_on_draw",amount,None);
                }
            }
            EventKind::StatusApplied if event.source == "poison" && event.amount > 0 => {
                let amount = self.power_stacks(PowerId::ToxicReversal);
                let idx = event.target.as_deref()
                    .and_then(|text| text.strip_prefix("foe:"))
                    .and_then(|text| text.parse::<usize>().ok());
                if let Some(idx) = idx.filter(|_| amount > 0) {
                    // The bonus is a separate status proposal. It must pass
                    // through the same reaction stack as the original poison,
                    // but its distinct source must NOT trigger this power again.
                    let capacity = self.battle.as_ref().expect("battle").foes.get(idx)
                        .map_or(0, |foe| if foe.hp > 0 {
                            9999i32.saturating_sub(foe.poison)
                        } else { 0 });
                    if capacity > 0 {
                        let target = format!("foe:{idx}");
                        if let Some(adjusted) = self.preflight_status(
                            "poison:toxic_reversal", &target, amount.min(capacity)) {
                            let applied = self.battle.as_mut().expect("battle").foes
                                .get_mut(idx).map_or(0, |foe| {
                                    if foe.hp <= 0 { return 0; }
                                    let before = foe.poison;
                                    foe.poison = foe.poison.saturating_add(adjusted).min(9999);
                                    foe.poison - before
                                });
                            if applied > 0 {
                                self.combat_event(EventKind::PowerTriggered, "toxic_reversal",
                                    applied, Some(target.clone()));
                                self.combat_event(EventKind::StatusApplied,
                                    "poison:toxic_reversal", applied, Some(target));
                            }
                        }
                    }
                }
            }
            EventKind::FoeDefeated => {
                let amount = self.power_stacks(PowerId::VictoryDrum);
                if amount > 0 {
                    self.battle.as_mut().expect("battle").energy += amount;
                    self.combat_event(EventKind::PowerTriggered,"victory_drum",amount,event.target.clone());
                }
            }
            _ => {}
        }
    }
    fn power_stacks(&self, id: PowerId) -> i32 {
        self.battle.as_ref().map_or(0, |b|
            b.powers.iter().find(|p| p.id == id).map_or(0, |p| p.stacks))
    }
    fn gain_power(&mut self, id: PowerId, stacks: i32) {
        if stacks <= 0 { return; }
        // A power already at cap has no proposed delta. Do not consume
        // listeners on an effect that could not change the actual state.
        let capacity=(99-self.power_stacks(id)).max(0);
        if capacity==0 {return;}
        let proposed=stacks.min(capacity);
        let Some(adjusted)=self.preflight_event(
            EventKind::PowerGained,id.key(),"hero",proposed) else {return;};
        // An external Replace can exceed remaining room; only actual gain is
        // journalled, and the power never exceeds its original cap.
        let gain=adjusted.min(capacity);
        let applied = {
            let b = self.battle.as_mut().expect("battle");
            if let Some(existing) = b.powers.iter_mut().find(|p| p.id == id) {
                let gained = gain.min(99 - existing.stacks);
                existing.stacks += gained;
                gained
            } else {
                let gained = gain.min(99);
                b.powers.push(PowerStack { id, stacks: gained });
                gained
            }
        };
        if applied > 0 {
            self.combat_event(EventKind::PowerGained, id.name(), applied, None);
        }
    }
    fn trigger_granary(&mut self) {
        let stacks = self.power_stacks(PowerId::GranarySupport);
        if stacks == 0 { return; }
        let b = self.battle.as_mut().expect("battle");
        if b.troops_hp == 0 { return; }
        let restored = (40 - b.troops_hp).max(0).min(stacks * 4);
        b.troops_hp += restored;
        if restored > 0 {
            self.combat_event(EventKind::PowerTriggered, "granary_support", restored, None);
        }
    }


    /// Resolve a proposed status or enemy-damage delta BEFORE settlement.
    /// Each consumed rule writes its own audit event in deterministic order.
    /// The proposal is NOT the post-block DamageDealt event stored in the journal.
    fn preflight_event(&mut self, kind:EventKind, source:&str, target:&str, amount:i32) -> Option<i32> {
        if amount<=0 {return None;}
        let proposed=CombatEvent{
            seq:0,kind,
            source:source.into(),amount,target:Some(target.into()),
        };
        let outcome=self.battle.as_mut().expect("battle").reactions.resolve(&proposed);
        for step in &outcome.steps {
            if step.cancelled || step.after==0 {
                self.combat_event(EventKind::EventCountered,&step.listener_id,step.before,
                    Some(target.into()));
            } else if step.after!=step.before {
                self.combat_event(EventKind::EventReplaced,&step.listener_id,
                    step.before-step.after,Some(target.into()));
            }
        }
        (!outcome.cancelled && outcome.amount>0).then_some(outcome.amount)
    }
    fn preflight_status(&mut self, status:&str, target:&str, amount:i32) -> Option<i32> {
        self.preflight_event(EventKind::StatusApplied,status,target,amount)
    }

    /// Saturating hero-debuff application after all registered observers have
    /// had an opportunity to cancel, reduce or replace the incoming duration.
    fn apply_hero_status(&mut self, status: &str, amount: u8) {
        if !matches!(status,"weak"|"vulnerable") {return;}
        // Status already capped: no effect is proposed, therefore an
        // untriggered one-shot listener must not lose a charge.
        let saturated=self.battle.as_ref().is_some_and(|b|match status {
            "weak"=>b.weak==u8::MAX,
            "vulnerable"=>b.vulnerable==u8::MAX,
            _=>false,
        });
        if saturated {return;}
        let Some(amount)=self.preflight_status(status,"hero",i32::from(amount)) else {return;};
        let amount=amount.clamp(0,255) as u8;
        let applied = {
            let b = self.battle.as_mut().expect("battle");
            match status {
                "weak" => {
                    let before = b.weak;
                    b.weak = b.weak.saturating_add(amount);
                    b.weak - before
                }
                "vulnerable" => {
                    let before = b.vulnerable;
                    b.vulnerable = b.vulnerable.saturating_add(amount);
                    b.vulnerable - before
                }
                _ => unreachable!("status prevalidated"),
            }
        };
        if applied>0 {
            self.combat_event(EventKind::StatusApplied,status,i32::from(applied),
                Some("hero".into()));
        }
    }

    /// Enemy debuffs, including potion applications, use the SAME interception
    /// stack. A missing or defeated target must not consume a reaction charge.
    fn apply_foe_status(&mut self, idx: usize, status: &str, amount: i32) {
        if !matches!(status,"poison"|"weak"|"vulnerable")
            || self.battle.as_ref().is_none_or(|b|
                b.foes.get(idx).is_none_or(|foe|foe.hp<=0)) {return;}
        let exhausted_capacity=self.battle.as_ref().is_some_and(|b| {
            let foe=&b.foes[idx];
            match status {
                "poison"=>foe.poison>=9999,
                "weak"=>foe.weak==u8::MAX,
                "vulnerable"=>foe.vulnerable==u8::MAX,
                _=>false,
            }
        });
        // Reserve the rule for the next *effective* status application.
        if exhausted_capacity {return;}
        let target=format!("foe:{idx}");
        let Some(amount)=self.preflight_status(status,&target,amount) else {return;};
        let actual = {
            let b = self.battle.as_mut().expect("battle");
            let foe=&mut b.foes[idx];
            match status {
                "poison" => {
                    let old=foe.poison;
                    foe.poison=foe.poison.saturating_add(amount).min(9999);
                    foe.poison-old
                }
                "weak" => {
                    let old=foe.weak;
                    foe.weak=foe.weak.saturating_add(amount.min(255) as u8);
                    i32::from(foe.weak-old)
                }
                "vulnerable" => {
                    let old=foe.vulnerable;
                    foe.vulnerable=foe.vulnerable.saturating_add(amount.min(255) as u8);
                    i32::from(foe.vulnerable-old)
                }
                _ => unreachable!("status prevalidated"),
            }
        };
        if actual>0 {
            self.combat_event(EventKind::StatusApplied,status,actual,Some(target));
        }
    }


    /// Played attack cards alone inherit hero strength, weakness and class buffs.
    /// Potions and discard-triggered effects are flat, blockable damage and never
    /// inherit the attacker's card-only modifiers.
    fn damage_enemy(&mut self, idx: usize, base: i32) {
        let damage = {
            let b = self.battle.as_ref().expect("battle");
            let raw = base.saturating_add(b.strength)
                .saturating_add(if self.hero == Hero::GuanYu { b.momentum / 3 } else { 0 })
                .saturating_add(if self.hero == Hero::LuBu { b.fury / 4 } else { 0 })
                .max(0);
            let weakened = if b.weak > 0 { raw.saturating_mul(3) / 4 } else { raw };
            let foe = &b.foes[idx];
            if foe.vulnerable > 0 { weakened.saturating_mul(3) / 2 } else { weakened }
        };
        self.deal_enemy_damage(idx, damage, "hero");
    }

    /// Flat non-attack damage retains armor, defeat and power-trigger semantics,
    /// but cannot borrow hero strength, weakness or foe vulnerability.
    fn damage_enemy_fixed(&mut self, idx: usize, amount: i32, source: &str) {
        self.deal_enemy_damage(idx, amount.max(0), source);
    }

    /// Central settlement for every damage source that targets an enemy.
    /// Piercing, poison and thorns bypass armor; all other sources consume
    /// armor first. A defeated enemy cannot take damage or die twice.
    fn settle_foe_damage(&mut self, idx: usize, damage: i32, source: &str, ignores_block: bool) -> i32 {
        // Dead targets never consume a charge. Active reactions see the raw hit
        // before armor, so cancelling a hit cannot secretly strip enemy block.
        if self.battle.as_ref().expect("battle").foes[idx].hp <= 0 { return 0; }
        let damage = if damage > 0 {
            self.preflight_event(EventKind::DamageDealt,source,&format!("foe:{idx}"),damage)
                .unwrap_or(0)
        } else { 0 };
        let (actual, killed) = {
            let b = self.battle.as_mut().expect("battle");
            let foe = &mut b.foes[idx];
            if foe.hp <= 0 { return 0; }
            let incoming = damage.max(0);
            let blocked = if ignores_block { 0 } else { incoming.min(foe.block) };
            foe.block -= blocked;
            let actual = foe.hp.min(incoming.saturating_sub(blocked));
            foe.hp -= actual;
            (actual, actual > 0 && foe.hp == 0)
        };
        self.combat_event(EventKind::DamageDealt, source, actual, Some(format!("foe:{idx}")));
        if killed {
            self.combat_event(EventKind::FoeDefeated, source, 1, Some(format!("foe:{idx}")));
        }
        actual
    }

    fn deal_enemy_damage(&mut self, idx: usize, damage: i32, source: &str) -> i32 {
        self.settle_foe_damage(idx, damage, source, false)
    }

    fn deal_enemy_damage_unblockable(&mut self, idx: usize, damage: i32, source: &str) -> i32 {
        self.settle_foe_damage(idx, damage, source, true)
    }
    fn defeated(&self) -> bool {
        self.battle.as_ref().is_some_and(|b| b.foes.iter().all(|f| f.hp <= 0))
    }

    fn select_relic(&mut self) -> Option<String> {
        let available: Vec<&str> = items::RELICS.iter().map(|(id, _)| *id)
            .filter(|id| !self.relic_history.iter().any(|r| r == *id)).collect();
        if available.is_empty() { None }
        else { Some(available[self.random(available.len())].to_string()) }
    }
    fn grant_relic(&mut self) {
        if let Some(id) = self.select_relic() {
            if id == "厚甲" { self.max_hp += 8; self.hp += 8; }
            self.relic_history.push(id.clone());
            self.relics.push(id);
        } else {
            self.gold += 50;
        }
    }
    fn win_battle(&mut self) -> String {
        // Emit a final boundary while the battle journal still exists.
        // Legacy archives without this marker remain readable, but cannot be
        // certified as sealed by the event-only projection.
        self.combat_event(EventKind::BattleEnded, "victory", 1, None);
        let node = self.last_node.unwrap_or(Node::Battle);
        self.record_metric(|c| c.battles_won = c.battles_won.saturating_add(1));
        self.capture_turn_metrics(false);
        self.battle = None;
        let payout = match node { Node::Boss => 100, Node::Elite => 60, _ => 24 + self.random(15) as i32 };
        self.gold += payout;
        if self.relics.iter().any(|r| r == "铜雀") { self.gold += 12; }
        if self.relics.iter().any(|r| r == "神医") {
            self.hp = (self.hp + 6).min(self.max_hp);
        }
        if self.relics.iter().any(|r| r == "百草") {
            self.hp = (self.hp + 2).min(self.max_hp);
        }
        if self.boons.iter().any(|id| id == "renewal") {
            self.hp = (self.hp + 8).min(self.max_hp);
        }
        if node == Node::Elite || node == Node::Boss {
            self.grant_relic();
            if node == Node::Boss {
                self.max_hp += 5;
                self.hp = (self.hp + 5).min(self.max_hp);
            }
        }
        let chance = if self.relics.iter().any(|r| r == "药囊") { 60 } else { 32 };
        if self.potions < 3 && self.random(100) < chance {
            let pool = ["healing", "fire", "shield", "energy", "poison", "strength", "insight", "weak", "cleanse", "blood"];
            let potion = pool[self.random(pool.len())];
            self.potion_bag.push(potion.into());
            self.potions = self.potion_bag.len() as u8;
        }
        if self.floor >= MAX_FLOOR {
            self.victories = self.victories.saturating_add(1);
            self.stage = Stage::Won;
            return "三军凯旋！三幕征程全部通关。".into();
        }
        self.rewards = self.pick_reward_cards();
        self.stage = Stage::Reward;
        format!("战斗胜利！获得{payout}金，选择一张战利品，或输入 choose 0 跳过。")
    }
    /// Keep the read-only card picker and the authoritative resolver in sync.
    fn eligible_pile_card(card: &Card, attack_only: bool) -> bool {
        card_spec(&card.id).is_some_and(|spec| {
            !matches!(spec.effect, Effect::Unplayable | Effect::CopyDiscard)
                && (!attack_only || Self::attack_effect(spec.effect))
        })
    }

    /// Returns legal 1-based secondary choices, top of pile first, for a hand card.
    /// None means this card needs no secondary choice. Reading never changes RNG or the save.
    pub fn secondary_choices(&self, hand_number: usize)
        -> Result<Option<Vec<(usize, Card)>>, String>
    {
        let b = self.battle.as_ref().ok_or("当前不在战斗中")?;
        let idx = hand_number.checked_sub(1).ok_or("手牌编号从1开始")?;
        let card = b.hand.get(idx).ok_or("无效手牌编号")?;
        let spec = card_spec(&card.id).ok_or("未知卡牌")?;
        let (pile, attack_only) = match spec.effect {
            Effect::TutorAttack => (&b.draw, true),
            Effect::CopyDiscard => (&b.discard, false),
            _ => return Ok(None),
        };
        Ok(Some(pile.iter().rev().enumerate()
            .filter(|(_, card)| Self::eligible_pile_card(card, attack_only))
            .map(|(idx, card)| (idx + 1, card.clone())).collect()))
    }

    /// Pile choices use 1-based visual order, top first.
    fn choose_pile_card(cards: &[Card], selection: Option<usize>, attack_only: bool)
        -> Result<Option<usize>, String>
    {
        if let Some(number) = selection {
            let idx = number.checked_sub(1)
                .and_then(|n| cards.len().checked_sub(n+1))
                .ok_or("选牌编号超出当前牌堆")?;
            if !Self::eligible_pile_card(&cards[idx], attack_only) {
                return Err("所选卡牌不符合检索条件".into());
            }
            Ok(Some(idx))
        } else {
            Ok(cards.iter().rposition(|card| Self::eligible_pile_card(card, attack_only)))
        }
    }

    /// Temporary copies and generated cards leave combat when used or on turn end.
    fn add_temporary_to_hand(&mut self, card: Card, kind: EventKind) -> bool {
        let id = card.id.clone();
        let b = self.battle.as_mut().expect("battle");
        if b.hand.len() >= 10 { return false; }
        b.hand.push(Card { temporary: true, ..card });
        self.combat_event(kind,&id,1,Some("hero".into()));
        true
    }

    fn play(&mut self, index: usize, target: usize, selection: Option<usize>) -> Result<String, String> {
        let b = self.battle.as_ref().ok_or("当前不在战斗中")?;
        let hand_idx = index.checked_sub(1).ok_or("手牌编号从1开始")?;
        let card = b.hand.get(hand_idx).ok_or("无效手牌编号")?.clone();
        let spec = card_spec(&card.id).ok_or("未知卡牌")?;
        if matches!(spec.effect, Effect::Unplayable) { return Err("这张牌无法打出".into()); }
        // X cards spend all available energy. Swift never discounts an X cost.
        let cost = if spec.cost == -1 { b.energy } else if card.enchantment.as_deref() == Some("迅捷") {
            (spec.cost - 1).max(0)
        } else { spec.cost };
        if b.energy < cost { return Err("军令不足".into()); }
        let requires_target = matches!(
            spec.effect, Effect::Hit(_) | Effect::HitDraw(_, _) | Effect::HitTwice(_)
                | Effect::Poison(_) | Effect::HitWeak(_, _) | Effect::HitVulnerable(_, _)
                | Effect::MomentumStrike(_) | Effect::SchemeStrike(_) | Effect::ComboStrike(_)
                | Effect::CommandStrike(_) | Effect::FuryStrike(_) | Effect::BloodPrice(_, _)
                | Effect::Leech(_) | Effect::HitPoison(_, _) | Effect::Pierce(_)
                | Effect::DiscardStrike(_) | Effect::XHit(_)
        );
        // A failed pile choice never spends energy or consumes a card.
        let chosen = match spec.effect {
            Effect::TutorAttack => Self::choose_pile_card(&b.draw, selection, true)?,
            Effect::CopyDiscard => Self::choose_pile_card(&b.discard, selection, false)?,
            _ => {
                if selection.is_some() { return Err("此卡牌不需要副选项编号".into()); }
                None
            }
        };
        let target_idx = if requires_target { Some(self.target_index(target)?) } else { None };
        {
            let b = self.battle.as_mut().expect("battle");
            b.energy -= cost;
            b.hand.remove(hand_idx);
        }
        self.record_metric(|c| c.cards_played = c.cards_played.saturating_add(1));
        self.combat_event(EventKind::CardPlayed, &card.id, cost, None);
        let bonus = if card.upgraded { 3 } else { 0 }
            + if card.enchantment.as_deref() == Some("锐锋") { 3 } else { 0 };
        match spec.effect {
            Effect::Hit(n) => self.damage_enemy(target_idx.expect("target"), n + bonus),
            Effect::HitDraw(n, draw) => {
                self.damage_enemy(target_idx.expect("target"), n + bonus);
                self.draw_cards(draw);
            }
            Effect::HitTwice(n) => {
                for _ in 0..2 { self.damage_enemy(target_idx.expect("target"), n + bonus); }
            }
            Effect::All(n) => {
                let count = self.battle.as_ref().expect("battle").foes.len();
                let extra = if self.relics.iter().any(|r| r == "朱雀") { 3 } else { 0 }
                    + if self.boons.iter().any(|id| id == "flaming_sky") { 4 } else { 0 };
                for i in 0..count {
                    if self.battle.as_ref().expect("battle").foes[i].hp > 0 {
                        self.damage_enemy(i, n + bonus + extra);
                    }
                }
            }
            Effect::Guard(n) => self.battle.as_mut().expect("battle").block += n + bonus,
            Effect::GuardEnergy(n) => {
                let b = self.battle.as_mut().expect("battle");
                b.block += n + bonus;
                b.energy += 1;
            }
            Effect::Poison(n) =>
                self.apply_foe_status(target_idx.expect("target"),"poison",n+bonus),
            Effect::Draw(n) => self.draw_cards(n + usize::from(card.upgraded)),
            Effect::Strength(n) => self.battle.as_mut().expect("battle").strength += n + i32::from(card.upgraded),
            Effect::Heal(n) => self.hp = (self.hp + n + bonus).min(self.max_hp),
            Effect::HitWeak(n, turns) => {
                let idx = target_idx.expect("target");
                self.damage_enemy(idx, n + bonus);
                self.apply_foe_status(idx,"weak",i32::from(turns));
            }
            Effect::HitVulnerable(n, turns) => {
                let idx = target_idx.expect("target");
                self.damage_enemy(idx, n + bonus);
                self.apply_foe_status(idx,"vulnerable",i32::from(turns));
            }
            Effect::WeakAll(turns) => {
                let count = self.battle.as_ref().expect("battle").foes.len();
                for i in 0..count { self.apply_foe_status(i,"weak",i32::from(turns)); }
            }
            Effect::ApplyVulnerable(turns) => {
                let count = self.battle.as_ref().expect("battle").foes.len();
                for i in 0..count { self.apply_foe_status(i,"vulnerable",i32::from(turns)); }
            }
            Effect::GainMomentum(n) => self.battle.as_mut().expect("battle").momentum += n + bonus,
            Effect::MomentumStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").momentum * 3;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
                self.battle.as_mut().expect("battle").momentum = 0;
            }
            Effect::Scheme(n) => self.battle.as_mut().expect("battle").scheme += n + bonus,
            Effect::SchemeStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").scheme * 4;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
                self.battle.as_mut().expect("battle").scheme = 0;
            }
            Effect::ComboStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").combo * 3;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
            }
            Effect::Command(n) => self.battle.as_mut().expect("battle").command += n + bonus,
            Effect::CommandStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").command * 3;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
                self.battle.as_mut().expect("battle").command = 0;
            }
            Effect::Fury(n) => self.battle.as_mut().expect("battle").fury += n + bonus,
            Effect::FuryStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").fury * 3;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
                self.battle.as_mut().expect("battle").fury = 0;
            }
            Effect::BloodPrice(loss, power) => {
                let actual = loss.min((self.hp-1).max(0));
                self.hp -= actual;
                self.battle.as_mut().expect("battle").fury += actual;
                if actual > 0 {
                    self.combat_event(EventKind::DamageReceived,"blood_price",actual,Some("hero".into()));
                }
                self.damage_enemy(target_idx.expect("target"), power + bonus);
            }
            Effect::Thorns(n) => self.battle.as_mut().expect("battle").thorns += n + bonus,
            Effect::Leech(n) => {
                let idx = target_idx.expect("target");
                let before = self.battle.as_ref().expect("battle").foes[idx].hp;
                self.damage_enemy(idx, n + bonus);
                let after = self.battle.as_ref().expect("battle").foes[idx].hp;
                self.hp = (self.hp + (before - after) / 3).min(self.max_hp);
            }
            Effect::HitPoison(n, poison) => {
                let idx = target_idx.expect("target");
                self.damage_enemy(idx, n + bonus);
                self.apply_foe_status(idx,"poison",poison);
            }
            Effect::Pierce(n) => {
                let idx = target_idx.expect("target");
                self.deal_enemy_damage_unblockable(idx, n.saturating_add(bonus), "pierce");
            }
            Effect::DrawEnergy(n, energy) => {
                self.battle.as_mut().expect("battle").energy += energy;
                self.draw_cards(n + usize::from(card.upgraded));
            }
            Effect::Energy(n) => self.battle.as_mut().expect("battle").energy += n + i32::from(card.upgraded),
            Effect::Companion(amount) => {
                let b = self.battle.as_mut().expect("battle");
                b.troops_hp += amount + bonus;
                b.troops_attack += (amount / 4).max(1);
            }
            Effect::Tactics(amount) => {
                self.battle.as_mut().expect("battle").discard_tokens += amount;
            }
            Effect::DiscardStrike(amount) => {
                self.damage_enemy(target_idx.expect("target"), amount + bonus);
            }
            Effect::XHit(per_energy) => {
                let invested=cost+i32::from(self.relics.iter().any(|r|r=="玄机炉"));
                if invested>0 {
                    self.damage_enemy(target_idx.expect("target"),
                        (per_energy + 2*i32::from(card.upgraded))*invested
                        + if card.enchantment.as_deref() == Some("锐锋") { 3 } else { 0 });
                }
            }
            Effect::XGuard(per_energy) => {
                let invested=cost+i32::from(self.relics.iter().any(|r|r=="玄机炉"));
                let amount = (per_energy + 2*i32::from(card.upgraded))*invested;
                self.battle.as_mut().expect("battle").block += amount;
            }
            Effect::TutorAttack => {
                if let Some(idx) = chosen
                    && self.battle.as_ref().expect("battle").hand.len() < 10
                {
                    let b = self.battle.as_mut().expect("battle");
                    let found = b.draw.remove(idx);
                    let id = found.id.clone();
                    b.hand.push(found);
                    self.combat_event(EventKind::CardTutored,&id,1,Some("hero".into()));
                }
            }
            Effect::CopyDiscard => {
                if let Some(idx) = chosen {
                    let found = self.battle.as_ref().expect("battle").discard[idx].clone();
                    self.add_temporary_to_hand(found,EventKind::CardCopied);
                }
            }
            Effect::Generate(id, number) => {
                for _ in 0..(number + usize::from(card.upgraded)) {
                    if !self.add_temporary_to_hand(Card::new(id),EventKind::CardGenerated) { break; }
                }
            }
            Effect::Power(id, stacks) => {
                self.gain_power(id, stacks + i32::from(card.upgraded));
            }
            Effect::Unplayable => unreachable!(),
        }
        if card.enchantment.as_deref() == Some("护身") {
            self.battle.as_mut().expect("battle").block += 4;
        }
        if card.enchantment.as_deref() == Some("明悟") { self.draw_cards(1); }
        let attack = requires_target || matches!(spec.effect, Effect::All(_));
        {
            let b = self.battle.as_mut().expect("battle");
            b.played += 1;
            b.played_this_turn = b.played_this_turn.saturating_add(1);
            match self.hero {
                Hero::GuanYu if attack => b.momentum += 1,
                Hero::ZhugeLiang if !attack => b.scheme += 1,
                Hero::SunShangxiang if attack => b.combo += 1,
                Hero::CaoCao if !attack => b.command += 1,
                _ => {}
            }
        }
        if self.hero == Hero::ZhugeLiang && self.battle.as_ref().expect("battle").scheme >= 3 {
            self.battle.as_mut().expect("battle").scheme -= 3;
            self.battle.as_mut().expect("battle").energy += 1;
            self.draw_cards(1);
        }
        if self.hero == Hero::SunShangxiang && attack
            && self.battle.as_ref().expect("battle").combo % 3 == 0 {
            // Combo mobility: the every-third-attack card draw now also gives
            // a small guard. This rewards active combat without raising HP.
            self.battle.as_mut().expect("battle").block += 3;
            self.record_metric(|c| {
                c.sun_combo_procs = c.sun_combo_procs.saturating_add(1);
                c.sun_combo_block = c.sun_combo_block.saturating_add(3);
            });
            self.combat_event(EventKind::PowerTriggered,"sun_combo_guard",3,None);
            let old_len=self.battle.as_ref().expect("battle").hand.len();
            self.draw_cards(1);
            let added=self.battle.as_ref().expect("battle").hand.len().saturating_sub(old_len) as u64;
            self.record_metric(|c| c.sun_combo_draws = c.sun_combo_draws.saturating_add(added));
        }
        if self.hero == Hero::CaoCao && self.battle.as_ref().expect("battle").command >= 4 {
            let b = self.battle.as_mut().expect("battle");
            b.command -= 4;
            b.block += 5;
            b.energy += 1;
            self.record_metric(|c| {
                c.command_procs = c.command_procs.saturating_add(1);
                c.command_energy = c.command_energy.saturating_add(1);
                c.command_block = c.command_block.saturating_add(5);
            });
        }
        if self.relics.iter().any(|r| r == "百战") && attack {
            self.battle.as_mut().expect("battle").strength += 1;
        }
        self.combat_event(EventKind::CardResolved,&card.id,0,None);
        let msg = format!("使用了【{}】。", card.name());
        if spec.exhaust || card.temporary {
            self.combat_event(EventKind::CardExhausted,&card.id,1,None);
            self.battle.as_mut().expect("battle").exhausted.push(card);
        } else {
            self.battle.as_mut().expect("battle").discard.push(card);
        }
        if self.defeated() { Ok(format!("{msg} {}", self.win_battle())) } else { Ok(msg) }
    }
    fn discard_card(&mut self, index: usize) -> Result<String, String> {
        if self.stage != Stage::Combat { return Err("仅战斗中可弃换".into()); }
        let idx = index.checked_sub(1).ok_or("手牌编号从1开始")?;
        let battle = self.battle.as_ref().expect("battle");
        if battle.discard_tokens == 0 { return Err("没有弃换次数；需先打出换阵类卡牌".into()); }
        if idx >= battle.hand.len() { return Err("无效手牌编号".into()); }
        let card = self.battle.as_mut().expect("battle").hand.remove(idx);
        let auto = card.id == "feishi";
        let temporary = card.temporary;
        let card_id = card.id.clone();
        self.combat_event(EventKind::CardDiscarded, &card.id, 1, None);
        let msg = if auto { format!("弃换【{}】：触发策应攻击", card.name()) }
                  else { format!("弃换【{}】", card.name()) };
        {
            let b = self.battle.as_mut().expect("battle");
            b.discard_tokens -= 1;
            if temporary { b.exhausted.push(card); } else { b.discard.push(card); }
            if self.hero == Hero::SunShangxiang {
                b.combo += 1;
                b.block += 2;
            }
        }
        if self.hero == Hero::SunShangxiang {
            self.record_metric(|c| {
                c.sun_discards = c.sun_discards.saturating_add(1);
                c.sun_discard_block = c.sun_discard_block.saturating_add(2);
            });
            self.combat_event(EventKind::PowerTriggered,"sun_discard_guard",2,None);
        }
        if temporary {
            self.combat_event(EventKind::CardExhausted,&card_id,1,None);
        }
        if auto {
            let target = self.battle.as_ref().expect("battle").foes
                .iter().position(|foe| foe.hp > 0);
            if let Some(target) = target { self.damage_enemy_fixed(target, 7, "feishi"); }
        }
        self.draw_cards(1);
        if self.defeated() { Ok(format!("{msg}。{}", self.win_battle())) }
        else { Ok(msg) }
    }


    fn troop_attack(&mut self) {
        let b = self.battle.as_ref().expect("battle");
        if b.troops_hp <= 0 || b.troops_attack <= 0 { return; }
        let attack = b.troops_attack;
        let target = b.foes.iter().position(|foe| foe.hp > 0);
        if let Some(idx) = target {
            let dealt = self.deal_enemy_damage(idx, attack, "troops");
            self.record_metric(|counter| {
                counter.troop_attacks = counter.troop_attacks.saturating_add(1);
                counter.troop_damage = counter.troop_damage.saturating_add(dealt as u64);
            });
        }
    }

    fn enemy_attack(&mut self, foe_idx: usize, power: i32) -> i32 {
        // First compute a proposed enemy hit WITHOUT mutating hero armor or
        // troops. An unranked custom listener may cancel or replace this raw
        // value before the normal block -> troop -> HP settlement.
        if self.battle.as_ref().expect("battle").foes.get(foe_idx)
            .is_none_or(|foe|foe.hp<=0) {return 0;}
        let modified = {
            let b = self.battle.as_ref().expect("battle");
            let reduced = if b.foes[foe_idx].weak > 0 { power * 3 / 4 } else { power };
            if b.vulnerable > 0 { reduced * 3 / 2 } else { reduced }
        };
        let modified = if modified>0 {
            match self.preflight_event(EventKind::DamageReceived,
                &format!("foe:{foe_idx}"),"hero",modified) {
                Some(value) => value,
                // Cancelled enemy attacks never strip armor or troops, never
                // inflict HP damage, and cannot trigger an on-hit follow-up.
                None => return 0,
            }
        } else { modified };
        let (damage, protected, reflected) = {
            let b = self.battle.as_mut().expect("battle");
            let blocked = modified.min(b.block);
            b.block -= blocked;
            let incoming = (modified - blocked).max(0);
            let protected = incoming.min(b.troops_hp);
            b.troops_hp -= protected;
            let damage = self.hp.min(incoming - protected);
            self.hp -= damage;
            if self.hero == Hero::LuBu { b.fury += damage / 2; }
            (damage, protected, b.thorns)
        };
        self.record_metric(|counter| {
            counter.hero_hp_damage = counter.hero_hp_damage.saturating_add(damage as u64);
            counter.troop_absorbed = counter.troop_absorbed.saturating_add(protected as u64);
        });
        // A received hit is observed before its reflected damage, even when
        // the reflector kills the attacker. The common settlement awards any
        // ensuing death exactly once and dispatches VictoryDrum.
        self.combat_event(EventKind::DamageReceived, &format!("foe:{foe_idx}"),
            damage, Some("hero".into()));
        if reflected > 0 {
            self.deal_enemy_damage_unblockable(foe_idx, reflected, "thorns");
        }
        damage
    }
    fn ally_support(&mut self, foe_idx: usize, armor: i32) -> String {
        let (origin, recipients, shielding) = {
            let b = self.battle.as_mut().expect("battle");
            let origin = b.foes[foe_idx].id.clone();
            let partners = b.foes.iter().enumerate()
                .filter(|(idx,foe)| *idx != foe_idx && foe.hp > 0)
                .map(|(idx,_)| idx).collect::<Vec<_>>();
            let targets = if partners.is_empty() { vec![foe_idx] } else { partners };
            for idx in &targets { b.foes[*idx].block += armor; }
            let count = targets.len() as i32;
            (origin,targets.len(),count*armor)
        };
        self.combat_event(EventKind::EnemyTechnique,&origin,shielding,Some("allies".into()));
        format!("{}为{}名友军布阵，累计护甲+{}",enemy_name(&origin),recipients,shielding)
    }

    fn boss_signature(&mut self, idx: usize, technique: BossMove) -> String {
        let (id,phase) = {
            let foe = &self.battle.as_ref().expect("battle").foes[idx];
            (foe.id.clone(),foe.phase)
        };
        // An intent is committed before the hit: thorns can defeat the caster
        // during DamageReceived, before any dependent technique effects resolve.
        self.combat_event(EventKind::EnemyTechnique,&id,1,Some(format!("foe:{idx}")));
        match technique {
            BossMove::Thunder(attack) => {
                let hit = self.enemy_attack(idx,attack);
                let caster_alive = self.battle.as_ref().is_some_and(|b| b.foes[idx].hp > 0);
                if hit > 0 && caster_alive {
                    let turns = 2+u8::from(phase > 0);
                    self.apply_hero_status("weak", turns);
                }
                format!("雷咒命中，损失{hit}生命{}",
                    if !caster_alive {"；施术者遭反击阵亡，虚弱效果中断"}
                    else if hit>0 {"并遭虚弱"} else {"，被护阵抵挡"})
            }
            BossMove::TigerRush(attack) => {
                let hits = 2+usize::from(phase > 0);
                let mut sum = 0;
                for _ in 0..hits {
                    if self.hp == 0 || self.battle.as_ref().expect("battle").foes[idx].hp == 0 {break}
                    sum += self.enemy_attack(idx,attack);
                }
                format!("虎袭连击{hits}段，损失{sum}生命")
            }
            BossMove::TyrantLevy(attack) => {
                let damage = self.enemy_attack(idx,attack);
                let caster_alive = self.battle.as_ref().is_some_and(|b| b.foes[idx].hp > 0);
                let taken = if caster_alive { self.gold.min(10+5*i32::from(phase>0)) } else { 0 };
                self.gold -= taken;
                if caster_alive {
                    format!("暴政施压损失{damage}生命，军资被掠夺{taken}金")
                } else {
                    format!("暴政施压损失{damage}生命；施术者遭反击阵亡，掠夺中断")
                }
            }
            BossMove::GrandAlliance(armor) => {
                let count = {
                    let b = self.battle.as_mut().expect("battle");
                    let mut count = 0;
                    for foe in &mut b.foes {
                        if foe.hp>0 {
                            foe.block += armor;
                            if phase>0 && foe.id!="yuanshao" {foe.strength += 1}
                            count +=1;
                        }
                    }
                    count
                };
                format!("四世三公列阵，{count}名敌军护甲+{armor}{}",
                    if phase>0 {"，援军武勇+1"} else {""})
            }
            BossMove::HiddenScheme(penalty) => {
                let b = self.battle.as_mut().expect("battle");
                b.next_turn_energy_penalty = b.next_turn_energy_penalty.max(penalty).min(2);
                let curses = b.draw.iter().chain(b.hand.iter()).chain(b.discard.iter())
                    .filter(|c|c.id=="cursed").count();
                if curses < 4 { b.discard.push(Card::new("cursed")); }
                format!("暗度陈仓，下回合军令-{}{}",penalty,
                    if curses<4 {"并混入军心涣散"} else {""})
            }
            BossMove::Mirage(amount) => {
                let b = self.battle.as_mut().expect("battle");
                let foe = &mut b.foes[idx];
                let restored = amount.min(foe.max_hp-foe.hp);
                foe.hp += restored;
                foe.block += amount/2;
                format!("遁甲幻身回复{restored}生命，护甲+{}",amount/2)
            }
        }
    }

    fn boss_phase_transitions(&mut self) -> Vec<String> {
        let changes: Vec<(usize,String)> = self.battle.as_ref().expect("battle").foes
            .iter().enumerate().filter(|(_,foe)|
                foe.hp > 0 && foe.phase == 0 && foe.hp * 2 <= foe.max_hp
                && matches!(foe.id.as_str(),"zhangjiao"|"sunjian"|"dongzhuo"|"yuanshao"|"simayi"|"zuoci"
                    |"elite"|"war_elephant"|"royal_guard"|"tiger_general"|"sacred_beast"))
            .map(|(i,foe)| (i,foe.id.clone())).collect();
        let mut messages = vec![];
        for (idx,id) in changes {
            let b = self.battle.as_mut().expect("battle");
            let mut summoned: Option<(&'static str, usize)> = None;
            b.foes[idx].phase = 1;
            match id.as_str() {
                "zhangjiao" | "yuanshao" | "zuoci" => {
                    let ally = match id.as_str() {
                        "zhangjiao" => "yellow_turban",
                        "yuanshao" => "officer", _ => "shaman",
                    };
                    if b.foes.len() < 4 {
                        let hp = enemy_hp(ally);
                        let new_index = b.foes.len();
                        b.foes.push(Foe {
                            id: ally.into(), hp, max_hp: hp, block: 0, strength: 0,
                            poison: 0, vulnerable: 0, weak: 0,
                            intent: Intent::Guard(4), phase: 0,
                        });
                        summoned = Some((ally, new_index));
                    }
                    if id == "zuoci" {
                        let foe = &mut b.foes[idx];
                        foe.hp = (foe.hp + 18).min(foe.max_hp);
                    }
                    messages.push(format!("{}进入第二阶段{}",
                        enemy_name(&id),
                        if summoned.is_some() {"并呼叫援军（援军下回合开始行动）"}
                        else {"，但战场编制已满，无法增援"}));
                }
                "sunjian" => {
                    b.foes[idx].strength += 4;
                    b.foes[idx].block += 8;
                    messages.push("孙坚激发虎威，获得4武勇和8护甲".into());
                }
                "dongzhuo" => {
                    b.foes[idx].strength += 3;
                    b.weak += 2;
                    self.gold = (self.gold - 30).max(0);
                    messages.push("董卓暴虐：我军虚弱两回合并损失最多30金".into());
                }
                "simayi" => {
                    b.discard.push(Card::new("cursed"));
                    b.discard.push(Card::new("cursed"));
                    b.vulnerable += 2;
                    messages.push("司马懿奇袭：两张军心涣散混入弃牌堆，我军易伤两回合".into());
                }
                "elite" => {
                    b.foes[idx].strength += 2;
                    b.foes[idx].block += 6;
                    messages.push("精锐校尉下令死战：武勇+2、护甲+6".into());
                }
                "war_elephant" => {
                    b.foes[idx].strength += 2;
                    b.foes[idx].block += 14;
                    messages.push("战象披甲冲阵：武勇+2、护甲+14".into());
                }
                "royal_guard" => {
                    b.foes[idx].strength += 1;
                    b.foes[idx].block += 10;
                    for (ally_index, ally) in b.foes.iter_mut().enumerate() {
                        if ally_index != idx && ally.hp > 0 { ally.block += 5; }
                    }
                    messages.push("禁卫整备铁壁：自身护甲+10，存活友军护甲+5".into());
                }
                "tiger_general" => {
                    b.foes[idx].strength += 3;
                    b.foes[idx].block += 5;
                    messages.push("虎将奋威：武勇+3、护甲+5，后续预告三连虎袭".into());
                }
                "sacred_beast" => {
                    let foe = &mut b.foes[idx];
                    let heal = (foe.max_hp - foe.hp).min(12);
                    foe.hp += heal;
                    foe.block += 6;
                    messages.push(format!("灵兽回生：恢复{heal}生命、护甲+6，后续加强诡术"));
                }
                _ => {}
            }
            if let Some((ally, new_index)) = summoned {
                self.combat_event(EventKind::FoeSummoned, ally, 1,
                    Some(format!("foe:{new_index}")));
            }
            if matches!(id.as_str(), "elite"|"war_elephant"|"royal_guard"|"tiger_general"|"sacred_beast") {
                self.combat_event(EventKind::EnemyTechnique, &format!("{id}_phase_two"), 1,
                    Some(format!("foe:{idx}")));
            }
        }
        messages
    }
    fn end_turn(&mut self) -> Result<String, String> {
        if self.stage != Stage::Combat { return Err("当前不在战斗中".into()); }
        self.combat_event(EventKind::TurnEnded, "hero", 0, None);
        // Retain stays in hand. Ethereal cards that were not played are exhausted;
        // other unplayed cards go to discard. Exhaust and discard are distinct events.
        let expirations = {
            let b = self.battle.as_mut().expect("battle");
            let mut retained = Vec::new();
            let mut exhausted = Vec::new();
            for card in std::mem::take(&mut b.hand) {
                if card.temporary || card_spec(&card.id).is_some_and(|spec| spec.ethereal) {
                    exhausted.push(card.id.clone());
                    b.exhausted.push(card);
                } else if card_spec(&card.id).is_some_and(|spec| spec.retain) {
                    retained.push(card);
                } else {
                    b.discard.push(card);
                }
            }
            b.hand = retained;
            exhausted
        };
        for card_id in expirations {
            self.combat_event(EventKind::CardExhausted,&card_id,1,None);
        }

        let poison_events = {
            let b = self.battle.as_mut().expect("battle");
            let mut pending = Vec::new();
            for (idx, foe) in b.foes.iter_mut().enumerate() {
                if foe.hp > 0 && foe.poison > 0 {
                    pending.push((idx, foe.poison));
                    foe.poison -= 1;
                }
            }
            pending
        };
        for (idx, damage) in poison_events {
            self.deal_enemy_damage_unblockable(idx, damage, "poison");
        }
        if self.defeated() { return Ok(self.win_battle()); }
        self.troop_attack();
        if self.defeated() { return Ok(self.win_battle()); }
        // Remove armor earned in earlier rounds, then let a phase transition
        // grant fresh armor which remains effective during the next player turn.
        if let Some(b) = self.battle.as_mut() {
            for f in &mut b.foes { f.block = 0; }
        }
        // Never execute a newly summoned foe's intent before the player has
        // seen that intent on the next turn. Existing foes act normally.
        let existing_foes = self.battle.as_ref().expect("battle").foes.len();
        let phase_messages = self.boss_phase_transitions();
        let intents: Vec<(usize, Intent)> = self.battle.as_ref().expect("battle").foes.iter()
            .enumerate().filter(|(idx, f)| *idx < existing_foes && f.hp > 0)
            .map(|(i, f)| (i, f.intent.clone())).collect();
        let mut messages = phase_messages;
        for (i, intent) in intents {
            if self.battle.as_ref().is_some_and(|b| b.foes[i].hp == 0) { continue; }
            let name = self.battle.as_ref().expect("battle").foes[i].name().to_string();

            match intent {
                Intent::Attack(power) => {
                    let damage = self.enemy_attack(i, power);
                    messages.push(format!("{name}进攻，受到{damage}点伤害"));
                }
                Intent::MultiAttack(power, hits) => {
                    let mut total = 0;
                    for _ in 0..hits {
                        if self.hp == 0 || self.battle.as_ref().expect("battle").foes[i].hp == 0 { break; }
                        total += self.enemy_attack(i, power);
                    }
                    messages.push(format!("{name}连续进攻{hits}次，受到{total}点伤害"));
                }
                Intent::Hex(weak, vulnerable) => {
                    self.apply_hero_status("weak", weak);
                    self.apply_hero_status("vulnerable", vulnerable);
                    messages.push(format!("{name}施放诡计：虚弱{weak}/易伤{vulnerable}"));
                }
                Intent::Guard(amount) => {
                    self.battle.as_mut().expect("battle").foes[i].block += amount;
                    messages.push(format!("{name}增加{amount}护甲"));
                }
                Intent::Rally(amount) => {
                    self.battle.as_mut().expect("battle").foes[i].strength += amount;
                    messages.push(format!("{name}聚势+{amount}"));
                }
                Intent::Assist(armor) => {
                    messages.push(self.ally_support(i,armor));
                }
                Intent::Boss(technique) => {
                    messages.push(format!("{name}：{}",self.boss_signature(i,technique)));
                }
            }
            if self.hp == 0 {
                self.record_metric(|c| c.battles_lost = c.battles_lost.saturating_add(1));
                self.capture_turn_metrics(true);
                self.combat_event(EventKind::BattleEnded, "defeat", 0, None);
                self.stage = Stage::Lost;
                self.battle = None;
                return Ok(format!("{}。将星陨落，征途结束。", messages.join("；")));
            }
        }
        // A lethal thorn counterattack can defeat the final foe during its own
        // action. Resolve victory after hero death checks but BEFORE attrition,
        // draw, TurnStarted, or another roll of enemy intents. This also makes
        // simultaneous lethal damage remain a player loss.
        if self.defeated() {
            return Ok(format!("{}。反击清场！{}", messages.join("；"), self.win_battle()));
        }
        // Very long battles can deadlock under entirely defensive decks:
        // Zuo Ci can repeatedly heal while both armies take no net damage.
        // Starting on enemy turn 40, supply exhaustion is an explicitly
        // logged, armor-bypassing battlefield hazard with rising cost.
        // Normal-duration battles (1..39 turns) are mechanically unchanged.
        let turn=self.battle.as_ref().expect("battle").turn;
        if turn>=40 {
            let depletion=1+((turn-40)/4) as i32;
            let actual=depletion.min(self.hp);
            self.hp-=actual;
            self.combat_event(EventKind::DamageReceived,"supply_exhaustion",actual,
                Some("hero".into()));
            messages.push(format!("持久战粮尽（第{turn}回合），失去{actual}体力"));
            if self.hp==0 {
                self.record_metric(|c| c.battles_lost = c.battles_lost.saturating_add(1));
                self.capture_turn_metrics(true);
                self.combat_event(EventKind::BattleEnded, "defeat", 0, None);
                self.stage=Stage::Lost;
                self.battle=None;
                return Ok(format!("{}。军粮枯竭，征途结束。",messages.join("；")));
            }
        }
        self.capture_turn_metrics(true);
        {
            let b = self.battle.as_mut().expect("battle");
            b.block = 0;
            b.weak = b.weak.saturating_sub(1);
            b.vulnerable = b.vulnerable.saturating_sub(1);
            let base_energy = 3 - i32::from(self.modifiers.contains(&ChallengeModifier::ThinSupplies))
                + i32::from(self.relics.iter().any(|r| r == "玉玺"));
            b.energy = (base_energy - i32::from(b.next_turn_energy_penalty)).max(0);
            b.next_turn_energy_penalty = 0;
            b.last_turn_cards = b.played_this_turn;
            b.played_this_turn = 0;
            b.combo = 0;
            b.discard_tokens = 0;
            b.thought_used = false;
            b.turn += 1;
            for foe in &mut b.foes {
                foe.vulnerable = foe.vulnerable.saturating_sub(1);
                foe.weak = foe.weak.saturating_sub(1);
            }
        }
        self.roll_intents();
        self.draw_cards(5 + usize::from(self.relics.iter().any(|r| r == "虎翼")));
        self.combat_event(EventKind::TurnStarted, "hero", 0, None);
        if self.defeated() { return Ok(self.win_battle()); }
        Ok(format!("回合结束：{}。", messages.join("；")))
    }
    fn choose_reward(&mut self, index: usize) -> Result<String, String> {
        let result = if index == 0 {
            "放弃卡牌奖励。".into()
        } else {
            let card = self.rewards.get(index - 1).ok_or("无效奖励编号")?.clone();
            let msg = format!("加入卡牌【{}】。", card.name());
            self.deck.push(card);
            msg
        };
        if self.last_node == Some(Node::Boss) {
            self.boon_choices = self.choose_boons();
            self.stage = Stage::Ancient;
        } else { self.finish_scene(); }
        Ok(result)
    }
    fn choose_boons(&mut self) -> Vec<String> {
        let mut pool: Vec<&str> = items::BOONS.iter().map(|(id,_)| *id)
            .filter(|id| !self.boons.iter().any(|owned| owned == id)).collect();
        let mut picks = vec![];
        while !pool.is_empty() && picks.len() < 3 {
            let idx = self.random(pool.len());
            picks.push(pool.remove(idx).to_string());
        }
        picks
    }
    fn choose_boon(&mut self, index: usize) -> Result<String, String> {
        if self.stage != Stage::Ancient { return Err("当前不在先贤赐福场景".into()); }
        let idx = index.checked_sub(1).ok_or("赐福编号从1开始")?;
        let id = self.boon_choices.get(idx).ok_or("无效赐福编号")?.clone();
        match id.as_str() {
            "imperial_seal" => { self.max_hp += 12; self.hp += 12; }
            "blood_oath" => { self.max_hp = (self.max_hp - 8).max(1); self.hp = self.hp.min(self.max_hp).max(1); }
            "caravan" => self.gold += 140,
            "arsenal" => {
                let mut eligible: Vec<usize> = self.deck.iter().enumerate()
                    .filter(|(_,c)| !c.upgraded && c.id != "cursed" && c.id != "quest_tulu")
                    .map(|(i,_)| i).collect();
                for _ in 0..3 {
                    if eligible.is_empty() { break; }
                    let index = self.random(eligible.len());
                    let picked = eligible.remove(index);
                    self.deck[picked].upgraded = true;
                }
            }
            "herb_garden" => {
                while self.potion_bag.len() < 3 { self.potion_bag.push("healing".into()); }
                self.potions = self.potion_bag.len() as u8;
            }
            "founder" => self.grant_relic(),
            _ => {}
        }
        self.boons.push(id.clone());
        self.finish_scene();
        Ok(format!("获得先贤赐福：{}", items::boon_name(&id)))
    }

    fn enchant_card(&mut self, index: usize, kind: &str) -> Result<String, String> {
        if self.stage != Stage::Camp { return Err("附魔必须在营寨进行".into()); }
        if !matches!(kind, "迅捷" | "锐锋" | "护身" | "明悟") {
            return Err("附魔类型：迅捷 / 锐锋 / 护身 / 明悟".into());
        }
        let idx = index.checked_sub(1).ok_or("牌组编号从1开始")?;
        let card = self.deck.get(idx).ok_or("无效牌组编号")?;
        if card.enchantment.is_some() || matches!(card.id.as_str(), "cursed" | "quest_tulu") {
            return Err("此卡无法附魔".into());
        }
        if self.gold < 35 { return Err("附魔需要35金".into()); }
        self.gold -= 35;
        self.deck[idx].enchantment = Some(kind.to_owned());
        let message = format!("附魔【{}】，获得{kind}效果。", self.deck[idx].name());
        self.finish_scene();
        Ok(message)
    }
    fn upgrade_card(&mut self, index: usize) -> Result<String, String> {
        if self.stage != Stage::Camp { return Err("只能在营寨升级卡牌".into()); }
        let idx = index.checked_sub(1).ok_or("卡牌编号从1开始")?;
        let card = self.deck.get_mut(idx).ok_or("无效卡牌编号")?;
        if card.upgraded || card.id == "cursed" { return Err("此卡不可升级".into()); }
        card.upgraded = true;
        let msg = format!("升级【{}】。", card.name());
        self.finish_scene();
        Ok(msg)
    }
    fn remove_card(&mut self, index: usize) -> Result<String, String> {
        if self.stage != Stage::Market { return Err("只能在军市移除卡牌".into()); }
        if self.deck.len() <= 1 { return Err("至少保留一张卡牌".into()); }
        let idx = index.checked_sub(1).ok_or("卡牌编号从1开始")?;
        if idx >= self.deck.len() { return Err("无效卡牌编号".into()); }
        if self.gold < 75 { return Err("移牌需要75金".into()); }
        self.gold -= 75;
        let card = self.deck.remove(idx);
        Ok(format!("消耗75金移除【{}】。", card.name()))
    }
    fn choose_camp(&mut self, index: usize) -> Result<String, String> {
        let msg = match index {
            1 => {
                if self.modifiers.contains(&ChallengeModifier::NoRest) {
                    return Err("行军不息挑战禁止营寨休息，可选择升级或操练".into());
                }
                let extra = if self.relics.iter().any(|r| r == "军粮") { 8 } else { 0 };
                let heal = (self.max_hp * 30 / 100).max(1) + extra;
                let actual = heal.min(self.max_hp - self.hp);
                self.hp += actual;
                format!("休整回复{actual}点体力。")
            }
            2 => {
                let index = self.deck.iter().position(|c| !c.upgraded && c.id != "cursed")
                    .ok_or("没有可升级的卡牌")? + 1;
                return self.upgrade_card(index);
            }
            3 => {
                self.max_hp += 2; self.hp += 2;
                "操练兵卒：最大体力+2。".into()
            }
            _ => return Err("营寨：choose 1 休整、choose 2 升级首张可升级牌、choose 3 操练；升级指定牌用 upgrade N".into()),
        };
        self.finish_scene();
        Ok(msg)
    }
    fn choose_market(&mut self, index: usize) -> Result<String, String> {
        if index == 0 { self.finish_scene(); return Ok("离开军市。".into()); }
        if (1..=3).contains(&index) {
            let idx = index - 1;
            let cost = 45 + idx as i32 * 15;
            if self.gold < cost { return Err(format!("需要{cost}金")); }
            let card = self.market.get(idx).ok_or("该商品已售出")?.clone();
            self.gold -= cost;
            self.market.remove(idx);
            // Market cards are permanent acquisitions, just like reward cards.
            // Never charge and remove a stock item without granting the card.
            let name = card.name();
            self.deck.push(card);
            return Ok(format!("购得【{name}】，花费{cost}金，已加入永久牌组。"));
        }
        match index {
            4 => {
                if self.gold < 55 || self.potions >= 3 { return Err("药剂55金；最多携带3瓶".into()); }
                self.gold -= 55;
                let id = items::POTIONS[self.random(items::POTIONS.len())].0;
                self.potion_bag.push(id.to_owned());
                self.potions = self.potion_bag.len() as u8;
                Ok(format!("购得药剂：{}", items::potion_name(id)))
            }
            5 => {
                let id = self.shop_relic.as_ref().ok_or("遗物已售罄")?.clone();
                if self.gold < 155 { return Err("遗物价格155金".into()); }
                self.gold -= 155;
                if id == "厚甲" { self.max_hp += 8; self.hp += 8; }
                self.relics.push(id.clone()); self.relic_history.push(id.clone());
                self.shop_relic = None;
                Ok(format!("购得遗物【{}】：{}", id, items::relic_name(&id)))
            }
            6 => {
                let idx = self.deck.iter().position(|c| c.id == "cursed")
                    .or_else(|| self.deck.iter().position(|c| c.id == "strike"))
                    .ok_or("无可快捷移除卡牌，用 remove N 指定其他卡牌")? + 1;
                self.remove_card(idx)
            }
            _ => Err("军市选项不存在".into()),
        }
    }

    pub fn story_options(&self) -> &'static str {
        match self.event_id {
            0 => "驿站筹粮：1 购粮恢复 / 2 火攻古卷附诅咒 / 3 劫军资",
            1 => "破庙祭祀：1 损血得遗物 / 2 恢复 / 3 离开",
            2 => "渡江船工：1 购粮 / 2 损血赚金 / 3 专属牌",
            3 => "流民投靠：1 买药 / 2 获金 / 3 中立牌",
            4 => "军师论道：1 付费升级 / 2 专属牌 / 3 离开",
            5 => "古战场：1 损血得遗物 / 2 金币附诅咒 / 3 离开",
            6 => "雨夜守寨：1 恢复 / 2 损血增加最大生命 / 3 获金币",
            7 => "奇门阵：1 免费升级 / 2 专属牌附诅咒 / 3 离开",
            8 => "粮道纠纷：1 购粮 / 2 损血换金币 / 3 领药",
            9 => "铁匠相逢：1 付费升级 / 2 中立牌 / 3 离开",
            10 => "故旧重逢：1 恢复 / 2 增加最大生命 / 3 专属牌",
            _ => "星象异变：1 损血得遗物 / 2 获金币 / 3 恢复",
        }
    }
    pub fn active_bonds(&self) -> Vec<&'static lore::Bond> {
        lore::bonds_for(self.hero, &self.lore_flags)
    }
    pub fn lore_hint(&self) -> String {
        if let Some(id) = &self.lore_current
            && let Some(b) = lore::beat(id) {
            return format!("【演义残卷·第{}回】{}——{}：{}\n  1. {}（结缘并记下因果）\n  2. {}（获得金银，错过羁绊）\n  3. 不介入",
                b.chapter,b.title,b.character,b.premise,b.vow,b.bargain);
        }
        self.story_options().to_string()
    }
    fn choose_lore(&mut self, index: usize) -> Result<String, String> {
        if !(1..=3).contains(&index) { return Err("演义事件请选1、2或3".into()); }
        let id = self.lore_current.as_deref().ok_or("没有当前演义章节")?;
        let b = lore::beat(id).ok_or("不存在的演义章节")?;
        let result = match index {
            1 => {
                if !self.lore_flags.iter().any(|f| f == b.flag) {
                    self.lore_flags.push(b.flag.into());
                }
                self.hp = (self.hp + b.reward_heal).min(self.max_hp);
                self.gold += b.reward_gold;
                self.deck.push(Card::new(b.reward_card));
                let mut narration = format!("{}：结下【{}】，获得{}金、恢复{}体力并得到【{}】。原著参考第{}回。",
                    b.title,b.flag,b.reward_gold,b.reward_heal,card_spec(b.reward_card).map_or("未知",|spec| spec.name),b.chapter);
                for egg in lore::newly_completed_eggs(self.hero,&self.lore_flags,&self.easter_unlocked) {
                    self.easter_unlocked.push(egg.id.into());
                    self.gold += egg.reward_gold;
                    self.hp = (self.hp + egg.reward_heal).min(self.max_hp);
                    narration.push_str(&format!(" 隐藏结局【{}】：{}（额外{}金、回复{}体力）",
                        egg.name,egg.hint,egg.reward_gold,egg.reward_heal));
                }
                narration
            }
            2 => {
                // Alternate choice pays immediately but never completes a future foreshadowing key.
                let loot = 30 + b.reward_gold;
                self.gold += loot;
                self.hp = (self.hp - 6).max(0);
                format!("{}：取走{}金，失去6体力；相关后续因果不会因此解锁。",b.title,loot)
            }
            _ => format!("{}：不介入，因果暂且埋入尘埃。",b.title),
        };
        self.lore_seen.push(b.id.into());
        if self.hp == 0 { self.stage = Stage::Lost; } else { self.finish_scene(); }
        Ok(result)
    }
    fn choose_story(&mut self, index: usize) -> Result<String, String> {
        if self.lore_current.is_some() { return self.choose_lore(index); }
        if !(1..=3).contains(&index) { return Err("奇遇请选 choose 1、2 或 3".into()); }
        let event = self.event_id;
        let cost = match (event,index) {
            (0,1) => 15, (2,1) => 25, (3,1) => 30,
            (4,1) => 40, (8,1) => 20, (9,1) => 40, _ => 0,
        };
        if self.gold < cost { return Err(format!("需要{cost}金")); }
        self.gold -= cost;
        let msg = match (event,index) {
            (0,1) | (2,1) | (8,1) => {
                self.hp = (self.hp + 18).min(self.max_hp); "购得粮草，回复18体力。"
            }
            (0,2) => {
                self.deck.push(Card::new("huogong")); self.deck.push(Card::new("cursed"));
                "获得火攻古卷，但沾染军心涣散。"
            }
            (0,3) => {
                self.hp = (self.hp - 8).max(0); self.gold += 65;
                "损失8体力，获得65金。"
            }
            (1,1) | (5,1) | (11,1) => {
                self.hp = (self.hp - if event == 1 { 6 } else { 10 }).max(0);
                self.grant_relic(); "献祭体力，获得遗物。"
            }
            (1,2) | (6,1) | (10,1) | (11,3) => {
                self.hp = (self.hp + 15).min(self.max_hp); "恢复15体力。"
            }
            (2,2) | (8,2) => {
                self.hp = (self.hp - 7).max(0); self.gold += 40;
                "失去7体力，得到40金。"
            }
            (4,2) | (7,2) => {
                self.deck.push(Card::new("quest_tulu"));
                if event == 7 { self.deck.push(Card::new("cursed")); }
                "得军情图录：再经历3次奇遇后蜕变为决断之牌。"
            }
            (2,3) | (10,3) => {
                let pool = hero_pool(self.hero);
                let picked = pool[self.random(pool.len())];
                self.deck.push(Card::new(picked));
                if event == 7 { self.deck.push(Card::new("cursed")); }
                "获得一张武将专属牌。"
            }
            (3,1) | (8,3) => {
                if self.potions < 3 { self.potion_bag.push("healing".into()); self.potions += 1; }
                else { self.gold += 15; }
                "获得药剂，满栏则转换为15金。"
            }
            (3,2) | (6,3) | (11,2) => {
                self.gold += if event == 11 {45} else {30}; "获得金币。"
            }
            (3,3) | (9,2) => {
                let picked = NEUTRAL_POOL[self.random(NEUTRAL_POOL.len())];
                self.deck.push(Card::new(picked));
                "获得一张中立牌。"
            }
            (4,1) | (7,1) | (9,1) => {
                if let Some(card) = self.deck.iter_mut().find(|c| !c.upgraded && c.id != "cursed") {
                    card.upgraded = true; "获得一次卡牌升级。"
                } else { self.gold += cost; "没有可升级卡牌，退回所付费用。" }
            }
            (5,2) => { self.gold += 60; self.deck.push(Card::new("cursed")); "拾得60金，但附上诅咒。" }
            (6,2) => { self.max_hp += 4; self.hp = (self.hp - 8).max(0); "最大体力加4，失去8体力。" }
            (10,2) => { self.max_hp += 3; self.hp += 3; "最大体力增加3。" }
            _ => "避开纷争，继续行军。",
        };
        if self.relics.iter().any(|r| r == "聚宝") && msg.contains("金") {
            self.gold += 10;
        }
        if self.hp == 0 { self.stage = Stage::Lost; } else { self.finish_scene(); }
        Ok(msg.into())
    }
    fn use_potion(&mut self, slot: usize, target: usize) -> Result<String, String> {
        if self.stage != Stage::Combat { return Err("只能战斗时使用药剂".into()); }
        let idx = slot.checked_sub(1).ok_or("药剂编号从1开始")?;
        let id = self.potion_bag.get(idx).ok_or("无效药剂编号")?.clone();
        let foe_index = if matches!(id.as_str(), "fire" | "poison") {
            Some(self.target_index(target)?)
        } else { None };
        self.potion_bag.remove(idx);
        self.potions = self.potion_bag.len() as u8;
        // All potions enter the same ordered combat journal as cards and powers.
        // Targeted potions were validated before consuming the inventory slot.
        let event_target = foe_index.map(|index| format!("foe:{index}"))
            .or_else(|| (id == "weak").then(|| "all_foes".to_owned()));
        self.combat_event(EventKind::PotionUsed, &id, 1, event_target);
        let label = items::potion_name(&id).to_string();
        match id.as_str() {
            "healing" => self.hp = (self.hp + 16).min(self.max_hp),
            "fire" => self.damage_enemy_fixed(foe_index.expect("target"), 22, "fire"),
            "shield" => self.battle.as_mut().expect("battle").block += 20,
            "energy" => self.battle.as_mut().expect("battle").energy += 2,
            "poison" => self.apply_foe_status(foe_index.expect("target"), "poison", 12),
            "strength" => self.battle.as_mut().expect("battle").strength += 3,
            "insight" => self.draw_cards(3),
            "weak" => {
                // Use the saturating status pipeline for every living foe.
                // This also records one StatusApplied event per affected target.
                let count = self.battle.as_ref().expect("battle").foes.len();
                for index in 0..count {
                    self.apply_foe_status(index, "weak", 3);
                }
            },
            "cleanse" => {
                self.hp = (self.hp + 8).min(self.max_hp);
                let b = self.battle.as_mut().expect("battle");
                b.weak = 0; b.vulnerable = 0;
            }
            "blood" => {
                self.hp = (self.hp - 4).max(1);
                self.battle.as_mut().expect("battle").strength += 5;
            }
            _ => return Err("未知药剂".into()),
        }
        if self.relics.iter().any(|r| r == "青囊") {
            self.hp = (self.hp + 4).min(self.max_hp);
        }
        if self.defeated() { Ok(format!("{label}。{}", self.win_battle())) }
        else { Ok(format!("使用药剂：{label}。")) }
    }
    fn apply_unchecked(&mut self, action: Action) -> Result<Vec<String>, String> {
        let msg = match action {
            Action::Choose { index } => match self.stage {
                Stage::Map => self.choose_route(index)?,
                Stage::Reward => self.choose_reward(index)?,
                Stage::Camp => self.choose_camp(index)?,
                Stage::Market => self.choose_market(index)?,
                Stage::Story => self.choose_story(index)?,
                Stage::Ancient => self.choose_boon(index)?,
                _ => return Err("当前场景无法选择".into()),
            },
            Action::Play { index, target } => self.play(index, target, None)?,
            Action::PlaySelected { index, target, selection } =>
                self.play(index, target, Some(selection))?,
            Action::EndTurn => self.end_turn()?,
            Action::Upgrade { index } => self.upgrade_card(index)?,
            Action::Enchant { index, kind } => self.enchant_card(index, &kind)?,
            Action::Discard { index } => self.discard_card(index)?,
            Action::RegisterReaction { listener } => {
                if self.mode!=RunMode::Custom || self.stage!=Stage::Combat {
                    return Err("仅自由模式战斗可动态登记事件监听器".into());
                }
                self.battle.as_mut().expect("battle").reactions.install_custom(listener.clone())
                    .map_err(str::to_owned)?;
                format!("已登记动态监听器 {}（优先级 {}，剩余次数 {}）",
                    listener.id,listener.priority,listener.charges)
            },
            Action::UnregisterReaction { id } => {
                if self.mode!=RunMode::Custom || self.stage!=Stage::Combat {
                    return Err("仅自由模式战斗可注销事件监听器".into());
                }
                self.battle.as_mut().expect("battle").reactions.remove_custom(&id)
                    .map_err(str::to_owned)?;
                format!("已注销动态监听器 {id}")
            },
            Action::SetModifier { id, enabled } => {
                if self.mode != RunMode::Custom || self.floor != 0 || self.stage != Stage::Map {
                    return Err("挑战修饰符仅能在自由模式出征前调整".into());
                }
                let modifier = ChallengeModifier::parse(&id).ok_or("未知挑战修饰符")?;
                if enabled {
                    if !self.modifiers.contains(&modifier) {
                        self.modifiers.push(modifier);
                        self.modifiers.sort();
                    }
                } else {
                    self.modifiers.retain(|entry| *entry != modifier);
                }
                format!("挑战修饰符【{}】{}", modifier.name(), if enabled {"已启用"} else {"已停用"})
            },
            Action::SetDoctrine { id } => {
                if !matches!(self.stage,Stage::Map | Stage::Camp) {
                    return Err("仅可在地图或营寨调整战法流派".into());
                }
                self.doctrine = if id=="auto" {None} else {
                    let focus=archetypes::Doctrine::parse(&id)
                        .ok_or("未知战法代码；输入 doctrine 查看可选战法")?;
                    if focus.hero()!=self.hero {
                        return Err("此战法不属于当前武将".into());
                    }
                    Some(focus)
                };
                self.doctrine_status()
            },
            Action::SetVariants { act1, act2, act3 } => {
                if self.floor != 0 || self.stage != Stage::Map {
                    return Err("只能在进入第一层前设置章节变体".into());
                }
                if [act1,act2,act3].iter().any(|n| *n > 1) {
                    return Err("每幕仅允许变体0或1".into());
                }
                self.variants = Some([act1,act2,act3]);
                format!("自定义章节已选：{act1}/{act2}/{act3}")
            },
            Action::Remove { index } => self.remove_card(index)?,
            Action::UsePotionAt { slot, target } => self.use_potion(slot, target)?,
            Action::UsePotion => self.use_potion(1, 1)?,
        };
        self.revision = self.revision.saturating_add(1);
        self.log.push(msg.clone());
        if self.log.len() > 12 { self.log.remove(0); }
        self.validate()?;
        Ok(vec![msg])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn secondary_choices_share_resolver_order_and_never_mutate_snapshot() {
        let mut game = Game::new(Hero::ZhugeLiang, 983);
        game.apply(Action::Choose { index: 1 }).unwrap();
        {
            let battle = game.battle.as_mut().unwrap();
            battle.hand = vec![Card::new("xingpan_tutor")];
            battle.draw = vec![Card::new("defend"), Card::new("strike"), Card::new("qinglong")];
        }
        let before = game.to_json().unwrap();
        let candidates = game.secondary_choices(1).unwrap().unwrap();
        assert_eq!(candidates.iter().map(|(n, c)| (*n, c.id.as_str())).collect::<Vec<_>>(),
            vec![(1, "qinglong"), (2, "strike")]);
        assert_eq!(game.to_json().unwrap(), before);
        assert!(game.secondary_choices(0).is_err());
        assert!(game.secondary_choices(2).is_err());
        assert!(game.apply(Action::PlaySelected { index: 1, target: 1, selection: 3 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::PlaySelected { index: 1, target: 1, selection: 2 }).unwrap();
        assert!(game.battle.as_ref().unwrap().hand.iter().any(|c| c.id == "strike"));

        let battle = game.battle.as_mut().unwrap();
        battle.hand = vec![Card::new("feiyu_copy")];
        battle.discard = vec![Card::new("strike"), Card::new("defend")];
        assert_eq!(game.secondary_choices(1).unwrap().unwrap().iter()
            .map(|(n, c)| (*n, c.id.as_str())).collect::<Vec<_>>(),
            vec![(1, "defend"), (2, "strike")]);
    }
    use super::*;
    #[test]
    fn invalid_economy_and_zero_max_hp_enemy_snapshots_are_rejected() {
        let mut game = Game::new(Hero::GuanYu, 96011);
        let intact = game.to_json().unwrap();
        game.gold = -1;
        assert!(Game::from_json(&game.to_json().unwrap()).unwrap_err().contains("存档属性无效"));
        assert_eq!(Game::from_json(&intact).unwrap().to_json().unwrap(), intact);
        game.gold = 90;
        game.apply(Action::Choose { index: 1 }).unwrap();
        let battle_intact = game.to_json().unwrap();
        game.battle.as_mut().unwrap().foes[0].max_hp = 0;
        assert!(Game::from_json(&game.to_json().unwrap()).unwrap_err().contains("无效战斗数据"));
        assert_eq!(Game::from_json(&battle_intact).unwrap().to_json().unwrap(), battle_intact);
    }
    #[test]
    fn market_card_purchase_adds_exactly_one_permanent_card_and_is_atomic() {
        let mut g = Game::new(Hero::GuanYu, 93401);
        g.stage = Stage::Market;
        g.last_node = Some(Node::Market);
        g.market = vec![Card::new("huogong")];
        g.gold = 90;
        let deck_before = g.deck.len();
        let revision = g.revision;
        let untouched = g.to_json().unwrap();
        assert!(g.apply(Action::Choose { index: 2 }).is_err());
        assert_eq!(g.to_json().unwrap(), untouched,
            "buying an unavailable market slot must not spend gold or RNG");
        let messages = g.apply(Action::Choose { index: 1 }).unwrap();
        assert!(messages[0].contains("已加入永久牌组"));
        assert_eq!(g.gold, 45);
        assert_eq!(g.deck.len(), deck_before + 1);
        assert_eq!(g.deck.iter().filter(|c| c.id == "huogong").count(), 1);
        assert!(g.market.is_empty());
        assert_eq!(g.revision, revision + 1);
        let snapshot = g.to_json().unwrap();
        let restored = Game::from_json(&snapshot).unwrap();
        assert_eq!(restored.to_json().unwrap(), snapshot);
        assert_eq!(restored.deck.last().unwrap().id, "huogong");
        assert!(g.apply(Action::Choose { index: 1 }).is_err());
        assert_eq!(g.to_json().unwrap(), snapshot,
            "sold-out stock cannot be charged a second time");
        g.apply(Action::Choose { index: 0 }).unwrap();
        assert_eq!(g.stage, Stage::Map);
        assert_eq!(g.deck.iter().filter(|c| c.id == "huogong").count(), 1);
    }
    #[test]
    fn market_card_purchase_rejects_insufficient_funds_without_state_drift() {
        let mut g = Game::new(Hero::ZhugeLiang, 93402);
        g.stage = Stage::Market;
        g.market = vec![Card::new("huogong")];
        g.gold = 44;
        let snapshot = g.to_json().unwrap();
        assert!(g.apply(Action::Choose { index: 1 }).is_err());
        assert_eq!(g.to_json().unwrap(), snapshot);
    }
    #[test]
    fn capped_power_and_debuff_events_report_effective_changes_only() {
        let mut g = Game::new(Hero::ZhugeLiang, 93409);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.gain_power(PowerId::WisdomOnDraw, 97);
        g.gain_power(PowerId::WisdomOnDraw, 10);
        g.gain_power(PowerId::WisdomOnDraw, 3);
        assert_eq!(g.power_stacks(PowerId::WisdomOnDraw), 99);
        let gained: Vec<_> = g.battle.as_ref().unwrap().events.iter()
            .filter(|e| e.kind == EventKind::PowerGained)
            .map(|e| e.amount).collect();
        assert_eq!(gained, vec![97, 2], "capped powers cannot emit fictitious extra stacks");

        let b = g.battle.as_mut().unwrap();
        b.weak = 254;
        b.vulnerable = 255;
        let before = b.event_seq;
        g.apply_hero_status("weak", 3);
        g.apply_hero_status("vulnerable", 3);
        let b = g.battle.as_ref().unwrap();
        assert_eq!((b.weak, b.vulnerable), (255, 255));
        let status: Vec<_> = b.events.iter().filter(|e|
            e.seq > before && e.kind == EventKind::StatusApplied).collect();
        assert_eq!(status.len(), 1);
        assert_eq!((status[0].source.as_str(), status[0].amount),
            ("weak", 1));
        let snapshot = g.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(), snapshot);
    }

    #[test]
    fn fire_potion_is_flat_blockable_damage_with_attributed_events() {
        let mut g = Game::new(Hero::GuanYu, 93410);
        g.apply(Action::Choose { index: 1 }).unwrap();
        {
            let b = g.battle.as_mut().unwrap();
            b.foes[0].max_hp = 250;
            b.foes[0].hp = 250;
            b.foes[0].block = 5;
            b.foes[0].vulnerable = 2;
            b.weak = 3;
            b.strength = 18;
            b.momentum = 30;
        }
        g.potion_bag = vec!["fire".into()];
        g.potions = 1;
        let initial_seq = g.battle.as_ref().unwrap().event_seq;
        g.apply(Action::UsePotionAt { slot: 1, target: 1 }).unwrap();
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].hp, 233, "fire should deal 22 minus 5 armor, without attack modifiers");
        assert_eq!(b.foes[0].block, 0);
        let events: Vec<_> = b.events.iter().filter(|e| e.seq > initial_seq).collect();
        assert_eq!(events[0].kind, EventKind::PotionUsed);
        assert_eq!(events[0].source, "fire");
        assert_eq!(events[1].kind, EventKind::DamageDealt);
        assert_eq!(events[1].source, "fire");
        assert_eq!(events[1].amount, 17);
        let snapshot = g.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(), snapshot);
        assert!(g.apply(Action::UsePotionAt { slot: 1, target: 1 }).is_err());
        assert_eq!(g.to_json().unwrap(), snapshot, "invalid second use must be atomic");
    }

    #[test]
    fn discard_power_and_feishi_do_not_inherit_card_attack_multipliers() {
        let mut g = Game::new(Hero::SunShangxiang, 93411);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.gain_power(PowerId::ArrowsOnDiscard, 1);
        {
            let b = g.battle.as_mut().unwrap();
            b.foes[0].max_hp = 250;
            b.foes[0].hp = 250;
            b.foes[0].block = 2;
            b.foes[0].vulnerable = 3;
            b.weak = 2;
            b.strength = 20;
            b.discard_tokens = 2;
            b.hand = vec![Card::new("defend"), Card::new("feishi")];
        }
        g.apply(Action::Discard { index: 1 }).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().foes[0].hp, 248,
            "power deals 4 flat damage minus 2 armor");
        g.apply(Action::Discard { index: 1 }).unwrap();
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].hp, 237, "second power 4 + feishi 7, neither scaled");
        let damage_events: Vec<_> = b.events.iter().filter(|e|
            e.kind == EventKind::DamageDealt && matches!(e.source.as_str(),
                "arrows_on_discard" | "feishi")).collect();
        assert_eq!(damage_events.iter().map(|e| (e.source.as_str(), e.amount)).collect::<Vec<_>>(),
            vec![("arrows_on_discard", 2), ("arrows_on_discard", 4), ("feishi", 7)]);
        let saved = g.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
    }

    #[test]
    fn fixed_damage_kill_emits_one_attributed_death_and_triggers_victory_drum() {
        let mut g = Game::new(Hero::GuanYu, 93412);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.gain_power(PowerId::VictoryDrum, 2);
        {
            let b = g.battle.as_mut().unwrap();
            b.foes[0].hp = 8;
            b.foes[0].block = 0;
            b.energy = 0;
        }
        let seq = g.battle.as_ref().unwrap().event_seq;
        g.damage_enemy_fixed(0, 22, "fire");
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].hp, 0);
        assert_eq!(b.energy, 2);
        let events: Vec<_> = b.events.iter().filter(|e| e.seq > seq).collect();
        assert_eq!(events.iter().map(|e| e.kind).collect::<Vec<_>>(),
            vec![EventKind::DamageDealt, EventKind::FoeDefeated, EventKind::PowerTriggered]);
        assert_eq!(events[0].source, "fire");
        assert_eq!(events[1].source, "fire");
        assert_eq!(events[2].source, "victory_drum");
    }

    #[test]
    fn piercing_card_ignores_armor_without_scaling_and_keeps_its_source() {
        let mut game = Game::new(Hero::GuanYu, 97501);
        game.apply(Action::Choose { index: 1 }).unwrap();
        {
            let battle = game.battle.as_mut().unwrap();
            battle.foes[0].hp = 25;
            battle.foes[0].block = 90;
            battle.foes[0].vulnerable = 3;
            battle.strength = 50;
            battle.weak = 2;
            battle.hand = vec![Card::new("jianshu")];
        }
        let cursor = game.battle.as_ref().unwrap().event_seq;
        game.apply(Action::Play { index: 1, target: 1 }).unwrap();
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].hp, 18);
        assert_eq!(battle.foes[0].block, 90);
        let hits: Vec<_> = battle.events.iter().filter(|e|
            e.seq > cursor && e.kind == EventKind::DamageDealt).collect();
        assert_eq!(hits.len(), 1);
        assert_eq!((hits[0].source.as_str(), hits[0].amount), ("pierce", 7));
        let saved = game.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
    }

    #[test]
    fn troop_damage_routes_through_settlement_and_counts_only_health_loss() {
        let mut game = Game::new(Hero::CaoCao, 97502);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.gain_power(PowerId::VictoryDrum, 2);
        {
            let battle = game.battle.as_mut().unwrap();
            battle.foes[0].hp = 7;
            battle.foes[0].block = 4;
            battle.troops_hp = 20;
            battle.troops_attack = 15;
            battle.energy = 0;
        }
        let cursor = game.battle.as_ref().unwrap().event_seq;
        game.troop_attack();
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].hp, 0);
        assert_eq!(battle.foes[0].block, 0);
        assert_eq!(game.telemetry.totals.troop_damage, 7);
        assert_eq!(battle.energy, 2);
        let events: Vec<_> = battle.events.iter().filter(|e| e.seq > cursor).collect();
        assert_eq!(events.iter().map(|e| e.kind).collect::<Vec<_>>(),
            vec![EventKind::DamageDealt, EventKind::FoeDefeated, EventKind::PowerTriggered]);
        assert_eq!(events[0].source, "troops");
        assert_eq!(events[0].amount, 7);
        let cursor_after_kill = battle.event_seq;
        game.deal_enemy_damage(0, 15, "troops");
        assert_eq!(game.telemetry.totals.troop_damage, 7, "no second hit on a corpse");
        assert_eq!(game.battle.as_ref().unwrap().event_seq, cursor_after_kill);
    }

    #[test]
    fn thorn_retaliation_observes_hit_before_unblockable_death() {
        let mut game = Game::new(Hero::GuanYu, 97503);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.gain_power(PowerId::VictoryDrum, 1);
        {
            let battle = game.battle.as_mut().unwrap();
            battle.foes[0].hp = 4;
            battle.foes[0].block = 50;
            battle.thorns = 6;
            battle.block = 0;
            battle.energy = 0;
        }
        let cursor = game.battle.as_ref().unwrap().event_seq;
        game.enemy_attack(0, 2);
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].hp, 0);
        assert_eq!(battle.foes[0].block, 50);
        assert_eq!(battle.energy, 1);
        let events: Vec<_> = battle.events.iter().filter(|e| e.seq > cursor).collect();
        assert_eq!(events.iter().map(|e| e.kind).collect::<Vec<_>>(),
            vec![EventKind::DamageReceived, EventKind::DamageDealt,
                 EventKind::FoeDefeated, EventKind::PowerTriggered]);
        assert_eq!(events[1].source, "thorns");
        assert_eq!(events[1].amount, 4);
        let cursor_after_kill = events.last().unwrap().seq;
        game.deal_enemy_damage_unblockable(0, 6, "thorns");
        assert_eq!(game.battle.as_ref().unwrap().event_seq, cursor_after_kill);
    }

    #[test]
    fn poison_tick_bypasses_armor_and_reports_exact_damage() {
        let mut game = Game::new(Hero::GuanYu, 97504);
        game.apply(Action::Choose { index: 1 }).unwrap();
        {
            let battle = game.battle.as_mut().unwrap();
            battle.foes[0].hp = 20;
            battle.foes[0].block = 10;
            battle.foes[0].poison = 3;
            battle.troops_hp = 0;
            battle.troops_attack = 0;
        }
        let cursor = game.battle.as_ref().unwrap().event_seq;
        game.end_turn().unwrap();
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].hp, 17);
        let poison_events: Vec<_> = battle.events.iter().filter(|e|
            e.seq > cursor && e.source == "poison" && e.kind == EventKind::DamageDealt).collect();
        assert_eq!(poison_events.len(), 1);
        assert_eq!(poison_events[0].amount, 3);
        assert_eq!(battle.foes[0].poison, 2);
    }

    #[test]
    fn thorn_kill_interrupts_thunder_follow_up_and_preserves_event_order() {
        let mut g = Game::new(Hero::GuanYu, 93201);
        g.apply(Action::Choose { index: 1 }).unwrap();
        {
            let b = g.battle.as_mut().unwrap();
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
            b.foes[0].phase = 1;
            b.thorns = 5;
            b.block = 0;
        }
        let before = g.battle.as_ref().unwrap().event_seq;
        let message = g.boss_signature(0, BossMove::Thunder(8));
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].hp, 0);
        assert_eq!(b.weak, 0, "a defeated caster cannot apply delayed weakness");
        assert!(message.contains("虚弱效果中断"));
        let events = b.events.iter().filter(|event| event.seq > before).collect::<Vec<_>>();
        assert_eq!(events.iter().map(|event| event.kind).collect::<Vec<_>>(),
            vec![EventKind::EnemyTechnique, EventKind::DamageReceived,
                EventKind::DamageDealt, EventKind::FoeDefeated]);
        assert_eq!(events.last().unwrap().source, "thorns");
        let snapshot = g.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(), snapshot);
    }
    #[test]
    fn thorn_kill_interrupts_tyrant_levy_before_gold_is_taken() {
        let mut g = Game::new(Hero::GuanYu, 93202);
        g.apply(Action::Choose { index: 1 }).unwrap();
        {
            let b = g.battle.as_mut().unwrap();
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
            b.foes[0].phase = 1;
            b.thorns = 5;
        }
        let gold = g.gold;
        let result = g.boss_signature(0, BossMove::TyrantLevy(8));
        assert_eq!(g.gold, gold);
        assert_eq!(g.battle.as_ref().unwrap().foes[0].hp, 0);
        assert!(result.contains("掠夺中断"));
        assert_eq!(g.battle.as_ref().unwrap().events.iter()
            .filter(|event| event.kind == EventKind::FoeDefeated && event.source == "thorns")
            .count(), 1);
    }
    #[test]
    fn mutual_lethal_attack_and_thorns_is_still_a_player_loss() {
        let mut g = Game::new(Hero::GuanYu, 93203);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.hp = 1;
        {
            let b = g.battle.as_mut().unwrap();
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
            b.foes[0].phase = 1;
            b.foes[0].intent = Intent::Attack(40);
            b.thorns = 5;
            b.block = 0;
            b.troops_hp = 0;
        }
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.hp, 0);
        assert_eq!(g.stage, Stage::Lost, "simultaneous lethal damage cannot grant victory");
    }
    #[test]
    fn last_foe_thorns_counterattack_wins_in_enemy_turn_without_phantom_round() {
        let mut g = Game::new(Hero::GuanYu, 93231);
        g.apply(Action::Choose { index: 1 }).unwrap();
        {
            let b = g.battle.as_mut().unwrap();
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
            b.foes[0].intent = Intent::Attack(0);
            b.thorns = 5;
            b.block = 0;
            b.troops_hp = 0;
        }
        let result = g.apply(Action::EndTurn).unwrap().join("");
        assert!(result.contains("反击清场"), "the CLI/TUI should explain the win: {result}");
        assert_eq!(g.stage, Stage::Reward);
        assert!(g.battle.is_none());
        assert_eq!(g.telemetry.totals.battles_won, 1);
        assert_eq!(g.telemetry.totals.battles_lost, 0);
        assert_eq!(g.telemetry.totals.turns_completed, 0,
            "the reward should not be preceded by a phantom new turn");
        assert_eq!(g.telemetry.recent_turns.last().unwrap().turn, 1);
        assert!(!g.telemetry.recent_turns.last().unwrap().completed);
        let json = g.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(), json);
    }

    #[test]
    fn last_foe_thorns_kill_precedes_lethal_turn_44_supply_attrition() {
        let mut g = Game::new(Hero::GuanYu, 93232);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.hp = 1;
        {
            let b = g.battle.as_mut().unwrap();
            b.turn = 44; // Would lose 2 health to supply attrition without early settlement.
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
            b.foes[0].intent = Intent::Attack(0);
            b.thorns = 5;
            b.block = 0;
            b.troops_hp = 0;
        }
        let result = g.apply(Action::EndTurn).unwrap().join("");
        assert!(result.contains("反击清场"));
        assert_eq!(g.stage, Stage::Reward);
        assert_eq!(g.hp, 1, "supply exhaustion must not run after the last foe dies");
        assert_eq!(g.telemetry.totals.battles_won, 1);
        assert_eq!(g.telemetry.totals.battles_lost, 0);
        assert_eq!(g.telemetry.recent_turns.last().unwrap().turn, 44);
        assert!(!g.telemetry.recent_turns.last().unwrap().completed);
        assert_eq!(g.revision, 2);
    }

    #[test]
    fn lethal_hero_damage_still_precedes_last_foe_thorns_victory_on_turn_44() {
        let mut g = Game::new(Hero::GuanYu, 93233);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.hp = 1;
        {
            let b = g.battle.as_mut().unwrap();
            b.turn = 44;
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
            b.foes[0].intent = Intent::Attack(10);
            b.thorns = 5;
            b.block = 0;
            b.troops_hp = 0;
        }
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.hp, 0);
        assert_eq!(g.stage, Stage::Lost);
        assert_eq!(g.telemetry.totals.battles_won, 0);
        assert_eq!(g.telemetry.totals.battles_lost, 1);
    }

    #[test]
    fn poison_potion_uses_status_pipeline_and_triggers_power_once() {
        let mut game = Game::new(Hero::ZhugeLiang, 8181);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.gain_power(PowerId::ToxicReversal, 2);
        game.potion_bag = vec!["poison".into()];
        game.potions = 1;
        let previous_seq = game.battle.as_ref().unwrap().event_seq;
        game.apply(Action::UsePotionAt { slot: 1, target: 1 }).unwrap();
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].poison, 14);
        let emitted = battle.events.iter().filter(|event| event.seq > previous_seq)
            .collect::<Vec<_>>();
        assert_eq!(emitted.iter().map(|event| event.kind).collect::<Vec<_>>(),
            vec![EventKind::PotionUsed, EventKind::StatusApplied,
                EventKind::PowerTriggered, EventKind::StatusApplied]);
        assert_eq!(emitted[0].source, "poison");
        assert_eq!(emitted[0].target.as_deref(), Some("foe:0"));
        assert_eq!(emitted[1].amount, 12);
        assert_eq!(emitted[2].amount, 2);
        assert_eq!(emitted[3].source, "poison:toxic_reversal");
        assert_eq!(emitted[3].amount, 2);
        assert_eq!(emitted[3].target.as_deref(), Some("foe:0"));
        assert_eq!(game.potions, 0);
        let snapshot = game.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(), snapshot);
    }
    #[test]
    fn weak_potion_saturates_and_records_only_living_targets() {
        let mut game = Game::new(Hero::GuanYu, 8182);
        game.apply(Action::Choose { index: 1 }).unwrap();
        let battle = game.battle.as_mut().unwrap();
        let foe = battle.foes[0].clone();
        battle.foes = vec![foe.clone(), foe.clone(), foe];
        battle.foes[0].weak = 254;
        battle.foes[2].hp = 0;
        game.potion_bag = vec!["weak".into()];
        game.potions = 1;
        let previous_seq = game.battle.as_ref().unwrap().event_seq;
        game.apply(Action::UsePotionAt { slot: 1, target: 999 }).unwrap();
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].weak, 255);
        assert_eq!(battle.foes[1].weak, 3);
        assert_eq!(battle.foes[2].weak, 0);
        let emitted = battle.events.iter().filter(|event| event.seq > previous_seq)
            .collect::<Vec<_>>();
        assert_eq!(emitted.len(), 3);
        assert_eq!(emitted[0].kind, EventKind::PotionUsed);
        assert_eq!(emitted[0].target.as_deref(), Some("all_foes"));
        assert_eq!(emitted[1].kind, EventKind::StatusApplied);
        assert_eq!(emitted[1].amount, 1);
        assert_eq!(emitted[2].kind, EventKind::StatusApplied);
        assert_eq!(emitted[2].amount, 3);
    }
    #[test]
    fn invalid_targeted_potion_is_atomic_including_journal_and_inventory() {
        let mut game = Game::new(Hero::GuanYu, 8183);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.potion_bag = vec!["poison".into()];
        game.potions = 1;
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::UsePotionAt { slot: 1, target: 999 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
    }
    #[test]
    fn snapshot_survives_mid_combat() {
        let mut game = Game::new(Hero::GuanYu, 2026);
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(game.stage, Stage::Combat);
        let snapshot = game.to_json().unwrap();
        let mut restored = Game::from_json(&snapshot).unwrap();
        assert_eq!(restored.rng_state, game.rng_state);
        assert_eq!(restored.battle.as_ref().unwrap().hand.len(), game.battle.as_ref().unwrap().hand.len());
        assert_eq!(restored.to_json().unwrap(), snapshot);
        game.apply(Action::EndTurn).unwrap();
        restored.apply(Action::EndTurn).unwrap();
        assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    }
    #[test]
    fn invalid_actions_do_not_spend_energy_or_advance_revision() {
        let mut game = Game::new(Hero::LuBu, 8);
        game.apply(Action::Choose { index: 1 }).unwrap();
        let previous = game.to_json().unwrap();
        assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
        assert_eq!(previous, game.to_json().unwrap());
    }
    #[test]
    fn map_has_three_boss_floors() {
        let mut game = Game::new(Hero::CaoCao, 7);
        for floor in 1..=MAX_FLOOR {
            game.floor = floor - 1;
            game.refresh_route();
            assert_eq!(game.route.iter().all(|node| *node == Node::Boss),
                floor.is_multiple_of(7));
        }
    }
    #[test]
    fn corrupt_snapshot_rejected() {
        let game = Game::new(Hero::ZhugeLiang, 2);
        let json = game.to_json().unwrap().replace("\"version\": 2", "\"version\": 99");
        assert!(Game::from_json(&json).is_err());
    }

    #[test]
    fn every_original_card_is_playable_and_serializable() {
        let classes = [Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang, Hero::CaoCao, Hero::LuBu];
        let mut all = vec!["strike", "defend"];
        all.extend(REWARD_POOL.iter().copied());
        all.extend(NEUTRAL_POOL.iter().copied());
        for hero in classes { all.extend(hero_pool(hero).iter().copied()); }
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len() + 3, content::card_count());
        for id in all {
            let mut g = Game::new(Hero::GuanYu, 42);
            g.apply(Action::Choose { index: 1 }).unwrap();
            let b = g.battle.as_mut().unwrap();
            b.energy = 99;
            b.hand = vec![Card::new(id)];
            for foe in &mut b.foes { foe.max_hp = 10000; foe.hp = 10000; }
            assert!(g.apply(Action::Play { index: 1, target: 1 }).is_ok(), "unplayable card {id}");
            let json = g.to_json().unwrap();
            assert!(Game::from_json(&json).is_ok(), "invalid snapshot after {id}");
        }
    }
    #[test]
    fn each_hero_passive_has_a_distinct_mechanical_effect() {
        let entries = [
            (Hero::GuanYu, "strike"), (Hero::ZhugeLiang, "defend"),
            (Hero::SunShangxiang, "strike"), (Hero::CaoCao, "defend"),
            (Hero::LuBu, "xuezhai"),
        ];
        for (hero, card) in entries {
            let mut g = Game::new(hero, 321);
            g.apply(Action::Choose { index: 1 }).unwrap();
            let b = g.battle.as_mut().unwrap();
            b.energy = 99;
            b.hand = vec![Card::new(card)];
            g.apply(Action::Play { index: 1, target: 1 }).unwrap();
            let b = g.battle.as_ref().unwrap();
            assert!(match hero {
                Hero::GuanYu => b.momentum > 0,
                Hero::ZhugeLiang => b.scheme > 0,
                Hero::SunShangxiang => b.combo > 0,
                Hero::CaoCao => b.command > 0,
                Hero::LuBu => b.fury > 0,
            });
        }
    }
    #[test]
    fn v1_saves_migrate_without_losing_battle_state() {
        let mut g = Game::new(Hero::SunShangxiang, 51);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let mut raw: serde_json::Value = serde_json::from_str(&g.to_json().unwrap()).unwrap();
        raw["version"] = serde_json::json!(1);
        for field in ["ascension", "event_id", "potion_bag", "shop_relic", "relic_history", "victories"] {
            raw.as_object_mut().unwrap().remove(field);
        }
        for field in ["vulnerable", "momentum", "scheme", "combo", "command", "fury", "thorns", "played"] {
            raw["battle"].as_object_mut().unwrap().remove(field);
        }
        for enemy in raw["battle"]["foes"].as_array_mut().unwrap() {
            enemy.as_object_mut().unwrap().remove("weak");
        }
        let restored = Game::from_json(&serde_json::to_string(&raw).unwrap()).unwrap();
        assert_eq!(restored.version, SAVE_VERSION);
        assert_eq!(restored.battle.as_ref().unwrap().hand.len(), g.battle.as_ref().unwrap().hand.len());
        assert_eq!(restored.potion_bag, vec!["healing".to_string()]);
    }
    #[test]
    fn relics_are_unique_and_potions_are_consumed() {
        let mut g = Game::new(Hero::CaoCao, 771);
        for _ in 0..items::RELICS.len() { g.grant_relic(); }
        let unique: std::collections::HashSet<_> = g.relics.iter().collect();
        assert_eq!(unique.len(), g.relics.len());
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.potion_bag = vec!["shield".into(), "poison".into()];
        g.potions = 2;
        g.apply(Action::UsePotionAt { slot: 1, target: 1 }).unwrap();
        assert!(g.battle.as_ref().unwrap().block >= 20);
        assert_eq!(g.potion_bag, vec!["poison".to_string()]);
        g.apply(Action::UsePotionAt { slot: 1, target: 1 }).unwrap();
        assert_eq!(g.potions, 0);
    }
    #[test]
    fn alternate_act_bosses_and_enemy_catalogue_are_valid() {
        for id in content::ENEMY_IDS {
            assert!(content::enemy_hp(id) > 0);
            assert_ne!(content::enemy_name(id), "无名敌军");
        }
        for seed in [2_u64, 3_u64] {
            let mut g = Game::new(Hero::GuanYu, seed);
            for floor in [7, 14, 21] {
                g.floor = floor;
                let foes = g.generate_foes(Node::Boss);
                assert_eq!(foes.len(), 1);
                assert!(foes[0].max_hp >= 75);
            }
        }
    }

    #[test]
    fn card_enchantments_reduce_energy_cost_and_survive_resume() {
        let mut g = Game::new(Hero::GuanYu, 33);
        g.stage = Stage::Camp;
        g.apply(Action::Enchant { index: 1, kind: "迅捷".into() }).unwrap();
        assert_eq!(g.gold, 55);
        assert_eq!(g.deck[0].enchantment.as_deref(), Some("迅捷"));
        let mut restored = Game::from_json(&g.to_json().unwrap()).unwrap();
        restored.apply(Action::Choose { index: 1 }).unwrap();
        let b = restored.battle.as_mut().unwrap();
        b.hand = vec![restored.deck[0].clone()];
        let prior_energy = b.energy;
        restored.apply(Action::Play { index: 1, target: 1 }).unwrap();
        assert_eq!(restored.battle.as_ref().unwrap().energy, prior_energy);
    }
    #[test]
    fn quest_transforms_after_three_story_nodes() {
        let mut g = Game::new(Hero::ZhugeLiang, 77);
        g.deck.push(Card::new("quest_tulu"));
        for progress in 1..=3 {
            g.route = vec![Node::Story];
            g.route_lanes = vec![0];
            g.apply(Action::Choose { index: 1 }).unwrap();
            if progress < 3 {
                assert_eq!(g.deck.last().unwrap().quest_progress, progress);
            }
            g.apply(Action::Choose { index: 3 }).unwrap();
            assert_ne!(g.stage, Stage::Combat);
        }
        assert_eq!(g.deck.last().unwrap().id, "quest_resolved");
    }

    #[test]
    fn cao_cao_troops_block_damage_and_attack() {
        let mut g = Game::new(Hero::CaoCao, 123);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.troops_hp, 12);
        assert_eq!(b.troops_attack, 2);
        let enemy_hp = b.foes[0].hp;
        g.troop_attack();
        assert_eq!(g.battle.as_ref().unwrap().foes[0].hp, enemy_hp - 2);
        let hp_before = g.hp;
        let dmg = g.enemy_attack(0, 8);
        assert_eq!(dmg, 0);
        assert_eq!(g.hp, hp_before);
        assert!(g.battle.as_ref().unwrap().troops_hp < 12);
    }
    #[test]
    fn sun_shangxiang_discard_trigger_and_resume() {
        let mut g = Game::new(Hero::SunShangxiang, 888);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let b = g.battle.as_mut().unwrap();
        b.discard_tokens = 1;
        b.hand = vec![Card::new("feishi")];
        let before = b.foes[0].hp;
        g.apply(Action::Discard { index: 1 }).unwrap();
        assert!(g.battle.as_ref().is_some_and(|b| b.foes[0].hp < before));
        assert!(Game::from_json(&g.to_json().unwrap()).is_ok());
    }

    #[test]
    fn rejected_actions_rollback_all_mutations_including_rng() {
        let mut g = Game::new(Hero::SunShangxiang, 15);
        g.stage = Stage::Market;
        g.market = g.pick_reward_cards();
        g.gold = 0;
        let before = g.to_json().unwrap();
        assert!(g.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(g.to_json().unwrap(), before);
        g.stage = Stage::Combat;
        g.start_battle(Node::Battle);
        g.potion_bag = vec!["unknown".into()];
        g.potions = 1;
        let before = g.to_json().unwrap();
        assert!(g.apply(Action::UsePotionAt { slot: 1, target: 1 }).is_err());
        assert_eq!(g.to_json().unwrap(), before);
    }

    #[test]
    fn boss_reward_flows_into_ancient_choice_without_losing_save() {
        let mut g = Game::new(Hero::GuanYu, 444);
        g.floor = 7;
        g.last_node = Some(Node::Boss);
        g.stage = Stage::Combat;
        g.battle = None;
        let _ = g.win_battle();
        assert_eq!(g.stage, Stage::Reward);
        g.apply(Action::Choose { index: 0 }).unwrap();
        assert_eq!(g.stage, Stage::Ancient);
        assert_eq!(g.boon_choices.len(), 3);
        let snap = g.to_json().unwrap();
        let mut restored = Game::from_json(&snap).unwrap();
        restored.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(restored.stage, Stage::Map);
        assert_eq!(restored.boons.len(), 1);
    }

    #[test]
    fn act_variants_select_distinct_bosses_and_save_across_resume() {
        let mut g = Game::new(Hero::GuanYu, 1);
        g.apply(Action::SetVariants { act1: 1, act2: 1, act3: 1 }).unwrap();
        for (floor,id) in [(7,"sunjian"),(14,"yuanshao"),(21,"zuoci")] {
            g.floor = floor;
            assert_eq!(g.generate_foes(Node::Boss)[0].id,id);
        }
        let before = g.to_json().unwrap();
        let loaded = Game::from_json(&before).unwrap();
        assert_eq!(loaded.to_json().unwrap(), before);
        g.floor = 1;
        assert!(g.apply(Action::SetVariants { act1:0,act2:0,act3:0 }).is_err());
    }

    #[test]
    fn six_bosses_transition_once_and_preserve_summons_in_snapshot() {
        for (id,ally) in [
            ("zhangjiao",true),("sunjian",false),("dongzhuo",false),
            ("yuanshao",true),("simayi",false),("zuoci",true),
        ] {
            let mut g = Game::new(Hero::GuanYu, 88);
            g.floor = 7;
            g.start_battle(Node::Boss);
            let b = g.battle.as_mut().unwrap();
            let hp = enemy_hp(id);
            b.foes[0] = Foe {
                id: id.into(), hp: hp/2, max_hp: hp, block: 0,
                strength: 0, poison: 0, vulnerable: 0, weak: 0,
                intent: Intent::Guard(0), phase: 0,
            };
            assert!(!g.boss_phase_transitions().is_empty());
            assert_eq!(g.battle.as_ref().unwrap().foes[0].phase,1);
            assert_eq!(g.battle.as_ref().unwrap().foes.len() > 1,ally);
            assert!(g.boss_phase_transitions().is_empty());
            let snapshot = g.to_json().unwrap();
            assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
        }
    }

    #[test]
    fn connected_map_lanes_reject_nonadjacent_moves_and_resume() {
        let mut g = Game::new(Hero::GuanYu,111);
        assert_eq!(g.route.len(),3);
        assert!(g.act_map.as_ref().is_some_and(|m| m.is_valid()));
        assert_eq!(g.route_lanes,vec![0,1,2]);
        let next = g.act_map.as_ref().unwrap().routes(1,Some(0));
        g.apply(Action::Choose { index: 1 }).unwrap();
        // New graph topology may have one or two explicit edges from lane zero.
        g.battle = None;
        g.finish_scene();
        assert_eq!(g.route_lanes,next.iter().map(|(lane,_)|*lane).collect::<Vec<_>>());
        assert_eq!(g.route,next.iter().map(|(_,node)|*node).collect::<Vec<_>>());
        let json = g.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(),json);
    }

    #[test]
    fn romance_choices_unlock_foreshadowed_future_payoffs() {
        let mut g = Game::new(Hero::GuanYu, 11);
        g.stage = Stage::Story;
        g.lore_current = Some("baima".into());
        g.apply(Action::Choose { index: 1 }).unwrap();
        assert!(g.lore_flags.contains(&"baima_debt".to_string()));
        assert!(g.active_bonds().iter().any(|bond| bond.companion == "曹操"));
        assert_eq!(lore::next_beat(g.hero,1,&g.lore_flags,&g.lore_seen).unwrap().id,"huarong");
        let snap = g.to_json().unwrap();
        let restored = Game::from_json(&snap).unwrap();
        assert_eq!(restored.to_json().unwrap(),snap);
        let mut other = Game::new(Hero::GuanYu,11);
        other.stage = Stage::Story;
        other.lore_current = Some("baima".into());
        other.apply(Action::Choose { index: 2 }).unwrap();
        assert!(lore::next_beat(other.hero,1,&other.lore_flags,&other.lore_seen).is_none());
        assert_eq!(other.active_bonds().len(),0);
    }
    #[test]
    fn lore_flag_applies_to_combat_before_first_turn() {
        let mut g = Game::new(Hero::GuanYu,12);
        g.lore_flags.push("taoyuan_oath".into());
        g.floor = 1;
        g.start_battle(Node::Battle);
        assert!(g.battle.as_ref().unwrap().block >= 10);
    }
    #[test]
    fn invalid_lore_data_and_choice_are_transactional() {
        let mut g = Game::new(Hero::ZhugeLiang,13);
        g.stage = Stage::Story;
        g.lore_current = Some("longzhong".into());
        let snapshot = g.to_json().unwrap();
        assert!(g.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(g.to_json().unwrap(), snapshot);
        g.lore_current = Some("forged_chapter".into());
        assert!(Game::from_json(&g.to_json().unwrap()).is_err());
    }

    #[test]
    fn hidden_ending_unlocked_once_when_baima_and_huarong_are_both_kept() {
        let mut game = Game::new(Hero::GuanYu,77);
        game.lore_flags.push("baima_debt".into());
        game.stage = Stage::Story;
        game.lore_current = Some("huarong".into());
        let log = game.apply(Action::Choose { index:1 }).unwrap();
        assert!(log[0].contains("隐藏结局【青龙旧债】"));
        assert_eq!(game.easter_unlocked,vec!["sword_debt".to_string()]);
        let result = game.to_json().unwrap();
        let mut resumed = Game::from_json(&result).unwrap();
        resumed.stage = Stage::Story;
        resumed.lore_current = Some("huarong".into());
        let before = resumed.gold;
        resumed.apply(Action::Choose { index:1 }).unwrap();
        assert_eq!(resumed.gold-before,16); // no duplicate hidden bonus
        assert_eq!(resumed.easter_unlocked.len(),1);
    }

    #[test]
    fn persistent_powers_stack_and_emit_ordered_events() {
        let mut g = Game::new(Hero::GuanYu, 701);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let b = g.battle.as_mut().unwrap();
        b.energy = 20;
        b.hand = vec![Card::new("oath_guard"), Card::new("oath_guard"), Card::new("strike")];
        let base_guard = b.block;
        g.apply(Action::Play { index: 1, target: 1 }).unwrap();
        g.apply(Action::Play { index: 1, target: 1 }).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().powers[0].stacks,2);
        g.apply(Action::Play { index: 1, target: 1 }).unwrap();
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.block,base_guard+6);
        let kinds = b.events.iter().map(|e|e.kind).collect::<Vec<_>>();
        assert!(kinds.contains(&EventKind::CardPlayed));
        assert!(kinds.contains(&EventKind::DamageDealt));
        assert!(kinds.contains(&EventKind::PowerTriggered));
        assert!(b.events.windows(2).all(|w|w[0].seq < w[1].seq));
        assert!(b.events.iter().any(|e|e.source=="oath_guard" && e.amount==6));
        let snap = g.to_json().unwrap();
        assert_eq!(Game::from_json(&snap).unwrap().to_json().unwrap(),snap);
    }
    #[test]
    fn advice_power_procs_only_once_each_turn() {
        let mut g = Game::new(Hero::ZhugeLiang, 702);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let b = g.battle.as_mut().unwrap();
        b.energy = 20;
        b.hand = vec![Card::new("thought_spring"), Card::new("defend"), Card::new("defend")];
        g.apply(Action::Play { index: 1, target: 1 }).unwrap();
        let triggered = g.battle.as_ref().unwrap().events.iter()
            .filter(|e|e.source=="thought_spring" && e.kind==EventKind::PowerTriggered).count();
        assert_eq!(triggered,1);
        g.apply(Action::Play { index: 1, target: 1 }).unwrap();
        let triggered = g.battle.as_ref().unwrap().events.iter()
            .filter(|e|e.source=="thought_spring" && e.kind==EventKind::PowerTriggered).count();
        assert_eq!(triggered,1);
        assert!(g.battle.as_ref().unwrap().thought_used);
        // A replayed turn boundary must reset once-per-turn procs deterministically.
        g.battle.as_mut().unwrap().thought_used = false;
        assert!(!g.battle.as_ref().unwrap().thought_used);
    }
    #[test]
    fn archery_power_only_activates_on_manual_discard() {
        let mut g = Game::new(Hero::SunShangxiang, 703);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let b = g.battle.as_mut().unwrap();
        b.energy = 20;
        b.hand = vec![Card::new("arrows_on_discard"), Card::new("defend")];
        b.discard_tokens = 1;
        g.apply(Action::Play { index: 1, target: 1 }).unwrap();
        let before = g.battle.as_ref().unwrap().foes[0].hp;
        g.apply(Action::Discard { index: 1 }).unwrap();
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].hp,before-4);
        assert!(b.events.iter().any(|e|e.kind==EventKind::CardDiscarded));
        assert!(b.events.iter().any(|e|e.source=="arrows_on_discard"));
    }
    #[test]
    fn granary_only_resupplies_living_troops_and_courage_needs_actual_damage() {
        let mut g = Game::new(Hero::CaoCao, 704);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.gain_power(PowerId::GranarySupport,2);
        g.battle.as_mut().unwrap().troops_hp = 2;
        g.trigger_granary();
        assert_eq!(g.battle.as_ref().unwrap().troops_hp,10);
        g.battle.as_mut().unwrap().troops_hp=0;
        g.trigger_granary();
        assert_eq!(g.battle.as_ref().unwrap().troops_hp,0);
        let mut lu = Game::new(Hero::LuBu,705);
        lu.apply(Action::Choose { index: 1 }).unwrap();
        lu.gain_power(PowerId::BloodCourage,1);
        lu.battle.as_mut().unwrap().block=30;
        let baseline = lu.battle.as_ref().unwrap().fury;
        lu.enemy_attack(0,5);
        assert_eq!(lu.battle.as_ref().unwrap().fury,baseline);
        lu.battle.as_mut().unwrap().block=0;
        lu.enemy_attack(0,20);
        assert!(lu.battle.as_ref().unwrap().fury >= baseline+2);
    }
    #[test]
    fn bounded_event_journal_stays_valid_through_save_roundtrip() {
        let mut g = Game::new(Hero::GuanYu, 706);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let initial_seq = g.battle.as_ref().unwrap().event_seq;
        for n in 0..135 {
            g.combat_event(EventKind::TurnStarted,"test",n,None);
        }
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.events.len(),MAX_RECENT_EVENTS);
        assert_eq!(b.event_seq,initial_seq+135);
        let snap = g.to_json().unwrap();
        assert_eq!(Game::from_json(&snap).unwrap().to_json().unwrap(),snap);
    }

    #[test]
    fn invalid_replayed_events_and_duplicate_power_ids_are_rejected() {
        let mut g = Game::new(Hero::ZhugeLiang, 707);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.gain_power(PowerId::ThoughtSpring, 1);
        let snap = g.to_json().unwrap();
        let restored = Game::from_json(&snap).unwrap();
        assert!(restored.battle.as_ref().unwrap().event_seq > 0);
        let b = g.battle.as_mut().unwrap();
        b.powers.push(b.powers[0].clone());
        assert!(Game::from_json(&g.to_json().unwrap()).is_err());
        let b = g.battle.as_mut().unwrap();
        b.powers.pop();
        b.events.push(b.events[0].clone());
        assert!(Game::from_json(&g.to_json().unwrap()).is_err());
    }

    #[test]
    fn retain_and_ethereal_cards_take_distinct_end_turn_paths() {
        let mut g = Game::new(Hero::GuanYu, 709);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let b = g.battle.as_mut().unwrap();
        b.hand = vec![Card::new("hold_line"),Card::new("fleeting_blade"),Card::new("strike")];
        b.foes[0].intent = Intent::Guard(1);
        g.apply(Action::EndTurn).unwrap();
        let b = g.battle.as_ref().unwrap();
        assert!(b.hand.iter().any(|c|c.id=="hold_line"),"保留牌必须跨越回合边界");
        assert!(b.exhausted.iter().any(|c|c.id=="fleeting_blade"),"虚无牌回合末消耗");
        assert!(!b.discard.iter().any(|c|c.id=="fleeting_blade"));
        assert_eq!(b.events.iter().filter(|e|
            e.kind==EventKind::CardExhausted && e.source=="fleeting_blade").count(),1);
        let snap=g.to_json().unwrap();
        assert_eq!(Game::from_json(&snap).unwrap().to_json().unwrap(),snap);
    }
    #[test]
    fn wisdom_shields_exactly_one_per_draw_per_stack() {
        let mut g = Game::new(Hero::ZhugeLiang, 710);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.gain_power(PowerId::WisdomOnDraw,2);
        let b = g.battle.as_mut().unwrap();
        b.draw = vec![Card::new("defend"),Card::new("strike")];
        b.discard.clear();
        let before=b.block;
        g.draw_cards(2);
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.block,before+4);
        assert_eq!(b.events.iter().filter(|e|e.kind==EventKind::CardDrawn).count(),7);
        assert!(b.events.windows(2).all(|w|w[0].seq<w[1].seq));
        let snap=g.to_json().unwrap();
        assert_eq!(Game::from_json(&snap).unwrap().to_json().unwrap(),snap);
    }
    #[test]
    fn poison_reversal_stacks_without_recursive_status_dispatch() {
        let mut g=Game::new(Hero::ZhugeLiang,711);
        g.apply(Action::Choose {index:1}).unwrap();
        g.gain_power(PowerId::ToxicReversal,2);
        g.apply_foe_status(0,"poison",3);
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].poison,5);
        let statuses=b.events.iter().filter(|e|e.kind==EventKind::StatusApplied
            && e.source=="poison").count();
        assert_eq!(statuses,1);
        assert_eq!(b.events.iter().filter(|e|e.kind==EventKind::PowerTriggered
            && e.source=="toxic_reversal").count(),1);
        assert_eq!(b.events.iter().filter(|e|e.kind==EventKind::StatusApplied
            && e.source=="poison:toxic_reversal" && e.amount==2
            && e.target.as_deref()==Some("foe:0")).count(),1);
        let poison_events: Vec<_> = b.events.iter()
            .filter(|e| e.kind == EventKind::StatusApplied && e.target.as_deref() == Some("foe:0"))
            .collect();
        assert_eq!(poison_events.iter().map(|e| e.amount).sum::<i32>(), 5);
        assert!(b.events.windows(2).all(|pair| pair[0].seq < pair[1].seq));
        let first= b.events.len();
        g.apply_foe_status(0,"weak",2);
        assert_eq!(g.battle.as_ref().unwrap().events.len(),first+1);
    }
    #[test]
    fn toxic_reversal_journal_reports_only_effective_poison_at_cap() {
        let mut g = Game::new(Hero::ZhugeLiang, 93420);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.gain_power(PowerId::ToxicReversal, 4);
        g.battle.as_mut().unwrap().foes[0].poison = 9994;
        let cursor = g.battle.as_ref().unwrap().event_seq;
        g.apply_foe_status(0, "poison", 1);
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].poison, 9999);
        let events: Vec<_> = b.events.iter().filter(|e| e.seq > cursor).collect();
        assert_eq!(events.len(), 3);
        assert_eq!((events[0].kind, events[0].source.as_str(), events[0].amount),
            (EventKind::StatusApplied, "poison", 1));
        assert_eq!((events[1].kind, events[1].source.as_str(), events[1].amount),
            (EventKind::PowerTriggered, "toxic_reversal", 4));
        assert_eq!((events[2].kind, events[2].source.as_str(), events[2].amount),
            (EventKind::StatusApplied, "poison:toxic_reversal", 4));
        assert!(events.iter().all(|e| e.target.as_deref() == Some("foe:0")));
        let snap = g.to_json().unwrap();
        assert_eq!(Game::from_json(&snap).unwrap().to_json().unwrap(), snap);
        let before_seq = g.battle.as_ref().unwrap().event_seq;
        g.apply_foe_status(0, "poison", 10);
        assert_eq!(g.battle.as_ref().unwrap().event_seq, before_seq,
            "poison at cap must not emit fictitious bonus or recursively trigger");
    }
    #[test]
    fn defeat_drum_triggers_once_for_each_enemy_not_each_hit() {
        let mut g=Game::new(Hero::GuanYu,712);
        g.apply(Action::Choose {index:1}).unwrap();
        g.gain_power(PowerId::VictoryDrum,2);
        let b=g.battle.as_mut().unwrap();
        b.foes.push(b.foes[0].clone());
        b.foes[0].hp=1;
        b.foes[0].block=0;
        let energy=b.energy;
        g.damage_enemy(0,20);
        assert_eq!(g.battle.as_ref().unwrap().energy,energy+2);
        let events=g.battle.as_ref().unwrap().events.iter()
            .filter(|e|e.kind==EventKind::FoeDefeated).count();
        assert_eq!(events,1);
        g.damage_enemy(0,20);
        assert_eq!(g.battle.as_ref().unwrap().energy,energy+2);
        assert_eq!(g.battle.as_ref().unwrap().events.iter()
            .filter(|e|e.kind==EventKind::FoeDefeated).count(),1);
    }
    #[test]
    fn thorns_poison_and_direct_pierce_record_single_defeat_event() {
        let mut g=Game::new(Hero::GuanYu,713);
        g.apply(Action::Choose {index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.foes.push(b.foes[0].clone());
        b.foes.push(b.foes[0].clone());
        b.foes[0].hp=2;
        b.foes[0].block=30;
        b.thorns=3;
        g.enemy_attack(0,5);
        assert_eq!(g.battle.as_ref().unwrap().foes[0].hp,0);
        assert_eq!(g.battle.as_ref().unwrap().events.iter().filter(|e|
            e.kind==EventKind::FoeDefeated && e.source=="thorns").count(),1);
        let b=g.battle.as_mut().unwrap();
        b.foes[1].hp=2;
        b.foes[1].block=30;
        g.apply_foe_status(1,"poison",3);
        let poison=g.battle.as_ref().unwrap().foes[1].poison;
        assert_eq!(poison,3);
        g.apply(Action::EndTurn).unwrap();
        assert!(g.battle.as_ref().unwrap().events.iter().any(|e|
            e.kind==EventKind::FoeDefeated && e.source=="poison"));
    }
    #[test]
    fn event_reactions_cannot_mutate_rejected_action_or_rng() {
        let mut g=Game::new(Hero::ZhugeLiang,714);
        g.apply(Action::Choose {index:1}).unwrap();
        g.gain_power(PowerId::WisdomOnDraw,1);
        let snap=g.to_json().unwrap();
        assert!(g.apply(Action::Play {index:999,target:1}).is_err());
        assert_eq!(g.to_json().unwrap(),snap);
    }
    #[test]
    fn x_cost_consumes_available_energy_and_scales_damage_and_guard() {
        let mut g=Game::new(Hero::GuanYu,901);
        g.apply(Action::Choose {index:1}).unwrap();
        {
            let b=g.battle.as_mut().unwrap();
            b.energy=3;
            b.foes[0].hp=500;
            b.foes[0].max_hp=500;
            b.hand=vec![Card::new("yidao_x")];
        }
        let hp=g.battle.as_ref().unwrap().foes[0].hp;
        g.apply(Action::Play {index:1,target:1}).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.energy,0);
        assert_eq!(hp-b.foes[0].hp,19); // 3*6 plus starting 1 Guan strength
        assert!(b.events.iter().any(|evt|evt.kind==EventKind::CardPlayed
            && evt.source=="yidao_x" && evt.amount==3));
        let before=b.foes[0].hp;
        let b=g.battle.as_mut().unwrap();
        b.hand=vec![Card::new("yidao_x")];
        g.apply(Action::Play {index:1,target:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().foes[0].hp,before); // zero-energy X
        let b=g.battle.as_mut().unwrap();
        b.energy=2;
        let mut card=Card::new("x_bulwark");
        card.upgraded=true;
        card.enchantment=Some("迅捷".into());
        b.hand=vec![card];
        let block=b.block;
        g.apply(Action::Play {index:1,target:1}).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.energy,0); // Swift does not discount X
        assert_eq!(b.block-block,14); // 2 * (5+2 upgrade)
    }

    #[test]
    fn tutor_accepts_explicit_top_first_choice_and_does_not_trigger_draw_power() {
        let mut g=Game::new(Hero::ZhugeLiang,902);
        g.apply(Action::Choose {index:1}).unwrap();
        g.gain_power(PowerId::WisdomOnDraw,1);
        {
            let b=g.battle.as_mut().unwrap();
            b.draw=vec![Card::new("defend"),Card::new("strike"),Card::new("qinglong")];
            b.hand=vec![Card::new("xingpan_tutor")];
            b.energy=3;
        }
        let prior=g.to_json().unwrap();
        assert!(g.apply(Action::PlaySelected {index:1,target:1,selection:99}).is_err());
        assert_eq!(g.to_json().unwrap(),prior);
        assert!(g.apply(Action::PlaySelected {index:1,target:1,selection:3}).is_err());
        assert_eq!(g.to_json().unwrap(),prior);
        let prior_draw=g.battle.as_ref().unwrap().events.iter()
            .filter(|e|e.kind==EventKind::CardDrawn).count();
        let before_guard=g.battle.as_ref().unwrap().block;
        g.apply(Action::PlaySelected {index:1,target:1,selection:2}).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert!(b.hand.iter().any(|c|c.id=="strike"));
        assert_eq!(b.draw.len(),2);
        assert_eq!(b.block,before_guard); // Tutor is not a draw
        assert_eq!(b.events.iter().filter(|e|e.kind==EventKind::CardDrawn).count(),prior_draw);
        assert!(b.events.iter().any(|e|e.kind==EventKind::CardTutored && e.source=="strike"));
        let saved=g.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(),saved);
    }

    #[test]
    fn copy_from_discard_produces_battle_only_card_and_exhausts_after_play() {
        let mut g=Game::new(Hero::SunShangxiang,903);
        g.apply(Action::Choose {index:1}).unwrap();
        {
            let b=g.battle.as_mut().unwrap();
            b.discard=vec![Card::new("strike"),Card::new("defend")];
            b.hand=vec![Card::new("feiyu_copy")];
            b.energy=5;
        }
        let deck_len=g.deck.len();
        g.apply(Action::PlaySelected {index:1,target:1,selection:2}).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert_eq!(g.deck.len(),deck_len);
        assert!(b.hand.iter().any(|c|c.id=="strike" && c.temporary));
        assert!(b.events.iter().any(|e|e.kind==EventKind::CardCopied && e.source=="strike"));
        let snap=g.to_json().unwrap();
        let mut restored=Game::from_json(&snap).unwrap();
        assert_eq!(restored.to_json().unwrap(),snap);
        restored.apply(Action::Play {index:1,target:1}).unwrap();
        let b=restored.battle.as_ref().unwrap();
        assert!(b.exhausted.iter().any(|c|c.id=="strike" && c.temporary));
        assert!(!b.discard.iter().any(|c|c.id=="strike" && c.temporary));
    }

    #[test]
    fn generated_cards_respect_hand_cap_and_exhaust_on_discard_or_end_turn() {
        let mut g=Game::new(Hero::CaoCao,904);
        g.apply(Action::Choose {index:1}).unwrap();
        {
            let b=g.battle.as_mut().unwrap();
            b.hand=vec![Card::new("muji")];
            b.energy=5;
        }
        g.apply(Action::Play {index:1,target:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().hand.iter()
            .filter(|c|c.temporary && c.id=="defend").count(),2);
        let b=g.battle.as_mut().unwrap();
        b.discard_tokens=1;
        g.apply(Action::Discard {index:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().exhausted.iter()
            .filter(|c|c.temporary && c.id=="defend").count(),1);
        let b=g.battle.as_mut().unwrap();
        b.foes[0].intent=Intent::Guard(2);
        g.apply(Action::EndTurn).unwrap();
        assert!(g.battle.as_ref().unwrap().exhausted.iter()
            .filter(|c|c.temporary && c.id=="defend").count()>=2);
        let mut h=Game::new(Hero::CaoCao,905);
        h.apply(Action::Choose {index:1}).unwrap();
        let b=h.battle.as_mut().unwrap();
        b.hand=vec![Card::new("field_recruit")];
        b.draw.clear();
        b.discard.clear();
        b.energy=5;
        h.apply(Action::Play {index:1,target:1}).unwrap();
        assert!(h.battle.as_ref().unwrap().hand.iter().any(|c|c.temporary));
        let b=h.battle.as_mut().unwrap();
        b.hand.resize(10,Card::new("defend"));
        // A full hand discards generated effects without writing bogus audit events.
        b.hand[0]=Card::new("muji");
        let seen=b.events.iter().filter(|e|e.kind==EventKind::CardGenerated).count();
        h.apply(Action::Play {index:1,target:1}).unwrap();
        assert_eq!(h.battle.as_ref().unwrap().events.iter()
            .filter(|e|e.kind==EventKind::CardGenerated).count(),seen+1);
        assert_eq!(h.battle.as_ref().unwrap().hand.len(),10);
    }

    #[test]
    fn temporary_copy_of_retained_card_exhausts_instead_of_staying_forever() {
        let mut g=Game::new(Hero::SunShangxiang,906);
        g.apply(Action::Choose {index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.hand=vec![Card::new("feiyu_copy")];
        b.discard=vec![Card::new("hold_line")];
        b.energy=4;
        b.foes[0].intent=Intent::Guard(2);
        g.apply(Action::Play {index:1,target:1}).unwrap();
        assert!(g.battle.as_ref().unwrap().hand.iter().any(|c|c.temporary && c.id=="hold_line"));
        g.apply(Action::EndTurn).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert!(!b.hand.iter().any(|c|c.temporary && c.id=="hold_line"));
        assert!(b.exhausted.iter().any(|c|c.temporary && c.id=="hold_line"));
    }

    fn test_boss(id: &str) -> Game {
        let mut g = Game::new(Hero::GuanYu, 951);
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.floor = match id {
            "zhangjiao" | "sunjian" => 7,
            "dongzhuo" | "yuanshao" => 14,
            _ => 21,
        };
        let b = g.battle.as_mut().unwrap();
        let hp = enemy_hp(id);
        let foe = &mut b.foes[0];
        foe.id = id.into();
        foe.hp = hp;
        foe.max_hp = hp;
        foe.block = 0;
        foe.weak = 0;
        foe.vulnerable = 0;
        foe.phase = 0;
        b.foes.truncate(1);
        b.block = 0;
        // This test deliberately replaces the current encounter with an act
        // boss on another floor; keep its seeded test archive coherent too.
        if let Some(events)=&mut g.event_history {
            for event in events { event.floor=g.floor; }
        }
        g.roll_intents();
        g
    }
    #[test]
    fn all_six_bosses_telegraph_unique_serializable_moves() {
        let bosses = ["zhangjiao","sunjian","dongzhuo","yuanshao","simayi","zuoci"];
        for id in bosses {
            let g = test_boss(id);
            let move_id = match &g.battle.as_ref().unwrap().foes[0].intent {
                Intent::Boss(move_id) => move_id,
                other => panic!("{id} must telegraph a unique BossMove, got {other:?}"),
            };
            assert!(matches!((id,move_id),
                ("zhangjiao",BossMove::Thunder(_))
                | ("sunjian",BossMove::TigerRush(_))
                | ("dongzhuo",BossMove::TyrantLevy(_))
                | ("yuanshao",BossMove::GrandAlliance(_))
                | ("simayi",BossMove::HiddenScheme(_))
                | ("zuoci",BossMove::Mirage(_))),"{id}");
            let saved = g.to_json().unwrap();
            let mut restored = Game::from_json(&saved).unwrap();
            assert_eq!(saved,restored.to_json().unwrap());
            let mut original = g;
            original.apply(Action::EndTurn).unwrap();
            restored.apply(Action::EndTurn).unwrap();
            assert_eq!(original.to_json().unwrap(),restored.to_json().unwrap(),
                "boss turn diverged after restoring {id}");
        }
    }
    #[test]
    fn zhangjiao_thunder_applies_weak_only_after_real_health_damage() {
        let mut g = test_boss("zhangjiao");
        g.battle.as_mut().unwrap().block=200;
        let hp = g.hp;
        g.apply(Action::EndTurn).unwrap();
        let b = g.battle.as_ref().unwrap();
        assert_eq!(g.hp,hp);
        assert_eq!(b.weak,0);
        let mut g=test_boss("zhangjiao");
        let hp=g.hp;
        g.apply(Action::EndTurn).unwrap();
        assert!(g.hp<hp);
        let b=g.battle.as_ref().unwrap();
        assert!(b.weak>0);
        assert!(b.events.iter().any(|e|e.kind==EventKind::StatusApplied
            && e.source=="weak" && e.target.as_deref()==Some("hero")));
    }
    #[test]
    fn sunjian_rush_hits_multiple_times_and_dongzhuo_levies_actual_gold() {
        let mut tiger=test_boss("sunjian");
        tiger.apply(Action::EndTurn).unwrap();
        assert!(tiger.battle.as_ref().unwrap().events.iter()
            .filter(|e|e.kind==EventKind::DamageReceived && e.source=="foe:0").count()>=2);
        let mut tyrant=test_boss("dongzhuo");
        tyrant.gold=7;
        tyrant.apply(Action::EndTurn).unwrap();
        assert_eq!(tyrant.gold,0);
        assert!(tyrant.battle.as_ref().unwrap().events.iter()
            .any(|e|e.kind==EventKind::EnemyTechnique && e.source=="dongzhuo"));
    }
    #[test]
    fn yuan_shao_alliance_grants_persistent_shield_to_living_allies() {
        let mut g=test_boss("yuanshao");
        let hp=enemy_hp("officer");
        g.battle.as_mut().unwrap().foes.push(Foe {
            id:"officer".into(), hp, max_hp:hp,block:0,strength:0,
            poison:0,vulnerable:0,weak:0,phase:0,intent:Intent::Attack(1),
        });
        g.apply(Action::EndTurn).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert!(b.foes[0].block>=9 && b.foes[1].block>=9);
        assert!(b.events.iter().any(|e|e.kind==EventKind::EnemyTechnique && e.source=="yuanshao"));
        // Killing the ally must not prevent the boss from shielding itself.
        g.battle.as_mut().unwrap().foes[1].hp=0;
        g.battle.as_mut().unwrap().foes[0].intent=Intent::Boss(BossMove::GrandAlliance(4));
        g.apply(Action::EndTurn).unwrap();
        assert!(g.battle.as_ref().unwrap().foes[0].block>=4);
        assert_eq!(g.battle.as_ref().unwrap().foes[1].block,0);
    }
    #[test]
    fn simayi_scheme_is_applied_on_next_turn_and_card_pressure_retriggers() {
        let mut g=test_boss("simayi");
        g.apply(Action::EndTurn).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.energy,2);
        assert_eq!(b.next_turn_energy_penalty,0);
        assert!(b.draw.iter().chain(b.hand.iter()).chain(b.discard.iter())
            .any(|c|c.id=="cursed"));
        // Next turn: a high-card turn forces the schemer to telegraph sabotage
        // again rather than using random intents. Saved intent survives restore.
        let b=g.battle.as_mut().unwrap();
        b.last_turn_cards=4;
        g.roll_intents();
        assert!(matches!(g.battle.as_ref().unwrap().foes[0].intent,
            Intent::Boss(BossMove::HiddenScheme(_))));
        let json=g.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(),json);
    }
    #[test]
    fn zuoci_mirage_heals_without_exceeding_maximum() {
        let mut g=test_boss("zuoci");
        let b=g.battle.as_mut().unwrap();
        b.foes[0].hp-=8;
        let original=b.foes[0].hp;
        g.apply(Action::EndTurn).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert!(b.foes[0].hp>original);
        assert_eq!(b.foes[0].hp,b.foes[0].max_hp);
        assert!(b.foes[0].block>0);
    }
    #[test]
    fn officers_coordinate_only_with_living_partners() {
        let mut g=Game::new(Hero::GuanYu, 952);
        g.apply(Action::Choose {index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        let hp=enemy_hp("officer");
        b.foes = vec![
            Foe {id:"officer".into(),hp,max_hp:hp,block:0,strength:0,poison:0,
                vulnerable:0,weak:0,intent:Intent::Attack(1),phase:0},
            Foe {id:"bandit".into(),hp:32,max_hp:32,block:0,strength:0,poison:0,
                vulnerable:0,weak:0,intent:Intent::Attack(1),phase:0}
        ];
        b.turn=2;
        g.roll_intents();
        assert!(matches!(g.battle.as_ref().unwrap().foes[0].intent,Intent::Assist(4)));
        g.apply(Action::EndTurn).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert!(b.foes[1].block>=4);
        assert!(b.events.iter().any(|e|e.source=="officer" && e.kind==EventKind::EnemyTechnique));
    }
    #[test]
    fn half_health_boss_transitions_only_once_and_retains_fresh_phase_armor() {
        let mut g=test_boss("sunjian");
        let b=g.battle.as_mut().unwrap();
        b.foes[0].hp=b.foes[0].max_hp/2;
        g.apply(Action::EndTurn).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].phase,1);
        assert!(b.foes[0].strength>=4);
        assert!(b.foes[0].block>=8);
        let strength=b.foes[0].strength;
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().foes[0].strength,strength);
    }
    #[test]
    fn old_v2_snapshots_default_new_enemy_ai_fields_and_reject_invalid_penalties() {
        let g=test_boss("simayi");
        let raw=g.to_json().unwrap();
        let mut old:serde_json::Value=serde_json::from_str(&raw).unwrap();
        let battle=old.get_mut("battle").and_then(serde_json::Value::as_object_mut).unwrap();
        battle.remove("played_this_turn");
        battle.remove("last_turn_cards");
        battle.remove("next_turn_energy_penalty");
        let migrated=Game::from_json(&old.to_string()).unwrap();
        let b=migrated.battle.as_ref().unwrap();
        assert_eq!(b.played_this_turn,0);
        assert_eq!(b.last_turn_cards,0);
        assert_eq!(b.next_turn_energy_penalty,0);
        assert!(matches!(b.foes[0].intent,Intent::Boss(BossMove::HiddenScheme(_))));
        let mut invalid=migrated;
        invalid.battle.as_mut().unwrap().next_turn_energy_penalty=3;
        assert!(Game::from_json(&invalid.to_json().unwrap()).is_err());
    }
    #[test]
    fn playing_four_cards_only_counts_current_turn_and_is_reset_after_turn_end() {
        let mut g=test_boss("simayi");
        g.battle.as_mut().unwrap().hand=vec![Card::new("guji");4];
        for _ in 0..4 {
            g.apply(Action::Play { index:1,target:1 }).unwrap();
        }
        assert_eq!(g.battle.as_ref().unwrap().played_this_turn,4);
        g.apply(Action::EndTurn).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.played_this_turn,0);
        assert_eq!(b.last_turn_cards,4);
        assert!(matches!(b.foes[0].intent,Intent::Boss(BossMove::HiddenScheme(_))));
        // The pressure is measured only over the preceding completed turn.
        g.apply(Action::EndTurn).unwrap();
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.played_this_turn,0);
        assert_eq!(b.last_turn_cards,0);
    }
    #[test]
    fn sun_three_attacks_give_guard_and_draw_once_with_rollback_safety() {
        let mut g=Game::new(Hero::SunShangxiang,7711);
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.energy=10;
        b.hand=vec![Card::new("strike");3];
        b.foes[0].hp=500;
        b.foes[0].max_hp=500;
        let prior=b.block;
        for _ in 0..3 {
            g.apply(Action::Play{index:1,target:1}).unwrap();
        }
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.combo,3);
        assert_eq!(b.block,prior+3);
        assert_eq!(b.events.iter().filter(|e|e.kind==EventKind::PowerTriggered
            && e.source=="sun_combo_guard").count(),1);
        let json=g.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(),json);
        assert!(g.apply(Action::Play{index:999,target:1}).is_err());
        assert_eq!(g.to_json().unwrap(),json);
    }
    #[test]
    fn sun_manual_discard_grants_guard_only_after_valid_token_use() {
        let mut g=Game::new(Hero::SunShangxiang,7712);
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.hand=vec![Card::new("strike")];
        let before=g.to_json().unwrap();
        assert!(g.apply(Action::Discard{index:1}).is_err());
        assert_eq!(g.to_json().unwrap(),before);
        g.battle.as_mut().unwrap().discard_tokens=1;
        let prior=g.battle.as_ref().unwrap().block;
        g.apply(Action::Discard{index:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().block,prior+2);
        assert!(g.battle.as_ref().unwrap().events.iter().any(|e|
            e.kind==EventKind::PowerTriggered && e.source=="sun_discard_guard"));
    }
    #[test]
    fn telemetry_counts_actual_troop_damage_after_armor_not_nominal_attack() {
        let mut g=Game::new(Hero::CaoCao,6001);
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.foes[0].hp=7;
        b.foes[0].max_hp=7;
        b.foes[0].block=1;
        b.troops_attack=8;
        g.troop_attack();
        assert_eq!(g.telemetry.totals.troop_damage,7);
        assert_eq!(g.telemetry.totals.troop_attacks,1);
        assert_eq!(g.telemetry.current_turn.troop_damage,7);
        assert_eq!(g.battle.as_ref().unwrap().foes[0].hp,0);
        let saved=g.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(),saved);
    }
    #[test]
    fn telemetry_tracks_troops_protection_after_guard_and_hero_health_damage() {
        let mut g=Game::new(Hero::CaoCao,6002);
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.block=4;
        b.troops_hp=3;
        let hp=g.hp;
        assert_eq!(g.enemy_attack(0,11),4);
        assert_eq!(g.hp,hp-4);
        assert_eq!(g.telemetry.totals.troop_absorbed,3);
        assert_eq!(g.telemetry.totals.hero_hp_damage,4);
        assert_eq!(g.battle.as_ref().unwrap().troops_hp,0);
        assert_eq!(g.telemetry.totals.battles_started,1);
    }
    #[test]
    fn caocao_command_four_threshold_prevents_a_three_stack_free_refund() {
        let mut g=Game::new(Hero::CaoCao,6006);
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.command=2;
        b.energy=3;
        b.hand=vec![Card::new("defend"),Card::new("defend")];
        g.apply(Action::Play{index:1,target:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().command,3);
        assert_eq!(g.telemetry.totals.command_energy,0);
        g.apply(Action::Play{index:1,target:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().command,0);
        assert_eq!(g.telemetry.totals.command_energy,1);
        assert_eq!(g.telemetry.totals.command_block,5);
    }
    #[test]
    fn telemetry_records_exact_command_energy_and_guard_from_one_proc() {
        let mut g=Game::new(Hero::CaoCao,6003);
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.command=3;
        b.energy=2;
        b.hand=vec![Card::new("defend")];
        g.apply(Action::Play{index:1,target:1}).unwrap();
        let c=g.telemetry.totals;
        assert_eq!(c.command_procs,1);
        assert_eq!(c.command_energy,1);
        assert_eq!(c.command_block,5);
        assert_eq!(c.cards_played,1);
        assert_eq!(g.battle.as_ref().unwrap().command,0);
    }
    #[test]
    fn telemetry_retains_complete_turn_indices_and_separates_partial_win() {
        let mut g=Game::new(Hero::CaoCao,6004);
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.foes[0].intent=Intent::Guard(1);
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.telemetry.totals.turns_completed,1);
        assert_eq!(g.telemetry.recent_turns.len(),1);
        assert_eq!(g.telemetry.recent_turns[0].turn,1);
        assert_eq!(g.telemetry.recent_turns[0].floor,1);
        assert!(g.telemetry.recent_turns[0].completed);
        // Mid-player-turn win should not count as an additional finished turn.
        g.battle.as_mut().unwrap().foes[0].hp=1;
        g.battle.as_mut().unwrap().foes[0].block=0;
        g.battle.as_mut().unwrap().hand=vec![Card::new("strike")];
        g.apply(Action::Play{index:1,target:1}).unwrap();
        assert_eq!(g.telemetry.totals.turns_completed,1);
        assert_eq!(g.telemetry.totals.battles_won,1);
        assert_eq!(g.telemetry.recent_turns.len(),2);
        assert_eq!(g.telemetry.recent_turns[1].turn,2);
        assert!(!g.telemetry.recent_turns[1].completed);
    }
    #[test]
    fn telemetry_defaults_old_v2_saves_and_rejects_corrupted_trace_atomically() {
        let g=Game::new(Hero::SunShangxiang,6005);
        let mut old:serde_json::Value=serde_json::from_str(&g.to_json().unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("telemetry");
        let migrated=Game::from_json(&old.to_string()).unwrap();
        assert_eq!(migrated.telemetry,CombatTelemetry::default());
        let mut bad=migrated.clone();
        bad.telemetry.recent_turns=vec![TurnTelemetry{
            floor:1,turn:1,completed:true,hero_hp_after:1,troop_hp_after:0,
            counters:CombatCounters::default(),
        };MAX_RECENT_TURN_METRICS+1];
        assert!(Game::from_json(&bad.to_json().unwrap()).is_err());
        assert_eq!(migrated.to_json().unwrap(),g.to_json().unwrap());
    }
    #[test]
    fn x_multiplier_relic_adds_virtual_energy_without_spending_it() {
        let mut g=Game::new(Hero::GuanYu,994321);
        g.relics.push("玄机炉".into());
        g.relic_history.push("玄机炉".into());
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.energy=0;
        b.strength=0; // Remove Guan Yu's innate +1, isolate the relic coefficient.
        b.hand=vec![Card::new("yidao_x"),Card::new("x_bulwark")];
        b.foes[0].hp=100;
        b.foes[0].max_hp=100;
        b.foes[0].block=0;
        let hp=g.battle.as_ref().unwrap().foes[0].hp;
        let armor=g.battle.as_ref().unwrap().block;
        g.apply(Action::Play{index:1,target:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().foes[0].hp,hp-6);
        assert_eq!(g.battle.as_ref().unwrap().energy,0);
        g.apply(Action::Play{index:1,target:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().block,armor+5);
        assert_eq!(g.battle.as_ref().unwrap().energy,0);
        let s=g.to_json().unwrap();
        assert_eq!(Game::from_json(&s).unwrap().to_json().unwrap(),s);
    }
    #[test]
    fn full_action_replay_reconstructs_seeded_combat_route_and_saves() {
        let mut g=Game::new(Hero::SunShangxiang,99131);
        assert_eq!(g.verify_action_replay(),Ok(0));
        g.apply(Action::SetDoctrine{id:"sun_combo".into()}).unwrap();
        g.apply(Action::Choose{index:1}).unwrap();
        g.apply(Action::EndTurn).unwrap();
        let snapshot=g.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().verify_action_replay(),Ok(3));
        let before=g.to_json().unwrap();
        assert!(g.apply(Action::Play{index:9999,target:1}).is_err());
        assert_eq!(g.to_json().unwrap(),before);
        assert_eq!(g.verify_action_replay(),Ok(3));
    }
    #[test]
    fn old_v2_saves_without_action_history_are_compatible_but_not_verifiable() {
        let mut game=Game::new(Hero::GuanYu,99132);
        game.apply(Action::Choose{index:1}).unwrap();
        let mut value:serde_json::Value=serde_json::from_str(&game.to_json().unwrap()).unwrap();
        value.as_object_mut().unwrap().remove("action_history");
        let old=Game::from_json(&value.to_string()).unwrap();
        assert!(old.action_history.is_none());
        assert!(old.verify_action_replay().is_err());
        // The feature must not invalidate or rewrite the campaign's state.
        assert_eq!(old.floor,game.floor);
        assert_eq!(old.rng_state,game.rng_state);
        assert!(old.to_json().is_ok());
    }
    #[test]
    fn replay_verifier_rejects_mutated_snapshot_without_touching_it() {
        let mut g=Game::new(Hero::CaoCao,99133);
        g.apply(Action::Choose{index:1}).unwrap();
        g.gold+=1;
        let before=g.to_json().unwrap();
        assert!(g.verify_action_replay().is_err());
        assert_eq!(before,g.to_json().unwrap());
    }
    #[test]
    fn doctrine_is_user_controlled_transactional_and_save_portable() {
        use archetypes::Doctrine;
        let mut g=Game::new(Hero::SunShangxiang,8001);
        assert_eq!(g.effective_doctrine(),None);
        let saved=g.to_json().unwrap();
        assert!(!saved.contains(r#""doctrine""#));
        assert!(g.apply(Action::SetDoctrine{id:"cao_troops".into()}).is_err());
        assert!(g.apply(Action::SetDoctrine{id:"unknown".into()}).is_err());
        assert_eq!(g.to_json().unwrap(),saved);
        g.apply(Action::SetDoctrine{id:"sun_discard".into()}).unwrap();
        assert_eq!(g.effective_doctrine(),Some(Doctrine::SunDiscard));
        let snapshot=g.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
        g.apply(Action::Choose{index:1}).unwrap();
        assert!(g.apply(Action::SetDoctrine{id:"sun_combo".into()}).is_err());
        let combat=g.to_json().unwrap();
        assert_eq!(g.to_json().unwrap(),combat);
        // Reversion to automatic detection does not destroy the current deck.
        g.stage=Stage::Map;
        g.battle=None;
        g.apply(Action::SetDoctrine{id:"auto".into()}).unwrap();
        assert_eq!(g.doctrine,None);
    }
    #[test]
    fn doctrine_default_rng_and_reward_selection_match_original_without_focus() {
        let mut g=Game::new(Hero::SunShangxiang,8101);
        let mut reference=g.clone();
        let hero=hero_pool(reference.hero);
        let mut expected:Vec<Card>=Vec::new();
        while expected.len()<3 {
            let roll=reference.random(100);
            let id=if roll<62 {hero[reference.random(hero.len())]}
                else if roll<85 {NEUTRAL_POOL[reference.random(NEUTRAL_POOL.len())]}
                else {REWARD_POOL[reference.random(REWARD_POOL.len())]};
            if !expected.iter().any(|c|c.id==id) {expected.push(Card::new(id));}
        }
        let selected=g.pick_reward_cards();
        assert_eq!(selected.iter().map(|c|c.id.as_str()).collect::<Vec<_>>(),
            expected.iter().map(|c|c.id.as_str()).collect::<Vec<_>>());
        assert_eq!(g.rng_state,reference.rng_state);
    }
    #[test]
    fn doctrine_guidance_is_probabilistic_duplicate_free_and_no_starting_card_grant() {
        use archetypes::Doctrine;
        let mut g=Game::new(Hero::SunShangxiang,8102);
        let deck=g.deck.len();
        g.apply(Action::SetDoctrine{id:"sun_discard".into()}).unwrap();
        let mut compatible=0;
        let mut incompatible=0;
        for _ in 0..160 {
            let choices=g.pick_reward_cards();
            assert_eq!(choices.len(),3);
            assert!(choices.iter().enumerate().all(|(i,c)|
                choices.iter().skip(i+1).all(|other|c.id!=other.id)));
            if choices.iter().any(|card|Doctrine::SunDiscard.contains(&card.id)) {
                compatible+=1;
            } else {incompatible+=1;}
            assert_eq!(g.deck.len(),deck,"Reward offer must not grant cards");
        }
        assert!(compatible>70,"reward guidance should occur, observed {compatible}");
        assert!(incompatible>10,"reward guidance must not force every screen");
    }
    #[test]
    fn inferred_doctrine_requires_two_distinct_owned_synergies() {
        use archetypes::Doctrine;
        let mut g=Game::new(Hero::SunShangxiang,8103);
        g.deck.push(Card::new("huanshe"));
        assert_eq!(g.effective_doctrine(),None);
        g.deck.push(Card::new("huanshe"));
        assert_eq!(g.effective_doctrine(),None);
        g.deck.push(Card::new("feishi"));
        assert_eq!(g.effective_doctrine(),Some(Doctrine::SunDiscard));
        g.deck.push(Card::new("liandao"));
        g.deck.push(Card::new("gonglian"));
        assert_eq!(g.effective_doctrine(),None);
        g.apply(Action::SetDoctrine{id:"sun_combo".into()}).unwrap();
        assert_eq!(g.effective_doctrine(),Some(Doctrine::SunCombo));
    }
    #[test]
    fn lengthy_battles_trigger_rising_armor_piercing_supply_exhaustion() {
        let mut g=Game::new(Hero::ZhugeLiang,8201);
        g.apply(Action::Choose{index:1}).unwrap();
        let b=g.battle.as_mut().unwrap();
        b.turn=39;
        b.block=100;
        b.foes[0].intent=Intent::Guard(0);
        let hp=g.hp;
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.hp,hp);
        let b=g.battle.as_mut().unwrap();
        b.turn=40;
        b.block=100;
        b.foes[0].intent=Intent::Guard(0);
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.hp,hp-1);
        assert!(g.telemetry.recent_turns.iter().any(|t|t.turn==40&&t.completed));
        assert!(g.battle.as_ref().unwrap().events.iter().any(|event|
            event.kind==EventKind::DamageReceived
            && event.source=="supply_exhaustion" && event.amount==1));
        let s=g.to_json().unwrap();
        assert_eq!(Game::from_json(&s).unwrap().to_json().unwrap(),s);
        let b=g.battle.as_mut().unwrap();
        b.turn=48;
        b.block=100;
        b.foes[0].intent=Intent::Guard(0);
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.hp,hp-4); // 1 on turn40 + 3 on turn48
    }
    #[test]
    fn attrition_can_end_an_unwinnable_battle_instead_of_softlocking() {
        let mut g=Game::new(Hero::ZhugeLiang,8202);
        g.apply(Action::Choose{index:1}).unwrap();
        g.hp=1;
        let b=g.battle.as_mut().unwrap();
        b.turn=40;
        b.block=99;
        b.foes[0].intent=Intent::Guard(0);
        g.apply(Action::EndTurn).unwrap();
        assert_eq!(g.stage,Stage::Lost);
        assert_eq!(g.hp,0);
        assert!(g.battle.is_none());
        assert_eq!(g.telemetry.totals.battles_lost,1);
        assert_eq!(g.telemetry.recent_turns.last().unwrap().turn,40);
    }
    #[test]
    fn doctrine_select_during_reward_cannot_reroll_existing_three_choices() {
        let mut g=Game::new(Hero::SunShangxiang,8104);
        g.apply(Action::Choose{index:1}).unwrap();
        g.battle.as_mut().unwrap().foes[0].hp=1;
        g.battle.as_mut().unwrap().foes[0].block=0;
        g.battle.as_mut().unwrap().hand=vec![Card::new("strike")];
        g.apply(Action::Play{index:1,target:1}).unwrap();
        assert_eq!(g.stage,Stage::Reward);
        let previous=g.to_json().unwrap();
        assert!(g.apply(Action::SetDoctrine{id:"sun_discard".into()}).is_err());
        assert_eq!(g.to_json().unwrap(),previous);
    }
    #[test]
    fn targeted_camp_upgrade_and_shop_removal_work() {
        let mut g = Game::new(Hero::ZhugeLiang, 8);
        g.stage = Stage::Camp;
        g.apply(Action::Upgrade { index: 10 }).unwrap();
        assert!(g.deck[9].upgraded);
        g.stage = Stage::Market;
        g.apply(Action::Remove { index: 1 }).unwrap();
        assert_eq!(g.deck.len(), 9);
        assert_eq!(g.gold, 15);
    }
}

#[cfg(test)]
mod elite_phase_regression_tests {
    use super::*;

    #[test]
    fn five_elite_phase_transitions_are_one_shot_and_serializable() {
        for id in ["elite", "war_elephant", "royal_guard", "tiger_general", "sacred_beast"] {
            let mut game = Game::new(Hero::GuanYu, 626);
            game.floor = 10;
            game.start_battle(Node::Elite);
            let battle = game.battle.as_mut().unwrap();
            battle.foes[0].id = id.into();
            battle.foes[0].max_hp = 40;
            battle.foes[0].hp = 18;
            battle.foes[0].phase = 0;
            let original_seq = battle.event_seq;
            let effects = game.boss_phase_transitions();
            assert_eq!(effects.len(), 1, "{id}");
            assert_eq!(game.battle.as_ref().unwrap().foes[0].phase, 1, "{id}");
            let b = game.battle.as_ref().unwrap();
            assert!(b.event_seq > original_seq, "{id}");
            assert!(b.events.iter().any(|e|e.kind == EventKind::EnemyTechnique
                && e.source == format!("{id}_phase_two")), "{id}");
            assert!(game.boss_phase_transitions().is_empty(), "{id} must not phase twice");
            let saved = game.to_json().unwrap();
            let loaded = Game::from_json(&saved).unwrap();
            assert_eq!(loaded.battle.unwrap().foes[0].phase, 1, "{id}");
        }
    }

    #[test]
    fn elite_special_intents_are_telegraphed_after_phase() {
        for (id, turn) in [
            ("tiger_general", 2_u32),
            ("war_elephant", 3),
            ("sacred_beast", 3),
            ("royal_guard", 2),
            ("elite", 3),
        ] {
            let mut game = Game::new(Hero::GuanYu, 888);
            game.floor = 10;
            game.start_battle(Node::Elite);
            let battle = game.battle.as_mut().unwrap();
            battle.foes[0].id = id.into();
            battle.foes[0].phase = 1;
            battle.turn = turn;
            if id == "royal_guard" {
                let hp = content::enemy_hp("officer");
                battle.foes.push(Foe {
                    id: "officer".into(), hp, max_hp: hp, block: 0, strength: 0,
                    poison: 0, vulnerable: 0, weak: 0,
                    intent: Intent::Attack(6), phase: 0,
                });
            }
            game.roll_intents();
            let intent = &game.battle.as_ref().unwrap().foes[0].intent;
            assert!(matches!((id, intent),
                ("tiger_general", Intent::MultiAttack(_, 3))
                | ("war_elephant", Intent::Guard(_))
                | ("sacred_beast", Intent::Hex(2, 1))
                | ("royal_guard", Intent::Assist(_))
                | ("elite", Intent::Rally(3))
            ), "expected phase-specific intent for {id}: {intent:?}");
        }
    }
}

#[cfg(test)]
mod summon_turn_order_p1_tests {
    use super::*;

    fn setup(id: &str, seed: u64) -> Game {
        let mut game = Game::new(Hero::GuanYu, seed);
        game.floor = 7;
        game.max_hp = 250;
        game.hp = 250;
        game.start_battle(Node::Boss);
        let boss = &mut game.battle.as_mut().unwrap().foes[0];
        boss.id = id.into();
        boss.hp = 29;
        boss.max_hp = 60;
        boss.phase = 0;
        boss.intent = Intent::Guard(5);
        game
    }

    fn foe(id: &str, intent: Intent) -> Foe {
        let hp = enemy_hp(id);
        Foe {
            id: id.into(), hp, max_hp: hp, block: 0, strength: 0,
            poison: 0, vulnerable: 0, weak: 0, intent, phase: 0,
        }
    }

    #[test]
    fn summoned_units_wait_one_turn_and_existing_units_still_act() {
        let mut game = setup("zhangjiao", 4141);
        game.battle.as_mut().unwrap().foes.push(foe("officer", Intent::Guard(5)));
        let message = game.end_turn().unwrap();
        assert!(message.contains("援军下回合开始行动"), "{message}");
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes.len(), 3);
        assert_eq!(battle.foes[1].block, 5);
        assert_eq!(battle.foes[2].block, 0);
        assert_eq!(battle.foes[2].id, "yellow_turban");
        assert!(battle.events.iter().any(|event|
            event.kind == EventKind::FoeSummoned && event.source == "yellow_turban"
                && event.target.as_deref() == Some("foe:2")));
        // Isolate the summon's action from its existing ally's shield effect.
        game.battle.as_mut().unwrap().foes[1].intent = Intent::Attack(0);
        game.battle.as_mut().unwrap().foes[2].intent = Intent::Guard(7);
        game.end_turn().unwrap();
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes.len(), 3);
        assert_eq!(battle.foes[2].block, 7);
        assert_eq!(battle.events.iter().filter(|event|event.kind == EventKind::FoeSummoned).count(), 1);
    }

    #[test]
    fn summon_cap_never_logs_a_phantom_reinforcement() {
        let mut game = setup("zhangjiao", 9144);
        for _ in 0..3 {
            game.battle.as_mut().unwrap().foes.push(foe("officer", Intent::Guard(2)));
        }
        let message = game.end_turn().unwrap();
        assert!(message.contains("无法增援"), "{message}");
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes.len(), 4);
        assert!(!battle.events.iter().any(|event|event.kind == EventKind::FoeSummoned));
    }

    #[test]
    fn summoning_bosses_share_order_and_save_contract() {
        for (boss, ally) in [
            ("zhangjiao", "yellow_turban"),
            ("yuanshao", "officer"),
            ("zuoci", "shaman"),
        ] {
            let mut game = setup(boss, 6183);
            game.end_turn().unwrap();
            let original = game.battle.as_ref().unwrap();
            assert_eq!(original.foes.len(), 2, "{boss}");
            assert_eq!(original.foes[1].id, ally, "{boss}");
            assert_eq!(original.foes[1].block, 0, "{boss}");
            let json = game.to_json().unwrap();
            let restored = Game::from_json(&json).unwrap();
            let loaded = restored.battle.unwrap();
            assert_eq!(loaded.foes[1].id, ally);
            assert_eq!(loaded.events, original.events);
            assert_eq!(loaded.event_seq, original.event_seq);
        }
    }
}

 
#[cfg(test)]
mod p1_event_cursor_tests {
    use super::*;

    fn overfill_journal() -> Game {
        let mut game = Game::new(Hero::GuanYu, 20261019);
        game.apply(Action::Choose { index: 1 }).unwrap();
        // Trigger the engine's real bounded append logic; not a synthetic Vec fixture.
        for count in 0..(MAX_RECENT_EVENTS + 23) {
            game.combat_event(EventKind::EnemyTechnique, "audit_probe", count as i32, None);
        }
        game
    }

    #[test]
    fn event_pages_are_ordered_report_eviction_and_resume_without_duplicates() {
        let game = overfill_journal();
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.events.len(), MAX_RECENT_EVENTS);
        let oldest = battle.events[0].seq;
        assert!(oldest > 1);
        let first = battle.events_after(0, 7);
        assert_eq!(first.oldest_available, oldest);
        assert_eq!(first.missed, oldest - 1);
        assert_eq!(first.entries.len(), 7);
        assert_eq!(first.entries[0].seq, oldest);
        assert_eq!(first.next_cursor, oldest + 6);
        assert!(first.has_more);
        let second = battle.events_after(first.next_cursor, 7);
        assert_eq!(second.missed, 0);
        assert_eq!(second.entries[0].seq, first.next_cursor + 1);
        assert!(first.entries.iter().all(|old|
            second.entries.iter().all(|new| old.seq != new.seq)));
        let end = battle.events_after(battle.event_seq, 7);
        assert!(end.entries.is_empty() && !end.has_more && !end.reset);
        assert_eq!(end.next_cursor, battle.event_seq);
        let reset = battle.events_after(u64::MAX, 7);
        assert!(reset.reset && reset.entries.is_empty());
        assert_eq!(reset.next_cursor, battle.event_seq);
    }

    #[test]
    fn bounded_cursor_survives_v2_save_and_rejects_corrupt_journal_order() {
        let game = overfill_journal();
        let snapshot = game.to_json().unwrap();
        let restored = Game::from_json(&snapshot).unwrap();
        let battle = restored.battle.as_ref().unwrap();
        let seq = battle.events[4].seq;
        let page = battle.events_after(seq, 4);
        assert_eq!(page.entries[0].seq, seq + 1);
        assert_eq!(page.missed, 0);
        assert!(!page.reset);
        assert_eq!(snapshot, restored.to_json().unwrap());

        let mut missing_middle = game.clone();
        missing_middle.battle.as_mut().unwrap().events[1].seq += 1;
        assert!(Game::from_json(&missing_middle.to_json().unwrap()).is_err());
        let mut mismatched_tail = game;
        mismatched_tail.battle.as_mut().unwrap().event_seq += 1;
        assert!(Game::from_json(&mismatched_tail.to_json().unwrap()).is_err());
    }


    #[test]
    fn counter_stack_resolves_before_status_and_survives_round_trip() {
        let mut g=Game::new(Hero::GuanYu, 40022);
        g.apply(Action::Choose{index:1}).unwrap();
        g.battle.as_mut().unwrap().hand.insert(0,Card::new("hold_line"));
        g.apply(Action::Play{index:1,target:1}).unwrap();
        assert_eq!(g.battle.as_ref().unwrap().reactions.listeners.len(),1);
        g.apply_hero_status("weak",2);
        assert_eq!(g.battle.as_ref().unwrap().weak,0);
        assert!(g.battle.as_ref().unwrap().events.iter()
            .any(|e|e.kind==EventKind::EventCountered));
        g.apply_hero_status("weak",2);
        assert_eq!(g.battle.as_ref().unwrap().weak,2);
        let snapshot=g.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
    }
    #[test]
    fn campaign_event_source_reconstructs_from_actions_and_rejects_gaps() {
        let mut g=Game::new(Hero::ZhugeLiang,40023);
        g.apply(Action::Choose{index:1}).unwrap();
        let source=g.reconstruct_event_history().unwrap();
        assert!(!source.is_empty());
        assert_eq!(source,g.event_history.clone().unwrap());
        let mut corrupt=g.clone();
        corrupt.event_history.as_mut().unwrap().get_mut(1).unwrap().event.seq+=2;
        assert!(Game::from_json(&corrupt.to_json().unwrap()).is_err());
        assert_eq!(g.reconstruct_event_history().unwrap(),source);
        g.event_history=None; // Legacy snapshots cannot claim a stored event archive.
        assert!(g.reconstruct_event_history().is_ok());
    }
    #[test]
    fn old_v2_empty_journal_stays_readable_without_claiming_full_audit() {
        let mut game = Game::new(Hero::GuanYu, 20261020);
        game.apply(Action::Choose { index: 1 }).unwrap();
        let battle = game.battle.as_mut().unwrap();
        battle.events.clear();
        let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        let page = restored.battle.as_ref().unwrap().events_after(0, 30);
        assert!(page.entries.is_empty());
        assert!(page.missed > 0);
        assert!(!page.reset);
    }
    #[test]
    fn all_enemy_ids_have_a_serializable_first_turn_for_all_five_classes() {
        let heroes=[Hero::GuanYu,Hero::ZhugeLiang,Hero::SunShangxiang,Hero::CaoCao,Hero::LuBu];
        let mut covered=0;
        for hero in heroes {
            for id in content::ENEMY_IDS {
                let mut g=Game::new(hero,951);
                g.apply(Action::Choose{index:1}).unwrap();
                let health=enemy_hp(id);
                {
                    let b=g.battle.as_mut().unwrap();
                    b.foes[0].id=(*id).into();
                    b.foes[0].hp=health;
                    b.foes[0].max_hp=health;
                    b.foes[0].block=0;
                    b.foes[0].phase=0;
                    b.foes.truncate(1);
                }
                g.roll_intents();
                g.apply(Action::EndTurn).unwrap_or_else(|err|
                    panic!("enemy {id} failed hero {}: {err}",hero.name()));
                let saved=g.to_json().unwrap();
                assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(),saved,
                    "enemy {id} hero {}",hero.name());
                covered+=1;
            }
        }
        assert_eq!(covered,31*5);
    }

}

#[cfg(test)]
mod p1_registered_status_interceptors {
    use super::*;
    use event_stack::{Listener,Response};

    fn prepare() -> Game {
        let mut g=Game::new_with_mode(Hero::ZhugeLiang,925145,0,RunMode::Custom);
        g.apply(Action::Choose{index:1}).unwrap();
        g
    }
    #[test]
    fn custom_reaction_actions_are_atomic_serializable_and_replayable() {
        let mut g=prepare();
        let mut rule=Listener::once("outside:weak","weak",Response::Reduce(2));
        rule.charges=2;
        g.apply(Action::RegisterReaction{listener:rule.clone()}).unwrap();
        assert_eq!(g.verify_action_replay().unwrap(),2);
        let snapshot=g.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
        let mut illegal=rule.clone();
        illegal.effect=Response::Replace(-1);
        let before=g.to_json().unwrap();
        assert!(g.apply(Action::RegisterReaction{listener:illegal}).is_err());
        assert_eq!(g.to_json().unwrap(),before);
        let mut normal=Game::new(Hero::ZhugeLiang,123);
        normal.apply(Action::Choose{index:1}).unwrap();
        assert!(normal.apply(Action::RegisterReaction{listener:rule}).is_err());
    }

    #[test]
    fn multiple_hero_reactions_write_each_intermediate_change_before_status() {
        let mut g=prepare();
        let mut first=Listener::once("half","weak",Response::Reduce(2));
        first.priority=-5;
        let mut last=Listener::once("one","weak",Response::Replace(1));
        last.priority=8;
        g.apply(Action::RegisterReaction{listener:last}).unwrap();
        g.apply(Action::RegisterReaction{listener:first}).unwrap();
        let from=g.battle.as_ref().unwrap().event_seq;
        g.apply_hero_status("weak",5);
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.weak,1);
        let events=b.events.iter().filter(|e|e.seq>from).collect::<Vec<_>>();
        assert_eq!(events.iter().map(|e|e.kind).collect::<Vec<_>>(),
            vec![EventKind::EventReplaced,EventKind::EventReplaced,EventKind::StatusApplied]);
        assert_eq!(events.iter().map(|e|e.amount).collect::<Vec<_>>(),vec![2,2,1]);
        assert_eq!(events[0].source,"half");
        assert_eq!(events[1].source,"one");
    }

    #[test]
    fn foe_poison_is_intercepted_before_toxic_reversal_and_tracks_true_delta() {
        let mut g=prepare();
        g.gain_power(PowerId::ToxicReversal,1);
        let mut rule=Listener::once("enemy:poison","poison",Response::Reduce(3));
        rule.target=Some("foe:0".into());
        g.apply(Action::RegisterReaction{listener:rule}).unwrap();
        let original=g.battle.as_ref().unwrap().foes[0].poison;
        let from=g.battle.as_ref().unwrap().event_seq;
        g.apply_foe_status(0,"poison",5);
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].poison,original+3);
        let events=b.events.iter().filter(|e|e.seq>from).collect::<Vec<_>>();
        assert_eq!(events.iter().map(|e|e.kind).collect::<Vec<_>>(),
            vec![EventKind::EventReplaced,EventKind::StatusApplied,
                 EventKind::PowerTriggered,EventKind::StatusApplied]);
        assert_eq!(events.iter().map(|e|e.amount).collect::<Vec<_>>(),vec![3,2,1,1]);
        assert_eq!(events[3].source,"poison:toxic_reversal");
        assert!(g.battle.as_ref().unwrap().reactions.listeners.is_empty());
        let json=g.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(),json);
    }
}


#[cfg(test)]
mod p1_enemy_potion_matrix_regression {
    use super::*;

    const HEROES: [Hero; 5] = [
        Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang, Hero::CaoCao, Hero::LuBu,
    ];
    const SECOND_PHASE: [&str; 11] = [
        "elite", "war_elephant", "royal_guard", "tiger_general", "sacred_beast",
        "zhangjiao", "sunjian", "dongzhuo", "yuanshao", "simayi", "zuoci",
    ];

    fn versus(hero: Hero, foe_id: &str, ascension: u8, seed: u64) -> Game {
        let mut game=Game::new_with_ascension(hero,seed,ascension);
        game.apply(Action::Choose{index:1}).expect("enter the first battle");
        game.max_hp=5000;
        game.hp=5000;
        {
            let b=game.battle.as_mut().expect("battle after route");
            b.foes.truncate(1);
            let foe=&mut b.foes[0];
            foe.id=foe_id.to_owned();
            foe.hp=enemy_hp(foe_id);
            foe.max_hp=foe.hp;
            foe.block=0;
            foe.poison=0;
            foe.weak=0;
            foe.vulnerable=0;
            foe.phase=0;
        }
        game.roll_intents();
        game
    }

    #[test]
    fn every_enemy_and_boss_phase_resolves_three_real_turns_for_each_class_at_low_and_high_ascension() {
        assert_eq!(content::ENEMY_IDS.len(),31,"update matrix when enemy roster changes");
        let mut cases=0;
        let mut turns=0;
        for hero in HEROES {
            for foe in content::ENEMY_IDS {
                for ascension in [0_u8,10_u8] {
                    let phases:&[u8]=if SECOND_PHASE.contains(foe) {&[0,1]} else {&[0]};
                    for phase in phases {
                        let mut game=versus(hero,foe,ascension,970_000+cases as u64);
                        game.battle.as_mut().expect("combat").foes[0].phase=*phase;
                        game.roll_intents();
                        for index in 0..3 {
                            assert_eq!(game.stage,Stage::Combat,
                                "premature encounter exit: {} / {foe} / asc {ascension} / phase {phase} / turn {index}",hero.name());
                            let previous=game.battle.as_ref().expect("combat").event_seq;
                            game.apply(Action::EndTurn).unwrap_or_else(|err|panic!(
                                "{} / {foe} / asc {ascension} / phase {phase} / turn {index}: {err}",hero.name()));
                            let b=game.battle.as_ref().expect("combat must persist");
                            assert!(b.event_seq>previous, "missing turn events for {foe}");
                            assert!(b.events.windows(2).all(|pair|pair[0].seq+1==pair[1].seq),
                                "unordered event journal: {foe}");
                            assert!(b.foes.iter().all(|foe|foe.hp>=0 && foe.hp<=foe.max_hp),
                                "invalid foe state: {foe}");
                            let snapshot=game.to_json().unwrap();
                            let restored=Game::from_json(&snapshot).unwrap_or_else(|err|panic!(
                                "enemy {foe} / phase {phase} / turn {index} invalid save: {err}"));
                            assert_eq!(restored.to_json().unwrap(),snapshot);
                            turns+=1;
                        }
                        cases+=1;
                    }
                }
            }
        }
        assert_eq!(cases,420);
        assert_eq!(turns,1260);
    }

    #[test]
    fn all_ten_potions_resolve_at_each_enemy_for_every_hero_without_corrupting_saves() {
        assert_eq!(items::POTIONS.len(),10,"update the exhaustive potion roster");
        let mut cases=0;
        for hero in HEROES {
            for foe in content::ENEMY_IDS {
                for (potion,_) in items::POTIONS {
                    let mut game=versus(hero,foe,0,985_000+cases as u64);
                    game.potion_bag=vec![(*potion).into()];
                    game.potions=1;
                    let previous=game.battle.as_ref().expect("battle").event_seq;
                    let revision=game.revision;
                    game.apply(Action::UsePotionAt{slot:1,target:1}).unwrap_or_else(|err|panic!(
                        "{} against {foe} using {potion}: {err}",hero.name()));
                    assert_eq!(game.revision,revision+1);
                    assert!(game.potion_bag.is_empty() && game.potions==0,
                        "potion {potion} was not consumed for {foe}");
                    if let Some(b)=&game.battle {
                        assert!(b.event_seq>previous, "potion {potion} has no journal entry");
                        assert!(b.events.iter().any(|e|e.kind==EventKind::PotionUsed
                            && e.source==*potion), "missing potion provenance: {potion}");
                    }
                    let snapshot=game.to_json().unwrap();
                    assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot,
                        "potion {potion} against {foe} corrupts save");
                    cases+=1;
                }
            }
        }
        assert_eq!(cases,1550);
    }
}

#[cfg(test)]
mod reaction_damage_tests;

#[cfg(test)]
mod p1_incoming_damage_reaction_regression {
    use super::*;
    use crate::event_stack::{Listener, Response};

    fn incoming(id: &str, index: usize, effect: Response) -> Listener {
        Listener {
            id: id.into(),
            kind: EventKind::DamageReceived,
            source: format!("foe:{index}"),
            target: Some("hero".into()),
            priority: 0,
            charges: 1,
            effect,
        }
    }
    fn custom() -> Game {
        let mut g = Game::new_with_mode(Hero::CaoCao, 951, 0, RunMode::Custom);
        g.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(g.stage, Stage::Combat);
        g
    }
    #[test]
    fn incoming_cancel_does_not_spend_armor_or_troops_or_reflect_thorns() {
        let mut g=custom();
        {
            let b=g.battle.as_mut().unwrap();
            b.block=10;
            b.troops_hp=8;
            b.thorns=4;
        }
        g.apply(Action::RegisterReaction { listener:
            incoming("parry",0,Response::Cancel) }).unwrap();
        let hp=g.hp;
        let foe_hp=g.battle.as_ref().unwrap().foes[0].hp;
        let start=g.battle.as_ref().unwrap().event_seq;
        assert_eq!(g.enemy_attack(0,25),0);
        let b=g.battle.as_ref().unwrap();
        assert_eq!(g.hp,hp);
        assert_eq!(b.block,10);
        assert_eq!(b.troops_hp,8);
        assert_eq!(b.foes[0].hp,foe_hp);
        assert!(b.reactions.listeners.is_empty());
        let events=b.events.iter().filter(|e|e.seq>start).collect::<Vec<_>>();
        assert_eq!(events.len(),1);
        assert_eq!(events[0].kind,EventKind::EventCountered);
        assert_eq!(events[0].source,"parry");

        // Expired listeners cannot cancel the next hit; original protection
        // and reflection semantics remain intact after the cancelled proposal.
        assert_eq!(g.enemy_attack(0,25),7);
        let b=g.battle.as_ref().unwrap();
        assert_eq!(g.hp,hp-7);
        assert_eq!(b.block,0);
        assert_eq!(b.troops_hp,0);
        assert_eq!(b.foes[0].hp,foe_hp-4);
    }

    #[test]
    fn ordered_incoming_replacements_precede_true_hp_damage() {
        let mut g=custom();
        {
            let b=g.battle.as_mut().unwrap();
            b.block=3;
            b.troops_hp=2;
            b.thorns=0;
        }
        let mut a=incoming("a",0,Response::Reduce(5));
        a.priority=-2;
        a.charges=2;
        g.apply(Action::RegisterReaction { listener:a }).unwrap();
        let mut z=incoming("z",0,Response::Replace(8));
        z.priority=2;
        g.apply(Action::RegisterReaction { listener:z }).unwrap();
        let hp=g.hp;
        let start=g.battle.as_ref().unwrap().event_seq;
        assert_eq!(g.enemy_attack(0,20),3);
        assert_eq!(g.hp,hp-3);
        let b=g.battle.as_ref().unwrap();
        assert_eq!(b.block,0);
        assert_eq!(b.troops_hp,0);
        let events=b.events.iter().filter(|e|e.seq>start).collect::<Vec<_>>();
        assert_eq!(events.iter().map(|e|e.kind).collect::<Vec<_>>(),
            vec![EventKind::EventReplaced,EventKind::EventReplaced,
                EventKind::DamageReceived]);
        assert_eq!(events.iter().map(|e|e.amount).collect::<Vec<_>>(),
            vec![5,7,3]);
        assert_eq!(b.reactions.listeners.len(),1);
        assert_eq!(b.reactions.listeners[0].charges,1);
        let saved=g.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(),saved);
    }

    #[test]
    fn incoming_reactions_reject_arbitrary_sources_and_ranked_modes_atomically() {
        let mut g=custom();
        let valid=incoming("ward",3,Response::Cancel);
        let original=g.to_json().unwrap();
        for invalid in [
            Listener{source:"supply_exhaustion".into(),..valid.clone()},
            Listener{source:"foe:4".into(),..valid.clone()},
            Listener{target:Some("foe:0".into()),..valid.clone()},
        ] {
            assert!(g.apply(Action::RegisterReaction{listener:invalid}).is_err());
            assert_eq!(g.to_json().unwrap(),original);
        }
        g.apply(Action::RegisterReaction{listener:valid}).unwrap();
        let mut ranked=Game::new(Hero::CaoCao,951);
        ranked.apply(Action::Choose{index:1}).unwrap();
        let baseline=ranked.to_json().unwrap();
        assert!(ranked.apply(Action::RegisterReaction{
            listener:incoming("ward",0,Response::Cancel)}).is_err());
        assert_eq!(ranked.to_json().unwrap(),baseline);
    }

    #[test]
    fn custom_incoming_listener_survives_seed_action_and_event_rebuild() {
        // Find a seed with a first-foe Attack intent; no injected battle fields,
        // so this checks the actual deterministic action journal.
        let mut found=None;
        for seed in 1..=128 {
            let mut g=Game::new_with_mode(Hero::CaoCao,seed,0,RunMode::Custom);
            g.apply(Action::Choose{index:1}).unwrap();
            if matches!(g.battle.as_ref().unwrap().foes[0].intent,Intent::Attack(_)) {
                found=Some(g);
                break;
            }
        }
        let mut g=found.expect("at least one seed attacks on turn one");
        g.apply(Action::RegisterReaction{
            listener:incoming("seeded_parry",0,Response::Cancel),
        }).unwrap();
        g.apply(Action::EndTurn).unwrap();
        assert!(g.event_history.as_ref().unwrap().iter()
            .any(|e|e.event.kind==EventKind::EventCountered
                && e.event.source=="seeded_parry"));
        assert_eq!(g.reconstruct_event_history().unwrap(),
            g.event_history.clone().unwrap());
        let saved=g.to_json().unwrap();
        assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(),saved);
    }
}

#[cfg(test)]
mod p1_saturated_status_listener_regression {
    use super::*;
    use crate::event_stack::{Listener, Response};

    #[test]
    fn one_shot_status_reactions_survive_saturated_hero_and_foe_targets() {
        for (source, target, ceiling) in [
            ("weak", "hero", 255i32),
            ("vulnerable", "hero", 255),
            ("weak", "foe:0", 255),
            ("vulnerable", "foe:0", 255),
            ("poison", "foe:0", 9999),
        ] {
            let mut g=Game::new_with_mode(Hero::ZhugeLiang, 9011, 0, RunMode::Custom);
            g.apply(Action::Choose {index:1}).unwrap();
            let mut listener=Listener::once("unspent", source, Response::Cancel);
            listener.target=Some(target.into());
            g.apply(Action::RegisterReaction {listener}).unwrap();
            {
                let b=g.battle.as_mut().unwrap();
                if target=="hero" {
                    if source=="weak" {b.weak=u8::MAX;} else {b.vulnerable=u8::MAX;}
                } else {
                    let foe=&mut b.foes[0];
                    match source {
                        "weak"=>foe.weak=u8::MAX,
                        "vulnerable"=>foe.vulnerable=u8::MAX,
                        _=>foe.poison=9999,
                    }
                }
            }
            let start_seq=g.battle.as_ref().unwrap().event_seq;
            if target=="hero" {
                g.apply_hero_status(source,5);
            } else {
                g.apply_foe_status(0,source,5);
            }
            let b=g.battle.as_ref().unwrap();
            assert_eq!(b.event_seq,start_seq,"phantom event at cap {source}/{target}");
            assert_eq!(b.reactions.listeners.len(),1,
                "saturated status consumed listener {source}/{target}");

            // Lower cap, retry; the *same* listener must now fire once.
            {
                let b=g.battle.as_mut().unwrap();
                if target=="hero" {
                    if source=="weak" {b.weak=254;} else {b.vulnerable=254;}
                } else {
                    let foe=&mut b.foes[0];
                    match source {
                        "weak"=>foe.weak=254,
                        "vulnerable"=>foe.vulnerable=254,
                        _=>foe.poison=9998,
                    }
                }
            }
            if target=="hero" {g.apply_hero_status(source,5);}
            else {g.apply_foe_status(0,source,5);}
            let b=g.battle.as_ref().unwrap();
            assert_eq!(b.event_seq,start_seq+1,"missing counter {source}/{target}");
            assert_eq!(b.events.last().unwrap().kind,EventKind::EventCountered);
            assert!(b.reactions.listeners.is_empty());
            assert_eq!(if target=="hero" {
                if source=="weak" {i32::from(b.weak)} else {i32::from(b.vulnerable)}
            } else {
                match source {
                    "weak"=>i32::from(b.foes[0].weak),
                    "vulnerable"=>i32::from(b.foes[0].vulnerable),
                    _=>b.foes[0].poison,
                }
            },ceiling-1);
            let raw=g.to_json().unwrap();
            assert_eq!(Game::from_json(&raw).unwrap().to_json().unwrap(),raw);
        }
    }
}

#[cfg(test)]
mod dynamic_power_gain_reactions {
    use super::*;
    use event_stack::{Listener,Response};

    fn custom() -> Game {
        let mut game=Game::new_with_mode(Hero::ZhugeLiang,925145,0,RunMode::Custom);
        game.apply(Action::Choose{index:1}).unwrap();
        assert_eq!(game.stage,Stage::Combat);
        game
    }
    fn power_listener(id:&str,source:&str,effect:Response,priority:i16,charges:u8)->Listener {
        Listener {
            id:id.into(),kind:EventKind::PowerGained,source:source.into(),
            target:Some("hero".into()),priority,charges,effect,
        }
    }
    #[test]
    fn power_delta_is_ordered_and_persisted_before_settlement() {
        let mut game=custom();
        game.apply(Action::RegisterReaction {
            listener:power_listener("last","oath_guard",Response::Replace(1),5,1)
        }).unwrap();
        game.apply(Action::RegisterReaction {
            listener:power_listener("first","oath_guard",Response::Reduce(2),-5,2)
        }).unwrap();
        assert_eq!(game.verify_action_replay().unwrap(),3);
        let seq=game.battle.as_ref().unwrap().event_seq;
        game.gain_power(PowerId::OathGuard,6);
        assert_eq!(game.power_stacks(PowerId::OathGuard),1);
        let b=game.battle.as_ref().unwrap();
        let audit=b.events.iter().filter(|e|e.seq>seq).collect::<Vec<_>>();
        assert_eq!(audit.iter().map(|e|e.kind).collect::<Vec<_>>(),
            vec![EventKind::EventReplaced,EventKind::EventReplaced,EventKind::PowerGained]);
        assert_eq!(audit.iter().map(|e|e.amount).collect::<Vec<_>>(),vec![2,3,1]);
        assert_eq!(audit[0].source,"first");
        assert_eq!(audit[1].source,"last");
        assert_eq!(b.reactions.listeners.len(),1);
        assert_eq!(b.reactions.listeners[0].charges,1);
        game.gain_power(PowerId::OathGuard,3);
        assert_eq!(game.power_stacks(PowerId::OathGuard),2);
        assert!(game.battle.as_ref().unwrap().reactions.is_empty());
        let raw=game.to_json().unwrap();
        assert_eq!(Game::from_json(&raw).unwrap().to_json().unwrap(),raw);
    }
    #[test]
    fn cancelled_gain_and_full_stack_do_not_consume_unrelated_charges() {
        let mut game=custom();
        game.apply(Action::RegisterReaction {
            listener:power_listener("stop","victory_drum",Response::Cancel,0,2)
        }).unwrap();
        let seq=game.battle.as_ref().unwrap().event_seq;
        game.gain_power(PowerId::VictoryDrum,5);
        assert_eq!(game.power_stacks(PowerId::VictoryDrum),0);
        let after=&game.battle.as_ref().unwrap().events;
        let audit=after.iter().filter(|e|e.seq>seq).collect::<Vec<_>>();
        assert_eq!(audit.len(),1);
        assert_eq!(audit[0].kind,EventKind::EventCountered);
        assert_eq!(audit[0].source,"stop");
        assert_eq!(game.battle.as_ref().unwrap().reactions.listeners[0].charges,1);
        // Fill a different power to the cap, then register an on-gain watcher.
        game.gain_power(PowerId::OathGuard,99);
        game.apply(Action::RegisterReaction {
            listener:power_listener("cap","oath_guard",Response::Cancel,0,1)
        }).unwrap();
        let before=game.battle.as_ref().unwrap().event_seq;
        game.gain_power(PowerId::OathGuard,5);
        assert_eq!(game.battle.as_ref().unwrap().event_seq,before);
        assert_eq!(game.battle.as_ref().unwrap().reactions.listeners
            .iter().find(|r|r.id=="cap").unwrap().charges,1);
    }
    #[test]
    fn unknown_or_mistargeted_power_proposals_are_atomic_rejections() {
        let mut game=custom();
        for invalid in [
            power_listener("bad","unknown",Response::Cancel,0,1),
            Listener {target:Some("foe:0".into()),
                ..power_listener("bad","oath_guard",Response::Cancel,0,1)},
        ] {
            let saved=game.to_json().unwrap();
            assert!(game.apply(Action::RegisterReaction{listener:invalid}).is_err());
            assert_eq!(game.to_json().unwrap(),saved);
        }
        let mut standard=Game::new(Hero::ZhugeLiang,925145);
        standard.apply(Action::Choose{index:1}).unwrap();
        assert!(standard.apply(Action::RegisterReaction {
            listener:power_listener("ranked","oath_guard",Response::Reduce(1),0,1)
        }).is_err());
    }
}

#[cfg(test)]
mod nested_toxic_reversal_interception {
    use super::*;
    use event_stack::{Listener, Response};

    fn prepared() -> Game {
        let mut game = Game::new_with_mode(Hero::ZhugeLiang, 925145, 0, RunMode::Custom);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.gain_power(PowerId::ToxicReversal, 4);
        game
    }

    fn bonus_rule(id: &str, effect: Response, priority: i16, charges: u8) -> Listener {
        Listener {
            id: id.into(),
            kind: EventKind::StatusApplied,
            source: "poison:toxic_reversal".into(),
            target: Some("foe:0".into()),
            priority,
            charges,
            effect,
        }
    }

    #[test]
    fn nested_bonus_runs_ordered_replacements_once_without_poison_recursion() {
        let mut game = prepared();
        game.apply(Action::RegisterReaction {
            listener: bonus_rule("late", Response::Replace(1), 5, 1),
        }).unwrap();
        game.apply(Action::RegisterReaction {
            listener: bonus_rule("early", Response::Reduce(1), -5, 1),
        }).unwrap();
        let start = game.battle.as_ref().unwrap().event_seq;
        game.apply_foe_status(0, "poison", 5);
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].poison, 6);
        assert!(battle.reactions.is_empty());
        let events = battle.events.iter().filter(|e| e.seq > start).collect::<Vec<_>>();
        assert_eq!(events.iter().map(|e| e.kind).collect::<Vec<_>>(), vec![
            EventKind::StatusApplied, EventKind::EventReplaced,
            EventKind::EventReplaced, EventKind::PowerTriggered,
            EventKind::StatusApplied,
        ]);
        assert_eq!(events.iter().map(|e| e.amount).collect::<Vec<_>>(),
            vec![5, 1, 2, 1, 1]);
        assert_eq!(events[0].source, "poison");
        assert_eq!(events[1].source, "early");
        assert_eq!(events[2].source, "late");
        assert_eq!(events[4].source, "poison:toxic_reversal");
        let json = game.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(), json);
    }

    #[test]
    fn countered_base_never_dispatches_nested_bonus_or_consumes_nested_rule() {
        let mut game = prepared();
        let mut base = Listener::once("base_stop", "poison", Response::Cancel);
        base.target = Some("foe:0".into());
        game.apply(Action::RegisterReaction { listener: base }).unwrap();
        game.apply(Action::RegisterReaction {
            listener: bonus_rule("bonus_stop", Response::Cancel, 0, 1),
        }).unwrap();
        let start = game.battle.as_ref().unwrap().event_seq;
        game.apply_foe_status(0, "poison", 5);
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].poison, 0);
        let events = battle.events.iter().filter(|e| e.seq > start).collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::EventCountered);
        assert_eq!(events[0].source, "base_stop");
        assert_eq!(battle.reactions.listeners.len(), 1);
        assert_eq!(battle.reactions.listeners[0].id, "bonus_stop");
    }

    #[test]
    fn saturated_bonus_does_not_consume_one_shot_and_later_counters() {
        let mut game = prepared();
        game.apply(Action::RegisterReaction {
            listener: bonus_rule("bonus_stop", Response::Cancel, 0, 1),
        }).unwrap();
        game.battle.as_mut().unwrap().foes[0].poison = 9998;
        let start = game.battle.as_ref().unwrap().event_seq;
        game.apply_foe_status(0, "poison", 1);
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].poison, 9999);
        assert_eq!(battle.event_seq, start + 1);
        assert_eq!(battle.reactions.listeners[0].charges, 1);
        game.battle.as_mut().unwrap().foes[0].poison = 9997;
        let next = game.battle.as_ref().unwrap().event_seq;
        game.apply_foe_status(0, "poison", 1);
        let battle = game.battle.as_ref().unwrap();
        assert_eq!(battle.foes[0].poison, 9998);
        assert!(battle.reactions.is_empty());
        let events = battle.events.iter().filter(|e| e.seq > next).collect::<Vec<_>>();
        assert_eq!(events.iter().map(|e| e.kind).collect::<Vec<_>>(),
            vec![EventKind::StatusApplied, EventKind::EventCountered]);
        assert_eq!(events[1].source, "bonus_stop");
        assert_eq!(events[1].amount, 1);
    }
}

#[cfg(test)]
mod encounter_seal_regression {
    use super::*;
    use crate::event_stack::projection::project_sealed;

    #[test]
    fn real_battle_victory_emits_a_terminal_seal_before_battle_disappears() {
        let mut game = Game::new(Hero::GuanYu, 20261024);
        game.floor = 1;
        game.start_battle(Node::Battle);
        assert!(game.battle.is_some());
        let _ = game.win_battle();
        assert!(game.battle.is_none());
        let archive = game.event_history.as_deref().unwrap();
        let last = &archive.last().unwrap().event;
        assert_eq!(last.kind, EventKind::BattleEnded);
        assert_eq!((last.source.as_str(), last.amount, last.target.as_deref()),
            ("victory", 1, None));
        assert!(project_sealed(archive).unwrap().encounters[0].closed);
        let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        assert_eq!(restored.event_history, game.event_history);
    }
}
