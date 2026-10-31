//! Fixed Mao/Jiajing local comparisons; not full-text collation or natural-map proof.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) { g.stage = Stage::Story; g.lore_current = Some(id.into()); }
fn count(xs: &[String], name: &str) -> usize { xs.iter().filter(|x| x.as_str() == name).count() }

#[test]
fn pinned_sources_and_reward_contracts_are_unchanged() {
    let doc = include_str!("../docs/CHAPTERS_055_057_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in ["2568018", "2568019", "2568020", "2554719", "2554735",
                "2554799", "2554775", "2554811", "2554760"] {
        assert!(doc.contains(rev), "missing revision {rev}");
    }
    for phrase in ["甜言美語", "兩情歡洽", "身中數箭", "王朗", "钟繇",
                   "兩面紅旗", "麒麟阁", "三百餘口", "原刻影印", "0/120"] {
        assert!(doc.contains(phrase), "missing compared witness {phrase}");
    }
    assert_eq!(audit.lines().skip(1).filter(|row| !row.is_empty()).count(), 120);
    for ch in 55..=57 {
        assert!(audit.lines().any(|row| row.starts_with(&format!("{ch:03}\t"))
            && row.contains("CHAPTERS_055_057_JIAJING_VARIANTS.md")));
    }
    for (id, chapter, flag, gold, heal, anchor) in [
        ("nanxu_escape_witness",55,"nanxu_escape_witness_record",18,8,"嘉靖109"),
        ("tongque_dispatch_ledger",56,"tongque_dispatch_record",19,7,"嘉靖111"),
        ("jingzhou_false_passage",56,"jingzhou_false_passage_record",20,8,"嘉靖112"),
        ("mateng_case",57,"mateng_case_record",18,7,"麒麟阁"),
    ] {
        let b=beat(id).unwrap();
        assert_eq!((b.chapter,b.flag,b.reward_gold,b.reward_heal),
                   (chapter,flag,gold,heal));
        assert!(b.premise.contains(anchor), "{id} missing source boundary");
    }
}

#[test]
fn sun_nanxu_to_river_choice_is_atomic_persistent_and_once_only() {
    let mut g=Game::new(Hero::SunShangxiang,55057);
    g.lore_flags.push("jiangdong_covenant".into());
    enter(&mut g,"nanxu_escape_witness");
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"river_child_witness");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g.easter_unlocked,"nanxu_river_witness"),1);
    let mut restored=Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut restored,"river_child_witness");
    restored.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&restored.lore_flags,"river_child_witness_record"),1);
    assert_eq!(count(&restored.easter_unlocked,"nanxu_river_witness"),1);
}

#[test]
fn cao_mateng_evidence_and_zhuge_refusal_do_not_change_old_outcomes() {
    let mut cao=Game::new(Hero::CaoCao,56057);
    cao.lore_flags.extend(["guandu_grain".into(),"river_banquet_record".into()]);
    enter(&mut cao,"tongque_dispatch_ledger");
    cao.apply(Action::Choose{index:1}).unwrap();
    enter(&mut cao,"mateng_case");
    cao.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&cao.lore_flags,"mateng_case_record"),1);
    assert_eq!(count(&cao.easter_unlocked,"tongque_river_warning"),1);
    assert!(Game::from_json(&cao.to_json().unwrap()).is_ok());
    let mut zhuge=Game::new(Hero::ZhugeLiang,56058);
    zhuge.lore_flags.push("jingzhou_pledge_copy_record".into());
    enter(&mut zhuge,"jingzhou_false_passage");
    zhuge.apply(Action::Choose{index:2}).unwrap();
    assert_eq!(count(&zhuge.lore_flags,"jingzhou_false_passage_record"),0);
    assert!(Game::from_json(&zhuge.to_json().unwrap()).is_ok());
}
