//! Chapter 004-006 pinned source differences and executable optional consequences.
use three_kingdoms_spire_reborn::{Action, Game, Node, Stage, content::Hero, lore};

fn scene(hero: Hero, id: &str, seed: u64) -> Game {
    let mut game = Game::new(hero, seed);
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
    game
}

#[test]
fn fixed_novels_and_pei_commentary_remain_partial_collation() {
    let evidence = include_str!("../docs/CHAPTERS_004_006_VARIANT_HISTORY_SPOTCHECK.md");
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2567966", "7910939", "2567969", "2554691", "2554674",
        "2554668", "2650987", "2554641", "2554658", "2629010"] {
        assert!(evidence.contains(oldid), "missing fixed source {oldid}");
    }
    for chapter in ["004", "005", "006"] {
        let row = ledger.lines().find(|s| s.starts_with(&format!("{chapter}\t")))
            .expect("chapter ledger row");
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("局部异文"));
        assert!(row.contains("未完成全回异文校勘"));
    }
    assert!(lore::beat("sevenstar").unwrap().premise.contains("蔡邕"));
    assert!(lore::beat("sevenstar").unwrap().premise.contains("李儒"));
    assert!(lore::beat("wenjiu").unwrap().premise.contains("孙坚军"));
    assert!(lore::beat("wenjiu").unwrap().premise.contains("陈寿正文"));
    assert!(lore::beat("imperial_jade").unwrap().premise.contains("裴注"));
    assert!(lore::beat("imperial_jade").unwrap().premise.contains("裴松之反驳"));
    assert!(lore::beat("imperial_jade").unwrap().vow.contains("裴注争议"));
}

#[test]
fn accepted_cao_source_chain_awards_once_and_survives_v2() {
    let mut game = scene(Hero::CaoCao, "sevenstar", 4006);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|f| f == "sevenstar_vow"));
    assert!(!game.easter_unlocked.iter().any(|id| id == "sevenstar_jade_source_debt"));
    game.stage = Stage::Story;
    game.lore_current = Some("imperial_jade".into());
    let gold = game.gold;
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(game.gold, gold + 17 + 14);
    assert!(game.lore_flags.iter().any(|f| f == "jade_whisper"));
    assert_eq!(game.easter_unlocked.iter().filter(|id| id.as_str() == "sevenstar_jade_source_debt").count(), 1);
    assert!(!lore::newly_completed_eggs(game.hero, &game.lore_flags, &game.easter_unlocked)
        .iter().any(|egg| egg.id == "sevenstar_jade_source_debt"));
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(restored.gold, game.gold);
    assert_eq!(restored.easter_unlocked, game.easter_unlocked);
}

#[test]
fn refused_or_missing_responsibility_cannot_unlock_cross_chapter_debt() {
    for (first, second) in [(1, 2), (1, 3), (2, 1), (3, 1)] {
        let mut game = scene(Hero::CaoCao, "sevenstar", 5000 + first as u64 * 10 + second as u64);
        game.apply(Action::Choose { index: first }).unwrap();
        game.stage = Stage::Story;
        game.lore_current = Some("imperial_jade".into());
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before, "invalid choice changed save");
        game.apply(Action::Choose { index: second }).unwrap();
        assert!(!game.easter_unlocked.iter().any(|id| id == "sevenstar_jade_source_debt"));
        assert!(lore::newly_completed_eggs(game.hero, &game.lore_flags, &game.easter_unlocked)
            .iter().all(|egg| egg.id != "sevenstar_jade_source_debt"));
        let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        assert_eq!(restored.easter_unlocked, game.easter_unlocked);
    }
}

#[test]
fn guan_wenjiu_acceptance_preserves_novel_and_history_without_changing_outcome() {
    for index in [1, 2, 3] {
        let mut game = scene(Hero::GuanYu, "wenjiu", 6000 + index as u64);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index }).unwrap();
        assert_eq!(game.lore_flags.iter().any(|f| f == "wenjiu_oath"), index == 1);
        assert!(game.lore_seen.iter().any(|id| id == "wenjiu"));
        let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        assert_eq!(restored.lore_flags, game.lore_flags);
    }
}

#[test]
fn independently_read_fixed_mao_004_006_and_distinguish_yingyang_reinforcements() {
    let evidence = include_str!("../docs/CHAPTERS_004_006_VARIANT_HISTORY_SPOTCHECK.md");
    for marker in ["2026-11-01 毛评固定电子本004", "oldid=2567966",
        "oldid=7910939", "oldid=2567969", "oldid=2554658",
        "吕伯奢", "陈宫弃曹", "祖茂", "曹洪", "十數騎", "數十騎",
        "回末孙坚被困尚未脱困", "裴松之"] {
        assert!(evidence.contains(marker), "missing verified narrative evidence {marker}");
    }
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for chapter in ["004", "005", "006"] {
        let row = ledger.lines().find(|line| line.starts_with(&format!("{chapter}\t")))
            .expect("chapter row");
        assert!(row.contains("2026-11-01独立复核毛评固定oldid="));
        assert!(row.contains("未完成全回异文校勘"));
    }
    let cao = lore::beat("sevenstar").unwrap();
    assert!(cao.premise.contains("伍孚") && cao.premise.contains("明知故杀"));
    let jade = lore::beat("imperial_jade").unwrap();
    assert!(jade.premise.contains("曹洪") && jade.premise.contains("十数骑"));
    assert!(jade.premise.contains("数十骑") && jade.premise.contains("裴注"));
}

#[test]
fn cao_first_natural_story_is_optional_atomic_and_restorable() {
    let mut reached = 0;
    for seed in 1..=512u64 {
        let mut game = Game::new(Hero::CaoCao, seed);
        let Some(index) = game.route.iter().position(|node| *node == Node::Story) else { continue; };
        game.apply(Action::Choose { index: index + 1 }).unwrap();
        assert_eq!(game.lore_current.as_deref(), Some("sevenstar"), "seed {seed}");
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(game.to_json().unwrap(), before, "rejected action changed seed {seed}");
        let accept = reached % 2 == 0;
        game.apply(Action::Choose { index: if accept { 1 } else { 3 } }).unwrap();
        assert_eq!(game.lore_flags.iter().any(|flag| flag == "sevenstar_vow"), accept);
        let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
        assert_eq!(restored.lore_flags, game.lore_flags);
        reached += 1;
        if reached == 4 { break; }
    }
    assert!(reached >= 2, "no natural Cao first-story route in 512 seeds");
}
