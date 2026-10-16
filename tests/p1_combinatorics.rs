//! Broad pairwise combinatorics: every relic × every boon × each potion.
//! The script never changes a production campaign; it tests rule boundaries.
use three_kingdoms_spire_reborn::{
    Action,Game,Stage,content::Hero,
    items::{BOONS,POTIONS,RELICS},
};

#[test]
fn every_relic_boon_and_potion_combination_is_atomic_saveable_and_event_ordered() {
    let mut cases=0usize;
    for &(relic,_) in RELICS {
        for &(boon,_) in BOONS {
            for &(potion,_) in POTIONS {
                let seed=1_500_001+cases as u64;
                let hero=[Hero::GuanYu,Hero::ZhugeLiang,Hero::SunShangxiang,
                    Hero::CaoCao,Hero::LuBu][cases%5];
                let mut g=Game::new_with_ascension(hero,seed,(cases%11) as u8);
                if !g.relics.iter().any(|r|r==relic) {
                    g.relics.push(relic.into());
                    g.relic_history.push(relic.into());
                }
                g.boons.push(boon.into());
                g.potion_bag=vec![potion.into()];
                g.potions=1;
                g.apply(Action::Choose{index:1}).unwrap_or_else(|e|
                    panic!("failed battle start relic={relic} boon={boon} potion={potion}: {e}"));
                assert_eq!(g.stage,Stage::Combat);
                let saved=g.to_json().unwrap();
                let restored=Game::from_json(&saved).unwrap();
                assert_eq!(restored.to_json().unwrap(),saved);
                assert!(g.apply(Action::Play{index:100,target:1}).is_err());
                assert_eq!(g.to_json().unwrap(),saved,
                    "invalid action mutated {relic}/{boon}/{potion}");
                g.apply(Action::UsePotionAt{slot:1,target:1}).unwrap_or_else(|e|
                    panic!("potion failed relic={relic} boon={boon} potion={potion}: {e}"));
                let snapshot=g.to_json().unwrap();
                assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
                if let Some(b)=&g.battle {
                    assert!(b.events.windows(2).all(|events|events[0].seq<events[1].seq));
                    assert!(b.events.len()<=80);
                }
                cases+=1;
            }
        }
    }
    assert_eq!(cases,16*12*10);
}

#[test]
fn invalid_actions_rollback_all_hero_and_ascension_variations() {
    let heroes=[Hero::GuanYu,Hero::ZhugeLiang,Hero::SunShangxiang,Hero::CaoCao,Hero::LuBu];
    let mut cases=0;
    for hero in heroes {
        for ascension in 0..=10 {
            for seed in 1..=8 {
                let mut g=Game::new_with_ascension(hero,seed,ascension);
                let before=g.to_json().unwrap();
                for action in [
                    Action::Choose{index:999},
                    Action::Play{index:1,target:1},
                    Action::UsePotionAt{slot:999,target:1},
                    Action::SetDoctrine{id:"invalid".into()},
                    Action::EndTurn,
                ] {
                    assert!(g.apply(action).is_err());
                    assert_eq!(g.to_json().unwrap(),before);
                }
                g.apply(Action::Choose{index:1}).unwrap();
                let battle=g.to_json().unwrap();
                for action in [
                    Action::Play{index:999,target:1},
                    Action::Play{index:1,target:999},
                    Action::Choose{index:999},
                    Action::SetDoctrine{id:"auto".into()},
                ] {
                    assert!(g.apply(action).is_err());
                    assert_eq!(g.to_json().unwrap(),battle);
                }
                assert_eq!(Game::from_json(&battle).unwrap().to_json().unwrap(),battle);
                assert_eq!(g.verify_action_replay(),Ok(1));
                cases+=1;
            }
        }
    }
    assert_eq!(cases,5*11*8);
}
