//! Campaign progression independent of terminal, files and frontends.
//! Stored separately from in-progress snapshots so the same state model works over FFI/WASM.
use crate::{Game, RunMode, Stage, content::Hero};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PROFILE_VERSION: u32 = 1;
/// Stable identity + player-readable unlock criteria. New items can be added
/// without changing the persisted profile version.
pub const ACHIEVEMENTS: &[(&str,&str,&str)] = &[
    ("first_step","初出茅庐","完成首次出征"),
    ("first_battle","沙场初胜","完成至少一场战斗"),
    ("first_elite","猛将破阵","击败至少一名精英敌人"),
    ("first_act","黄巾初定","首次抵达第二幕"),
    ("final_act","三分天命","首次抵达第三幕"),
    ("first_clear","天下归心","首次通关完整三幕"),
    ("five_heroes","五虎英豪","五名武将各完成至少一次通关"),
    ("ascension_five","逆境突围","标准战役通关飞升5或更高难度"),
    ("ascension_ten","问鼎巅峰","标准战役通关飞升10"),
    ("discover_40","博闻强识","收集至少40种卡牌图鉴"),
    ("relic_10","古物收藏","收集至少10种遗物图鉴"),
    ("story_10","演义拾遗","单局见证至少10则剧情"),
    ("hundred_turns","百战之师","单次战役完成累计100回合"),
];


#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// Unlocked permanent achievements; compatible with old V1 profiles.
    pub achievements: BTreeSet<String>,
    pub longest_campaign: u8,
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
    #[serde(default)] pub mode: RunMode,
    pub deck_size: usize,
    pub relics: Vec<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION, started: 0, wins: 0, losses: 0,
            best_floor: 0, best_score: 0, ascensions: BTreeMap::new(),
            hero_wins: BTreeMap::new(), seen_cards: BTreeSet::new(),
            seen_enemies: BTreeSet::new(), discovered_relics: BTreeSet::new(),
            recorded_results: BTreeSet::new(), records: Vec::new(),
            achievements: BTreeSet::new(), longest_campaign: 0,
        }
    }
}
impl Profile {
    pub fn new() -> Self { Self::default() }

    pub fn from_json(source: &str) -> Result<Self, String> {
        let mut result: Self = serde_json::from_str(source).map_err(|e| format!("战功簿解析失败: {e}"))?;
        if result.version != PROFILE_VERSION { return Err(format!("战功簿版本不兼容: {}", result.version)); }
        if result.achievements.iter().any(|id|!ACHIEVEMENTS.iter().any(|(known,_,_)|known==id)) {
            return Err("战功簿包含未知成就代码".into());
        }
        result.refresh_achievements();
        Ok(result)
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn on_start(&mut self, game: &Game) {
        self.started += 1;
        self.observe(game);
    }
    pub fn achievement_progress(&self)->Vec<(&'static str,&'static str,&'static str,bool)> {
        ACHIEVEMENTS.iter().map(|(id,name,desc)|
            (*id,*name,*desc,self.achievements.contains(*id))).collect()
    }
    fn refresh_achievements(&mut self) {
        let mut add=|id|{self.achievements.insert(id.to_string());};
        if self.started>0 {add("first_step");}
        if self.longest_campaign>=2 {add("first_act");}
        if self.longest_campaign>=3 {add("final_act");}
        if self.wins>0 {add("first_clear");}
        if self.records.iter().any(|r|r.won && r.mode==RunMode::Standard && r.ascension>=5) {add("ascension_five");}
        if self.records.iter().any(|r|r.won && r.mode==RunMode::Standard && r.ascension==10) {add("ascension_ten");}
        if self.hero_wins.len()>=5 {add("five_heroes");}
        if self.seen_cards.len()>=40 {add("discover_40");}
        if self.discovered_relics.len()>=10 {add("relic_10");}
    }

    pub fn observe(&mut self, game: &Game) {
        self.best_floor = self.best_floor.max(game.floor);
        self.longest_campaign=self.longest_campaign.max(if game.floor==0 {0} else {(game.floor-1)/7+1});
        if game.telemetry.totals.battles_won>0 {self.achievements.insert("first_battle".into());}
        if game.telemetry.totals.battles_won>0 && game.telemetry.totals.battles_started>1
            && game.relics.len()>1 {self.achievements.insert("first_elite".into());}
        if game.lore_seen.len()>=10 {self.achievements.insert("story_10".into());}
        if game.telemetry.totals.turns_completed>=100 {self.achievements.insert("hundred_turns".into());}
        for c in &game.deck { self.seen_cards.insert(c.id.clone()); }
        if let Some(b) = &game.battle {
            for foe in &b.foes { self.seen_enemies.insert(foe.id.clone()); }
        }
        for relic in &game.relics { self.discovered_relics.insert(relic.clone()); }
        self.refresh_achievements();
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
            if game.mode == RunMode::Standard {
                *difficulty = (*difficulty).max((game.ascension + 1).min(10));
            }
        } else { self.losses += 1; }
        self.best_score = self.best_score.max(score);
        self.records.push(RunRecord {
            identity, hero: game.hero, seed: game.seed, ascension: game.ascension,
            floor: game.floor, score, won, mode: game.mode, deck_size: game.deck.len(),
            relics: game.relics.clone(),
        });
        if self.records.len() > 100 { self.records.remove(0); }
        self.refresh_achievements();
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
    fn achievement_unlocks_are_persistent_and_old_profiles_migrate() {
        let mut p=Profile::new();
        let mut g=Game::new(Hero::GuanYu,1);
        p.on_start(&g);
        assert!(p.achievements.contains("first_step"));
        assert!(!p.achievements.contains("first_clear"));
        g.floor=15;
        p.observe(&g);
        assert!(p.achievements.contains("final_act"));
        g.stage=Stage::Won;
        p.record(&g,"achievement-1".into()).unwrap();
        assert!(p.achievements.contains("first_clear"));
        let restored=Profile::from_json(&p.to_json().unwrap()).unwrap();
        assert_eq!(restored.achievements,p.achievements);
        let mut old:serde_json::Value=serde_json::from_str(&p.to_json().unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("achievements");
        old.as_object_mut().unwrap().remove("longest_campaign");
        let old=Profile::from_json(&old.to_string()).unwrap();
        assert!(old.achievements.contains("first_clear"));
        assert!(!old.achievements.contains("final_act"));
        assert!(old.achievement_progress().len()>=12);
    }
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
    fn custom_and_daily_runs_do_not_unlock_ranked_difficulties() {
        let mut p = Profile::new();
        for mode in [RunMode::Daily,RunMode::Custom] {
            let mut g = Game::new_with_mode(Hero::LuBu,123,10,mode);
            g.stage = Stage::Won;
            p.record(&g,format!("{mode:?}")).unwrap();
        }
        assert_eq!(p.next_ascension(Hero::LuBu),0);
        assert_eq!(p.wins,2);
    }
    #[test]
    fn unfinished_games_cannot_be_recorded() {
        let mut profile = Profile::new();
        let game = Game::new(Hero::GuanYu, 1);
        assert!(profile.record(&game, "x".into()).is_err());
    }
}
