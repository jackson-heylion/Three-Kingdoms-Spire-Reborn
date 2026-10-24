//! Deterministic action replay and event archive reconstruction.
//! Kept separate from the large rules engine to reduce its change surface.
use crate::{Game, event_stack};

impl Game {
    /// Rebuild the campaign exclusively from its seed and committed actions,
    /// including reward RNG, combat event sequence, map topology and narrative.
    /// No side effects; the caller's live game is never modified.
    pub fn verify_action_replay(&self) -> Result<usize,String> {
        let actions=self.action_history.as_ref().ok_or(
            "该旧版存档没有起始操作日志，不能验证此前未记录的动作")?;
        if actions.len()>50_000 { return Err("操作历史超过单次回放保护上限".into()); }
        let mut replayed=Game::new_with_identity(
            self.hero,self.seed,self.ascension,self.mode,self.run_id.clone());
        if self.event_history.is_none() { replayed.event_history=None; }
        if self.resource_history.is_none() { replayed.resource_history=None; }
        if self.rng_history.is_none() { replayed.rng_history=None; }
        if self.battle_resource_history.is_none() { replayed.battle_resource_history=None; }
        if self.foe_board_history.is_none() { replayed.foe_board_history=None; }
        if self.card_zone_history.is_none() { replayed.card_zone_history=None; }
        if self.combat_effect_history.is_none() { replayed.combat_effect_history=None; }
        // Profile bookkeeping is external to the deterministic action journal.
        // Preserve its persisted receipt while replaying gameplay from the seed.
        replayed.profile_start_receipt = self.profile_start_receipt;
        for (index,action) in actions.iter().enumerate() {
            replayed.apply(action.clone()).map_err(|e|
                format!("重放第{}步失败：{e}",index+1))?;
        }
        if replayed.to_json()?!=self.to_json()? {
            return Err("重放的实际状态与当前快照不一致：可能存在修改或版本规则变更".into());
        }
        Ok(actions.len())
    }
    /// Replay successful actions and reconstruct even events from ended battles.
    /// Legacy snapshots must match the replayed gameplay state before claiming
    /// the result as a faithful full event history.
    pub fn reconstruct_event_history(&self) -> Result<Vec<event_stack::RunEvent>, String> {
        let actions=self.action_history.as_ref().ok_or(
            "legacy save has no complete action journal")?;
        if actions.len()>50_000 {return Err("action replay limit exceeded".into());}
        let mut replay=Game::new_with_identity(self.hero,self.seed,self.ascension,
            self.mode,self.run_id.clone());
        replay.profile_start_receipt=self.profile_start_receipt;
        if self.resource_history.is_none() { replay.resource_history=None; }
        if self.rng_history.is_none() { replay.rng_history=None; }
        if self.battle_resource_history.is_none() { replay.battle_resource_history=None; }
        if self.foe_board_history.is_none() { replay.foe_board_history=None; }
        if self.card_zone_history.is_none() { replay.card_zone_history=None; }
        if self.combat_effect_history.is_none() { replay.combat_effect_history=None; }
        for (i,action) in actions.iter().enumerate() {
            replay.apply(action.clone()).map_err(|e|format!("replay step {}: {e}",i+1))?;
        }
        let history=replay.event_history.clone().unwrap_or_default();
        event_stack::verify_record(&history)?;
        if let Some(expected)=&self.event_history {
            if &history!=expected {return Err("persisted event archive differs from replay".into());}
        } else {
            // A legacy snapshot cannot claim it persisted events it never recorded.
            replay.event_history=None;
        }
        // The event stream alone does not prove the live snapshot was untouched:
        // HP, gold, resource counters or an active listener could be modified
        // without altering any archived CombatEvent. Verify *all* game state.
        if replay.to_json()?!=self.to_json()? {
            return Err("event replay snapshot differs from committed actions".into());
        }
        Ok(history)
    }
}
