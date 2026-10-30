//! P1④ fixed electronic local variants, not full collation or natural-map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action,Game,Stage};
fn enter(g:&mut Game,id:&str){g.stage=Stage::Story;g.lore_current=Some(id.into());}
fn count(xs:&[String],s:&str)->usize{xs.iter().filter(|x|x.as_str()==s).count()}
#[test] fn fixed_sources_and_existing_playable_contracts(){
 let d=include_str!("../docs/CHAPTERS_073_075_JIAJING_VARIANTS.md");
 let a=include_str!("../data/chapter_body_audit.tsv");
 for id in ["2568035","2568037","2568039","2554728","2554815","2554816","2554764","2554730","2554765"]{assert!(d.contains(id),"{id}");}
 for s in ["一百二十人","八萬七千五百","斬白馬","夷其三族","定按軍法","原刻","0/120","史料"]{assert!(d.contains(s),"{s}");}
 assert_eq!(a.lines().skip(1).filter(|s|!s.is_empty()).count(),120);
 for n in 73..=75{assert!(a.lines().any(|s|s.starts_with(&format!("{n:03}\t"))&&s.contains("CHAPTERS_073_075_JIAJING_VARIANTS.md")));}
 for (id,ch,pre,flag,gold,heal,anchor) in [
 ("hanzhong_king_memorial",73,"hanzhong_route_record","hanzhong_king_record",20,8,"嘉靖"),
 ("jingxiang_border_warning",73,"lukou_terms_record","jingxiang_border_record",19,8,"毛评"),
 ("fancheng_flood_witness",74,"jingxiang_border_record","fancheng_flood_record",20,10,"八万七千五百"),
 ("pangde_yujin_ledger",74,"xiegu_ration_record","pangde_yujin_record",20,8,"八万七千五百"),
 ("blade_wound_watch",75,"fancheng_flood_record","blade_wound_record",19,9,"嘉靖"),
 ("white_cloak_signal_audit",75,"ruxu_rescue_record","white_cloak_signal_record",20,8,"夷其三族")]{
 let b=beat(id).unwrap();assert_eq!((b.chapter,b.prerequisite,b.flag),(ch,Some(pre),flag));
 assert_eq!((b.reward_gold,b.reward_heal),(gold,heal));assert!(b.premise.contains(anchor),"{id}");
 }}
#[test] fn guan_cross_chapter_reward_is_mutually_exclusive_atomic_and_idempotent(){
 let mut g=Game::new(Hero::GuanYu,73075);
 for s in ["anxi_seal","lukou_terms_record"]{g.lore_flags.push(s.into());}
 for id in ["jingxiang_border_warning","fancheng_flood_witness","blade_wound_watch","maicheng_relief_review"]{
 enter(&mut g,id);let before=g.to_json().unwrap();assert!(g.apply(Action::Choose{index:99}).is_err());
 assert_eq!(g.to_json().unwrap(),before);g.apply(Action::Choose{index:1}).unwrap();}
 for s in ["jingxiang_border_record","fancheng_flood_record","blade_wound_record","maicheng_relief_record"]{assert_eq!(count(&g.lore_flags,s),1);}
 assert_eq!(count(&g.easter_unlocked,"jingzhou_maicheng_courier_debt"),1);
 let saved=g.to_json().unwrap();let mut restored=Game::from_json(&saved).unwrap();assert_eq!(restored.to_json().unwrap(),saved);
 enter(&mut restored,"maicheng_relief_review");restored.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&restored.easter_unlocked,"jingzhou_maicheng_courier_debt"),1);
 assert_eq!(count(&restored.lore_flags,"maicheng_relief_record"),1);
}
#[test] fn refusal_and_cao_archive_remain_separate(){
 let mut guan=Game::new(Hero::GuanYu,73076);guan.lore_flags.push("lukou_terms_record".into());
 enter(&mut guan,"jingxiang_border_warning");guan.apply(Action::Choose{index:2}).unwrap();
 assert_eq!(count(&guan.lore_flags,"jingxiang_border_record"),0);
 let mut cao=Game::new(Hero::CaoCao,74075);cao.lore_flags.push("hanshui_retreat_record".into());
 for id in ["xiegu_ration_inquest","pangde_yujin_ledger"]{enter(&mut cao,id);cao.apply(Action::Choose{index:1}).unwrap();}
 assert_eq!(count(&cao.lore_flags,"pangde_yujin_record"),1);
 assert_eq!(count(&cao.lore_flags,"fancheng_flood_record"),0);
}
