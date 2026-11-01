//! Pinned 010–012 Mao/Jiajing *local* variants; full collation remains unverified.
use three_kingdoms_spire_reborn::{Action, Game, Stage, content::Hero, lore};

fn scene(hero: Hero, seed: u64, id: &str) -> Game {
    let mut game = Game::new(hero, seed);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game
}
fn choose(game: &mut Game, id: &str, index: usize) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.apply(Action::Choose { index }).unwrap();
}
fn count(xs: &[String], target: &str) -> usize {
    xs.iter().filter(|x| x.as_str() == target).count()
}

#[test]
fn pinned_nine_novel_revisions_and_source_grading_are_explicit() {
    let evidence = include_str!("../docs/CHAPTERS_010_012_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in ["2567973","2567975","2567974","2554685","2554657",
                "2554681","2554703","7908283","2554707","2585429"] {
        assert!(evidence.contains(rev), "missing fixed revision {rev}");
    }
    for term in ["夏侯惇","夏侯渊","十余萬","數十騎","十數騎",
                 "搜神記","掣劍自刎","裴注","原刻影印","新增0回"] {
        assert!(evidence.contains(term), "missing source distinction {term}");
    }
    assert_eq!(audit.lines().skip(1).filter(|s| !s.is_empty()).count(), 120);
    for ch in 10..=12 {
        let row = audit.lines().find(|s| s.starts_with(&format!("{ch:03}\t"))).unwrap();
        assert!(row.contains("正文通读/未完成异本对校"));
        assert!(row.contains("CHAPTERS_010_012_JIAJING_VARIANTS.md"));
    }
    for (id, ch, flag, gold, heal, anchor) in [
        ("xuzhou",10,"xuzhou_debt",24,6,"夏侯渊"),
        ("puyang",12,"puyang_supply",16,7,"十数骑"),
        ("beihai_xuzhou_relief_archive",11,"beihai_xuzhou_relief_record",17,7,"搜神记"),
        ("xuzhou_yanzhou_dispatch_archive",11,"xuzhou_yanzhou_dispatch_record",18,6,"嘉靖022"),
        ("yanzhou_taishan_ambush_archive",11,"yanzhou_taishan_ambush_record",17,7,"嘉靖022"),
    ] {
        let beat = lore::beat(id).unwrap();
        assert_eq!((beat.chapter,beat.flag,beat.reward_gold,beat.reward_heal),
                   (ch,flag,gold,heal), "{id} contract changed");
        assert!(beat.premise.contains(anchor), "{id} missing compared witness");
    }
}

#[test]
fn cao_xuzhou_dispatch_puyang_is_optional_atomic_persistent_and_once_only() {
    let mut g = scene(Hero::CaoCao, 101201, "xuzhou");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    choose(&mut g,"xuzhou_yanzhou_dispatch_archive",1);
    assert_eq!(count(&g.easter_unlocked,"xuzhou_yanzhou_ration_debt"),0);
    choose(&mut g,"puyang",1);
    assert_eq!(count(&g.easter_unlocked,"xuzhou_yanzhou_ration_debt"),1);
    let mut restored = Game::from_json(&g.to_json().unwrap()).unwrap();
    let gold = restored.gold;
    choose(&mut restored,"puyang",1);
    assert_eq!(count(&restored.easter_unlocked,"xuzhou_yanzhou_ration_debt"),1);
    assert_eq!(restored.gold,gold + lore::beat("puyang").unwrap().reward_gold);
    let mut refused = scene(Hero::CaoCao,101202,"xuzhou");
    refused.apply(Action::Choose { index: 2 }).unwrap();
    choose(&mut refused,"xuzhou_yanzhou_dispatch_archive",1);
    choose(&mut refused,"puyang",1);
    assert_eq!(count(&refused.lore_flags,"xuzhou_debt"),0);
    assert_eq!(count(&refused.easter_unlocked,"xuzhou_yanzhou_ration_debt"),0);
}

#[test]
fn guan_and_lubu_cross_chapter_archives_are_exclusive_and_v2_safe() {
    let mut guan = scene(Hero::GuanYu, 111201, "beihai_xuzhou_relief_archive");
    guan.lore_flags.extend(["taoyuan_oath".into(),"xuzhou_gate_account".into()]);
    let before = guan.to_json().unwrap();
    assert!(guan.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(guan.to_json().unwrap(),before);
    guan.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&guan.easter_unlocked,"beihai_xuzhou_civil_debt"),1);
    let restored = Game::from_json(&guan.to_json().unwrap()).unwrap();
    assert_eq!(restored.easter_unlocked,guan.easter_unlocked);
    let mut lu = scene(Hero::LuBu, 111202, "yanzhou_taishan_ambush_archive");
    lu.lore_flags.extend(["changan_account".into(),"xiaopei_refuge".into()]);
    lu.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&lu.lore_flags,"yanzhou_taishan_ambush_record"),0);
    assert_eq!(count(&lu.easter_unlocked,"changan_yanzhou_xiaopei_debt"),0);
    assert!(Game::from_json(&lu.to_json().unwrap()).is_ok());
    let mut accepted = scene(Hero::LuBu,111203,"yanzhou_taishan_ambush_archive");
    accepted.lore_flags.extend(["changan_account".into(),"xiaopei_refuge".into()]);
    accepted.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&accepted.easter_unlocked,"changan_yanzhou_xiaopei_debt"),1);
}

#[test]
fn independent_mao_010_012_chapter_boundaries_are_not_conflated() {
    let evidence = include_str!("../docs/CHAPTERS_010_012_JIAJING_VARIANTS.md");
    let ledger = include_str!("../data/chapter_body_audit.tsv");
    for marker in [
        "2026-11-01 毛评固定电子本010",
        "oldid=2567973", "oldid=2567975", "oldid=2567974",
        "回末提出守城计策者尚未揭名",
        "回末吕布仍在追击", "回末吕布未被擒",
        "曹操原欲继续攻城", "未做嘉靖原刻影印核字",
    ] {
        assert!(evidence.contains(marker), "missing chapter evidence: {marker}");
    }
    for (chapter, oldid) in [("010", "2567973"), ("011", "2567975"), ("012", "2567974")] {
        let row = ledger.lines().find(|s| s.starts_with(&format!("{chapter}\t")))
            .expect("chapter row exists");
        assert!(row.contains(&format!("2026-11-01独立复核毛评固定oldid={oldid}")));
        assert!(row.contains("非全回异文校勘"));
    }
    let puyang = lore::beat("puyang").unwrap();
    assert!(puyang.premise.contains("吕布仍与陈宫会合再引军"));
    assert!(puyang.premise.contains("兖州急报"));
}

#[test]
fn declining_xuzhou_dispatch_does_not_forge_the_cross_chapter_reward() {
    let mut game = scene(Hero::CaoCao, 101209, "xuzhou_yanzhou_dispatch_archive");
    game.lore_flags.push("xuzhou_debt".into());
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 3 }).unwrap();
    assert!(!game.lore_flags.iter().any(|f| f == "xuzhou_yanzhou_dispatch_record"));
    let restored = Game::from_json(&game.to_json().unwrap()).unwrap();
    assert_eq!(restored.lore_flags, game.lore_flags);
    assert!(!restored.easter_unlocked.iter().any(|x| x == "xuzhou_yanzhou_ration_debt"));
}
