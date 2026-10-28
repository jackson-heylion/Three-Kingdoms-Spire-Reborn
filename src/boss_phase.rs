//! Boss and elite second-health-bar transitions. Only authoritative Game actions
//! reach this method; the HUD does not trigger phases or modify health.
use super::{Card, EventKind, Foe, Game, Intent, enemy_hp, enemy_name};

impl Game {
    pub(crate) fn boss_phase_transitions(&mut self) -> Vec<String> {
        let changes: Vec<(usize,String)> = self.battle.as_ref().expect("battle").foes
            .iter().enumerate().filter(|(_,foe)|
                foe.hp > 0 && foe.phase == 0 && foe.hp <= foe.max_hp / 2
                && matches!(foe.id.as_str(),"zhangjiao"|"sunjian"|"dongzhuo"|"yuanshao"|"simayi"|"zuoci"
                    |"elite"|"war_elephant"|"royal_guard"|"tiger_general"|"sacred_beast"))
            .map(|(i,foe)| (i,foe.id.clone())).collect();
        let mut messages = vec![];
        for (idx,id) in changes {
            let b = self.battle.as_mut().expect("battle");
            let mut summoned: Option<(&'static str, usize)> = None;
            b.foes[idx].phase = 1;
            // A phase is a new full-health bar, not a partial heal.
            // Other foes remain untouched, and phase==1 prevents repeated refills.
            let previous_hp = b.foes[idx].hp;
            b.foes[idx].hp = b.foes[idx].max_hp;
            let restored = b.foes[idx].hp - previous_hp;
            match id.as_str() {
                "zhangjiao" | "yuanshao" | "zuoci" => {
                    let ally = match id.as_str() {
                        "zhangjiao" => "yellow_turban",
                        "yuanshao" => "officer", _ => "shaman",
                    };
                    if b.foes.len() < 4 {
                        let hp = enemy_hp(ally);
                        let new_index = b.foes.len();
                        b.foes.push(Foe {
                            id: ally.into(), hp, max_hp: hp, block: 0, strength: 0,
                            poison: 0, vulnerable: 0, weak: 0,
                            intent: Intent::Guard(4), phase: 0,
                        });
                        summoned = Some((ally, new_index));
                    }
                    messages.push(format!("{}进入第二阶段{}",
                        enemy_name(&id),
                        if summoned.is_some() {"并呼叫援军（援军下回合开始行动）"}
                        else {"，但战场编制已满，无法增援"}));
                }
                "sunjian" => {
                    b.foes[idx].strength += 4;
                    b.foes[idx].block += 8;
                    messages.push("孙坚激发虎威，获得4武勇和8护甲".into());
                }
                "dongzhuo" => {
                    b.foes[idx].strength += 3;
                    b.weak += 2;
                    self.gold = (self.gold - 30).max(0);
                    messages.push("董卓暴虐：我军虚弱两回合并损失最多30金".into());
                }
                "simayi" => {
                    b.discard.push(Card::new("cursed"));
                    b.discard.push(Card::new("cursed"));
                    b.vulnerable += 2;
                    messages.push("司马懿奇袭：两张军心涣散混入弃牌堆，我军易伤两回合".into());
                }
                "elite" => {
                    b.foes[idx].strength += 2;
                    b.foes[idx].block += 6;
                    messages.push("精锐校尉下令死战：武勇+2、护甲+6".into());
                }
                "war_elephant" => {
                    b.foes[idx].strength += 2;
                    b.foes[idx].block += 14;
                    messages.push("战象披甲冲阵：武勇+2、护甲+14".into());
                }
                "royal_guard" => {
                    b.foes[idx].strength += 1;
                    b.foes[idx].block += 10;
                    for (ally_index, ally) in b.foes.iter_mut().enumerate() {
                        if ally_index != idx && ally.hp > 0 { ally.block += 5; }
                    }
                    messages.push("禁卫整备铁壁：自身护甲+10，存活友军护甲+5".into());
                }
                "tiger_general" => {
                    b.foes[idx].strength += 3;
                    b.foes[idx].block += 5;
                    messages.push("虎将奋威：武勇+3、护甲+5，后续预告三连虎袭".into());
                }
                "sacred_beast" => {
                    let foe = &mut b.foes[idx];
                    foe.block += 6;
                    messages.push("灵兽回生：护甲+6，后续加强诡术".into());
                }
                _ => {}
            }
            // The exact restored HP is visible to players and combat reports.
            if let Some(message) = messages.last_mut() {
                *message = format!("【二阶段】{}恢复{restored}生命至满血（{}/{}）；{message}",
                    enemy_name(&id), b.foes[idx].hp, b.foes[idx].max_hp);
            }
            if let Some((ally, new_index)) = summoned {
                self.combat_event(EventKind::FoeSummoned, ally, 1,
                    Some(format!("foe:{new_index}")));
            }
            // Reuse the existing event kind; no save-schema or replay protocol change.
            self.combat_event(EventKind::EnemyTechnique, &format!("{id}_phase_two"), 1,
                Some(format!("foe:{idx}")));
        }
        messages
    }

}

#[cfg(test)]
mod phase_refill_tests {
    use super::*;
    use crate::{Action, Node, Stage, content::Hero};

    const PHASED: [&str; 11] = [
        "zhangjiao", "sunjian", "dongzhuo", "yuanshao", "simayi", "zuoci",
        "elite", "war_elephant", "royal_guard", "tiger_general", "sacred_beast",
    ];

    #[test]
    fn every_boss_and_elite_refills_exactly_once_and_keeps_existing_effects() {
        for id in PHASED {
            let mut game = Game::new(Hero::GuanYu, 20261028);
            game.floor = 7;
            game.start_battle(Node::Boss);
            {
                let foe = &mut game.battle.as_mut().unwrap().foes[0];
                foe.id = id.into();
                foe.max_hp = 91;
                foe.hp = 30;
                foe.phase = 0;
            }
            let effects = game.boss_phase_transitions();
            assert_eq!(effects.len(), 1, "{id}");
            assert!(effects[0].contains("恢复61生命至满血"), "{effects:?}");
            let b = game.battle.as_ref().unwrap();
            assert_eq!(b.foes[0].hp, 91, "{id}");
            assert_eq!(b.foes[0].phase, 1, "{id}");
            assert!(b.events.iter().any(|event|
                event.kind == EventKind::EnemyTechnique
                    && event.source == format!("{id}_phase_two")), "{id}");
            let snapshot = game.to_json().unwrap();
            assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
            assert!(game.boss_phase_transitions().is_empty(), "{id} refilled twice");
            assert_eq!(game.battle.as_ref().unwrap().foes[0].hp, 91);
        }
    }

    #[test]
    fn zero_hp_and_enemy_that_is_not_phased_never_refill() {
        let mut game = Game::new(Hero::GuanYu, 20261028);
        game.floor=7;
        game.start_battle(Node::Boss);
        let b = game.battle.as_mut().unwrap();
        b.foes[0].id = "zhangjiao".into();
        b.foes[0].hp = 0;
        assert!(game.boss_phase_transitions().is_empty());
        assert_eq!(game.battle.as_ref().unwrap().foes[0].hp, 0);
        let b = game.battle.as_mut().unwrap();
        b.foes[0].hp = 1;
        b.foes[0].phase = 1;
        assert!(game.boss_phase_transitions().is_empty());
        assert_eq!(game.battle.as_ref().unwrap().foes[0].hp, 1);
        assert_eq!(game.stage, Stage::Combat);
    }

    #[test]
    fn end_turn_phase_refill_is_persisted_and_only_affects_triggered_foe() {
        let mut game = Game::new(Hero::GuanYu, 8181);
        game.floor = 7;
        game.start_battle(Node::Boss);
        let b = game.battle.as_mut().unwrap();
        b.foes[0].id = "sunjian".into();
        b.foes[0].hp = 30;
        b.foes[0].max_hp = 100;
        b.foes[0].intent = Intent::Guard(0);
        b.foes[0].phase = 0;
        b.troops_attack = 0;
        b.troops_hp = 0;
        let response = game.apply(Action::EndTurn).unwrap().join("；");
        assert!(response.contains("恢复70生命至满血"), "{response}");
        let b = game.battle.as_ref().unwrap();
        assert_eq!(b.foes[0].hp, 100);
        assert_eq!(b.foes[0].phase, 1);
        assert_eq!(Game::from_json(&game.to_json().unwrap()).unwrap().battle.unwrap().foes[0].hp,100);
    }
}
