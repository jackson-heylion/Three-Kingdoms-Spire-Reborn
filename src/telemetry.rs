//! Run-wide combat counters and bounded turn diagnostics.
use serde::{Deserialize, Serialize};

/// Lifetime-of-run counters. Fields are monotonic and old V2 saves default to zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CombatCounters {
    pub battles_started: u64,
    pub battles_won: u64,
    pub battles_lost: u64,
    pub turns_completed: u64,
    pub cards_played: u64,
    pub hero_hp_damage: u64,
    pub troop_attacks: u64,
    pub troop_damage: u64,
    pub troop_absorbed: u64,
    pub command_procs: u64,
    pub command_energy: u64,
    pub command_block: u64,
    pub sun_discards: u64,
    pub sun_discard_block: u64,
    pub sun_combo_procs: u64,
    pub sun_combo_block: u64,
    pub sun_combo_draws: u64,
}

impl CombatCounters {
    pub fn accumulate(&mut self, other: &Self) {
        self.battles_started = self.battles_started.saturating_add(other.battles_started);
        self.battles_won = self.battles_won.saturating_add(other.battles_won);
        self.battles_lost = self.battles_lost.saturating_add(other.battles_lost);
        self.turns_completed = self.turns_completed.saturating_add(other.turns_completed);
        self.cards_played = self.cards_played.saturating_add(other.cards_played);
        self.hero_hp_damage = self.hero_hp_damage.saturating_add(other.hero_hp_damage);
        self.troop_attacks = self.troop_attacks.saturating_add(other.troop_attacks);
        self.troop_damage = self.troop_damage.saturating_add(other.troop_damage);
        self.troop_absorbed = self.troop_absorbed.saturating_add(other.troop_absorbed);
        self.command_procs = self.command_procs.saturating_add(other.command_procs);
        self.command_energy = self.command_energy.saturating_add(other.command_energy);
        self.command_block = self.command_block.saturating_add(other.command_block);
        self.sun_discards = self.sun_discards.saturating_add(other.sun_discards);
        self.sun_discard_block = self.sun_discard_block.saturating_add(other.sun_discard_block);
        self.sun_combo_procs = self.sun_combo_procs.saturating_add(other.sun_combo_procs);
        self.sun_combo_block = self.sun_combo_block.saturating_add(other.sun_combo_block);
        self.sun_combo_draws = self.sun_combo_draws.saturating_add(other.sun_combo_draws);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnTelemetry {
    pub floor: u8,
    pub turn: u32,
    pub completed: bool,
    pub hero_hp_after: i32,
    pub troop_hp_after: i32,
    pub counters: CombatCounters,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CombatTelemetry {
    pub totals: CombatCounters,
    pub current_turn: CombatCounters,
    /// Bounded diagnostic trace, not a full event-sourcing replay.
    pub recent_turns: Vec<TurnTelemetry>,
}
pub const MAX_RECENT_TURN_METRICS: usize = 24;

