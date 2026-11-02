//! 043-045 independent Mao fixed-revision reread; no full Jiajing or historical collation.
use three_kingdoms_spire_reborn::{Action, Game, Stage, content::Hero, lore::beat};
fn enter(g:&mut Game,id:&str){g.stage=Stage::Story;g.lore_current=Some(id.into());}
fn count(xs:&[String],flag:&str)->usize{xs.iter().filter(|x|x.as_str()==flag).count()}

#[test]
fn pinned_three_chapter_reread_and_responsibility_boundary(){
 let doc=include_str!("../docs/CHAPTERS_043_045_JIAJING_VARIANTS.md");
 let ledger=include_str!("../data/chapter_body_audit.tsv");
 assert_eq!(ledger.lines().skip(1).filter(|s|!s.is_empty()).count(),120);
 for (ch,oldid,anchor) in [
  ("043","2568006","鲁肃先于孔明"),
  ("044","2568008","授权者孙权"),
  ("045","2568007","伪书是周瑜设局")
 ] {
  let rows:Vec<_>=ledger.lines().filter(|l|l.starts_with(&format!("{ch}\t"))).collect();
  assert_eq!(rows.len(),1);
  assert!(rows[0].contains(&format!("2026-11-02独立复核毛评固定oldid={oldid}")));
  assert!(doc.contains(&format!("oldid={oldid}")));
  assert!(doc.contains(anchor));
 }
 for (id,chapter,flag) in [
  ("chaisang_envoy",43,"chaisang_envoy_record"),
  ("jiangdong_council",44,"jiangdong_council_record"),
  ("jiangdong_muster",44,"jiangdong_muster_record"),
  ("jiangkou_inquest",45,"jiangkou_inquest_record")
 ] {
  let b=beat(id).unwrap();
  assert_eq!((b.chapter,b.flag),(chapter,flag));
 }
 let b=beat("jiangkou_inquest").unwrap();
 for phrase in ["蒋干主动窃书","曹操未经核实","鲁肃明确反对","046才发生"] {
  assert!(b.premise.contains(phrase),"{phrase}");
 }
}

#[test]
fn jiangkou_to_chain_ship_is_optional_atomic_once_only_and_v2_safe(){
 let mut g=Game::new(Hero::CaoCao,430450);
 g.lore_flags.push("guandu_grain".into());
 enter(&mut g,"jiangkou_inquest");
 let before=g.to_json().unwrap();
 assert!(g.apply(Action::Choose{index:99}).is_err());
 assert_eq!(g.to_json().unwrap(),before);
 g.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&g.lore_flags,"jiangkou_inquest_record"),1);
 let mut g=Game::from_json(&g.to_json().unwrap()).unwrap();
 enter(&mut g,"chain_ship_ledger");
 g.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&g.lore_flags,"chain_ship_risk_record"),1);
 let mut g=Game::from_json(&g.to_json().unwrap()).unwrap();
 enter(&mut g,"chain_ship_ledger");
 g.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&g.lore_flags,"chain_ship_risk_record"),1);
 let mut refused=Game::new(Hero::CaoCao,430451);
 refused.lore_flags.push("guandu_grain".into());
 enter(&mut refused,"jiangkou_inquest");
 refused.apply(Action::Choose{index:2}).unwrap();
 let mut refused=Game::from_json(&refused.to_json().unwrap()).unwrap();
 assert_eq!(count(&refused.lore_flags,"jiangkou_inquest_record"),0);
 enter(&mut refused,"liaodong");
 refused.apply(Action::Choose{index:1}).unwrap();
 assert_eq!(count(&refused.lore_flags,"liaodong_letter"),1);
}
