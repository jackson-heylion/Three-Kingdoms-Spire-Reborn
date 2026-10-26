//! Causal invariants for settled enemy death, summons and hero damage.
//! This reads the complete archived combat events, not action replay or Game.
//! It cannot authenticate forged events or reconstruct a whole Game.
use crate::{
    combat::EventKind,
    event_stack::{RunEvent, verify_record_coordinate},
};
use std::collections::HashSet;

fn foe_index(slot: Option<&str>) -> Option<u8> {
    match slot {
        Some("foe:0") => Some(0),
        Some("foe:1") => Some(1),
        Some("foe:2") => Some(2),
        Some("foe:3") => Some(3),
        _ => None,
    }
}

fn foe_slot(slot: Option<&str>) -> bool { foe_index(slot).is_some() }

/// A death must immediately follow a positive settled hit to the same foe
/// from the same source. Death-triggered powers are emitted *after* the seal.
/// Summoned enemies append slots; no slot can be summoned or killed twice.
/// The archive is immutable and this function has no side effects.
pub fn validate(archive: &[RunEvent]) -> Result<(), String> {
    let mut previous = None;
    let mut last: Option<&RunEvent> = None;
    let mut encounter = 0;
    let mut dead = HashSet::<String>::new();
    let mut summoned = HashSet::<String>::new();
    // Foes occupy append-only slots, even after death. Earlier references
    // prove a slot existed before a later claimed summon.
    let mut highest_seen_slot: Option<u8> = None;
    for record in archive {
        verify_record_coordinate(previous, record)?;
        if record.encounter != encounter {
            encounter = record.encounter;
            dead.clear();
            summoned.clear();
            highest_seen_slot = None;
            last = None;
        }
        let event = &record.event;
        match event.kind {
            EventKind::DamageDealt => {
                if !foe_slot(event.target.as_deref()) || event.amount < 0 {
                    return Err(format!("encounter {encounter}: invalid foe damage"));
                }
                if event.target.as_ref().is_some_and(|slot| dead.contains(slot)) {
                    return Err(format!("encounter {encounter}: damage after foe death"));
                }
            }
            EventKind::FoeDefeated => {
                if event.amount != 1 || !foe_slot(event.target.as_deref()) {
                    return Err(format!("encounter {encounter}: invalid foe death"));
                }
                if !last.is_some_and(|prior|
                    prior.event.kind == EventKind::DamageDealt
                        && prior.event.amount > 0
                        && prior.event.source == event.source
                        && prior.event.target == event.target)
                {
                    return Err(format!("encounter {encounter}: death without preceding hit"));
                }
                if !dead.insert(event.target.as_ref().expect("checked slot").clone()) {
                    return Err(format!("encounter {encounter}: duplicate foe death"));
                }
            }
            EventKind::FoeSummoned => {
                if event.amount != 1 || !foe_slot(event.target.as_deref()) {
                    return Err(format!("encounter {encounter}: invalid summon"));
                }
                let slot = event.target.as_ref().expect("checked slot");
                let index = foe_index(Some(slot)).expect("checked slot");
                if highest_seen_slot.is_some_and(|seen| index <= seen) {
                    return Err(format!("encounter {encounter}: summon reuses an observed foe slot"));
                }
                if event.source.is_empty() {
                    return Err(format!("encounter {encounter}: summon lacks foe identity"));
                }
                if dead.contains(slot) || !summoned.insert(slot.clone()) {
                    return Err(format!("encounter {encounter}: duplicate or revived summon"));
                }
            }
            EventKind::DamageReceived
                if event.amount < 0 || event.target.as_deref() != Some("hero") =>
            {
                return Err(format!("encounter {encounter}: invalid hero damage"));
            }
            _ => {}
        }
        // Damage/status targets and enemy attack sources prove prior occupancy.
        // A subsequent summon must append after every observed foe slot.
        for slot in [event.target.as_deref(), Some(event.source.as_str())] {
            if let Some(index) = foe_index(slot) {
                highest_seen_slot = Some(highest_seen_slot.map_or(index, |seen| seen.max(index)));
            }
        }
        previous = Some((record.encounter, record.floor, event.seq));
        last = Some(record);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Action, Game, content::Hero, combat::CombatEvent};

    fn record(encounter:u64,seq:u64,kind:EventKind,source:&str,
              amount:i32,target:Option<&str>)->RunEvent {
        RunEvent {encounter,floor:encounter as u8,event:CombatEvent {
            seq,kind,source:source.into(),amount,target:target.map(str::to_owned),
        }}
    }
    fn valid() -> Vec<RunEvent> {
        vec![
            record(1,1,EventKind::CardPlayed,"strike",1,None),
            record(1,2,EventKind::DamageDealt,"hero",5,Some("foe:0")),
            record(1,3,EventKind::FoeDefeated,"hero",1,Some("foe:0")),
            record(1,4,EventKind::PowerTriggered,"victory_drum",1,Some("foe:0")),
            record(1,5,EventKind::BattleEnded,"victory",1,None),
            record(2,1,EventKind::DamageReceived,"foe:0",3,Some("hero")),
            record(2,2,EventKind::FoeSummoned,"militia",1,Some("foe:2")),
        ]
    }
    #[test]
    fn accepts_live_campaign_archive_and_v2_roundtrip() {
        validate(&valid()).unwrap();
        let mut game=Game::new(Hero::GuanYu,20261026);
        game.apply(Action::Choose {index:1}).unwrap();
        validate(game.event_history.as_deref().unwrap()).unwrap();
        let loaded=Game::from_json(&game.to_json().unwrap()).unwrap();
        validate(loaded.event_history.as_deref().unwrap()).unwrap();
        loaded.audit_event_archive_authority().unwrap();
    }
    #[test]
    fn rejects_orphan_relabelled_or_zero_damage_deaths() {
        let mut events=valid();
        events[1].event.kind=EventKind::CardResolved;
        assert!(validate(&events).is_err());
        let mut events=valid();
        events[2].event.source="poison".into();
        assert!(validate(&events).is_err());
        let mut events=valid();
        events[2].event.target=Some("foe:1".into());
        assert!(validate(&events).is_err());
        let mut events=valid();
        events[1].event.amount=0;
        assert!(validate(&events).is_err());
    }
    #[test]
    fn rejects_dead_slot_reuse_and_duplicate_death() {
        let mut events=valid();
        events[3]=record(1,4,EventKind::DamageDealt,"hero",2,Some("foe:0"));
        assert!(validate(&events).is_err());
        let mut events=valid();
        events[3]=record(1,4,EventKind::FoeDefeated,"hero",1,Some("foe:0"));
        assert!(validate(&events).is_err());
    }
    #[test]
    fn rejects_summons_into_slots_already_proven_occupied() {
        for preceding in [
            record(1,1,1,EventKind::DamageDealt,"hero",2,Some("foe:1")),
            record(1,1,1,EventKind::DamageReceived,"foe:2",2,Some("hero")),
            record(1,1,1,EventKind::StatusApplied,"poison",1,Some("foe:3")),
        ] {
            let attempted = record(1,2,EventKind::FoeSummoned,"militia",1,Some("foe:1"));
            let mut attempted = attempted;
            attempted.floor = 1;
            assert!(validate(&[preceding, attempted]).unwrap_err()
                .contains("reuses an observed foe slot"));
        }
    }

    #[test]
    fn permits_append_only_summons_and_resets_slot_tracking_per_encounter() {
        let archive = vec![
            record(1,1,1,EventKind::DamageDealt,"hero",3,Some("foe:0")),
            record(1,2,EventKind::FoeSummoned,"militia",1,Some("foe:1")),
            record(1,3,EventKind::DamageDealt,"hero",2,Some("foe:1")),
            record(1,4,EventKind::FoeSummoned,"militia",1,Some("foe:2")),
            record(2,1,EventKind::FoeSummoned,"militia",1,Some("foe:1")),
        ];
        validate(&archive).unwrap();
        let mut no_identity = archive;
        no_identity[1].event.source.clear();
        assert!(validate(&no_identity).unwrap_err().contains("lacks foe identity"));
    }

    #[test]
    fn rejects_invalid_or_duplicate_summons_and_hero_damage() {
        for slot in ["foe:01","foe:4","hero"] {
            let mut events=valid();
            events[6].event.target=Some(slot.into());
            assert!(validate(&events).is_err());
        }
        let mut events=valid();
        events[6].event.amount=2;
        assert!(validate(&events).is_err());
        let mut events=valid();
        events[5].event.target=Some("foe:0".into());
        assert!(validate(&events).is_err());
        let mut events=valid();
        events[5].event.amount=-1;
        assert!(validate(&events).is_err());
        let mut events=valid();
        events.push(record(2,3,EventKind::FoeSummoned,"militia",1,Some("foe:2")));
        assert!(validate(&events).is_err());
    }
}
