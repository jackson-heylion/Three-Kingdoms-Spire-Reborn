//! Fixed JiaJing/Mao local variants in 076–078 and real retrospective story actions.
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
fn keep(game: &mut Game, id: &str) {
    enter(game, id);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err(), "{id}");
    assert_eq!(game.to_json().unwrap(), before, "{id} rejected action mutated state");
    game.apply(Action::Choose { index: 1 }).unwrap();
}

#[test]
fn pinned_editions_and_six_existing_game_contracts() {
    let doc = include_str!("../docs/CHAPTERS_076_078_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2568038", "2568041", "2568040", "7908290", "2554731", "2554814", "2554743", "2554778", "2554754"] {
        assert!(doc.contains(oldid), "missing fixed edition {oldid}");
    }
    for anchor in ["杖四十", "杖八十", "三十余合", "三四合", "玉帝有詔", "棄卻刀馬", "长钩套索", "青囊书", "一兩葉", "华歆"] {
        assert!(doc.contains(anchor), "missing actual contrast {anchor}");
    }
    let rows: Vec<_> = audit.lines().skip(1).filter(|row| !row.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for chapter in 76..=78 {
        let row = rows.iter().find(|row| row.starts_with(&format!("{chapter:03}\t"))).unwrap();
        assert!(row.contains("CHAPTERS_076_078_JIAJING_VARIANTS.md"));
        assert!(row.contains("局部") && row.contains("未完成异本对校"));
    }
    for (id, chapter, prerequisite, flag, gold, heal) in [
        ("maicheng_relief_review", 76, "blade_wound_record", "maicheng_relief_record", 19, 8),
        ("jingzhou_aftermath_courier", 77, "white_cloak_signal_record", "jingzhou_aftermath_record", 20, 9),
        ("luoyang_head_rites", 77, "pangde_yujin_record", "luoyang_head_record", 21, 8),
        ("chengdu_mourning_register", 77, "hanzhong_king_record", "chengdu_mourning_record", 20, 9),
        ("huatuo_prison_review", 78, "luoyang_head_record", "huatuo_prison_record", 21, 8),
        ("jingzhou_mourning_counsel", 78, "chengdu_mourning_record", "jingzhou_mourning_record", 20, 9),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(prerequisite));
        assert_eq!(b.flag, flag);
        assert_eq!((b.reward_gold, b.reward_heal), (gold, heal));
        assert!(b.premise.contains("嘉靖") && b.premise.contains("毛评"), "{id}");
    }
}

#[test]
fn guan_and_sun_have_mutually_exclusive_evidence_and_one_time_consequences() {
    let mut guan = Game::new(Hero::GuanYu, 76077);
    for flag in ["anxi_seal", "jingxiang_border_record", "fancheng_flood_record"] {
        guan.lore_flags.push(flag.into());
    }
    keep(&mut guan, "blade_wound_watch");
    keep(&mut guan, "maicheng_relief_review");
    assert_eq!(count(&guan.easter_unlocked, "jingzhou_maicheng_courier_debt"), 1);
    let saved = guan.to_json().unwrap();
    let mut guan = Game::from_json(&saved).unwrap();
    keep(&mut guan, "maicheng_relief_review");
    assert_eq!(count(&guan.easter_unlocked, "jingzhou_maicheng_courier_debt"), 1);
    assert_eq!(count(&guan.lore_flags, "maicheng_relief_record"), 1);

    let mut sun = Game::new(Hero::SunShangxiang, 77078);
    for flag in ["ruxu_rescue_record", "yiling_ashes"] {
        sun.lore_flags.push(flag.into());
    }
    keep(&mut sun, "white_cloak_signal_audit");
    keep(&mut sun, "jingzhou_aftermath_courier");
    assert_eq!(count(&sun.easter_unlocked, "jiangdong_jingzhou_aftermath_debt"), 1);
    let saved = sun.to_json().unwrap();
    let mut sun = Game::from_json(&saved).unwrap();
    keep(&mut sun, "jingzhou_aftermath_courier");
    assert_eq!(count(&sun.easter_unlocked, "jiangdong_jingzhou_aftermath_debt"), 1);
}

#[test]
fn cao_and_zhuge_archive_chains_survive_restore_without_rewriting_deaths() {
    let mut cao = Game::new(Hero::CaoCao, 78077);
    for flag in ["guandu_grain", "pangde_yujin_record"] {
        cao.lore_flags.push(flag.into());
    }
    keep(&mut cao, "luoyang_head_rites");
    keep(&mut cao, "huatuo_prison_review");
    assert_eq!(count(&cao.easter_unlocked, "guandu_luoyang_rites_debt"), 1);
    let saved = cao.to_json().unwrap();
    let mut cao = Game::from_json(&saved).unwrap();
    keep(&mut cao, "huatuo_prison_review");
    assert_eq!(count(&cao.easter_unlocked, "guandu_luoyang_rites_debt"), 1);

    let mut zhuge = Game::new(Hero::ZhugeLiang, 78078);
    for flag in ["redcliff_wind", "hanzhong_king_record"] {
        zhuge.lore_flags.push(flag.into());
    }
    keep(&mut zhuge, "chengdu_mourning_register");
    keep(&mut zhuge, "jingzhou_mourning_counsel");
    assert_eq!(count(&zhuge.easter_unlocked, "redcliff_chengdu_mourning_debt"), 1);
    let saved = zhuge.to_json().unwrap();
    let mut zhuge = Game::from_json(&saved).unwrap();
    keep(&mut zhuge, "jingzhou_mourning_counsel");
    assert_eq!(count(&zhuge.easter_unlocked, "redcliff_chengdu_mourning_debt"), 1);
}

#[test]
fn rejecting_maicheng_or_luoyang_evidence_cannot_forge_later_flags() {
    let mut guan = Game::new(Hero::GuanYu, 76079);
    guan.lore_flags.push("blade_wound_record".into());
    enter(&mut guan, "maicheng_relief_review");
    guan.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&guan.lore_flags, "maicheng_relief_record"), 0);
    assert_eq!(count(&guan.easter_unlocked, "jingzhou_maicheng_courier_debt"), 0);

    let mut cao = Game::new(Hero::CaoCao, 78079);
    cao.lore_flags.push("pangde_yujin_record".into());
    enter(&mut cao, "luoyang_head_rites");
    cao.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&cao.lore_flags, "luoyang_head_record"), 0);
    // An explicitly forced Story scene bypasses natural-scene eligibility.
    // Refusing the prerequisite must not itself grant the downstream flag.
    assert_eq!(beat("huatuo_prison_review").unwrap().prerequisite, Some("luoyang_head_record"));
    assert_eq!(count(&cao.lore_flags, "huatuo_prison_record"), 0);
}
