//! 063–065: provenance, mutually exclusive choice, atomicity, V2 and legacy priority.
use three_kingdoms_spire_reborn::{Action,Game,Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat,next_beat,BEATS};
fn at(hero:Hero,id:&str,flags:&[&str])->Game {
    let mut g=Game::new(hero,6365);g.stage=Stage::Story;g.lore_current=Some(id.into());
    g.lore_flags=flags.iter().map(|s|(*s).into()).collect();g
}
fn enter(g:&mut Game,id:&str){g.stage=Stage::Story;g.lore_current=Some(id.into());}
fn has(g:&Game,f:&str)->bool{g.lore_flags.iter().any(|s|s==f)}
fn count(g:&Game,id:&str)->usize{g.easter_unlocked.iter().filter(|s|s.as_str()==id).count()}
#[test]
fn sources_are_pinned_but_not_full_collations(){
    for (n,rev,doc) in [(63,"2568024",include_str!("../docs/CHAPTER_063_READING.md")),(64,"2568026",include_str!("../docs/CHAPTER_064_READING.md")),(65,"2568028",include_str!("../docs/CHAPTER_065_READING.md"))] {
        assert!(doc.contains(&format!("oldid={rev}"))&&doc.contains("单底本"),"{n}");
    }
    let h=include_str!("../docs/CHAPTERS_063_065_HISTORY_SPOTCHECK.md");
    for rev in ["2583384","2583383","2583378","2074129"]{assert!(h.contains(rev));}
    assert!(h.contains("裴注")&&h.contains("局部")&&h.contains("0/120"));
    for (id,n,pre) in [("luofeng_mourning",63,"fu_pass_record"),("jingzhou_handover",63,"hanjin_ferry_record"),("jinyan_bridge_review",64,"luofeng_mourning_record"),("chengdu_land_decree",65,"jinyan_bridge_record"),("ma_chao_letter",65,"jingzhou_handover_record")]{
        let b=beat(id).unwrap();assert_eq!(b.chapter,n);assert_eq!(b.prerequisite,Some(pre));
    }
}
#[test]
fn zhuge_chain_is_atomic_saveable_mutually_exclusive_and_once_only(){
    let mut g=at(Hero::ZhugeLiang,"luofeng_mourning",&["fu_pass_record","redcliff_wind"]);
    let before=g.to_json().unwrap();assert!(g.apply(Action::Choose{index:99}).is_err());assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();assert!(has(&g,"luofeng_mourning_record"));
    let saved=g.to_json().unwrap();let mut g=Game::from_json(&saved).unwrap();assert_eq!(g.to_json().unwrap(),saved);
    enter(&mut g,"jinyan_bridge_review");g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"chengdu_land_decree");g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"luofeng_chengdu_reckoning"),1);
    enter(&mut g,"chengdu_land_decree");g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"luofeng_chengdu_reckoning"),1);
    let mut no=at(Hero::ZhugeLiang,"luofeng_mourning",&["fu_pass_record"]);
    no.apply(Action::Choose{index:2}).unwrap();assert!(!has(&no,"luofeng_mourning_record"));
    enter(&mut no,"jinyan_bridge_review");no.apply(Action::Choose{index:1}).unwrap();
    enter(&mut no,"chengdu_land_decree");no.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&no,"luofeng_chengdu_reckoning"),0);
}
#[test]
fn guan_old_jingzhou_and_maicheng_are_not_stolen(){
    let flags=["hanjin_ferry_record","anxi_seal","jingzhou_warning"];
    let owned:Vec<String>=flags.iter().map(|s|(*s).into()).collect();
    assert_eq!(next_beat(Hero::GuanYu,2,&owned,&[]).map(|b|b.id),Some("jingzhou"));
    assert_eq!(next_beat(Hero::GuanYu,2,&owned,&["jingzhou".into()]).map(|b|b.id),Some("maicheng"));
    let seen:Vec<String>=BEATS.iter().filter(|b|b.id!="jingzhou_handover").map(|b|b.id.into()).collect();
    assert_eq!(next_beat(Hero::GuanYu,2,&owned,&seen).map(|b|b.id),Some("jingzhou_handover"));
    let mut g=at(Hero::GuanYu,"jingzhou_handover",&flags);
    let before=g.to_json().unwrap();assert!(g.apply(Action::Choose{index:99}).is_err());assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();enter(&mut g,"ma_chao_letter");g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"jingzhou_guard_letter_debt"),1);
    let saved=g.to_json().unwrap();let mut g=Game::from_json(&saved).unwrap();
    enter(&mut g,"ma_chao_letter");g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"jingzhou_guard_letter_debt"),1);
    let mut no=at(Hero::GuanYu,"jingzhou_handover",&flags);no.apply(Action::Choose{index:2}).unwrap();
    assert!(!has(&no,"jingzhou_handover_record"));
    enter(&mut no,"ma_chao_letter");no.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&no,"jingzhou_guard_letter_debt"),0);
}
#[test]
fn new_story_can_follow_old_legacy_scene(){
    let flags=vec!["fu_pass_record".into(),"redcliff_wind".into()];
    assert_eq!(next_beat(Hero::ZhugeLiang,2,&flags,&[]).map(|b|b.id),Some("wuzhang"));
    let seen:Vec<String>=BEATS.iter().filter(|b|b.id!="luofeng_mourning").map(|b|b.id.into()).collect();
    assert_eq!(next_beat(Hero::ZhugeLiang,2,&flags,&seen).map(|b|b.id),Some("luofeng_mourning"));
}
