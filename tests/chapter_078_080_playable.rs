//! Chapters 078–080: evidence, original-vs-history boundaries, optional choices and old saves.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 7880);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|f| (*f).to_owned()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_sources_and_scene_prerequisites_are_explicit() {
    for (n, rev, text) in [
        (78, "2568040", include_str!("../docs/CHAPTER_078_READING.md")),
        (79, "2568043", include_str!("../docs/CHAPTER_079_READING.md")),
        (80, "7909161", include_str!("../docs/CHAPTER_080_READING.md")),
    ] {
        assert!(text.contains(&format!("oldid={rev}")) && text.contains("单底本"), "{n}");
        assert!(text.contains("回末"), "{n}");
    }
    let history = include_str!("../docs/CHAPTERS_078_080_HISTORY_SPOTCHECK.md");
    assert!(history.contains("卷29") && history.contains("卷40") && history.contains("卷02"));
    assert!(history.contains("0/120") && history.contains("山阳公"));
    for (id, chapter, prior) in [
        ("huatuo_prison_review", 78, "luoyang_head_record"),
        ("jingzhou_mourning_counsel", 78, "chengdu_mourning_record"),
        ("ye_court_succession_review", 79, "huatuo_prison_record"),
        ("shangyong_refusal_trial", 79, "jingzhou_mourning_record"),
        ("abdication_seal_audit", 80, "ye_succession_record"),
        ("chengdu_legitimacy_review", 80, "shangyong_trial_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(prior));
        assert_eq!(b.act, 2);
    }
    assert!(beat("abdication_seal_audit").unwrap().premise.contains("曹操死后"));
    assert!(beat("chengdu_legitimacy_review").unwrap().premise.contains("献帝未死"));
}

#[test]
fn cao_posthumous_records_are_atomic_exclusive_once_only_and_restorable() {
    let flags = ["guandu_grain", "luoyang_head_record"];
    let mut g = scene(Hero::CaoCao, "huatuo_prison_review", &flags);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "ye_court_succession_review");
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "abdication_seal_audit");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "guandu_succession_abdication_debt"), 1);
    let snapshot = g.to_json().unwrap();
    let mut restored = Game::from_json(&snapshot).unwrap();
    assert_eq!(restored.to_json().unwrap(), snapshot);
    enter(&mut restored, "abdication_seal_audit");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "guandu_succession_abdication_debt"), 1);
    let mut decline = scene(Hero::CaoCao, "huatuo_prison_review", &flags);
    decline.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut decline, "ye_court_succession_review");
    decline.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut decline, "abdication_seal_audit");
    decline.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&decline, "guandu_succession_abdication_debt"), 0);
}

#[test]
fn zhuge_mourning_shangyong_and_legitimacy_need_full_chain() {
    let mut g = scene(Hero::ZhugeLiang, "jingzhou_mourning_counsel", &[
        "redcliff_wind", "chengdu_mourning_record",
    ]);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "shangyong_refusal_trial");
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "chengdu_legitimacy_review");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "redcliff_chengdu_legitimacy_debt"), 1);
    let mut decline = scene(Hero::ZhugeLiang, "jingzhou_mourning_counsel", &[
        "redcliff_wind", "chengdu_mourning_record",
    ]);
    decline.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut decline, "shangyong_refusal_trial");
    decline.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut decline, "chengdu_legitimacy_review");
    decline.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&decline, "redcliff_chengdu_legitimacy_debt"), 0);
}

#[test]
fn legacy_act_two_routes_keep_priority_but_new_scenes_remain_optional() {
    for (hero, flags, old, new) in [
        (Hero::CaoCao, vec!["guandu_grain", "luoyang_head_record"], "liaodong", "huatuo_prison_review"),
        (Hero::ZhugeLiang, vec!["redcliff_wind", "chengdu_mourning_record"], "wuzhang", "jingzhou_mourning_counsel"),
    ] {
        let flags: Vec<String> = flags.into_iter().map(str::to_owned).collect();
        assert_eq!(next_beat(hero, 2, &flags, &[]).map(|b| b.id), Some(old));
        let seen: Vec<String> = BEATS.iter().filter(|b| b.id != new)
            .map(|b| b.id.to_owned()).collect();
        assert_eq!(next_beat(hero, 2, &flags, &seen).map(|b| b.id), Some(new));
    }
}
