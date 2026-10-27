//! Chapters 075–077: fixed sources, atomic refusal, cross-act rewards and old V2 priority.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 7577);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_editions_and_hero_prerequisites_are_explicit() {
    for (n,rev,doc) in [
        (75,"2568039",include_str!("../docs/CHAPTER_075_READING.md")),
        (76,"2568038",include_str!("../docs/CHAPTER_076_READING.md")),
        (77,"2568041",include_str!("../docs/CHAPTER_077_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={rev}")) && doc.contains("单底本"),"{n}");
        assert!(doc.contains("回末"),"{n}");
    }
    let history=include_str!("../docs/CHAPTERS_075_077_HISTORY_SPOTCHECK.md");
    for rev in ["2583383","2387393","10048966"] { assert!(history.contains(rev)); }
    assert!(history.contains("陈寿") && history.contains("裴松之") && history.contains("0/120"));
    for (id,ch,pre) in [
        ("blade_wound_watch",75,"fancheng_flood_record"),
        ("white_cloak_signal_audit",75,"ruxu_rescue_record"),
        ("maicheng_relief_review",76,"blade_wound_record"),
        ("jingzhou_aftermath_courier",77,"white_cloak_signal_record"),
        ("luoyang_head_rites",77,"pangde_yujin_record"),
        ("chengdu_mourning_register",77,"hanzhong_king_record"),
    ] {
        let b=beat(id).unwrap();
        assert_eq!(b.chapter,ch);
        assert_eq!(b.prerequisite,Some(pre));
    }
}

#[test]
fn guan_chapter_debt_is_atomic_exclusive_idempotent_and_restorable() {
    let flags=["anxi_seal","jingxiang_border_record","fancheng_flood_record"];
    let mut g=scene(Hero::GuanYu,"blade_wound_watch",&flags);
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g,"maicheng_relief_review");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g,"jingzhou_maicheng_courier_debt"),1);
    let saved=g.to_json().unwrap();
    let mut restored=Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(),saved);
    enter(&mut restored,"maicheng_relief_review");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored,"jingzhou_maicheng_courier_debt"),1);
    let mut refused=scene(Hero::GuanYu,"blade_wound_watch",&flags);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused,"maicheng_relief_review");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused,"jingzhou_maicheng_courier_debt"),0);
}

#[test]
fn sun_cao_zhuge_payoffs_require_complete_flag_chains() {
    let mut sun=scene(Hero::SunShangxiang,"white_cloak_signal_audit",&["ruxu_rescue_record","yiling_ashes"]);
    sun.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut sun,"jingzhou_aftermath_courier");
    sun.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&sun,"jiangdong_jingzhou_aftermath_debt"),1);
    let mut cao=scene(Hero::CaoCao,"luoyang_head_rites",&["guandu_grain","pangde_yujin_record"]);
    cao.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&cao,"guandu_luoyang_rites_debt"),1);
    let mut zhuge=scene(Hero::ZhugeLiang,"chengdu_mourning_register",&["redcliff_wind","hanzhong_king_record"]);
    zhuge.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&zhuge,"redcliff_chengdu_mourning_debt"),1);
}

#[test]
fn legacy_story_routes_are_not_stolen_by_optional_new_nodes() {
    for (hero,flags,old,new) in [
        (Hero::GuanYu,vec!["anxi_seal","jingzhou_warning","fancheng_flood_record"],"maicheng","blade_wound_watch"),
        (Hero::SunShangxiang,vec!["jiangdong_covenant","ruxu_rescue_record"],"yiling","white_cloak_signal_audit"),
        (Hero::CaoCao,vec!["guandu_grain","pangde_yujin_record"],"liaodong","luoyang_head_rites"),
        (Hero::ZhugeLiang,vec!["redcliff_wind","hanzhong_king_record"],"wuzhang","chengdu_mourning_register"),
    ] {
        let flags:Vec<String>=flags.into_iter().map(str::to_owned).collect();
        assert_eq!(next_beat(hero,2,&flags,&[]).map(|b|b.id),Some(old));
        let seen:Vec<String>=BEATS.iter().filter(|b|b.id!=new).map(|b|b.id.to_owned()).collect();
        assert_eq!(next_beat(hero,2,&flags,&seen).map(|b|b.id),Some(new));
    }
}
