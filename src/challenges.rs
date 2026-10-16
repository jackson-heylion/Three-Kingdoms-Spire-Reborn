//! Optional high-risk rules for unranked solo custom runs.
//! Stable IDs are serialized, not translated labels. Every modifier is deterministic.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeModifier {
    NoRest,
    IronLegion,
    ThinSupplies,
}
impl ChallengeModifier {
    pub const ALL: [Self; 3] = [Self::NoRest, Self::IronLegion, Self::ThinSupplies];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "no_rest" => Some(Self::NoRest),
            "iron_legion" => Some(Self::IronLegion),
            "thin_supplies" => Some(Self::ThinSupplies),
            _ => None,
        }
    }
    pub fn id(self) -> &'static str {
        match self {
            Self::NoRest => "no_rest",
            Self::IronLegion => "iron_legion",
            Self::ThinSupplies => "thin_supplies",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::NoRest => "行军不息",
            Self::IronLegion => "铁壁强敌",
            Self::ThinSupplies => "军资紧缩",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::NoRest => "营寨无法休息回血；仍可升级卡牌或操练体魄",
            Self::IronLegion => "每场战斗开战时每名敌军额外获得8点护甲",
            Self::ThinSupplies => "每回合基础军令减少1点（遗物和赐福仍独立加成）",
        }
    }
}
