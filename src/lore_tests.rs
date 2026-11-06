// Unit tests split from lore.rs to keep narrative integration module bounded.
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
    #[test]
    fn chapter_068_chen_wu_killed_by_pang_de_who_survives_until_074() {
        // In chapter 068 Pang De kills Chen Wu; he himself dies after
        // refusing Guan Yu's surrender offer in chapter 074.
        let ruxu = beat("ruxu_rescue_roll").expect("chapter 068 scene");
        let fancheng = beat("pangde_yujin_ledger").expect("chapter 074 scene");
        assert_eq!(ruxu.chapter, 68);
        assert_eq!(fancheng.chapter, 74);
        assert!(ruxu.premise.contains("陈武与庞德交战"));
        assert!(ruxu.character.contains("庞德"));
        assert!(ruxu.premise.contains("遂为庞德所杀"));
        assert!(!ruxu.premise.contains("庞德战死"));
        assert!(fancheng.premise.contains("庞德拒降被斩"));
    }
    #[test]
    fn chapters_097_102_preserve_one_envoy_and_separate_grain_campaigns() {
        let memorial = beat("second_memorial_supply_review").expect("chapter 097 memorial");
        let chencang = beat("chencang_siege_wounded_ledger").expect("chapter 097 siege");
        let retreat = beat("chencang_retreat_supply_audit").expect("chapter 098 withdrawal");
        let envoy = beat("wu_han_enthronement_dispatch").expect("chapter 098 envoy");
        let liyan = beat("liyan_dispatch_evidence").expect("chapter 101 dispatch");
        let ox = beat("wooden_ox_transport_audit").expect("chapter 102 transport");
        assert_eq!((memorial.chapter, chencang.chapter, retreat.chapter), (97, 97, 98));
        assert_eq!(retreat.prerequisite, Some(chencang.flag));
        assert!(retreat.premise.contains("粮尽退还"));
        assert!(retreat.premise.contains("王双"));
        assert_eq!(envoy.chapter, 98);
        assert!(envoy.premise.contains("杨震"));
        assert!(envoy.premise.contains("陈震"));
        assert!(envoy.premise.contains("不把使节异文判为两次出使"));
        assert_eq!(liyan.chapter, 101);
        assert!(liyan.premise.contains("李严"));
        assert!(liyan.premise.contains("前后书疏"));
        assert_eq!(ox.chapter, 102);
        assert!(ox.premise.contains("二十里"));
        assert!(ox.premise.contains("三十里"));
        assert!(ox.premise.contains("不能增生战斗或当史实参数"));
    }
    #[test]
    fn maliang_yiling_map_is_novel_story_not_verified_history() {
        let map = beat("yiling_camp_map_warning").expect("chapter 083 archive");
        let report = beat("fishbelly_eightfold_report").expect("chapter 084 archive");
        assert_eq!((map.chapter, report.chapter), (83, 84));
        assert_eq!(map.flag, "yiling_camp_map_record");
        assert_eq!(report.prerequisite, Some(map.flag));
        assert!(map.premise.contains("小说写马良送营图到成都"));
        assert!(map.premise.contains("《马良传》记其赴武陵"));
        assert!(report.premise.contains("不可把小说的预警信使路线当作正史"));
    }

    #[test]
    fn chapter_080_cao_empress_separates_two_novels_from_hou_han_shu() {
        let succession = beat("ye_court_succession_review").expect("chapter 079 succession");
        let abdication = beat("abdication_seal_audit").expect("chapter 080 abdication");
        assert_eq!((succession.chapter, abdication.chapter), (79, 80));
        assert_eq!(abdication.prerequisite, Some(succession.flag));
        assert!(abdication.premise.contains("嘉靖159写曹皇后为曹丕辩护"));
        assert!(abdication.premise.contains("毛评080却写曹后斥曹洪曹休"));
        assert!(abdication.premise.contains("《後漢書》卷10下《獻穆曹皇后》"));
        assert!(abdication.premise.contains("拒交并以璽抵轩下"));
        assert!(abdication.premise.contains("不能证明毛评的逼宫对白逐字属实"));
    }
    #[test]
    fn chapter_097_chencang_envoy_is_separated_from_wei_lue_testimony() {
        let siege = beat("chencang_siege_wounded_ledger").expect("chapter 097");
        let retreat = beat("chencang_retreat_supply_audit").expect("chapter 098");
        assert_eq!((siege.chapter, retreat.chapter), (97, 98));
        assert_eq!(retreat.prerequisite, Some(siege.flag));
        assert!(siege.character.contains("鄞祥"));
        assert!(siege.premise.contains("毛评097与嘉靖193固定电子转录"));
        assert!(siege.premise.contains("裴注引《魏略》写同乡「靳詳」"));
        assert!(siege.premise.contains("在城外遥说、郝昭于城楼回应"));
        assert!(siege.premise.contains("不得将小说会面细节冒充陈寿正文"));
    }

    #[test]
    fn southern_campaign_archives_follow_eight_explicit_accountability_handoffs() {
        // Nine independently chosen records are longer than a seven-node act.
        // This verifies the literary chain; a natural Story entry still needs
        // its own seed-backed CLI proof and must not be inferred here.
        let ids = [
            "fishbelly_eightfold_report",
            "yongan_regency_register",
            "dengzhi_alliance_dispatch",
            "nanzhong_yongchang_memorial",
            "nanzhong_first_release_ledger",
            "lushui_ferry_care",
            "xier_spring_civilian_water",
            "nanzhong_vine_fire_account",
            "nanzhong_local_governance",
        ];
        for pair in ids.windows(2) {
            let previous = beat(pair[0]).expect("previous archive must exist");
            let next = beat(pair[1]).expect("next archive must exist");
            assert_eq!(previous.act, 2);
            assert_eq!(next.act, 2);
            assert_eq!(previous.hero, Some(Hero::ZhugeLiang));
            assert_eq!(next.hero, Some(Hero::ZhugeLiang));
            assert_eq!(next.prerequisite, Some(previous.flag), "{}", next.id);
            assert!(next.chapter >= previous.chapter);
        }
    }
    #[test]
    fn yongan_to_nanzhong_hidden_debts_require_every_record_and_pay_once() {
        // These are flag-injected unit tests, NOT evidence of natural map/CLI reachability.
        let cases: [(&str, Hero, &[&str], i32, i32); 3] = [
            (
                "yongan_nanzhong_civilian_debt",
                Hero::ZhugeLiang,
                &["yongan_regency_record", "dengzhi_alliance_record",
                  "nanzhong_yongchang_record", "nanzhong_first_release_record"],
                18, 7,
            ),
            (
                "redcliff_nanzhong_water_debt",
                Hero::ZhugeLiang,
                &["redcliff_wind", "dengzhi_alliance_record",
                  "nanzhong_yongchang_record", "nanzhong_first_release_record",
                  "lushui_ferry_record", "xier_spring_record"],
                29, 13,
            ),
            (
                "nanzhong_mourning_debt",
                Hero::ZhugeLiang,
                &["xier_spring_record", "nanzhong_vine_fire_record",
                  "nanzhong_local_rule_record", "lushui_memorial_record"],
                27, 12,
            ),
        ];
        for (id, hero, required, gold, heal) in cases {
            let egg = EASTER_EGGS.iter().find(|egg| egg.id == id).expect("hidden debt");
            assert_eq!(egg.hero, Some(hero), "{id}");
            assert_eq!(egg.flags, required, "{id}");
            assert_eq!((egg.reward_gold, egg.reward_heal), (gold, heal), "{id}");
            for missing in required {
                let partial: Vec<String> = required.iter()
                    .filter(|flag| *flag != missing)
                    .map(|flag| (*flag).to_owned()).collect();
                assert!(
                    !newly_completed_eggs(hero, &partial, &[])
                        .iter().any(|unlocked| unlocked.id == id),
                    "{id} incorrectly pays without {missing}"
                );
            }
            let complete: Vec<String> = required.iter().map(|flag| (*flag).to_owned()).collect();
            assert_eq!(
                newly_completed_eggs(hero, &complete, &[])
                    .iter().filter(|unlocked| unlocked.id == id).count(),
                1,
                "{id} must unlock exactly once with all flags"
            );
            assert!(
                !newly_completed_eggs(hero, &complete, &[id.to_owned()])
                    .iter().any(|unlocked| unlocked.id == id),
                "{id} must not pay twice after save restoration"
            );
            assert!(
                !newly_completed_eggs(Hero::GuanYu, &complete, &[])
                    .iter().any(|unlocked| unlocked.id == id),
                "{id} must not pay to another hero"
            );
        }
    }

}
