//! Fixed 081–083 text, narrative boundaries, save atomicity, rewards, and old-route priority.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 8183);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|flag| (*flag).to_owned()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|value| value.as_str() == id).count()
}

#[test]
fn fixed_chapters_and_history_are_scoped_not_falsely_collated() {
    for (number, revision, evidence) in [
        (81, "8730381", include_str!("../docs/CHAPTER_081_READING.md")),
        (82, "2568044", include_str!("../docs/CHAPTER_082_READING.md")),
        (83, "8730382", include_str!("../docs/CHAPTER_083_READING.md")),
    ] {
        assert!(evidence.contains(&format!("oldid={revision}")), "{number}");
        assert!(evidence.contains("单底本") && evidence.contains("回末"), "{number}");
    }
    let history = include_str!("../docs/CHAPTERS_081_083_HISTORY_SPOTCHECK.md");
    for revision in ["2583383", "2602668", "2131313"] {
        assert!(history.contains(revision));
    }
    assert!(history.contains("220") && history.contains("222") && history.contains("0/120"));
    for (id, chapter, prerequisite) in [
        ("eastward_campaign_counsel", 81, "chengdu_legitimacy_record"),
        ("langzhong_heads_receipt", 81, "jingzhou_aftermath_record"),
        ("wu_wei_envoy_audit", 82, "langzhong_heads_record"),
        ("wei_wu_tribute_review", 82, "abdication_seal_record"),
        ("yiling_defense_memorial", 83, "wu_wei_envoy_record"),
        ("yiling_camp_map_warning", 83, "eastward_counsel_record"),
    ] {
        let beat = beat(id).unwrap();
        assert_eq!(beat.chapter, chapter);
        assert_eq!(beat.act, 2);
        assert_eq!(beat.prerequisite, Some(prerequisite));
    }
    assert!(beat("wei_wu_tribute_review").unwrap().premise.contains("曹操已于078病卒"));
    assert!(beat("yiling_defense_memorial").unwrap().premise.contains("220年"));
    assert!(beat("yiling_camp_map_warning").unwrap().premise.contains("084"));
}

#[test]
fn sun_envoy_to_yiling_chain_is_atomic_exclusive_once_only_and_restorable() {
    let flags = ["jiangdong_covenant", "jingzhou_aftermath_record"];
    let mut game = scene(Hero::SunShangxiang, "langzhong_heads_receipt", &flags);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "wu_wei_envoy_audit");
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "yiling_defense_memorial");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "jiangdong_yiling_guard_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "yiling_defense_memorial");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "jiangdong_yiling_guard_debt"), 1);

    let mut declined = scene(Hero::SunShangxiang, "langzhong_heads_receipt", &flags);
    declined.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut declined, "wu_wei_envoy_audit");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut declined, "yiling_defense_memorial");
    declined.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&declined, "jiangdong_yiling_guard_debt"), 0);
}

#[test]
fn zhuge_camp_map_and_wei_posthumous_archives_require_complete_flags() {
    let mut zhuge = scene(Hero::ZhugeLiang, "eastward_campaign_counsel", &[
        "redcliff_wind", "chengdu_legitimacy_record",
    ]);
    zhuge.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut zhuge, "yiling_camp_map_warning");
    zhuge.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&zhuge, "redcliff_yiling_counsel_debt"), 1);

    let mut cao = scene(Hero::CaoCao, "wei_wu_tribute_review", &[
        "guandu_grain", "abdication_seal_record",
    ]);
    cao.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&cao, "guandu_wei_wu_tribute_debt"), 1);

    let mut refused = scene(Hero::ZhugeLiang, "eastward_campaign_counsel", &[
        "redcliff_wind", "chengdu_legitimacy_record",
    ]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "yiling_camp_map_warning");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "redcliff_yiling_counsel_debt"), 0);
}

#[test]
fn new_optional_records_never_steal_legacy_routes_and_remain_selectable() {
    for (hero, flags, old, new) in [
        (Hero::SunShangxiang, vec!["jiangdong_covenant", "jingzhou_aftermath_record"], "yiling", "langzhong_heads_receipt"),
        (Hero::CaoCao, vec!["guandu_grain", "abdication_seal_record"], "liaodong", "wei_wu_tribute_review"),
        (Hero::ZhugeLiang, vec!["redcliff_wind", "chengdu_legitimacy_record"], "wuzhang", "eastward_campaign_counsel"),
    ] {
        let flags: Vec<String> = flags.into_iter().map(str::to_owned).collect();
        assert_eq!(next_beat(hero, 2, &flags, &[]).map(|beat| beat.id), Some(old));
        let seen: Vec<String> = BEATS.iter().filter(|beat| beat.id != new)
            .map(|beat| beat.id.to_owned()).collect();
        assert_eq!(next_beat(hero, 2, &flags, &seen).map(|beat| beat.id), Some(new));
    }
}
