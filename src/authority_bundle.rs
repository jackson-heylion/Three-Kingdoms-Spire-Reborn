//! Cross-check the four action-boundary authority journals at *every* revision.
//! This detects mutually inconsistent histories that still project to valid
//! individual final snapshots. It does not authenticate coordinated tampering
//! or reconstruct powers, listeners, RNG or intermediate combat events.
use crate::{
    Game,
    combat_resource_events::{BattleResourceProjector, BattleResources},
    foe_board_events::{FoeBoard, FoeBoardProjector},
    card_zone_events::{CardZoneProjector, CardZones},
    resource_events::{ResourceProjector, Resources},
};

impl Game {
    /// Independently reduce all four journals in lockstep, without action
    /// replay or RNG. Legacy saves missing any journal are not certified.
    /// Returns the number of boundaries checked, including revision zero.
    pub fn audit_authority_bundle(&self) -> Result<usize, String> {
        let resources = self.resource_history.as_ref()
            .ok_or("legacy save has no persistent resource authority journal")?;
        let combat = self.battle_resource_history.as_ref()
            .ok_or("legacy save has no combat resource authority journal")?;
        let foes = self.foe_board_history.as_ref()
            .ok_or("legacy save has no enemy board authority journal")?;
        let cards = self.card_zone_history.as_ref()
            .ok_or("legacy save has no card zone authority journal")?;

        let count = resources.len();
        if count == 0 || count > 50_001
            || combat.len() != count || foes.len() != count || cards.len() != count
            || usize::try_from(self.revision).ok().and_then(|r| r.checked_add(1)) != Some(count)
        {
            return Err("authority journals disagree on committed revision count".into());
        }

        let mut resource_projector = ResourceProjector::default();
        let mut combat_projector = BattleResourceProjector::default();
        let mut foe_projector = FoeBoardProjector::default();
        let mut card_projector = CardZoneProjector::default();

        for index in 0..count {
            resource_projector.push(&resources[index])
                .map_err(|e| format!("resource boundary {index}: {e}"))?;
            combat_projector.push(&combat[index])
                .map_err(|e| format!("combat boundary {index}: {e}"))?;
            foe_projector.push(&foes[index])
                .map_err(|e| format!("enemy boundary {index}: {e}"))?;
            card_projector.push(&cards[index])
                .map_err(|e| format!("card boundary {index}: {e}"))?;

            let (_, resource_revision) = resource_projector.state()?;
            let (battle, combat_revision) = combat_projector.state()?;
            let (board, foe_revision) = foe_projector.state()?;
            let (zones, card_revision) = card_projector.state()?;
            if [resource_revision, combat_revision, foe_revision, card_revision]
                .iter().any(|revision| *revision != index as u64)
            {
                return Err(format!("authority revision mismatch at boundary {index}"));
            }
            match (battle, board.as_ref(), zones.battle.as_ref()) {
                (None, None, None) => {}
                (Some(b), Some(f), Some(c))
                    if b.floor == f.floor && b.floor == c.floor
                        && b.turn == f.turn && b.turn == c.turn => {}
                _ => return Err(format!(
                    "enemy, card and combat lifecycle disagree at boundary {index}"
                )),
            }
        }

        let (resources, _) = resource_projector.state()?;
        let (battle, _) = combat_projector.state()?;
        let (board, _) = foe_projector.state()?;
        let (zones, _) = card_projector.state()?;
        if resources != (Resources { hp: self.hp, max_hp: self.max_hp, gold: self.gold })
            || battle != BattleResources::current(self)
            || board != FoeBoard::current(self)
            || zones != CardZones::current(self)
        {
            return Err("authority journals disagree with the current game snapshot".into());
        }
        Ok(count)
    }
}
