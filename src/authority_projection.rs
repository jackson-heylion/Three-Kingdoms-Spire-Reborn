//! Atomic, lockstep projection of six committed action-boundary journals.
//!
//! This joins *every revision*, not merely the final snapshots: individually
//! legal journals with contradictory intermediate combat coordinates fail.
//! No action replay or Game snapshot is consulted by project(). This remains
//! partial authority, not whole-Game event sourcing or authentication.
use crate::{
    Game,
    card_zone_events::{self, CardZones},
    combat_effect_events::{self, BattleEffects},
    combat_resource_events::{self, BattleResources},
    foe_board_events::{self, FoeBoard},
    resource_events::{self, Resources},
    rng_events,
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

    /// Separate final snapshot comparison; does not authenticate either side.
    pub fn verify_snapshot(&self, game: &Game) -> Result<(), String> {
        if self.revision != game.revision
            || self.resources != (Resources {
                hp: game.hp, max_hp: game.max_hp, gold: game.gold,
            })
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

/// One complete committed action boundary from six independent event streams.
pub struct AuthorityStep<'a> {
    pub resources: &'a resource_events::ResourceEvent,
    pub rng: &'a rng_events::RngEvent,
    pub combat: &'a combat_resource_events::BattleResourceEvent,
    pub foes: &'a foe_board_events::FoeBoardEvent,
    pub cards: &'a card_zone_events::CardZoneEvent,
    pub effects: &'a combat_effect_events::CombatEffectEvent,
}

/// Incremental six-stream reducer. Failed steps cannot partially advance any
/// domain, even if five of the six individual events were otherwise valid.
#[derive(Clone, Debug, Default)]
pub struct CompositeAuthorityProjector {
    resources: resource_events::ResourceProjector,
    rng: rng_events::RngProjector,
    combat: combat_resource_events::BattleResourceProjector,
    foes: foe_board_events::FoeBoardProjector,
    cards: card_zone_events::CardZoneProjector,
    effects: combat_effect_events::CombatEffectProjector,
}

impl CompositeAuthorityProjector {
    pub fn new() -> Self { Self::default() }

    pub fn view(&self) -> Result<AuthorityView, String> {
        let (resources, r0) = self.resources.state()?;
        let (rng_state, r1) = self.rng.state()?;
        let (combat, r2) = self.combat.state()?;
        let (foes, r3) = self.foes.state()?;
        let (cards, r4) = self.cards.state()?;
        let (effects, r5) = self.effects.state()?;
        if [r1, r2, r3, r4, r5].iter().any(|r| *r != r0) {
            return Err("cross-journal committed revision mismatch".into());
        }
        let view = AuthorityView {
            revision: r0, resources, rng_state, combat, foes, cards, effects,
        };
        view.check_combat_coordinates()?;
        Ok(view)
    }

    pub fn push(&mut self, step: AuthorityStep<'_>) -> Result<(), String> {
        let mut next = self.clone();
        next.resources.push(step.resources)?;
        next.rng.push(step.rng)?;
        next.combat.push(step.combat)?;
        next.foes.push(step.foes)?;
        next.cards.push(step.cards)?;
        next.effects.push(step.effects)?;
        next.view()?; // Includes cross-stream revision and floor/turn agreement.
        *self = next;
        Ok(())
    }
}

/// Reduce the six complete journals in lockstep at *every* revision. Missing
/// legacy journals or mismatched journal lengths are not repaired or inferred.
/// This function reads only persisted event journals, never the Game snapshot.
pub fn project(game: &Game) -> Result<AuthorityView, String> {
    let resources = game.resource_history.as_ref()
        .ok_or("legacy save has no persistent resource authority journal")?;
    let rng = game.rng_history.as_ref()
        .ok_or("legacy save has no RNG authority journal")?;
    let combat = game.battle_resource_history.as_ref()
        .ok_or("legacy save has no combat resource authority journal")?;
    let foes = game.foe_board_history.as_ref()
        .ok_or("legacy save has no enemy board authority journal")?;
    let cards = game.card_zone_history.as_ref()
        .ok_or("legacy save has no card zone authority journal")?;
    let effects = game.combat_effect_history.as_ref()
        .ok_or("legacy save has no combat effect authority journal")?;

    let length = resources.len();
    if [rng.len(), combat.len(), foes.len(), cards.len(), effects.len()]
        .iter().any(|n| *n != length)
    {
        return Err("authority journals disagree on committed revision count".into());
    }
    let mut projector = CompositeAuthorityProjector::new();
    for index in 0..length {
        projector.push(AuthorityStep {
            resources: &resources[index], rng: &rng[index],
            combat: &combat[index], foes: &foes[index],
            cards: &cards[index], effects: &effects[index],
        }).map_err(|error| format!("authority boundary {index}: {error}"))?;
    }
    projector.view()
}

impl Game {
    /// Six journal reductions, each revision's cross-stream consistency and
    /// final snapshot agreement. Never writes or mutates the saved Game.
    pub fn audit_composite_authority(&self) -> Result<AuthorityView, String> {
        let view = project(self)?;
        view.verify_snapshot(self)?;
        Ok(view)
    }
}
