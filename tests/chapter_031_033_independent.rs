//! Independent Mao 031-033 reread; only pinned single-base evidence, not full variant/history collation.
use three_kingdoms_spire_reborn::{Action, Game, Stage, content::Hero, lore::{beat, next_beat}};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn has(g: &Game, flag: &str) -> bool { g.lore_flags.iter().any(|s| s == flag) }
fn egg_count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_mao_031_033_independent_evidence_and_liaodong_agency() {
    let doc = include_str!("../docs/CHAPTERS_031_033_JIAJING_VARIANTS.md");
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for (ch, oldid, anchor) in [
        ("031", "2567992", "逢纪谗称"),
        ("032", "2567994", "审荣因怨献门"),
        ("033", "2567995", "公孙恭劝先侦"),
    ] {
        let rows: Vec<_> = ledger.lines().filter(|r| r.starts_with(&format!("{ch}\t"))).collect();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].contains(&format!("2026-11-02独立复核毛评固定oldid={oldid}")));
        assert!(doc.contains(&format!("oldid={oldid}")));
        assert!(doc.contains(anchor));
    }
    let after = beat("northern_settlement").unwrap();
    for anchor in ["公孙恭", "公孙康", "袁熙袁尚", "不能写成曹军直接处决"] {
        assert!(after.premise.contains(anchor), "{anchor}");
    }
    assert_eq!(beat("liaodong").unwrap().prerequisite, Some("guandu_grain"));
    assert_eq!(next_beat(Hero::CaoCao, 1,
        &["guandu_grain".into(), "guandu_captured_letters".into()],
        &["guandu_letters".into()]).map(|b| b.id), Some("cangting_memorial"));
}

#[test]
fn chapter_031_033_refusal_does_not_forge_cross_act_credits_or_block_old_saves() {
    let mut g = Game::new(Hero::CaoCao, 31036);
    g.lore_flags.extend(["guandu_grain", "guandu_captured_letters"].map(str::to_string));
    enter(&mut g, "cangting_memorial");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "cangting_memorial_record"));
    enter(&mut g, "ye_flood_ledger");
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&g, "ye_flood_record"));
    let mut g = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g, "liaodong");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "liaodong_letter"));
    assert_eq!(egg_count(&g, "guandu_ye_civilians"), 0);
    enter(&mut g, "northern_settlement");
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&g, "northern_settlement_account"));
    assert_eq!(egg_count(&g, "northern_aftermath_debt"), 0);
    let restored = Game::from_json(&g.to_json().unwrap()).unwrap();
    assert!(!has(&restored, "northern_settlement_account"));
    let mut old = Game::new(Hero::CaoCao, 31037);
    old.lore_flags.push("guandu_grain".into());
    enter(&mut old, "liaodong");
    old.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&old, "liaodong_letter"));
}
