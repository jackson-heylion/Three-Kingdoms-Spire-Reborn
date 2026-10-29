//! Fixed JiaJing/Mao spotchecks, actual game choices and V2 roundtrip for 097-099.
//! Scripted Story entry is not evidence of natural map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_string());
}
fn count(game: &Game, flag: &str) -> usize {
    game.lore_flags.iter().filter(|f| f.as_str() == flag).count()
}
#[test]
fn ch097_099_fixed_sources_and_playable_contracts() {
    let doc = include_str!("../docs/CHAPTERS_097_099_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for id in ["2568059","2568061","2568060","2555887","2555888","2555889","7908288","2555892","2555896"] {
        assert!(doc.contains(id), "missing fixed source {id}");
    }
    for token in ["孫策","孫權","霍昭","郝昭","連珠炮","楊震","陳震","五萬","五千"] {
        assert!(doc.contains(token), "missing anchored variant {token}");
    }
    for (chapter, id, flag, pre, gold, heal) in [
        (97,"second_memorial_supply_review","second_memorial_supply_record","chushi_supply_record",25,10),
        (97,"chencang_siege_wounded_ledger","chencang_siege_record","redcliff_wind",23,12),
        (98,"chencang_retreat_supply_audit","chencang_retreat_supply_record","chencang_siege_record",25,12),
        (98,"wu_han_enthronement_dispatch","wu_han_enthronement_record","jiangdong_covenant",22,8),
        (99,"wudu_yinping_civil_register","wudu_yinping_civil_record","redcliff_wind",22,9),
    ] {
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter:03}\\t"))).unwrap();
        assert!(row.contains("CHAPTERS_097_099_JIAJING_VARIANTS.md"));
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.flag, flag);
        assert_eq!(b.prerequisite, Some(pre));
        assert_eq!((b.reward_gold,b.reward_heal),(gold,heal));
        assert!(b.premise.contains("嘉靖") && b.premise.contains("毛评"), "{id}");
    }
}
#[test]
fn ch097_099_shu_logistics_to_lucheng_v2_is_atomic_and_mutually_exclusive() {
    let mut g = Game::new(Hero::ZhugeLiang, 97099);
    g.lore_flags.extend(["chushi_supply_record","redcliff_wind"].map(str::to_owned));
    enter(&mut g,"second_memorial_supply_review");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"second_memorial_supply_record"),1);
    enter(&mut g,"chencang_siege_wounded_ledger");
    g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"chencang_retreat_supply_audit");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"chencang_retreat_supply_record"),1);
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(),saved);
    enter(&mut g,"wudu_yinping_civil_register");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"wudu_yinping_civil_record"),1);
    enter(&mut g,"lucheng_wheat_household");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"lucheng_wheat_rotation_record"),1);
    assert_eq!(Game::from_json(&g.to_json().unwrap()).unwrap().to_json().unwrap(),g.to_json().unwrap());

    let mut bargain = Game::new(Hero::ZhugeLiang, 97099);
    bargain.lore_flags.push("redcliff_wind".to_owned());
    enter(&mut bargain,"chencang_siege_wounded_ledger");
    bargain.apply(Action::Choose{index:2}).unwrap();
    assert_eq!(count(&bargain,"chencang_siege_record"),0);
    assert_eq!(count(&bargain,"chencang_retreat_supply_record"),0);
}
#[test]
fn ch098_wu_envoy_choice_restores_without_creating_second_mission() {
    let mut g = Game::new(Hero::SunShangxiang, 98099);
    g.lore_flags.push("jiangdong_covenant".into());
    enter(&mut g,"wu_han_enthronement_dispatch");
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"wu_han_enthronement_record"),1);
    let saved=g.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(),saved);
}
