//! 085-087 fixed JiaJing/Mao local variants and playable cross-chapter civilian debt.
//! Scripted story entry proves state transitions, not natural campaign reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
}
fn egg_count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}
fn flag_count(game: &Game, id: &str) -> usize {
    game.lore_flags.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn nine_pinned_revisions_and_six_existing_playable_contracts() {
    let evidence = include_str!("../docs/CHAPTERS_085_087_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in ["2568048", "2568049", "2568051", "2554780", "2554830",
                "2554784", "2554739", "2554829", "2554806"] {
        assert!(evidence.contains(rev), "missing fixed revision {rev}");
    }
    for anchor in ["不識虛實", "能識虛實", "龍舟一隻", "龍舟十隻", "關索",
                   "原刻", "0/120", "史料"] {
        assert!(evidence.contains(anchor), "missing source distinction {anchor}");
    }
    assert_eq!(audit.lines().skip(1).filter(|line| !line.is_empty()).count(), 120);
    for chapter in 85..=87 {
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("CHAPTERS_085_087_JIAJING_VARIANTS.md"));
        assert!(row.contains("局部"));
    }
    for (id, chapter, pre, flag, gold, heal) in [
        ("yongan_regency_register", 85, "fishbelly_report_record", "yongan_regency_record", 23, 9),
        ("wei_three_fronts_review", 85, "wei_wu_tribute_record", "wei_three_fronts_record", 22, 8),
        ("dengzhi_alliance_dispatch", 86, "yongan_regency_record", "dengzhi_alliance_record", 24, 10),
        ("guangling_river_defense", 86, "yiling_fire_refuge_record", "guangling_defense_record", 23, 10),
        ("nanzhong_yongchang_memorial", 87, "dengzhi_alliance_record", "nanzhong_yongchang_record", 22, 10),
        ("nanzhong_first_release_ledger", 87, "nanzhong_yongchang_record", "nanzhong_first_release_record", 21, 10),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.prerequisite, b.flag), (chapter, Some(pre), flag));
        assert_eq!((b.reward_gold, b.reward_heal), (gold, heal));
        assert!(b.premise.contains("嘉靖") && b.premise.contains("毛评"), "{id}");
    }
}

#[test]
fn regency_envoy_yongchang_and_first_release_unlock_once_after_v2_restore() {
    let mut game = Game::new(Hero::ZhugeLiang, 85087);
    for flag in ["redcliff_wind", "yiling_camp_map_record"] {
        game.lore_flags.push(flag.to_owned());
    }
    for id in ["fishbelly_eightfold_report", "yongan_regency_register",
               "dengzhi_alliance_dispatch", "nanzhong_yongchang_memorial",
               "nanzhong_first_release_ledger"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before, "invalid choice mutated {id}");
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    for flag in ["fishbelly_report_record", "yongan_regency_record",
                 "dengzhi_alliance_record", "nanzhong_yongchang_record",
                 "nanzhong_first_release_record"] {
        assert_eq!(flag_count(&game, flag), 1, "{flag}");
    }
    assert_eq!(egg_count(&game, "redcliff_yongan_envoy_debt"), 1);
    assert_eq!(egg_count(&game, "yongan_nanzhong_civilian_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "nanzhong_first_release_ledger");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(egg_count(&restored, "yongan_nanzhong_civilian_debt"), 1);
    assert_eq!(flag_count(&restored, "nanzhong_first_release_record"), 1);
}

#[test]
fn refusing_first_release_does_not_pay_optional_civilian_reward() {
    let mut game = Game::new(Hero::ZhugeLiang, 85088);
    for flag in ["yongan_regency_record", "dengzhi_alliance_record",
                 "nanzhong_yongchang_record"] {
        game.lore_flags.push(flag.to_owned());
    }
    enter(&mut game, "nanzhong_first_release_ledger");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(flag_count(&game, "nanzhong_first_release_record"), 0);
    assert_eq!(egg_count(&game, "yongan_nanzhong_civilian_debt"), 0);
}
