//! Pinned edition evidence and real Game::apply story settlements.
use three_kingdoms_spire_reborn::{Action, Game, Stage, content::Hero, lore};

fn scene(hero: Hero, seed: u64, id: &str) -> Game {
    let mut game = Game::new(hero, seed);
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
    game
}
fn choose(game: &mut Game, id: &str, index: usize) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
    game.apply(Action::Choose { index }).unwrap();
}

#[test]
fn pinned_sources_and_partial_status_are_explicit() {
    let evidence = include_str!("../docs/CHAPTERS_007_009_VARIANT_HISTORY_SPOTCHECK.md");
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2567970","5314829","2567971","2646264","2554673",
                  "2554651","2554711","2554640","2554683","2132349","2452141"] {
        assert!(evidence.contains(oldid), "missing fixed source {oldid}");
    }
    for ch in ["007","008","009"] {
        let row = ledger.lines().find(|r| r.starts_with(&format!("{ch}\t"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("局部对校") && row.contains("非全回异文校勘"));
    }
    assert!(lore::beat("sun_father_letter").unwrap().premise.contains("郭图"));
    assert!(lore::beat("pavilion").unwrap().premise.contains("二百五十里"));
    assert!(lore::beat("pavilion").unwrap().premise.contains("二百六十里"));
    assert!(lore::beat("changan").unwrap().premise.contains("裴松之"));
}

#[test]
fn three_accepted_lu_ledgers_unlock_once_and_survive_restore() {
    let mut game = scene(Hero::LuBu, 7009, "redhare");
    game.apply(Action::Choose { index: 1 }).unwrap();
    choose(&mut game, "pavilion", 1);
    assert!(!game.easter_unlocked.iter().any(|x| x == "changan_court_military_debt"));
    choose(&mut game, "changan", 1);
    assert!(game.lore_flags.iter().any(|x| x == "changan_account"));
    assert_eq!(game.easter_unlocked.iter().filter(|x| x.as_str() == "changan_court_military_debt").count(), 1);
    assert!(lore::newly_completed_eggs(game.hero, &game.lore_flags, &game.easter_unlocked)
        .iter().all(|x| x.id != "changan_court_military_debt"));
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(restored.easter_unlocked, game.easter_unlocked);
    assert_eq!(restored.gold, game.gold);
}

#[test]
fn all_choices_are_exclusive_and_invalid_choice_is_atomic() {
    for (hero, id, flag) in [
        (Hero::SunShangxiang, "sun_father_letter", "sun_father_dispatch"),
        (Hero::LuBu, "pavilion", "phoenix_pavilion"),
        (Hero::LuBu, "changan", "changan_account"),
    ] {
        for index in [1,2,3] {
            let mut game = scene(hero, 7080 + index as u64, id);
            let before = game.to_json().unwrap();
            assert!(game.apply(Action::Choose { index: 4 }).is_err());
            assert_eq!(game.to_json().unwrap(), before);
            game.apply(Action::Choose { index }).unwrap();
            assert_eq!(game.lore_flags.iter().any(|x| x == flag), index == 1);
            let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
            assert_eq!(restored.lore_flags, game.lore_flags);
        }
    }
}

#[test]
fn rejecting_any_required_ledger_blocks_the_conclusion() {
    for rejected in ["redhare","pavilion","changan"] {
        let mut game = scene(Hero::LuBu, 7099, "redhare");
        for id in ["redhare","pavilion","changan"] {
            choose(&mut game, id, if id == rejected { 2 } else { 1 });
        }
        assert!(!game.easter_unlocked.iter().any(|x| x == "changan_court_military_debt"));
    }
    let flags = vec!["red_hare".into(),"phoenix_pavilion".into(),"changan_account".into()];
    assert!(lore::newly_completed_eggs(Hero::LuBu, &flags, &[])
        .iter().any(|x| x.id == "changan_court_military_debt"));
    assert!(lore::newly_completed_eggs(Hero::LuBu, &flags, &["changan_court_military_debt".into()])
        .iter().all(|x| x.id != "changan_court_military_debt"));
}

#[test]
fn independent_mao_007_009_reading_preserves_three_chapter_boundaries() {
    let evidence = include_str!("../docs/CHAPTERS_007_009_VARIANT_HISTORY_SPOTCHECK.md");
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for marker in [
        "2026-11-01 毛评固定电子本007",
        "oldid=2567970",
        "oldid=5314829",
        "oldid=2567971",
        "oldid=2646264",
        "oldid=2554651",
        "oldid=2554683",
        "桓楷",
        "李儒",
        "李蒙王方",
        "献帝尚未被杀",
        "未做原刻影印核字",
    ] {
        assert!(evidence.contains(marker), "missing chapter boundary evidence: {marker}");
    }
    for (chapter, oldid) in [("007", "2567970"), ("008", "5314829"), ("009", "2567971")] {
        let row = ledger
            .lines()
            .find(|line| line.starts_with(&format!("{chapter}\t")))
            .expect("unique chapter row");
        assert!(row.contains(&format!("2026-11-01独立复核毛评固定oldid={oldid}")));
        assert!(row.contains("非全回异文校勘"));
        assert!(row.contains("未完成全回史传核证"));
    }
}

#[test]
fn sun_and_lu_first_natural_story_choices_are_atomic_and_restorable() {
    use three_kingdoms_spire_reborn::Node;

    for (hero, first_id, accepted_flag) in [
        (Hero::SunShangxiang, "sun_father_letter", "sun_father_dispatch"),
        (Hero::LuBu, "redhare", "red_hare"),
    ] {
        let mut reached = 0;
        for seed in 1..=512u64 {
            let mut game = Game::new(hero, seed);
            let Some(index) = game.route.iter().position(|node| *node == Node::Story) else {
                continue;
            };
            game.apply(Action::Choose { index: index + 1 }).unwrap();
            assert_eq!(game.lore_current.as_deref(), Some(first_id), "seed {seed}");
            let before = game.to_json().unwrap();
            assert!(game.apply(Action::Choose { index: 4 }).is_err());
            assert_eq!(game.to_json().unwrap(), before, "rejected seed {seed}");
            let accepted = reached % 2 == 0;
            game.apply(Action::Choose { index: if accepted { 1 } else { 3 } }).unwrap();
            assert_eq!(
                game.lore_flags.iter().any(|flag| flag == accepted_flag),
                accepted
            );
            let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
            assert_eq!(restored.lore_flags, game.lore_flags);
            reached += 1;
            if reached == 4 {
                break;
            }
        }
        assert!(reached >= 2, "no natural opening Story for {first_id} in 512 seeds");
    }
}
