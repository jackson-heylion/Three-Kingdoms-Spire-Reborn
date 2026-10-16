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
    CardTutored,
    CardCopied,
    CardGenerated,
    EnemyTechnique,
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
