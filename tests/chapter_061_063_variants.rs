//! Chapter 061-063 fixed dual-edition spot checks, not full-collation or natural-map proof.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(flags: &[String], name: &str) -> usize {
    flags.iter().filter(|s| s.as_str() == name).count()
}
fn count_egg(game: &Game, id: &str) -> usize {
    count(&game.easter_unlocked, id)
}

#[test]
fn local_variants_have_fixed_witnesses_and_preserve_existing_rewards() {
    let doc = include_str!("../docs/CHAPTERS_061_063_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for oldid in ["2568023", "2568027", "2568024", "2554726", "2554716",
                  "2554767", "2554812", "2554742", "2554718"] {
        assert!(doc.contains(oldid), "missing source revision {oldid}");
    }
    for anchor in ["张紘", "彩段五千疋", "字永年", "字永言", "癸巳",
                   "癸亥", "2583390", "原刻影印", "0/120"] {
        assert!(doc.contains(anchor), "missing variant anchor {anchor}");
    }
    let rows: Vec<_> = audit.lines().skip(1).filter(|s| !s.is_empty()).collect();
    assert_eq!(rows.len(), 120);
    for ch in 61..=63 {
        assert!(rows.iter().any(|row| row.starts_with(&format!("{ch:03}\t"))
            && row.contains("CHAPTERS_061_063_JIAJING_VARIANTS.md")));
    }
    for (id, chapter, flag, gold, heal, anchor) in [
        ("river_child_witness", 61, "river_child_witness_record", 17, 8, "嘉靖122"),
        ("fu_pass_review", 62, "fu_pass_record", 20, 8, "嘉靖123"),
        ("luofeng_mourning", 63, "luofeng_mourning_record", 19, 8, "癸巳"),
        ("jingzhou_handover", 63, "jingzhou_handover_record", 18, 8, "嘉靖126"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!((b.chapter, b.flag, b.reward_gold, b.reward_heal),
                   (chapter, flag, gold, heal));
        assert!(b.premise.contains(anchor), "{id} lacks variant boundary");
    }
}

#[test]
fn zhuge_fu_pass_to_luofeng_is_atomic_optional_and_v2_compatible() {
    let mut game = Game::new(Hero::ZhugeLiang, 61063);
    game.lore_flags.push("yizhou_map_record".into());
    game.lore_flags.push("leiyang_case_record".into());
    for id in ["fu_pass_review", "luofeng_mourning"] {
        enter(&mut game, id);
        let before = game.to_json().unwrap();
        assert!(game.apply(Action::Choose { index: 99 }).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        game.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&game.lore_flags, "fu_pass_record"), 1);
    assert_eq!(count(&game.lore_flags, "luofeng_mourning_record"), 1);
    assert_eq!(count_egg(&game, "yizhou_fu_pass_debt"), 1);
    let saved = game.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    enter(&mut restored, "luofeng_mourning");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "luofeng_mourning_record"), 1);
    assert_eq!(count_egg(&restored, "yizhou_fu_pass_debt"), 1);
}

#[test]
fn sun_refusal_cannot_forge_river_record_and_guan_handover_remains_optional() {
    let mut sun = Game::new(Hero::SunShangxiang, 61064);
    sun.lore_flags.push("nanxu_escape_witness_record".into());
    enter(&mut sun, "river_child_witness");
    sun.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&sun.lore_flags, "river_child_witness_record"), 0);
    assert_eq!(count_egg(&sun, "nanxu_river_witness"), 0);
    assert!(Game::from_json(&sun.to_json().unwrap()).is_ok());

    let mut guan = Game::new(Hero::GuanYu, 61065);
    guan.lore_flags.push("hanjin_ferry_record".into());
    enter(&mut guan, "jingzhou_handover");
    let before = guan.to_json().unwrap();
    assert!(guan.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(guan.to_json().unwrap(), before);
    guan.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&guan.lore_flags, "jingzhou_handover_record"), 1);
    assert!(Game::from_json(&guan.to_json().unwrap()).is_ok());
}
