//! Pairwise build interaction smoke: every declared core/support cross-pair,
//! in both play orders and both upgrade states, against all 31 enemy IDs.
//! Injected deck/energy/foe fixtures establish mechanical reachability ONLY;
//! they are NOT natural drafting, strategic boss counters or win-rate samples.
use three_kingdoms_spire_reborn::{
    Action, Card, Game, Stage,
    builds::PLANS,
    combat::EventKind,
    content::{ENEMY_IDS, enemy_hp},
};

#[test]
fn every_build_core_support_pair_resolves_in_both_orders_against_all_enemy_ids() {
    let mut checked=0usize;
    for (plan_index,plan) in PLANS.iter().enumerate() {
        assert!(plan.is_catalog_valid(), "broken build catalogue: {}",plan.id);
        for (enemy_index,enemy_id) in ENEMY_IDS.iter().enumerate() {
            for upgraded in [false,true] {
                for (core_index,core_id) in plan.core.iter().enumerate() {
                    for (support_index,support_id) in plan.support.iter().enumerate() {
                        for reversed in [false,true] {
                            let seed=2_610_201_u64
                                + (plan_index * 100_000 + enemy_index * 1_000
                                  + usize::from(upgraded) * 500 + core_index * 100
                                  + support_index * 10 + usize::from(reversed)) as u64;
                            let mut game=Game::new(plan.hero,seed);
                            game.apply(Action::Choose{index:1}).expect("battle fixture");
                            assert_eq!(game.stage,Stage::Combat);
                            let (first,second)=if reversed {
                                (*support_id,*core_id)
                            } else {(*core_id,*support_id)};
                            let mut c1=Card::new(first);
                            let mut c2=Card::new(second);
                            c1.upgraded=upgraded;
                            c2.upgraded=upgraded;
                            {
                                let b=game.battle.as_mut().unwrap();
                                b.foes.truncate(1);
                                let foe=&mut b.foes[0];
                                foe.id=(*enemy_id).into();
                                // Keep the target alive long enough to exercise both cards.
                                let hp=enemy_hp(enemy_id).max(3_000);
                                foe.hp=hp;
                                foe.max_hp=hp;
                                foe.block=5;
                                foe.weak=0;
                                foe.vulnerable=0;
                                foe.poison=0;
                                b.hand=vec![c1,c2];
                                b.draw=vec![Card::new("strike"),Card::new("defend"),
                                    Card::new("strike"),Card::new("defend")];
                                b.discard=vec![Card::new("strike"),Card::new("defend")];
                                b.energy=25;
                            }
                            for (index,card_id) in [first,second].iter().enumerate() {
                                // An X card spends the whole energy pool. Replenish
                                // only to make *both* combinations reachable as a
                                // mechanical fixture; do not interpret as game balance.
                                game.battle.as_mut().unwrap().energy=25;
                                game.apply(Action::Play{index:1,target:1})
                                    .unwrap_or_else(|err|panic!(
                                        "pair {}/{} enemy={} upgraded={} reversed={} step={} card={}: {err}",
                                        plan.id,plan.hero.name(),enemy_id,upgraded,reversed,index+1,card_id));
                                assert_eq!(game.stage,Stage::Combat,
                                    "fixture foe died while testing {}/{}",plan.id,enemy_id);
                            }
                            let b=game.battle.as_ref().unwrap();
                            assert!(b.events.windows(2).all(|pair|
                                pair[1].seq==pair[0].seq+1), "event gap: {}",plan.id);
                            for card_id in [first,second] {
                                assert!(b.events.iter().any(|event|
                                    event.kind==EventKind::CardPlayed && event.source==card_id),
                                    "card missing from event journal: {}/{}",plan.id,card_id);
                            }
                            assert!(game.event_history.as_ref().is_some_and(|hist|
                                hist.last().is_some_and(|tail| tail.event.seq==b.event_seq)),
                                "campaign archive lost pair effects: {}",plan.id);
                            let snapshot=game.to_json().unwrap();
                            assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),
                                snapshot, "invalid saved pair: {}/{}",plan.id,enemy_id);
                            checked+=1;
                        }
                    }
                }
            }
        }
    }
    let expected=PLANS.iter()
        .map(|plan|plan.core.len()*plan.support.len())
        .sum::<usize>()*ENEMY_IDS.len()*2*2;
    assert_eq!(checked,expected);
    assert!(checked>=14_000, "a build or enemy coverage axis went missing: {checked}");
}
