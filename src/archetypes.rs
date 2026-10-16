//! Voluntary archetype drafting. Original game content; no forced card grants.
//! IDs are stable across CLI/native/Web snapshots.
use crate::Card;
use crate::content::{Hero, hero_pool};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Doctrine {
    GuanCharge, GuanOath,
    ZhugeScheme, ZhugeWisdom,
    SunDiscard, SunCombo,
    CaoTroops, CaoCommand,
    LuFury, LuBlood,
}

impl Doctrine {
    pub const ALL: [Self; 10] = [
        Self::GuanCharge, Self::GuanOath, Self::ZhugeScheme, Self::ZhugeWisdom,
        Self::SunDiscard, Self::SunCombo, Self::CaoTroops, Self::CaoCommand,
        Self::LuFury, Self::LuBlood,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::GuanCharge => "guan_charge", Self::GuanOath => "guan_oath",
            Self::ZhugeScheme => "zhuge_scheme", Self::ZhugeWisdom => "zhuge_wisdom",
            Self::SunDiscard => "sun_discard", Self::SunCombo => "sun_combo",
            Self::CaoTroops => "cao_troops", Self::CaoCommand => "cao_command",
            Self::LuFury => "lu_fury", Self::LuBlood => "lu_blood",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.id() == s)
    }
    pub fn hero(self) -> Hero {
        match self {
            Self::GuanCharge | Self::GuanOath => Hero::GuanYu,
            Self::ZhugeScheme | Self::ZhugeWisdom => Hero::ZhugeLiang,
            Self::SunDiscard | Self::SunCombo => Hero::SunShangxiang,
            Self::CaoTroops | Self::CaoCommand => Hero::CaoCao,
            Self::LuFury | Self::LuBlood => Hero::LuBu,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::GuanCharge => "单骑破阵", Self::GuanOath => "桃园义阵",
            Self::ZhugeScheme => "八卦奇谋", Self::ZhugeWisdom => "观星奇策",
            Self::SunDiscard => "飞羽换阵", Self::SunCombo => "连弩追击",
            Self::CaoTroops => "虎豹亲军", Self::CaoCommand => "魏武号令",
            Self::LuFury => "无双怒袭", Self::LuBlood => "浴血战魂",
        }
    }
    pub fn members(self) -> &'static [&'static str] {
        match self {
            Self::GuanCharge => &["yidao_x","yanyue","qianli","danqi","qinglongwu","pojun","guanxi","lianzhan","yidao"],
            Self::GuanOath => &["hold_line","victory_drum","oath_guard","zhongyi","juyi","yibao","qinying"],
            Self::ZhugeScheme => &["xingpan_tutor","toxic_reversal","bagua","lianji","wuhou","kuce","fenglei","mouduan"],
            Self::ZhugeWisdom => &["wisdom_on_draw","thought_spring","kongcheng","jiecaodong","shanchang","xinghuo","yingxin","zhouyuji"],
            Self::SunDiscard => &["feiyu_copy","arrows_on_discard","huanshe","jiecang","feishi","qiaoying","jibao"],
            Self::SunCombo => &["fleeting_blade","gonglian","tupo","jianshu","liandao","yitian","lianhua","huashe","lianying","yidong","chuanxin"],
            Self::CaoTroops => &["muji","granary_support","qingqi","hujun","longwei","xiongbi","jianbing","junshang"],
            Self::CaoCommand => &["tuntian","tieqi","junjin","jianxiong","xiaoling","qunxiong","shengming","wuwei","zhaoxian"],
            Self::LuFury => &["huaji_x","fennu","huaji","chijian","shuangji","baonu","tianxia"],
            Self::LuBlood => &["blood_courage","xuezhai","ranxue","xiongmeng","yanren","zhanqi","dangguan"],
        }
    }
    pub fn contains(self, id: &str) -> bool { self.members().contains(&id) }
    pub fn for_hero(hero: Hero) -> [Self; 2] {
        match hero {
            Hero::GuanYu => [Self::GuanCharge,Self::GuanOath],
            Hero::ZhugeLiang => [Self::ZhugeScheme,Self::ZhugeWisdom],
            Hero::SunShangxiang => [Self::SunDiscard,Self::SunCombo],
            Hero::CaoCao => [Self::CaoTroops,Self::CaoCommand],
            Hero::LuBu => [Self::LuFury,Self::LuBlood],
        }
    }
    /// Count distinct owned archetype IDs; duplicate card rewards do not artificially
    /// activate a focus. Standard starter cards cannot accidentally force a doctrine.
    pub fn distinct_owned(self, deck: &[Card]) -> usize {
        self.members().iter().filter(|id|deck.iter().any(|c|c.id==**id)).count()
    }
}

/// No forced starting build: learn the focus only after the player owns two
/// distinct nonstarter cards of one archetype. Ties deliberately stay neutral.
pub fn inferred(hero: Hero, deck: &[Card]) -> Option<Doctrine> {
    let [left,right]=Doctrine::for_hero(hero);
    let a=left.distinct_owned(deck);
    let b=right.distinct_owned(deck);
    if a>=2 && a>b {Some(left)}
    else if b>=2 && b>a {Some(right)}
    else {None}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::card_spec;
    #[test]
    fn catalog_is_hero_owned_with_no_cross_doctrine_overlap() {
        for hero in [Hero::GuanYu,Hero::ZhugeLiang,Hero::SunShangxiang,Hero::CaoCao,Hero::LuBu] {
            let [a,b]=Doctrine::for_hero(hero);
            for d in [a,b] {
                assert_eq!(d.hero(),hero);
                assert_eq!(Doctrine::parse(d.id()),Some(d));
                for id in d.members() {
                    assert!(hero_pool(hero).contains(id),"{} lacks {} in hero pool",hero.name(),id);
                    assert!(card_spec(id).is_some(),"unknown {id}");
                    assert!(![a,b].into_iter().find(|other|*other!=d).unwrap().contains(id),
                        "{id} belongs to both doctrines");
                }
            }
            for id in hero_pool(hero) {
                assert!(a.contains(id)||b.contains(id),"{id} has no archetype");
            }
        }
    }
    #[test]
    fn inferred_requires_two_distinct_cards_not_duplicate_starters() {
        let hero=Hero::SunShangxiang;
        assert_eq!(inferred(hero,&[Card::new("strike"),Card::new("lianhuan")]),None);
        assert_eq!(inferred(hero,&[Card::new("huanshe"),Card::new("huanshe")]),None);
        assert_eq!(inferred(hero,&[Card::new("huanshe"),Card::new("feishi")]),Some(Doctrine::SunDiscard));
        assert_eq!(inferred(hero,&[Card::new("huanshe"),Card::new("feishi"),Card::new("liandao"),
            Card::new("gonglian")]),None);
    }
}
