//! Deterministic action replay and event archive reconstruction.
//! Kept separate from the large rules engine to reduce its change surface.
#[path = "revision_checkpoint.rs"]
mod revision_checkpoint;
pub use revision_checkpoint::RevisionCheckpoint;

use crate::{Action, Game, Stage, event_stack};

/// One committed action's observable effects, reconstructed by executing the
/// actual rules. Event order is preserved across nested reaction dispatches.
/// RNG states are boundary snapshots, NOT a record of individual draws.
#[derive(Clone, Debug)]
pub struct ActionEventTrace {
    pub revision: u64,
    pub action: Action,
    pub stage_before: Stage,
    pub stage_after: Stage,
    pub rng_before: u64,
    pub rng_after: u64,
    pub events: Vec<event_stack::RunEvent>,
}

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
    /// Attribute each archived combat event to the committed action that emitted
    /// it, including nested triggers and cross-battle transitions. This is a
    /// read-only deterministic diagnostic; it does NOT authenticate the journal,
    /// reconstruct individual RNG draws or turn events into an event-sourced Game.
    ///
    /// Unlike a plain archive read, this checks every emitted event prefix against
    /// the persisted stream, then verifies the complete final Game snapshot.
    /// Old saves without an archived event stream cannot claim causal coverage.
    pub fn reconstruct_action_event_trace(&self) -> Result<Vec<ActionEventTrace>, String> {
        let source = self.event_history.as_ref()
            .ok_or("存档缺少完整战斗事件档案，无法认证动作与事件的对应关系")?;
        let actions = self.action_history.as_ref()
            .ok_or("存档缺少完整操作日志，无法认证事件因果关系")?;
        let mut replay = self.reconstruct_game_at(0)?;
        let mut offset = replay.event_history.as_ref()
            .ok_or("重放初始状态缺少战斗事件档案")?.len();
        if offset > source.len() || replay.event_history.as_ref().unwrap().as_slice() != &source[..offset] {
            return Err("战斗事件档案的初始前缀与重放不一致".into());
        }
        let mut trace = Vec::with_capacity(actions.len());
        for (index, action) in actions.iter().enumerate() {
            let before_rng = replay.rng_state;
            let before_stage = replay.stage;
            replay.apply(action.clone()).map_err(|error|
                format!("动作事件因果重放第{}步失败：{error}", index + 1))?;
            let emitted = replay.event_history.as_ref()
                .ok_or("重放过程中丢失战斗事件档案")?;
            let new_events = emitted.get(offset..)
                .ok_or("重放事件档案意外缩短")?;
            if source.get(offset..emitted.len()) != Some(new_events) {
                return Err(format!("动作第{}步的事件序列与存档档案不一致", index + 1));
            }
            trace.push(ActionEventTrace {
                revision: replay.revision,
                action: action.clone(),
                stage_before: before_stage,
                stage_after: replay.stage,
                rng_before: before_rng,
                rng_after: replay.rng_state,
                events: new_events.to_vec(),
            });
            offset = emitted.len();
        }
        if offset != source.len() {
            return Err("存档事件档案包含无法归属到成功动作的尾部记录".into());
        }
        if replay.to_json()? != self.to_json()? {
            return Err("动作事件因果重放的最终完整 Game 与存档不一致".into());
        }
        Ok(trace)
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
