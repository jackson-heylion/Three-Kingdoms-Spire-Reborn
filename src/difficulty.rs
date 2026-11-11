//! Opt-in gameplay-expansion campaign tiers; legacy ascension runs have None.
//! IDs are stable save/chronicle identifiers, not translated display names.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Warlords,
    Conqueror,
    Destiny,
}

impl Difficulty {
    pub const ALL: [Self; 3] = [Self::Warlords, Self::Conqueror, Self::Destiny];

    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "warlords" => Some(Self::Warlords),
            "conqueror" => Some(Self::Conqueror),
            "destiny" => Some(Self::Destiny),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Warlords => "warlords",
            Self::Conqueror => "conqueror",
            Self::Destiny => "destiny",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Warlords => "群雄逐鹿",
            Self::Conqueror => "乱世枭雄",
            Self::Destiny => "天命难违",
        }
    }

    // Initial pressure for the vertical slice; tactical mechanics will expand.
    pub fn enemy_hp_bonus(self) -> i32 {
        match self {
            Self::Warlords => 2,
            Self::Conqueror => 4,
            Self::Destiny => 6,
        }
    }

    pub fn attack_bonus(self) -> i32 {
        match self {
            Self::Warlords => 2,
            Self::Conqueror => 4,
            Self::Destiny => 6,
        }
    }

    pub fn support_bonus(self) -> i32 {
        match self {
            Self::Warlords => 0,
            Self::Conqueror => 1,
            Self::Destiny => 2,
        }
    }

    pub fn supply_toll(self) -> i32 {
        match self {
            Self::Warlords => 4,
            Self::Conqueror => 8,
            Self::Destiny => 12,
        }
    }
}
