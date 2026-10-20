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
    Bond { protagonist: Hero::CaoCao, companion: "荀彧", relation: Relation::Mentor, chapter: 14, flag: "xu_capital", note: "迎天子定许都，谋主与雄主各有守望" },
    Bond { protagonist: Hero::CaoCao, companion: "郭嘉", relation: Relation::Friend, chapter: 33, flag: "liaodong_letter", note: "遗计定辽东，人才并非一次性消耗" },
    Bond { protagonist: Hero::CaoCao, companion: "许攸", relation: Relation::Patron, chapter: 30, flag: "guandu_grain", note: "乌巢粮道的转机由情报和旧交构成" },
    Bond { protagonist: Hero::CaoCao, companion: "关羽", relation: Relation::Patron, chapter: 25, flag: "baima_debt", note: "白马结恩，到华容留下回响" },
    Bond { protagonist: Hero::CaoCao, companion: "袁绍", relation: Relation::Rival, chapter: 30, flag: "guandu_grain", note: "兵多未必胜，粮断则势倾" },
    Bond { protagonist: Hero::LuBu, companion: "丁原", relation: Relation::Betrayal, chapter: 3, flag: "red_hare", note: "赤兔美名之下的第一道背盟伤痕" },
    Bond { protagonist: Hero::LuBu, companion: "董卓", relation: Relation::Betrayal, chapter: 9, flag: "phoenix_pavilion", note: "一时枭雄的权势无法换来忠义" },
    Bond { protagonist: Hero::LuBu, companion: "貂蝉", relation: Relation::Alliance, chapter: 8, flag: "phoenix_pavilion", note: "连环计推动权力裂变；戏曲演义非正史铁证" },
    Bond { protagonist: Hero::LuBu, companion: "陈宫", relation: Relation::Friend, chapter: 19, flag: "white_gate", note: "劝谏不听，遗憾延续到白门楼" },
    Bond { protagonist: Hero::LuBu, companion: "刘备", relation: Relation::Rival, chapter: 16, flag: "halberd_shot", note: "辕门射戟的解围不代表恩怨尽解" },
];

/// Each beat belongs to a literary act of *this game*, not a strict timeline of historical dates.
/// A foreshadowing flag only becomes available through the corresponding first choice.
pub const BEATS: &[Beat] = &[
    Beat {id:"taoyuan",title:"桃园残誓",chapter:1,act:0,hero:Some(Hero::GuanYu),prerequisite:None,flag:"taoyuan_oath",character:"刘备、张飞",premise:"荒园旧土有三杯浊酒。今夜再起誓言，是背负责任，还是只取其名？",vow:"重整义旗",bargain:"卖掉盟誓旧物",reward_card:"oath_guard",reward_gold:12,reward_heal:8},
    Beat {id:"anxi",title:"安喜断印",chapter:2,act:0,hero:Some(Hero::GuanYu),prerequisite:Some("taoyuan_oath"),flag:"anxi_seal",character:"刘备、张飞、督邮",premise:"督邮逼县吏诬陷刘备，门前父老求告无门；张飞怒鞭督邮，刘备制止杀戮。关羽想起桃园誓词：救民与快意报复，并非一回事。",vow:"劝止杀戮，缴印离任保全百姓",bargain:"私收督邮赔银，暂保虚职",reward_card:"hold_line",reward_gold:25,reward_heal:7},
    Beat {id:"wenjiu",title:"温酒与断粮",chapter:5,act:0,hero:Some(Hero::GuanYu),prerequisite:Some("anxi_seal"),flag:"wenjiu_oath",character:"曹操、袁术、孙坚、华雄",premise:"孙坚因袁术疑忌断粮失势，祖茂战殁；关羽斩华雄归帐，酒尚温而袁术仍以马弓手轻之。功名与粮秣，孰轻孰重？",vow:"记下曹操举荐与盟军断粮之责",bargain:"独取斩将赏格，不问粮道",reward_card:"pojun",reward_gold:18,reward_heal:5},
    Beat {id:"gucheng",title:"古城重会",chapter:28,act:0,hero:Some(Hero::GuanYu),prerequisite:Some("taoyuan_oath"),flag:"ancient_reunion",character:"张飞、周仓",premise:"刀锋误认故人，旧部周仓递上一面未曾褪色的义旗。",vow:"释疑重整旧部",bargain:"先攻疑敌",reward_card:"yidao",reward_gold:24,reward_heal:8},
    Beat {id:"baima",title:"白马旧恩",chapter:25,act:0,hero:Some(Hero::GuanYu),prerequisite:None,flag:"baima_debt",character:"曹操、颜良",premise:"战书背后是曹营待客的厚礼；刀下功名也会成为未来的债。",vow:"记下曹营恩义",bargain:"只收军资",reward_card:"qinglong",reward_gold:28,reward_heal:2},
    Beat {id:"huarong",title:"华容雨路",chapter:50,act:1,hero:Some(Hero::GuanYu),prerequisite:Some("baima_debt"),flag:"huarong_mercy",character:"曹操",premise:"旧日恩客逃至岔路。军令与私义在刀锋相撞，先前因果终于来偿。",vow:"以义偿恩，留其生路",bargain:"按军令索回赏格",reward_card:"guanxi",reward_gold:16,reward_heal:12},
    Beat {id:"jingzhou",title:"荆州警柝",chapter:75,act:2,hero:Some(Hero::GuanYu),prerequisite:Some("anxi_seal"),flag:"jingzhou_warning",character:"吕蒙、陆逊",premise:"江上看似无兵，渔火却连成密信。傲敌或审势，只在一念之间。",vow:"审查江防",bargain:"轻骑抢夺粮仓",reward_card:"tiebi",reward_gold:25,reward_heal:4},
    Beat {id:"longzhong",title:"草庐三请",chapter:37,act:0,hero:Some(Hero::ZhugeLiang),prerequisite:None,flag:"longzhong_plan",character:"刘备、徐庶",premise:"草庐尚寒，故人荐书未干。三顾之礼，换来三分天下的责任。",vow:"定下隆中之策",bargain:"出售古籍残卷",reward_card:"thought_spring",reward_gold:10,reward_heal:4},
    Beat {id:"borrow_arrows",title:"雾江借箭",chapter:46,act:1,hero:Some(Hero::ZhugeLiang),prerequisite:Some("longzhong_plan"),flag:"mist_arrows",character:"周瑜、鲁肃",premise:"江面浓雾拥船。胜负未必在造箭的匠炉，而在识天时的人心。",vow:"以雾布疑兵",bargain:"冒进夺船",reward_card:"caochuan",reward_gold:20,reward_heal:0},
    Beat {id:"redcliff",title:"东风未至",chapter:49,act:1,hero:Some(Hero::ZhugeLiang),prerequisite:Some("longzhong_plan"),flag:"redcliff_wind",character:"周瑜、黄盖",premise:"连环战船已系，风向却未转。苦肉与借风相隔一线，盟友各藏心思。",vow:"守候风候，点燃火阵",bargain:"抢先夺粮",reward_card:"huogong",reward_gold:18,reward_heal:5},
    Beat {id:"fengchu",title:"凤雏书信",chapter:57,act:1,hero:Some(Hero::ZhugeLiang),prerequisite:Some("redcliff_wind"),flag:"phoenix_letter",character:"庞统、周瑜",premise:"白纸上有江东故友的墨痕。权谋虽胜，人的命运却未因此轻盈。",vow:"保存凤雏往简",bargain:"卖书换甲",reward_card:"mouduan",reward_gold:22,reward_heal:5},
    Beat {id:"jiangwei",title:"天水传书",chapter:93,act:2,hero:Some(Hero::ZhugeLiang),prerequisite:Some("longzhong_plan"),flag:"successor_scroll",character:"姜维",premise:"天水新识有胆有识，接过谋略容易，背负旧志却难。",vow:"传授兵法",bargain:"借其声望征粮",reward_card:"bagua",reward_gold:19,reward_heal:3},
    Beat {id:"wuzhang",title:"五丈星灯",chapter:104,act:2,hero:Some(Hero::ZhugeLiang),prerequisite:Some("redcliff_wind"),flag:"star_lantern",character:"姜维、司马懿",premise:"七星灯火照见前路，也照见不可逆的岁月。把余火留给后人。",vow:"写下传承锦囊",bargain:"以寿数换军令",reward_card:"qimou",reward_gold:18,reward_heal:6},
    Beat {id:"sun_father_letter",title:"岘山旧军报",chapter:7,act:0,hero:Some(Hero::SunShangxiang),prerequisite:None,flag:"sun_father_dispatch",character:"孙坚、孙策、程普、韩当",premise:"后人收得旧军报：孙坚先渡江破黄祖，继而不听韩当劝谏、仅率数十骑追吕公入岘山伏地。袁术来信挑起旧怨；孙氏女儿并未亲历此战，却要辨清功名与军情之间的代价。",vow:"保存伏击和撤军的证言，交还旧部家属",bargain:"只留下胜战捷报，换取军中赏银",reward_card:"qiaoying",reward_gold:12,reward_heal:7},
    Beat {id:"jiangdong_wedding",title:"甘露寺旧约",chapter:54,act:0,hero:Some(Hero::SunShangxiang),prerequisite:None,flag:"jiangdong_covenant",character:"刘备、孙权、吴国太",premise:"红烛映着两国地图。婚盟与兵盟不是同一张契书。",vow:"把盟约交与良知",bargain:"换取东吴密帛",reward_card:"arrows_on_discard",reward_gold:20,reward_heal:6},
    Beat {id:"river",title:"截江帆影",chapter:61,act:1,hero:Some(Hero::SunShangxiang),prerequisite:Some("jiangdong_covenant"),flag:"river_divide",character:"赵云、张飞",premise:"归帆带着家书，也带着两国互不相让的人质抉择。",vow:"放下兵刃护送妇孺",bargain:"夺船扬帆",reward_card:"yidong",reward_gold:20,reward_heal:9},
    Beat {id:"sun_yiling_shelter",title:"岘山旧信·夷陵新舟",chapter:84,act:2,hero:Some(Hero::SunShangxiang),prerequisite:Some("sun_father_dispatch"),flag:"sun_yiling_relief",character:"孙策旧部、陆逊、夷陵溃卒",premise:"夷陵山道火光四起，旧军报记着父亲追敌入险地的代价。面对蜀吴两军和沿江逃难百姓，孙家后人是否又要把一次取胜写成无人付账的捷报？",vow:"按旧信清点伤亡，开放江舟护送各方难民",bargain:"封锁渡口抢运军械，隐去平民损失",reward_card:"yidong",reward_gold:17,reward_heal:10},
    Beat {id:"yiling",title:"夷陵余烬",chapter:84,act:2,hero:Some(Hero::SunShangxiang),prerequisite:Some("jiangdong_covenant"),flag:"yiling_ashes",character:"陆逊、刘备",premise:"七百里连营燃尽旧约，只有残局还在问：昔日联姻何以至此？",vow:"收容溃卒，修补旧盟",bargain:"追击孤军",reward_card:"huogong",reward_gold:15,reward_heal:12},
    Beat {id:"sevenstar",title:"七星宝刀",chapter:4,act:0,hero:Some(Hero::CaoCao),prerequisite:None,flag:"sevenstar_vow",character:"王允、陈宫、吕伯奢",premise:"王允借刀，曹操失刺而托词献刀；中牟得陈宫相救，吕家误杀又伤同行信义。讨贼之名与自保的代价，不能当成同一件事。",vow:"承认错杀，记下陈宫规劝",bargain:"隐去旧案，只收逃亡资粮",reward_card:"jianxiong",reward_gold:12,reward_heal:5},
    Beat {id:"xudu",title:"迁都许昌",chapter:14,act:0,hero:Some(Hero::CaoCao),prerequisite:None,flag:"xu_capital",character:"汉献帝、荀彧",premise:"护送天子的车辙延伸至许。权力可聚，名分也会反噬持有者。",vow:"整饬朝廷粮道",bargain:"以官印换重赏",reward_card:"granary_support",reward_gold:15,reward_heal:4},
    Beat {id:"guandu",title:"乌巢粮火",chapter:30,act:1,hero:Some(Hero::CaoCao),prerequisite:Some("xu_capital"),flag:"guandu_grain",character:"许攸、袁绍",premise:"十万旌旗之外，决定战局的是无人守好的粮囤。旧识的一纸情报，比百万刀枪更重。",vow:"急袭乌巢",bargain:"守营收取军需",reward_card:"tuntian",reward_gold:55,reward_heal:2},
    Beat {id:"liaodong",title:"郭嘉遗计",chapter:33,act:2,hero:Some(Hero::CaoCao),prerequisite:Some("guandu_grain"),flag:"liaodong_letter",character:"郭嘉、袁尚",premise:"军师遗书封泥未开，辽东的雪却已落定。懂得等候，亦是用兵。",vow:"遵遗计缓图",bargain:"即刻远征",reward_card:"haoling",reward_gold:25,reward_heal:12},
    Beat {id:"redhare",title:"赤兔旧主",chapter:3,act:0,hero:Some(Hero::LuBu),prerequisite:None,flag:"red_hare",character:"丁原、李肃、董卓",premise:"洛阳外兵入京，温明园丁原抗争；李肃先献赤兔，再用金珠与功名相诱，吕布亲口许下杀义父之诺。刀锋未出，信任已断。",vow:"向旧部说明背盟之责",bargain:"收下财宝掩饰旧约",reward_card:"blood_courage",reward_gold:25,reward_heal:0},
    Beat {id:"pavilion",title:"凤仪亭风波",chapter:8,act:0,hero:Some(Hero::LuBu),prerequisite:Some("red_hare"),flag:"phoenix_pavilion",character:"王允、貂蝉、董卓",premise:"凤仪亭里，一段私情牵动权臣生死。连环计能除暴，亦能撕开人的尊严。",vow:"识破权术与妄念",bargain:"纵兵夺下密函",reward_card:"huaji",reward_gold:26,reward_heal:2},
    Beat {id:"halberd",title:"辕门一箭",chapter:16,act:1,hero:Some(Hero::LuBu),prerequisite:Some("phoenix_pavilion"),flag:"halberd_shot",character:"刘备、纪灵",premise:"一箭定兵戈，是神力，也是给双方一次停战的机会。",vow:"解围止战",bargain:"以箭威索粮",reward_card:"huaji",reward_gold:35,reward_heal:8},
    Beat {id:"whitegate",title:"白门楼残雪",chapter:19,act:2,hero:Some(Hero::LuBu),prerequisite:Some("halberd_shot"),flag:"white_gate",character:"陈宫、曹操",premise:"陈宫犹记昔日劝谏，铁锁之下再无回头路。此行是否愿承认旧日错失？",vow:"偿还故人谏语",bargain:"搏命突围",reward_card:"tianxia",reward_gold:40,reward_heal:10},
    Beat {id:"maicheng",title:"麦城粮断",chapter:76,act:2,hero:Some(Hero::GuanYu),prerequisite:Some("jingzhou_warning"),flag:"maicheng_supply",character:"徐晃、糜芳、傅士仁",premise:"旧日轻慢积成后方疑惧。麦城粮使带回空囊：不只是江防失察，旧部信任也已经断裂。",vow:"先遣使安抚旧部与百姓",bargain:"严令催粮、强攻突围",reward_card:"hold_line",reward_gold:30,reward_heal:6},
    Beat {id:"yiling_bridge",title:"八阵渡火",chapter:84,act:2,hero:Some(Hero::SunShangxiang),prerequisite:Some("river_divide"),flag:"yiling_bridge",character:"陆逊、刘备、吴班",premise:"江边火光照出旧日截江的裂隙。陆逊识破诱敌之兵，七百里营寨终成灰烬；旧盟只剩救人的余地。",vow:"开放渡口收容双方伤兵",bargain:"乘胜截取蜀军粮船",reward_card:"qiaoying",reward_gold:34,reward_heal:11},
    Beat {id:"jieting_debt",title:"街亭违令",chapter:95,act:2,hero:Some(Hero::ZhugeLiang),prerequisite:Some("successor_scroll"),flag:"jieting_accountability",character:"马谡、王平、司马懿",premise:"马谡弃道路屯山，王平劝谏不从。知兵书并不等于知兵情；受托之后，该如何承担用人之失？",vow:"保存王平军报、追究军令责任",bargain:"掩盖误判以保军心",reward_card:"kongcheng",reward_gold:16,reward_heal:9},
    Beat {id:"imperial_jade",title:"井中玉玺",chapter:6,act:0,hero:None,prerequisite:None,flag:"jade_whisper",character:"孙坚、程普、袁绍、刘表",premise:"洛阳焚毁，孙坚先灭火祭宗庙；军士自井中救起宫女遗体，锦囊中有金锁朱匣及缺角镶金的玺。孙坚秘藏而遭告发，袁绍据此迫问，又致书刘表截归途。",vow:"向盟军明示印匣来源、记下私藏争端",bargain:"隐匿印匣消息、换取紧急军资",reward_card:"zhentu",reward_gold:17,reward_heal:2},
    Beat {id:"changban",title:"长坂遗袍",chapter:41,act:0,hero:None,prerequisite:None,flag:"changban_pledge",character:"赵云、糜夫人、刘备",premise:"阵中小小襁褓比千军更沉。护一人，可能失去一城。",vow:"先护百姓",bargain:"绕阵劫粮",reward_card:"yidong",reward_gold:8,reward_heal:10},
    Beat {id:"mandate",title:"衣带密诏",chapter:20,act:1,hero:None,prerequisite:Some("jade_whisper"),flag:"edict_secret",character:"献帝、董承",premise:"锦缎中缝的密字藏不住权柄倾覆的风声。此前所见玉玺只是第一个疑点。",vow:"把密诏交给可信之人",bargain:"焚诏取赏",reward_card:"qimou",reward_gold:18,reward_heal:0},
    Beat {id:"south_wilds",title:"七擒之约",chapter:90,act:2,hero:None,prerequisite:None,flag:"seven_captures",character:"孟获、诸葛亮",premise:"南中七次围困之后，信服不是束缚。刀兵可折，民心不可强取。",vow:"释俘安边",bargain:"席卷山寨",reward_card:"tiebi",reward_gold:10,reward_heal:12},
    Beat {id:"empty_city",title:"空城余弦",chapter:95,act:2,hero:None,prerequisite:None,flag:"empty_city",character:"司马懿、诸葛亮、马谡",premise:"街亭失守，空城仍有琴声。用谋诈敌的前提，是摸清对手知己知彼的弱点。",vow:"守静示虚",bargain:"以险搏功",reward_card:"qimou",reward_gold:15,reward_heal:6},
    Beat {id:"wooden_ox",title:"木牛流马",chapter:102,act:2,hero:None,prerequisite:Some("empty_city"),flag:"wooden_ox",character:"诸葛亮、司马懿",premise:"连年征伐，走不动的不是战马，而是断续的粮道。运粮机关正待破局。",vow:"修整运输机关",bargain:"拆售机关换金",reward_card:"caochuan",reward_gold:25,reward_heal:8},
    Beat {id:"jin_unification",title:"三分归一",chapter:120,act:2,hero:None,prerequisite:Some("jade_whisper"),flag:"jin_epilogue",character:"杜预、孙皓、司马炎",premise:"一方古玺的传闻走到末章，权柄终归新姓。英雄们的誓愿是否还有继承者？",vow:"藏起史书以待后人",bargain:"送旧玺入新府",reward_card:"zonghenggong",reward_gold:40,reward_heal:4},
];

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
    EasterEgg {id:"rivers_two",name:"江水两岸",hero:Some(Hero::SunShangxiang),chapters:&[54,61,84],
        flags:&["jiangdong_covenant","river_divide","yiling_ashes"],reward_gold:18,reward_heal:15,
        hint:"江水自东流，棋盘上终于摆下两国百姓的位置。"},
    EasterEgg {id:"grain_book",name:"乌巢遗策",hero:Some(Hero::CaoCao),chapters:&[14,30,33],
        flags:&["xu_capital","guandu_grain","liaodong_letter"],reward_gold:70,reward_heal:0,
        hint:"郭嘉不在，粮道与旧书却仍替曹营作答。"},
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
