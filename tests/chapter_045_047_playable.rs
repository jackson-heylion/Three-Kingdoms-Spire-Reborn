//! Pinned chapters 045-047: Cao's retrospective intelligence choices, no duplicated borrow_arrows.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};
fn at(id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(Hero::CaoCao, 4547);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|f| (*f).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn has(g: &Game, f: &str) -> bool { g.lore_flags.iter().any(|x| x == f) }
fn awards(g: &Game) -> usize { g.easter_unlocked.iter().filter(|x| x.as_str() == "river_spy_chain_debt").count() }
#[test]
fn fixed_sources_and_unique_playable_edges_are_explicit() {
    for (chapter, oldid, doc) in [
        (45, "2568007", include_str!("../docs/CHAPTER_045_READING.md")),
        (46, "2568009", include_str!("../docs/CHAPTER_046_READING.md")),
        (47, "2568010", include_str!("../docs/CHAPTER_047_READING.md")),
    ] { assert!(doc.contains(&format!("oldid={oldid}")) && doc.contains("单底本"), "{chapter}"); }
    assert_eq!(beat("jiangkou_inquest").unwrap().prerequisite, Some("guandu_grain"));
    assert_eq!(beat("chain_ship_ledger").unwrap().prerequisite, Some("jiangkou_inquest_record"));
    assert_eq!(beat("jiangkou_inquest").unwrap().chapter, 45);
    assert_eq!(beat("chain_ship_ledger").unwrap().chapter, 47);
    assert_eq!(beat("borrow_arrows").unwrap().chapter, 46);
    assert_eq!(beat("jiangkou_inquest").unwrap().act, 1);
    assert_eq!(beat("chain_ship_ledger").unwrap().act, 2);
}
#[test]
fn false_letter_chain_is_atomic_persistent_and_once_only() {
    let mut g = at("jiangkou_inquest", &["guandu_grain"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "jiangkou_inquest_record"));
    assert_eq!(awards(&g), 0);
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(), saved);
    enter(&mut g, "chain_ship_ledger");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "chain_ship_risk_record"));
    assert_eq!(awards(&g), 1);
    enter(&mut g, "chain_ship_ledger");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g), 1);
}
#[test]
fn refusal_does_not_force_new_chain_or_block_existing_liaodong() {
    let mut g = at("jiangkou_inquest", &["guandu_grain"]);
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&g, "jiangkou_inquest_record"));
    assert_eq!(awards(&g), 0);
    assert_eq!(next_beat(Hero::CaoCao, 2, &g.lore_flags, &g.lore_seen).map(|b| b.id), Some("liaodong"));
    enter(&mut g, "liaodong");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "liaodong_letter"));
    assert_eq!(awards(&g), 0);
}
