//! Serializable, deterministic pre-resolution reaction stack.
//! Registered by successful card resolutions; no hidden wall-clock callbacks.
use crate::combat::{CombatEvent, EventKind};
use serde::{Deserialize, Serialize};

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
impl Listener {
    pub fn once(id: &str, source: &str, effect: Response) -> Self {
        Self { id:id.into(), kind: EventKind::StatusApplied, source:source.into(),
            target:Some("hero".into()), priority:0, charges:1, effect }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stack { pub listeners: Vec<Listener> }

pub struct Resolution { pub amount:i32, pub cancelled:bool, pub by:Option<String> }
impl Stack {
    pub fn is_empty(&self) -> bool { self.listeners.is_empty() }
    pub fn valid(&self) -> bool {
        self.listeners.len()<=16 && self.listeners.iter().enumerate().all(|(i,r)|
            !r.id.is_empty() && r.id.len()<=64 && !r.source.is_empty()
            && r.source.len()<=64 && r.charges>0 && r.charges<=16
            && r.target.as_ref().is_none_or(|s|s.len()<=64)
            && !matches!(r.effect,Response::Reduce(n) if n<0)
            && !self.listeners[..i].iter().any(|old|old.id==r.id))
    }
    pub fn install(&mut self, rule: Listener) -> Result<(), &'static str> {
        if self.listeners.iter().any(|r|r.id==rule.id) {
            self.listeners.retain(|r|r.id!=rule.id);
        }
        if self.listeners.len()>=16 {return Err("reaction stack full");}
        self.listeners.push(rule);
        if !self.valid() {return Err("invalid reaction");}
        Ok(())
    }
    pub fn resolve(&mut self, event:&CombatEvent) -> Resolution {
        self.listeners.sort_by(|a,b| (a.priority,&a.id).cmp(&(b.priority,&b.id)));
        let mut out=Resolution {amount:event.amount,cancelled:false,by:None};
        for rule in &mut self.listeners {
            if rule.kind!=event.kind || rule.source!=event.source
                || rule.target.as_ref().is_some_and(|t|event.target.as_ref()!=Some(t)) {continue;}
            rule.charges-=1;
            out.by=Some(rule.id.clone());
            match rule.effect {
                Response::Cancel => {out.cancelled=true;out.amount=0;break;},
                Response::Replace(n) => out.amount=n.max(0),
                Response::Reduce(n) => out.amount=out.amount.saturating_sub(n).max(0),
            }
        }
        self.listeners.retain(|r|r.charges>0);
        out
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunEvent { pub encounter:u64, pub floor:u8, pub event:CombatEvent }
pub fn verify_record(events:&[RunEvent]) -> Result<(),String> {
    for (i,e) in events.iter().enumerate() {
        if e.encounter==0 || e.floor>crate::MAX_FLOOR || e.event.seq==0
            {return Err("invalid event coordinate".into());}
        if let Some(p)=i.checked_sub(1).and_then(|j|events.get(j)) {
            if e.encounter==p.encounter {
                if e.floor!=p.floor || e.event.seq!=p.event.seq.saturating_add(1)
                    {return Err("gap in battle event stream".into());}
            } else if e.encounter!=p.encounter.saturating_add(1) || e.event.seq!=1
                {return Err("gap between battles".into());}
        } else if e.encounter!=1 || e.event.seq!=1
            {return Err("archive missing campaign start".into());}
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
