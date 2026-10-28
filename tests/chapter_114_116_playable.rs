//! P1④ chapters 114–116: pinned evidence, choices, V2 and legacy priority.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 114116);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn pinned_readings_and_partial_history_are_explicit() {
    for (chapter, oldid, doc, body_range, consequence) in [
        (114, "2568077", include_str!("../docs/CHAPTER_114_READING.md"),
            "L119–164", "坑杀"),
        (115, "2568078", include_str!("../docs/CHAPTER_115_READING.md"),
            "L119–144", "刘琰"),
        (116, "2568079", include_str!("../docs/CHAPTER_116_READING.md"),
            "L119–147", "王含、蒋斌"),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
        assert!(doc.contains(body_range) && doc.contains(consequence),
            "{chapter}: the pinned full-body boundary or a material consequence was lost");
    }
    assert!(beat("tanshan_grain_fire_audit").unwrap().premise.contains("坑杀"));
    assert!(beat("tazhong_farming_dispatch").unwrap().premise.contains("三诏"));
    assert!(beat("yangping_gate_betrayal").unwrap().premise.contains("王含蒋斌"));
    let history = include_str!("../docs/CHAPTERS_114_116_HISTORY_SPOTCHECK.md");
    for id in ["8746774", "2583379", "2386518", "12068919"] {
        assert!(history.contains(id));
    }
    assert!(history.contains("裴注") && history.contains("0/120"));
    let rows: Vec<_> = include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(117).skip(114) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    for (n,id) in [(114,"wei_caomao_palace_evidence"),(114,"wei_wangjing_family_relief"),
        (114,"tanshan_grain_fire_audit"),(115,"taoyang_xiahouba_casualty"),
        (115,"tazhong_farming_dispatch"),(115,"wei_invasion_two_routes"),
        (116,"yangping_gate_betrayal"),(116,"dingjun_civilian_protection"),
        (116,"yinping_jiangge_defense"),(116,"wu_relief_warning_archive")] {
        assert_eq!(beat(id).unwrap().chapter,n);
    }
}

#[test]
fn palace_choice_is_atomic_exclusive_persistent_and_once_only() {
    let mut g=scene(Hero::CaoCao,"wei_caomao_palace_evidence",&["caofang_deposition_record"]);
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"wei_wangjing_family_relief");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"caofang_caomao_wangjing_debt"),1);
    let saved=g.to_json().unwrap();
    let mut restored=Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(),saved);
    enter(&mut restored,"wei_wangjing_family_relief");
    restored.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&restored,"caofang_caomao_wangjing_debt"),1);
    let mut refused=scene(Hero::CaoCao,"wei_caomao_palace_evidence",&["caofang_deposition_record"]);
    refused.apply(Action::Choose{index:2}).unwrap();
    assert!(!refused.lore_flags.iter().any(|s|s=="caomao_palace_evidence_record"));
    assert_eq!(count(&refused,"caofang_caomao_wangjing_debt"),0);
}

#[test]
fn accepted_shu_wei_wu_chains_pay_without_duplication() {
    let mut shu=scene(Hero::ZhugeLiang,"tanshan_grain_fire_audit",&["qishan_retreat_courier_record"]);
    for id in ["tanshan_grain_fire_audit","taoyang_xiahouba_casualty",
        "tazhong_farming_dispatch","yangping_gate_betrayal","yinping_jiangge_defense"] {
        enter(&mut shu,id);
        shu.apply(Action::Choose{index:1}).unwrap();
    }
    assert_eq!(count(&shu,"tanshan_taoyang_grain_debt"),1);
    assert_eq!(count(&shu,"qishan_tazhong_yangping_debt"),1);
    let mut wei=scene(Hero::CaoCao,"wei_invasion_two_routes",&["wei_wangjing_family_relief_record"]);
    wei.apply(Action::Choose{index:1}).unwrap();
    enter(&mut wei,"dingjun_civilian_protection");
    wei.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&wei,"wei_invasion_dingjun_civilian_debt"),1);
    let mut wu=scene(Hero::SunShangxiang,"wu_relief_warning_archive",&["sunxiu_court_review_record"]);
    wu.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&wu,"sunxiu_wu_relief_warning_debt"),1);
}


#[test]
fn qishan_payoff_needs_all_accepted_114_to_116_responsibilities() {
    let mut missing_tanshan = scene(Hero::ZhugeLiang, "yangping_gate_betrayal",
        &["qishan_retreat_courier_record", "taoyang_xiahouba_casualty_record",
          "tazhong_farming_dispatch_record"]);
    missing_tanshan.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&missing_tanshan, "qishan_tazhong_yangping_debt"), 0,
        "the 114 grain-loss audit cannot be silently skipped");

    let mut missing_taoyang = scene(Hero::ZhugeLiang, "yangping_gate_betrayal",
        &["qishan_retreat_courier_record", "tanshan_grain_fire_record",
          "tazhong_farming_dispatch_record"]);
    missing_taoyang.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&missing_taoyang, "qishan_tazhong_yangping_debt"), 0,
        "the 115 civilian and casualty account cannot be silently skipped");

    let mut complete = scene(Hero::ZhugeLiang, "yangping_gate_betrayal",
        &["qishan_retreat_courier_record", "tanshan_grain_fire_record",
          "taoyang_xiahouba_casualty_record", "tazhong_farming_dispatch_record"]);
    complete.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&complete, "qishan_tazhong_yangping_debt"), 1);
    let snapshot = complete.to_json().unwrap();
    let mut restored = Game::from_json(&snapshot).unwrap();
    enter(&mut restored, "yangping_gate_betrayal");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "qishan_tazhong_yangping_debt"), 1,
        "restoring a V2 save must not pay the debt twice");

    let mut refused = scene(Hero::ZhugeLiang, "yangping_gate_betrayal",
        &["qishan_retreat_courier_record", "tanshan_grain_fire_record",
          "taoyang_xiahouba_casualty_record", "tazhong_farming_dispatch_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&refused, "qishan_tazhong_yangping_debt"), 0);
    assert!(!refused.lore_flags.iter().any(|flag| flag == "yangping_gate_betrayal_record"));
}


#[test]
fn pinned_jiajing_114_116_and_hu_victim_choice_are_enforced() {
    let evidence = include_str!("../docs/CHAPTERS_114_116_HISTORY_SPOTCHECK.md");
    for marker in ["2475396", "2473250", "2568077", "2568078", "2568079",
        "曹髦之首", "二十萬", "三十萬", "胡氏", "未知傅僉性命如何", "非全回校勘"] {
        assert!(evidence.contains(marker), "missing fixed local comparison {marker}");
    }
    let beat = beat("tazhong_farming_dispatch").unwrap();
    assert_eq!(beat.chapter, 115);
    assert_eq!(beat.prerequisite, Some("taoyang_xiahouba_casualty_record"));
    assert!(beat.premise.contains("胡氏") && beat.premise.contains("不归责姜维"));
    assert!(beat.vow.contains("胡氏受害证词"));
    assert!(beat.bargain.contains("隐去胡氏受害"));
    assert_eq!(beat.reward_gold, 26);

    let mut accepted = scene(Hero::ZhugeLiang, beat.id, &["taoyang_xiahouba_casualty_record"]);
    let before = accepted.to_json().unwrap();
    assert!(accepted.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(accepted.to_json().unwrap(), before);
    accepted.apply(Action::Choose { index: 1 }).unwrap();
    assert!(accepted.lore_flags.iter().any(|f| f == beat.flag));
    let saved = accepted.to_json().unwrap();
    let restored = Game::from_json(&saved).unwrap();
    assert!(restored.lore_flags.iter().any(|f| f == beat.flag));
    let mut refused = scene(Hero::ZhugeLiang, beat.id, &["taoyang_xiahouba_casualty_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!refused.lore_flags.iter().any(|f| f == beat.flag));
    assert_eq!(count(&refused, "qishan_tazhong_yangping_debt"), 0);
}

#[test]
fn old_main_story_wins_and_new_archives_remain_selectable() {
    assert_eq!(next_beat(Hero::ZhugeLiang,2,&["redcliff_wind".into(),"qishan_retreat_courier_record".into()],&[]).map(|b|b.id),Some("wuzhang"));
    assert_eq!(next_beat(Hero::CaoCao,2,&["guandu_grain".into(),"caofang_deposition_record".into()],&[]).map(|b|b.id),Some("liaodong"));
    assert_eq!(next_beat(Hero::SunShangxiang,2,&["jiangdong_covenant".into(),"sunxiu_court_review_record".into()],&[]).map(|b|b.id),Some("yiling"));
    for (hero,id,prerequisite) in [
        (Hero::ZhugeLiang,"tanshan_grain_fire_audit","qishan_retreat_courier_record"),
        (Hero::CaoCao,"wei_caomao_palace_evidence","caofang_deposition_record"),
        (Hero::SunShangxiang,"wu_relief_warning_archive","sunxiu_court_review_record"),
    ] {
        let seen:Vec<String>=BEATS.iter().filter(|b|b.id!=id).map(|b|b.id.into()).collect();
        assert_eq!(next_beat(hero,2,&[prerequisite.into()],&seen).map(|b|b.id),Some(id));
    }
    for seed in 114116..114124 {
        let saved=Game::new(Hero::ZhugeLiang,seed).to_json().unwrap();
        assert!(Game::from_json(&saved).is_ok());
    }
}
