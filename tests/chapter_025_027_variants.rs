//! Chapters 025-027: pinned electronic Mao/Jiajing spot checks, not full collation.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(items: &[String], id: &str) -> usize {
    items.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_witnesses_and_existing_rewards_are_preserved() {
    let doc = include_str!("../docs/CHAPTERS_025_027_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2567988", "8185862", "8186000", "2554662", "2554712",
                  "2554647", "2554655", "2554680", "2554652", "2583383"] {
        assert!(doc.contains(oldid), "missing fixed revision {oldid}");
    }
    for anchor in ["凡有殺戮", "下務其功", "不務其功", "壽亭侯印",
                   "漢壽亭侯之印", "裴松之", "原刻影印", "0回"] {
        assert!(doc.contains(anchor), "missing textual witness {anchor}");
    }
    let rows: Vec<_> = audit.lines().skip(1).filter(|s| !s.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for chapter in 25..=27 {
        assert!(rows.iter().any(|row| row.starts_with(&format!("{chapter:03}\\t"))
            && row.contains("CHAPTERS_025_027_JIAJING_VARIANTS.md")));
    }
    for (id, chapter, flag, gold, heal, anchor) in [
        ("baima", 25, "baima_debt", 28, 2, "《三国志》卷36"),
        ("baima_terms", 25, "three_terms_record", 18, 6, "嘉靖049"),
        ("seal_return", 26, "seal_return_record", 16, 7, "嘉靖051"),
        ("five_passes", 27, "five_passes_record", 17, 6, "嘉靖053"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.flag, b.reward_gold, b.reward_heal),
                   (chapter, flag, gold, heal));
        assert!(b.premise.contains(anchor), "{id} lacks source boundary");
    }
}

#[test]
fn guan_accepted_chain_is_atomic_v2_safe_and_rewards_only_once() {
    let mut game = Game::new(Hero::GuanYu, 25027);
    game.lore_flags.push("wenjiu_oath".into());
    for id in ["baima", "baima_terms", "seal_return", "five_passes"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    for flag in ["baima_debt", "three_terms_record", "seal_return_record",
                 "five_passes_record"] {
        assert_eq!(count(&game.lore_flags, flag), 1);
    }
    assert_eq!(count(&game.easter_unlocked, "three_terms_huarong"), 0);
    let mut restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut restored, "five_passes");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "five_passes_record"), 1);
    enter(&mut restored, "gucheng_witness");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut restored, "huarong");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    for egg in ["three_terms_huarong", "five_passes_huarong"] {
        assert_eq!(count(&restored.easter_unlocked, egg), 1);
    }
    let mut twice = Game::from_json(&restored.to_json().unwrap()).unwrap();
    enter(&mut twice, "huarong");
    twice.apply(Action::Choose { index: 1 }).unwrap();
    for egg in ["three_terms_huarong", "five_passes_huarong"] {
        assert_eq!(count(&twice.easter_unlocked, egg), 1);
    }
}

#[test]
fn refusal_preserves_older_route_without_forged_acceptance() {
    let mut game = Game::new(Hero::GuanYu, 25028);
    game.lore_flags.push("wenjiu_oath".into());
    enter(&mut game, "baima");
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "baima_terms");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&game.lore_flags, "three_terms_record"), 0);
    enter(&mut game, "huarong");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game.easter_unlocked, "sword_debt"), 1);
    assert_eq!(count(&game.easter_unlocked, "three_terms_huarong"), 0);
    let mut seal = Game::new(Hero::GuanYu, 25029);
    seal.lore_flags.push("three_terms_record".into());
    enter(&mut seal, "seal_return");
    seal.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&seal.lore_flags, "seal_return_record"), 0);
    assert!(Game::from_json(&seal.to_json().unwrap()).is_ok());
}
