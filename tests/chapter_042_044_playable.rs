//! Chapters 042–044: source-pinned choices, atomic rejection, old saves and cross-act payoffs.
//! A direct Story fixture tests the actual Game::apply rules; natural-map multi-seed reachability
//! remains a separate unverified acceptance item.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn at(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 42044);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|flag| (*flag).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn has(game: &Game, flag: &str) -> bool {
    game.lore_flags.iter().any(|item| item == flag)
}
fn awards(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|item| item.as_str() == id).count()
}

#[test]
fn each_source_and_story_edge_is_explicit_and_chronological() {
    for (chapter, oldid, doc) in [
        (42, "2568005", include_str!("../docs/CHAPTER_042_READING.md")),
        (43, "2568006", include_str!("../docs/CHAPTER_043_READING.md")),
        (44, "2568008", include_str!("../docs/CHAPTER_044_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")) && doc.contains("单底本"), "{chapter}");
    }
    for (id, chapter, pre) in [
        ("hanjin_ferry", 42, "xutian_guarded_emperor"),
        ("chaisang_envoy", 43, "dangyang_refuge_account"),
        ("jiangdong_council", 44, "chaisang_envoy_record"),
        ("jiangdong_muster", 44, "jiangdong_hostage_record"),
    ] {
        let scene = beat(id).expect("playable scene exists");
        assert_eq!(scene.chapter, chapter);
        assert_eq!(scene.prerequisite, Some(pre));
        assert_eq!(scene.act, 2);
    }
    assert!(beat("hanjin_ferry").unwrap().premise.contains("拆桥"));
    assert!(beat("chaisang_envoy").unwrap().premise.contains("复疑"));
    assert!(beat("jiangdong_council").unwrap().premise.contains("尚未发生"));
    assert!(beat("jiangdong_muster").unwrap().premise.contains("后世"));
}

#[test]
fn guan_hanjin_records_ferry_and_pays_old_huarong_once() {
    assert_eq!(
        next_beat(Hero::GuanYu, 2, &["xutian_guarded_emperor".into()], &[])
            .map(|scene| scene.id),
        Some("hanjin_ferry")
    );
    let mut game = at(Hero::GuanYu, "hanjin_ferry", &["xutian_guarded_emperor"]);
    let snapshot = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), snapshot);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&game, "hanjin_ferry_record"));
    assert_eq!(awards(&game, "hanjin_huarong_debt"), 0);
    let mut game = Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut game, "huarong");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&game, "hanjin_huarong_debt"), 1);
    enter(&mut game, "huarong");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&game, "hanjin_huarong_debt"), 1);
}

#[test]
fn zhuge_chaisang_war_council_records_rival_views_and_pays_once() {
    assert_eq!(
        next_beat(Hero::ZhugeLiang, 2, &["dangyang_refuge_account".into()], &[])
            .map(|scene| scene.id),
        Some("chaisang_envoy")
    );
    let mut game = at(
        Hero::ZhugeLiang, "chaisang_envoy",
        &["dangyang_refuge_account", "redcliff_wind"]
    );
    let snapshot = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), snapshot);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&game, "chaisang_envoy_record"));
    assert_eq!(awards(&game, "chaisang_redcliff_debt"), 0);
    assert_eq!(
        next_beat(
            Hero::ZhugeLiang, 2, &game.lore_flags, &["chaisang_envoy".into()]
        ).map(|scene| scene.id),
        Some("jiangdong_council")
    );
    let mut game = Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut game, "jiangdong_council");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&game, "chaisang_redcliff_debt"), 1);
    enter(&mut game, "jiangdong_council");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&game, "chaisang_redcliff_debt"), 1);
}

#[test]
fn sun_muster_is_optional_and_does_not_rewrite_wedding_yiling() {
    let mut game = at(Hero::SunShangxiang, "jiangdong_muster", &["jiangdong_hostage_record"]);
    let snapshot = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 3 }).is_err());
    assert_eq!(game.to_json().unwrap(), snapshot);
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&game, "jiangdong_muster_record"));
    enter(&mut game, "jiangdong_wedding");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&game, "jiangdong_covenant"));
    enter(&mut game, "yiling");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&game, "yiling_ashes"));
    assert_eq!(awards(&game, "jiangdong_muster_debt"), 0);
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}

#[test]
fn sun_muster_choice_pays_once_after_original_cross_act_chain() {
    let mut game = at(Hero::SunShangxiang, "jiangdong_muster", &["jiangdong_hostage_record"]);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&game, "jiangdong_muster_record"));
    enter(&mut game, "jiangdong_wedding");
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "yiling");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&game, "jiangdong_muster_debt"), 1);
    let mut game = Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut game, "yiling");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&game, "jiangdong_muster_debt"), 1);
}

#[test]
fn legacy_save_routes_stay_playable_without_new_flags() {
    let mut guan = at(Hero::GuanYu, "huarong", &["baima_debt"]);
    guan.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&guan, "huarong_mercy"));
    assert_eq!(awards(&guan, "hanjin_huarong_debt"), 0);

    let mut zhuge = at(Hero::ZhugeLiang, "redcliff", &["longzhong_plan"]);
    zhuge.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&zhuge, "redcliff_wind"));
    assert_eq!(awards(&zhuge, "chaisang_redcliff_debt"), 0);

    let mut sun = at(Hero::SunShangxiang, "jiangdong_wedding", &[]);
    sun.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&sun, "jiangdong_covenant"));
    assert_eq!(awards(&sun, "jiangdong_muster_debt"), 0);
}
