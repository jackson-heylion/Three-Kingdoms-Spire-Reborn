//! Chapters 028–030: fixed electronic-edition spot checks, not full collation.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|x| x.as_str() == id).count()
}

#[test]
fn edition_sources_and_existing_reward_contracts_are_pinned() {
    let doc = include_str!("../docs/CHAPTERS_028_030_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for id in ["2567993", "2567991", "2567996", "2554710", "2554648",
               "2554650", "2554660", "2554642", "2554656", "2629010", "2585429"] {
        assert!(doc.contains(id), "missing fixed witness {id}");
    }
    for phrase in ["二百餘騎", "三百餘騎", "二十餘合", "五十餘萬",
                   "七十五萬", "陈寿", "裴松之", "原刻影印"] {
        assert!(doc.contains(phrase), "missing variant anchor {phrase}");
    }
    for chapter in ["028", "029", "030"] {
        let row = audit.lines().find(|row| row.starts_with(&format!("{chapter}\t"))).unwrap();
        assert!(row.contains("CHAPTERS_028_030_JIAJING_VARIANTS.md"));
    }
    assert_eq!(beat("gucheng_witness").unwrap().prerequisite, Some("five_passes_record"));
    assert_eq!(beat("sun_succession").unwrap().prerequisite, Some("taishi_trust"));
    assert_eq!(beat("guandu_letters").unwrap().prerequisite, Some("guandu_grain"));
    assert!(beat("gucheng_witness").unwrap().premise.contains("三百余骑"));
    assert!(beat("sun_succession").unwrap().premise.contains("卷46"));
    assert!(beat("guandu_letters").unwrap().premise.contains("七十五万"));
}

#[test]
fn guan_witness_is_atomic_persistent_and_old_huarong_stays_available() {
    let mut game = Game::new(Hero::GuanYu, 28030);
    game.lore_flags.extend(["five_passes_record".into(), "baima_debt".into()]);
    enter(&mut game, "gucheng_witness");
    let original = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), original);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(game.lore_flags.iter().any(|x| x == "gucheng_witness_ledger"));
    let snapshot = game.to_json().unwrap();
    let mut restored = Game::from_json(&snapshot).unwrap();
    assert_eq!(restored.to_json().unwrap(), snapshot);
    enter(&mut restored, "huarong");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "five_passes_huarong"), 1,
        "both earlier escort and Gucheng witness should pay the optional debt once");
    enter(&mut restored, "huarong");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "five_passes_huarong"), 1);
    let mut legacy = Game::new(Hero::GuanYu, 28031);
    legacy.lore_flags.extend(["five_passes_record".into(), "baima_debt".into()]);
    enter(&mut legacy, "gucheng_witness");
    legacy.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!legacy.lore_flags.iter().any(|x| x == "gucheng_witness_ledger"));
    enter(&mut legacy, "huarong");
    legacy.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&legacy, "five_passes_huarong"), 0);
}

#[test]
fn sun_and_cao_choices_remain_mutually_exclusive_and_rewards_once() {
    let mut sun = Game::new(Hero::SunShangxiang, 29030);
    sun.lore_flags.push("taishi_trust".into());
    enter(&mut sun, "sun_succession");
    let before = sun.to_json().unwrap();
    assert!(sun.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(sun.to_json().unwrap(), before);
    sun.apply(Action::Choose { index: 1 }).unwrap();
    assert!(sun.lore_flags.iter().any(|x| x == "sun_succession_account"));
    let saved = sun.to_json().unwrap();
    let mut sun = Game::from_json(&saved).unwrap();
    enter(&mut sun, "jiangdong_wedding");
    sun.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut sun, "yiling");
    sun.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&sun, "jiangdong_succession_debt"), 1);
    enter(&mut sun, "yiling");
    sun.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&sun, "jiangdong_succession_debt"), 1);

    let mut cao = Game::new(Hero::CaoCao, 30030);
    cao.lore_flags.push("guandu_grain".into());
    enter(&mut cao, "guandu_letters");
    cao.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!cao.lore_flags.iter().any(|x| x == "guandu_captured_letters"));
    enter(&mut cao, "liaodong");
    cao.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&cao, "guandu_ye_civilians"), 0);
}
