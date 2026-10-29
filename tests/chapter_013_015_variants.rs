//! Ch013-015: pinned local variants and source-graded playable choices.
use three_kingdoms_spire_reborn::{Action, Game, Stage, content::Hero, lore};

fn scene(hero: Hero, id: &str, seed: u64) -> Game {
    let mut game = Game::new(hero, seed);
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
    game
}

#[test]
fn local_variants_are_pinned_without_claiming_full_collation() {
    let evidence = include_str!("../docs/CHAPTERS_013_015_VARIANT_HISTORY_SPOTCHECK.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["8185567", "2613836", "8729938", "2554688", "2554677",
        "2554690", "2554659", "2596801", "2554675", "2585429", "2452141", "2075209"] {
        assert!(evidence.contains(oldid), "missing pinned source {oldid}");
    }
    for chapter in ["013", "014", "015"] {
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter}\t")))
            .expect("chapter ledger row");
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("局部异文"));
        assert!(row.contains("未完成全回异文及史料校勘"));
    }
    let xiaopei = lore::beat("xiaopei").unwrap();
    let xudu = lore::beat("xudu").unwrap();
    let shenting = lore::beat("shenting").unwrap();
    assert_eq!((xiaopei.chapter, xudu.chapter, shenting.chapter), (13, 14, 15));
    for term in ["袁绍", "颜良", "二十余万", "四十余万", "军粮"] {
        assert!(xiaopei.premise.contains(term), "013 missing {term}");
    }
    for term in ["二虎竞食", "二虎竞餐", "南郡", "南阳", "陈寿正文"] {
        assert!(xudu.premise.contains(term), "014 missing {term}");
    }
    for term in ["严舆", "坐席", "吴历", "六十日", "裴松之"] {
        assert!(shenting.premise.contains(term), "015 missing {term}");
    }
}

#[test]
fn three_chapter_responsibility_choices_remain_atomic_mutually_exclusive_and_v2_safe() {
    for (hero, id, flag) in [
        (Hero::LuBu, "xiaopei", "xiaopei_refuge"),
        (Hero::CaoCao, "xudu", "xu_capital"),
        (Hero::SunShangxiang, "shenting", "taishi_trust"),
    ] {
        for choice in [1, 2, 3] {
            let mut game = scene(hero, id, 13000 + choice as u64);
            let before = game.to_json().unwrap();
            assert!(game.apply(Action::Choose { index: 4 }).is_err());
            assert_eq!(game.to_json().unwrap(), before, "{id}: invalid action changed state");
            game.apply(Action::Choose { index: choice }).unwrap();
            assert_eq!(game.lore_flags.iter().any(|s| s == flag), choice == 1);
            assert!(game.lore_seen.iter().any(|s| s == id));
            let saved = game.to_json().unwrap();
            assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
        }
    }
}

#[test]
fn shenting_existing_cross_act_reward_requires_three_accepted_ledgers_and_pays_once() {
    let mut game = scene(Hero::SunShangxiang, "sun_father_letter", 15015);
    game.apply(Action::Choose { index: 1 }).unwrap();
    game.stage = Stage::Story;
    game.lore_current = Some("shenting".into());
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(!game.easter_unlocked.iter().any(|id| id == "taishi_return"));
    game.stage = Stage::Story;
    game.lore_current = Some("sun_yiling_shelter".into());
    let gold = game.gold;
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.gold, gold + 17 + 24);
    assert_eq!(game.easter_unlocked.iter().filter(|id| id.as_str() == "taishi_return").count(), 1);
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(restored.easter_unlocked, game.easter_unlocked);
    assert!(lore::newly_completed_eggs(game.hero, &game.lore_flags, &game.easter_unlocked)
        .iter().all(|egg| egg.id != "taishi_return"));
    for missing in ["sun_father_dispatch", "taishi_trust", "sun_yiling_relief"] {
        let incomplete: Vec<String> = game.lore_flags.iter()
            .filter(|flag| flag.as_str() != missing).cloned().collect();
        assert!(!lore::newly_completed_eggs(game.hero, &incomplete, &[])
            .iter().any(|egg| egg.id == "taishi_return"), "unearned without {missing}");
    }
}
