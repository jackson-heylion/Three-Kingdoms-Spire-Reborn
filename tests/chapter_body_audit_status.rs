//! Separate ledger status verification; metadata is not proof of edition collation.
#[test]
fn chapter_reading_statuses_cannot_be_mistaken_for_completed_textual_collation() {
    let lines=include_str!("../data/chapter_body_audit.tsv").lines().skip(1)
        .map(|line|line.split('\t').map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let reviewed=lines.iter().filter(|fields|fields[3]=="正文通读/未完成异本对校")
        .map(|fields|fields[0].as_str()).collect::<Vec<_>>();
    let expected=(1..=62).map(|n|format!("{n:03}")).collect::<Vec<_>>();
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
    for (n,oldid,id) in [(45,"2568007","jiangkou_inquest"),(46,"2568009","borrow_arrows"),(47,"2568010","chain_ship_ledger")] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(id));
    }
    for (n,oldid,id) in [(48,"2568011","river_banquet_inquest"),(49,"2568012","redcliff"),(50,"2568013","huarong")] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(id));
    }
    for (n,oldid,id) in [(51,"2568014","nanjun_claim_ledger"),(52,"2568016","jingxiang_lease_ledger"),(53,"2568015","changsha_mercy")] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(id));
    }
    assert!(lines[52][5].contains("hefei_memorial"));
    for (n,oldid,id) in [(54,"2568017","jingzhou_pledge_copy"),(55,"2568018","nanxu_escape_witness"),(56,"2568019","tongque_dispatch_ledger")] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(id));
    }
    assert!(lines[55][5].contains("jingzhou_false_passage"));
    for (n,oldid,id) in [(57,"2568020","fengchu"),(58,"7909160","tongguan_retreat"),(59,"2568022","weinan_division")] {
        assert!(lines[n-1][4].contains(&format!("oldid={oldid}")));
        assert!(lines[n-1][5].contains(id));
    }
    assert!(lines[56][5].contains("leiyang_review") && lines[56][5].contains("mateng_case"));
    assert_eq!(lines.iter().filter(|fields|fields[3]=="已完成全回正文审校").count(),0);
}
