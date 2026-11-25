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
            || (b.id == "zongyu_wu_border_receipt" && flag("jiangdong_covenant") && unseen("yiling"))
            || (matches!(b.id, "dangu_casualty_roll" | "qiaozhou_war_counsel" | "qishan_retreat_courier") && flag("redcliff_wind") && unseen("wuzhang"))
            || (matches!(b.id, "wei_huainan_summons_audit" | "shouchun_city_captives") && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "zhugedan_wu_hostage_receipt" | "shouchun_wu_relief" | "sunxiu_court_review") && flag("jiangdong_covenant") && unseen("yiling"))
            // Optional late archives must not displace existing V2 main-story entries.
            || (b.chapter >= 114 && ((flag("redcliff_wind") && unseen("wuzhang"))
                || (flag("guandu_grain") && unseen("liaodong"))
                || (flag("jiangdong_covenant") && unseen("yiling"))));
        (if protect_legacy { 4 } else if is_hero && is_payoff { 0 } else if is_hero { 1 } else if is_payoff { 2 } else { 3 }, b.chapter)
    })
}

fn tactical_anchor_rank(id: &str) -> Option<u8> {
    match id {
        "taoyuan" | "sevenstar" | "sun_father_letter" | "longzhong" | "redhare" | "empty_city" => Some(0),
        "baima" | "xudu" | "jiangdong_wedding" | "borrow_arrows" | "pavilion" | "south_wilds" => Some(1),
        "anxi" | "guandu" | "river" | "redcliff" | "halberd" | "wooden_ox" => Some(2),
        "wenjiu" | "huarong" | "liaodong" | "yiling_bridge" | "jiangwei" | "whitegate" => Some(3),
        "jingzhou" | "yiling" | "wuzhang" => Some(4),
        "maicheng" | "jieting_debt" => Some(5),
        "fancheng_flood_witness" => Some(6),
        "ma_chao_letter" => Some(7),
        "lukou_terms_record" => Some(8),
        "jingxiang_border_warning" => Some(9),
        _ => None,
    }
}

fn anchor_max_act(id: &str) -> u8 {
    match id {
        "taoyuan" | "sevenstar" | "sun_father_letter" | "longzhong" | "redhare" => 0,
        "baima" | "huarong" | "guandu" | "pavilion" => 1,
        "xudu" | "wenjiu" | "anxi" | "empty_city" | "jiangdong_wedding" => 2,
        "halberd" | "river" | "borrow_arrows" | "south_wilds" | "fancheng_flood_witness"
        | "ma_chao_letter" | "lukou_terms_record" | "jingxiang_border_warning" => 3,
        "jingzhou" | "maicheng" | "liaodong" | "yiling" | "yiling_bridge" | "wuzhang"
        | "jiangwei" | "jieting_debt" | "whitegate" | "wooden_ox" => 4,
        _ => 4,
    }
}

/// Deterministic scene ordering for expanded five-act campaigns (acts 0..=4).
/// Prioritizes core cross-act tactical anchors in acts I–III (recovering upcoming
/// skipped hero anchors and shared tactical anchors ahead of generic filler in acts
/// II–V) without modifying `BEATS` or legacy 21-floor `next_beat` semantics.
pub fn next_expanded_beat(hero: Hero, act: u8, flags: &[String], seen: &[String]) -> Option<&'static Beat> {
    let eligible = |b: &&Beat| {
        !seen.iter().any(|id| id == b.id)
            && b.prerequisite.is_none_or(|key| flags.iter().any(|flag| flag == key))
    };
    if act <= 2 {
        if let Some(anchor) = BEATS.iter()
            .filter(|b| eligible(b) && b.act <= act && tactical_anchor_rank(b.id).is_some()
                && b.hero.is_none_or(|required| required == hero)
                && (b.hero.is_some() || b.act == act))
            .min_by_key(|b| {
                let upcoming = anchor_max_act(b.id) >= act;
                let tier = if b.hero == Some(hero) && b.act == act { 0 }
                    else if b.hero == Some(hero) && upcoming { 1 }
                    else if b.hero == Some(hero) { 2 }
                    else { 3 };
                (tier, act.saturating_sub(b.act), tactical_anchor_rank(b.id).unwrap_or(255), b.chapter)
            })
        {
            return Some(anchor);
        }
        return next_beat(hero, act, flags, seen);
    }
    if let Some(anchor) = BEATS.iter()
        .filter(|b| eligible(b) && b.act <= 2 && b.hero == Some(hero)
            && tactical_anchor_rank(b.id).is_some())
        .min_by_key(|b| (
            u8::from(anchor_max_act(b.id) < act),
            2_u8.saturating_sub(b.act),
            tactical_anchor_rank(b.id).unwrap_or(255),
            b.chapter,
        ))
    {
        return Some(anchor);
    }
    if let Some(shared_anchor) = BEATS.iter()
        .filter(|b| eligible(b) && b.act <= 2 && b.hero.is_none()
            && tactical_anchor_rank(b.id).is_some() && anchor_max_act(b.id) >= act)
        .min_by_key(|b| (
            2_u8.saturating_sub(b.act),
            tactical_anchor_rank(b.id).unwrap_or(255),
            u8::from(b.prerequisite.is_none()),
            b.chapter,
        ))
    {
        return Some(shared_anchor);
    }
    for candidate_act in (0..=2).rev() {
        if let Some(beat) = BEATS.iter()
            .filter(|b| eligible(b) && b.act == candidate_act && b.hero == Some(hero))
            .min_by_key(|b| (
                u8::from(tactical_anchor_rank(b.id).is_none()),
                tactical_anchor_rank(b.id).unwrap_or(255),
                u8::from(b.prerequisite.is_none()),
                b.chapter,
            ))
        {
            return Some(beat);
        }
    }
    BEATS.iter()
        .filter(|b| eligible(b) && b.hero.is_none())
        .min_by_key(|b| (
            u8::from(tactical_anchor_rank(b.id).is_none()),
            2_u8.saturating_sub(b.act),
            tactical_anchor_rank(b.id).unwrap_or(255),
            u8::from(b.prerequisite.is_none()),
            b.chapter,
        ))
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
