//! Proposed three build blueprints per hero. This is a tested catalogue,
//! NOT evidence that any build has achieved a target win rate.
use crate::{Card, content::{Hero, ENEMY_IDS, hero_pool, card_spec}};
#[derive(Clone, Copy, Debug)]
pub struct BuildPlan {
    pub id:&'static str,
    pub hero:Hero,
    pub name:&'static str,
    pub core:&'static [&'static str],
    pub support:&'static [&'static str],
    pub counter_enemy:&'static str,
    pub counterplay:&'static str,
    pub risk:&'static str,
}
impl BuildPlan {
    pub fn acquired(&self, deck:&[Card]) -> usize {
        self.core.iter().filter(|id|deck.iter().any(|c|c.id==**id)).count()
    }
    pub fn is_catalog_valid(&self) -> bool {
        let pool=hero_pool(self.hero);
        self.core.len()>=4 && self.support.len()>=2
            && ENEMY_IDS.contains(&self.counter_enemy)
            && self.core.iter().chain(self.support.iter()).all(|id|
                pool.contains(id) && card_spec(id).is_some())
            && self.core.iter().chain(self.support.iter()).enumerate().all(|(i,id)|
                !self.core.iter().chain(self.support.iter()).take(i).any(|other|other==id))
    }
}
pub const PLANS:&[BuildPlan]=&[
    BuildPlan{id:"guan_burst",hero:Hero::GuanYu,name:"单骑爆发",core:&["qianli","yanyue","yidao_x","pojun"],support:&["lianzhan","danqi"],counter_enemy:"royal_guard",counterplay:"高护甲禁卫克制小段输出；预留破军易伤后再打 X 重击",risk:"避免未成型前过多高费"},
    BuildPlan{id:"guan_oath",hero:Hero::GuanYu,name:"义阵反击",core:&["oath_guard","hold_line","zhongyi","victory_drum"],support:&["yibao","qinying"],counter_enemy:"pyromancer",counterplay:"灼伤与多段技消耗护甲；先叠义阵再保留挡牌",risk:"防牌过多导致输出不足"},
    BuildPlan{id:"guan_cleave",hero:Hero::GuanYu,name:"群斩破围",core:&["qinglongwu","yidao","lianzhan","guanxi"],support:&["qianli","pojun"],counter_enemy:"yuanshao",counterplay:"盟军列阵和多目标护甲反制；分配火力击杀支援",risk:"群攻卡抽不到时需单体终结"},
    BuildPlan{id:"zhuge_poison",hero:Hero::ZhugeLiang,name:"七擒毒谋",core:&["kuce","toxic_reversal","mouduan","xingpan_tutor"],support:&["bagua","fenglei"],counter_enemy:"zuoci",counterplay:"左慈自愈拖长战线；毒层积累配合穿透爆发",risk:"防止只有毒牌缺即时伤害"},
    BuildPlan{id:"zhuge_cycle",hero:Hero::ZhugeLiang,name:"观星循环",core:&["wisdom_on_draw","thought_spring","jiecaodong","xinghuo"],support:&["lianji","kongcheng"],counter_enemy:"simayi",counterplay:"司马懿反制高频出牌；优先维持每回合少出高价值牌",risk:"高频过牌触发敌方惩罚"},
    BuildPlan{id:"zhuge_schemes",hero:Hero::ZhugeLiang,name:"八卦破军",core:&["bagua","wuhou","lianji","xinghuo"],support:&["mouduan","fenglei"],counter_enemy:"royal_guard",counterplay:"护甲厚的禁卫压制普通谋攻；用穿透处理关键目标",risk:"过多谋略卡会降低防御密度"},
    BuildPlan{id:"sun_discard",hero:Hero::SunShangxiang,name:"飞羽弃换",core:&["arrows_on_discard","huanshe","jiecang","feishi"],support:&["feiyu_copy","jibao"],counter_enemy:"tiger_general",counterplay:"虎将二阶段提高火力；弃换自动伤害需搭配闪避护甲",risk:"没有弃换 token 时收益骤降"},
    BuildPlan{id:"sun_combo",hero:Hero::SunShangxiang,name:"连弩快攻",core:&["fleeting_blade","tupo","gonglian","liandao"],support:&["lianying","lianhua"],counter_enemy:"simayi",counterplay:"四牌以上司马懿追加军令惩罚；在关键回合控制张数",risk:"过牌过量会吃敌方反制"},
    BuildPlan{id:"sun_poison",hero:Hero::SunShangxiang,name:"穿心游击",core:&["huashe","chuanxin","jianshu","yidong"],support:&["jibao","qiaoying"],counter_enemy:"war_elephant",counterplay:"战象高血高攻；虚弱与小护甲不足时容易被消耗",risk:"需要集中穿透斩杀而非广撒毒"},
    BuildPlan{id:"cao_troops",hero:Hero::CaoCao,name:"虎豹屯军",core:&["muji","qingqi","hujun","granary_support"],support:&["longwei","jianbing"],counter_enemy:"dongzhuo",counterplay:"董卓强攻与军资掠夺逼迫缩短蓄兵期",risk:"亲兵阵亡后补给无效"},
    BuildPlan{id:"cao_command",hero:Hero::CaoCao,name:"魏武统御",core:&["tuntian","junjin","tieqi","xiaoling"],support:&["wuwei","zhaoxian"],counter_enemy:"simayi",counterplay:"高频发令被司马懿限制下回合军令；规划三四张关键动作",risk:"高循环无真实输出会形成慢局"},
    BuildPlan{id:"cao_control",hero:Hero::CaoCao,name:"铁壁权术",core:&["jianxiong","shengming","xiongbi","junshang"],support:&["qunxiong","hujun"],counter_enemy:"zuoci",counterplay:"左慈回血使纯防御软锁；保留易伤爆发终结",risk:"防御溢出需主动输出牌"},
    BuildPlan{id:"lu_fury",hero:Hero::LuBu,name:"无双怒袭",core:&["fennu","baonu","huaji","huaji_x"],support:&["shuangji","chijian"],counter_enemy:"royal_guard",counterplay:"禁卫高甲吃掉零散伤害；怒气集中打突破口",risk:"无怒气来源时关键牌失效"},
    BuildPlan{id:"lu_blood",hero:Hero::LuBu,name:"浴血战魂",core:&["blood_courage","xuezhai","ranxue","yanren"],support:&["zhanqi","dangguan"],counter_enemy:"sunjian",counterplay:"孙坚多段攻击加速生命损失；治疗牌优先留手",risk:"血价结算可能自伤致败"},
    BuildPlan{id:"lu_combo",hero:Hero::LuBu,name:"方天连斩",core:&["shuangji","tianxia","huaji","chijian"],support:&["huaji_x","dangguan"],counter_enemy:"zhangjiao",counterplay:"张角虚弱降低连斩收益；用反击或护甲减少受控",risk:"连打需要怒气与能量同步供给"},
];
pub fn for_hero(hero:Hero) -> impl Iterator<Item=&'static BuildPlan> {
    PLANS.iter().filter(move |p|p.hero==hero)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn fifteen_unique_legal_blueprints_cover_every_hero() {
        let mut ids=std::collections::BTreeSet::new();
        assert_eq!(PLANS.len(),15);
        for plan in PLANS {
            assert!(ids.insert(plan.id));
            assert!(plan.is_catalog_valid(),"{:?}",plan);
            assert!(!plan.counterplay.is_empty() && !plan.risk.is_empty());
        }
        for hero in [Hero::GuanYu,Hero::ZhugeLiang,Hero::SunShangxiang,Hero::CaoCao,Hero::LuBu] {
            assert_eq!(for_hero(hero).count(),3);
        }
    }
    #[test] fn zhuge_cycle_has_two_direct_damage_outlets_after_underperforming() {
        use crate::content::Effect;
        let plan=PLANS.iter().find(|p|p.id=="zhuge_cycle").unwrap();
        let offensive=plan.core.iter().chain(plan.support.iter())
            .filter(|id|crate::content::card_spec(id).is_some_and(|spec|
                matches!(spec.effect,Effect::All(_) | Effect::SchemeStrike(_) |
                    Effect::Hit(_) | Effect::Pierce(_))))
            .count();
        assert!(offensive>=2, "cycling without damage cannot close boss fights");
    }
    #[test] fn acquisition_requires_distinct_actual_core_cards() {
        let plan=&PLANS[0];
        assert_eq!(plan.acquired(&[Card::new(plan.core[0]),Card::new(plan.core[0])]),1);
        assert_eq!(plan.acquired(&[]),0);
        assert_eq!(plan.acquired(&plan.core.iter().map(|id|Card::new(id)).collect::<Vec<_>>()),4);
    }
}
