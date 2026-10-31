//! 040–042: pinned local second-witness differences, real choices and old-save compatibility.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};
fn enter(g:&mut Game,id:&str){g.stage=Stage::Story;g.lore_current=Some(id.into());}
fn count(xs:&[String],s:&str)->usize{xs.iter().filter(|x|x.as_str()==s).count()}
#[test]
fn pinned_two_witnesses_and_existing_story_ids_are_preserved(){
 let d=include_str!("../docs/CHAPTERS_040_042_JIAJING_VARIANTS.md");
 let a=include_str!("../data/chapter_body_audit.tsv");
 for rev in ["2568003","2568004","2568005","2554772","2554722","2554768","2554769","2554813","2554724"]{assert!(d.contains(rev),"missing {rev}");}
 for anchor in ["孔融","劉琦","不十余合","袖傳一課","夏侯霸","夏侯傑","裴松之","原刻影印"]{assert!(d.contains(anchor),"missing {anchor}");}
 assert_eq!(a.lines().skip(1).filter(|x|!x.is_empty()).count(),120);
 for n in 40..=42{assert!(a.lines().any(|s|s.starts_with(&format!("{n:03}\\t").replace("\\t","\t"))&&s.contains("CHAPTERS_040_042_JIAJING_VARIANTS.md")),"chapter {n}");}
 for (id,ch,pre,flag,anchor) in [
 ("xinye_evacuation",40,"bowang_orders_record","xinye_civilians","嘉靖080"),
 ("dangyang_ledger",41,"xinye_civilians","dangyang_refuge_account","嘉靖082"),
 ("hanjin_ferry",42,"xutian_guarded_emperor","hanjin_ferry_record","夏侯霸")
 ]{
  let b=beat(id).unwrap();assert_eq!((b.chapter,b.prerequisite,b.flag),(ch,Some(pre),flag));assert!(b.premise.contains(anchor),"{id}");
 }
}
#[test]
fn zhuge_civilian_debt_is_atomic_optional_and_once_only(){
 let mut g=Game::new(Hero::ZhugeLiang,40042);
 g.lore_flags.push("bowang_orders_record".into());
 for id in ["xinye_evacuation","dangyang_ledger"]{
  enter(&mut g,id);let before=g.to_json().unwrap();
  assert!(g.apply(Action::Choose{index:99}).is_err());
  assert_eq!(g.to_json().unwrap(),before);
  g.apply(Action::Choose{index:1}).unwrap();
 }
 assert_eq!(count(&g.lore_flags,"xinye_civilians"),1);
 assert_eq!(count(&g.lore_flags,"dangyang_refuge_account"),1);
 // The four-flag conclusion pays only after the later Red Cliffs wind record.
 assert_eq!(count(&g.easter_unlocked,"xinye_dangyang_debt"),0);
 enter(&mut g,"redcliff");g.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&g.lore_flags,"redcliff_wind"),1);
 assert_eq!(count(&g.easter_unlocked,"xinye_dangyang_debt"),1);
 let saved=g.to_json().unwrap();let mut restored=Game::from_json(&saved).unwrap();
 assert_eq!(restored.to_json().unwrap(),saved);
 enter(&mut restored,"dangyang_ledger");restored.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&restored.easter_unlocked,"xinye_dangyang_debt"),1);
}
#[test]
fn refusal_and_legacy_hanjin_huarong_remain_playable(){
 let mut z=Game::new(Hero::ZhugeLiang,40043);z.lore_flags.push("bowang_orders_record".into());
 enter(&mut z,"xinye_evacuation");z.apply(Action::Choose{index:2}).unwrap();
 assert_eq!(count(&z.lore_flags,"xinye_civilians"),0);
 enter(&mut z,"redcliff");z.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&z.easter_unlocked,"xinye_dangyang_debt"),0);
 let mut g=Game::new(Hero::GuanYu,42043);g.lore_flags.push("xutian_guarded_emperor".into());
 enter(&mut g,"hanjin_ferry");g.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&g.lore_flags,"hanjin_ferry_record"),1);
 let mut g=Game::from_json(&g.to_json().unwrap()).unwrap();
 enter(&mut g,"huarong");g.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&g.easter_unlocked,"hanjin_huarong_debt"),1);
 enter(&mut g,"huarong");g.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&g.easter_unlocked,"hanjin_huarong_debt"),1);
 let mut old=Game::new(Hero::GuanYu,42044);enter(&mut old,"huarong");
 old.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&old.easter_unlocked,"hanjin_huarong_debt"),0);
}
