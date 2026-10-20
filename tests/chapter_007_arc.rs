//! Chapter VII's Sun family memory and the third-act Yiling consequence.
//! Both are original selectable game writing, not assertions that Sun Shangxiang
//! witnessed her father's death in the historical episode.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage, content::Hero, lore::{beat,next_beat},
};

fn choice(id:&str, which:usize) -> Game {
    let mut game=Game::new(Hero::SunShangxiang, 7007);
    game.stage=Stage::Story;
    game.lore_current=Some(id.into());
    let rejected=game.to_json().unwrap();
    assert!(game.apply(Action::Choose{index:0}).is_err());
    assert_eq!(game.to_json().unwrap(),rejected);
    game.apply(Action::Choose{index:which}).unwrap();
    let saved=game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(),saved);
    game
}

#[test]
fn family_report_precedes_wedding_and_never_implies_direct_witness() {
    let first=next_beat(Hero::SunShangxiang,0,&[],&[]).unwrap();
    assert_eq!(first.id,"sun_father_letter");
    assert!(first.premise.contains("韩当") && first.premise.contains("孙坚"));
    assert!(first.premise.contains("未亲历"));
    assert_eq!(beat("sun_father_letter").unwrap().chapter,7);
    let older=beat("imperial_jade").unwrap();
    assert_eq!(older.chapter,6);
    let taken=vec!["sun_father_letter".to_string()];
    assert_eq!(next_beat(Hero::SunShangxiang,0,&[],&taken).unwrap().id,
        "jiangdong_wedding");
}

#[test]
fn only_accepted_responsibility_unblocks_third_act_relief() {
    let kept=choice("sun_father_letter",1);
    let loot=choice("sun_father_letter",2);
    let avoid=choice("sun_father_letter",3);
    assert!(kept.lore_flags.iter().any(|flag|flag=="sun_father_dispatch"));
    assert!(!loot.lore_flags.iter().any(|flag|flag=="sun_father_dispatch"));
    assert!(!avoid.lore_flags.iter().any(|flag|flag=="sun_father_dispatch"));
    assert!(kept.deck.iter().any(|card|card.id=="qiaoying"));
    assert!(loot.gold>kept.gold);
    let after=next_beat(Hero::SunShangxiang,2,&kept.lore_flags,&[]).unwrap();
    assert_eq!(after.id,"sun_yiling_shelter");
    assert_ne!(next_beat(Hero::SunShangxiang,2,&loot.lore_flags,&[])
        .map(|beat|beat.id),Some("sun_yiling_shelter"));
    assert_ne!(next_beat(Hero::SunShangxiang,2,&avoid.lore_flags,&[])
        .map(|beat|beat.id),Some("sun_yiling_shelter"));
    let relief=choice("sun_yiling_shelter",1);
    assert!(relief.lore_flags.iter().any(|f|f=="sun_yiling_relief"));
    assert!(relief.deck.iter().any(|card|card.id=="yidong"));
    assert!(!choice("sun_yiling_shelter",2)
        .lore_flags.iter().any(|f|f=="sun_yiling_relief"));
    assert_eq!(beat("sun_yiling_shelter").unwrap().prerequisite,
        Some("sun_father_dispatch"));
}
