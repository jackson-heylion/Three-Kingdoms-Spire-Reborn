//! Regression tests for replayable, pre-armor foe damage interception.
//! The registered rules are an unranked custom-mode facility, not balance inputs.
use super::*;
use crate::event_stack::{Listener,Response};

fn custom(hero:Hero, seed:u64)->Game {
    let mut g=Game::new_with_mode(hero,seed,0,RunMode::Custom);
    g.apply(Action::Choose{index:1}).expect("enter first combat");
    assert_eq!(g.stage,Stage::Combat);
    g
}
fn rule(id:&str,source:&str,effect:Response)->Listener {
    let mut r=Listener::once(id,source,effect);
    r.kind=EventKind::DamageDealt;
    r.target=Some("foe:0".into());
    r
}
#[test]
fn cancelled_hit_does_not_spend_armor_or_trigger_defeat() {
    let mut g=custom(Hero::GuanYu,38181);
    let b=g.battle.as_mut().unwrap();
    b.foes[0].block=20;
    b.foes[0].hp=10;
    let before_hp=b.foes[0].hp;
    g.apply(Action::RegisterReaction{listener:rule("deny-fire","fire",Response::Cancel)})
        .unwrap();
    let seq=g.battle.as_ref().unwrap().event_seq;
    g.damage_enemy_fixed(0,30,"fire");
    let b=g.battle.as_ref().unwrap();
    assert_eq!(b.foes[0].hp,before_hp);
    assert_eq!(b.foes[0].block,20);
    assert!(b.reactions.listeners.is_empty());
    let events=b.events.iter().filter(|e|e.seq>seq).collect::<Vec<_>>();
    assert_eq!(events.iter().map(|e|e.kind).collect::<Vec<_>>(),
        vec![EventKind::EventCountered,EventKind::DamageDealt]);
    assert_eq!(events[1].amount,0);
    assert_eq!(events.iter().filter(|e|e.kind==EventKind::FoeDefeated).count(),0);
    g.damage_enemy_fixed(0,30,"fire");
    assert_eq!(g.battle.as_ref().unwrap().foes[0].hp,0);
}
#[test]
fn priorities_modify_raw_damage_before_armor_and_audit_every_step() {
    let mut g=custom(Hero::CaoCao,38182);
    {
        let b=g.battle.as_mut().unwrap();
        b.foes[0].hp=35;
        b.foes[0].max_hp=35;
        b.foes[0].block=2;
    }
    let mut reduce=rule("early","hero",Response::Reduce(5));
    reduce.priority=-5;
    let mut replace=rule("late","hero",Response::Replace(9));
    replace.priority=5;
    g.apply(Action::RegisterReaction{listener:replace}).unwrap();
    g.apply(Action::RegisterReaction{listener:reduce}).unwrap();
    let seq=g.battle.as_ref().unwrap().event_seq;
    g.deal_enemy_damage(0,12,"hero");
    let b=g.battle.as_ref().unwrap();
    assert_eq!(b.foes[0].hp,28); // 12 -> 7 -> 9, then 2 armor = 7 HP
    assert_eq!(b.foes[0].block,0);
    let events=b.events.iter().filter(|e|e.seq>seq).collect::<Vec<_>>();
    assert_eq!(events.iter().map(|e|e.kind).collect::<Vec<_>>(),
        vec![EventKind::EventReplaced,EventKind::EventReplaced,EventKind::DamageDealt]);
    assert_eq!((events[0].source.as_str(),events[0].amount),("early",5));
    assert_eq!((events[1].source.as_str(),events[1].amount),("late",-2));
    assert_eq!(events[2].amount,7);
}
#[test]
fn poison_tick_cancellation_spends_poison_but_not_armor_or_hp() {
    let mut g=custom(Hero::ZhugeLiang,38183);
    {
        let b=g.battle.as_mut().unwrap();
        b.foes[0].hp=24;
        b.foes[0].max_hp=24;
        b.foes[0].poison=3;
        b.foes[0].block=8;
        b.foes[0].intent=Intent::Guard(0);
        b.troops_hp=0;
    }
    g.apply(Action::RegisterReaction{listener:rule("antidote","poison",Response::Cancel)})
        .unwrap();
    // EndTurn removes old armor after resolving poison. Measure HP, not final armor.
    let seq=g.battle.as_ref().unwrap().event_seq;
    g.apply(Action::EndTurn).unwrap();
    let b=g.battle.as_ref().unwrap();
    assert_eq!(b.foes[0].hp,24);
    assert_eq!(b.foes[0].poison,2);
    let events=b.events.iter().filter(|e|e.seq>seq).collect::<Vec<_>>();
    assert!(events.iter().any(|e| e.kind==EventKind::EventCountered && e.source=="antidote"));
    assert!(events.iter().any(|e| e.kind==EventKind::DamageDealt && e.source=="poison" && e.amount==0));
    assert!(!events.iter().any(|e|e.kind==EventKind::FoeDefeated));
}
#[test]
fn piercing_replacement_preserves_bypass_and_defeat_triggers_once() {
    let mut g=custom(Hero::LuBu,38184);
    g.gain_power(PowerId::VictoryDrum,1);
    {
        let b=g.battle.as_mut().unwrap();
        b.foes[0].hp=5;
        b.foes[0].block=60;
    }
    g.apply(Action::RegisterReaction{listener:rule("soften","pierce",Response::Replace(2))})
        .unwrap();
    let seq=g.battle.as_ref().unwrap().event_seq;
    g.deal_enemy_damage_unblockable(0,20,"pierce");
    assert_eq!(g.battle.as_ref().unwrap().foes[0].hp,3);
    assert_eq!(g.battle.as_ref().unwrap().foes[0].block,60);
    g.deal_enemy_damage_unblockable(0,20,"pierce");
    g.deal_enemy_damage_unblockable(0,20,"pierce"); // dead foe does not emit or spend
    let b=g.battle.as_ref().unwrap();
    let events=b.events.iter().filter(|e|e.seq>seq).collect::<Vec<_>>();
    assert_eq!(events.iter().filter(|e|e.kind==EventKind::FoeDefeated).count(),1);
    assert_eq!(events.iter().filter(|e|e.kind==EventKind::PowerTriggered
        && e.source=="victory_drum").count(),1);
    assert_eq!(b.foes[0].block,60);
}
#[test]
fn damage_rule_actions_roundtrip_replay_and_reject_invalid_target_or_source() {
    // Find a deterministic starter hand containing a strike without manipulating
    // the authoritative state, so action replay covers an actual intercepted hit.
    let mut g=(1..100).map(|seed|custom(Hero::GuanYu,seed))
        .find(|g|g.battle.as_ref().unwrap().hand.iter().any(|c|c.id=="strike"))
        .expect("one starter hand must contain a strike");
    let action=Action::RegisterReaction{listener:rule("guard","hero",Response::Reduce(10))};
    g.apply(action).unwrap();
    let before=g.to_json().unwrap();
    for bad in [rule("guard","hero",Response::Replace(-1)),
        {let mut r=rule("guard","unknown",Response::Cancel);r.target=Some("hero".into());r},
        {let mut r=rule("guard","fire",Response::Cancel);r.kind=EventKind::FoeDefeated;r}]
    {
        assert!(g.apply(Action::RegisterReaction{listener:bad}).is_err());
        assert_eq!(g.to_json().unwrap(),before);
    }
    let index=g.battle.as_ref().unwrap().hand.iter().position(|c|c.id=="strike").unwrap()+1;
    let seq=g.battle.as_ref().unwrap().event_seq;
    g.apply(Action::Play{index,target:1}).unwrap();
    assert!(g.event_history.as_ref().unwrap().iter().any(|record|
        record.event.seq>seq && record.event.source=="guard"
            && record.event.kind==EventKind::EventReplaced));
    let snapshot=g.to_json().unwrap();
    assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
    assert_eq!(g.verify_action_replay().unwrap(),3);
    assert_eq!(g.reconstruct_event_history().unwrap(),g.event_history.clone().unwrap());
}
