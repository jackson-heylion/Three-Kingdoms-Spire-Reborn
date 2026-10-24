//! The single atomic commit boundary for every successful game action.
//! Journals are appended only on a validated draft, never on the live state.
use crate::{
    Action, Game, card_zone_events, combat_resource_events, foe_board_events, resource_events,
};

impl Game {
    /// Rejected actions leave live state, RNG, revision and all journals intact.
    pub fn apply(&mut self, action: Action) -> Result<Vec<String>, String> {
        let mut draft = self.clone();
        let messages = draft.apply_unchecked(action.clone())?;
        if let Some(history) = draft.resource_history.as_mut() {
            history.push(resource_events::ResourceEvent::Changed {
                revision: draft.revision,
                hp_delta: i64::from(draft.hp) - i64::from(self.hp),
                max_hp_delta: i64::from(draft.max_hp) - i64::from(self.max_hp),
                gold_delta: i64::from(draft.gold) - i64::from(self.gold),
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
        let card_event = if draft.card_zone_history.is_some() {
            Some(card_zone_events::transition(
                &card_zone_events::CardZones::current(self),
                &card_zone_events::CardZones::current(&draft),
                draft.revision,
            )?)
        } else {
            None
        };
        if let (Some(history), Some(event)) = (draft.card_zone_history.as_mut(), card_event) {
            history.push(event);
        }
        if let Some(actions) = draft.action_history.as_mut() {
            actions.push(action);
        }
        draft.validate()?;
        *self = draft;
        Ok(messages)
    }
}
