//! Deterministic headless simulation for balancing and softlock discovery.
//! Run: cargo run --release --bin tks-sim -- 250
use std::collections::BTreeMap;
use std::fs;
use serde::Serialize;
use three_kingdoms_spire_reborn::{
    Action, Card, Game, Intent, Node, Stage,
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
    death_encounter: Option<String>,
    death_node: Option<Node>,
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
    let available=game.battle.as_ref().map_or(0,|b|b.energy);
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
            (novelty+if is_power{14}else{0}
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
            let threat=next_enemy_damage(game);
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
fn simulate_policy(hero:Hero,seed:u64,ascension:u8,policy:Policy)->Result<Outcome,String> {
    let mut game = Game::new_with_ascension(hero, seed, ascension);
    let mut death_context:Option<(String,Node)>=None;
    for count in 0..MAX_ACTIONS {
        if matches!(game.stage,Stage::Won | Stage::Lost) {
            return Ok(Outcome {
                hero, seed, floor: game.floor, won: game.stage == Stage::Won,
                actions: count, snapshot: game.to_json()?,
                death_encounter:if game.stage==Stage::Won {None}
                    else {death_context.as_ref().map(|(id,_)|id.clone())},
                death_node:if game.stage==Stage::Won {None}
                    else {death_context.as_ref().map(|(_,node)|*node)},
            });
        }
        let action = preferred_action_policy(&game,policy)?;
        if game.stage==Stage::Combat {
            if let Some(b)=&game.battle {
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
    Err(format!("suspected softlock: {} seed={} exceeded {} actions",
        hero.name(),seed,MAX_ACTIONS))
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
    println!("TOP BASELINE DEATH ENCOUNTERS: {:?}",
        report.baseline_death_encounters.iter().take(8)
            .map(|e|format!("{}:{}",e.encounter,e.deaths)).collect::<Vec<_>>());
    println!("TOP TACTICAL DEATH ENCOUNTERS: {:?}",
        report.tactical_death_encounters.iter().take(8)
            .map(|e|format!("{}:{}",e.encounter,e.deaths)).collect::<Vec<_>>());
}
fn main() {
    let arguments=std::env::args().collect::<Vec<_>>();
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
