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
}
