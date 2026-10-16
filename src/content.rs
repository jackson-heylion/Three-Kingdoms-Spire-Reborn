//! Independent, original Three Kingdoms content catalogue.
//! Add new IDs without changing the save schema; never serialize localized display names.
use serde::{Deserialize, Serialize};
use crate::combat::PowerId;

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
    HitWeak(i32, u8),
    HitVulnerable(i32, u8),
    WeakAll(u8),
    ApplyVulnerable(u8),
    GainMomentum(i32),
    MomentumStrike(i32),
    Scheme(i32),
    SchemeStrike(i32),
    ComboStrike(i32),
    Command(i32),
    CommandStrike(i32),
    Fury(i32),
    FuryStrike(i32),
    BloodPrice(i32, i32),
    Thorns(i32),
    Leech(i32),
    HitPoison(i32, i32),
    Pierce(i32),
    DrawEnergy(usize, i32),
    Energy(i32),
    Companion(i32),
    Tactics(i32),
    DiscardStrike(i32),
    Power(PowerId, i32),
    XHit(i32),
    XGuard(i32),
    TutorAttack,
    CopyDiscard,
    Generate(&'static str, usize),
    Unplayable,
}

#[derive(Clone, Copy, Debug)]
pub struct CardSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
    pub effect: Effect,
    pub exhaust: bool,
    pub retain: bool,
    pub ethereal: bool,
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
        "yanyue" => ("拖刀计", "伤害10，战意加成", 1, Effect::MomentumStrike(10), false),
        "zhongyi" => ("义薄云天", "护甲12", 2, Effect::Guard(12), false),
        "qianli" => ("千里突进", "获得2战意", 0, Effect::GainMomentum(2), false),
        "juyi" => ("义军响应", "武勇加2", 2, Effect::Strength(2), false),
        "danqi" => ("单骑冲阵", "战意重击", 2, Effect::MomentumStrike(17), false),
        "qinglongwu" => ("青龙狂舞", "全体伤害8", 2, Effect::All(8), false),
        "yibao" => ("义勇掩护", "护甲8，军令加1", 1, Effect::GuardEnergy(8), false),
        "pojun" => ("破军一击", "伤害9，易伤2", 1, Effect::HitVulnerable(9, 2), false),
        "guanxi" => ("关隘血战", "伤害16并吸血", 3, Effect::Leech(16), false),
        "lianzhan" => ("连战不休", "抽2牌，军令加1", 1, Effect::DrawEnergy(2, 1), false),
        "yidao" => ("义刀追击", "连续攻击两次", 2, Effect::HitTwice(6), false),
        "qinying" => ("亲兵盾阵", "反击5", 1, Effect::Thorns(5), false),
        "bagua" => ("八卦阵", "获得2谋略", 0, Effect::Scheme(2), false),
        "kongcheng" => ("空城计", "护甲11", 2, Effect::Guard(11), false),
        "jiecaodong" => ("借东风", "抽3张牌", 1, Effect::Draw(3), false),
        "lianji" => ("连环计", "谋略加成伤害", 1, Effect::SchemeStrike(9), false),
        "shanchang" => ("神机妙算", "军令加2", 1, Effect::Energy(2), false),
        "xinghuo" => ("星火燎原", "全体伤害9", 2, Effect::All(9), false),
        "yingxin" => ("疑兵之计", "全体虚弱2", 1, Effect::WeakAll(2), false),
        "zhouyuji" => ("江东借兵", "护甲5，军令加1", 1, Effect::GuardEnergy(5), false),
        "mouduan" => ("谋定后动", "易伤3", 1, Effect::ApplyVulnerable(3), false),
        "wuhou" => ("卧龙观星", "谋略加4", 2, Effect::Scheme(4), false),
        "kuce" => ("苦肉谋", "中毒7", 2, Effect::Poison(7), false),
        "fenglei" => ("风雷符", "穿透伤害18", 3, Effect::Pierce(18), false),
        "gonglian" => ("弓连", "连击增伤", 1, Effect::ComboStrike(8), false),
        "tupo" => ("突围骑射", "伤害5抽1", 0, Effect::HitDraw(5, 1), false),
        "jianshu" => ("疾箭", "穿透伤害7", 1, Effect::Pierce(7), false),
        "liandao" => ("连弩阵", "两次伤害5", 1, Effect::HitTwice(5), false),
        "qiaoying" => ("巧影回旋", "护甲8", 1, Effect::Guard(8), false),
        "yitian" => ("倚天弓", "武勇加1", 1, Effect::Strength(1), false),
        "lianhua" => ("莲花步", "抽3张", 1, Effect::Draw(3), false),
        "huashe" => ("花射", "伤害7，中毒3", 1, Effect::HitPoison(7, 3), false),
        "lianying" => ("连营", "连击重击", 2, Effect::ComboStrike(15), false),
        "yidong" => ("移形换位", "护甲6，军令加1", 1, Effect::GuardEnergy(6), false),
        "chuanxin" => ("穿心箭", "伤害10，易伤2", 2, Effect::HitVulnerable(10, 2), false),
        "jibao" => ("急报", "抽2，军令加1", 1, Effect::DrawEnergy(2, 1), false),
        "tuntian" => ("屯田", "统御加2", 0, Effect::Command(2), false),
        "tieqi" => ("虎豹铁骑", "统御强化伤害", 1, Effect::CommandStrike(9), false),
        "junjin" => ("军令齐发", "统御加2", 1, Effect::Command(2), false),
        "xiongbi" => ("雄兵坚阵", "护甲12", 2, Effect::Guard(12), false),
        "jianxiong" => ("奸雄权术", "伤害5，虚弱2", 1, Effect::HitWeak(5, 2), false),
        "xiaoling" => ("校令", "军令加2", 1, Effect::Energy(2), false),
        "qunxiong" => ("群雄尽聚", "武勇加2", 2, Effect::Strength(2), false),
        "shengming" => ("声东击西", "伤害10，易伤2", 2, Effect::HitVulnerable(10, 2), false),
        "jianbing" => ("精兵援护", "反击6", 1, Effect::Thorns(6), false),
        "junshang" => ("军粮供给", "回复7", 2, Effect::Heal(7), false),
        "wuwei" => ("魏武挥军", "群攻9", 2, Effect::All(9), false),
        "zhaoxian" => ("招贤纳士", "抽3张", 1, Effect::Draw(3), false),
        "fennu" => ("怒吼", "怒气加3", 0, Effect::Fury(3), false),
        "huaji" => ("方天画戟", "消耗怒气重击", 1, Effect::FuryStrike(10), false),
        "xuezhai" => ("血债", "损4血伤害19", 1, Effect::BloodPrice(4, 19), false),
        "chijian" => ("赤兔疾驰", "抽2军令加1", 1, Effect::DrawEnergy(2, 1), false),
        "shuangji" => ("双戟乱舞", "两次伤害7", 2, Effect::HitTwice(7), false),
        "baonu" => ("暴怒", "怒气加5", 1, Effect::Fury(5), false),
        "ranxue" => ("燃血", "损3血伤害16", 1, Effect::BloodPrice(3, 16), false),
        "tianxia" => ("天下无双", "怒气重击23", 3, Effect::FuryStrike(23), false),
        "xiongmeng" => ("凶猛一击", "伤害14", 2, Effect::Hit(14), false),
        "yanren" => ("燕人援手", "回复7", 2, Effect::Heal(7), false),
        "zhanqi" => ("战骑护主", "护甲11", 2, Effect::Guard(11), false),
        "dangguan" => ("当关", "反击5", 1, Effect::Thorns(5), false),
        "duanliang" => ("断粮", "伤害4虚弱2", 1, Effect::HitWeak(4, 2), false),
        "yingji" => ("营救", "回复8", 2, Effect::Heal(8), false),
        "guji" => ("孤军", "军令加2", 0, Effect::Energy(2), false),
        "zhuangji" => ("撞击", "伤害11", 2, Effect::Hit(11), false),
        "zhentu" => ("阵图", "护甲7", 1, Effect::Guard(7), false),
        "duyao" => ("毒药", "中毒6", 2, Effect::Poison(6), false),
        "bensi" => ("奔袭", "伤害4抽2", 1, Effect::HitDraw(4, 2), false),
        "lianshe" => ("连射", "两次伤害5", 2, Effect::HitTwice(5), false),
        "jianbi" => ("坚壁清野", "护甲16", 3, Effect::Guard(16), false),
        "fengcheng" => ("烽城", "全体虚弱2", 2, Effect::WeakAll(2), false),
        "shoucang" => ("收仓", "抽2军令加1", 1, Effect::DrawEnergy(2, 1), false),
        "zonghenggong" => ("纵横弓", "穿透伤害10", 2, Effect::Pierce(10), false),
        "quest_tulu" => ("军情图录", "计谋：经历3次奇遇后蜕变为军情决断；不可打出", 0, Effect::Unplayable, false),
        "quest_resolved" => ("军情决断", "抽3张牌并获得1军令，使用后消耗", 1, Effect::DrawEnergy(3, 1), true),
        "qingqi" => ("轻骑扈从", "提升亲兵战力与护卫体力4点", 1, Effect::Companion(4), false),
        "hujun" => ("护军屯驻", "提升亲兵战力与护卫体力8点", 2, Effect::Companion(8), false),
        "longwei" => ("龙骧飞骑", "提升亲兵战力与护卫体力12点", 3, Effect::Companion(12), false),
        "huanshe" => ("幻影换阵", "获得2次弃换行动", 1, Effect::Tactics(2), false),
        "jiecang" => ("借势回旋", "获得1次弃换行动", 0, Effect::Tactics(1), false),
        "feishi" => ("策应飞矢", "造成7点伤害；被弃换时自动触发", 1, Effect::DiscardStrike(7), false),
        "oath_guard" => ("桃园义阵", "持续：每次打出攻击牌，获得3护甲/层", 1, Effect::Power(PowerId::OathGuard, 1), true),
        "thought_spring" => ("卧龙筹策", "持续：每回合首次打出非攻击牌，回复1军令/层并抽1", 1, Effect::Power(PowerId::ThoughtSpring, 1), true),
        "arrows_on_discard" => ("飞羽随形", "持续：主动弃换手牌时对敌军造成4伤害/层", 1, Effect::Power(PowerId::ArrowsOnDiscard, 1), true),
        "granary_support" => ("屯田补给", "持续：回合末存活亲兵恢复4体力/层", 1, Effect::Power(PowerId::GranarySupport, 1), true),
        "blood_courage" => ("饮血方天", "持续：受实际伤害额外得到2怒气/层", 1, Effect::Power(PowerId::BloodCourage, 1), true),
        "victory_drum" => ("斩将凯歌", "持续：击败敌军后恢复1军令/层", 2, Effect::Power(PowerId::VictoryDrum, 1), true),
        "wisdom_on_draw" => ("察势观星", "持续：抽到牌时获得1护甲/层", 1, Effect::Power(PowerId::WisdomOnDraw, 1), true),
        "toxic_reversal" => ("七擒余毒", "持续：对敌军施加中毒时额外施加1层/层", 1, Effect::Power(PowerId::ToxicReversal, 1), true),
        "hold_line" => ("待命伏兵", "保留：获得6点护甲及1军令，不打出可留在下回合", 1, Effect::GuardEnergy(6), false),
        "fleeting_blade" => ("惊鸿一掠", "虚无：造成6点伤害并施加1层易伤，回合末未打出则消耗", 0, Effect::HitVulnerable(6, 1), false),
        "yidao_x" => ("义绝连斩", "X费：消耗全部剩余军令，每点造成6伤害；可零费发动", -1, Effect::XHit(6), false),
        "xingpan_tutor" => ("星盘识阵", "从抽牌堆指定检索一张攻击牌；不触发抽牌事件", 1, Effect::TutorAttack, false),
        "feiyu_copy" => ("飞羽映射", "从弃牌堆指定复制一张可打出的牌至手中；复制品为临时牌", 1, Effect::CopyDiscard, true),
        "muji" => ("募军敕令", "生成2张本场战斗临时招架；出牌或回合末消耗", 1, Effect::Generate("defend", 2), false),
        "huaji_x" => ("无双蓄击", "X费：消耗全部剩余军令，每点造成8伤害；可零费发动", -1, Effect::XHit(8), false),
        "x_bulwark" => ("长城连营", "X费：消耗全部剩余军令，每点获得5护甲", -1, Effect::XGuard(5), false),
        "field_recruit" => ("战地征募", "生成1张本场战斗临时突刺；出牌或回合末消耗", 1, Effect::Generate("strike", 1), false),
        "cursed" => ("军心涣散", "无法主动打出", 0, Effect::Unplayable, false),
        _ => return None,
    };
    Some(CardSpec { name, description, cost, effect, exhaust,
        retain: matches!(id, "hold_line"), ethereal: matches!(id, "fleeting_blade") })
}

pub const REWARD_POOL: &[&str] = &[
    "qinglong", "qimou", "lianhuan", "haoling", "wushuang", "huogong",
    "duji", "caochuan", "taoyuan", "junling", "tiebi", "jilie",
];

pub const NEUTRAL_POOL: &[&str] = &["x_bulwark", "field_recruit", "duanliang", "yingji", "guji", "zhuangji", "zhentu", "duyao", "bensi", "lianshe", "jianbi", "fengcheng", "shoucang", "zonghenggong"];

pub fn hero_pool(hero: Hero) -> &'static [&'static str] {
    match hero {
        Hero::GuanYu => &["yidao_x", "hold_line", "victory_drum", "oath_guard", "yanyue", "zhongyi", "qianli", "juyi", "danqi", "qinglongwu", "yibao", "pojun", "guanxi", "lianzhan", "yidao", "qinying"],
        Hero::ZhugeLiang => &["xingpan_tutor", "toxic_reversal", "wisdom_on_draw", "thought_spring", "bagua", "kongcheng", "jiecaodong", "lianji", "shanchang", "xinghuo", "yingxin", "zhouyuji", "mouduan", "wuhou", "kuce", "fenglei"],
        Hero::SunShangxiang => &["feiyu_copy", "fleeting_blade", "arrows_on_discard", "huanshe", "jiecang", "feishi", "gonglian", "tupo", "jianshu", "liandao", "qiaoying", "yitian", "lianhua", "huashe", "lianying", "yidong", "chuanxin", "jibao"],
        Hero::CaoCao => &["muji", "granary_support", "qingqi", "hujun", "longwei", "tuntian", "tieqi", "junjin", "xiongbi", "jianxiong", "xiaoling", "qunxiong", "shengming", "jianbing", "junshang", "wuwei", "zhaoxian"],
        Hero::LuBu => &["huaji_x", "blood_courage", "fennu", "huaji", "xuezhai", "chijian", "shuangji", "baonu", "ranxue", "tianxia", "xiongmeng", "yanren", "zhanqi", "dangguan"],
    }
}
pub fn card_count() -> usize { 112 }

pub const ENEMY_IDS: &[&str] = &["bandit", "archer", "officer", "shaman", "yellow_turban", "pyromancer", "spearman", "cuirass", "raider", "crossbowman", "wolf", "tomb_guard", "firelancer", "tactician", "horse_archer", "assassin", "panther", "iron_guard", "witch", "sling", "elite", "war_elephant", "royal_guard", "tiger_general", "sacred_beast", "zhangjiao", "sunjian", "dongzhuo", "yuanshao", "simayi", "zuoci"];

pub fn enemy_hp(id: &str) -> i32 {
    match id {
        "bandit" => 32,
        "archer" => 23,
        "officer" => 43,
        "shaman" => 53,
        "yellow_turban" => 38,
        "pyromancer" => 34,
        "spearman" => 35,
        "cuirass" => 48,
        "raider" => 49,
        "crossbowman" => 42,
        "wolf" => 49,
        "tomb_guard" => 52,
        "firelancer" => 57,
        "tactician" => 48,
        "horse_archer" => 45,
        "assassin" => 43,
        "panther" => 58,
        "iron_guard" => 66,
        "witch" => 47,
        "sling" => 36,
        "elite" => 58,
        "war_elephant" => 76,
        "royal_guard" => 70,
        "tiger_general" => 68,
        "sacred_beast" => 78,
        "zhangjiao" => 75,
        "sunjian" => 91,
        "dongzhuo" => 106,
        "yuanshao" => 115,
        "simayi" => 142,
        "zuoci" => 132,
        _ => 30,
    }
}

pub fn enemy_name(id: &str) -> &'static str {
    match id {
        "bandit" => "山贼",
        "archer" => "黄巾弓手",
        "officer" => "西凉校尉",
        "shaman" => "南蛮祭司",
        "yellow_turban" => "黄巾力士",
        "pyromancer" => "火符道士",
        "spearman" => "枪兵",
        "cuirass" => "重甲刀盾兵",
        "raider" => "乌桓掠袭者",
        "crossbowman" => "连弩手",
        "wolf" => "南蛮狼骑",
        "tomb_guard" => "陵卫",
        "firelancer" => "火枪精锐",
        "tactician" => "行军参谋",
        "horse_archer" => "匈奴骑射",
        "assassin" => "潜伏刺客",
        "panther" => "豹骑前锋",
        "iron_guard" => "铁甲军",
        "witch" => "巫蛊师",
        "sling" => "投石客",
        "elite" => "虎豹骑统领",
        "war_elephant" => "战象统帅",
        "royal_guard" => "禁卫长",
        "tiger_general" => "猛将",
        "sacred_beast" => "山中异兽",
        "zhangjiao" => "张角·天公将军",
        "sunjian" => "孙坚·江东猛虎",
        "dongzhuo" => "董卓·暴虐之主",
        "yuanshao" => "袁绍·四世三公",
        "simayi" => "司马懿·冢虎",
        "zuoci" => "左慈·遁甲宗师",
        _ => "无名敌军",
    }
}
