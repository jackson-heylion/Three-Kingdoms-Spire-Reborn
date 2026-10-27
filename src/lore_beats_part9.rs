// Chapters 045 and 047: Cao Cao's retrospective intelligence ledger.
// Chapter 046 already has the playable Zhuge Liang borrow_arrows scene;
// no duplicate event is introduced for that episode.
pub const BEATS_PART_9: &[Beat] = &[
    Beat {id:"jiangkou_inquest",title:"三江口伪书与蔡张冤案",chapter:45,act:1,hero:Some(Hero::CaoCao),prerequisite:Some("guandu_grain"),flag:"jiangkou_inquest_record",character:"曹操、周瑜、蒋干、蔡瑁、张允、鲁肃、关羽、刘备",premise:"第044回周瑜对孔明生忌，第045回先设断粮杀孔明之局，鲁肃劝止。周瑜又设宴诱刘备赴营，关羽持剑随行使伏兵未发；周瑜斩曹使，甘宁水战获胜。蔡瑁张允整顿曹军水寨，周瑜借蒋干偷取伪书，使曹操误杀两人；曹操事后知中计，却谎称二人怠慢军法，改用毛玠于禁。玩家只在曹营事后军簿记录责任，不得倒改蔡张之死。",vow:"原创选择：归档伪书来源、未核验的指控、杀将责任及新水军将领的训练成本",bargain:"原创选择：只记整肃军法，隐去伪书与错误处决",reward_card:"qimou",reward_gold:17,reward_heal:7},
    Beat {id:"chain_ship_ledger",title:"雾江折箭与连环船风险",chapter:47,act:2,hero:Some(Hero::CaoCao),prerequisite:Some("jiangkou_inquest_record"),flag:"chain_ship_risk_record",character:"曹操、荀攸、蔡中、蔡和、诸葛亮、鲁肃、黄盖、阚泽、蒋干、庞统",premise:"承第045回蔡张冤案，第046回孔明凭大雾与鲁肃二十草船取得箭枝，曹操令弓弩手隔雾放箭；荀攸遂荐蔡中蔡和诈降，周瑜识破而将计就计，黄盖受杖行苦肉计。第047回阚泽持诈降书说服曹操，蒋干再次入吴被周瑜引见庞统；庞统以北军晕船为由劝曹操连锁战船，曹操下令打造铁环。庞统求安民榜文后离营，回末被神秘道士拦住，尚未发生赤壁火攻。玩家可追记船病、火险和误判，不改曹操下令连船的小说事实。",vow:"原创选择：核对雾江折箭、真假降书、船病与连船火险，拨军粮给水军救护",bargain:"原创选择：只记连船便于操练，不记录情报漏洞与火险",reward_card:"granary_support",reward_gold:19,reward_heal:8},
];
