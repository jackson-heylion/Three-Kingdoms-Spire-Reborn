//! Chapters 102–104: pinned full reading, original outcomes, V2 atomic choices and one-time debts.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 102104);
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
fn pinned_reading_and_history_layers_are_not_conflated() {
    for (chapter, oldid, doc) in [
        (102, "2568065", include_str!("../docs/CHAPTER_102_READING.md")),
        (103, "2568064", include_str!("../docs/CHAPTER_103_READING.md")),
        (104, "2568066", include_str!("../docs/CHAPTER_104_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={oldid}")), "{chapter}");
        assert!(doc.contains("单底本全文通读") && doc.contains("回末"), "{chapter}");
    }
    let history = include_str!("../docs/CHAPTERS_102_104_HISTORY_SPOTCHECK.md");
    assert!(history.contains("2115776") && history.contains("2583390"));
    assert!(history.contains("裴注") && history.contains("0/120"));
    let rows: Vec<_> = include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(rows.len(), 121);
    for (chapter, row) in rows.iter().enumerate().take(105).skip(102) {
        assert!(row.starts_with(&format!("{chapter:03}\t")));
        assert!(row.contains(&format!("CHAPTER_{chapter:03}_READING.md")));
        assert!(row.contains("正文通读/未完成异本对校"));
    }
    for (id, chapter) in [
        ("weibei_bridge_loss_ledger", 102),
        ("wooden_ox_transport_audit", 102),
        ("shangfang_fire_account", 103),
        ("wu_threefront_retreat_archive", 103),
        ("wuzhang_succession_register", 104),
        ("wuzhang_withdrawal_evidence", 104),
    ] {
        assert_eq!(beat(id).unwrap().chapter, chapter);
    }
}

#[test]
fn redcliff_to_wuzhang_debt_is_once_only_and_survives_v2_restore() {
    let mut game = scene(Hero::ZhugeLiang, "wooden_ox_transport_audit", &["redcliff_wind"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "redcliff_wuzhang_transport_debt"), 0);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "wuzhang_succession_register");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "redcliff_wuzhang_transport_debt"), 1);
    enter(&mut restored, "wuzhang_succession_register");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "redcliff_wuzhang_transport_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "wooden_ox_transport_audit", &["redcliff_wind"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "wuzhang_succession_register");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "redcliff_wuzhang_transport_debt"), 0);
    assert!(!refused.lore_flags.iter().any(|s| s == "weinan_transport_record"));
}

#[test]
fn bridge_losses_fire_risk_and_withdrawal_require_three_distinct_acceptances() {
    let mut game = scene(Hero::ZhugeLiang, "weibei_bridge_loss_ledger", &["redcliff_wind"]);
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "wooden_ox_transport_audit");
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "shangfang_fire_account");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "weibei_shangfang_withdrawal_debt"), 0);
    enter(&mut game, "wuzhang_withdrawal_evidence");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game, "weibei_shangfang_withdrawal_debt"), 1);
    let mut refused = scene(Hero::ZhugeLiang, "weibei_bridge_loss_ledger", &["redcliff_wind"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "wooden_ox_transport_audit");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut refused, "shangfang_fire_account");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut refused, "wuzhang_withdrawal_evidence");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "weibei_shangfang_withdrawal_debt"), 0);
}

#[test]
fn wu_court_archive_and_existing_wuzhang_scene_remain_optional_and_stable() {
    let mut wu = scene(Hero::SunShangxiang, "wu_threefront_retreat_archive", &["jiangdong_covenant"]);
    wu.apply(Action::Choose { index: 1 }).unwrap();
    assert!(wu.lore_flags.iter().any(|f| f == "wu_threefront_retreat_record"));
    let flags = vec!["redcliff_wind".to_owned()];
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &[]).map(|b| b.id), Some("wuzhang"));
    let seen: Vec<String> = BEATS.iter()
        .filter(|b| b.id != "weibei_bridge_loss_ledger")
        .map(|b| b.id.to_owned()).collect();
    assert_eq!(next_beat(Hero::ZhugeLiang, 2, &flags, &seen).map(|b| b.id), Some("weibei_bridge_loss_ledger"));
    for seed in 102100..102108 {
        let saved = Game::new(Hero::ZhugeLiang, seed).to_json().unwrap();
        assert!(Game::from_json(&saved).is_ok(), "seed {seed}");
    }
}
