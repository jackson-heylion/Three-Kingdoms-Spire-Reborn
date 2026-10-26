//! Chapters 048–050: fixed-source causality, optional Cao accountability,
//! and unchanged Zhuge Red Cliffs / Guan Yu Huarong outcomes.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn at(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 4850);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|f| (*f).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn has(g: &Game, f: &str) -> bool { g.lore_flags.iter().any(|x| x == f) }
fn reward_count(g: &Game) -> usize {
    g.easter_unlocked.iter().filter(|x| x.as_str() == "redcliff_warning_ledger").count()
}

#[test]
fn fixed_sources_and_existing_scenes_keep_literary_order() {
    for (chapter, oldid, doc) in [
        (48, "2568011", include_str!("../docs/CHAPTER_048_READING.md")),
        (49, "2568012", include_str!("../docs/CHAPTER_049_READING.md")),
        (50, "2568013", include_str!("../docs/CHAPTER_050_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")) && doc.contains("单底本"), "{chapter}");
    }
    let new = beat("river_banquet_inquest").unwrap();
    assert_eq!((new.chapter, new.act, new.prerequisite), (48, 2, Some("chain_ship_risk_record")));
    assert!(new.premise.contains("程昱") && new.premise.contains("刘馥"));
    assert_eq!(beat("redcliff").unwrap().chapter, 49);
    assert!(beat("redcliff").unwrap().premise.contains("黄盖火船"));
    assert_eq!(beat("huarong").unwrap().chapter, 50);
    assert!(beat("huarong").unwrap().premise.contains("回营请罪"));
    assert!(include_str!("../docs/CHAPTERS_048_050_HISTORY_SPOTCHECK.md").contains("oldid=2085075"));
}

#[test]
fn cao_three_chapter_intelligence_chain_is_atomic_persistent_and_once_only() {
    let mut g = at(Hero::CaoCao, "jiangkou_inquest", &["guandu_grain"]);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "chain_ship_ledger");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&g), 0);
    assert_eq!(next_beat(Hero::CaoCao, 2, &g.lore_flags,
        &["liaodong".into(), "chain_ship_ledger".into()]).map(|b| b.id),
        Some("river_banquet_inquest"));
    enter(&mut g, "river_banquet_inquest");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 4 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "river_banquet_record"));
    assert_eq!(reward_count(&g), 1);
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(), saved);
    enter(&mut g, "river_banquet_inquest");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(reward_count(&g), 1);
}

#[test]
fn refusing_or_skipping_the_new_ledger_preserves_legacy_routes() {
    for choice in [2, 3] {
        let mut g = at(Hero::CaoCao, "river_banquet_inquest",
            &["guandu_grain", "jiangkou_inquest_record", "chain_ship_risk_record"]);
        g.apply(Action::Choose { index: choice }).unwrap();
        assert!(!has(&g, "river_banquet_record"));
        assert_eq!(reward_count(&g), 0);
        assert_eq!(next_beat(Hero::CaoCao, 2, &g.lore_flags, &g.lore_seen).map(|b| b.id), Some("liaodong"));
        enter(&mut g, "liaodong");
        g.apply(Action::Choose { index: 1 }).unwrap();
        assert!(has(&g, "liaodong_letter"));
        assert_eq!(reward_count(&g), 0);
    }
}

#[test]
fn old_redcliff_and_huarong_choices_still_work_without_new_flags() {
    let mut zhuge = at(Hero::ZhugeLiang, "redcliff", &["longzhong_plan"]);
    zhuge.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&zhuge, "redcliff_wind"));
    let mut guan = at(Hero::GuanYu, "huarong", &["baima_debt"]);
    guan.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&guan, "huarong_mercy"));
    let mut no_new = at(Hero::GuanYu, "huarong", &["baima_debt"]);
    no_new.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&no_new, "huarong_mercy"));
    assert_eq!(reward_count(&guan), 0);
    assert_eq!(reward_count(&zhuge), 0);
}
