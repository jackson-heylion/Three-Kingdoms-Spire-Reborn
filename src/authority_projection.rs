//! Atomic, lockstep projection of nine committed action-boundary journals.
//!
//! This joins *every revision*, not merely the final snapshots: individually
//! legal journals with contradictory intermediate combat coordinates fail.
//! No action replay or Game snapshot is consulted by project(). This remains
//! partial authority, not whole-Game event sourcing or authentication.
use crate::{
    Game,
    campaign_authority::{self, CampaignState},
    card_zone_events::{self, CardZones},
    combat_effect_events::{self, BattleEffects},
    battle_runtime_events::{self, BattleRuntime},
    combat_resource_events::{self, BattleResources},
    foe_board_events::{self, FoeBoard},
    resource_events::{self, Resources},
    rng_events,
    telemetry_authority::{self, TelemetryProjector},
    CombatTelemetry,
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
    pub runtime: Option<BattleRuntime>,
    pub campaign: CampaignState,
    pub telemetry: CombatTelemetry,
}

impl AuthorityView {
    fn check_combat_coordinates(&self) -> Result<(), String> {
        let combat = self.combat.map(|s| (s.floor, s.turn));
        let foes = self.foes.as_ref().map(|s| (s.floor, s.turn));
        let cards = self.cards.battle.as_ref().map(|s| (s.floor, s.turn));
        let effects = self.effects.as_ref().map(|s| (s.floor, s.turn));
        let runtime = self.runtime.as_ref().map(|s| (s.floor, s.turn));
        if combat != foes || combat != cards || combat != effects || combat != runtime {
            return Err("cross-journal combat floor/turn mismatch".into());
        }
        // A started encounter is either still active or has exactly one
        // recorded outcome. A separately well-formed telemetry stream may
        // otherwise invent battles while all combat/campaign journals agree.
        let completed = self.telemetry.totals.battles_won
            .checked_add(self.telemetry.totals.battles_lost)
            .ok_or("combat telemetry outcome counter overflow")?;
        let expected_started = completed
            .checked_add(u64::from(self.combat.is_some()))
            .ok_or("combat telemetry encounter counter overflow")?;
        if self.telemetry.totals.battles_started != expected_started {
            return Err("cross-journal battle lifecycle and telemetry mismatch".into());
        }
        if (self.campaign.stage == crate::Stage::Combat) != self.combat.is_some()
            || self.combat.as_ref().is_some_and(|battle| battle.floor != self.campaign.floor)
        {
            return Err("cross-journal campaign stage/floor mismatch".into());
        }
        Ok(())
    }

    /// Rebuild the entire active Battle from nine independently reduced journals.
    /// This never executes an Action, samples RNG, or reads the live Game battle.
    /// It does not reconstruct noncombat Game fields or authenticate the save.
    pub fn rebuild_battle(&self) -> Result<Option<crate::Battle>, String> {
        let (resources, foes, cards, effects, runtime) = match (
            self.combat.as_ref(), self.foes.as_ref(), self.cards.battle.as_ref(),
            self.effects.as_ref(), self.runtime.as_ref(),
        ) {
            (None, None, None, None, None) => return Ok(None),
            (Some(resources), Some(foes), Some(cards), Some(effects), Some(runtime)) =>
                (resources, foes, cards, effects, runtime),
            _ => return Err("incomplete cross-journal battle reconstruction".into()),
        };
        self.check_combat_coordinates()?;
        Ok(Some(crate::Battle {
            foes: foes.foes.clone(),
            draw: cards.draw.clone(), hand: cards.hand.clone(),
            discard: cards.discard.clone(), exhausted: cards.exhausted.clone(),
            energy: resources.energy, block: resources.block,
            strength: runtime.strength, weak: runtime.weak,
            vulnerable: runtime.vulnerable, momentum: runtime.momentum,
            scheme: runtime.scheme, combo: runtime.combo, command: runtime.command,
            fury: runtime.fury, thorns: runtime.thorns,
            played: runtime.played, played_this_turn: runtime.played_this_turn,
            last_turn_cards: runtime.last_turn_cards,
            next_turn_energy_penalty: runtime.next_turn_energy_penalty,
            troops_hp: runtime.troops_hp, troops_attack: runtime.troops_attack,
            discard_tokens: runtime.discard_tokens, thought_used: runtime.thought_used,
            powers: effects.powers.clone(), reactions: effects.reactions.clone(),
            events: runtime.events.clone(), event_seq: runtime.event_seq,
            turn: resources.turn,
        }))
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
            || self.runtime != BattleRuntime::current(game)
            || self.campaign != CampaignState::current(game)
            || self.telemetry != game.telemetry
        {
            return Err("composite authority differs from Game snapshot".into());
        }
        Ok(())
    }
}

/// One complete committed action boundary from nine independent event streams.
pub struct AuthorityStep<'a> {
    pub resources: &'a resource_events::ResourceEvent,
    pub rng: &'a rng_events::RngEvent,
    pub combat: &'a combat_resource_events::BattleResourceEvent,
    pub foes: &'a foe_board_events::FoeBoardEvent,
    pub cards: &'a card_zone_events::CardZoneEvent,
    pub effects: &'a combat_effect_events::CombatEffectEvent,
    pub runtime: &'a battle_runtime_events::BattleRuntimeEvent,
    pub campaign: &'a campaign_authority::CampaignEvent,
    pub telemetry: &'a telemetry_authority::TelemetryEvent,
}

/// Incremental eight-stream reducer. Failed steps cannot partially advance any
/// domain, even if seven of the eight individual events were otherwise valid.
#[derive(Clone, Debug, Default)]
pub struct CompositeAuthorityProjector {
    resources: resource_events::ResourceProjector,
    rng: rng_events::RngProjector,
    combat: combat_resource_events::BattleResourceProjector,
    foes: foe_board_events::FoeBoardProjector,
    cards: card_zone_events::CardZoneProjector,
    effects: combat_effect_events::CombatEffectProjector,
    runtime: battle_runtime_events::BattleRuntimeProjector,
    campaign: campaign_authority::CampaignProjector,
    telemetry: TelemetryProjector,
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
        let (runtime, r8) = self.runtime.state()?;
        let (campaign, r6) = self.campaign.state()?;
        let (telemetry, r7) = self.telemetry.state()?;
        if [r1, r2, r3, r4, r5, r6, r7, r8].iter().any(|r| *r != r0) {
            return Err("cross-journal committed revision mismatch".into());
        }
        let view = AuthorityView {
            revision: r0, resources, rng_state, combat, foes, cards, effects, runtime, campaign, telemetry,
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
        next.runtime.push(step.runtime)?;
        next.campaign.push(step.campaign)?;
        next.telemetry.push(step.telemetry)?;
        next.view()?; // Includes cross-stream revision and floor/turn agreement.
        *self = next;
        Ok(())
    }
}

/// Reduce the nine complete journals in lockstep at *every* revision. Missing
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
    let runtime = game.battle_runtime_history.as_ref()
        .ok_or("legacy save has no battle runtime authority journal")?;
    let campaign = game.campaign_history.as_ref()
        .ok_or("legacy save has no campaign authority journal")?;
    let telemetry = game.telemetry_history.as_ref()
        .ok_or("legacy save has no telemetry authority journal")?;

    let length = resources.len();
    if [rng.len(), combat.len(), foes.len(), cards.len(), effects.len(), runtime.len(), campaign.len(), telemetry.len()]
        .iter().any(|n| *n != length)
    {
        return Err("authority journals disagree on committed revision count".into());
    }
    let mut projector = CompositeAuthorityProjector::new();
    for index in 0..length {
        projector.push(AuthorityStep {
            resources: &resources[index], rng: &rng[index],
            combat: &combat[index], foes: &foes[index],
            cards: &cards[index], effects: &effects[index], runtime: &runtime[index],
            campaign: &campaign[index], telemetry: &telemetry[index],
        }).map_err(|error| format!("authority boundary {index}: {error}"))?;
    }
    projector.view()
}

impl Game {
    /// Nine journal reductions, each revision's cross-stream consistency and
    /// final snapshot agreement. Never writes or mutates the saved Game.
    pub fn audit_composite_authority(&self) -> Result<AuthorityView, String> {
        let view = project(self)?;
        view.verify_snapshot(self)?;
        Ok(view)
    }

    /// Reconcile the complete combat-event archive with all eight independently
    /// reduced action-boundary journals. This is a read-only cross-source audit,
    /// not action replay, full-Game reconstruction, or cryptographic attestation.
    /// Missing legacy archives cannot be silently promoted to certified status.
    pub fn audit_event_archive_authority(&self) -> Result<
        (AuthorityView, crate::event_stack::projection::CampaignProjection), String
    > {
        let view = self.audit_composite_authority()?;
        let archive = self.event_history.as_deref()
            .ok_or("legacy save has no complete combat event archive")?;
        let projection = crate::event_stack::projection::project(archive)?;
        projection.reconcile_telemetry(&view.telemetry.totals, view.combat.is_some())?;

        // Event-only outcome reconciliation proves counts, but an attacker
        // could also move an otherwise well-formed active encounter to a
        // different floor. Bind its coordinate to the campaign journal.
        if let Some(combat) = view.combat {
            let current = projection.encounters.last()
                .ok_or("active combat missing from event archive")?;
            if current.closed || current.floor != combat.floor {
                return Err("event archive active encounter differs from campaign".into());
            }
        } else if projection.encounters.last().is_some_and(|last| !last.closed) {
            return Err("event archive has an unsealed inactive encounter".into());
        }
        // Cursors add per-action causality checks for new saves. Old V2
        // archives remain eligible for the existing final-only audit.
        self.audit_event_archive_boundaries()?;
        Ok((view, projection))
    }

    /// Compare each persisted combat-event prefix with the independent
    /// telemetry and campaign state at that exact successful action revision.
    /// No actions, RNG stepping, or mutable Game state are used. Pre-upgrade
    /// saves return None instead of inventing historical event boundaries.
    pub fn audit_event_archive_boundaries(&self) -> Result<Option<usize>, String> {
        use crate::event_stack::projection::CampaignProjector as EventProjector;
        let archive = self.event_history.as_deref()
            .ok_or("legacy save has no complete combat event archive")?;
        let telemetry = self.telemetry_history.as_deref()
            .ok_or("legacy save has no telemetry authority journal")?;
        let campaign = self.campaign_history.as_deref()
            .ok_or("legacy save has no campaign authority journal")?;
        if telemetry.len() != campaign.len()
            || usize::try_from(self.revision).ok()
                .and_then(|revision| revision.checked_add(1)) != Some(telemetry.len())
        {
            return Err("event boundary journals disagree on revision count".into());
        }
        if telemetry.first().and_then(telemetry_authority::TelemetryEvent::archive_len).is_none() {
            if telemetry.iter().any(|event| event.archive_len().is_some()) {
                return Err("legacy event boundary cursors unexpectedly appear mid-run".into());
            }
            return Ok(None);
        }
        // Legacy saves without revision cursors keep their explicit
        // uncertified result, even if the ninth runtime journal is absent.
        let runtime = self.battle_runtime_history.as_deref()
            .ok_or("legacy save has no battle runtime authority journal")?;
        if runtime.len() != telemetry.len() {
            return Err("event boundary runtime journal revision count mismatch".into());
        }

        let mut stats = telemetry_authority::TelemetryProjector::default();
        let mut stages = campaign_authority::CampaignProjector::default();
        let mut runtimes = battle_runtime_events::BattleRuntimeProjector::default();
        let mut events = EventProjector::new();
        // Keep only the latest bounded window. Re-scanning the full archive
        // at each revision would turn long campaigns into quadratic audits.
        let mut event_window: Vec<crate::combat::CombatEvent> = Vec::new();
        let mut window_encounter: Option<u64> = None;
        let mut offset = 0usize;
        for (index, (telemetry_event, campaign_event)) in
            telemetry.iter().zip(campaign).enumerate()
        {
            stats.push(telemetry_event)?;
            stages.push(campaign_event)?;
            runtimes.push(&runtime[index])?;
            let cursor = stats.archive_cursor()
                .ok_or("event archive cursor missing at committed revision")?;
            let end = usize::try_from(cursor)
                .map_err(|_| "event archive cursor exceeds platform range")?;
            if end < offset || end > archive.len() {
                return Err(format!("event revision {index}: archive cursor out of bounds"));
            }
            for record in &archive[offset..end] {
                events.push(record).map_err(|error|
                    format!("event revision {index}: {error}"))?;
                if window_encounter != Some(record.encounter) {
                    event_window.clear();
                    window_encounter = Some(record.encounter);
                }
                event_window.push(record.event.clone());
                if event_window.len() > crate::combat::MAX_RECENT_EVENTS {
                    event_window.remove(0);
                }
            }
            offset = end;
            let (telemetry_state, _) = stats.state()?;
            let (campaign_state, _) = stages.state()?;
            let active = campaign_state.stage == crate::Stage::Combat;
            events.projection().reconcile_telemetry(&telemetry_state.totals, active)
                .map_err(|error| format!("event revision {index}: {error}"))?;
            if active {
                let encounter = events.projection().encounters.last()
                    .ok_or_else(|| format!("event revision {index}: active encounter missing"))?;
                if encounter.closed || encounter.floor != campaign_state.floor {
                    return Err(format!("event revision {index}: active encounter floor/seal mismatch"));
                }
                // A newly started encounter may have no settled events yet.
                if window_encounter != Some(telemetry_state.totals.battles_started) {
                    event_window.clear();
                    window_encounter = Some(telemetry_state.totals.battles_started);
                }
                let (runtime_state, _) = runtimes.state()?;
                let runtime_state = runtime_state
                    .ok_or_else(|| format!("event revision {index}: active runtime missing"))?;
                if runtime_state.floor != campaign_state.floor
                    || runtime_state.events != event_window
                    || runtime_state.event_seq != event_window.last().map_or(0, |e| e.seq)
                {
                    return Err(format!(
                        "event revision {index}: battle runtime window differs from event archive"
                    ));
                }
            } else if events.projection().encounters.last().is_some_and(|last| !last.closed) {
                return Err(format!("event revision {index}: unsealed inactive encounter"));
            }
        }
        if offset != archive.len() {
            return Err("event archive contains records beyond final committed cursor".into());
        }
        Ok(Some(telemetry.len()))
    }
}
