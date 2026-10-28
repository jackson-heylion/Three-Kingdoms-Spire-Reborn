// Deterministic scene ordering; extracted from lore.rs to keep modules small.
// A later optional retrospective must not steal the original Jingzhou entry
// from an old save that has already earned the Anxi responsibility flag.
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
        let flag = |f: &str| flags.iter().any(|x| x == f);
        let unseen = |id: &str| !seen.iter().any(|x| x == id);
        let protect_old_jingzhou = b.id == "hanjin_ferry"
            && flag("anxi_seal") && unseen("jingzhou");
        let protect_legacy = protect_old_jingzhou
            || (b.id == "changsha_mercy" && flag("anxi_seal") && unseen("jingzhou"))
            || (b.id == "nanjun_claim_ledger" && flag("redcliff_wind") && unseen("wuzhang"))
            || (b.id == "hefei_memorial" && flag("jiangdong_covenant") && unseen("yiling"))
            || (b.id == "nanxu_escape_witness" && flag("jiangdong_covenant") && unseen("yiling"))
            || ((b.id == "jingzhou_pledge_copy" || b.id == "jingzhou_false_passage")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (b.id == "tongque_dispatch_ledger" && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "mateng_case" | "tongguan_retreat" | "weinan_division" | "zhangsong_audience")
                && flag("guandu_grain") && unseen("liaodong"))
            || (b.id == "river_child_witness" && flag("jiangdong_covenant") && unseen("yiling"))
            || (matches!(b.id, "yizhou_map_audit" | "fu_pass_review" | "luofeng_mourning" | "jinyan_bridge_review" | "chengdu_land_decree")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "jingzhou_handover" | "ma_chao_letter")
                && flag("anxi_seal") && (unseen("jingzhou") || (flag("jingzhou_warning") && unseen("maicheng"))))
            || (b.id == "lukou_terms_record" && flag("anxi_seal") && (unseen("jingzhou") || (flag("jingzhou_warning") && unseen("maicheng"))))
            || (b.id == "xiangshui_partition" && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "hanzhong_civil_ledger" | "zuoci_court_record") && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "xiaoyao_ferry_witness" | "ruxu_rescue_roll") && flag("jiangdong_covenant") && unseen("yiling"))
            || (matches!(b.id, "xudu_fire_inquest" | "hanshui_retreat_roll") && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "wakou_civilian_route" | "dingjun_signal_register" | "hanzhong_route_review" | "hanzhong_king_memorial") && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "xiegu_ration_inquest" | "pangde_yujin_ledger") && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "jingxiang_border_warning" | "fancheng_flood_witness") && flag("anxi_seal")
                && (unseen("jingzhou") || (flag("jingzhou_warning") && unseen("maicheng"))))
            || (matches!(b.id, "blade_wound_watch" | "maicheng_relief_review") && flag("anxi_seal")
                && (unseen("jingzhou") || (flag("jingzhou_warning") && unseen("maicheng"))))
            || (matches!(b.id, "white_cloak_signal_audit" | "jingzhou_aftermath_courier")
                && flag("jiangdong_covenant") && unseen("yiling"))
            || (b.id == "luoyang_head_rites" && flag("guandu_grain") && unseen("liaodong"))
            || (b.id == "chengdu_mourning_register" && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "huatuo_prison_review" | "ye_court_succession_review" | "abdication_seal_audit")
                && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "jingzhou_mourning_counsel" | "shangyong_refusal_trial" | "chengdu_legitimacy_review")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "eastward_campaign_counsel" | "yiling_camp_map_warning")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "langzhong_heads_receipt" | "wu_wei_envoy_audit" | "yiling_defense_memorial")
                && flag("jiangdong_covenant") && unseen("yiling"))
            || (matches!(b.id, "yiling_fire_refuge_ledger" | "guangling_river_defense") && flag("jiangdong_covenant") && unseen("yiling"))
            || (matches!(b.id, "fishbelly_eightfold_report" | "yongan_regency_register" | "dengzhi_alliance_dispatch") && flag("redcliff_wind") && unseen("wuzhang"))
            || (b.id == "wei_three_fronts_review" && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "nanzhong_yongchang_memorial" | "nanzhong_first_release_ledger" | "lushui_ferry_care" | "xier_spring_civilian_water")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "nanzhong_vine_fire_account" | "nanzhong_local_governance" | "lushui_memorial_rites" | "chushi_memorial_supply" | "ziwu_route_risk_hearing" | "tianshui_false_dispatch_review")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "tianshui_recruitment_receipt" | "qiang_captive_return" | "jieting_water_order" | "xicheng_civilian_retreat")
                && flag("redcliff_wind") && (unseen("wuzhang") || (flag("longzhong_plan") && unseen("jiangwei"))))
            || (matches!(b.id, "wanglang_death_archive" | "xincheng_secret_dispatch")
                && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "masu_family_account" | "second_memorial_supply_review" | "chencang_siege_wounded_ledger" | "chencang_retreat_supply_audit")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "shiting_false_defection_audit" | "wu_han_enthronement_dispatch")
                && flag("jiangdong_covenant") && unseen("yiling"))
            || (matches!(b.id, "wudu_yinping_civil_register" | "qishan_rain_rations" | "jigu_defiance_casualties" | "lucheng_wheat_household" | "liyan_dispatch_evidence")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (b.id == "wei_caozhen_death_archive" && flag("guandu_grain") && unseen("liaodong"))
            || (b.id == "wu_nanzhong_border_archive" && flag("jiangdong_covenant") && unseen("yiling"))
            || (b.id == "wei_wu_tribute_review" && flag("guandu_grain") && unseen("liaodong"))
            // The 102–104 archival additions must not displace an old Wuzhang save.
            // This fixes the real Ubuntu chapter_051_053 regression on #157.
            || (matches!(b.id, "weibei_bridge_loss_ledger" | "wooden_ox_transport_audit" | "shangfang_fire_account" | "wuzhang_succession_register" | "wuzhang_withdrawal_evidence")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "weiyan_double_memorial_audit" | "qushan_water_rescue_dispatch")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "wei_palace_labor_register" | "liaodong_rain_siege_ledger" | "wei_regency_sickbed_register" | "gaoping_oath_court_review")
                && flag("guandu_grain") && unseen("liaodong"))
            || (b.id == "zongyu_wu_border_receipt" && flag("jiangdong_covenant") && unseen("yiling"));
        (if protect_legacy { 4 } else if is_hero && is_payoff { 0 } else if is_hero { 1 } else if is_payoff { 2 } else { 3 }, b.chapter)
    })
}


#[cfg(test)]
mod compatibility_tests {
    use super::*;
    #[test]
    fn optional_hanjin_never_forces_old_anxi_saves_past_jingzhou() {
        let flags = vec!["anxi_seal".into(), "xutian_guarded_emperor".into()];
        assert_eq!(next_beat(Hero::GuanYu, 2, &flags, &[]).map(|b| b.id), Some("jingzhou"));
        assert_eq!(next_beat(Hero::GuanYu, 2, &flags, &["jingzhou".into()]).map(|b| b.id), Some("hanjin_ferry"));
        assert_eq!(next_beat(Hero::GuanYu, 2, &["xutian_guarded_emperor".into()], &[]).map(|b| b.id), Some("hanjin_ferry"));
    }
}
