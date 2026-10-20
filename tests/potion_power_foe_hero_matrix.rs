//! 10 real potion types × all 8 persistent Powers × 31 foe IDs × five
//! heroes = 12,400 actual potion actions, with numeric effect oracles.
//! These are injected encounter/power fixtures, not natural-drafting,
//! randomized intent exhaustiveness, or a proof of arbitrary combinations.
use three_kingdoms_spire_reborn::{
    Action, Game, Stage,
    combat::{EventKind, PowerId, PowerStack},
    content::{Hero, ENEMY_IDS},
    items::POTIONS,
};

const HEROES:[Hero;5]=[
    Hero::GuanYu,Hero::ZhugeLiang,Hero::SunShangxiang,Hero::CaoCao,Hero::LuBu
];
const POWERS:[PowerId;8]=[
    PowerId::OathGuard,PowerId::ThoughtSpring,
    PowerId::ArrowsOnDiscard,PowerId::GranarySupport,
    PowerId::BloodCourage,PowerId::WisdomOnDraw,
    PowerId::ToxicReversal,PowerId::VictoryDrum
];

#[test]
fn every_potion_power_foe_hero_combination_has_correct_immediate_delta() {
    let mut cases=0;
    assert_eq!(POTIONS.len(),10);
    assert_eq!(ENEMY_IDS.len(),31);
    for (h,hero) in HEROES.iter().enumerate() {
        for (p,power) in POWERS.iter().enumerate() {
            for (f,enemy) in ENEMY_IDS.iter().enumerate() {
                for (n,(potion,_name)) in POTIONS.iter().enumerate() {
                    let seed=26_102_000_u64 + (h*100_000+p*10_000+f*100+n) as u64;
                    let mut game=Game::new(*hero,seed);
                    game.apply(Action::Choose {index:1}).expect("valid opening combat");
                    assert_eq!(game.stage,Stage::Combat);
                    game.hp=game.max_hp-20;
                    game.potions=1;
                    game.potion_bag=vec![(*potion).into()];
                    {
                        let battle=game.battle.as_mut().unwrap();
                        battle.foes.truncate(1);
                        battle.foes[0].id=(*enemy).into();
                        battle.foes[0].max_hp=1_000;
                        battle.foes[0].hp=1_000;
                        battle.foes[0].block=5;
                        battle.foes[0].weak=0;
                        battle.foes[0].vulnerable=0;
                        battle.foes[0].poison=0;
                        battle.powers=vec![PowerStack{id:*power,stacks:1}];
                        battle.weak=2;
                        battle.vulnerable=2;
                    }
                    let (start_seq,start_block,start_strength,start_energy,start_hand)={
                        let battle=game.battle.as_ref().unwrap();
                        (battle.event_seq,battle.block,battle.strength,battle.energy,battle.hand.len())
                    };
                    let hp=game.hp;
                    game.apply(Action::UsePotionAt{slot:1,target:1}).unwrap_or_else(|err|
                        panic!("hero={} power={power:?} enemy={enemy} potion={potion}: {err}",
                            hero.name()));
                    assert!(game.potion_bag.is_empty() && game.potions==0,
                        "consumption: {potion}/{power:?}/{enemy}");
                    assert_eq!(game.stage,Stage::Combat,
                        "test target must survive: {potion}/{enemy}");
                    let battle=game.battle.as_ref().unwrap();
                    let events=battle.events.iter()
                        .filter(|event| event.seq>start_seq).collect::<Vec<_>>();
                    assert!(!events.is_empty());
                    assert_eq!(events[0].seq,start_seq+1);
                    assert_eq!(events[0].kind,EventKind::PotionUsed);
                    assert_eq!(events[0].source,*potion);
                    assert_eq!(events[0].amount,1);
                    assert!(events.windows(2).all(|pair|pair[1].seq==pair[0].seq+1));
                    assert_eq!(battle.powers.len(),1);
                    assert_eq!(battle.powers[0].id,*power);

                    // Effect oracles test actual numeric semantics (not just
                    // an event count or a no-crash property).
                    let expected_hp=match *potion {
                        "healing"=>hp+16,
                        "cleanse"=>hp+8,
                        "blood"=>hp-4,
                        _=>hp,
                    };
                    assert_eq!(game.hp,expected_hp,"HP {potion}/{power:?}/{enemy}");
                    // WisdomOnDraw reacts to each card actually drawn by
                    // Insight, independently of the potion's own value.
                    let drawn_guard=if *potion=="insight"
                        && *power==PowerId::WisdomOnDraw {3}else{0};
                    assert_eq!(battle.block,start_block
                        +if *potion=="shield"{20}else{0}+drawn_guard,
                        "armor {potion}/{power:?}/{enemy}");
                    assert_eq!(battle.energy,start_energy+if *potion=="energy"{2}else{0},
                        "energy {potion}/{power:?}/{enemy}");
                    assert_eq!(battle.strength,start_strength+match *potion {
                        "strength"=>3,"blood"=>5,_=>0,
                    },"strength {potion}/{power:?}/{enemy}");
                    if *potion=="insight" {
                        assert_eq!(battle.hand.len(),start_hand+3,"insight card gain");
                    }
                    if *potion=="cleanse" {
                        assert_eq!((battle.weak,battle.vulnerable),(0,0));
                    } else {
                        assert_eq!((battle.weak,battle.vulnerable),(2,2));
                    }
                    let foe=&battle.foes[0];
                    assert_eq!(foe.block,if *potion=="fire" {0}else{5});
                    assert_eq!(foe.hp,if *potion=="fire" {983}else{1_000});
                    assert_eq!(foe.weak,if *potion=="weak" {3}else{0});
                    assert_eq!(foe.poison,if *potion=="poison" {
                        12+if *power==PowerId::ToxicReversal {1}else{0}
                    } else {0});
                    assert!(game.event_history.as_ref().is_some_and(|history|
                        history.last().is_some_and(|tail|
                            tail.event.seq==battle.event_seq && tail.event==*battle.events.last().unwrap())),
                        "full event archive diverged: {potion}/{power:?}/{enemy}");
                    let snapshot=game.to_json().unwrap();
                    assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot,
                        "save roundtrip: {potion}/{power:?}/{enemy}");
                    cases+=1;
                }
            }
        }
    }
    assert_eq!(cases,10*8*31*5);
    assert_eq!(cases,12_400);
}
