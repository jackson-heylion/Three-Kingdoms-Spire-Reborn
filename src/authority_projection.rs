//! Unified read-only projection of six *committed action-boundary* journals.
//!
//! Unlike Game action replay, each component reduces only its own persisted
//! authority events. This joins their revisions and combat coordinates, so
//! independently well-formed but mutually inconsistent journals are rejected.
//! It does NOT restore the whole Game (map, lore, relics, boons, telemetry,
//! stage, enemy-event trigger internals are still missing), authenticate a
//! forged set of journals, or provide a transactional multi-file store.
use crate::{
    Game, card_zone_events::{self, CardZones},
    combat_effect_events::{self, BattleEffects},
    combat_resource_events::{self, BattleResources},
    foe_board_events::{self, FoeBoard},
    resource_events::{self, Resources}, rng_events,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorityView {
    pub revision: u64,
    pub resources: Resources,
    pub rng_state: u64,
    pub combat: Option<BattleResources>,
    pub foes: Option<FoeBoard>,
    pub cards: CardZones,
    pub effects: Option<BattleEffects>,
}

impl AuthorityView {
    /// The four independently reduced battle views must describe the same
    /// active encounter floor/turn, or all four must be absent.
    fn check_combat_coordinates(&self) -> Result<(), String> {
        let combat = self.combat.map(|s| (s.floor, s.turn));
        let foes = self.foes.as_ref().map(|s| (s.floor, s.turn));
        let cards = self.cards.battle.as_ref().map(|s| (s.floor, s.turn));
        let effects = self.effects.as_ref().map(|s| (s.floor, s.turn));
        if combat != foes || combat != cards || combat != effects {
            return Err("cross-journal combat floor/turn mismatch".into());
        }
        Ok(())
    }

    /// Separately compare the independently reduced view with the current
    /// snapshot. This is consistency checking, NOT a cryptographic signature.
    pub fn verify_snapshot(&self, game: &Game) -> Result<(), String> {
        if self.revision != game.revision
            || self.resources != Resources {
                hp: game.hp, max_hp: game.max_hp, gold: game.gold,
            }
            || self.rng_state != game.rng_state
            || self.combat != BattleResources::current(game)
            || self.foes != FoeBoard::current(game)
            || self.cards != CardZones::current(game)
            || self.effects != BattleEffects::current(game)
        {
            return Err("composite authority differs from Game snapshot".into());
        }
        Ok(())
    }
}

/// Reduce six complete journals, without consulting action history, RNG
/// algorithm, current Game state or event archive. Reject missing legacy
/// journals rather than inventing an authoritative history.
pub fn project(game: &Game) -> Result<AuthorityView, String> {
    let resources = game.resource_history.as_ref()
        .ok_or("missing resource authority journal")?;
    let rng = game.rng_history.as_ref()
        .ok_or("missing RNG authority journal")?;
    let combat = game.battle_resource_history.as_ref()
        .ok_or("missing combat resource authority journal")?;
    let foes = game.foe_board_history.as_ref()
        .ok_or("missing enemy board authority journal")?;
    let cards = game.card_zone_history.as_ref()
        .ok_or("missing card zone authority journal")?;
    let effects = game.combat_effect_history.as_ref()
        .ok_or("missing combat effect authority journal")?;

    let (resources, r0) = crate::resource_events::project(resources)?;
    let (rng_state, r1) = rng_events::project(rng)?;
    let (combat, r2) = combat_resource_events::project(combat)?;
    let (foes, r3) = foe_board_events::project(foes)?;
    let (cards, r4) = card_zone_events::project(cards)?;
    let (effects, r5) = combat_effect_events::project(effects)?;
    if [r1, r2, r3, r4, r5].iter().any(|r| *r != r0) {
        return Err("cross-journal committed revision mismatch".into());
    }
    let view = AuthorityView {
        revision: r0, resources, rng_state, combat, foes, cards, effects,
    };
    view.check_combat_coordinates()?;
    Ok(view)
}

impl Game {
    /// Verify six independent committed journal projections together, including
    /// revision, floor/turn and snapshot agreement. Never mutate the save.
    pub fn audit_composite_authority(&self) -> Result<AuthorityView, String> {
        let view = project(self)?;
        view.verify_snapshot(self)?;
        Ok(view)
    }
}
