//! 069–071: fixed sources, atomic choice, V2, mutual exclusion and legacy priority.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
fn at(hero:Hero,id:&str,flags:&[&str])->Game {
    let mut g=Game::new(hero,6971); g.stage=Stage::Story;
    g.lore_current=Some(id.into()); g.lore_flags=flags.iter().map(|s|(*s).into()).collect(); g
}
fn enter(g:&mut Game,id:&str){g.stage=Stage::Story;g.lore_current=Some(id.into());}
fn count(g:&Game,id:&str)->usize{g.easter_unlocked.iter().filter(|s|s.as_str()==id).count()}
#[test]
fn source_revisions_and_scene_chapters() {
    for (n,rev,doc) in [(69,"7907113",include_str!("../docs/CHAPTER_069_READING.md")),(70,"2568033",include_str!("../docs/CHAPTER_070_READING.md")),(71,"2568034",include_str!("../docs/CHAPTER_071_READING.md"))] {
        assert!(doc.contains(&format!("oldid={rev}"))&&doc.contains("单底本"),"{n}");
    }
    let h=include_str!("../docs/CHAPTERS_069_071_HISTORY_SPOTCHECK.md");
    for rev in ["10048966","2583383","2274283"] {assert!(h.contains(rev));}
    assert!(h.contains("裴松之")&&h.contains("0/120"));
    for (id,n,pre) in [("xudu_fire_inquest",69,"zuoci_court_record"),("wakou_civilian_route",70,"chengdu_land_record"),("dingjun_signal_register",71,"wakou_civilian_record"),("hanshui_retreat_roll",71,"xudu_fire_record")] {
        let b=beat(id).unwrap();assert_eq!(b.chapter,n);assert_eq!(b.prerequisite,Some(pre));
    }
}
#[test]
fn cao_atomic_exclusive_v2_once_only() {
    let mut g=at(Hero::CaoCao,"xudu_fire_inquest",&["zuoci_court_record"]);
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"hanshui_retreat_roll");g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"xudu_hanshui_debt"),1);
    let save=g.to_json().unwrap();let mut g=Game::from_json(&save).unwrap();
    assert_eq!(g.to_json().unwrap(),save);
    enter(&mut g,"hanshui_retreat_roll");g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"xudu_hanshui_debt"),1);
    let mut no=at(Hero::CaoCao,"xudu_fire_inquest",&["zuoci_court_record"]);
    no.apply(Action::Choose{index:2}).unwrap();
    enter(&mut no,"hanshui_retreat_roll");no.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&no,"xudu_hanshui_debt"),0);
}
#[test]
fn zhuge_chain_requires_both_flags_and_is_idempotent() {
    let mut g=at(Hero::ZhugeLiang,"wakou_civilian_route",&["chengdu_land_record"]);
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"wakou_dingjun_debt"),0);
    enter(&mut g,"dingjun_signal_register");g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&g,"wakou_dingjun_debt"),1);
    let mut restored=Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut restored,"dingjun_signal_register");restored.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&restored,"wakou_dingjun_debt"),1);
    let mut no=at(Hero::ZhugeLiang,"wakou_civilian_route",&["chengdu_land_record"]);
    no.apply(Action::Choose{index:2}).unwrap();
    enter(&mut no,"dingjun_signal_register");no.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&no,"wakou_dingjun_debt"),0);
}
#[test]
fn old_routes_win_but_optional_records_are_still_selectable() {
    for (hero,flags,old,new) in [(Hero::CaoCao,vec!["guandu_grain","zuoci_court_record"],"liaodong","xudu_fire_inquest"),(Hero::ZhugeLiang,vec!["redcliff_wind","chengdu_land_record"],"wuzhang","wakou_civilian_route")] {
        let flags:Vec<String>=flags.into_iter().map(str::to_owned).collect();
        assert_eq!(next_beat(hero,2,&flags,&[]).map(|b|b.id),Some(old));
        let seen:Vec<String>=BEATS.iter().filter(|b|b.id!=new).map(|b|b.id.to_owned()).collect();
        assert_eq!(next_beat(hero,2,&flags,&seen).map(|b|b.id),Some(new));
    }
}
