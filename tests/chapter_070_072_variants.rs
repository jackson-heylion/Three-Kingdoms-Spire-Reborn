//! Fixed dual-electronic-edition local variants only; not full collation or natural map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action,Game,Stage};
fn enter(g:&mut Game,id:&str){g.stage=Stage::Story;g.lore_current=Some(id.into());}
fn count(xs:&[String],s:&str)->usize{xs.iter().filter(|x|x.as_str()==s).count()}
#[test] fn fixed_revision_and_playable_sources(){
 let d=include_str!("../docs/CHAPTERS_070_072_JIAJING_VARIANTS.md");
 let a=include_str!("../data/chapter_body_audit.tsv");
 for id in ["2568033","2568034","2568036","7908281","2554727","2554729","7911948","2554766","2554820"]{assert!(d.contains(id),"{id}");}
 for term in ["三十餘合","二十餘合","建安二十三年","建安二十二年","白旗","紅旗","忽一人報","黃忠謂趙雲","0/120","原刻影印"]{assert!(d.contains(term),"{term}");}
 assert_eq!(a.lines().skip(1).filter(|s|!s.is_empty()).count(),120);
 for n in 70..=72{assert!(a.lines().any(|s|s.starts_with(&format!("{n:03}\t"))&&s.contains("CHAPTERS_070_072_JIAJING_VARIANTS.md")));}
 for (id,ch,pre,flag,gold,heal,anchor) in [
 ("wakou_civilian_route",70,"chengdu_land_record","wakou_civilian_record",20,9,"三十余合"),
 ("dingjun_signal_register",71,"wakou_civilian_record","dingjun_signal_record",21,9,"嘉靖141"),
 ("hanshui_retreat_roll",71,"xudu_fire_record","hanshui_retreat_record",20,8,"二十三年"),
 ("hanzhong_route_review",72,"dingjun_signal_record","hanzhong_route_record",20,9,"不具名军报"),
 ("xiegu_ration_inquest",72,"hanshui_retreat_record","xiegu_ration_record",21,8,"嘉靖143")
 ]{
 let b=beat(id).unwrap();assert_eq!((b.chapter,b.prerequisite,b.flag),(ch,Some(pre),flag));assert_eq!((b.reward_gold,b.reward_heal),(gold,heal));assert!(b.premise.contains(anchor),"{id}");
 }
}
#[test] fn zhuge_wakou_dingjun_hanzhong_is_atomic_optional_and_idempotent(){
 let mut g=Game::new(Hero::ZhugeLiang,70072);g.lore_flags.push("chengdu_land_record".into());
 for id in ["wakou_civilian_route","dingjun_signal_register","hanzhong_route_review"]{
 enter(&mut g,id);let before=g.to_json().unwrap();assert!(g.apply(Action::Choose{index:99}).is_err());assert_eq!(g.to_json().unwrap(),before);
 g.apply(Action::Choose{index:1}).unwrap();
 }
 for flag in ["wakou_civilian_record","dingjun_signal_record","hanzhong_route_record"]{assert_eq!(count(&g.lore_flags,flag),1);}
 assert_eq!(count(&g.easter_unlocked,"wakou_dingjun_debt"),1);
 let saved=g.to_json().unwrap();let mut restored=Game::from_json(&saved).unwrap();assert_eq!(restored.to_json().unwrap(),saved);
 enter(&mut restored,"dingjun_signal_register");restored.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&restored.easter_unlocked,"wakou_dingjun_debt"),1);assert_eq!(count(&restored.lore_flags,"dingjun_signal_record"),1);
}
#[test] fn refusal_and_cao_command_archive_do_not_cross_heroes(){
 let mut z=Game::new(Hero::ZhugeLiang,70073);z.lore_flags.push("chengdu_land_record".into());
 enter(&mut z,"wakou_civilian_route");z.apply(Action::Choose{index:2}).unwrap();
 assert_eq!(count(&z.lore_flags,"wakou_civilian_record"),0);assert_eq!(count(&z.easter_unlocked,"wakou_dingjun_debt"),0);
 let mut c=Game::new(Hero::CaoCao,71072);c.lore_flags.push("xudu_fire_record".into());
 for id in ["hanshui_retreat_roll","xiegu_ration_inquest"]{enter(&mut c,id);c.apply(Action::Choose{index:1}).unwrap();}
 assert_eq!(count(&c.lore_flags,"hanshui_retreat_record"),1);assert_eq!(count(&c.lore_flags,"xiegu_ration_record"),1);assert_eq!(count(&c.lore_flags,"dingjun_signal_record"),0);
}
