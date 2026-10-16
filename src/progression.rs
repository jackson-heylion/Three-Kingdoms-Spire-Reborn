//! Campaign progression independent of terminal, files and frontends.
//! Stored separately from in-progress snapshots so the same state model works over FFI/WASM.
use crate::{Game, Stage, content::Hero};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PROFILE_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub version: u32,
    pub started: u64,
    pub wins: u64,
    pub losses: u64,
    pub best_floor: u8,
    pub best_score: u64,
    pub ascensions: BTreeMap<String, u8>,
    pub hero_wins: BTreeMap<String, u64>,
    pub seen_cards: BTreeSet<String>,
    pub seen_enemies: BTreeSet<String>,
    pub discovered_relics: BTreeSet<String>,
    pub recorded_results: BTreeSet<String>,
    pub records: Vec<RunRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRecord {
    pub identity: String,
    pub hero: Hero,
    pub seed: u64,
    pub ascension: u8,
    pub floor: u8,
    pub score: u64,
    pub won: bool,
    pub deck_size: usize,
    pub relics: Vec<String>,
}

impl Profile {
    pub fn new() -> Self {
        Self { version: PROFILE_VERSION, ..Self::default() }
    }

    pub fn from_json(source: &str) -> Result<Self, String> {
        let result: Self = serde_json::from_str(source).map_err(|e| format!("战功簿解析失败: {e}"))?;
        if result.version != PROFILE_VERSION { return Err(format!("战功簿版本不兼容: {}", result.version)); }
        Ok(result)
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn on_start(&mut self, game: &Game) {
        self.started += 1;
        self.observe(game);
    }

    pub fn observe(&mut self, game: &Game) {
        self.best_floor = self.best_floor.max(game.floor);
        for c in &game.deck { self.seen_cards.insert(c.id.clone()); }
        if let Some(b) = &game.battle {
            for foe in &b.foes { self.seen_enemies.insert(foe.id.clone()); }
        }
        for relic in &game.relics { self.discovered_relics.insert(relic.clone()); }
    }

    /// identity is frontend-independent: a coordinator can supply a unique persistent ID;
    /// for CLI it is the hero/seed with run revision and outcome, so repeated calls are idempotent.
    pub fn record(&mut self, game: &Game, identity: String) -> Result<bool, String> {
        if !matches!(game.stage, Stage::Won | Stage::Lost) {
            return Err("战局尚未结束，不能记入战功簿".into());
        }
        self.observe(game);
        if !self.recorded_results.insert(identity.clone()) { return Ok(false); }
        let won = game.stage == Stage::Won;
        let score = score(game);
        if won {
            self.wins += 1;
            let key = game.hero.name().to_string();
            *self.hero_wins.entry(key.clone()).or_default() += 1;
            let difficulty = self.ascensions.entry(key).or_default();
            *difficulty = (*difficulty).max((game.ascension + 1).min(10));
        } else { self.losses += 1; }
        self.best_score = self.best_score.max(score);
        self.records.push(RunRecord {
            identity, hero: game.hero, seed: game.seed, ascension: game.ascension,
            floor: game.floor, score, won, deck_size: game.deck.len(),
            relics: game.relics.clone(),
        });
        if self.records.len() > 100 { self.records.remove(0); }
        Ok(true)
    }

    pub fn next_ascension(&self, hero: Hero) -> u8 {
        self.ascensions.get(hero.name()).copied().unwrap_or(0)
    }
}

/// Stable original run ranking, not the official Slay the Spire scoring formula.
pub fn score(game: &Game) -> u64 {
    let victory = u64::from(game.stage == Stage::Won);
    u64::from(game.floor) * 100
        + u64::from(game.ascension) * 50
        + game.hp.max(0) as u64 * 2
        + game.gold.max(0) as u64 / 5
        + game.relics.len() as u64 * 30
        + victory * 800
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Hero;

    #[test]
    fn progression_counts_once_and_unlocks_ascension() {
        let mut profile = Profile::new();
        let mut g = Game::new_with_ascension(Hero::CaoCao, 42, 3);
        profile.on_start(&g);
        g.floor = 21;
        g.stage = Stage::Won;
        assert_eq!(profile.record(&g, "campaign-1".into()), Ok(true));
        assert_eq!(profile.record(&g, "campaign-1".into()), Ok(false));
        assert_eq!(profile.started, 1);
        assert_eq!(profile.wins, 1);
        assert_eq!(profile.next_ascension(Hero::CaoCao), 4);
        let restored = Profile::from_json(&profile.to_json().unwrap()).unwrap();
        assert_eq!(restored.best_floor, 21);
        assert_eq!(restored.best_score, score(&g));
    }

    #[test]
    fn unfinished_games_cannot_be_recorded() {
        let mut profile = Profile::new();
        let game = Game::new(Hero::GuanYu, 1);
        assert!(profile.record(&game, "x".into()).is_err());
    }
}
