//! Chapters 100–102 fixed second-edition spotchecks and actual story state transitions.
//! Scripted Story entry is not proof of natural map reachability.
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

#[test]
fn fixed_ch100_102_variants_preserve_story_contracts() {
    let doc = include_str!("../docs/CHAPTERS_100_102_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2568063","2568067","2568065","2555897","2555901","2555906","2558091","2558088","2558087"] {
        assert!(doc.contains(oldid), "missing fixed revision {oldid}");
    }
    for token in ["魏延", "陳式之子陳壽", "杖五十", "杖一百", "群行者二十里",
        "群行者三十里", "二寸二分", "二寸五分", "黄氏", "三千", "二千"] {
        assert!(doc.contains(token), "missing anchored variant {token}");
    }
    for (ch,id,flag,pre,gold,heal) in [
        (100,"jigu_defiance_casualties","jigu_casualty_record","redcliff_wind",24,11),
        (100,"wei_caozhen_death_archive","wei_caozhen_archive_record","guandu_grain",21,8),
        (101,"lucheng_wheat_household","lucheng_wheat_rotation_record","wudu_yinping_civil_record",26,11),
        (101,"liyan_dispatch_evidence","liyan_dispatch_evidence_record","jigu_casualty_record",25,10),
        (102,"weibei_bridge_loss_ledger","weibei_bridge_losses_record","redcliff_wind",22,11),
        (102,"wooden_ox_transport_audit","weinan_transport_record","redcliff_wind",24,10),
    ] {
        let row=audit.lines().find(|x|x.starts_with(&format!("{ch:03}\t"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("CHAPTERS_100_102_JIAJING_VARIANTS.md"));
        let b=beat(id).unwrap();
        assert_eq!(b.chapter,ch);
        assert_eq!(b.flag,flag);
        assert_eq!(b.prerequisite,Some(pre));
        assert_eq!((b.reward_gold,b.reward_heal),(gold,heal));
        assert!(b.premise.contains("嘉靖"), "{id}");
        assert!(b.premise.contains("毛评"), "{id}");
    }
}
fn enter(g:&mut Game,id:&str) {
    g.stage=Stage::Story;
    g.lore_current=Some(id.to_string());
}
fn has(g:&Game,flag:&str)->bool {g.lore_flags.iter().any(|f|f==flag)}
#[test]
fn jigu_liyan_weibei_transport_choices_are_atomic_mutually_exclusive_and_restore_v2() {
    let mut g=Game::new(Hero::ZhugeLiang,100102);
    g.lore_flags.push("redcliff_wind".into());
    enter(&mut g,"jigu_defiance_casualties");
    let before=g.to_json().unwrap();
    assert!(g.apply(Action::Choose{index:99}).is_err());
    assert_eq!(g.to_json().unwrap(),before);
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"jigu_casualty_record"));
    let saved=g.to_json().unwrap();
    let mut g=Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(),saved);
    enter(&mut g,"liyan_dispatch_evidence");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"liyan_dispatch_evidence_record"));
    enter(&mut g,"weibei_bridge_loss_ledger");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"weibei_bridge_losses_record"));
    enter(&mut g,"wooden_ox_transport_audit");
    g.apply(Action::Choose{index:1}).unwrap();
    assert!(has(&g,"weinan_transport_record"));
    let save=g.to_json().unwrap();
    assert_eq!(Game::from_json(&save).unwrap().to_json().unwrap(),save);

    let mut declined=Game::new(Hero::ZhugeLiang,100102);
    declined.lore_flags.push("redcliff_wind".into());
    enter(&mut declined,"jigu_defiance_casualties");
    declined.apply(Action::Choose{index:2}).unwrap();
    assert!(!has(&declined,"jigu_casualty_record"));
    // A scripted Story entry bypasses map eligibility: do not claim an absent
    // prerequisite makes the scene itself unselectable in this harness.
    assert!(!has(&declined,"liyan_dispatch_evidence_record"));
}
#[test]
fn ch102_transport_to_ch103_fire_payoff_is_once_only_across_restore() {
    let mut g=Game::new(Hero::ZhugeLiang,100102);
    g.lore_flags.extend(["redcliff_wind","weibei_bridge_losses_record"].map(str::to_owned));
    enter(&mut g,"wooden_ox_transport_audit");
    g.apply(Action::Choose{index:1}).unwrap();
    let mut g=Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g,"shangfang_fire_account");
    g.apply(Action::Choose{index:1}).unwrap();
    enter(&mut g,"wuzhang_withdrawal_evidence");
    g.apply(Action::Choose{index:1}).unwrap();
    let egg="weibei_shangfang_withdrawal_debt";
    assert_eq!(g.easter_unlocked.iter().filter(|x|x.as_str()==egg).count(),1);
    let mut g=Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g,"wuzhang_withdrawal_evidence");
    g.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(g.easter_unlocked.iter().filter(|x|x.as_str()==egg).count(),1);
}
