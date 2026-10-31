//! Fixed electronic Mao/Jiajing local variants, not full collation or natural-map proof.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};
fn enter(game: &mut Game, id: &str) { game.stage = Stage::Story; game.lore_current = Some(id.into()); }
fn count(items: &[String], id: &str) -> usize { items.iter().filter(|s| s.as_str() == id).count() }
#[test]
fn fixed_sources_and_existing_rewards() {
    let doc = include_str!("../docs/CHAPTERS_058_060_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for id in ["7909160","2568022","2568025","2554762","2554751","2554773","7908282","2554810","2554759"] {
        assert!(doc.contains(id), "missing revision {id}");
    }
    for a in ["書略曰","不三合","二十合","互換打旗","建安十三年冬","原刻影印","0/120"] {
        assert!(doc.contains(a), "missing evidence {a}");
    }
    assert_eq!(audit.lines().skip(1).filter(|s| !s.is_empty()).count(), 120);
    for ch in 58..=60 {
        assert!(audit.lines().any(|row| row.starts_with(&format!("{ch:03}\t"))
            && row.contains("CHAPTERS_058_060_JIAJING_VARIANTS.md")));
    }
    for (id,ch,flag,gold,heal,anchor) in [
        ("tongguan_retreat",58,"tongguan_retreat_record",18,8,"嘉靖116"),
        ("weinan_division",59,"weinan_division_record",20,8,"嘉靖117"),
        ("zhangsong_audience",60,"zhangsong_envoy_record",19,8,"建安十三年冬"),
        ("yizhou_map_audit",60,"yizhou_map_record",18,8,"嘉靖119"),
    ] {
        let b=beat(id).unwrap();
        assert_eq!((b.chapter,b.flag,b.reward_gold,b.reward_heal),(ch,flag,gold,heal));
        assert!(b.premise.contains(anchor), "{id} missing variant boundary");
    }
}
#[test]
fn cao_three_chapter_ledger_is_atomic_once_only_and_v2_safe() {
    let mut game=Game::new(Hero::CaoCao,58060);
    game.lore_flags.push("mateng_case_record".into());
    for id in ["tongguan_retreat","weinan_division","zhangsong_audience"] {
        enter(&mut game,id);
        let before=game.to_json().unwrap();
        assert!(game.apply(Action::Choose{index:99}).is_err());
        assert_eq!(game.to_json().unwrap(),before);
        game.apply(Action::Choose{index:1}).unwrap();
    }
    for f in ["tongguan_retreat_record","weinan_division_record","zhangsong_envoy_record"] {
        assert_eq!(count(&game.lore_flags,f),1);
    }
    assert_eq!(count(&game.easter_unlocked,"xuchang_shu_envoy_debt"),1);
    let mut restored=Game::from_json(&game.to_json().unwrap()).unwrap();
    enter(&mut restored,"zhangsong_audience");
    restored.apply(Action::Choose{index:1}).unwrap();
    assert_eq!(count(&restored.lore_flags,"zhangsong_envoy_record"),1);
    assert_eq!(count(&restored.easter_unlocked,"xuchang_shu_envoy_debt"),1);
}
#[test]
fn refusal_does_not_forge_zhangsong_or_map_record() {
    let mut cao=Game::new(Hero::CaoCao,58061);
    cao.lore_flags.push("weinan_division_record".into());
    enter(&mut cao,"zhangsong_audience");
    cao.apply(Action::Choose{index:2}).unwrap();
    assert_eq!(count(&cao.lore_flags,"zhangsong_envoy_record"),0);
    let mut zhuge=Game::new(Hero::ZhugeLiang,58062);
    zhuge.lore_flags.push("leiyang_case_record".into());
    enter(&mut zhuge,"yizhou_map_audit");
    zhuge.apply(Action::Choose{index:2}).unwrap();
    assert_eq!(count(&zhuge.lore_flags,"yizhou_map_record"),0);
    assert!(Game::from_json(&zhuge.to_json().unwrap()).is_ok());
}
