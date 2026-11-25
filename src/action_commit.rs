//! The single atomic commit boundary for every successful game action.
//! Journals are appended only on a validated draft, never on the live state.
// Keep the explicit cross-journal audit separate from the atomic write path.
#[path = "authority_bundle.rs"]
mod authority_bundle;

use crate::{
    Action, Game, campaign_authority, card_zone_events, combat_effect_events, battle_runtime_events, combat_resource_events, foe_board_events, resource_events, rng_events, telemetry_authority,
};

impl Game {
    /// Rejected actions leave live state, RNG, revision and all journals intact.
    pub fn apply(&mut self, action: Action) -> Result<Vec<String>, String> {
        self.verify_rules_epoch_anchor()?;
        // apply_unchecked uses a saturating revision increment. Never allow a
        // successful action to reuse the previous revision at u64::MAX.
        if self.revision == u64::MAX {
            return Err("action revision overflow: refusing an unversioned commit".into());
        }
        // Do not clone the growing card and action journals on every action: that would
        // make a long campaign quadratic in its recorded snapshots.
        // Restore every taken live journal before any fallible game operation.
        let action_history = self.action_history.take();
        let resource_history = self.resource_history.take();
        let rng_history = self.rng_history.take();
        let battle_resource_history = self.battle_resource_history.take();
        let foe_board_history = self.foe_board_history.take();
        let card_history = self.card_zone_history.take();
        let effect_history = self.combat_effect_history.take();
        let runtime_history = self.battle_runtime_history.take();
        let campaign_history = self.campaign_history.take();
        let telemetry_history = self.telemetry_history.take();
        let mut draft = self.clone();
        self.action_history = action_history;
        self.resource_history = resource_history;
        self.rng_history = rng_history;
        self.battle_resource_history = battle_resource_history;
        self.foe_board_history = foe_board_history;
        self.card_zone_history = card_history;
        self.combat_effect_history = effect_history;
        self.battle_runtime_history = runtime_history;
        self.campaign_history = campaign_history;
        self.telemetry_history = telemetry_history;
        let messages = draft.apply_unchecked(action.clone())?;
        let resource_event = self.resource_history.is_some().then(|| {
            resource_events::ResourceEvent::Changed {
                revision: draft.revision,
                hp_delta: i64::from(draft.hp) - i64::from(self.hp),
                max_hp_delta: i64::from(draft.max_hp) - i64::from(self.max_hp),
                gold_delta: i64::from(draft.gold) - i64::from(self.gold),
            }
        });
        let rng_event = self.rng_history.is_some().then_some(rng_events::RngEvent::Committed {
            revision: draft.revision,
            state: draft.rng_state,
        });
        let combat_event = if self.battle_resource_history.is_some() {
            Some(combat_resource_events::transition(
                combat_resource_events::BattleResources::current(self),
                combat_resource_events::BattleResources::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        let foe_event = if self.foe_board_history.is_some() {
            Some(foe_board_events::transition(
                foe_board_events::FoeBoard::current(self),
                foe_board_events::FoeBoard::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        let effect_event = if self.combat_effect_history.is_some() {
            Some(combat_effect_events::transition(
                combat_effect_events::BattleEffects::current(self),
                combat_effect_events::BattleEffects::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        let runtime_event = if self.battle_runtime_history.is_some() {
            Some(battle_runtime_events::transition(
                battle_runtime_events::BattleRuntime::current(self),
                battle_runtime_events::BattleRuntime::current(&draft),
                draft.revision,
            )?)
        } else { None };
        let card_event = if self.card_zone_history.is_some() {
            Some(card_zone_events::transition(
                &card_zone_events::CardZones::current(self),
                &card_zone_events::CardZones::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        let campaign_event = if let Some(history) = self.campaign_history.as_ref() {
            let mut before = campaign_authority::CampaignState::current(self);
            let mut after = campaign_authority::CampaignState::current(&draft);
            // Pre-identity V2 journals remain uncertified after resuming.
            // Never introduce an identity mid-stream: the projector correctly
            // rejects that transition, even when the live Game has an identity.
            if matches!(history.first(), Some(campaign_authority::CampaignEvent::Started { state }) if state.identity.is_none()) {
                before.identity = None;
                after.identity = None;
            }
            Some(campaign_authority::transition(&before, &after, draft.revision)?)
        } else { None };
        let telemetry_event = if let Some(history) = self.telemetry_history.as_ref() {
            // Preserve the absence of revision cursors in older V2 journals:
            // resuming a legacy campaign must not invent earlier event evidence.
            let archive_len = if history.last().and_then(|event| event.archive_len()).is_some() {
                Some(u64::try_from(draft.event_history.as_ref()
                    .ok_or("event archive missing for cursor-enabled campaign")?.len())
                    .map_err(|_| "event archive cursor overflow")?)
            } else { None };
            Some(telemetry_authority::transition_with_archive(
                &self.telemetry, &draft.telemetry, draft.revision, archive_len,
            )?)
        } else { None };
        draft.validate()?;
        if self.action_history.as_ref().is_some_and(|actions| actions.len() >= 50_000) {
            return Err("操作日志长度超限".into());
        }
        if rng_event.is_some()
            && self.rng_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("rng journal exceeds action limit".into());
        }
        if resource_event.is_some()
            && self.resource_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("resource journal exceeds action limit".into());
        }
        if combat_event.is_some()
            && self.battle_resource_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("combat resource journal exceeds action limit".into());
        }
        if foe_event.is_some()
            && self.foe_board_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("enemy board journal exceeds action limit".into());
        }
        if card_event.is_some()
            && self.card_zone_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("card zone journal exceeds action limit".into());
        }
        if effect_event.is_some()
            && self.combat_effect_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("combat effect journal exceeds action limit".into());
        }
        if runtime_event.is_some()
            && self.battle_runtime_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("battle runtime journal exceeds action limit".into());
        }
        if campaign_event.is_some()
            && self.campaign_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("campaign journal exceeds action limit".into());
        }
        if telemetry_event.is_some()
            && self.telemetry_history.as_ref().is_some_and(|history| history.len() >= 50_001)
        {
            return Err("telemetry journal exceeds action limit".into());
        }
        // Large histories move only after all fallible validation succeeds.
        if let Some(mut actions) = self.action_history.take() {
            actions.push(action);
            draft.action_history = Some(actions);
        }
        if let Some(event) = resource_event {
            let mut history = self.resource_history.take().expect("checked resource journal");
            history.push(event);
            draft.resource_history = Some(history);
        }
        if let Some(event) = rng_event {
            let mut history = self.rng_history.take().expect("checked rng journal");
            history.push(event);
            draft.rng_history = Some(history);
        }
        if let Some(event) = combat_event {
            let mut history = self.battle_resource_history.take().expect("checked combat journal");
            history.push(event);
            draft.battle_resource_history = Some(history);
        }
        if let Some(event) = foe_event {
            let mut history = self.foe_board_history.take().expect("checked foe journal");
            history.push(event);
            draft.foe_board_history = Some(history);
        }
        if let Some(event) = card_event {
            let mut history = self.card_zone_history.take().expect("checked card journal");
            history.push(event);
            draft.card_zone_history = Some(history);
        }
        if let Some(event) = effect_event {
            let mut history = self.combat_effect_history.take().expect("checked effect journal");
            history.push(event);
            draft.combat_effect_history = Some(history);
        }
        if let Some(event) = runtime_event {
            let mut history = self.battle_runtime_history.take().expect("checked runtime journal");
            history.push(event);
            draft.battle_runtime_history = Some(history);
        }
        if let Some(event) = campaign_event {
            let mut history = self.campaign_history.take().expect("checked campaign journal");
            history.push(event);
            draft.campaign_history = Some(history);
        }
        if let Some(event) = telemetry_event {
            let mut history = self.telemetry_history.take().expect("checked telemetry journal");
            history.push(event);
            draft.telemetry_history = Some(history);
        }
        *self = draft;
        Ok(messages)
    }
}
