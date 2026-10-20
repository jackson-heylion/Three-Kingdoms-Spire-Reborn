//! Chapter 6: the jade source and legal cross-act player branches.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage, content::Hero, lore::{beat,next_beat},
};

#[test]
fn jade_story_explains_source_and_preserves_act_i_and_ii_links() {
    let jade=beat("imperial_jade").unwrap();
    assert_eq!((jade.chapter,jade.act,jade.flag),(6,0,"jade_whisper"));
    assert!(jade.premise.contains("朱匣") && jade.premise.contains("井中"));
    assert!(jade.premise.contains("袁绍") && jade.premise.contains("刘表"));
    assert!(jade.premise.contains("孙坚"));
    assert_eq!(beat("mandate").unwrap().prerequisite,Some("jade_whisper"));
    assert_eq!(beat("jin_unification").unwrap().prerequisite,Some("jade_whisper"));
    let flagged=vec!["jade_whisper".to_owned()];
    let first=next_beat(Hero::CaoCao,1,&flagged,&[]).unwrap();
    assert_eq!(first.id,"mandate");
    let last=next_beat(Hero::CaoCao,2,&flagged,&[]).unwrap();
    assert_eq!(last.id,"jin_unification");
    let without=next_beat(Hero::CaoCao,1,&[],&[]);
    assert_ne!(without.map(|b|b.id),Some("mandate"));
}

#[test]
fn mutually_exclusive_jade_choices_persist_without_changing_old_flags() {
    fn choose(which:usize)->Game {
        // Inject exactly one legal Story scene to exercise the authoritative
        // decision pipeline; not a naturally sampled map encounter.
        let mut g=Game::new(Hero::LuBu,1206);
        g.stage=Stage::Story;
        g.lore_current=Some("imperial_jade".into());
        g.apply(Action::Choose{index:which}).unwrap();
        assert!(g.lore_seen.iter().any(|id|id=="imperial_jade"));
        let raw=g.to_json().unwrap();
        assert_eq!(Game::from_json(&raw).unwrap().to_json().unwrap(),raw);
        g
    }
    let public=choose(1);
    let private=choose(2);
    let abstain=choose(3);
    assert!(public.lore_flags.iter().any(|flag|flag=="jade_whisper"));
    assert!(!private.lore_flags.iter().any(|flag|flag=="jade_whisper"));
    assert!(!abstain.lore_flags.iter().any(|flag|flag=="jade_whisper"));
    assert!(public.deck.iter().any(|card|card.id=="zhentu"));
    assert!(private.gold>public.gold,
        "the fictional concealment branch must carry its advertised payout");
}
