//! Fixed-edition spot checks for chapters 034-036; executable optional choices.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;

fn at(id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(Hero::GuanYu, 34036);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn has(g: &Game, flag: &str) -> bool { g.lore_flags.iter().any(|f| f == flag) }
fn awards(g: &Game, id: &str) -> usize { g.easter_unlocked.iter().filter(|f| f.as_str() == id).count() }

#[test]
fn witnesses_are_pinned_and_variant_claims_are_scoped() {
    let evidence = include_str!("../docs/CHAPTERS_034_036_JIAJING_VARIANTS.md");
    for oldid in ["2567997","2568001","2568002","2554672","2554654","2554684","2554679","2554661","2554704"] {
        assert!(evidence.contains(oldid), "missing fixed witness {oldid}");
    }
    for phrase in ["張武、陳孫", "張虎、陳生", "山谷有賢", "四海有賢", "十倍於昱", "昱十分，得徐庶一二"] {
        assert!(evidence.contains(phrase), "missing contrast {phrase}");
    }
    for (id, ch, pre) in [
        ("tanxi_warning",34,"ancient_reunion"),
        ("xinye_counsel",35,"tanxi_warning_record"),
        ("fancheng_counsel",36,"xinye_counsel_record"),
    ] {
        let b=beat(id).unwrap();
        assert_eq!(b.chapter,ch);
        assert_eq!(b.prerequisite,Some(pre));
    }
    assert!(beat("tanxi_warning").unwrap().premise.contains("张虎"));
    assert!(beat("xinye_counsel").unwrap().premise.contains("四海有贤"));
    assert!(beat("fancheng_counsel").unwrap().premise.contains("十倍于昱"));
}

#[test]
fn guan_yu_034_036_choice_chain_is_atomic_and_v2_persistent() {
    let mut g=at("tanxi_warning",&["ancient_reunion","anxi_seal"]);
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"tanxi_warning_record"));
    enter(&mut g,"xinye_counsel");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"xinye_counsel_record"));
    let saved=g.to_json().unwrap();
    let mut g=Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(),saved);
    enter(&mut g,"fancheng_counsel");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"fancheng_counsel_record"));
    enter(&mut g,"xushu_departure");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"xushu_departure_record"));
    enter(&mut g,"jingzhou");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(awards(&g,"fancheng_xushu_jingzhou"),1);
    enter(&mut g,"jingzhou");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(awards(&g,"fancheng_xushu_jingzhou"),1);
}

#[test]
fn refusing_new_evidence_keeps_old_jingzhou_playable() {
    let mut g=at("tanxi_warning",&["ancient_reunion","anxi_seal"]);
    g.apply(Action::Choose{index:2}).unwrap();
    assert!(!has(&g,"tanxi_warning_record"));
    enter(&mut g,"jingzhou");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"jingzhou_warning"));
    assert_eq!(awards(&g,"fancheng_xushu_jingzhou"),0);
    let mut g=at("xinye_counsel",&["tanxi_warning_record","anxi_seal"]);
    g.apply(Action::Choose{index:2}).unwrap();
    assert!(!has(&g,"xinye_counsel_record"));
    enter(&mut g,"jingzhou");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"jingzhou_warning"));
}
