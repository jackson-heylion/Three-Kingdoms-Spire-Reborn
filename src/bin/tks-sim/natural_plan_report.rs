//! Natural formation is measured at actual boss entry, before battle rewards.
//! End-of-run deck completion must not be misreported as boss-ready formation.
use super::*;

#[derive(Debug, Clone)]
pub(super) struct NaturalBossEvidence {
    pub(super) floor: u8,
    pub(super) foes: Vec<String>,
    pub(super) core_owned: usize,
    pub(super) support_owned: usize,
    pub(super) core_complete: bool,
    pub(super) support_complete: bool,
    pub(super) survived: bool,
}

pub(super) struct NaturalBossStart {
    floor: u8,
    foes: Vec<String>,
    core_owned: usize,
    support_owned: usize,
    core_total: usize,
    support_total: usize,
}

impl NaturalBossStart {
    pub(super) fn capture(game: &Game, plan: &BuildPlan) -> Self {
        let core_owned = plan.core.iter()
            .filter(|id| game.deck.iter().any(|card| card.id == **id)).count();
        let support_owned = plan.support.iter()
            .filter(|id| game.deck.iter().any(|card| card.id == **id)).count();
        Self {
            floor: game.floor,
            foes: game.battle.as_ref().map(|battle|
                battle.foes.iter().map(|foe| foe.id.clone()).collect()
            ).unwrap_or_default(),
            core_owned,
            support_owned,
            core_total: plan.core.len(),
            support_total: plan.support.len(),
        }
    }

    pub(super) fn finish(self, survived: bool) -> NaturalBossEvidence {
        NaturalBossEvidence {
            floor: self.floor,
            foes: self.foes,
            core_owned: self.core_owned,
            support_owned: self.support_owned,
            core_complete: self.core_owned == self.core_total,
            support_complete: self.support_owned == self.support_total,
            survived,
        }
    }
}

#[derive(Debug,Serialize)]
struct NaturalPlanCell {
    id:String,
    hero:String,
    ascension:u8,
    samples:usize,
    baseline_wins:usize,
    focused_wins:usize,
    baseline_floor_sum:u64,
    focused_floor_sum:u64,
    /// Number of games obtaining ALL required distinct core card IDs.
    complete_core:usize,
    /// Number obtaining all distinct support IDs.
    complete_support:usize,
    /// Denominator: real boss encounters reached, not all simulated runs.
    boss_battles:usize,
    boss_wins:usize,
    boss_core_formed:usize,
    boss_full_formed:usize,
    boss_core_wins:usize,
    counter_boss_battles:usize,
    counter_boss_wins:usize,
    core_distinct_sum:usize,
    support_distinct_sum:usize,
    reward_screens:u64,
    focus_offer_screens:u64,
    focus_picks:u64,
}
#[derive(Debug,Serialize)]
struct NaturalPlanReport {
    schema_version:u8,
    sample_policy:String,
    samples_per_cell:usize,
    interpretation:String,
    cells:Vec<NaturalPlanCell>,
}
fn natural_plan_report(samples:usize)->Result<NaturalPlanReport,String> {
    let mut cells=Vec::new();
    for (plan_index,plan) in PLANS.iter().enumerate() {
        for ascension in 0..=10_u8 {
            let mut cell=NaturalPlanCell {
                id:plan.id.into(),hero:plan.hero.name().into(),ascension,
                samples,baseline_wins:0,focused_wins:0,
                baseline_floor_sum:0,focused_floor_sum:0,
                complete_core:0,complete_support:0,
                boss_battles:0,boss_wins:0,boss_core_formed:0,
                boss_full_formed:0,boss_core_wins:0,
                counter_boss_battles:0,counter_boss_wins:0,
                core_distinct_sum:0,support_distinct_sum:0,
                reward_screens:0,focus_offer_screens:0,focus_picks:0,
            };
            for trial in 0..samples {
                let ordinal=8_000_000u64+(plan_index as u64)*11*10_000
                    +u64::from(ascension)*10_000+trial as u64+1;
                let seed=ordinal.wrapping_mul(0x9E3779B97F4A7C15);
                let baseline=simulate_config_blueprint(plan.hero,seed,ascension,
                    Policy::Tactical,SunPackage::Standard,None,DeckTreatment::Standard)?;
                let focused=simulate_config_blueprint(plan.hero,seed,ascension,
                    Policy::Tactical,SunPackage::Standard,None,DeckTreatment::Natural(plan))?;
                let final_game=Game::from_json(&focused.snapshot)?;
                let core=plan.acquired(&final_game.deck);
                let support=plan.support.iter()
                    .filter(|id|final_game.deck.iter().any(|card|card.id==**id)).count();
                cell.complete_core+=usize::from(core==plan.core.len());
                cell.complete_support+=usize::from(support==plan.support.len());
                cell.core_distinct_sum+=core;
                cell.support_distinct_sum+=support;
                cell.baseline_wins+=usize::from(baseline.won);
                cell.focused_wins+=usize::from(focused.won);
                cell.baseline_floor_sum+=u64::from(baseline.floor);
                cell.focused_floor_sum+=u64::from(focused.floor);
                cell.reward_screens+=u64::from(focused.reward_screens);
                cell.focus_offer_screens+=u64::from(focused.focus_offer_screens);
                cell.focus_picks+=u64::from(focused.focus_picks);
                for boss in &focused.natural_boss_evidence {
                    cell.boss_battles+=1;
                    cell.boss_wins+=usize::from(boss.survived);
                    cell.boss_core_formed+=usize::from(boss.core_complete);
                    cell.boss_full_formed+=usize::from(boss.core_complete && boss.support_complete);
                    cell.boss_core_wins+=usize::from(boss.core_complete && boss.survived);
                    if boss.foes.iter().any(|id| id == plan.counter_enemy) {
                        cell.counter_boss_battles+=1;
                        cell.counter_boss_wins+=usize::from(boss.survived);
                    }
                }
            }
            cells.push(cell);
        }
    }
    Ok(NaturalPlanReport {
        schema_version:2,
        sample_policy:"tactical bot; baseline vs core/support preference; identical initial seeds".into(),
        samples_per_cell:samples,
        interpretation:"All 15 blueprints use legal normal starter decks and actual reward rolls. Terminal formed means distinct core card IDs held at terminal state; boss_core_formed and boss_full_formed count actual boss entries with those cards already owned, before boss rewards. Boss wins are per encountered boss battle, not run wins. Market, upgrades and strategy are unchanged. Tiny seed counts are exploratory, not proof of natural win-rate balance or boss-specific matchup.".into(),
        cells,
    })
}

/// Keep report formatting next to its metrics so the binary entry point stays small.
pub(super) fn run_natural_plan_report(samples: usize, output: Option<&str>) -> Result<(), String> {
    let report = natural_plan_report(samples)?;
    println!("NATURAL BUILDS: {} cells, {} paired normal-starter runs; seed count is exploratory.",
        report.cells.len(), report.cells.len() * samples * 2);
    for plan in PLANS {
        let rows = report.cells.iter().filter(|c| c.id == plan.id).collect::<Vec<_>>();
        let n = rows.iter().map(|c| c.samples).sum::<usize>();
        let base = rows.iter().map(|c| c.baseline_wins).sum::<usize>();
        let focus = rows.iter().map(|c| c.focused_wins).sum::<usize>();
        let terminal = rows.iter().map(|c| c.complete_core).sum::<usize>();
        let boss = rows.iter().map(|c| c.boss_battles).sum::<usize>();
        let boss_core = rows.iter().map(|c| c.boss_core_formed).sum::<usize>();
        let boss_full = rows.iter().map(|c| c.boss_full_formed).sum::<usize>();
        let boss_wins = rows.iter().map(|c| c.boss_wins).sum::<usize>();
        println!("{}: baseline {base}/{n}, focused {focus}/{n}, terminal core {terminal}/{n}; boss entries {boss}, core ready {boss_core}/{boss}, full ready {boss_full}/{boss}, boss wins {boss_wins}/{boss}", plan.id);
    }
    if let Some(path) = output {
        fs::write(path, serde_json::to_string_pretty(&report)
            .map_err(|e| format!("serialize natural report: {e}"))?)
            .map_err(|e| format!("write {path}: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formation_is_frozen_at_boss_entry_not_inferred_from_terminal_deck() {
        let plan = &PLANS[0];
        let mut game = Game::new(plan.hero, 26026);
        let before = NaturalBossStart::capture(&game, plan);
        for id in plan.core.iter().chain(plan.support.iter()) {
            game.deck.push(Card::new(id));
        }
        let old = before.finish(false);
        let new = NaturalBossStart::capture(&game, plan).finish(true);
        assert!(!old.core_complete && !old.support_complete);
        assert!(new.core_complete && new.support_complete);
        assert_eq!(old.survived, false);
        assert_eq!(new.survived, true);
        assert!(old.core_owned < new.core_owned);
        assert!(old.support_owned < new.support_owned);
    }
}
