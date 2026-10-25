//! Compatibility entry points for the existing six-source authority audit.
//! The shared atomic projector now owns lockstep reduction, so this module
//! cannot drift from the standalone CLI or per-revision authority checks.
use crate::{Game, event_stack::authority::{AuthorityStep, CompositeAuthorityProjector}};

impl Game {
    /// Reduce all six action-boundary journals, including every intermediate
    /// revision, then compare the result with the current Game snapshot.
    /// Legacy saves lacking any journal remain playable but are not certified.
    pub fn audit_authority_bundle(&self) -> Result<usize, String> {
        self.audit_composite_authority()?;
        let count = self.resource_history.as_ref()
            .ok_or("missing resource authority journal")?.len();
        if count == 0 || count > 50_001
            || usize::try_from(self.revision).ok()
                .and_then(|revision| revision.checked_add(1)) != Some(count)
        {
            return Err("authority journals disagree on committed revision count".into());
        }
        Ok(count)
    }

    /// Restore a typed, cross-stream-checked prefix and emit the original JSON
    /// contract. Audit the *entire* history first: a valid prefix must never
    /// conceal a corrupt later revision or a forged final Game snapshot.
    pub fn project_authority_boundary(&self, revision: u64) -> Result<serde_json::Value, String> {
        if revision > self.revision {
            return Err("requested authority revision is beyond current game".into());
        }
        self.audit_authority_bundle()?;
        let index = usize::try_from(revision).map_err(|_| "authority revision too large")?;
        let rng = self.rng_history.as_ref().ok_or("missing RNG journal")?;
        let resources = self.resource_history.as_ref()
            .ok_or("missing persistent resource journal")?;
        let combat = self.battle_resource_history.as_ref()
            .ok_or("missing combat resource journal")?;
        let foes = self.foe_board_history.as_ref()
            .ok_or("missing enemy board journal")?;
        let cards = self.card_zone_history.as_ref()
            .ok_or("missing card zone journal")?;
        let effects = self.combat_effect_history.as_ref()
            .ok_or("missing combat effect journal")?;

        let mut projector = CompositeAuthorityProjector::new();
        for i in 0..=index {
            projector.push(AuthorityStep {
                resources: &resources[i], rng: &rng[i],
                combat: &combat[i], foes: &foes[i],
                cards: &cards[i], effects: &effects[i],
            })?;
        }
        let view = projector.view()?;
        Ok(serde_json::json!({
            "revision": view.revision,
            "rng_state": view.rng_state,
            "resources": view.resources,
            "combat": view.combat,
            "foes": view.foes,
            "cards": { "deck": view.cards.deck, "battle": view.cards.battle },
            "effects": view.effects
        }))
    }
}
