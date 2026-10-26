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
        ("river_banquet_inquest","chain_ship_risk_record",2),
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

