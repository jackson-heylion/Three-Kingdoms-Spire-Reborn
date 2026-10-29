//! P1④ chapters 117–119: pinned body evidence, accepted debts, V2 and legacy priorities.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game=Game::new(hero,117119);
    game.stage=Stage::Story;
    game.lore_current=Some(id.into());
    game.lore_flags=flags.iter().map(|s|(*s).into()).collect();
    game
}
fn enter(game:&mut Game,id:&str) {game.stage=Stage::Story;game.lore_current=Some(id.into());}
fn count(game:&Game,id:&str)->usize {game.easter_unlocked.iter().filter(|s|s.as_str()==id).count()}

#[test]
fn pinned_readings_are_not_mistaken_for_complete_variant_collation() {
    for (n,oldid,span,doc) in [(117,"2568080","L119–162",include_str!("../docs/CHAPTER_117_READING.md")),(118,"2568082","L119–171",include_str!("../docs/CHAPTER_118_READING.md")),(119,"2568081","L119–179",include_str!("../docs/CHAPTER_119_READING.md"))] {
        assert!(doc.contains(&format!("oldid={oldid}")) && doc.contains(&format!("正文 {span}")) && doc.contains("回末") && doc.contains("单底本全文通读"),"{n}");
    }
    let variants=include_str!("../docs/CHAPTERS_117_119_VARIANT_SPOTCHECK.md");
    assert!(variants.contains("oldid=2473250") && variants.contains("2568080")
        && variants.contains("2568082") && variants.contains("2568081")
        && variants.contains("中箭落馬而死") && variants.contains("荊州牧")
        && variants.contains("益州牧") && variants.contains("二千斤")
        && variants.contains("三千斤") && variants.contains("非全回异本校勘"));
    let history=include_str!("../docs/CHAPTERS_117_119_HISTORY_SPOTCHECK.md");
    assert!(history.contains("裴注") && history.contains("0/120") && history.contains("姜维自刎"));
    let rows:Vec<_>=include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(rows.len(),121);
    for (chapter,row) in rows.iter().enumerate().take(120).skip(117) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    for (n,id) in [(117,"jiange_yinping_detour_warning"),(117,"mianzhu_family_casualties"),(117,"wei_yinping_march_log"),(117,"wu_mianzhu_relief_dispatch"),(118,"chengdu_surrender_household"),(118,"wei_chengdu_occupation_receipt"),(118,"wu_relief_arrival_dispute"),(119,"zhonghui_false_edict_inquest"),(119,"jiangwei_last_courier_roll"),(119,"wei_jin_abdication_court"),(119,"wu_post_shu_frontier_register")] {assert_eq!(beat(id).unwrap().chapter,n);}
}

#[test]
fn invalid_action_is_atomic_refusal_is_exclusive_and_v2_reward_once() {
    let mut game=scene(Hero::ZhugeLiang,"jiange_yinping_detour_warning",&["yinping_jiangge_defense_record"]);
    let before=game.to_json().unwrap();
    assert!(game.apply(Action::Choose{index:99}).is_err());
    assert_eq!(game.to_json().unwrap(),before);
    for id in ["jiange_yinping_detour_warning","mianzhu_family_casualties","chengdu_surrender_household","jiangwei_last_courier_roll"] {
        enter(&mut game,id);
        game.apply(Action::Choose{index:1}).unwrap();
    }
    assert_eq!(count(&game,"jiange_mianzhu_chengdu_debt"),1);
    assert_eq!(count(&game,"chengdu_jiangwei_aftercare_debt"),1);
    let saved=game.to_json().unwrap();
    let mut restored=Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(),saved);
    enter(&mut restored,"jiangwei_last_courier_roll");
    restored.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&restored,"chengdu_jiangwei_aftercare_debt"),1);
    let mut refused=scene(Hero::ZhugeLiang,"mianzhu_family_casualties",&["jiange_detour_warning_record"]);
    refused.apply(Action::Choose{index:2}).unwrap();
    assert!(!refused.lore_flags.iter().any(|s|s=="mianzhu_family_casualties_record"));
    assert_eq!(count(&refused,"jiange_mianzhu_chengdu_debt"),0);
}

#[test]
fn wei_and_wu_successor_debts_are_one_time() {
    let mut wei=scene(Hero::CaoCao,"wei_yinping_march_log",&["dingjun_civilian_protection_record"]);
    for id in ["wei_yinping_march_log","wei_chengdu_occupation_receipt","zhonghui_false_edict_inquest","wei_jin_abdication_court"] {enter(&mut wei,id);wei.apply(Action::Choose{index:1}).unwrap();}
    assert_eq!(count(&wei,"yinping_zhonghui_court_debt"),1);
    let mut wu=scene(Hero::SunShangxiang,"wu_mianzhu_relief_dispatch",&["wu_relief_warning_record"]);
    for id in ["wu_mianzhu_relief_dispatch","wu_relief_arrival_dispute"] {enter(&mut wu,id);wu.apply(Action::Choose{index:1}).unwrap();}
    assert_eq!(count(&wu,"wu_mianzhu_relief_debt"),1);
}

#[test]
fn legacy_entries_remain_prioritized_and_new_archives_can_be_selected() {
    assert_eq!(next_beat(Hero::ZhugeLiang,2,&["redcliff_wind".into(),"yinping_jiangge_defense_record".into()],&[]).map(|b|b.id),Some("wuzhang"));
    assert_eq!(next_beat(Hero::CaoCao,2,&["guandu_grain".into(),"dingjun_civilian_protection_record".into()],&[]).map(|b|b.id),Some("liaodong"));
    assert_eq!(next_beat(Hero::SunShangxiang,2,&["jiangdong_covenant".into(),"wu_relief_warning_record".into()],&[]).map(|b|b.id),Some("yiling"));
    for (hero,id,required) in [(Hero::ZhugeLiang,"jiange_yinping_detour_warning","yinping_jiangge_defense_record"),(Hero::CaoCao,"wei_yinping_march_log","dingjun_civilian_protection_record"),(Hero::SunShangxiang,"wu_mianzhu_relief_dispatch","wu_relief_warning_record")] {
        let seen:Vec<String>=BEATS.iter().filter(|b|b.id!=id).map(|b|b.id.into()).collect();
        assert_eq!(next_beat(hero,2,&[required.into()],&seen).map(|b|b.id),Some(id));
    }
    for seed in 117119..117127 {let saved=Game::new(Hero::ZhugeLiang,seed).to_json().unwrap();assert!(Game::from_json(&saved).is_ok());}
}

#[test]
fn successor_rewards_require_every_accepted_link_and_are_once_only() {
    use three_kingdoms_spire_reborn::lore::newly_completed_eggs;
    let cases: &[(Hero, &str, &[&str])] = &[
        (Hero::ZhugeLiang, "jiange_mianzhu_chengdu_debt",
            &["yinping_jiangge_defense_record", "jiange_detour_warning_record",
              "mianzhu_family_casualties_record", "chengdu_surrender_household_record"]),
        (Hero::ZhugeLiang, "chengdu_jiangwei_aftercare_debt",
            &["chengdu_surrender_household_record", "jiangwei_last_courier_record"]),
        (Hero::CaoCao, "yinping_zhonghui_court_debt",
            &["wei_yinping_march_record", "wei_chengdu_occupation_record",
              "zhonghui_false_edict_record", "wei_jin_abdication_record"]),
        (Hero::SunShangxiang, "wu_mianzhu_relief_debt",
            &["wu_relief_warning_record", "wu_mianzhu_relief_dispatch_record",
              "wu_relief_arrival_dispute_record"]),
        (Hero::SunShangxiang, "wu_after_shu_frontier_debt",
            &["wu_relief_warning_record", "wu_mianzhu_relief_dispatch_record",
              "wu_relief_arrival_dispute_record", "wu_post_shu_frontier_record"]),
    ];
    for (hero, egg, required) in cases {
        let complete: Vec<String> = required.iter().map(|s| (*s).into()).collect();
        assert_eq!(newly_completed_eggs(*hero, &complete, &[])
            .iter().filter(|item| item.id == *egg).count(), 1, "{egg}: full chain");
        for missing in 0..required.len() {
            let incomplete: Vec<String> = required.iter().enumerate()
                .filter(|(i, _)| *i != missing)
                .map(|(_, flag)| (*flag).into()).collect();
            assert!(!newly_completed_eggs(*hero, &incomplete, &[])
                .iter().any(|item| item.id == *egg),
                "{egg}: missing {}", required[missing]);
        }
        assert!(!newly_completed_eggs(*hero, &complete, &[(*egg).into()])
            .iter().any(|item| item.id == *egg), "{egg}: duplicate payout");
    }
}

#[test]
fn wu_post_shu_frontier_is_a_real_exclusive_v2_choice_with_once_only_payoff() {
    let inherited = &["wu_relief_warning_record", "wu_mianzhu_relief_dispatch_record",
        "wu_relief_arrival_dispute_record"];
    let mut accepted = scene(Hero::SunShangxiang, "wu_post_shu_frontier_register", inherited);
    let snapshot = accepted.to_json().unwrap();
    assert!(accepted.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(accepted.to_json().unwrap(), snapshot, "invalid choice must not mutate RNG or revision");
    accepted = Game::from_json(&snapshot).unwrap();
    accepted.apply(Action::Choose { index: 1 }).unwrap();
    assert!(accepted.lore_flags.iter().any(|flag| flag == "wu_post_shu_frontier_record"));
    assert_eq!(count(&accepted, "wu_after_shu_frontier_debt"), 1);
    let persisted = accepted.to_json().unwrap();
    let mut restored = Game::from_json(&persisted).unwrap();
    assert_eq!(restored.to_json().unwrap(), persisted);
    enter(&mut restored, "wu_post_shu_frontier_register");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "wu_after_shu_frontier_debt"), 1, "no duplicate hidden reward");

    let mut refused = scene(Hero::SunShangxiang, "wu_post_shu_frontier_register", inherited);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!refused.lore_flags.iter().any(|flag| flag == "wu_post_shu_frontier_record"));
    assert_eq!(count(&refused, "wu_after_shu_frontier_debt"), 0);
    assert!(refused.lore_seen.iter().any(|id| id == "wu_post_shu_frontier_register"));
}
