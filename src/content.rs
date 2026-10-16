//! Independent, original Three Kingdoms content catalogue.
//! Add new IDs without changing the save schema; never serialize localized display names.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hero {
    GuanYu,
    ZhugeLiang,
    SunShangxiang,
    CaoCao,
    LuBu,
}

impl Hero {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "guanyu" | "guan" | "1" => Some(Self::GuanYu),
            "zhuge" | "zhugeliang" | "2" => Some(Self::ZhugeLiang),
            "sun" | "sunshangxiang" | "3" => Some(Self::SunShangxiang),
            "cao" | "caocao" | "4" => Some(Self::CaoCao),
            "lubu" | "lu" | "5" => Some(Self::LuBu),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::GuanYu => "关羽",
            Self::ZhugeLiang => "诸葛亮",
            Self::SunShangxiang => "孙尚香",
            Self::CaoCao => "曹操",
            Self::LuBu => "吕布",
        }
    }
    pub fn max_hp(self) -> i32 {
        match self {
            Self::GuanYu => 80,
            Self::ZhugeLiang => 68,
            Self::SunShangxiang => 72,
            Self::CaoCao => 76,
            Self::LuBu => 84,
        }
    }
    pub fn signature(self) -> &'static str {
        match self {
            Self::GuanYu => "qinglong",
            Self::ZhugeLiang => "qimou",
            Self::SunShangxiang => "lianhuan",
            Self::CaoCao => "haoling",
            Self::LuBu => "wushuang",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Hit(i32),
    Guard(i32),
    HitDraw(i32, usize),
    HitTwice(i32),
    All(i32),
    Poison(i32),
    Draw(usize),
    Strength(i32),
    Heal(i32),
    GuardEnergy(i32),
    Unplayable,
}

#[derive(Clone, Copy, Debug)]
pub struct CardSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
    pub effect: Effect,
    pub exhaust: bool,
}

pub fn card_spec(id: &str) -> Option<CardSpec> {
    let (name, description, cost, effect, exhaust) = match id {
        "strike" => ("突刺", "对一名敌军造成6点伤害", 1, Effect::Hit(6), false),
        "defend" => ("招架", "获得5点护甲", 1, Effect::Guard(5), false),
        "qinglong" => ("青龙偃月", "造成9点伤害，抽1张牌", 1, Effect::HitDraw(9, 1), false),
        "qimou" => ("锦囊妙计", "抽2张牌", 1, Effect::Draw(2), false),
        "lianhuan" => ("连环弩", "连续攻击两次，每次4点", 1, Effect::HitTwice(4), false),
        "haoling" => ("挟天子令", "本场战斗获得2点武勇", 1, Effect::Strength(2), false),
        "wushuang" => ("无双突袭", "对一名敌军造成14点伤害，使用后消耗", 2, Effect::Hit(14), true),
        "huogong" => ("火攻", "对所有敌军造成7点伤害", 2, Effect::All(7), false),
        "duji" => ("毒计", "对一名敌军施加4层中毒", 1, Effect::Poison(4), false),
        "caochuan" => ("草船借箭", "获得7点护甲和1点军令", 1, Effect::GuardEnergy(7), false),
        "taoyuan" => ("桃园结义", "回复5点体力", 1, Effect::Heal(5), true),
        "junling" => ("军令如山", "对一名敌军造成12点伤害", 2, Effect::Hit(12), false),
        "tiebi" => ("铁壁阵", "获得11点护甲", 2, Effect::Guard(11), false),
        "jilie" => ("疾烈冲锋", "连续攻击两次，每次6点", 2, Effect::HitTwice(6), false),
        "cursed" => ("军心涣散", "无法主动打出", 0, Effect::Unplayable, false),
        _ => return None,
    };
    Some(CardSpec { name, description, cost, effect, exhaust })
}

pub const REWARD_POOL: &[&str] = &[
    "qinglong", "qimou", "lianhuan", "haoling", "wushuang", "huogong",
    "duji", "caochuan", "taoyuan", "junling", "tiebi", "jilie",
];

pub fn enemy_name(id: &str) -> &'static str {
    match id {
        "bandit" => "山贼",
        "archer" => "黄巾弓手",
        "officer" => "西凉校尉",
        "shaman" => "南蛮祭司",
        "elite" => "虎豹骑统领",
        "zhangjiao" => "张角·天公将军",
        "dongzhuo" => "董卓·暴虐之主",
        "simayi" => "司马懿·冢虎",
        _ => "无名敌军",
    }
}
