//! Pinned Mao/Jiajing electronic local witnesses, not full collation or natural-map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};
fn enter(game: &mut Game, id: &str) { game.stage=Stage::Story; game.lore_current=Some(id.into()); }
fn count(items: &[String], id: &str) -> usize { items.iter().filter(|s| s.as_str()==id).count() }

#[test]
fn pinned_editions_and_unchanged_reward_contracts() {
    let doc=include_str!("../docs/CHAPTERS_019_021_JIAJING_VARIANTS.md");
    let audit=include_str!("../data/chapter_body_audit.tsv");
    for revision in ["2567981","8187289","2567982","2554682","2554702","2554709","2554669","2554671","2554646"] {
        assert!(doc.contains(revision), "missing fixed revision {revision}");
    }
    for anchor in ["十數騎","數十騎","貂蟬","妻女","問我取金帛","君命有所不受","原刻影印","0回"] {
        assert!(doc.contains(anchor), "missing real variant anchor {anchor}");
    }
    assert_eq!(audit.lines().skip(1).filter(|x| !x.is_empty()).count(),120);
    for ch in 19..=21 {
        assert!(audit.lines().any(|row| row.starts_with(&format!("{ch:03}\t"))
            && row.contains("CHAPTERS_019_021_JIAJING_VARIANTS.md")));
    }
    for (id,chapter,flag,gold,heal,anchor) in [
        ("xiapi_orders",19,"xiapi_orders_account",18,5,"十数骑"),
        ("whitegate",19,"white_gate",40,10,"嘉靖038"),
        ("mandate",20,"edict_secret",18,0,"貂蝉"),
        ("xutian_guard",20,"xutian_guarded_emperor",15,6,"嘉靖039"),
        ("xuzhou_gate",21,"xuzhou_gate_account",14,7,"嘉靖042"),
    ] {
        let b=beat(id).unwrap();
        assert_eq!((b.chapter,b.flag,b.reward_gold,b.reward_heal),(chapter,flag,gold,heal));
        assert!(b.premise.contains(anchor), "{id} missing edition boundary");
    }
}
#[test]
fn xiapi_whitegate_reckoning_is_atomic_once_only_and_v2_safe() {
    let mut game=Game::new(Hero::LuBu,19019);
    game.lore_flags.extend(["xiaopei_courier_guard","halberd_shot"].map(str::to_string));
    for id in ["xiapi_orders","whitegate"] {
        enter(&mut game,id);
        let before=game.to_json().unwrap();
        assert!(game.apply(Action::Choose{index:99}).is_err());
        assert_eq!(game.to_json().unwrap(),before);
        game.apply(Action::Choose{index:1}).unwrap();
    }
    assert_eq!(count(&game.lore_flags,"xiapi_orders_account"),1);
    assert_eq!(count(&game.lore_flags,"white_gate"),1);
    assert_eq!(count(&game.easter_unlocked,"xiapi_orders_debt"),1);
    let mut restored=Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut restored,"whitegate");
    restored.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&restored.lore_flags,"white_gate"),1);
    assert_eq!(count(&restored.easter_unlocked,"xiapi_orders_debt"),1);
}
#[test]
fn guanyu_xutian_xuzhou_flags_are_optional_and_future_debt_is_idempotent() {
    let mut game=Game::new(Hero::GuanYu,20021);
    game.lore_flags.extend(["wenjiu_oath","jingzhou_warning"].map(str::to_string));
    for id in ["xutian_guard","xuzhou_gate"] {
        enter(&mut game,id);
        game.apply(Action::Choose{index:1}).unwrap();
    }
    assert_eq!(count(&game.easter_unlocked,"xutian_xuzhou_debt"),1);
    let mut restored=Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut restored,"xuzhou_gate");
    restored.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&restored.easter_unlocked,"xutian_xuzhou_debt"),1);
    let mut refused=Game::new(Hero::GuanYu,21021);
    refused.lore_flags.push("xutian_guarded_emperor".into());
    enter(&mut refused,"xuzhou_gate");
    refused.apply(Action::Choose{index:2}).unwrap();
    assert_eq!(count(&refused.lore_flags,"xuzhou_gate_account"),0);
    assert!(Game::from_json(&refused.to_json().unwrap()).is_ok());
}
