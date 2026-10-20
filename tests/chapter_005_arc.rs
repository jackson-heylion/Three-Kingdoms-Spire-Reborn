//! Verify chapter V's optional Guan Yu storyline and its delayed payoff.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage, content::Hero, lore::{beat,next_beat},
};

#[test]
fn chapter_v_follows_anxi_but_does_not_gate_original_jingzhou() {
    let old=vec!["taoyuan".to_owned(),"anxi".to_owned()];
    let no_anxi=vec!["taoyuan_oath".to_owned()];
    let first=next_beat(Hero::GuanYu,0,&no_anxi,&old);
    assert_ne!(first.map(|b|b.id),Some("wenjiu"));

    let valid=vec!["taoyuan_oath".to_owned(),"anxi_seal".to_owned()];
    let scene=next_beat(Hero::GuanYu,0,&valid,&old).unwrap();
    assert_eq!((scene.id,scene.chapter,scene.prerequisite),
        ("wenjiu",5,Some("anxi_seal")));
    assert_eq!((scene.flag,scene.reward_card),("wenjiu_oath","pojun"));

    let without_fifth=next_beat(Hero::GuanYu,2,&valid,&[]).unwrap();
    assert_eq!(without_fifth.id,"jingzhou",
        "original late-story gating must not require an optional early encounter");
    assert!(beat("maicheng").is_some());
    let lu=next_beat(Hero::LuBu,0,&valid,&[]);
    assert_ne!(lu.map(|s|s.id),Some("wenjiu"));
}

#[test]
fn warmwine_and_jingzhou_jointly_affect_only_later_opening_block() {
    fn opening(flags:&[&str])->(i32,i32,i32) {
        let mut game=Game::new(Hero::GuanYu,951);
        game.lore_flags=flags.iter().map(|x|(*x).into()).collect();
        game.apply(Action::Choose{index:1}).unwrap();
        assert_eq!(game.stage,Stage::Combat);
        let b=game.battle.as_ref().unwrap();
        let stats=(b.block,b.strength,b.momentum);
        let snapshot=game.to_json().unwrap();
        assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
        stats
    }
    let baseline=opening(&[]);
    let warmwine=opening(&["wenjiu_oath"]);
    let jingzhou=opening(&["jingzhou_warning"]);
    let both=opening(&["wenjiu_oath","jingzhou_warning"]);
    assert_eq!(warmwine,baseline,"no immediate extra bonus for a single flag");
    assert_eq!(jingzhou,baseline,"the original Jingzhou flag alone is not altered");
    assert_eq!(both,(baseline.0+3,baseline.1,baseline.2));
}
