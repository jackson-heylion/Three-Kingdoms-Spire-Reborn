//! The single atomic commit boundary for every successful game action.
//! Journals are appended only on a validated draft, never on the live state.
// Keep the explicit cross-journal audit separate from the atomic write path.
#[path = "authority_bundle.rs"]
mod authority_bundle;

use crate::{
    Action, Game, card_zone_events, combat_effect_events, combat_resource_events, foe_board_events, resource_events, rng_events,
};

impl Game {
    /// Rejected actions leave live state, RNG, revision and all journals intact.
    pub fn apply(&mut self, action: Action) -> Result<Vec<String>, String> {
        // apply_unchecked uses a saturating revision increment. Never allow a
        // successful action to reuse the previous revision at u64::MAX.
        if self.revision == u64::MAX {
            return Err("action revision overflow: refusing an unversioned commit".into());
        }
        // Do not clone the growing card journal on every action: that would
        // make a long campaign quadratic in its recorded card snapshots.
        // Restore the live journal before any fallible game operation.
        let card_history = self.card_zone_history.take();
        let effect_history = self.combat_effect_history.take();
        let mut draft = self.clone();
        self.card_zone_history = card_history;
        self.combat_effect_history = effect_history;
        let messages = draft.apply_unchecked(action.clone())?;
        if let Some(history) = draft.resource_history.as_mut() {
            history.push(resource_events::ResourceEvent::Changed {
                revision: draft.revision,
                hp_delta: i64::from(draft.hp) - i64::from(self.hp),
                max_hp_delta: i64::from(draft.max_hp) - i64::from(self.max_hp),
                gold_delta: i64::from(draft.gold) - i64::from(self.gold),
            });
        }
        if let Some(history) = draft.rng_history.as_mut() {
            history.push(rng_events::RngEvent::Committed {
                revision: draft.revision,
                state: draft.rng_state,
            });
        }
        let combat_event = if draft.battle_resource_history.is_some() {
            Some(combat_resource_events::transition(
                combat_resource_events::BattleResources::current(self),
                combat_resource_events::BattleResources::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        if let (Some(history), Some(event)) = (draft.battle_resource_history.as_mut(), combat_event) {
            history.push(event);
        }
        let foe_event = if draft.foe_board_history.is_some() {
            Some(foe_board_events::transition(
                foe_board_events::FoeBoard::current(self),
                foe_board_events::FoeBoard::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        if let (Some(history), Some(event)) = (draft.foe_board_history.as_mut(), foe_event) {
            history.push(event);
        }
        let effect_event = if self.combat_effect_history.is_some() {
            Some(combat_effect_events::transition(
                combat_effect_events::BattleEffects::current(self),
                combat_effect_events::BattleEffects::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        let card_event = if self.card_zone_history.is_some() {
            Some(card_zone_events::transition(
                &card_zone_events::CardZones::current(self),
                &card_zone_events::CardZones::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        if let Some(actions) = draft.action_history.as_mut() {
            actions.push(action);
        }
        draft.validate()?;
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
        // Both large histories move only after all fallible validation succeeds.
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
        *self = draft;
        Ok(messages)
    }
}
