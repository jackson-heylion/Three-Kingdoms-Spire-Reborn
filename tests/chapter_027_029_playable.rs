//! Chapters 27–29: pinned single-witness reading, optional causal choices and old-route compatibility.
//! These choices document counterfactual responsibility; they never rewrite the novel.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, bonds_for};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 27029);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn earned(game: &Game, id: &str) -> bool {
    game.lore_flags.iter().any(|flag| flag == id)
}
fn unlocked(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn three_chapters_have_distinct_provenance_and_real_prior_setups() {
    let passes = beat("five_passes").unwrap();
    let city = beat("gucheng_witness").unwrap();
    let succession = beat("sun_succession").unwrap();
    assert_eq!((passes.chapter, passes.act, passes.prerequisite),
        (27, 1, Some("seal_return_record")));
    assert_eq!((city.chapter, city.act, city.prerequisite),
        (28, 1, Some("five_passes_record")));
    assert_eq!((succession.chapter, succession.act, succession.prerequisite),
        (29, 1, Some("taishi_trust")));
    assert!(passes.premise.contains("孔秀") && passes.premise.contains("秦琪"));
    assert!(passes.premise.contains("夏侯惇"));
    assert!(city.premise.contains("旗卒") && city.premise.contains("蔡阳"));
    assert!(succession.premise.contains("许贡") && succession.premise.contains("鲁肃"));
    assert_eq!(beat("gucheng").unwrap().chapter, 28);
    assert_eq!(beat("huarong").unwrap().prerequisite, Some("baima_debt"));
    assert_eq!(beat("jiangdong_wedding").unwrap().prerequisite, None);
    assert_eq!(next_beat(Hero::GuanYu, 1, &["seal_return_record".into()], &[])
        .map(|b| b.id), Some("five_passes"));
    assert_eq!(next_beat(Hero::GuanYu, 1,
        &["seal_return_record".into(), "five_passes_record".into()], &[])
        .map(|b| b.id), Some("gucheng_witness"));
    assert_eq!(next_beat(Hero::SunShangxiang, 1, &["taishi_trust".into()], &[])
        .map(|b| b.id), Some("sun_succession"));
}

#[test]
fn guan_yu_escort_and_gucheng_testimony_pay_once_at_huarong() {
    let mut game = scene(Hero::GuanYu, "five_passes",
        &["seal_return_record", "baima_debt"]);
    let original = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(game.to_json().unwrap(), original);
    let deck_before = game.deck.len();
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(earned(&game, "five_passes_record"));
    assert_eq!(game.deck.len(), deck_before + 1);
    assert!(bonds_for(Hero::GuanYu, &game.lore_flags)
        .iter().any(|bond| bond.companion == "普净"));
    enter(&mut game, "gucheng_witness");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(earned(&game, "gucheng_witness_ledger"));
    assert_eq!(unlocked(&game, "five_passes_huarong"), 0);
    let snapshot = game.to_json().unwrap();
    let mut restored = Game::from_json(&snapshot).unwrap();
    assert_eq!(restored.to_json().unwrap(), snapshot);
    enter(&mut restored, "huarong");
    let output = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(output.contains("过关护眷"));
    assert_eq!(unlocked(&restored, "five_passes_huarong"), 1);
    enter(&mut restored, "huarong");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&restored, "five_passes_huarong"), 1);
}

#[test]
fn refusing_new_pass_or_witness_records_never_blocks_legacy_huarong() {
    let mut no_passes = scene(Hero::GuanYu, "five_passes",
        &["seal_return_record", "baima_debt"]);
    no_passes.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!earned(&no_passes, "five_passes_record"));
    assert_ne!(next_beat(Hero::GuanYu, 1, &no_passes.lore_flags, &no_passes.lore_seen)
        .map(|b| b.id), Some("gucheng_witness"));
    enter(&mut no_passes, "huarong");
    no_passes.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&no_passes, "sword_debt"), 1);
    assert_eq!(unlocked(&no_passes, "five_passes_huarong"), 0);

    let mut no_witness = scene(Hero::GuanYu, "gucheng_witness",
        &["seal_return_record", "five_passes_record", "baima_debt"]);
    let before = no_witness.to_json().unwrap();
    assert!(no_witness.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(no_witness.to_json().unwrap(), before);
    no_witness.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!earned(&no_witness, "gucheng_witness_ledger"));
    enter(&mut no_witness, "huarong");
    no_witness.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&no_witness, "five_passes_huarong"), 0);
}

#[test]
fn sun_succession_account_survives_save_and_pays_once_after_yiling() {
    let mut game = scene(Hero::SunShangxiang, "sun_succession", &["taishi_trust"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert!(earned(&game, "sun_succession_account"));
    assert!(bonds_for(Hero::SunShangxiang, &game.lore_flags)
        .iter().any(|bond| bond.companion == "孙权"));
    let snapshot = game.to_json().unwrap();
    let mut restored = Game::from_json(&snapshot).unwrap();
    assert_eq!(restored.to_json().unwrap(), snapshot);
    enter(&mut restored, "jiangdong_wedding");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut restored, "yiling");
    let output = restored.apply(Action::Choose { index: 1 }).unwrap().join(" ");
    assert!(output.contains("江东继业"));
    assert_eq!(unlocked(&restored, "jiangdong_succession_debt"), 1);
    enter(&mut restored, "yiling");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&restored, "jiangdong_succession_debt"), 1);
}

#[test]
fn sun_can_refuse_succession_record_and_keep_old_wedding_and_yiling() {
    let mut game = scene(Hero::SunShangxiang, "sun_succession", &["taishi_trust"]);
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!earned(&game, "sun_succession_account"));
    enter(&mut game, "jiangdong_wedding");
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "yiling");
    game.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(unlocked(&game, "jiangdong_succession_debt"), 0);
    assert_eq!(beat("yiling").unwrap().prerequisite, Some("jiangdong_covenant"));
}
