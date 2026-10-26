//! Completeness of provenance *index* only, not automated proof of close reading.
use three_kingdoms_spire_reborn::lore::BEATS;
#[test]
fn every_chapter_has_one_explicitly_scoped_audit_row_and_live_beat_refs() {
    let file=include_str!("../data/chapter_body_audit.tsv");
    let lines=file.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(),121);
    assert!(lines[0].contains("body_review_status"));
    for (index,line) in lines.iter().skip(1).enumerate() {
        let fields=line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(),6,"chapter {}",index+1);
        assert_eq!(fields[0],format!("{:03}",index+1));
        assert!(fields[2].ends_with(&format!("/第{:03}回",index+1)));
        assert_ne!(fields[3],"已完成全回正文审校");
        assert!(!fields[4].is_empty());
        for id in fields[5].split(',').filter(|id|*id!="无") {
            assert!(BEATS.iter().any(|b|b.id==id && usize::from(b.chapter)==index+1),
                "unmapped scene {id}");
        }
        for beat in BEATS.iter().filter(|b|usize::from(b.chapter)==index+1) {
            assert!(fields[5].split(',').any(|id|id==beat.id),"missing scene {}",beat.id);
        }
    }
}
#[test]
fn three_cross_act_debts_require_their_prior_flags() {
    for (id,needed,act) in [
        ("anxi","taoyuan_oath",0),
        ("jingzhou","anxi_seal",2),
        ("maicheng","jingzhou_warning",2),
        ("yiling_bridge","river_divide",2),
        ("jieting_debt","successor_scroll",2),
        ("seven_armies","halberd_shot",1),
        ("shouchun_grain","puyang_supply",1),
        ("anzhong","shouchun_grain_account",1),
        ("xiaopei_courier","seven_armies_account",1),
        ("xiapi_orders","xiaopei_courier_guard",1),
        ("xiapi_flood","anzhong_field_report",1),
        ("wangzhong_prisoners","wenjiu_oath",1),
        ("xiapi_household","wangzhong_release_record",1),
        ("jiping_testimony","xuhuang_honor",1),
        ("palace_ledger","jiping_case_record",1),
        ("five_passes","seal_return_record",1),
        ("gucheng_witness","five_passes_record",1),
        ("sun_succession","taishi_trust",0),
        ("guandu_letters","guandu_grain",1),
        ("cangting_memorial","guandu_captured_letters",1),
        ("ye_flood_ledger","cangting_memorial_record",2),
        ("jiangxia_pardon","jiangdong_hostage_record",1),
        ("bowang_orders","longzhong_conditions_record",1),
        ("xinye_evacuation","bowang_orders_record",1),
        ("dangyang_ledger","xinye_civilians",2),
        ("hanjin_ferry","xutian_guarded_emperor",2),
        ("chaisang_envoy","dangyang_refuge_account",2),
        ("jiangdong_council","chaisang_envoy_record",2),
        ("jiangdong_muster","jiangdong_hostage_record",2),
    ] {
        let beat=BEATS.iter().find(|b|b.id==id).unwrap();
        assert_eq!(beat.prerequisite,Some(needed));
        assert_eq!(beat.act,act);
        assert!(BEATS.iter().any(|earlier|
            earlier.flag == needed && (earlier.chapter < beat.chapter
                || (id == "guandu_letters" && earlier.id == "guandu"
                    && earlier.chapter == beat.chapter))),
            "prerequisite {needed} must be planted before scene {id}");
    }
}

#[test]
fn chapter_reading_statuses_cannot_be_mistaken_for_completed_textual_collation() {
    let lines=include_str!("../data/chapter_body_audit.tsv").lines().skip(1)
        .map(|line|line.split('\t').map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let reviewed=lines.iter().filter(|fields|fields[3]=="正文通读/未完成异本对校")
        .map(|fields|fields[0].as_str()).collect::<Vec<_>>();
    let expected=(1..=44).map(|n|format!("{n:03}")).collect::<Vec<_>>();
    assert_eq!(reviewed,expected.iter().map(String::as_str).collect::<Vec<_>>());
    assert!(lines[0][4].contains("oldid=2583915"));
    assert!(lines[1][4].contains("oldid=8185068"));
    assert!(lines[2][4].contains("oldid=2567965"));
    assert!(lines[3][4].contains("oldid=2567966"));
    assert!(lines[4][4].contains("oldid=7910939"));
    assert!(lines[5][4].contains("oldid=2567969"));
    assert!(lines[6][4].contains("oldid=2567970"));
    assert!(lines[7][4].contains("oldid=5314829"));
    assert!(lines[8][4].contains("oldid=2567971"));
    assert!(lines[9][4].contains("oldid=2567973"));
    assert!(lines[9][5].contains("xuzhou"));
    assert!(lines[10][4].contains("oldid=2567975"));
    assert!(lines[11][4].contains("oldid=2567974"));
    assert!(lines[11][5].contains("puyang"));
    assert!(lines[12][4].contains("oldid=8185567"));
    assert!(lines[12][5].contains("xiaopei"));
    assert!(lines[13][4].contains("oldid=2613836"));
    assert!(lines[13][5].contains("xuhuang"));
    assert!(lines[14][4].contains("oldid=8729938"));
    assert!(lines[14][5].contains("shenting"));
    assert!(lines[15][4].contains("oldid=2567978"));
    assert!(lines[15][5].contains("halberd"));
    assert!(lines[16][4].contains("oldid=2567979"));
    assert!(lines[16][5].contains("seven_armies"));
    assert!(lines[16][5].contains("shouchun_grain"));
    assert!(lines[17][4].contains("oldid=2567980"));
    assert!(lines[17][5].contains("anzhong"));
    assert!(lines[17][5].contains("xiaopei_courier"));
    assert!(lines[18][4].contains("oldid=2567981"));
    assert!(lines[18][5].contains("xiapi_orders"));
    assert!(lines[18][5].contains("xiapi_flood"));
    assert!(lines[19][4].contains("oldid=8187289"));
    assert!(lines[19][5].contains("xutian_guard"));
    assert!(lines[20][4].contains("oldid=2567982"));
    assert!(lines[20][5].contains("xuzhou_gate"));
    for (n,oldid,beat) in [
        (22,"2567985","wangzhong_prisoners"),
        (23,"2567986","jiping_testimony"),
        (24,"2567987","xiapi_household"),
    ] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(beat));
    }
    assert!(lines[23][5].contains("palace_ledger"));
    for (n,oldid,beat) in [
        (27,"8186000","five_passes"),
        (28,"2567993","gucheng_witness"),
        (29,"2567991","sun_succession"),
    ] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(beat));
    }
    for (n,oldid,beat) in [
        (30,"2567996","guandu_letters"),
        (31,"2567992","cangting_memorial"),
        (32,"2567994","ye_flood_ledger"),
    ] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(beat));
    }
    for (n,oldid,beat) in [
        (33,"2567995","northern_settlement"),
        (34,"2567997","tanxi_warning"),
        (35,"2568001","xinye_counsel"),
    ] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(beat));
    }
    for (n,oldid,beat) in [
        (36,"2568002","fancheng_counsel"),
        (37,"2567998","xushu_departure"),
        (38,"4256828","longzhong_conditions"),
    ] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(beat));
    }
    assert!(lines[36][5].contains("longzhong_letter"));
    assert!(lines[37][5].contains("jiangdong_hostage"));
    for (n,oldid,id) in [(39,"2567999","bowang_orders"),(40,"2568003","xinye_evacuation"),(41,"2568004","dangyang_ledger")] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(id));
    }
    assert!(lines[38][5].contains("jiangxia_pardon"));
    assert!(lines[40][5].contains("changban"));
    for (n,oldid,id) in [(42,"2568005","hanjin_ferry"),(43,"2568006","chaisang_envoy"),(44,"2568008","jiangdong_council")] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(id));
    }
    assert!(lines[43][5].contains("jiangdong_muster"));
    assert_eq!(lines.iter().filter(|fields|fields[3]=="已完成全回正文审校").count(),0);
}
