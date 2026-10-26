//! Three Kingdoms narrative adaptation, based on the 120-chapter Mao edition.
//! The chapter references are literary provenance, not a claim of historical accuracy.
//! Story text and rewards are original game writing, not verbatim source passages.
use crate::content::Hero;

#[derive(Clone, Copy, Debug)]
pub struct Beat {
    pub id: &'static str,
    pub title: &'static str,
    pub chapter: u8,
    pub act: u8,
    pub hero: Option<Hero>,
    pub prerequisite: Option<&'static str>,
    pub flag: &'static str,
    pub character: &'static str,
    pub premise: &'static str,
    pub vow: &'static str,
    pub bargain: &'static str,
    pub reward_card: &'static str,
    pub reward_gold: i32,
    pub reward_heal: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Sworn,
    Family,
    Mentor,
    Friend,
    Rival,
    Patron,
    Betrayal,
    Alliance,
    Inherited,
}

impl Relation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Sworn => "结义", Self::Family => "宗亲", Self::Mentor => "师徒",
            Self::Friend => "知己", Self::Rival => "宿敌", Self::Patron => "知遇",
            Self::Betrayal => "背盟", Self::Alliance => "联姻/同盟", Self::Inherited => "传承",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bond {
    pub protagonist: Hero,
    pub companion: &'static str,
    pub relation: Relation,
    pub chapter: u8,
    pub flag: &'static str,
    pub note: &'static str,
}

pub const BONDS: &[Bond] = &[
    Bond { protagonist: Hero::GuanYu, companion: "刘备", relation: Relation::Sworn, chapter: 1, flag: "taoyuan_oath", note: "一诺穿越兴亡，战斗开场义气护甲增加" },
    Bond { protagonist: Hero::GuanYu, companion: "张飞", relation: Relation::Sworn, chapter: 1, flag: "taoyuan_oath", note: "三兄弟相救相携" },
    Bond { protagonist: Hero::GuanYu, companion: "张飞", relation: Relation::Sworn, chapter: 2, flag: "anxi_seal", note: "安喜县拒贿弃印，见义须同时顾及暴烈与怜民" },
    Bond { protagonist: Hero::GuanYu, companion: "曹操", relation: Relation::Patron, chapter: 5, flag: "wenjiu_oath", note: "汜水关前以武勇举荐无职微者；盟军粮秣不济是另一笔旧账" },
    Bond { protagonist: Hero::GuanYu, companion: "曹操", relation: Relation::Patron, chapter: 25, flag: "baima_debt", note: "曹营礼遇与华容报恩构成跨幕因果" },
    Bond { protagonist: Hero::GuanYu, companion: "周仓", relation: Relation::Friend, chapter: 28, flag: "ancient_reunion", note: "古城聚义与旧部重逢" },
    Bond { protagonist: Hero::GuanYu, companion: "吕蒙", relation: Relation::Rival, chapter: 75, flag: "jingzhou_warning", note: "荆州之得失从白衣渡江埋下回声" },
    Bond { protagonist: Hero::ZhugeLiang, companion: "刘备", relation: Relation::Patron, chapter: 37, flag: "longzhong_plan", note: "草庐三请之后，构筑开局获得谋略" },
    Bond { protagonist: Hero::ZhugeLiang, companion: "庞统", relation: Relation::Friend, chapter: 57, flag: "phoenix_letter", note: "卧龙凤雏，同辈相知亦各负使命" },
    Bond { protagonist: Hero::ZhugeLiang, companion: "周瑜", relation: Relation::Rival, chapter: 44, flag: "redcliff_wind", note: "同盟相争，风火相济" },
    Bond { protagonist: Hero::ZhugeLiang, companion: "姜维", relation: Relation::Inherited, chapter: 93, flag: "successor_scroll", note: "衣钵可传，使命未必遂愿" },
    Bond { protagonist: Hero::ZhugeLiang, companion: "司马懿", relation: Relation::Rival, chapter: 95, flag: "empty_city", note: "知其谨慎，借彼猜疑" },
    Bond { protagonist: Hero::SunShangxiang, companion: "孙策", relation: Relation::Family, chapter: 7, flag: "sun_father_dispatch", note: "孙坚岘山遇伏后，孙氏子女从旧部证言继承的不只是兵权，也包括质疑孤进的责任" },
    Bond { protagonist: Hero::SunShangxiang, companion: "孙权", relation: Relation::Family, chapter: 54, flag: "jiangdong_covenant", note: "江东家国与个人抉择交叠" },
    Bond { protagonist: Hero::SunShangxiang, companion: "刘备", relation: Relation::Alliance, chapter: 54, flag: "jiangdong_covenant", note: "联姻既是策谋，也是人的命运" },
    Bond { protagonist: Hero::SunShangxiang, companion: "吴国太", relation: Relation::Family, chapter: 54, flag: "jiangdong_covenant", note: "佛寺相看，既见家事亦见国事" },
    Bond { protagonist: Hero::SunShangxiang, companion: "赵云", relation: Relation::Rival, chapter: 61, flag: "river_divide", note: "截江夺阿斗的冲突不能抹去" },
    Bond { protagonist: Hero::SunShangxiang, companion: "陆逊", relation: Relation::Alliance, chapter: 84, flag: "yiling_ashes", note: "夷陵之火对应江东内政与蜀吴裂痕" },
    Bond { protagonist: Hero::CaoCao, companion: "陈宫", relation: Relation::Friend, chapter: 4, flag: "sevenstar_vow", note: "中牟释缚与吕家误杀，忠义与疑惧都随逃亡延续" },
    Bond { protagonist: Hero::CaoCao, companion: "陈宫", relation: Relation::Rival, chapter: 10, flag: "xuzhou_debt", note: "徐州劝止连坐百姓遭拒；当年救曹的人最终因这笔军纪与民命旧账再度远去" },
    Bond { protagonist: Hero::CaoCao, companion: "荀彧", relation: Relation::Mentor, chapter: 14, flag: "xu_capital", note: "迎天子定许都，谋主与雄主各有守望" },
    Bond { protagonist: Hero::CaoCao, companion: "郭嘉", relation: Relation::Friend, chapter: 33, flag: "liaodong_letter", note: "遗计定辽东，人才并非一次性消耗" },
    Bond { protagonist: Hero::CaoCao, companion: "许攸", relation: Relation::Patron, chapter: 30, flag: "guandu_grain", note: "乌巢粮道的转机由情报和旧交构成" },
    Bond { protagonist: Hero::CaoCao, companion: "关羽", relation: Relation::Patron, chapter: 25, flag: "baima_debt", note: "白马结恩，到华容留下回响" },
    Bond { protagonist: Hero::CaoCao, companion: "袁绍", relation: Relation::Rival, chapter: 30, flag: "guandu_grain", note: "兵多未必胜，粮断则势倾" },
    Bond { protagonist: Hero::LuBu, companion: "丁原", relation: Relation::Betrayal, chapter: 3, flag: "red_hare", note: "赤兔美名之下的第一道背盟伤痕" },
    Bond { protagonist: Hero::LuBu, companion: "董卓", relation: Relation::Betrayal, chapter: 9, flag: "phoenix_pavilion", note: "一时枭雄的权势无法换来忠义" },
    Bond { protagonist: Hero::LuBu, companion: "王允", relation: Relation::Alliance, chapter: 9, flag: "changan_account", note: "诛董不等于善后；拒赦与失守留下政治责任" },
    Bond { protagonist: Hero::LuBu, companion: "貂蝉", relation: Relation::Alliance, chapter: 8, flag: "phoenix_pavilion", note: "连环计推动权力裂变；戏曲演义非正史铁证" },
    Bond { protagonist: Hero::LuBu, companion: "陈宫", relation: Relation::Friend, chapter: 19, flag: "white_gate", note: "劝谏不听，遗憾延续到白门楼" },
    Bond { protagonist: Hero::LuBu, companion: "刘备", relation: Relation::Rival, chapter: 16, flag: "halberd_shot", note: "辕门射戟的解围不代表恩怨尽解" },
    Bond { protagonist: Hero::CaoCao, companion: "王垕", relation: Relation::Betrayal, chapter: 17, flag: "shouchun_grain_account", note: "小斛是主帅之令，王垕按令发粮却被斩顶罪；玩家可架空选择公开军令担责" },
    Bond { protagonist: Hero::LuBu, companion: "陈登", relation: Relation::Alliance, chapter: 17, flag: "seven_armies_account", note: "陈登联络韩暹杨奉反戈破袁，后来暗通曹操；盟约与军纪应分别记账" },
    Bond { protagonist: Hero::CaoCao, companion: "贾诩", relation: Relation::Rival, chapter: 18, flag: "anzhong_field_report", note: "贾诩先阻追后劝追，曹军安众初胜不掩二次追击损失；敌军判断也须入军报" },
    Bond { protagonist: Hero::LuBu, companion: "陈宫", relation: Relation::Mentor, chapter: 18, flag: "xiaopei_courier_guard", note: "截获密书不等于可以杀使；玩家可架空选择保全来使并预警百姓" },
];

include!("lore_beats.rs");

#[derive(Clone, Copy, Debug)]
pub struct EasterEgg {
    pub id: &'static str,
    pub name: &'static str,
    pub hero: Option<Hero>,
    pub chapters: &'static [u8],
    pub flags: &'static [&'static str],
    pub reward_gold: i32,
    pub reward_heal: i32,
    pub hint: &'static str,
}

/// Hidden conclusions are only awarded after the *entire* flag chain.
/// Alternate monetary choices never plant a prerequisite, so they cannot
/// claim the same narrative ending accidentally.
pub const EASTER_EGGS: &[EasterEgg] = &[
    EasterEgg {id:"sword_debt",name:"青龙旧债",hero:Some(Hero::GuanYu),chapters:&[25,50],
        flags:&["baima_debt","huarong_mercy"],reward_gold:35,reward_heal:8,
        hint:"青龙刀刃上的旧日刀痕，终于不再指向那条泥泞的山道。"},
    EasterEgg {id:"brothers_return",name:"义旗归故城",hero:Some(Hero::GuanYu),chapters:&[1,28],
        flags:&["taoyuan_oath","ancient_reunion"],reward_gold:15,reward_heal:12,
        hint:"风吹义字旗，无人再问旧主是否还认得故人。"},
    EasterEgg {id:"after_wind",name:"东风遗灯",hero:Some(Hero::ZhugeLiang),chapters:&[37,49,104],
        flags:&["longzhong_plan","redcliff_wind","star_lantern"],reward_gold:20,reward_heal:16,
        hint:"昔日借风之人，将最后一盏灯留给了未能归来的春天。"},
    EasterEgg {id:"phoenix_legacy",name:"卧龙凤雏",hero:Some(Hero::ZhugeLiang),chapters:&[37,57],
        flags:&["longzhong_plan","phoenix_letter"],reward_gold:28,reward_heal:5,
        hint:"一纸旧书不能使凤雏再来，却能让后世记住两位谋士。"},
    EasterEgg {id:"taishi_return",name:"神亭守诺旧簿",hero:Some(Hero::SunShangxiang),chapters:&[7,15,84],
        flags:&["sun_father_dispatch","taishi_trust","sun_yiling_relief"],reward_gold:24,reward_heal:7,
        hint:"岘山旧军报、神亭按期归营与夷陵渡口的救济账，记着孙氏军令应如何对人负责。"},
    EasterEgg {id:"rivers_two",name:"江水两岸",hero:Some(Hero::SunShangxiang),chapters:&[54,61,84],
        flags:&["jiangdong_covenant","river_divide","yiling_ashes"],reward_gold:18,reward_heal:15,
        hint:"江水自东流，棋盘上终于摆下两国百姓的位置。"},
    EasterEgg {id:"xuhuang_oath",name:"许都不弑旧主",hero:Some(Hero::CaoCao),chapters:&[14,30,33],
        flags:&["xuhuang_honor","guandu_grain","liaodong_letter"],reward_gold:22,reward_heal:6,
        hint:"徐晃不杀旧主而择新主；乌巢粮火与辽东遗计之后，军中仍留有不以人头换官赏的旧令。"},
    EasterEgg {id:"grain_book",name:"乌巢遗策",hero:Some(Hero::CaoCao),chapters:&[14,30,33],
        flags:&["xu_capital","guandu_grain","liaodong_letter"],reward_gold:70,reward_heal:0,
        hint:"郭嘉不在，粮道与旧书却仍替曹营作答。"},
    EasterEgg {id:"xuzhou_ledger",name:"徐州民命旧账",hero:Some(Hero::CaoCao),chapters:&[4,10,14,30],
        flags:&["sevenstar_vow","xuzhou_debt","xu_capital","guandu_grain"],
        reward_gold:20,reward_heal:5,
        hint:"从中牟救命到徐州劝止，从迎驾到官渡粮道，军令与民命都记在旧账上。"},
    EasterEgg {id:"yan_base",name:"兖州守本粮策",hero:Some(Hero::CaoCao),chapters:&[10,12,14,30],
        flags:&["xuzhou_debt","puyang_supply","xu_capital","guandu_grain"],
        reward_gold:24,reward_heal:8,
        hint:"濮阳火后不忘兖州粮荒，徐州旧债与官渡军粮终于连成一条守本之策。"},
    EasterEgg {id:"xiaopei_guest_oath",name:"小沛寄军旧恩",hero:Some(Hero::LuBu),chapters:&[9,13,16,19],
        flags:&["changan_account","xiaopei_refuge","halberd_shot","white_gate"],
        reward_gold:24,reward_heal:7,
        hint:"长安失守时无人肯留，徐州却给了小沛与军粮；辕门一箭和白门残雪，终须回答客军应守的信义。"},
    EasterEgg {id:"shouchun_grain_debt",name:"寿春粮秤旧案",hero:Some(Hero::CaoCao),chapters:&[12,17,30],
        flags:&["puyang_supply","shouchun_grain_account","guandu_grain"],reward_gold:16,reward_heal:5,
        hint:"濮阳退兵、寿春小斛与乌巢粮道连成一册，主帅终于不再让仓官独担军粮之责。"},
    EasterEgg {id:"seven_armies_debt",name:"辕门军书后账",hero:Some(Hero::LuBu),chapters:&[16,17,19],
        flags:&["halberd_shot","seven_armies_account","white_gate"],reward_gold:18,reward_heal:5,
        hint:"辕门止战、七路反戈与白门问责相照，胜仗之外还须清点被军马惊散的百姓。"},
    EasterEgg {id:"anzhong_reckoning",name:"安众虚实军报",hero:Some(Hero::CaoCao),chapters:&[17,18,30],
        flags:&["shouchun_grain_account","anzhong_field_report","guandu_grain"],reward_gold:18,reward_heal:5,
        hint:"寿春小斛、安众两次追击与官渡粮道都入同一份军报；不再只记录对自己有利的胜绩。"},
    EasterEgg {id:"xiaopei_courier_reckoning",name:"小沛留使旧账",hero:Some(Hero::LuBu),chapters:&[16,17,18,19],
        flags:&["halberd_shot","seven_armies_account","xiaopei_courier_guard","white_gate"],reward_gold:16,reward_heal:5,
        hint:"辕门止兵、七路反戈与小沛密使三事，终于在白门楼前追问一纸军令如何对人负责。"},
    EasterEgg {id:"redhare_returns",name:"赤兔无主",hero:Some(Hero::LuBu),chapters:&[3,8,16,19],
        flags:&["red_hare","phoenix_pavilion","halberd_shot","white_gate"],reward_gold:30,reward_heal:10,
        hint:"赤兔不再认旧旗，马蹄声却仍追问那一次背盟。"},
    EasterEgg {id:"dynasty_cycle",name:"玉玺归尘",hero:None,chapters:&[6,120],
        flags:&["jade_whisper","jin_epilogue"],reward_gold:120,reward_heal:0,
        hint:"从井中玉匣到晋朝新诏，金印从来不会替人回答得失。"},
];

pub fn newly_completed_eggs(
    hero: Hero, flags: &[String], already: &[String]
) -> Vec<&'static EasterEgg> {
    EASTER_EGGS.iter().filter(|e|
        e.hero.is_none_or(|owner| owner == hero)
            && !already.iter().any(|unlocked| unlocked == e.id)
            && e.flags.iter().all(|flag| flags.iter().any(|known| known == flag))
    ).collect()
}

pub fn beat(id: &str) -> Option<&'static Beat> { BEATS.iter().find(|b| b.id == id) }

/// For deterministic scenario selection: favor a hero-specific prerequisite payoff,
/// then the hero's introductory chapter, finally a wider shared historic clue.
/// It consumes no randomness; each encounter is used at most once per run.
pub fn next_beat(hero: Hero, act: u8, flags: &[String], seen: &[String]) -> Option<&'static Beat> {
    let eligible = |b: &&Beat| {
        b.act == act && !seen.iter().any(|id| id == b.id)
        && b.prerequisite.is_none_or(|key| flags.iter().any(|flag| flag == key))
        && b.hero.is_none_or(|required| required == hero)
    };
    BEATS.iter().filter(eligible).min_by_key(|b| {
        let is_hero = b.hero == Some(hero);
        let is_payoff = b.prerequisite.is_some();
        (if is_hero && is_payoff { 0 } else if is_hero { 1 } else if is_payoff { 2 } else { 3 }, b.chapter)
    })
}

pub fn bonds_for(hero: Hero, flags: &[String]) -> Vec<&'static Bond> {
    BONDS.iter().filter(|b| b.protagonist == hero && flags.iter().any(|s| s == b.flag)).collect()
}

pub fn provenance_url(chapter: u8) -> String {
    format!("https://zh.wikisource.org/wiki/三國演義/第{chapter:03}回")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_beats_have_source_chapters_unique_ids_and_known_cards() {
        let mut ids = std::collections::BTreeSet::new();
        for beat in BEATS {
            assert!((1..=120).contains(&beat.chapter), "{:?}",beat.id);
            assert!(ids.insert(beat.id), "duplicate beat ID {}",beat.id);
            assert!(crate::content::card_spec(beat.reward_card).is_some(), "{}",beat.reward_card);
            assert!(beat.act <= 2);
            assert!(!beat.title.is_empty() && !beat.premise.is_empty());
            assert!(provenance_url(beat.chapter).contains("三國演義"));
        }
        for b in BEATS {
            if let Some(required) = b.prerequisite {
                assert!(BEATS.iter().any(|old| old.flag == required && old.chapter <= b.chapter),
                    "orphan foreshadowing {}",b.id);
            }
        }
    }
    #[test]
    fn chapter_links_and_relationships_have_provenance() {
        assert!(BONDS.len() >= 25);
        for bond in BONDS {
            assert!((1..=120).contains(&bond.chapter));
            assert!(BEATS.iter().any(|b| b.flag == bond.flag),
                "bond missing storyline: {}",bond.companion);
        }
    }
    #[test]
    fn all_easter_eggs_have_real_and_reachable_source_chains() {
        let mut ids = std::collections::BTreeSet::new();
        for egg in EASTER_EGGS {
            assert!(ids.insert(egg.id), "duplicate easter egg");
            assert!(egg.chapters.iter().all(|ch| (1..=120).contains(ch)));
            for flag in egg.flags {
                assert!(BEATS.iter().any(|beat| beat.flag == *flag), "unreachable {}",egg.id);
            }
        }
        let flags = vec!["baima_debt".into(),"huarong_mercy".into()];
        assert_eq!(newly_completed_eggs(Hero::GuanYu,&flags,&[])[0].id, "sword_debt");
        assert!(newly_completed_eggs(Hero::ZhugeLiang,&flags,&[]).is_empty());
        assert!(newly_completed_eggs(Hero::GuanYu,&flags,&["sword_debt".into()]).is_empty());
    }
    #[test]
    fn early_pledge_and_public_office_create_a_cross_act_debt() {
        let seen=vec!["taoyuan".into()];
        assert_eq!(next_beat(Hero::GuanYu,0,&["taoyuan_oath".into()],&seen)
            .map(|b|b.id),Some("anxi"));
        assert_ne!(next_beat(Hero::GuanYu,2,&["taoyuan_oath".into()],&[])
            .map(|b|b.id),Some("jingzhou"),
            "an oath without the anti-bribery choice cannot unlock Jingzhou");
        assert_eq!(next_beat(Hero::GuanYu,2,
            &["taoyuan_oath".into(),"anxi_seal".into()],&[])
            .map(|b|b.id),Some("jingzhou"));
        assert_eq!(next_beat(Hero::GuanYu,2,
            &["taoyuan_oath".into(),"anxi_seal".into(),"jingzhou_warning".into()],
            &["jingzhou".into()]).map(|b|b.id),Some("maicheng"));
    }
    #[test]
    fn without_an_earlier_choice_its_payoff_is_not_selected() {
        let no_flags = Vec::new();
        let seen = Vec::new();
        assert_eq!(next_beat(Hero::GuanYu, 0, &no_flags, &seen).unwrap().id,"taoyuan");
        assert!(next_beat(Hero::GuanYu, 1, &no_flags, &seen).is_none());
        assert_eq!(next_beat(Hero::GuanYu, 1, &["baima_debt".into()], &seen).unwrap().id,"huarong");
    }
}
