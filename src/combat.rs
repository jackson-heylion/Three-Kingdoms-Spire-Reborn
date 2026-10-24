//! Deterministic, serializable combat event journal and persistent power identifiers.
//! The journal is bounded for compact autosaves; it is a recent event trace, not a complete replay.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerId {
    OathGuard,
    ThoughtSpring,
    ArrowsOnDiscard,
    GranarySupport,
    BloodCourage,
    WisdomOnDraw,
    ToxicReversal,
    VictoryDrum,
}
impl PowerId {
    /// Stable identifier for event preflight; localized labels are for the settled journal.
    pub fn key(self) -> &'static str {
        match self {
            Self::OathGuard => "oath_guard",
            Self::ThoughtSpring => "thought_spring",
            Self::ArrowsOnDiscard => "arrows_on_discard",
            Self::GranarySupport => "granary_support",
            Self::BloodCourage => "blood_courage",
            Self::WisdomOnDraw => "wisdom_on_draw",
            Self::ToxicReversal => "toxic_reversal",
            Self::VictoryDrum => "victory_drum",
        }
    }
    /// Reject unknown power names rather than accepting inert listeners.
    pub fn from_key(key: &str) -> Option<Self> {
        [
            Self::OathGuard, Self::ThoughtSpring, Self::ArrowsOnDiscard,
            Self::GranarySupport, Self::BloodCourage, Self::WisdomOnDraw,
            Self::ToxicReversal, Self::VictoryDrum,
        ].into_iter().find(|power| power.key() == key)
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::OathGuard => "义军护阵",
            Self::ThoughtSpring => "锦囊连思",
            Self::ArrowsOnDiscard => "飞羽追袭",
            Self::GranarySupport => "粮道援护",
            Self::BloodCourage => "浴血鏖战",
            Self::WisdomOnDraw => "察势观星",
            Self::ToxicReversal => "七擒余毒",
            Self::VictoryDrum => "斩将凯歌",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::OathGuard => "每次打出攻击牌，获得3护甲/层",
            Self::ThoughtSpring => "每回合首次打出非攻击牌，回复1军令/层并抽1张牌",
            Self::ArrowsOnDiscard => "主动弃换手牌时，造成4伤害/层给首个存活敌人",
            Self::GranarySupport => "回合结束时，存活亲兵回复4体力/层（上限40）",
            Self::BloodCourage => "英雄生命受到实际伤害时，额外获得2怒气/层",
            Self::WisdomOnDraw => "每抽到一张牌，获得1护甲/层",
            Self::ToxicReversal => "对敌人施加中毒时，额外附加1层中毒/层",
            Self::VictoryDrum => "击败一名敌人后，恢复1军令/层",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerStack {
    pub id: PowerId,
    pub stacks: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    CardPlayed,
    CardDiscarded,
    DamageDealt,
    DamageReceived,
    PowerGained,
    PowerTriggered,
    TurnEnded,
    TurnStarted,
    CardResolved,
    CardDrawn,
    CardExhausted,
    StatusApplied,
    FoeDefeated,
    /// A new foe has entered combat; its first actionable intent is next turn.
    FoeSummoned,
    /// A consumed potion, emitted before its effects and subsequent status triggers.
    PotionUsed,
    CardTutored,
    CardCopied,
    CardGenerated,
    EnemyTechnique,
    EventCountered,
    EventReplaced,
    /// Terminal encounter seal: source victory/defeat, amount 1/0, no target.
    BattleEnded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatEvent {
    pub seq: u64,
    pub kind: EventKind,
    /// Human-readable but stable ID, not localized display text.
    pub source: String,
    pub amount: i32,
    pub target: Option<String>,
}
pub const MAX_RECENT_EVENTS: usize = 80;

 
/// Read-only, battle-scoped view of the bounded combat journal.
/// Cursors are event sequence numbers, not a persistent cross-battle event stream.
#[derive(Debug)]
pub struct EventPage<'a> {
    pub entries: &'a [CombatEvent],
    /// Earliest sequence still retained (latest + 1 when the journal is empty).
    pub oldest_available: u64,
    pub latest: u64,
    /// Number of events already evicted before the requested cursor.
    pub missed: u64,
    /// Resume with this value to avoid repeating events on the next poll.
    pub next_cursor: u64,
    pub has_more: bool,
    /// Cursor belongs to a later point than this battle's current sequence.
    pub reset: bool,
}
