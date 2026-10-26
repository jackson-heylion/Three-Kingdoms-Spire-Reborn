//! Chapters 20–21: Xutian restraint, imperial edict and Xuzhou gate accountability.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn scene(id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(Hero::GuanYu, 20021);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s|(*s).into()).collect();
    game
}
fn count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|x|x.as_str()==id).count()
}
#[test]
fn source_chapters_and_optional_legacy_paths_remain_distinct() {
    let x = beat("xutian_guard").unwrap();
    assert_eq!((x.chapter,x.act,x.prerequisite),(20,1,Some("wenjiu_oath")));
    let y = beat("xuzhou_gate").unwrap();
    assert_eq!((y.chapter,y.act,y.prerequisite),(21,1,Some("xutian_guarded_emperor")));
    let mandate = beat("mandate").unwrap();
    assert_eq!((mandate.chapter,mandate.prerequisite,mandate.flag),(20,Some("jade_whisper"),"edict_secret"));
    assert!(mandate.premise.contains("第021回"));
    assert_eq!(next_beat(Hero::GuanYu,1,&["wenjiu_oath".into()],&[])
        .unwrap().id,"xutian_guard");
    assert_eq!(next_beat(Hero::GuanYu,1,
        &["wenjiu_oath".into(),"xutian_guarded_emperor".into()],
        &["xutian_guard".into()]).unwrap().id,"xuzhou_gate");
    assert_eq!(next_beat(Hero::GuanYu,2,&["anxi_seal".into()],&[])
        .unwrap().id,"jingzhou","old saves must not require chapters 20 or 21");
}
#[test]
fn accepted_responsibility_survives_save_and_pays_once_after_jingzhou() {
    let mut game = scene("xutian_guard",&["anxi_seal","wenjiu_oath"]);
    let before=game.to_json().unwrap();
    assert!(game.apply(Action::Choose {index:0}).is_err());
    assert_eq!(game.to_json().unwrap(),before,"rejected choice must be atomic");
    let deck=game.deck.len();
    game.apply(Action::Choose {index:1}).unwrap();
    assert!(game.lore_flags.iter().any(|s|s=="xutian_guarded_emperor"));
    assert_eq!(game.deck.len(),deck+1);
    assert_eq!(game.deck.last().unwrap().id,"hold_line");
    game.stage=Stage::Story;
    game.lore_current=Some("xuzhou_gate".into());
    game.apply(Action::Choose {index:1}).unwrap();
    assert!(game.lore_flags.iter().any(|s|s=="xuzhou_gate_account"));
    assert_eq!(count(&game,"xutian_xuzhou_debt"),0);
    let saved=game.to_json().unwrap();
    let mut restored=Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(),saved);
    restored.stage=Stage::Story;
    restored.lore_current=Some("jingzhou".into());
    let output=restored.apply(Action::Choose {index:1}).unwrap().join(" ");
    assert!(output.contains("许田收刀"));
    assert_eq!(count(&restored,"xutian_xuzhou_debt"),1);
    restored.stage=Stage::Story;
    restored.lore_current=Some("jingzhou".into());
    restored.apply(Action::Choose {index:1}).unwrap();
    assert_eq!(count(&restored,"xutian_xuzhou_debt"),1);
}
#[test]
fn refusing_either_new_scene_does_not_grant_cross_act_reward() {
    let mut game=scene("xutian_guard",&["anxi_seal","wenjiu_oath"]);
    game.apply(Action::Choose {index:2}).unwrap();
    assert!(!game.lore_flags.iter().any(|s|s=="xutian_guarded_emperor"));
    assert_ne!(next_beat(Hero::GuanYu,1,&game.lore_flags,&game.lore_seen)
        .unwrap().id,"xuzhou_gate");
    assert_eq!(next_beat(Hero::GuanYu,2,&game.lore_flags,&game.lore_seen)
        .unwrap().id,"jingzhou");
    let mut other=scene("xuzhou_gate",&["anxi_seal","wenjiu_oath","xutian_guarded_emperor"]);
    let before=other.to_json().unwrap();
    assert!(other.apply(Action::Choose {index:99}).is_err());
    assert_eq!(other.to_json().unwrap(),before);
    other.apply(Action::Choose {index:2}).unwrap();
    assert!(!other.lore_flags.iter().any(|s|s=="xuzhou_gate_account"));
    other.stage=Stage::Story;
    other.lore_current=Some("jingzhou".into());
    other.apply(Action::Choose {index:1}).unwrap();
    assert_eq!(count(&other,"xutian_xuzhou_debt"),0);
}
