//! Deterministic headless simulation for balancing and softlock discovery.
//! Run: cargo run --release --bin tks-sim -- 250
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use serde::Serialize;
use three_kingdoms_spire_reborn::{
    Action, Card, CombatCounters, Game, Intent, Node, Stage,
    archetypes::Doctrine,
    builds::{BuildPlan, PLANS},
    content::{Effect, Hero, card_spec},
};

const MAX_ACTIONS: u32 = 6_000;
const HEROES: [Hero; 5] = [
    Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang,
    Hero::CaoCao, Hero::LuBu,
];

#[derive(Debug)]
struct Outcome {
    hero: Hero,
    seed: u64,
    floor: u8,
    won: bool,
    actions: u32,
    snapshot: String,
    telemetry: CombatCounters,
    death_encounter: Option<String>,
    death_node: Option<Node>,
    /// Distinct foe IDs observed at a combat decision point anywhere in the run.
    /// Run-level exposure is not a per-encounter win/loss denominator.
    encountered_foes: Vec<String>,
    reward_screens: u32,
    focus_offer_screens: u32,
    focus_picks: u32,
    discard_distinct: usize,
    combo_distinct: usize,
}
#[derive(Clone, Copy, Debug)]
enum Policy { FirstPlayable, Tactical }


fn preferred_action(game: &Game) -> Result<Action, String> {
    match game.stage {
        Stage::Map => Ok(Action::Choose { index: 1 }),
        Stage::Reward => Ok(Action::Choose { index: 1 }),
        Stage::Ancient => Ok(Action::Choose { index: 1 }),
        Stage::Camp => Ok(Action::Choose { index: 1 }),
        Stage::Market => Ok(Action::Choose { index: 0 }),
        Stage::Story => Ok(Action::Choose { index: 3 }),
        Stage::Combat => {
            let battle = game.battle.as_ref().ok_or("no battle data")?;
            let target = battle.foes.iter().position(|f| f.hp > 0)
                .map_or(1, |i| i+1);
            let playable = battle.hand.iter().enumerate().find(|(_,card)| {
                card_spec(&card.id).is_some_and(|spec| {
                    let cost = if spec.cost == -1 { battle.energy }
                    else if card.enchantment.as_deref() == Some("迅捷") {
                        (spec.cost - 1).max(0)
                    } else { spec.cost };
                    !matches!(spec.effect,Effect::Unplayable) && cost <= battle.energy
                })
            });
            if let Some((index,_)) = playable {
                Ok(Action::Play { index: index+1, target })
            } else { Ok(Action::EndTurn) }
        }
        Stage::Won | Stage::Lost => Err("terminal state".into()),
    }
}


fn card_energy(card: &Card, available: i32) -> Option<i32> {
    let spec=card_spec(&card.id)?;
    if matches!(spec.effect,Effect::Unplayable) {return None;}
    Some(if spec.cost == -1 {available}
        else if card.enchantment.as_deref() == Some("迅捷") {(spec.cost-1).max(0)}
        else {spec.cost})
}
fn next_enemy_damage(game: &Game) -> i32 {
    game.battle.as_ref().map_or(0,|battle|
        battle.foes.iter().filter(|f|f.hp>0).map(|f| match f.intent {
            Intent::Attack(n) => n,
            Intent::MultiAttack(n,hits) => n*i32::from(hits),
            Intent::Boss(ref move_id) => {
                use three_kingdoms_spire_reborn::BossMove;
                match move_id {
                    BossMove::Thunder(n) | BossMove::TyrantLevy(n) => *n,
                    BossMove::TigerRush(n) => *n*(2+i32::from(f.phase>0)),
                    _ => 0,
                }
            }
            _ => 0,
        }).sum())
}
fn tactical_card_value(game:&Game,card:&Card,threat:i32)->i32 {
    let spec=match card_spec(&card.id){Some(s)=>s,None=>return -999};
    // Outside combat, estimate reward value against a standard 3-energy turn.
    let available=game.battle.as_ref().map_or(3,|b|b.energy);
    let cost=card_energy(card,available).unwrap_or(999);
    if cost>available {return -999;}
    let upfront=match spec.effect {
        Effect::Hit(n) | Effect::All(n) | Effect::Leech(n) | Effect::Pierce(n)
        | Effect::DiscardStrike(n) => n*2,
        Effect::HitTwice(n) => n*4,
        Effect::HitDraw(n,draw) => n*2 + draw as i32*9,
        Effect::HitWeak(n,_) | Effect::HitVulnerable(n,_) => n*2+9,
        Effect::HitPoison(n,poison) => n*2+poison*2,
        Effect::Guard(n) => (n*2).min((threat+5)*2),
        Effect::GuardEnergy(n) => (n*2).min((threat+5)*2)+10,
        Effect::Poison(n) => n*3,
        Effect::Draw(n) => n as i32*10,
        Effect::Strength(n) => n*19,
        Effect::Heal(n) => n.min(game.max_hp-game.hp)*2,
        Effect::WeakAll(_) | Effect::ApplyVulnerable(_) => 16,
        Effect::GainMomentum(n) | Effect::Scheme(n) | Effect::Command(n)
            | Effect::Fury(n) => n*8,
        Effect::MomentumStrike(n) | Effect::SchemeStrike(n) |
        Effect::CommandStrike(n) | Effect::FuryStrike(n) |
        Effect::ComboStrike(n) => n*2+18,
        Effect::BloodPrice(_,n) => n*2,
        Effect::Thorns(n) => n*9,
        Effect::DrawEnergy(n,energy) => n as i32*10 + energy*13,
        Effect::Energy(n) => n*12,
        Effect::Companion(n) => n*3,
        Effect::Tactics(n) => n*8,
        Effect::Power(_,_) => 38,
        Effect::XHit(n) => n*cost*2,
        Effect::XGuard(n) => (n*cost*2).min((threat+5)*2),
        Effect::TutorAttack => 21,
        Effect::CopyDiscard => 14,
        Effect::Generate(_,n) => 8*n as i32,
        Effect::Unplayable => -999,
    };
    let attack = matches!(spec.effect,
        Effect::Hit(_) | Effect::HitTwice(_) | Effect::HitDraw(_,_) |
        Effect::All(_) | Effect::HitWeak(_,_) | Effect::HitVulnerable(_,_) |
        Effect::MomentumStrike(_) | Effect::SchemeStrike(_) |
        Effect::ComboStrike(_) | Effect::CommandStrike(_) | Effect::FuryStrike(_) |
        Effect::BloodPrice(_,_) | Effect::Leech(_) | Effect::HitPoison(_,_) |
        Effect::Pierce(_) | Effect::DiscardStrike(_) | Effect::XHit(_));
    upfront + if attack {5} else {0}
        + i32::from(card.upgraded)*5
        - cost*3
        - i32::from(matches!(spec.effect,Effect::XHit(_) | Effect::XGuard(_))
            && cost==0)*20
}
fn tactical_reward(game:&Game)->usize {
    game.rewards.iter().enumerate()
        .max_by_key(|(idx,card)| {
            let novelty= if game.deck.iter().any(|owned|owned.id==card.id){-10}else{10};
            let is_power=card_spec(&card.id)
                .is_some_and(|s|matches!(s.effect,Effect::Power(_,_)));
            let focus_synergy=if game.doctrine.is_some_and(|focus|focus.contains(&card.id))
                {32}else{0};
            let sun_synergy=if game.hero==Hero::SunShangxiang &&
                card_spec(&card.id).is_some_and(|spec|matches!(spec.effect,
                    Effect::Tactics(_) | Effect::DiscardStrike(_) |
                    Effect::Power(three_kingdoms_spire_reborn::combat::PowerId::ArrowsOnDiscard,_)))
                {20}else{0};
            (novelty+if is_power{14}else{0}+sun_synergy+focus_synergy
                +tactical_card_value(game,card,14),-(*idx as i32))
        })
        .map_or(0,|(i,card)|if card_spec(&card.id).is_some_and(|s|matches!(s.effect,Effect::Unplayable)){0}else{i+1})
}
fn tactical_action(game:&Game)->Result<Action,String> {
    match game.stage {
        Stage::Map => {
            let mut best=(i32::MIN,1);
            for (i,node) in game.route.iter().enumerate() {
                let deficit=(game.max_hp-game.hp).max(0);
                let score=match node {
                    Node::Camp => if deficit>=15 {45}else{22},
                    Node::Market => if game.gold>=150 {32}else{5},
                    Node::Story => 24,
                    Node::Battle => if deficit>25 {-5}else{17},
                    Node::Elite => if deficit>25 {-20}else{7},
                    Node::Boss => 50,
                };
                if score>best.0 {best=(score,i+1);}
            }
            Ok(Action::Choose{index:best.1})
        }
        Stage::Reward => Ok(Action::Choose{index:tactical_reward(game)}),
        Stage::Ancient => Ok(Action::Choose{index:1}),
        Stage::Camp => {
            let deficit=game.max_hp-game.hp;
            let can_upgrade=game.deck.iter().any(|card|!card.upgraded&&card.id!="cursed");
            Ok(Action::Choose {index:if deficit >= 12 {1}
                else if can_upgrade {2} else {3}})
        }
        Stage::Market => {
            let can_remove=game.deck.iter().any(|c|c.id=="cursed");
            Ok(Action::Choose{index:if can_remove&&game.gold>=75 {6}
                else if game.shop_relic.is_some()&&game.gold>=155 {5}
                else {0}})
        }
        Stage::Story => Ok(Action::Choose{index:if game.lore_current.is_some(){1}else{3}}),
        Stage::Combat => {
            let b=game.battle.as_ref().ok_or("missing tactical battle")?;
            // Build-focused draw/energy cycles can yield unbounded optional plays.
            // Human players can press "end"; the bot must not interpret such
            // positive-energy loops as a hard game-engine softlock.
            if b.played_this_turn >= 36 {
                return Ok(Action::EndTurn);
            }
            let threat=next_enemy_damage(game);
            // Discard is a separately-paid action and matters especially for
            // Sun Shangxiang's combo. A no-discard bot badly understates her
            // power when it rewards cards granting discard tokens.
            if b.discard_tokens > 0 {
                if let Some((index,_))=b.hand.iter().enumerate()
                    .find(|(_,card)|card_spec(&card.id)
                        .is_some_and(|spec|matches!(spec.effect,Effect::Unplayable)))
                {
                    return Ok(Action::Discard{index:index+1});
                }
                if b.energy==0 && !b.hand.is_empty() {
                    return Ok(Action::Discard{index:b.hand.len()});
                }
            }
            let target=b.foes.iter().enumerate().filter(|(_,f)|f.hp>0)
                .min_by_key(|(_,f)|f.hp+f.block)
                .map_or(1,|(idx,_)|idx+1);
            if let Some((slot,_))=game.potion_bag.iter().enumerate().find(|(_,id)|
                matches!(id.as_str(),"healing"|"cleanse") &&
                (game.max_hp-game.hp>=18 || game.hp<=game.max_hp/3))
            {
                return Ok(Action::UsePotionAt{slot:slot+1,target});
            }
            if b.energy<=1 && b.hand.iter().any(|card|card_energy(card,3).is_some_and(|cost|cost>=2))
                && let Some(slot)=game.potion_bag.iter().position(|id|id=="energy")
            {return Ok(Action::UsePotionAt{slot:slot+1,target})}
            let playable=b.hand.iter().enumerate()
                .filter(|(_,card)|card_energy(card,b.energy).is_some_and(|cost|cost<=b.energy))
                .map(|(i,card)|(i,tactical_card_value(game,card,threat)))
                .filter(|(_,score)|*score>-100)
                .max_by_key(|(idx,score)|(*score,-(*idx as i32)));
            if let Some((index,_))=playable {Ok(Action::Play{index:index+1,target})}
            else {Ok(Action::EndTurn)}
        }
        Stage::Won|Stage::Lost => Err("terminal tactical state".into()),
    }
}
fn preferred_action_policy(game:&Game,policy:Policy)->Result<Action,String> {
    match policy {
        Policy::FirstPlayable=>preferred_action(game),
        Policy::Tactical=>tactical_action(game),
    }
}

fn simulate(hero: Hero, seed: u64, ascension: u8) -> Result<Outcome, String> {
    simulate_policy(hero,seed,ascension,Policy::FirstPlayable)
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum SunPackage { Standard, Discard, Combo }
impl SunPackage {
    const ALL: [Self;3] = [Self::Standard,Self::Discard,Self::Combo];
    fn id(self)->&'static str {
        match self {Self::Standard=>"standard",Self::Discard=>"discard",Self::Combo=>"combo"}
    }
    fn cards(self)->Option<[&'static str;4]> {
        match self {
            Self::Standard=>None,
            Self::Discard=>Some(["huanshe","feishi","arrows_on_discard","jiecang"]),
            Self::Combo=>Some(["liandao","gonglian","lianhua","fleeting_blade"]),
        }
    }
}
fn apply_sun_package(game:&mut Game,package:SunPackage)->Result<(),String> {
    if game.hero!=Hero::SunShangxiang {
        return Err("Sun package experiments require Sun Shangxiang".into());
    }
    if let Some(cards)=package.cards() {
        // Equal deck size; swap two strikes and two guards for a curated package.
        // This changes the starting deck and is NOT a normal-mode victory claim.
        for (idx,id) in [0usize,1,5,6].into_iter().zip(cards) {
            let expected=if idx<5 {"strike"} else {"defend"};
            if game.deck[idx].id!=expected {
                return Err("unexpected baseline starter slots".into());
            }
            if card_spec(id).is_none() {return Err(format!("unknown Sun package card {id}"));}
            game.deck[idx]=Card::new(id);
        }
    }
    Ok(())
}
fn simulate_policy(hero:Hero,seed:u64,ascension:u8,policy:Policy)->Result<Outcome,String> {
    simulate_policy_package(hero,seed,ascension,policy,SunPackage::Standard)
}
fn simulate_policy_package(hero:Hero,seed:u64,ascension:u8,policy:Policy,package:SunPackage)
    ->Result<Outcome,String>
{
    simulate_config(hero,seed,ascension,policy,package,None)
}
fn simulate_config(hero:Hero,seed:u64,ascension:u8,policy:Policy,package:SunPackage,
    focus:Option<Doctrine>)->Result<Outcome,String>
{
    simulate_config_blueprint(hero,seed,ascension,policy,package,focus,DeckTreatment::Standard)
}
/// Curated card injections and normal-drafting focus cannot coexist.
#[derive(Clone,Copy)]
enum DeckTreatment<'a> {
    Standard,
    Curated(&'a BuildPlan),
    Natural(&'a BuildPlan),
}
/// Paired runs keep the same initial seed and tactical policy, with one
/// explicit, type-safe treatment choice and no eighth configuration argument.
fn simulate_config_blueprint(hero:Hero,seed:u64,ascension:u8,policy:Policy,
    package:SunPackage,focus:Option<Doctrine>,treatment:DeckTreatment<'_>)
    ->Result<Outcome,String>
{
    let (blueprint,natural_plan)=match treatment {
        DeckTreatment::Standard=>(None,None),
        DeckTreatment::Curated(plan)=>(Some(plan),None),
        DeckTreatment::Natural(plan)=>(None,Some(plan)),
    };
    if natural_plan.is_some_and(|plan|plan.hero!=hero) {
        return Err("natural plan hero mismatch".into());
    }
    let mut game = Game::new_with_ascension(hero, seed, ascension);
    if let Some(plan)=blueprint { apply_curated_plan(&mut game,plan)?; }
    if package!=SunPackage::Standard {apply_sun_package(&mut game,package)?;}
    if let Some(focus)=focus {
        game.apply(Action::SetDoctrine{id:focus.id().into()})?;
    }
    let mut reward_screens=0;
    let mut focus_offer_screens=0;
    let mut focus_picks=0;
    let mut death_context:Option<(String,Node)>=None;
    let mut encountered_foes=BTreeSet::<String>::new();
    for count in 0..MAX_ACTIONS {
        if matches!(game.stage,Stage::Won | Stage::Lost) {
            return Ok(Outcome {
                hero, seed, floor: game.floor, won: game.stage == Stage::Won,
                actions: count, snapshot: game.to_json()?,
                telemetry: game.telemetry.totals,
                death_encounter:if game.stage==Stage::Won {None}
                    else {death_context.as_ref().map(|(id,_)|id.clone())},
                death_node:if game.stage==Stage::Won {None}
                    else {death_context.as_ref().map(|(_,node)|*node)},
                encountered_foes:encountered_foes.into_iter().collect(),
                reward_screens,
                focus_offer_screens,
                focus_picks,
                discard_distinct:Doctrine::SunDiscard.distinct_owned(&game.deck),
                combo_distinct:Doctrine::SunCombo.distinct_owned(&game.deck),
            });
        }
        let mut action = preferred_action_policy(&game,policy)?;
        if game.stage==Stage::Reward {
            reward_screens+=1;
            if let Some(plan)=natural_plan {
                // Focusing modifies only genuine reward *choices*. The
                // starting deck, RNG, map, market and reward rolls stay legal.
                // Never pick an already-owned core/support card merely for
                // the appearance of "formation".
                let requested=|id:&str| {
                    !game.deck.iter().any(|card|card.id==id)
                        && (plan.core.contains(&id) || plan.support.contains(&id))
                };
                if game.rewards.iter().any(|c|requested(&c.id)) {
                    focus_offer_screens+=1;
                }
                if let Some((index,_))=game.rewards.iter().enumerate()
                    .filter(|(_,card)|requested(&card.id))
                    .max_by_key(|(index,card)| (
                        i32::from(plan.core.contains(&card.id.as_str())),
                        tactical_card_value(&game,card,14),
                        -(*index as i32)
                    )) {
                    action=Action::Choose{index:index+1};
                    focus_picks+=1;
                }
            } else if let Some(active)=game.effective_doctrine() {
                if game.rewards.iter().any(|c|active.contains(&c.id)) {
                    focus_offer_screens+=1;
                }
                if let Action::Choose{index}=action
                    && index>0 && game.rewards.get(index-1).is_some_and(|c|active.contains(&c.id))
                {
                    focus_picks+=1;
                }
            }
        }
        if game.stage==Stage::Combat {
            if let Some(b)=&game.battle {
                for foe in &b.foes {
                    encountered_foes.insert(foe.id.clone());
                }
                // Encounter context, not a claim about the exact killing blow.
                let id=b.foes.iter().filter(|f|f.hp>0)
                    .map(|f|f.id.as_str()).collect::<Vec<_>>().join("+");
                death_context=Some((id,game.last_node.unwrap_or(Node::Battle)));
            }
        } else {death_context=None;}
        game.apply(action).map_err(|e|
            format!("{} seed={} step={} floor={} stage={:?}: {}",
                hero.name(),seed,count,game.floor,game.stage,e))?;
        // A state snapshot must validate on every action, not just at the end.
        if count.is_multiple_of(13) {
            let snapshot = game.to_json()?;
            let restored = Game::from_json(&snapshot)?;
            if restored.to_json()? != snapshot {
                return Err("snapshot changed after round trip".into());
            }
        }
    }
    Err(format!("suspected softlock: {} seed={} exceeded {} actions at floor={} stage={:?} hp={}/{} current_turn={:?}",
        hero.name(),seed,MAX_ACTIONS,game.floor,game.stage,game.hp,game.max_hp,
        game.battle.as_ref().map(|b|(b.turn,b.played_this_turn,b.energy,
            b.foes.iter().map(|f|(&f.id,f.hp)).collect::<Vec<_>>()))))
}


#[derive(Debug,Serialize)]
struct BalanceCell {
    hero:String,
    ascension:u8,
    samples:usize,
    baseline_wins:usize,
    tactical_wins:usize,
    baseline_floor_sum:u64,
    tactical_floor_sum:u64,
    baseline_boss_deaths:usize,
    tactical_boss_deaths:usize,
    baseline_telemetry:CombatCounters,
    tactical_telemetry:CombatCounters,
}
#[derive(Debug,Serialize)]
struct EncounterEntry {
    encounter:String,
    deaths:usize,
}
#[derive(Debug,Serialize)]
struct BalanceReport {
    schema_version:u8,
    seed_formula:String,
    samples_per_cell:usize,
    policies:[String;2],
    cells:Vec<BalanceCell>,
    baseline_death_encounters:Vec<EncounterEntry>,
    tactical_death_encounters:Vec<EncounterEntry>,
}
fn ranked_encounters(entries:BTreeMap<String,usize>)->Vec<EncounterEntry> {
    let mut all=entries.into_iter().map(|(encounter,deaths)|
        EncounterEntry{encounter,deaths}).collect::<Vec<_>>();
    all.sort_by(|a,b|b.deaths.cmp(&a.deaths)
        .then_with(||a.encounter.cmp(&b.encounter)));
    all
}
fn build_report(samples_per_cell:usize)->Result<BalanceReport,String> {
    let mut cells=Vec::with_capacity(55);
    let mut baseline_deaths=BTreeMap::new();
    let mut tactical_deaths=BTreeMap::new();
    for (hero_index,hero) in HEROES.iter().enumerate() {
        for ascension in 0..=10_u8 {
            let mut cell=BalanceCell{
                hero:hero.name().into(),ascension,samples:samples_per_cell,
                baseline_wins:0,tactical_wins:0,
                baseline_floor_sum:0,tactical_floor_sum:0,
                baseline_boss_deaths:0,tactical_boss_deaths:0,
                baseline_telemetry:CombatCounters::default(),
                tactical_telemetry:CombatCounters::default(),
            };
            for iteration in 0..samples_per_cell {
                // Independently fixed inputs for each difficulty, hero and trial.
                let ordinal=(hero_index as u64)*11*10_000+
                    u64::from(ascension)*10_000+iteration as u64+1;
                let seed=ordinal.wrapping_mul(0x9E3779B97F4A7C15);
                for policy in [Policy::FirstPlayable,Policy::Tactical] {
                    let outcome=simulate_policy(*hero,seed,ascension,policy)?;
                    let (wins,floors,boss_deaths,deaths)=match policy {
                        Policy::FirstPlayable=>(&mut cell.baseline_wins,
                            &mut cell.baseline_floor_sum,&mut cell.baseline_boss_deaths,
                            &mut baseline_deaths),
                        Policy::Tactical=>(&mut cell.tactical_wins,
                            &mut cell.tactical_floor_sum,&mut cell.tactical_boss_deaths,
                            &mut tactical_deaths),
                    };
                    *wins+=usize::from(outcome.won);
                    *floors+=u64::from(outcome.floor);
                    match policy {
                        Policy::FirstPlayable=>cell.baseline_telemetry.accumulate(&outcome.telemetry),
                        Policy::Tactical=>cell.tactical_telemetry.accumulate(&outcome.telemetry),
                    }
                    if !outcome.won {
                        if outcome.death_node==Some(Node::Boss){*boss_deaths+=1;}
                        let encounter=outcome.death_encounter.unwrap_or_else(||"non-combat".into());
                        *deaths.entry(encounter).or_insert(0usize)+=1;
                    }
                }
            }
            cells.push(cell);
        }
    }
    Ok(BalanceReport {
        schema_version:1,
        seed_formula:"(((hero_index*11+ascension)*10000+trial+1)*0x9E3779B97F4A7C15) mod 2^64".into(),
        samples_per_cell,
        policies:["first_playable_baseline".into(),"tactical_heuristic".into()],
        cells,
        baseline_death_encounters:ranked_encounters(baseline_deaths),
        tactical_death_encounters:ranked_encounters(tactical_deaths),
    })
}
fn print_balance(report:&BalanceReport) {
    println!("BALANCE REPORT: {} samples per hero/ascension, {} paired runs; fixed seeds, policies are heuristic bots NOT human win rates.",
        report.samples_per_cell,report.samples_per_cell*55*2);
    println!("Hero | Tier | Runs | Baseline wins | Tactical wins | Baseline avg floor | Tactical avg floor | Boss encounter deaths baseline/tactical");
    println!("---|---|---:|---:|---:|---:|---:|---:");
    for hero in HEROES {
        for (tier,start,end) in [("0-3",0,3),("4-7",4,7),("8-10",8,10)] {
            let rows=report.cells.iter().filter(|c|
                c.hero==hero.name() && c.ascension>=start && c.ascension<=end).collect::<Vec<_>>();
            let n=rows.iter().map(|r|r.samples).sum::<usize>();
            let bw=rows.iter().map(|r|r.baseline_wins).sum::<usize>();
            let tw=rows.iter().map(|r|r.tactical_wins).sum::<usize>();
            let bf=rows.iter().map(|r|r.baseline_floor_sum).sum::<u64>();
            let tf=rows.iter().map(|r|r.tactical_floor_sum).sum::<u64>();
            let bd=rows.iter().map(|r|r.baseline_boss_deaths).sum::<usize>();
            let td=rows.iter().map(|r|r.tactical_boss_deaths).sum::<usize>();
            println!("{} | {} | {} | {:.1}% | {:.1}% | {:.2} | {:.2} | {}/{}",
                hero.name(),tier,n,100.0*bw as f64/n as f64,
                100.0*tw as f64/n as f64,bf as f64/n as f64,
                tf as f64/n as f64,bd,td);
        }
    }
    for hero in HEROES {
        let mut totals=CombatCounters::default();
        for cell in report.cells.iter().filter(|c|c.hero==hero.name()) {
            totals.accumulate(&cell.tactical_telemetry);
        }
        let turn_count=totals.turns_completed.max(1);
        println!("TELEMETRY {} tactical ({} runs): turns={} troop DPS/turn={:.2} absorbed/turn={:.2} command energy/turn={:.2}, Sun discards={} combo triggers={} / extra draws={}",
            hero.name(),report.samples_per_cell*11,totals.turns_completed,
            totals.troop_damage as f64/turn_count as f64,
            totals.troop_absorbed as f64/turn_count as f64,
            totals.command_energy as f64/turn_count as f64,
            totals.sun_discards,totals.sun_combo_procs,totals.sun_combo_draws);
    }
    println!("TOP BASELINE DEATH ENCOUNTERS: {:?}",
        report.baseline_death_encounters.iter().take(8)
            .map(|e|format!("{}:{}",e.encounter,e.deaths)).collect::<Vec<_>>());
    println!("TOP TACTICAL DEATH ENCOUNTERS: {:?}",
        report.tactical_death_encounters.iter().take(8)
            .map(|e|format!("{}:{}",e.encounter,e.deaths)).collect::<Vec<_>>());
}

#[derive(Debug,Serialize)]
struct NaturalDraftCell {
    policy:String,
    ascension:u8,
    samples:usize,
    wins:usize,
    floor_sum:u64,
    reward_screens:u64,
    focus_offer_screens:u64,
    focus_picks:u64,
    discard_formed:usize,
    combo_formed:usize,
    discard_distinct_sum:u64,
    combo_distinct_sum:u64,
}
#[derive(Debug,Serialize)]
struct NaturalDraftReport {
    schema_version:u8,
    study:String,
    samples_per_cell:usize,
    cells:Vec<NaturalDraftCell>,
}
fn build_natural_drafting(samples:usize)->Result<NaturalDraftReport,String>{
    let mut cells=Vec::new();
    for ascension in 0..=10_u8 {
        for (policy,focus) in [
            ("unfocused_tactical",None),
            ("choose_discard",Some(Doctrine::SunDiscard)),
            ("choose_combo",Some(Doctrine::SunCombo)),
        ] {
            let mut cell=NaturalDraftCell {
                policy:policy.into(),ascension,samples,wins:0,floor_sum:0,
                reward_screens:0,focus_offer_screens:0,focus_picks:0,
                discard_formed:0,combo_formed:0,discard_distinct_sum:0,combo_distinct_sum:0,
            };
            for trial in 0..samples {
                let ordinal=4_000_000u64+u64::from(ascension)*10_000+trial as u64+1;
                let seed=ordinal.wrapping_mul(0x9E3779B97F4A7C15);
                let outcome=simulate_config(Hero::SunShangxiang,seed,ascension,
                    Policy::Tactical,SunPackage::Standard,focus)?;
                cell.wins+=usize::from(outcome.won);
                cell.floor_sum+=u64::from(outcome.floor);
                cell.reward_screens+=u64::from(outcome.reward_screens);
                cell.focus_offer_screens+=u64::from(outcome.focus_offer_screens);
                cell.focus_picks+=u64::from(outcome.focus_picks);
                cell.discard_formed+=usize::from(outcome.discard_distinct>=2);
                cell.combo_formed+=usize::from(outcome.combo_distinct>=2);
                cell.discard_distinct_sum+=outcome.discard_distinct as u64;
                cell.combo_distinct_sum+=outcome.combo_distinct as u64;
            }
            cells.push(cell);
        }
    }
    Ok(NaturalDraftReport{
        schema_version:1,
        study:"Sun Shangxiang normal ten-card starter; doctrine choice only, no card injection; all three policies paired on the same seeds".into(),
        samples_per_cell:samples,cells,
    })
}
fn print_natural_drafting(report:&NaturalDraftReport){
    println!("NATURAL DRAFT: standard starting decks, {} trials/tier/policy, no injected cards",
        report.samples_per_cell);
    println!("Policy | Ascension band | Wins | Discard formed | Combo formed | Focus offer screens / total rewards | Focus cards chosen");
    for policy in ["unfocused_tactical","choose_discard","choose_combo"] {
        for (band,start,end) in [("0-3",0,3),("4-7",4,7),("8-10",8,10)] {
            let rows=report.cells.iter().filter(|c|c.policy==policy
                && c.ascension>=start && c.ascension<=end).collect::<Vec<_>>();
            let n=rows.iter().map(|c|c.samples).sum::<usize>();
            let wins=rows.iter().map(|c|c.wins).sum::<usize>();
            let d=rows.iter().map(|c|c.discard_formed).sum::<usize>();
            let comb=rows.iter().map(|c|c.combo_formed).sum::<usize>();
            let offered=rows.iter().map(|c|c.focus_offer_screens).sum::<u64>();
            let rewards=rows.iter().map(|c|c.reward_screens).sum::<u64>();
            let picked=rows.iter().map(|c|c.focus_picks).sum::<u64>();
            println!("{policy} | {band} | {wins}/{n} ({:.1}%) | {d}/{n} | {comb}/{n} | {offered}/{rewards} | {picked}",
                wins as f64*100.0/n as f64);
        }
    }
}

#[derive(Debug,Serialize)]
struct SunArchetypeCell {
    package:String,
    ascension:u8,
    samples:usize,
    wins:usize,
    floor_sum:u64,
    telemetry:CombatCounters,
}
#[derive(Debug,Serialize)]
struct SunArchetypeReport {
    schema_version:u8,
    sample_policy:String,
    rules:String,
    samples_per_cell:usize,
    cells:Vec<SunArchetypeCell>,
}
fn build_sun_archetypes(samples:usize)->Result<SunArchetypeReport,String> {
    let mut cells=Vec::new();
    for ascension in 0..=10_u8 {
        for package in SunPackage::ALL {
            let mut cell=SunArchetypeCell {package:package.id().into(),ascension,samples,
                wins:0,floor_sum:0,telemetry:CombatCounters::default()};
            for trial in 0..samples {
                let ordinal=3_000_000u64+u64::from(ascension)*10_000+trial as u64+1;
                let seed=ordinal.wrapping_mul(0x9E3779B97F4A7C15);
                let run=simulate_policy_package(Hero::SunShangxiang,seed,ascension,
                    Policy::Tactical,package)?;
                cell.wins+=usize::from(run.won);
                cell.floor_sum+=u64::from(run.floor);
                cell.telemetry.accumulate(&run.telemetry);
            }
            cells.push(cell);
        }
    }
    Ok(SunArchetypeReport {
        schema_version:1,
        sample_policy:"tactical_heuristic".into(),
        rules:"Curated package swaps precisely two strikes and two guards; different starting decks; identical seeds, not a normal-mode win-rate estimate".into(),
        samples_per_cell:samples,
        cells,
    })
}
fn print_sun_archetypes(report:&SunArchetypeReport) {
    println!("SUN ARCHETYPES: 33 cells, {} trials per tier/package ({} total), equal-size curated starting decks; experimental, not human win rates.",
        report.samples_per_cell,report.samples_per_cell*33);
    println!("Package | Tier | Wins | Mean floor | Manual discards | Combo triggers | Guard from hero passives");
    for package in SunPackage::ALL {
        for (tier,start,end) in [("0-3",0,3),("4-7",4,7),("8-10",8,10)] {
            let rows=report.cells.iter().filter(|c|c.package==package.id()
                && c.ascension>=start && c.ascension<=end).collect::<Vec<_>>();
            let samples=rows.iter().map(|c|c.samples).sum::<usize>();
            let wins=rows.iter().map(|c|c.wins).sum::<usize>();
            let floor=rows.iter().map(|c|c.floor_sum).sum::<u64>();
            let mut sum=CombatCounters::default();
            for cell in rows {sum.accumulate(&cell.telemetry);}
            println!("{} | {} | {}/{} ({:.1}%) | {:.2} | {} | {} | {}",
                package.id(),tier,wins,samples,100.0*wins as f64/samples as f64,
                floor as f64/samples as f64,
                sum.sun_discards,sum.sun_combo_procs,sum.sun_discard_block+sum.sun_combo_block);
        }
    }
}

/// Controlled build fixture: preserve ten-card deck and starting resource
/// budget. Swapping four strikes and two guards is NOT a normal reward path.
fn apply_curated_plan(game:&mut Game,plan:&BuildPlan)->Result<(),String> {
    if game.hero!=plan.hero || !plan.is_catalog_valid() {
        return Err("invalid or cross-hero curated plan".into());
    }
    for (id,idx) in plan.core.iter().chain(plan.support.iter())
        .copied().zip([0usize,1,2,3,5,6])
    {
        if game.deck[idx].id!=if idx<5 {"strike"} else {"defend"} {
            return Err("invalid baseline deck for blueprint".into());
        }
        game.deck[idx]=Card::new(id);
    }
    Ok(())
}
#[derive(Debug,Serialize)]
struct BuildCell {
    id:String,
    hero:String,
    counter_enemy:String,
    ascension:u8,
    samples:usize,
    baseline_wins:usize,
    blueprint_wins:usize,
    baseline_floor_sum:u64,
    blueprint_floor_sum:u64,
    /// Paired runs (not battles) with at least one actual sighting of the
    /// nominated counter foe; counts both wins and losses.
    baseline_counter_exposed_runs:usize,
    blueprint_counter_exposed_runs:usize,
    /// Losses where the last recorded live encounter included this enemy.
    /// A run-level exposure denominator is NOT a per-encounter defeat rate.
    counter_last_encounter_deaths:usize,
}
#[derive(Debug,Serialize)]
struct BuildBlueprintReport {
    schema_version:u8,
    seed_formula:String,
    interpretation:String,
    samples_per_cell:usize,
    cells:Vec<BuildCell>,
}
fn build_blueprint_report(samples:usize)->Result<BuildBlueprintReport,String> {
    let mut cells=Vec::new();
    for (plan_index,plan) in PLANS.iter().enumerate() {
        for ascension in 0..=10_u8 {
            let mut cell=BuildCell {
                id:plan.id.into(),hero:plan.hero.name().into(),
                counter_enemy:plan.counter_enemy.into(),ascension,samples,
                baseline_wins:0,blueprint_wins:0,baseline_floor_sum:0,
                blueprint_floor_sum:0,baseline_counter_exposed_runs:0,
                blueprint_counter_exposed_runs:0,counter_last_encounter_deaths:0,
            };
            for trial in 0..samples {
                let ordinal=(plan_index as u64)*11*10_000+
                    u64::from(ascension)*10_000+trial as u64+1;
                let seed=ordinal.wrapping_mul(0x9E3779B97F4A7C15);
                let baseline=simulate_config_blueprint(plan.hero,seed,ascension,
                    Policy::Tactical,SunPackage::Standard,None,DeckTreatment::Standard)?;
                let curated=simulate_config_blueprint(plan.hero,seed,ascension,
                    Policy::Tactical,SunPackage::Standard,None,DeckTreatment::Curated(plan))?;
                cell.baseline_wins+=usize::from(baseline.won);
                cell.blueprint_wins+=usize::from(curated.won);
                cell.baseline_floor_sum+=u64::from(baseline.floor);
                cell.blueprint_floor_sum+=u64::from(curated.floor);
                // Denominator: unique runs that really saw the named enemy;
                // never infer encounters from terminal deaths or a blueprint ID.
                cell.baseline_counter_exposed_runs+=usize::from(
                    baseline.encountered_foes.iter().any(|id|id==plan.counter_enemy));
                cell.blueprint_counter_exposed_runs+=usize::from(
                    curated.encountered_foes.iter().any(|id|id==plan.counter_enemy));
                if !curated.won && curated.death_encounter.as_ref()
                    .is_some_and(|s|s.split('+').any(|id|id==plan.counter_enemy))
                { cell.counter_last_encounter_deaths+=1; }
            }
            cells.push(cell);
        }
    }
    Ok(BuildBlueprintReport {
        schema_version:1,samples_per_cell:samples,
        seed_formula:"((plan_index*110000+ascension*10000+trial+1)*0x9E3779B97F4A7C15) mod 2^64".into(),
        interpretation:"equal 10-card curated fixture, four strikes and two guards replaced; paired fixed seeds, tactical heuristic bot, not natural draft nor human win rate; counter exposure counts distinct runs with a foe present at combat decision time (not total encounters); last-encounter death is not a matchup defeat rate".into(),
        cells,
    })
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
            }
            cells.push(cell);
        }
    }
    Ok(NaturalPlanReport {
        schema_version:1,
        sample_policy:"tactical bot; baseline vs core/support preference; identical initial seeds".into(),
        samples_per_cell:samples,
        interpretation:"All 15 blueprints use legal normal starter decks and actual reward rolls. Formed means distinct core card IDs held at terminal state. Market, upgrades and strategy are unchanged. Tiny seed counts are exploratory, not proof of natural win-rate balance or boss-specific matchup.".into(),
        cells,
    })
}

fn main() {
    let arguments=std::env::args().collect::<Vec<_>>();
    if arguments.get(1).is_some_and(|arg|arg=="--natural-builds") {
        let samples=arguments.get(2).and_then(|s|s.parse::<usize>().ok())
            .unwrap_or(1).clamp(1,20);
        let report=natural_plan_report(samples)
            .unwrap_or_else(|e|panic!("natural 15-plan study failed: {e}"));
        println!("NATURAL BUILDS: {} cells, {} paired normal-starter runs; seed count is exploratory.",
            report.cells.len(),report.cells.len()*samples*2);
        for plan in PLANS {
            let rows=report.cells.iter().filter(|c|c.id==plan.id).collect::<Vec<_>>();
            let n=rows.iter().map(|c|c.samples).sum::<usize>();
            let base=rows.iter().map(|c|c.baseline_wins).sum::<usize>();
            let focus=rows.iter().map(|c|c.focused_wins).sum::<usize>();
            let formed=rows.iter().map(|c|c.complete_core).sum::<usize>();
            println!("{}: baseline {base}/{n}, focused {focus}/{n}, full core {formed}/{n}",plan.id);
        }
        if let Some(path)=arguments.get(3) {
            fs::write(path,serde_json::to_string_pretty(&report).expect("serialize natural report"))
                .unwrap_or_else(|e|panic!("write {path}: {e}"));
        }
        return;
    }
    if arguments.get(1).is_some_and(|arg|arg=="--build-archetypes") {
        let samples=arguments.get(2).and_then(|s|s.parse::<usize>().ok())
            .unwrap_or(2).clamp(1,20);
        let report=build_blueprint_report(samples)
            .unwrap_or_else(|e|panic!("paired build fixture failed: {e}"));
        println!("BUILD FIXTURES: {} cells, {} paired simulated runs (curated ten-card decks, not human win rates).",
            report.cells.len(),report.cells.len()*samples*2);
        for plan in PLANS {
            let cells=report.cells.iter().filter(|c|c.id==plan.id).collect::<Vec<_>>();
            let n=cells.iter().map(|c|c.samples).sum::<usize>();
            let wins=cells.iter().map(|c|c.blueprint_wins).sum::<usize>();
            let baseline=cells.iter().map(|c|c.baseline_wins).sum::<usize>();
            println!("{}: curated {wins}/{n}, paired starter {baseline}/{n}",plan.id);
        }
        if let Some(path)=arguments.get(3) {
            fs::write(path,serde_json::to_string_pretty(&report)
                .expect("serialize 15-build report")).unwrap_or_else(|e|panic!("write {path}: {e}"));
        }
        return;
    }
    if arguments.get(1).is_some_and(|arg|arg=="--natural-drafting") {
        let samples=arguments.get(2).and_then(|s|s.parse::<usize>().ok())
            .unwrap_or(15).clamp(1,200);
        let report=build_natural_drafting(samples)
            .unwrap_or_else(|e|panic!("natural drafting simulation failed: {e}"));
        print_natural_drafting(&report);
        if let Some(path)=arguments.get(3) {
            fs::write(path,serde_json::to_string_pretty(&report)
                .expect("serialize natural draft")).unwrap_or_else(|e|panic!("write {path}: {e}"));
        }
        return;
    }
    if arguments.get(1).is_some_and(|arg|arg=="--sun-archetypes") {
        let samples=arguments.get(2).and_then(|raw|raw.parse::<usize>().ok())
            .unwrap_or(15).clamp(1,200);
        let report=build_sun_archetypes(samples)
            .unwrap_or_else(|e|panic!("Sun archetype simulation failed: {e}"));
        print_sun_archetypes(&report);
        if let Some(path)=arguments.get(3) {
            let json=serde_json::to_string_pretty(&report).expect("serialize Sun report");
            fs::write(path,json).unwrap_or_else(|e|panic!("write Sun report {path}: {e}"));
        }
        return;
    }
    if arguments.get(1).is_some_and(|arg|arg=="--balance") {
        let samples=arguments.get(2).and_then(|raw|raw.parse::<usize>().ok())
            .unwrap_or(20).clamp(1,200);
        let report=build_report(samples).unwrap_or_else(|e|panic!("balance simulation failed: {e}"));
        print_balance(&report);
        if let Some(path)=arguments.get(3) {
            let json=serde_json::to_string_pretty(&report).expect("serialize balance report");
            fs::write(path,json).unwrap_or_else(|e|panic!("could not write {path}: {e}"));
            println!("Machine-readable paired balance report: {path}");
        }
        return;
    }
    let count = arguments.get(1)
        .and_then(|raw| raw.parse::<usize>().ok()).unwrap_or(100).min(20_000);
    let mut victories = 0_u64;
    let mut deepest = 0_u8;
    let mut total_actions = 0_u64;
    let mut snapshot_bytes = 0_usize;
    for n in 0..count {
        let hero = HEROES[n % HEROES.len()];
        let seed = (n as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15);
        let difficulty = ((n/5) % 11) as u8;
        match simulate(hero,seed,difficulty) {
            Ok(outcome) => {
                assert_eq!(outcome.hero, hero);
                assert_eq!(outcome.seed, seed);
                snapshot_bytes += outcome.snapshot.len();
                victories += u64::from(outcome.won);
                deepest = deepest.max(outcome.floor);
                total_actions += u64::from(outcome.actions);
            }
            Err(err) => {
                eprintln!("Simulation failure: {err}");
                std::process::exit(1);
            }
        }
    }
    println!("Simulation OK: {} runs, {} wins, maximum floor {}, {} actions, {} snapshot bytes",
        count,victories,deepest,total_actions,snapshot_bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_simulation_records_foe_exposure_on_live_battle_snapshots() {
        let outcome=simulate_policy(Hero::GuanYu,20261020,0,Policy::Tactical).unwrap();
        assert!(!outcome.encountered_foes.is_empty());
        let mut sorted=outcome.encountered_foes.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted,outcome.encountered_foes);
        if let Some(id)=&outcome.death_encounter {
            for foe in id.split('+') {
                assert!(outcome.encountered_foes.iter().any(|seen|seen==foe),
                    "last-encounter enemy {foe} missing from exposure");
            }
        }
    }
    #[test]
    fn fifteen_curated_blueprints_are_equal_size_and_saveable() {
        for plan in PLANS {
            let mut game=Game::new(plan.hero,2221);
            apply_curated_plan(&mut game,plan).unwrap();
            assert_eq!(game.deck.len(),10);
            assert_eq!(game.deck[0].id,plan.core[0]);
            assert_eq!(game.deck[5].id,plan.support[0]);
            let json=game.to_json().unwrap();
            assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(),json);
        }
        let mut wrong=Game::new(Hero::GuanYu,2221);
        assert!(apply_curated_plan(&mut wrong,&PLANS[3]).is_err());
    }
    #[test]
    fn tactical_bot_uses_discard_tokens_instead_of_ignoring_sun_mechanics() {
        let mut game=Game::new(Hero::SunShangxiang,5151);
        game.apply(Action::Choose{index:1}).unwrap();
        let b=game.battle.as_mut().unwrap();
        b.energy=0;
        b.discard_tokens=2;
        b.hand=vec![Card::new("strike")];
        assert!(matches!(tactical_action(&game).unwrap(),
            Action::Discard{index:1}));
        let snapshot=game.to_json().unwrap();
        game.apply(tactical_action(&game).unwrap()).unwrap();
        assert_ne!(game.to_json().unwrap(),snapshot);
        assert_eq!(game.battle.as_ref().unwrap().discard_tokens,1);
    }
    #[test]
    fn two_policies_replay_to_identical_snapshots() {
        for hero in HEROES {
            for policy in [Policy::FirstPlayable,Policy::Tactical] {
                let first=simulate_policy(hero,123,3,policy).unwrap();
                let repeated=simulate_policy(hero,123,3,policy).unwrap();
                assert_eq!(first.snapshot,repeated.snapshot);
                assert_eq!(first.death_encounter,repeated.death_encounter);
            }
        }
    }
    #[test]
    fn sun_experimental_decks_have_equal_size_and_distinct_cards() {
        let original=Game::new_with_ascension(Hero::SunShangxiang,77,4);
        for package in SunPackage::ALL {
            let mut game=original.clone();
            apply_sun_package(&mut game,package).unwrap();
            assert_eq!(game.deck.len(),original.deck.len());
            assert_eq!(Game::from_json(&game.to_json().unwrap()).unwrap().deck.len(),
                original.deck.len());
            if package==SunPackage::Standard {
                assert!(game.deck.iter().zip(&original.deck).all(|(a,b)|a.id==b.id));
            } else {
                assert_ne!(game.deck[0].id,original.deck[0].id);
                assert_ne!(game.deck[5].id,original.deck[5].id);
            }
        }
        let mut outsider=Game::new(Hero::CaoCao,1);
        assert!(apply_sun_package(&mut outsider,SunPackage::Discard).is_err());
    }
    #[test]
    fn natural_draft_preserves_starter_deck_without_injected_packages() {
        let hero=Hero::SunShangxiang;
        let seed=8472;
        let a=simulate_config(hero,seed,0,Policy::Tactical,SunPackage::Standard,
            Some(Doctrine::SunDiscard)).unwrap();
        let b=simulate_config(hero,seed,0,Policy::Tactical,SunPackage::Standard,
            Some(Doctrine::SunDiscard)).unwrap();
        assert_eq!(a.snapshot,b.snapshot);
        assert_eq!(a.reward_screens,b.reward_screens);
        assert_eq!(a.focus_offer_screens,b.focus_offer_screens);
        assert!(a.focus_offer_screens<=a.reward_screens);
        assert!(a.focus_picks<=a.focus_offer_screens);
        assert!(a.discard_distinct<=Doctrine::SunDiscard.members().len());
    }
    #[test]
    fn run_counters_survive_replay_and_sum_without_double_counting() {
        let first=simulate_policy(Hero::CaoCao,8793,2,Policy::Tactical).unwrap();
        let replay=simulate_policy(Hero::CaoCao,8793,2,Policy::Tactical).unwrap();
        assert_eq!(first.telemetry,replay.telemetry);
        let mut total=CombatCounters::default();
        total.accumulate(&first.telemetry);
        total.accumulate(&replay.telemetry);
        assert_eq!(total.troop_damage,first.telemetry.troop_damage*2);
        assert_eq!(total.troop_absorbed,first.telemetry.troop_absorbed*2);
        assert!(total.command_energy>=first.telemetry.command_energy);
    }
    #[test]
    fn encounter_aggregation_is_deterministic() {
        let mut deaths=BTreeMap::new();
        deaths.insert("boss".to_string(),3);
        deaths.insert("bandit".to_string(),2);
        deaths.insert("archer".to_string(),3);
        let sorted=ranked_encounters(deaths);
        assert_eq!(sorted[0].encounter,"archer");
        assert_eq!(sorted[1].encounter,"boss");
        assert_eq!(sorted[2].deaths,2);
    }
    #[test]
    fn seeded_runs_replay_to_identical_snapshots() {
        for hero in HEROES {
            for seed in 1..=4 {
                let first = simulate(hero,seed,0).unwrap();
                let second = simulate(hero,seed,0).unwrap();
                assert_eq!(first.hero,second.hero);
                assert_eq!(first.seed,second.seed);
                assert_eq!(first.snapshot,second.snapshot);
            }
        }
    }
}
