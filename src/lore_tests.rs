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
}
