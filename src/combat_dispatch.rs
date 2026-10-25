//! Synchronous combat event dispatch, extracted from the oversized rules engine.
//! Nested emissions carry an opt-in parent edge only during offline replay.
//! This is not a general effect-stack graph or authoritative event sourcing.
use crate::*;

impl Game {
    pub(crate) fn combat_event(&mut self, kind: EventKind, source: &str, amount: i32, target: Option<String>) {
        // One synchronous, bounded trigger pipeline. All observers see the original
        // event before any subsequent PowerTriggered or CardDrawn events.
        let Some(b) = &mut self.battle else { return };
        b.event_seq = b.event_seq.saturating_add(1);
        let event = CombatEvent {
            seq: b.event_seq, kind, source: source.to_owned(), amount, target,
        };
        let coordinate = event_parent::EventCoordinate {
            encounter: self.telemetry.totals.battles_started,
            seq: event.seq,
        };
        let record = event_stack::RunEvent {
            encounter: coordinate.encounter, floor: self.floor, event: event.clone(),
        };
        if let Some(history) = self.event_history.as_mut() {
            history.push(record.clone());
        }
        // Only an event synchronously emitted while dispatching another event
        // has a known parent. Sequential sibling effects have no invented edge.
        if let Some(trace) = self.event_parent_trace.as_mut() {
            trace.push(event_parent::EventParentRecord {
                event: record, parent: self.dispatch_parent,
            });
        }
        b.events.push(event.clone());
        if b.events.len() > MAX_RECENT_EVENTS {
            let count = b.events.len() - MAX_RECENT_EVENTS;
            b.events.drain(..count);
        }
        let previous = self.dispatch_parent.replace(coordinate);
        self.dispatch_combat_event(&event);
        self.dispatch_parent = previous;
    }

    pub(crate) fn attack_effect(effect: Effect) -> bool {
        matches!(effect,
            Effect::Hit(_) | Effect::HitDraw(_,_) | Effect::HitTwice(_)
            | Effect::All(_) | Effect::HitWeak(_,_) | Effect::HitVulnerable(_,_)
            | Effect::MomentumStrike(_) | Effect::SchemeStrike(_) | Effect::ComboStrike(_)
            | Effect::CommandStrike(_) | Effect::FuryStrike(_) | Effect::BloodPrice(_,_)
            | Effect::Leech(_) | Effect::HitPoison(_,_) | Effect::Pierce(_)
            | Effect::DiscardStrike(_) | Effect::XHit(_)
        )
    }

    fn dispatch_combat_event(&mut self, event: &CombatEvent) {
        if event.kind==EventKind::CardResolved {
            use event_stack::{Listener,Response};
            let reaction=match event.source.as_str() {
                "hold_line"=>Some(Listener::once("hold_line:weak","weak",Response::Cancel)),
                "kongcheng"=>Some(Listener::once("kongcheng:vulnerable","vulnerable",Response::Reduce(1))),
                "qiaoying"=>Some(Listener::once("qiaoying:weak","weak",Response::Reduce(1))),
                "zhongyi"=>Some(Listener::once("zhongyi:weak","weak",Response::Replace(1))),
                _=>None,
            };
            if let Some(rule)=reaction {
                self.battle.as_mut().expect("battle").reactions.install(rule)
                    .expect("valid builtin listener");
            }
        }
        match event.kind {
            EventKind::CardResolved => {
                let attack = card_spec(&event.source).is_some_and(|card| Self::attack_effect(card.effect));
                if attack {
                    let guard = self.power_stacks(PowerId::OathGuard) * 3;
                    if guard > 0 {
                        self.battle.as_mut().expect("battle").block += guard;
                        self.combat_event(EventKind::PowerTriggered, "oath_guard", guard, None);
                    }
                } else {
                    let stacks = self.power_stacks(PowerId::ThoughtSpring);
                    let unused = self.battle.as_ref().is_some_and(|b| !b.thought_used);
                    if stacks > 0 && unused {
                        let b = self.battle.as_mut().expect("battle");
                        b.energy += stacks;
                        b.thought_used = true;
                        self.combat_event(EventKind::PowerTriggered, "thought_spring", stacks, None);
                        self.draw_cards(1);
                    }
                }
            }
            EventKind::CardDiscarded => {
                let amount = self.power_stacks(PowerId::ArrowsOnDiscard) * 4;
                if amount > 0 {
                    let target = self.battle.as_ref().expect("battle").foes.iter()
                        .position(|foe| foe.hp > 0);
                    if let Some(idx) = target {
                        self.combat_event(EventKind::PowerTriggered,"arrows_on_discard",
                            amount,Some(format!("foe:{idx}")));
                        self.damage_enemy_fixed(idx, amount, "arrows_on_discard");
                    }
                }
            }
            EventKind::TurnEnded => self.trigger_granary(),
            EventKind::DamageReceived if event.amount > 0 => {
                let stacks = self.power_stacks(PowerId::BloodCourage);
                let fury = stacks * 2;
                if fury > 0 {
                    self.battle.as_mut().expect("battle").fury += fury;
                    self.combat_event(EventKind::PowerTriggered, "blood_courage", fury, None);
                }
            }
            EventKind::CardDrawn => {
                let amount = self.power_stacks(PowerId::WisdomOnDraw);
                if amount > 0 {
                    self.battle.as_mut().expect("battle").block += amount;
                    self.combat_event(EventKind::PowerTriggered,"wisdom_on_draw",amount,None);
                }
            }
            EventKind::StatusApplied if event.source == "poison" && event.amount > 0 => {
                let amount = self.power_stacks(PowerId::ToxicReversal);
                let idx = event.target.as_deref()
                    .and_then(|text| text.strip_prefix("foe:"))
                    .and_then(|text| text.parse::<usize>().ok());
                if let Some(idx) = idx.filter(|_| amount > 0) {
                    // The bonus is a separate status proposal. It must pass
                    // through the same reaction stack as the original poison,
                    // but its distinct source must NOT trigger this power again.
                    let capacity = self.battle.as_ref().expect("battle").foes.get(idx)
                        .map_or(0, |foe| if foe.hp > 0 {
                            9999i32.saturating_sub(foe.poison)
                        } else { 0 });
                    if capacity > 0 {
                        let target = format!("foe:{idx}");
                        if let Some(adjusted) = self.preflight_status(
                            "poison:toxic_reversal", &target, amount.min(capacity)) {
                            let applied = self.battle.as_mut().expect("battle").foes
                                .get_mut(idx).map_or(0, |foe| {
                                    if foe.hp <= 0 { return 0; }
                                    let before = foe.poison;
                                    foe.poison = foe.poison.saturating_add(adjusted).min(9999);
                                    foe.poison - before
                                });
                            if applied > 0 {
                                self.combat_event(EventKind::PowerTriggered, "toxic_reversal",
                                    applied, Some(target.clone()));
                                self.combat_event(EventKind::StatusApplied,
                                    "poison:toxic_reversal", applied, Some(target));
                            }
                        }
                    }
                }
            }
            EventKind::FoeDefeated => {
                let amount = self.power_stacks(PowerId::VictoryDrum);
                if amount > 0 {
                    self.battle.as_mut().expect("battle").energy += amount;
                    self.combat_event(EventKind::PowerTriggered,"victory_drum",amount,event.target.clone());
                }
            }
            _ => {}
        }
    }
}
