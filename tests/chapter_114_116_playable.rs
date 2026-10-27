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
    for (chapter, oldid, doc) in [
        (114, "2568077", include_str!("../docs/CHAPTER_114_READING.md")),
        (115, "2568078", include_str!("../docs/CHAPTER_115_READING.md")),
        (116, "2568079", include_str!("../docs/CHAPTER_116_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
    }
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
