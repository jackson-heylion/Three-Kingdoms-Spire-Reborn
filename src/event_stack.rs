//! Serializable, deterministic pre-resolution reaction stack.
//! Registered by successful card resolutions; no hidden wall-clock callbacks.
use crate::combat::{CombatEvent, EventKind};
use serde::{Deserialize, Serialize};

// Public, inspectable trace type; the replay engine itself remains internal.
pub use crate::replay::{ActionEventTrace, RevisionCheckpoint};

// Read-only, event-only projection. This does not rebuild authoritative Game state.
#[path = "event_projection.rs"]
pub mod projection;

// Cross-journal, action-boundary authority view; not whole-Game event sourcing.
#[path = "authority_projection.rs"]
pub mod authority;

// The public custom slot limit must not crowd out the four defensive cards.
const CUSTOM_LISTENER_LIMIT: usize = 16;
const BUILTIN_LISTENER_LIMIT: usize = 4;
/// Slot identifiers must match the actual runtime event source/target strings.
/// Permissive integer parsing accepted foe:01 or foe:+1, producing dead listeners.
fn canonical_foe_slot(id: &str) -> bool {
    matches!(id, "foe:0" | "foe:1" | "foe:2" | "foe:3")
}
fn builtin_id(id: &str) -> bool {
    matches!(id, "hold_line:weak" | "kongcheng:vulnerable"
        | "qiaoying:weak" | "zhongyi:weak")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Response { Cancel, Replace(i32), Reduce(i32) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listener {
    pub id: String,
    pub kind: EventKind,
    pub source: String,
    pub target: Option<String>,
    pub priority: i16,
    pub charges: u8,
    pub effect: Response,
}
/// A reserved ID represents a fixed card effect, never a free-form custom slot.
fn valid_builtin_contract(rule: &Listener) -> bool {
    if !builtin_id(&rule.id) { return true; }
    if rule.kind != EventKind::StatusApplied
        || rule.target.as_deref() != Some("hero")
        || rule.priority != 0 || rule.charges != 1 {
        return false;
    }
    match rule.id.as_str() {
        "hold_line:weak" => rule.source == "weak" && rule.effect == Response::Cancel,
        "kongcheng:vulnerable" => rule.source == "vulnerable" && rule.effect == Response::Reduce(1),
        "qiaoying:weak" => rule.source == "weak" && rule.effect == Response::Reduce(1),
        "zhongyi:weak" => rule.source == "weak" && rule.effect == Response::Replace(1),
        _ => false,
    }
}
impl Listener {
    pub fn once(id: &str, source: &str, effect: Response) -> Self {
        Self { id:id.into(), kind: EventKind::StatusApplied, source:source.into(),
            target:Some("hero".into()), priority:0, charges:1, effect }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stack { pub listeners: Vec<Listener> }

/// Each observer is journalled in priority/ID order, including intermediate replacements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReactionStep {
    pub listener_id: String,
    pub before: i32,
    pub after: i32,
    pub cancelled: bool,
}
pub struct Resolution {
    pub amount:i32,
    pub cancelled:bool,
    pub by:Option<String>,
    pub steps:Vec<ReactionStep>,
}
impl Stack {
    pub fn is_empty(&self) -> bool { self.listeners.is_empty() }
    pub fn valid(&self) -> bool {
        self.listeners.len()<=CUSTOM_LISTENER_LIMIT+BUILTIN_LISTENER_LIMIT
            && self.listeners.iter().filter(|rule| !builtin_id(&rule.id)).count()<=CUSTOM_LISTENER_LIMIT
            && self.listeners.iter().enumerate().all(|(i,r)|
            !r.id.is_empty() && r.id.len()<=64
            // IDs reach CLI/TUI audit output; reject terminal escapes and line breaks.
            && !r.id.chars().any(char::is_control)
            && !r.source.is_empty()
            && r.source.len()<=64 && r.charges>0 && r.charges<=16
            && valid_builtin_contract(r)
            && r.target.as_deref().is_some_and(|t| t=="hero" || canonical_foe_slot(t))
            && match r.kind {
                EventKind::StatusApplied => match r.source.as_str() {
                    "weak" | "vulnerable" => true,
                    "poison" | "poison:toxic_reversal" =>
                        r.target.as_deref().is_some_and(canonical_foe_slot),
                    _ => false,
                },
                // Damage proposals carry raw incoming damage, before enemy armor.
                // Only known settled sources and living-foe slots can be intercepted.
                EventKind::DamageDealt =>
                    r.target.as_deref().is_some_and(canonical_foe_slot)
                    && matches!(r.source.as_str(),
                        "hero"|"fire"|"poison"|"pierce"|"troops"|"thorns"
                        |"arrows_on_discard"|"feishi"),
                // An incoming foe attack can be intercepted before hero block
                // or troop shielding. Only enemy slot IDs may originate hits;
                // hazards, self-damage and arbitrary source strings stay closed.
                EventKind::DamageReceived =>
                    r.target.as_deref()==Some("hero")
                    && canonical_foe_slot(&r.source),
                // Only actual stack increases on the eight defined powers can
                // be rewritten. No interruption of arbitrary passive dispatch.
                EventKind::PowerGained =>
                    r.target.as_deref()==Some("hero")
                    && crate::combat::PowerId::from_key(&r.source).is_some(),
                _ => false,
            }
            && match r.effect {
                Response::Cancel => true,
                Response::Replace(n) | Response::Reduce(n) => (0..=9999).contains(&n),
            }
            && !self.listeners[..i].iter().any(|old|old.id==r.id))
    }
    /// Dynamic registrations cannot impersonate the reserved card listeners.
    /// They occupy a separate 16-slot budget so built-in cards always resolve.
    pub fn install_custom(&mut self, rule: Listener) -> Result<(), &'static str> {
        if builtin_id(&rule.id) {return Err("reserved builtin reaction id");}
        self.install(rule)
    }
    pub fn install(&mut self, rule: Listener) -> Result<(), &'static str> {
        // Validate on a candidate first: a malformed replacement must not
        // remove the already installed, valid listener.
        let mut candidate=self.clone();
        candidate.listeners.retain(|r|r.id!=rule.id);
        candidate.listeners.push(rule);
        if candidate.listeners.len()>CUSTOM_LISTENER_LIMIT+BUILTIN_LISTENER_LIMIT
            || candidate.listeners.iter().filter(|rule| !builtin_id(&rule.id)).count()>CUSTOM_LISTENER_LIMIT {
            return Err("reaction stack full");
        }
        if !candidate.valid() {return Err("invalid reaction");}
        *self=candidate;
        Ok(())
    }
    /// Only user-created dynamic rules may be manually unregistered. Defensive
    /// card wards are consumed on matching events, not by custom commands.
    pub fn remove_custom(&mut self, id: &str) -> Result<(), &'static str> {
        if builtin_id(id) { return Err("reserved builtin reaction id"); }
        self.remove(id)
    }
    /// Explicit listener lifecycle action. The caller's Game::apply boundary
    /// journals the removal and rolls back an unknown ID atomically.
    pub fn remove(&mut self, id:&str) -> Result<(), &'static str> {
        if id.is_empty() || id.len()>64 { return Err("invalid reaction id"); }
        let before=self.listeners.len();
        self.listeners.retain(|rule|rule.id!=id);
        if self.listeners.len()==before { return Err("reaction not found"); }
        Ok(())
    }
    pub fn resolve(&mut self, event:&CombatEvent) -> Resolution {
        self.listeners.sort_by(|a,b| (a.priority,&a.id).cmp(&(b.priority,&b.id)));
        let mut out=Resolution {amount:event.amount,cancelled:false,by:None,steps:Vec::new()};
        if event.amount<=0 { return out; }
        for rule in &mut self.listeners {
            if rule.kind!=event.kind || rule.source!=event.source
                || rule.target.as_ref().is_some_and(|t|event.target.as_ref()!=Some(t)) {continue;}
            rule.charges-=1;
            out.by=Some(rule.id.clone());
            let before=out.amount;
            match rule.effect {
                Response::Cancel => {out.cancelled=true;out.amount=0;},
                Response::Replace(n) => out.amount=n.max(0),
                Response::Reduce(n) => out.amount=out.amount.saturating_sub(n).max(0),
            }
            out.steps.push(ReactionStep {
                listener_id:rule.id.clone(),before,after:out.amount,cancelled:out.cancelled,
            });
            if out.cancelled || out.amount==0 {break;}
        }
        self.listeners.retain(|r|r.charges>0);
        out
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunEvent { pub encounter:u64, pub floor:u8, pub event:CombatEvent }
/// Shared coordinate check for both batch archive validation and streaming
/// projection. Keep the two consumers on the same continuity contract.
pub(crate) fn verify_record_coordinate(
    previous: Option<(u64, u8, u64)>, current: &RunEvent,
) -> Result<(), String> {
    if current.encounter == 0 || current.floor == 0
        || current.floor > crate::MAX_FLOOR || current.event.seq == 0
    {
        return Err("invalid event coordinate".into());
    }
    if let Some((encounter, floor, seq)) = previous {
        if current.encounter == encounter {
            if current.floor != floor || seq.checked_add(1) != Some(current.event.seq) {
                return Err("gap in battle event stream".into());
            }
        } else if encounter.checked_add(1) != Some(current.encounter)
            || current.event.seq != 1 || current.floor < floor
        {
            return Err("gap between battles".into());
        }
    } else if current.encounter != 1 || current.event.seq != 1 {
        return Err("archive missing campaign start".into());
    }
    Ok(())
}

pub fn verify_record(events: &[RunEvent]) -> Result<(), String> {
    let mut previous = None;
    for event in events {
        verify_record_coordinate(previous, event)?;
        previous = Some((event.encounter, event.floor, event.event.seq));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn weak() -> CombatEvent {
        CombatEvent{seq:0,kind:EventKind::StatusApplied,source:"weak".into(),
            amount:3,target:Some("hero".into())}
    }
    #[test] fn ordered_replace_and_counter_are_one_shot() {
        let mut s=Stack::default();
        s.install(Listener::once("reduce","weak",Response::Reduce(1))).unwrap();
        let mut first=Listener::once("counter","weak",Response::Cancel);
        first.priority=-1;
        s.install(first).unwrap();
        assert!(s.resolve(&weak()).cancelled);
        assert_eq!(s.resolve(&weak()).amount,2);
        assert_eq!(s.resolve(&weak()).amount,3);
    }
    #[test] fn event_archive_rejects_floor_regression_and_invalid_coordinates() {
        let ev=|encounter, floor, seq|RunEvent{encounter,floor,
            event:CombatEvent{seq,..weak()}};
        assert!(verify_record(&[ev(1,1,1),ev(1,1,2),ev(2,1,1),ev(3,2,1)]).is_ok());
        assert!(verify_record(&[ev(1,2,1),ev(2,1,1)]).is_err());
        assert!(verify_record(&[ev(1,0,1)]).is_err());
        assert!(verify_record(&[ev(1,1,1),ev(1,1,1)]).is_err());
        assert!(verify_record(&[ev(1,1,1),ev(3,2,1)]).is_err());
        assert!(verify_record(&[ev(1,1,1),ev(2,crate::MAX_FLOOR,1),
            ev(3,1,1)]).is_err());
    }
    #[test] fn serialization_and_full_record_checks() {
        let mut s=Stack::default();
        s.install(Listener::once("ward","weak",Response::Replace(1))).unwrap();
        let d:Stack=serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(s,d);
        let ev=|battle,seq|RunEvent{encounter:battle,floor:1,
            event:CombatEvent{seq,..weak()}};
        assert!(verify_record(&[ev(1,1),ev(1,2),ev(2,1)]).is_ok());
        assert!(verify_record(&[ev(1,1),ev(1,3)]).is_err());
    }
}

#[cfg(test)]
mod reaction_integrity_regression {
    use super::*;
    fn event(source:&str,target:&str,amount:i32)->CombatEvent {
        CombatEvent{seq:0,kind:EventKind::StatusApplied,
            source:source.into(),target:Some(target.into()),amount}
    }
    #[test]
    fn failed_replacement_does_not_remove_existing_rule() {
        let mut stack=Stack::default();
        stack.install(Listener::once("ward","weak",Response::Cancel)).unwrap();
        let before=stack.clone();
        let mut invalid=Listener::once("ward","weak",Response::Replace(-1));
        invalid.charges=0;
        assert!(stack.install(invalid).is_err());
        assert_eq!(stack,before);
        assert!(stack.resolve(&event("weak","hero",3)).cancelled);
    }
    #[test]
    fn all_intermediate_reactions_are_ordered_and_auditable() {
        let mut stack=Stack::default();
        let mut a=Listener::once("a","poison",Response::Reduce(2));
        a.target=Some("foe:0".into());
        a.charges=2;
        let mut b=Listener::once("b","poison",Response::Replace(4));
        b.target=Some("foe:0".into());
        b.priority=1;
        stack.install(b).unwrap();
        stack.install(a).unwrap();
        let result=stack.resolve(&event("poison","foe:0",10));
        assert_eq!(result.amount,4);
        assert_eq!(result.steps.iter().map(|s|(s.listener_id.as_str(),s.before,s.after))
            .collect::<Vec<_>>(),vec![("a",10,8),("b",8,4)]);
        assert_eq!(stack.listeners.len(),1);
        let unrelated=stack.resolve(&event("weak","hero",3));
        assert!(unrelated.steps.is_empty());
        assert_eq!(stack.listeners[0].charges,1);
        assert_eq!(stack.resolve(&event("poison","foe:0",10)).amount,8);
        assert!(stack.listeners.is_empty());
    }
    #[test]
    fn zero_deltas_never_consume_charges() {
        let mut stack=Stack::default();
        stack.install(Listener::once("ward","weak",Response::Cancel)).unwrap();
        assert!(stack.resolve(&event("weak","hero",0)).steps.is_empty());
        assert_eq!(stack.listeners.len(),1);
    }
}
