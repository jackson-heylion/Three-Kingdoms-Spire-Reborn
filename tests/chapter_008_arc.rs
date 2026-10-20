//! Chapter VIII is a literary plot boundary, not a historical assertion
//! and not an early resolution of chapter IX's assassination.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage, content::Hero, lore::{beat, next_beat},
};

#[test]
fn chapter_eight_follows_red_hare_and_not_the_ninth_chapter_assassination() {
    let red_hare = beat("redhare").unwrap();
    let pavilion = beat("pavilion").unwrap();
    let halberd = beat("halberd").unwrap();
    assert_eq!(red_hare.chapter, 3);
    assert_eq!(pavilion.chapter, 8);
    assert_eq!(pavilion.prerequisite, Some("red_hare"));
    assert_eq!(halberd.prerequisite, Some("phoenix_pavilion"));
    assert!(pavilion.premise.contains("张温"));
    assert!(pavilion.premise.contains("貂蝉"));
    assert!(pavilion.premise.contains("李儒"));
    assert!(pavilion.premise.contains("掷戟"));
    assert!(pavilion.premise.contains("未死"));
    assert!(!pavilion.bargain.contains("密函"));

    let seen = vec!["redhare".to_string()];
    assert_ne!(next_beat(Hero::LuBu, 0, &[], &seen).map(|b|b.id), Some("pavilion"));
    assert_eq!(next_beat(Hero::LuBu, 0, &["red_hare".into()], &seen)
        .map(|b|b.id), Some("pavilion"));
}

fn take_pavilion(choice:usize) -> Game {
    let mut game = Game::new(Hero::LuBu, 8008);
    game.stage=Stage::Story;
    game.lore_current=Some("pavilion".into());
    let before=game.to_json().unwrap();
    assert!(game.apply(Action::Choose {index:0}).is_err());
    assert_eq!(game.to_json().unwrap(),before);
    game.apply(Action::Choose {index:choice}).unwrap();
    let snapshot=game.to_json().unwrap();
    assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
    game
}

#[test]
fn invented_alternative_keeps_mutually_exclusive_future_flag() {
    let vowed=take_pavilion(1);
    let bargained=take_pavilion(2);
    let declined=take_pavilion(3);
    assert!(vowed.lore_flags.iter().any(|f|f=="phoenix_pavilion"));
    assert!(!bargained.lore_flags.iter().any(|f|f=="phoenix_pavilion"));
    assert!(!declined.lore_flags.iter().any(|f|f=="phoenix_pavilion"));
    assert!(vowed.deck.iter().any(|c|c.id=="huaji"));
    assert!(bargained.gold>vowed.gold);
    assert_eq!(next_beat(Hero::LuBu,1,&vowed.lore_flags,&[])
        .map(|b|b.id),Some("halberd"));
    assert_ne!(next_beat(Hero::LuBu,1,&bargained.lore_flags,&[])
        .map(|b|b.id),Some("halberd"));
}
