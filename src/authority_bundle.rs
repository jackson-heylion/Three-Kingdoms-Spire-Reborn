//! Cross-check six action-boundary authority journals at *every* revision.
//! This detects mutually inconsistent histories that still project to valid
//! individual final snapshots. It does not authenticate coordinated tampering
//! or prove RNG draw causality or reconstruct intermediate combat events; powers and listeners
//! are projected only at successful action boundaries.
use crate::{
    Game,
    combat_resource_events::{BattleResourceProjector, BattleResources},
    foe_board_events::{FoeBoard, FoeBoardProjector},
    card_zone_events::{CardZoneProjector, CardZones},
    combat_effect_events::{BattleEffects, CombatEffectProjector},
    resource_events::{ResourceProjector, Resources},
    rng_events::RngProjector,
};

impl Game {
    /// Reduce all six journals in lockstep without action replay.
    /// RNG snapshots do not prove draw causality. Legacy saves are not certified.
    /// Returns the number of boundaries checked, including revision zero.
    pub fn audit_authority_bundle(&self) -> Result<usize, String> {
        let rng = self.rng_history.as_ref()
            .ok_or("legacy save has no RNG authority journal")?;
        let resources = self.resource_history.as_ref()
            .ok_or("legacy save has no persistent resource authority journal")?;
        let combat = self.battle_resource_history.as_ref()
            .ok_or("legacy save has no combat resource authority journal")?;
        let foes = self.foe_board_history.as_ref()
            .ok_or("legacy save has no enemy board authority journal")?;
        let cards = self.card_zone_history.as_ref()
            .ok_or("legacy save has no card zone authority journal")?;

        let effects = self.combat_effect_history.as_ref()
            .ok_or("legacy save has no combat effect authority journal")?;

        let count = resources.len();
        if count == 0 || count > 50_001
            || combat.len() != count || foes.len() != count || cards.len() != count
            || effects.len() != count || rng.len() != count
            || usize::try_from(self.revision).ok().and_then(|r| r.checked_add(1)) != Some(count)
        {
            return Err("authority journals disagree on committed revision count".into());
        }

        let mut rng_projector = RngProjector::default();
        let mut resource_projector = ResourceProjector::default();
        let mut combat_projector = BattleResourceProjector::default();
        let mut foe_projector = FoeBoardProjector::default();
        let mut card_projector = CardZoneProjector::default();
        let mut effect_projector = CombatEffectProjector::default();

        for index in 0..count {
            rng_projector.push(&rng[index])
                .map_err(|e| format!("rng boundary {index}: {e}"))?;
            resource_projector.push(&resources[index])
                .map_err(|e| format!("resource boundary {index}: {e}"))?;
            combat_projector.push(&combat[index])
                .map_err(|e| format!("combat boundary {index}: {e}"))?;
            foe_projector.push(&foes[index])
                .map_err(|e| format!("enemy boundary {index}: {e}"))?;
            card_projector.push(&cards[index])
                .map_err(|e| format!("card boundary {index}: {e}"))?;
            effect_projector.push(&effects[index])
                .map_err(|e| format!("effect boundary {index}: {e}"))?;

            let (_, rng_revision) = rng_projector.state()?;
            let (_, resource_revision) = resource_projector.state()?;
            let (battle, combat_revision) = combat_projector.state()?;
            let (board, foe_revision) = foe_projector.state()?;
            let (zones, card_revision) = card_projector.state()?;
            let (effects, effect_revision) = effect_projector.state()?;
            if [rng_revision, resource_revision, combat_revision, foe_revision, card_revision, effect_revision]
                .iter().any(|revision| *revision != index as u64)
            {
                return Err(format!("authority revision mismatch at boundary {index}"));
            }
            match (battle, board.as_ref(), zones.battle.as_ref(), effects.as_ref()) {
                (None, None, None, None) => {}
                (Some(b), Some(f), Some(c), Some(e))
                    if b.floor == f.floor && b.floor == c.floor && b.floor == e.floor
                        && b.turn == f.turn && b.turn == c.turn && b.turn == e.turn => {}
                _ => return Err(format!(
                    "enemy, card, effect and combat lifecycle disagree at boundary {index}"
                )),
            }
        }

        let (rng_state, _) = rng_projector.state()?;
        let (resources, _) = resource_projector.state()?;
        let (battle, _) = combat_projector.state()?;
        let (board, _) = foe_projector.state()?;
        let (zones, _) = card_projector.state()?;
        let (effects, _) = effect_projector.state()?;
        if rng_state != self.rng_state
            || resources != (Resources { hp: self.hp, max_hp: self.max_hp, gold: self.gold })
            || battle != BattleResources::current(self)
            || board != FoeBoard::current(self)
            || zones != CardZones::current(self)
            || effects != BattleEffects::current(self)
        {
            return Err("authority journals disagree with the current game snapshot".into());
        }
        Ok(count)
    }

    /// Project one committed boundary from all six journals.
    /// This is a partial view, not a reconstructed Game or RNG draw history.
    /// First audit the entire stream so a valid prefix cannot hide a corrupt tail.
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

        let mut rng_projector = RngProjector::default();
        let mut resource_projector = ResourceProjector::default();
        let mut combat_projector = BattleResourceProjector::default();
        let mut foe_projector = FoeBoardProjector::default();
        let mut card_projector = CardZoneProjector::default();
        let mut effect_projector = CombatEffectProjector::default();
        for i in 0..=index {
            rng_projector.push(&rng[i])?;
            resource_projector.push(&resources[i])?;
            combat_projector.push(&combat[i])?;
            foe_projector.push(&foes[i])?;
            card_projector.push(&cards[i])?;
            effect_projector.push(&effects[i])?;
        }
        let (rng_state, _) = rng_projector.state()?;
        let (resource_state, _) = resource_projector.state()?;
        let (combat_state, _) = combat_projector.state()?;
        let (foe_state, _) = foe_projector.state()?;
        let (card_state, _) = card_projector.state()?;
        let (effect_state, _) = effect_projector.state()?;
        Ok(serde_json::json!({
            "revision": revision,
            "rng_state": rng_state,
            "resources": resource_state,
            "combat": combat_state,
            "foes": foe_state,
            "cards": { "deck": card_state.deck, "battle": card_state.battle },
            "effects": effect_state
        }))
    }

}
