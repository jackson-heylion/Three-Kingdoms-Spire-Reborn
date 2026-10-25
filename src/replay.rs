//! Deterministic action replay and event archive reconstruction.
//! Kept separate from the large rules engine to reduce its change surface.
use crate::{Game, event_stack};

impl Game {
    /// Reconstruct the complete Game at an action revision from the initial seed
    /// and committed actions. This includes map, lore, inventory, combat, RNG,
    /// dynamic listeners and the journals available in the source save.
    ///
    /// This is deterministic ACTION REPLAY, not independent event sourcing:
    /// the action journal is stored alongside the snapshot and is not signed.
    /// Call verify_action_replay() to compare the final reconstructed Game
    /// against the saved snapshot; a historical prefix cannot authenticate
    /// omitted or deliberately rewritten actions.
    pub fn reconstruct_game_at(&self, revision: u64) -> Result<Game, String> {
        let actions = self.action_history.as_ref().ok_or(
            "该旧版存档没有完整操作日志，无法重建任意历史版本")?;
        if actions.len() > 50_000 {
            return Err("操作历史超过单次回放保护上限".into());
        }
        if usize::try_from(self.revision).ok() != Some(actions.len()) {
            return Err("操作日志与存档修订号不一致，拒绝历史重建".into());
        }
        if revision > self.revision {
            return Err("请求的历史修订号超过当前存档".into());
        }
        let mut replayed = Game::new_with_identity(
            self.hero, self.seed, self.ascension, self.mode, self.run_id.clone(),
        );
        // Old V2 saves may omit individual authority streams. Preserve that
        // absence instead of fabricating a complete historical journal.
        if self.event_history.is_none() { replayed.event_history = None; }
        if self.resource_history.is_none() { replayed.resource_history = None; }
        if self.rng_history.is_none() { replayed.rng_history = None; }
        if self.battle_resource_history.is_none() { replayed.battle_resource_history = None; }
        if self.foe_board_history.is_none() { replayed.foe_board_history = None; }
        if self.card_zone_history.is_none() { replayed.card_zone_history = None; }
        if self.combat_effect_history.is_none() { replayed.combat_effect_history = None; }
        // External chronicle bookkeeping is not an action. Preserve its
        // durable receipt without claiming to reconstruct the chronicle.
        replayed.profile_start_receipt = self.profile_start_receipt;
        let count = usize::try_from(revision).map_err(|_| "历史修订号超出平台范围")?;
        for (index, action) in actions.iter().take(count).enumerate() {
            replayed.apply(action.clone()).map_err(|error|
                format!("重放第{}步失败：{error}", index + 1))?;
        }
        Ok(replayed)
    }

    /// Rebuild the complete current Game from its seed and committed actions
    /// and compare every serialized field, not just the six partial journals.
    pub fn verify_action_replay(&self) -> Result<usize, String> {
        let replayed = self.reconstruct_game_at(self.revision)?;
        if replayed.to_json()? != self.to_json()? {
            return Err("重放的实际状态与当前快照不一致：可能存在修改或版本规则变更".into());
        }
        Ok(self.action_history.as_ref().map_or(0, Vec::len))
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
