//! Chapters 039–041: fixed sources, atomic refusal, V2 persistence, legacy routes and reward idempotence.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};
fn at(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 39041);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|f| (*f).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn has(g: &Game, f: &str) -> bool { g.lore_flags.iter().any(|x| x == f) }
fn awards(g: &Game, id: &str) -> usize { g.easter_unlocked.iter().filter(|x| x.as_str() == id).count() }
#[test]
fn fixed_sources_and_scene_mapping_are_auditable() {
    for (ch, oldid, doc) in [
        (39,"2567999",include_str!("../docs/CHAPTER_039_READING.md")),
        (40,"2568003",include_str!("../docs/CHAPTER_040_READING.md")),
        (41,"2568004",include_str!("../docs/CHAPTER_041_READING.md")),
    ] {
        assert!(doc.contains(oldid) && doc.contains("单底本"), "chapter {ch}");
    }
    for (id, chapter, pre) in [
        ("jiangxia_pardon",39,"jiangdong_hostage_record"),
        ("bowang_orders",39,"longzhong_conditions_record"),
        ("xinye_evacuation",40,"bowang_orders_record"),
        ("dangyang_ledger",41,"xinye_civilians"),
    ] {
        assert_eq!(beat(id).unwrap().chapter,chapter);
        assert_eq!(beat(id).unwrap().prerequisite,Some(pre));
    }
    assert!(beat("dangyang_ledger").unwrap().premise.contains("事后"));
    assert!(beat("xinye_evacuation").unwrap().premise.contains("曹仁未死"));
}
#[test]
fn zhuge_fixed_seed_chain_is_atomic_persistent_and_once_only() {
    let flags=vec!["longzhong_plan".into(),"longzhong_conditions_record".into()];
    assert_eq!(next_beat(Hero::ZhugeLiang,1,&flags,&[]).map(|b|b.id),Some("bowang_orders"));
    let mut g=at(Hero::ZhugeLiang,"bowang_orders",&["longzhong_plan","longzhong_conditions_record"]);
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"bowang_orders_record"));
    enter(&mut g,"xinye_evacuation");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"xinye_civilians"));
    let saved=g.to_json().unwrap();
    let mut g=Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(),saved);
    enter(&mut g,"redcliff");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(awards(&g,"xinye_dangyang_debt"),0);
    enter(&mut g,"dangyang_ledger");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(awards(&g,"xinye_dangyang_debt"),1);
    enter(&mut g,"dangyang_ledger");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(awards(&g,"xinye_dangyang_debt"),1);
}
#[test]
fn refusing_bowang_preserves_legacy_scenes_and_saves() {
    let mut g=at(Hero::ZhugeLiang,"bowang_orders",&["longzhong_plan","longzhong_conditions_record"]);
    g.apply(Action::Choose{index:2}).unwrap();
    assert!(!has(&g,"bowang_orders_record"));
    assert_eq!(next_beat(Hero::ZhugeLiang,1,&g.lore_flags,&g.lore_seen).map(|b|b.id),Some("borrow_arrows"));
    enter(&mut g,"redcliff");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"redcliff_wind"));
    assert_eq!(awards(&g,"xinye_dangyang_debt"),0);
    let mut old=at(Hero::ZhugeLiang,"changban",&[]);
    old.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&old,"changban_pledge"));
}
#[test]
fn sun_jiangxia_mercy_is_optional_and_once_only() {
    let mut g=at(Hero::SunShangxiang,"jiangxia_pardon",&["jiangdong_hostage_record"]);
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:0}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    let mut g=Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g,"jiangdong_wedding");
    g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"yiling");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(awards(&g,"jiangxia_mercy_debt"),1);
    enter(&mut g,"yiling");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(awards(&g,"jiangxia_mercy_debt"),1);
}
#[test]
fn sun_refusal_keeps_legacy_wedding_and_yiling() {
    let mut g=at(Hero::SunShangxiang,"jiangxia_pardon",&["jiangdong_hostage_record"]);
    g.apply(Action::Choose{index:2}).unwrap();
    assert!(!has(&g,"jiangxia_pardon_record"));
    enter(&mut g,"jiangdong_wedding");
    g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"yiling");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"yiling_ashes"));
    assert_eq!(awards(&g,"jiangxia_mercy_debt"),0);
}
