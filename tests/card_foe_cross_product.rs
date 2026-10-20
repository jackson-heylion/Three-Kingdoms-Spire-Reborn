//! Cross-product smoke tests for every collectible card in each class's pool
//! against all 31 enemy IDs. These injected fixtures are NOT natural builds or
//! statistically sampled battles; they verify the authoritative interaction path.
use std::collections::BTreeSet;
use three_kingdoms_spire_reborn::{
    Action,Card,Game,Intent,Stage,
    content::{self,Hero,ENEMY_IDS,NEUTRAL_POOL,REWARD_POOL,hero_pool,card_spec},
};

const HEROES:[Hero;5]=[
    Hero::GuanYu,Hero::ZhugeLiang,Hero::SunShangxiang,Hero::CaoCao,Hero::LuBu
];
fn available_cards(hero:Hero)->BTreeSet<&'static str> {
    let mut result:BTreeSet<&'static str> =
        ["strike","defend"].into_iter().collect();
    result.extend(REWARD_POOL.iter().copied());
    result.extend(NEUTRAL_POOL.iter().copied());
    result.extend(hero_pool(hero).iter().copied());
    result
}
#[test]
fn every_collectible_class_card_plays_against_every_enemy_with_valid_snapshot() {
    let mut cases=0usize;
    for (hero_number,hero) in HEROES.into_iter().enumerate() {
        let cards=available_cards(hero);
        assert!(cards.len()>40,"unexpectedly incomplete pool: {}",hero.name());
        for (card_number,card_id) in cards.iter().enumerate() {
            let spec=card_spec(card_id).unwrap_or_else(||panic!("invalid card {card_id}"));
            assert!(spec.cost>=-1 && spec.cost<=5,"unexpected card cost: {card_id}");
            for (foe_number,foe_id) in ENEMY_IDS.iter().enumerate() {
                for upgraded in [false,true] {
                    // This state is intentionally a legal but injected combat test
                    // fixture. It must not be logged as a naturally acquired deck.
                    let seed=2_610_200_u64
                        + (hero_number*10_000+card_number*100+foe_number*2
                            + usize::from(upgraded)) as u64;
                    let mut g=Game::new(hero,seed);
                    g.apply(Action::Choose{index:1}).expect("enter combat");
                    assert_eq!(g.stage,Stage::Combat);
                    let b=g.battle.as_mut().expect("combat fixture");
                    b.foes.truncate(1);
                    b.foes[0].id=(*foe_id).to_owned();
                    let hp=content::enemy_hp(foe_id).saturating_add(200);
                    b.foes[0].hp=hp;
                    b.foes[0].max_hp=hp;
                    b.foes[0].block=5;
                    b.foes[0].poison=0;
                    b.foes[0].weak=0;
                    b.foes[0].vulnerable=0;
                    b.foes[0].intent=Intent::Guard(1);
                    let mut card=Card::new(card_id);
                    card.upgraded=upgraded;
                    b.hand=vec![card];
                    b.draw=vec![Card::new("strike"),Card::new("defend")];
                    b.discard=vec![Card::new("strike")];
                    b.energy=6;
                    let before_seq=b.event_seq;
                    let action=Action::Play{index:1,target:1};
                    g.apply(action).unwrap_or_else(|err|panic!(
                        "card {} (upgraded={upgraded}) vs foe {} for {} rejected: {err}",
                        card_id,foe_id,hero.name()));
                    assert_eq!(g.stage,Stage::Combat,"unexpected battle finish {card_id}/{foe_id}");
                    let b=g.battle.as_ref().unwrap();
                    assert!(b.event_seq>before_seq,"card {card_id} emitted no event");
                    assert!(b.events.windows(2).all(|pair|pair[0].seq+1==pair[1].seq));
                    assert!(b.events.iter().any(|e|
                        e.kind==three_kingdoms_spire_reborn::combat::EventKind::CardPlayed
                        && e.source==*card_id));
                    let snapshot=g.to_json().unwrap();
                    assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),
                        snapshot,"save mismatch: {card_id}/{foe_id}");
                    cases+=1;
                }
            }
        }
    }
    let expected=HEROES.iter().map(|h|available_cards(*h).len()).sum::<usize>()
        * ENEMY_IDS.len() * 2;
    assert_eq!(cases,expected);
    assert!(cases>=12_000,"cross-product coverage unexpectedly shrank: {cases}");
}
#[test]
fn all_collectible_cards_reject_an_unsupported_secondary_choice_atomically() {
    let mut cases=0;
    for hero in HEROES {
        for card_id in available_cards(hero) {
            let mut g=Game::new(hero,7199);
            g.apply(Action::Choose{index:1}).unwrap();
            let b=g.battle.as_mut().unwrap();
            b.hand=vec![Card::new(card_id)];
            b.energy=6;
            b.draw=vec![Card::new("strike")];
            b.discard=vec![Card::new("strike")];
            let before=g.to_json().unwrap();
            let rejected=g.apply(Action::PlaySelected{
                index:1,target:1,selection:999,
            });
            assert!(rejected.is_err(),"unsupported pile selection accepted: {card_id}");
            assert_eq!(g.to_json().unwrap(),before,"rollback failed: {card_id}");
            cases+=1;
        }
    }
    assert!(cases>=200,"missing card coverage: {cases}");
}
