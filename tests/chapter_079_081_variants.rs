//! 079–081 fixed-edition spot checks and existing real playable cross-chapter accountability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.to_owned());
}
fn count(items: &[String], id: &str) -> usize {
    items.iter().filter(|x| x.as_str() == id).count()
}

#[test]
fn pinned_editions_and_six_playable_contracts() {
    let evidence = include_str!("../docs/CHAPTERS_079_081_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for revision in ["2568043","7909161","8730381","2554736","2554738","2554777","2554755","2554805","2554756"] {
        assert!(evidence.contains(revision), "missing fixed revision {revision}");
    }
    for anchor in ["武祖","武王","于禁","刘封","曹皇后","吾父掃清海內","亂賊","朕且罷兵","心中稍回"] {
        assert!(evidence.contains(anchor), "missing actual variant {anchor}");
    }
    let rows: Vec<_> = audit.lines().skip(1).filter(|row| !row.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for chapter in 79..=81 {
        let row = rows.iter().find(|row| row.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("CHAPTERS_079_081_JIAJING_VARIANTS.md"));
        assert!(row.contains("局部") && row.contains("未完成异本对校"));
    }
    for (id, chapter, prerequisite, flag, gold, heal) in [
        ("ye_court_succession_review",79,"huatuo_prison_record","ye_succession_record",21,8),
        ("shangyong_refusal_trial",79,"jingzhou_mourning_record","shangyong_trial_record",20,9),
        ("abdication_seal_audit",80,"ye_succession_record","abdication_seal_record",22,9),
        ("chengdu_legitimacy_review",80,"shangyong_trial_record","chengdu_legitimacy_record",22,10),
        ("eastward_campaign_counsel",81,"chengdu_legitimacy_record","eastward_counsel_record",20,9),
        ("langzhong_heads_receipt",81,"jingzhou_aftermath_record","langzhong_heads_record",19,8),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(prerequisite));
        assert_eq!(b.flag, flag);
        assert_eq!((b.reward_gold,b.reward_heal),(gold,heal));
        assert!(b.premise.contains("嘉靖") && b.premise.contains("毛评"), "{id}");
    }
}

#[test]
fn wei_succession_ledger_is_atomic_and_reward_is_once_after_v2_restore() {
    let mut game = Game::new(Hero::CaoCao, 79081);
    for flag in ["guandu_grain","luoyang_head_record"] { game.lore_flags.push(flag.into()); }
    for id in ["huatuo_prison_review","ye_court_succession_review","abdication_seal_audit"] {
        enter(&mut game,id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(),before);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&game.easter_unlocked,"guandu_succession_abdication_debt"),1);
    let saved = game.to_json().unwrap();
    let mut game = Game::from_json(&saved).unwrap();
    assert_eq!(game.to_json().unwrap(),saved);
    for id in ["ye_court_succession_review","abdication_seal_audit"] {
        enter(&mut game,id);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&game.easter_unlocked,"guandu_succession_abdication_debt"),1);
    assert_eq!(count(&game.lore_flags,"abdication_seal_record"),1);
}

#[test]
fn shangyong_to_eastward_counsel_is_playable_but_cannot_rewrite_the_campaign() {
    let mut game = Game::new(Hero::ZhugeLiang, 80081);
    for flag in ["redcliff_wind","chengdu_mourning_record"] { game.lore_flags.push(flag.into()); }
    for id in ["jingzhou_mourning_counsel","shangyong_refusal_trial","chengdu_legitimacy_review","eastward_campaign_counsel","yiling_camp_map_warning"] {
        enter(&mut game,id);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&game.easter_unlocked,"redcliff_chengdu_legitimacy_debt"),1);
    assert_eq!(count(&game.easter_unlocked,"redcliff_yiling_counsel_debt"),1);
    let saved=game.to_json().unwrap();
    let mut restored=Game::from_json(&saved).unwrap();
    enter(&mut restored,"eastward_campaign_counsel");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.easter_unlocked,"redcliff_chengdu_legitimacy_debt"),1);
    assert_eq!(count(&restored.easter_unlocked,"redcliff_yiling_counsel_debt"),1);
}

#[test]
fn refusing_shangyong_evidence_cannot_forge_the_later_reward() {
    let mut game=Game::new(Hero::ZhugeLiang, 81081);
    game.lore_flags.push("jingzhou_mourning_record".into());
    enter(&mut game,"shangyong_refusal_trial");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&game.lore_flags,"shangyong_trial_record"),0);
    enter(&mut game,"chengdu_legitimacy_review");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&game.easter_unlocked,"redcliff_chengdu_legitimacy_debt"),0);
}
