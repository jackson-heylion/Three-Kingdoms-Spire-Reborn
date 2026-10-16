//! Original Three Kingdoms relic and potion content, shared by every renderer.
pub const RELICS: &[(&str, &str)] = &[
    ("虎符", "战斗开始获得1武勇"),
    ("虎翼", "每回合额外抽1张牌"),
    ("玉玺", "每回合多获得1军令"),
    ("铜雀", "战斗胜利额外获得12金"),
    ("神医", "战斗胜利回复6生命"),
    ("兵书", "战斗开始获得1谋略"),
    ("百战", "每次打出攻击牌本场战斗增加1武勇"),
    ("厚甲", "首次获得时增加8最大生命"),
    ("铁骑", "战斗开始获得额外8护甲"),
    ("军粮", "营寨休息额外回复8生命"),
    ("朱雀", "群攻卡牌额外伤害3"),
    ("药囊", "胜利时获得药剂概率提高"),
    ("璇玑", "战斗首回合多1军令"),
    ("青囊", "使用药剂时回复额外4生命"),
    ("聚宝", "事件获得金币额外增加10"),
    ("百草", "战斗胜利回复2生命"),
];
pub const POTIONS: &[(&str, &str)] = &[
    ("healing", "军医汤：回复16生命"),
    ("fire", "火油罐：对目标造成22伤害"),
    ("shield", "藤甲散：获得20护甲"),
    ("energy", "急行丹：获得2军令"),
    ("poison", "毒箭囊：对目标施加12层中毒"),
    ("strength", "虎胆酒：本场战斗增加3武勇"),
    ("insight", "醒神汤：抽3张牌"),
    ("weak", "迷魂香：全体敌军虚弱3回合"),
    ("cleanse", "解毒丹：回复8生命并清除虚弱易伤"),
    ("blood", "血战酒：损失4生命，本场战斗获得5武勇"),
];
pub fn potion_name(id: &str) -> &'static str {
    POTIONS.iter().find(|(key, _)| *key == id).map_or("未知药剂", |(_, text)| text)
}
pub fn relic_name(id: &str) -> &'static str {
    RELICS.iter().find(|(key, _)| *key == id).map_or("未知遗物", |(_, text)| text)
}

pub const BOONS: &[(&str, &str)] = &[
    ("imperial_seal", "帝王敕令：最大体力+12并回复12"),
    ("war_drum", "擂鼓进军：每场战斗首回合+1军令"),
    ("strategist", "卧龙馈赠：每场战斗首回合额外抽2张牌"),
    ("blood_oath", "歃血为盟：最大体力-8、每场战斗武勇+2"),
    ("caravan", "粮道畅通：立刻获得140金"),
    ("arsenal", "百炼成钢：随机升级牌组中最多3张可升级牌"),
    ("veteran", "百战雄兵：每场战斗获得6护甲"),
    ("renewal", "济世仁心：战斗胜利回复8点体力"),
    ("flaming_sky", "火烧连营：群攻牌额外造成4点伤害"),
    ("herb_garden", "千金良方：药剂栏容量增至3并补充2瓶药"),
    ("founder", "开疆拓土：立即获得一件遗物"),
    ("formation", "连环阵法：每场战斗获得2点统御"),
];
pub fn boon_name(id: &str) -> &'static str {
    BOONS.iter().find(|(key,_)| *key == id).map_or("未知赐福", |(_,text)| text)
}
