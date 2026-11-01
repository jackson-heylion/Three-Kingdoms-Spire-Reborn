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

#[test]
fn independent_full_read_boundaries_and_refusal_keep_old_jingzhou_open() {
    let doc=include_str!("../docs/CHAPTERS_034_036_JIAJING_VARIANTS.md");
    let audit=include_str!("../data/chapter_body_audit.tsv");
    for marker in ["正文L119—189","正文L119—165","正文L116—165","第二次警告","刘备明确拒绝"] {
        assert!(doc.contains(marker), "missing evidence {marker}");
    }
    let rows:Vec<_>=audit.lines().skip(1).filter(|s|!s.is_empty()).collect();
    assert_eq!(rows.len(),120);
    for ch in 34..=36 {
        let prefix=format!("{ch:03}\t");
        let matches:Vec<_>=rows.iter().filter(|s|s.starts_with(&prefix)).collect();
        assert_eq!(matches.len(),1);
        assert!(matches[0].contains("2026-11-02独立固定单底本复核"));
    }
    let b=beat("fancheng_counsel").unwrap();
    for marker in ["徐母回帖笔迹","刘备拒绝以母命留人","徐母之死在第037回"] {
        assert!(b.premise.contains(marker), "missing moral causality {marker}");
    }
    let mut g=at("fancheng_counsel",&["xinye_counsel_record","anxi_seal"]);
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:2}).unwrap();
    assert!(!has(&g,"fancheng_counsel_record"));
    let mut g=Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g,"jingzhou");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"jingzhou_warning"));
    assert_eq!(awards(&g,"fancheng_xushu_jingzhou"),0);
}
