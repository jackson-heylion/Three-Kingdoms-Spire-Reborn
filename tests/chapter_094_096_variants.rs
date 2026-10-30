//! 094-096 fixed volume-level textual evidence and actual Story choice chains.
//! Scripted Story entry does not prove natural campaign reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn flag_count(g: &Game, flag: &str) -> usize {
    g.lore_flags.iter().filter(|x| x.as_str() == flag).count()
}
fn egg_count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|x| x.as_str() == id).count()
}
#[test]
fn fixed_volume_sources_are_distinct_from_displaced_section_190() {
    let doc = include_str!("../docs/CHAPTERS_094_096_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for id in ["2568057","2568058","2568062","2475479","2475422","2555875","2555879","2555892"] {
        assert!(doc.contains(id), "missing fixed source {id}");
    }
    for term in ["華陽巾","綸巾","拔西縣","初拔四縣","西城","王平","月米","周鲂","分则190"] {
        assert!(doc.contains(term), "missing textual distinction {term}");
    }
    assert_eq!(audit.lines().skip(1).filter(|line| !line.is_empty()).count(),120);
    for chapter in 94..=96 {
        let row = audit.lines().find(|line| line.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("CHAPTERS_094_096_JIAJING_VARIANTS.md"));
        assert!(row.contains("局部"));
    }
    for (id,chapter,flag,pre,gold,heal) in [
        ("qiang_captive_return",94,"qiang_captive_record","redcliff_wind",24,9),
        ("xincheng_secret_dispatch",94,"xincheng_secret_record","guandu_grain",22,8),
        ("jieting_water_order",95,"jieting_water_record","redcliff_wind",23,12),
        ("xicheng_civilian_retreat",95,"xicheng_civilian_record","jieting_water_record",26,12),
        ("masu_family_account",96,"masu_family_record","jieting_water_record",24,11),
        ("shiting_false_defection_audit",96,"shiting_spy_record","jiangdong_covenant",22,9),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter,chapter);
        assert_eq!(b.flag,flag);
        assert_eq!(b.prerequisite,Some(pre));
        assert_eq!((b.reward_gold,b.reward_heal),(gold,heal));
        assert!(b.premise.contains("嘉靖") && b.premise.contains("毛评"), "{id}");
    }
}
#[test]
fn street_water_to_masu_family_cross_chapter_eggs_are_once_after_v2_restore() {
    let mut g = Game::new(Hero::ZhugeLiang,94096);
    g.lore_flags.push("redcliff_wind".into());
    enter(&mut g,"qiang_captive_return");
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"jieting_water_order");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(egg_count(&g,"redcliff_jieting_supply_debt"),1);
    enter(&mut g,"xicheng_civilian_retreat");
    g.apply(Action::Choose{index:1}).unwrap();
    let saved=g.to_json().unwrap();
    let mut g=Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(),saved);
    enter(&mut g,"masu_family_account");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(flag_count(&g,"masu_family_record"),1);
    assert_eq!(egg_count(&g,"jieting_masu_family_debt"),1);
    enter(&mut g,"masu_family_account");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(flag_count(&g,"masu_family_record"),1);
    assert_eq!(egg_count(&g,"jieting_masu_family_debt"),1);
    assert_eq!(egg_count(&g,"redcliff_jieting_supply_debt"),1);
    assert_eq!(Game::from_json(&g.to_json().unwrap()).unwrap().to_json().unwrap(),g.to_json().unwrap());
}
#[test]
fn rejecting_water_record_cannot_unlock_family_chain() {
    let mut g=Game::new(Hero::ZhugeLiang,95096);
    g.lore_flags.push("redcliff_wind".into());
    enter(&mut g,"jieting_water_order");
    g.apply(Action::Choose{index:2}).unwrap();
    assert_eq!(flag_count(&g,"jieting_water_record"),0);
    assert_eq!(egg_count(&g,"redcliff_jieting_supply_debt"),0);
    enter(&mut g,"masu_family_account");
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(egg_count(&g,"jieting_masu_family_debt"),0);
}
#[test]
fn wei_secret_and_wu_shiting_choices_preserve_their_own_flags() {
    let mut wei=Game::new(Hero::CaoCao,94096);
    wei.lore_flags.push("guandu_grain".into());
    enter(&mut wei,"xincheng_secret_dispatch");
    wei.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(flag_count(&wei,"xincheng_secret_record"),1);
    let mut wu=Game::new(Hero::SunShangxiang,96094);
    wu.lore_flags.push("jiangdong_covenant".into());
    enter(&mut wu,"shiting_false_defection_audit");
    wu.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(flag_count(&wu,"shiting_spy_record"),1);
    assert_eq!(Game::from_json(&wu.to_json().unwrap()).unwrap().to_json().unwrap(),wu.to_json().unwrap());
}
