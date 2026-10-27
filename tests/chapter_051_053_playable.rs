//! Chapter 051-053 provenance, optional decisions and old-save safety.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};
fn at(hero:Hero,id:&str,flags:&[&str])->Game {
    let mut g=Game::new(hero,5153);g.stage=Stage::Story;g.lore_current=Some(id.into());
    g.lore_flags=flags.iter().map(|s|(*s).into()).collect();g
}
fn enter(g:&mut Game,id:&str){g.stage=Stage::Story;g.lore_current=Some(id.into());}
fn has(g:&Game,f:&str)->bool{g.lore_flags.iter().any(|x|x==f)}
fn count(g:&Game,id:&str)->usize{g.easter_unlocked.iter().filter(|x|x.as_str()==id).count()}
#[test]
fn fixed_readings_and_history_are_pinned() {
    for (n,rev,doc) in [(51,"2568014",include_str!("../docs/CHAPTER_051_READING.md")),(52,"2568016",include_str!("../docs/CHAPTER_052_READING.md")),(53,"2568015",include_str!("../docs/CHAPTER_053_READING.md"))] {
        assert!(doc.contains(&format!("oldid={rev}"))&&doc.contains("单底本"),"{n}");
    }
    let h=include_str!("../docs/CHAPTERS_051_053_HISTORY_SPOTCHECK.md");
    for rev in ["2387393","2583383","2583390","2075209"]{assert!(h.contains(rev));}
    assert!(h.contains("建安十一年"));
    for (id,word) in [("nanjun_claim_ledger","周瑜"),("jingxiang_lease_ledger","刘琦"),("changsha_mercy","魏延"),("hefei_memorial","太史慈")] {
        assert!(beat(id).unwrap().premise.contains(word));
    }
}
#[test]
fn zhuge_reward_is_atomic_persistent_and_once_only() {
    let mut g=at(Hero::ZhugeLiang,"nanjun_claim_ledger",&["jiangdong_council_record"]);
    let before=g.to_json().unwrap();assert!(g.apply(Action::Choose{index:4}).is_err());
    assert_eq!(g.to_json().unwrap(),before);g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"jingxiang_lease_ledger");g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"jingxiang_lease_record"));assert_eq!(count(&g,"nanjun_jingxiang_debt"),1);
    let saved=g.to_json().unwrap();let mut g=Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(),saved);enter(&mut g,"jingxiang_lease_ledger");
    g.apply(Action::Choose{index:1}).unwrap();assert_eq!(count(&g,"nanjun_jingxiang_debt"),1);
}
#[test]
fn refusal_and_optional_scenes_preserve_legacy_routes() {
    for choice in [2,3] {
        let mut g=at(Hero::ZhugeLiang,"nanjun_claim_ledger",&["jiangdong_council_record","redcliff_wind"]);
        g.apply(Action::Choose{index:choice}).unwrap();assert!(!has(&g,"nanjun_claim_record"));
        assert_eq!(next_beat(Hero::ZhugeLiang,2,&g.lore_flags,&g.lore_seen).map(|b|b.id),Some("wuzhang"));
    }
    let g=at(Hero::GuanYu,"changsha_mercy",&["huarong_mercy","anxi_seal"]);
    assert_eq!(next_beat(Hero::GuanYu,2,&g.lore_flags,&[]).map(|b|b.id),Some("jingzhou"));
}
#[test]
fn changsha_and_hefei_cross_act_rewards_are_idempotent() {
    let mut g=at(Hero::GuanYu,"changsha_mercy",&["huarong_mercy","anxi_seal"]);
    g.apply(Action::Choose{index:1}).unwrap();assert_eq!(count(&g,"huarong_changsha_mercy"),0);
    enter(&mut g,"jingzhou");g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"huarong_changsha_mercy"),1);
    let mut s=at(Hero::SunShangxiang,"hefei_memorial",&["jiangdong_muster_record","jiangdong_covenant"]);
    assert_eq!(next_beat(Hero::SunShangxiang,2,&s.lore_flags,&[]).map(|b|b.id),Some("yiling"));
    let before=s.to_json().unwrap();assert!(s.apply(Action::Choose{index:4}).is_err());
    assert_eq!(s.to_json().unwrap(),before);s.apply(Action::Choose{index:1}).unwrap();
    enter(&mut s,"yiling");s.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&s,"hefei_yiling_memorial"),1);
    let saved=s.to_json().unwrap();let mut s=Game::from_json(&saved).unwrap();
    enter(&mut s,"hefei_memorial");s.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&s,"hefei_yiling_memorial"),1);
}
