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
    combat::EventKind,
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
    /// Every finished combat encounter, including mixed enemy teams.
    /// These are battle-level outcomes, not run-level exposure flags.
    encounter_outcomes: Vec<EncounterOutcome>,
    reward_screens: u32,
    focus_offer_screens: u32,
    focus_picks: u32,
    discard_distinct: usize,
    combo_distinct: usize,
    natural_boss_evidence: Vec<NaturalBossEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct EncounterOutcome {
    foes: Vec<String>,
    survived: bool,
    /// Actual cumulative HP damage during this battle, including damage healed later.
    hero_hp_damage: u64,
    turns_completed: u64,
}

struct ActiveEncounter {
    foes: BTreeSet<String>,
    event_start: usize,
    turns_completed_before: u64,
    natural_boss_start: Option<NaturalBossStart>,
}

#[derive(Debug, Default, Serialize)]
struct CounterEncounterMetrics {
    encounters: usize,
    survived: usize,
    hero_hp_damage: u64,
    turns_completed: u64,
}
fn count_counter_encounters(outcomes: &[EncounterOutcome], foe_id: &str) -> CounterEncounterMetrics {
    let mut totals = CounterEncounterMetrics::default();
    for encounter in outcomes {
        if !encounter.foes.iter().any(|id| id == foe_id) { continue; }
        totals.encounters += 1;
        totals.survived += usize::from(encounter.survived);
        totals.hero_hp_damage = totals.hero_hp_damage.saturating_add(encounter.hero_hp_damage);
        totals.turns_completed = totals.turns_completed.saturating_add(encounter.turns_completed);
    }
    totals
}
impl CounterEncounterMetrics {
    fn add(&mut self, other: Self) {
        self.encounters += other.encounters;
        self.survived += other.survived;
        self.hero_hp_damage = self.hero_hp_damage.saturating_add(other.hero_hp_damage);
        self.turns_completed = self.turns_completed.saturating_add(other.turns_completed);
    }
}

#[derive(Clone, Copy, Debug)]
enum Policy { FirstPlayable, Tactical }


fn is_unplayable_blood_price(
    game: &Game,
    spec: &three_kingdoms_spire_reborn::content::CardSpec,
) -> bool {
    game.has_expanded_history_rules()
        && matches!(spec.effect, Effect::BloodPrice(loss, _) if game.hp <= loss)
}

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
                    !matches!(spec.effect,Effect::Unplayable)
                        && !is_unplayable_blood_price(game, &spec)
                        && cost <= battle.energy
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
fn combat_defense_need(game: &Game) -> i32 {
    let mut need = next_enemy_damage(game);
    if let Some(battle) = game.battle.as_ref() {
        for f in battle.foes.iter().filter(|f| f.hp > 0) {
            if let Intent::Boss(ref move_id) = f.intent {
                use three_kingdoms_spire_reborn::BossMove;
                let threshold = match move_id {
                    BossMove::FireCamp(stacks) if *stacks > 0 => 12,
                    BossMove::WhiteClothes => 8,
                    BossMove::NorthernExpedition(n) => 8 + 2 * i32::from(*n),
                    BossMove::StarOmen(phase) => 9 + 2 * i32::from(*phase),
                    BossMove::YinpingFlank(phase) => 10 + 2 * i32::from(*phase),
                    _ => 0,
                };
                need = need.max(threshold);
            }
        }
        if let (Some(hp), Some(loss)) = (battle.convoy_hp, game.projected_cargo_loss())
            && loss > 0
        {
            need += if loss >= hp { 18 } else { 8 };
        }
    }
    need
}
fn tactical_card_value(game:&Game,card:&Card,threat:i32)->i32 {
    let spec=match card_spec(&card.id){Some(s)=>s,None=>return -999};
    if game.battle.is_some() && is_unplayable_blood_price(game, &spec) {
        return -999;
    }
    // Outside combat, estimate reward value against a standard 3-energy turn.
    let available=game.battle.as_ref().map_or(3,|b|b.energy);
    let cost=card_energy(card,available).unwrap_or(999);
    if cost>available {return -999;}
    let convoy_risk = game.projected_cargo_loss().unwrap_or(0);
    let upfront=match spec.effect {
        Effect::Hit(n) | Effect::All(n) | Effect::Leech(n) | Effect::Pierce(n)
        | Effect::DiscardStrike(n) => n*2,
        Effect::HitTwice(n) => n*4,
        Effect::HitDraw(n,draw) => n*2 + draw as i32*9,
        Effect::HitWeak(n,_) | Effect::HitVulnerable(n,_) => n*2+9,
        Effect::HitPoison(n,poison) => n*2+poison*2,
        Effect::Guard(n) => (n*2).min((threat+5)*2) + if convoy_risk > 0 { 16 } else { 0 },
        Effect::GuardEnergy(n) => (n*2).min((threat+5)*2)+10 + if convoy_risk > 0 { 16 } else { 0 },
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
        Effect::Companion(n) => n*3 + if convoy_risk > 0 { 12 } else { 0 },
        Effect::Tactics(n) => n*8,
        Effect::Power(_,_) => 38,
        Effect::XHit(n) => n*cost*2,
        Effect::XGuard(n) => (n*cost*2).min((threat+5)*2) + if convoy_risk > 0 { 14 } else { 0 },
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
    upfront + if attack {5 + if convoy_risk > 0 { 8 } else { 0 }} else {0}
        + i32::from(card.upgraded)*5
        - cost*3
        - i32::from(matches!(spec.effect,Effect::XHit(_) | Effect::XGuard(_))
            && cost==0)*20
}
fn tactical_reward(game:&Game)->usize {
    let best = game.rewards.iter().enumerate()
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
        });
    let Some((i, card)) = best else { return 0; };
    if card_spec(&card.id).is_some_and(|s| matches!(s.effect, Effect::Unplayable)) {
        return 0;
    }
    let is_power = card_spec(&card.id).is_some_and(|s| matches!(s.effect, Effect::Power(_, _)));
    let focus_synergy = game.doctrine.is_some_and(|focus| focus.contains(&card.id));
    let score = tactical_card_value(game, card, 14);
    if !is_power && !focus_synergy && !card.upgraded
        && ((game.deck.len() >= 24 && score < 28) || (game.deck.len() >= 20 && score < 21))
    {
        return 0;
    }
    i + 1
}
fn tactical_action(game:&Game)->Result<Action,String> {
    match game.stage {
        Stage::Map => {
            let council_pending = game.uncommitted_council_hint().is_some();
            let story_priority = game.has_expanded_history_rules() && game.lore_seen.len() < 15;
            let mut best=(i32::MIN,1);
            for (i,node) in game.route.iter().enumerate() {
                let deficit=(game.max_hp-game.hp).max(0);
                let urgent_camp = story_priority && game.hp * 2 < game.max_hp;
                let score=match node {
                    Node::Camp => if deficit>=15 {
                        if council_pending && !urgent_camp && game.hp * 3 > game.max_hp {44}
                        else if story_priority && !urgent_camp {38}
                        else {52}
                    }else{22},
                    Node::Market => if (game.shop_relic.is_some() && game.gold>=155)
                        || (game.deck.iter().any(|c| c.id == "cursed") && game.gold>=75)
                    {
                        if story_priority {45} else {36}
                    } else if game.deck.len() >= 12 && game.gold >= 165 {
                        36
                    } else if game.gold >= 110 {25} else {5},
                    Node::Story => if council_pending {
                        48
                    } else if story_priority {
                        42
                    } else {
                        26
                    },
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
            let can_upgrade=game.deck.iter().any(|card|!card.upgraded&&card.id!="cursed"
                && card_spec(&card.id).is_some_and(|s| !matches!(s.effect, Effect::Unplayable)));
            Ok(Action::Choose {index:if deficit >= 12 {1}
                else if can_upgrade {2} else {3}})
        }
        Stage::Market => {
            let can_remove_cursed = game.deck.iter().any(|c| c.id == "cursed");
            if can_remove_cursed && game.gold >= 75 {
                return Ok(Action::Choose { index: 6 });
            }
            if game.shop_relic.is_some() && game.gold >= 155 {
                return Ok(Action::Choose { index: 5 });
            }
            if !game.market_upgraded && game.market_upgrade_price().is_some_and(|price| game.gold >= price)
                && game.gold >= 160
                && game.deck.iter().any(|c| !c.upgraded
                    && !matches!(c.id.as_str(), "cursed" | "quest_tulu")
                    && card_spec(&c.id).is_some_and(|s| !matches!(s.effect, Effect::Unplayable)))
                && game.deck.iter().filter(|c| c.upgraded).count() < 12
            {
                return Ok(Action::Choose { index: 7 });
            }
            if game.deck.len() < 23
                && let Some((idx, _)) = game.market.iter().enumerate()
                    .filter(|(idx, card)| {
                        let Some(cost) = Game::market_card_price(*idx + 1) else { return false; };
                        cost <= game.gold
                            && ((game.gold - cost) >= 75 || game.deck.len() < 16)
                            && card_spec(&card.id).is_some_and(|s| matches!(s.effect, Effect::Power(_, _))
                                || game.doctrine.is_some_and(|focus| focus.contains(&card.id))
                                || tactical_card_value(game, card, 14) >= 28)
                    })
                    .max_by_key(|(idx, card)| (tactical_card_value(game, card, 14), -(*idx as i32)))
            {
                return Ok(Action::Choose { index: idx + 1 });
            }
            if game.deck.len() >= 12 && game.gold >= 160
                && game.deck.iter().any(|c| c.id == "strike"
                    || (game.difficulty.is_some() && c.id == "defend"))
                && game.deck.iter().filter(|c| matches!(c.id.as_str(), "strike" | "defend")).count() >= 4
            {
                return Ok(Action::Choose { index: 6 });
            }
            if game.potions < 2 && game.gold >= 170 {
                return Ok(Action::Choose { index: 4 });
            }
            Ok(Action::Choose { index: 0 })
        }
        Stage::Story => {
            if game.in_council() {
                let is_late = game.xiapi_plan == Some(three_kingdoms_spire_reborn::XiapiPlan::Council)
                    || game.xiangfan_plan == Some(three_kingdoms_spire_reborn::XiangfanPlan::Council);
                let (gold_cost, hp_cost) = if is_late { (16, 6) } else { (20, 8) };
                return Ok(Action::Choose {
                    index: if game.gold >= gold_cost && game.hp * 2 >= game.max_hp { 1 }
                        else if game.hp > hp_cost + 12 { 2 }
                        else { 3 },
                });
            }
            if game.can_open_expanded_council() {
                return Ok(Action::Choose {
                    index: if game.story_fifth_option().is_some() { 5 } else { 4 },
                });
            }
            if game.lore_current.is_some() {
                return Ok(Action::Choose { index: 1 });
            }
            if game.story_fourth_option().is_some() {
                return Ok(Action::Choose { index: 4 });
            }
            let upgradeable = game.deck.iter().any(|c| !c.upgraded && c.id != "cursed"
                && card_spec(&c.id).is_some_and(|s| !matches!(s.effect, Effect::Unplayable)));
            let choice = match game.event_id {
                0 => if game.gold >= 15 && game.hp < game.max_hp { 1 } else if game.hp > 18 { 3 } else { 2 },
                1 => if game.hp + 12 <= game.max_hp { 2 } else if game.hp > game.max_hp / 2 { 1 } else { 3 },
                2 => if game.gold >= 25 && game.hp < game.max_hp { 1 } else { 3 },
                3 => if game.gold >= 30 && game.potions < 3 && game.hp * 2 < game.max_hp { 1 }
                    else if game.deck.len() < 18 { 3 } else { 2 },
                4 | 9 => if game.gold >= 40 && upgradeable { 1 }
                    else if game.event_id == 9 && game.deck.len() < 18 { 2 } else { 3 },
                5 => if game.hp * 2 > game.max_hp + 20 { 1 } else { 3 },
                6 => if game.hp + 10 <= game.max_hp { 1 } else { 3 },
                7 => if upgradeable { 1 } else { 3 },
                8 => if game.gold >= 20 && game.hp < game.max_hp { 1 }
                    else if game.potions < 3 { 3 } else if game.hp > 18 { 2 } else { 3 },
                10 => if game.hp + 10 <= game.max_hp { 1 } else { 2 },
                _ => if game.hp + 10 <= game.max_hp { 3 } else { 2 },
            };
            Ok(Action::Choose { index: choice })
        }
        Stage::Combat => {
            let b=game.battle.as_ref().ok_or("missing tactical battle")?;
            // Build-focused draw/energy cycles can yield unbounded optional plays.
            // Human players can press "end"; the bot must not interpret such
            // positive-energy loops as a hard game-engine softlock.
            if b.played_this_turn >= 36 {
                return Ok(Action::EndTurn);
            }
            let threat=combat_defense_need(game);
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
                .min_by_key(|(_,f)| {
                    let priority = if matches!(f.id.as_str(), "officer" | "reinforcement" | "banner") { -24 } else { 0 };
                    (priority + f.hp + f.block, f.hp)
                })
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
    let mut encounter_outcomes = Vec::<EncounterOutcome>::new();
    let mut natural_boss_evidence = Vec::<NaturalBossEvidence>::new();
    let mut active_encounter: Option<ActiveEncounter> = None;
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
                encounter_outcomes,
                natural_boss_evidence,
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
                let (offered, picked) = pick_focus_reward(&game, plan);
                if offered {
                    focus_offer_screens+=1;
                }
                if let Some(index) = picked {
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
                let tracked = active_encounter.get_or_insert_with(|| ActiveEncounter {
                    foes: BTreeSet::new(),
                    event_start: game.event_history.as_ref().map_or(0, Vec::len),
                    turns_completed_before: game.telemetry.totals.turns_completed,
                    natural_boss_start: if game.last_node == Some(Node::Boss) {
                        natural_plan.map(|plan| NaturalBossStart::capture(&game, plan))
                    } else { None },
                });
                for foe in &b.foes {
                    encountered_foes.insert(foe.id.clone());
                    tracked.foes.insert(foe.id.clone());
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
        // The battle terminates on this action: settle one denominator entry,
        // once per encounter (not per turn and not per foe in a mixed team).
        if game.stage != Stage::Combat
            && let Some(tracked) = active_encounter.take()
        {
            // Use the complete deterministic event archive: CombatCounters'
            // hero_hp_damage only includes enemy attacks, while blood_price
            // and supply_exhaustion can also cause real player HP loss.
            let history = game.event_history.as_ref()
                .ok_or("simulator requires a complete event archive")?;
            let events = history.get(tracked.event_start..)
                .ok_or("encounter event cursor exceeds archive")?;
            let hero_hp_damage = events.iter()
                .filter(|entry| entry.event.kind == EventKind::DamageReceived
                    && entry.event.target.as_deref() == Some("hero"))
                .map(|entry| entry.event.amount.max(0) as u64)
                .fold(0u64, u64::saturating_add);
            if let Some(start) = tracked.natural_boss_start {
                natural_boss_evidence.push(start.finish(game.stage != Stage::Lost));
            }
            encounter_outcomes.push(EncounterOutcome {
                foes: tracked.foes.into_iter().collect(),
                survived: game.stage != Stage::Lost,
                hero_hp_damage,
                turns_completed: game.telemetry.totals.turns_completed
                    .saturating_sub(tracked.turns_completed_before),
            });
        }
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
    /// Same-seed discordant outcomes, needed for an exact paired comparison.
    paired_baseline_only_wins:usize,
    paired_blueprint_only_wins:usize,
    /// True battle-level denominators for nominated counter foes, separately
    /// from the per-run flags above. In mixed enemy teams causality is shared.
    baseline_counter_battles: CounterEncounterMetrics,
    blueprint_counter_battles: CounterEncounterMetrics,
}
fn discordant_wins(baseline:bool, blueprint:bool)->(usize,usize) {
    (usize::from(baseline && !blueprint),usize::from(!baseline && blueprint))
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
                paired_baseline_only_wins:0,paired_blueprint_only_wins:0,
                baseline_counter_battles:CounterEncounterMetrics::default(),
                blueprint_counter_battles:CounterEncounterMetrics::default(),
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
                let (baseline_only,blueprint_only)=discordant_wins(baseline.won,curated.won);
                cell.paired_baseline_only_wins+=baseline_only;
                cell.paired_blueprint_only_wins+=blueprint_only;
                cell.baseline_floor_sum+=u64::from(baseline.floor);
                cell.blueprint_floor_sum+=u64::from(curated.floor);
                // Denominator: unique runs that really saw the named enemy;
                // never infer encounters from terminal deaths or a blueprint ID.
                cell.baseline_counter_exposed_runs+=usize::from(
                    baseline.encountered_foes.iter().any(|id|id==plan.counter_enemy));
                cell.blueprint_counter_exposed_runs+=usize::from(
                    curated.encountered_foes.iter().any(|id|id==plan.counter_enemy));
                cell.baseline_counter_battles.add(count_counter_encounters(
                    &baseline.encounter_outcomes, plan.counter_enemy));
                cell.blueprint_counter_battles.add(count_counter_encounters(
                    &curated.encounter_outcomes, plan.counter_enemy));
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
        interpretation:"equal 10-card curated fixture, four strikes and two guards replaced; paired fixed seeds, tactical heuristic bot, not natural draft nor human win rate; counter exposure counts distinct runs with a foe present at combat decision time; counter_battles count finished encounters with that foe and report battle survival, total actual HP damage and turns, including mixed enemy teams; neither is a controlled isolated boss counterfactual".into(),
        cells,
    })
}


#[path = "tks-sim/natural_plan_report.rs"]
mod natural_plan_report;
use natural_plan_report::{NaturalBossEvidence, NaturalBossStart};


// Fixed paired cohort for the expanded campaign. Every cell shares the same
// seed set and uses the normal starter deck. This is an observational report,
// not a claim of human balance or an assertion that every cell wins.
#[derive(Debug, Serialize)]
struct ExpandedFloorSample {
    floor: u8,
    stage: Stage,
    node: Option<Node>,
    // Foes present at this observation, not a full encounter denominator.
    foes: Vec<String>,
    hp: i32,
    max_hp: i32,
    gold: i32,
    cards: usize,
    relics: usize,
    potions: usize,
    turns_completed: u64,
}

impl ExpandedFloorSample {
    fn capture(game: &Game) -> Self {
        Self {
            floor: game.floor, stage: game.stage, node: game.last_node,
            foes: game.battle.as_ref().map(|battle| battle.foes.iter().map(|foe| foe.id.clone()).collect()).unwrap_or_default(),
            hp: game.hp, max_hp: game.max_hp,
            gold: game.gold, cards: game.deck.len(), relics: game.relics.len(),
            potions: game.potion_bag.len(),
            turns_completed: game.telemetry.totals.turns_completed,
        }
    }
}

#[derive(Debug, Default, Serialize)]
struct ExpandedHistoricalDensity {
    covered_lower_bound: usize,
    resource_or_deck: usize,
    map_effect: usize,
    battle_effect: usize,
    cross_act_effect: usize,
    untriggered_count: usize,
    untriggered_categories: BTreeMap<String, usize>,
}

impl ExpandedHistoricalDensity {
    fn from_report(report: &three_kingdoms_spire_reborn::campaign_authority::HistoricalDensityReport) -> Self {
        let mut untriggered_categories = BTreeMap::new();
        for reason in &report.untriggered_reasons {
            *untriggered_categories.entry(reason.category.to_string()).or_insert(0) += 1;
        }
        Self {
            covered_lower_bound: report.covered_lower_bound,
            resource_or_deck: report.resource_or_deck,
            map_effect: report.map_effect,
            battle_effect: report.battle_effect,
            cross_act_effect: report.cross_act_effect,
            untriggered_count: report.untriggered_reasons.len(),
            untriggered_categories,
        }
    }
}

#[derive(Debug, Serialize)]
struct ExpandedMatrixRun {
    hero: Hero,
    difficulty: three_kingdoms_spire_reborn::Difficulty,
    seed: u64,
    policy: &'static str,
    terminal: Stage,
    actions: u32,
    error: Option<String>,
    replay_verified: bool,
    chronicle_verified: bool,
    bosses_entered: Vec<(u8, String)>,
    floors: Vec<ExpandedFloorSample>,
    final_sample: ExpandedFloorSample,
    telemetry: CombatCounters,
    historical_density: ExpandedHistoricalDensity,
}

impl ExpandedMatrixRun {
    fn verified_terminal(&self) -> bool {
        self.error.is_none() && self.replay_verified && self.chronicle_verified
            && matches!(self.terminal, Stage::Won | Stage::Lost)
    }
}

#[derive(Debug, Default, Serialize)]
struct ExpandedActEconomy {
    reached: usize,
    hp_sum: i64,
    max_hp_sum: i64,
    gold_sum: i64,
    cards_sum: usize,
    relics_sum: usize,
    potions_sum: usize,
    turns_sum: u64,
}

#[derive(Debug, Default, Serialize)]
struct ExpandedPolicyTotals {
    observed: usize,
    wins: usize,
    losses: usize,
    incomplete: usize,
    // Only verified terminal runs contribute to these sums.
    terminal_floor_sum: u64,
    terminal_max_hp_sum: i64,
    terminal_gold_sum: i64,
    terminal_deck_size_sum: usize,
    terminal_relic_count_sum: usize,
    terminal_potions_sum: usize,
    terminal_turns_sum: u64,
    battles_won_sum: u64,
    battles_lost_sum: u64,
    density_covered_sum: usize,
    density_covered_min: usize,
    density_covered_max: usize,
    victory_density_sum: usize,
    victory_density_min: Option<usize>,
    victory_density_max: Option<usize>,
    density_battle_sum: usize,
    density_cross_act_sum: usize,
    per_act_economy: [ExpandedActEconomy; 5],
}

impl ExpandedPolicyTotals {
    fn record(&mut self, run: &ExpandedMatrixRun) {
        self.observed += 1;
        if !run.verified_terminal() {
            self.incomplete += 1;
            return;
        }
        let settled = self.wins + self.losses;
        if run.terminal == Stage::Won {
            self.wins += 1;
            let cov = run.historical_density.covered_lower_bound;
            self.victory_density_sum += cov;
            self.victory_density_min = Some(self.victory_density_min.map_or(cov, |m| m.min(cov)));
            self.victory_density_max = Some(self.victory_density_max.map_or(cov, |m| m.max(cov)));
        } else {
            self.losses += 1;
        }
        self.terminal_floor_sum += u64::from(run.final_sample.floor);
        self.terminal_max_hp_sum += i64::from(run.final_sample.max_hp);
        self.terminal_gold_sum += i64::from(run.final_sample.gold);
        self.terminal_deck_size_sum += run.final_sample.cards;
        self.terminal_relic_count_sum += run.final_sample.relics;
        self.terminal_potions_sum += run.final_sample.potions;
        self.terminal_turns_sum += run.final_sample.turns_completed;
        self.battles_won_sum += run.telemetry.battles_won;
        self.battles_lost_sum += run.telemetry.battles_lost;
        let covered = run.historical_density.covered_lower_bound;
        self.density_covered_sum += covered;
        self.density_covered_min = if settled == 0 { covered } else { self.density_covered_min.min(covered) };
        self.density_covered_max = self.density_covered_max.max(covered);
        self.density_battle_sum += run.historical_density.battle_effect;
        self.density_cross_act_sum += run.historical_density.cross_act_effect;
        for (act_idx, bucket) in self.per_act_economy.iter_mut().enumerate() {
            let low = (act_idx as u8) * 8 + 1;
            let high = low + 7;
            if let Some(sample) = run.floors.iter().rev().find(|s| (low..=high).contains(&s.floor)) {
                bucket.reached += 1;
                bucket.hp_sum += i64::from(sample.hp);
                bucket.max_hp_sum += i64::from(sample.max_hp);
                bucket.gold_sum += i64::from(sample.gold);
                bucket.cards_sum += sample.cards;
                bucket.relics_sum += sample.relics;
                bucket.potions_sum += sample.potions;
                bucket.turns_sum += sample.turns_completed;
            }
        }
    }
}

#[derive(Debug, Default, Serialize)]
struct ExpandedPairTotals {
    first_playable: ExpandedPolicyTotals,
    tactical: ExpandedPolicyTotals,
    both_won: usize,
    first_only_won: usize,
    tactical_only_won: usize,
    both_lost: usize,
    incomplete_pairs: usize,
}

impl ExpandedPairTotals {
    fn record(&mut self, first: &ExpandedMatrixRun, tactical: &ExpandedMatrixRun) {
        self.first_playable.record(first);
        self.tactical.record(tactical);
        if !first.verified_terminal() || !tactical.verified_terminal() {
            self.incomplete_pairs += 1;
            return;
        }
        match (first.terminal == Stage::Won, tactical.terminal == Stage::Won) {
            (true, true) => self.both_won += 1,
            (true, false) => self.first_only_won += 1,
            (false, true) => self.tactical_only_won += 1,
            (false, false) => self.both_lost += 1,
        }
    }
}

#[derive(Debug, Serialize)]
struct ExpandedMatrixCell {
    hero: Hero,
    difficulty: three_kingdoms_spire_reborn::Difficulty,
    totals: ExpandedPairTotals,
}

fn expanded_matrix_action(game: &Game, policy: Policy) -> Result<Action, String> {
    // Siege defenders can all be dead while the gate is still standing.
    // Both policies must use a legal action rather than target a dead foe.
    // FirstPlayable still fights defenders first; Tactical clears/blocks severe
    // incoming hits before rushing the gate unless at Turn-4 deadline or breach.
    if game.stage == Stage::Combat
        && let Some(battle) = game.battle.as_ref()
        && battle.siege_gate_hp.is_some_and(|hp| hp > 0)
    {
        let defenders_gone = battle.foes.iter().all(|foe| foe.hp <= 0);
        if defenders_gone {
            return Ok(if battle.energy >= 2 { Action::AssaultGate } else { Action::EndTurn });
        }
        if matches!(policy, Policy::Tactical) && battle.energy >= 2
            && (battle.turn >= 4
                || battle.siege_gate_hp.is_some_and(|g| g <= 4)
                || battle.block + 4 >= combat_defense_need(game))
        {
            return Ok(Action::AssaultGate);
        }
    }
    preferred_action_policy(game, policy)
}

fn expanded_matrix_run(
    hero: Hero, difficulty: three_kingdoms_spire_reborn::Difficulty, seed: u64,
    policy: Policy,
) -> ExpandedMatrixRun {
    use three_kingdoms_spire_reborn::progression::Profile;
    let policy_id = match policy {
        Policy::FirstPlayable => "first_playable",
        Policy::Tactical => "tactical",
    };
    let run_id = format!("expanded-matrix-{hero:?}-{}-{seed}-{policy_id}", difficulty.id());
    let mut game = Game::new_with_campaign_difficulty(hero, seed, difficulty, run_id.clone());
    let mut floors = vec![ExpandedFloorSample::capture(&game)];
    let mut bosses_entered = Vec::new();
    let mut actions = 0;
    let mut error = None;
    while actions < MAX_ACTIONS && !matches!(game.stage, Stage::Won | Stage::Lost) {
        let before = game.floor;
        let action = match expanded_matrix_action(&game, policy) {
            Ok(action) => action,
            Err(message) => { error = Some(message); break; }
        };
        if let Err(message) = game.apply(action) {
            error = Some(format!("floor {} stage {:?}: {message}", game.floor, game.stage));
            break;
        }
        actions += 1;
        if game.floor != before {
            floors.push(ExpandedFloorSample::capture(&game));
            if game.last_node == Some(Node::Boss)
                && let Some(foe) = game.battle.as_ref().and_then(|b| b.foes.first())
            {
                bosses_entered.push((game.floor, foe.id.clone()));
            }
        }
    }
    if error.is_none() && !matches!(game.stage, Stage::Won | Stage::Lost) {
        error = Some("action budget exceeded; incomplete run, not a defeat".into());
    }
    let mut replay_verified = false;
    let mut chronicle_verified = false;
    let mut historical_density = ExpandedHistoricalDensity::default();
    if error.is_none() {
        let verify = (|| -> Result<(), String> {
            let snapshot = game.to_json()?;
            let restored = Game::from_json(&snapshot)?;
            if restored.to_json()? != snapshot {
                return Err("terminal JSON changed on restore".into());
            }
            game.audit_campaign_authority()?;
            restored.verify_action_replay()?;
            historical_density = ExpandedHistoricalDensity::from_report(&restored.historical_density_report()?);
            replay_verified = true;
            if game.stage == Stage::Won
                && (game.floor != 40
                    || bosses_entered.iter().map(|(floor, _)| *floor)
                        .collect::<Vec<_>>() != [8, 16, 24, 32, 40])
            {
                return Err("victory did not traverse all five real bosses".into());
            }
            let mut profile = Profile::new();
            if !profile.record(&game, run_id.clone())? {
                return Err("first settlement was not recorded".into());
            }
            let persisted = profile.to_json()?;
            let mut restored_profile = Profile::from_json(&persisted)?;
            if restored_profile.record(&restored, run_id)?
                || restored_profile.to_json()? != persisted
            {
                return Err("terminal settlement duplicated after restore".into());
            }
            chronicle_verified = true;
            Ok(())
        })();
        if let Err(message) = verify { error = Some(message); }
    }
    ExpandedMatrixRun {
        hero, difficulty, seed, policy: policy_id, terminal: game.stage, actions, error,
        replay_verified, chronicle_verified, bosses_entered, floors,
        final_sample: ExpandedFloorSample::capture(&game), telemetry: game.telemetry.totals,
        historical_density,
    }
}

// Both the executable report and the CI test use the same natural cohort.
fn collect_expanded_matrix(samples: usize) -> (Vec<ExpandedMatrixRun>, Vec<ExpandedMatrixCell>) {
    use three_kingdoms_spire_reborn::Difficulty;
    let mut runs = Vec::new();
    let mut cells = Vec::new();
    for hero in HEROES {
        for difficulty in Difficulty::ALL {
            let mut totals = ExpandedPairTotals::default();
            for trial in 0..samples {
                let seed = 247_510 + trial as u64;
                let first = expanded_matrix_run(hero, difficulty, seed, Policy::FirstPlayable);
                let tactical = expanded_matrix_run(hero, difficulty, seed, Policy::Tactical);
                totals.record(&first, &tactical);
                runs.extend([first, tactical]);
            }
            cells.push(ExpandedMatrixCell { hero, difficulty, totals });
        }
    }
    (runs, cells)
}

fn expanded_matrix_integrity(runs: &[ExpandedMatrixRun]) -> Result<(), String> {
    let failures: Vec<_> = runs.iter().filter(|run| !run.verified_terminal())
        .map(|run| format!("{:?} {:?} {} seed {} floor {}: {:?}",
            run.hero, run.difficulty, run.policy, run.seed, run.final_sample.floor, run.error))
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("incomplete natural matrix cells: {}", failures.join(" | ")))
    }
}

#[cfg(test)]
#[test]
fn expanded_matrix_fixed_seed_all_cells_settle_and_replay() {
    let (runs, cells) = collect_expanded_matrix(1);
    assert_eq!(runs.len(), 30);
    assert_eq!(cells.len(), 15);
    expanded_matrix_integrity(&runs).expect("all natural runs must settle and replay");
    let (pairs, remainder) = runs.as_chunks::<2>();
    assert!(remainder.is_empty());
    for (cell, pair) in cells.iter().zip(pairs) {
        assert_eq!(pair[0].seed, pair[1].seed);
        assert_eq!(pair[0].policy, "first_playable");
        assert_eq!(pair[1].policy, "tactical");
        let totals = &cell.totals;
        assert_eq!(totals.incomplete_pairs, 0);
        assert_eq!(totals.both_won + totals.first_only_won + totals.tactical_only_won + totals.both_lost, 1);
        assert_eq!(totals.first_playable.wins, usize::from(pair[0].terminal == Stage::Won));
        assert_eq!(totals.tactical.wins, usize::from(pair[1].terminal == Stage::Won));
        assert_eq!(totals.first_playable.density_covered_sum, pair[0].historical_density.covered_lower_bound);
        assert_eq!(totals.tactical.density_covered_sum, pair[1].historical_density.covered_lower_bound);
        for (act_idx, bucket) in totals.tactical.per_act_economy.iter().enumerate() {
            let low = (act_idx as u8) * 8 + 1;
            let high = low + 7;
            let reached = usize::from(pair[1].floors.iter().any(|s| (low..=high).contains(&s.floor)));
            assert_eq!(bucket.reached, reached);
        }
        assert!(totals.tactical.per_act_economy.windows(2).all(|w| w[0].reached >= w[1].reached));
        assert_eq!(totals.tactical.terminal_max_hp_sum, i64::from(pair[1].final_sample.max_hp));
        assert_eq!(totals.tactical.terminal_potions_sum, pair[1].final_sample.potions);
        assert_eq!(totals.tactical.battles_won_sum, pair[1].telemetry.battles_won);
        assert_eq!(totals.tactical.battles_lost_sum, pair[1].telemetry.battles_lost);
        for run in pair {
            assert!(run.floors.windows(2).all(|samples| samples[0].floor < samples[1].floor));
            assert!(run.historical_density.covered_lower_bound >= run.historical_density.battle_effect);
            assert!(run.historical_density.cross_act_effect <= run.historical_density.battle_effect);
        }
    }
    let tactical_with_density = runs.iter()
        .filter(|r| r.policy == "tactical" && r.historical_density.covered_lower_bound > 0)
        .map(|r| r.hero.name())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(tactical_with_density.len(), HEROES.len(), "all 5 heroes must record historical density");
}

fn pick_focus_reward(game: &Game, plan: &BuildPlan) -> (bool, Option<usize>) {
    let wanted = |id: &str| {
        !game.deck.iter().any(|card| card.id == id)
            && (plan.core.contains(&id) || plan.support.contains(&id))
    };
    let offered = game.rewards.iter().any(|card| wanted(&card.id));
    let picked = game.rewards.iter().enumerate()
        .filter(|(_, card)| wanted(&card.id))
        .max_by_key(|(index, card)| (
            i32::from(plan.core.contains(&card.id.as_str())),
            tactical_card_value(game, card, 14),
            -(*index as i32),
        ))
        .map(|(index, _)| index);
    (offered, picked)
}

#[derive(Debug, Serialize)]
struct ExpandedBuildCell {
    id: &'static str,
    hero: Hero,
    difficulty: three_kingdoms_spire_reborn::Difficulty,
    samples: usize,
    standard_wins: usize,
    focused_wins: usize,
    standard_floor_sum: u64,
    focused_floor_sum: u64,
    core_distinct_sum: usize,
    support_distinct_sum: usize,
    complete_core: usize,
    complete_support: usize,
    boss_battles: usize,
    boss_wins: usize,
    boss_core_ready: usize,
    counter_boss_battles: usize,
    counter_boss_wins: usize,
    replay_verified_runs: usize,
    reward_screens: u64,
    focus_offer_screens: u64,
    focus_picks: u64,
    max_deck_cards: usize,
    max_actions: u32,
}

struct ExpandedBuildSimOutcome {
    game: Game,
    actions: u32,
    reward_screens: u64,
    focus_offer_screens: u64,
    focus_picks: u64,
    boss_battles: usize,
    boss_wins: usize,
    boss_core_ready: usize,
    counter_boss_battles: usize,
    counter_boss_wins: usize,
}

fn simulate_expanded_with_focus(
    hero: Hero,
    difficulty: three_kingdoms_spire_reborn::Difficulty,
    seed: u64,
    plan: Option<&BuildPlan>,
) -> Result<ExpandedBuildSimOutcome, String> {
    let tag = plan.map_or("std", |p| p.id);
    let run_id = format!("exp-build-{tag}-{}-{seed}", difficulty.id());
    let mut game = Game::new_with_campaign_difficulty(hero, seed, difficulty, run_id);
    let mut actions = 0_u32;
    let mut reward_screens = 0_u64;
    let mut focus_offer_screens = 0_u64;
    let mut focus_picks = 0_u64;
    let mut boss_battles = 0_usize;
    let mut boss_wins = 0_usize;
    let mut boss_core_ready = 0_usize;
    let mut counter_boss_battles = 0_usize;
    let mut counter_boss_wins = 0_usize;
    let mut in_boss_counter: Option<bool> = None;

    while actions < MAX_ACTIONS && !matches!(game.stage, Stage::Won | Stage::Lost) {
        let was_combat = game.stage == Stage::Combat;
        let mut action = expanded_matrix_action(&game, Policy::Tactical)?;
        if game.stage == Stage::Reward {
            reward_screens += 1;
            if let Some(p) = plan {
                let (offered, picked) = pick_focus_reward(&game, p);
                if offered {
                    focus_offer_screens += 1;
                }
                if let Some(idx) = picked {
                    focus_picks += 1;
                    action = Action::Choose { index: idx + 1 };
                }
            }
        }
        game.apply(action)?;
        actions += 1;
        if !was_combat && game.stage == Stage::Combat && game.last_node == Some(Node::Boss) {
            boss_battles += 1;
            let ready = plan.is_some_and(|p| p.acquired(&game.deck) == p.core.len());
            boss_core_ready += usize::from(ready);
            let is_counter = plan.is_some_and(|p|
                game.battle.as_ref().is_some_and(|b| b.foes.iter().any(|f| f.id == p.counter_enemy)));
            counter_boss_battles += usize::from(is_counter);
            in_boss_counter = Some(is_counter);
        }
        if was_combat && game.stage != Stage::Combat && let Some(was_counter) = in_boss_counter.take()
            && game.stage != Stage::Lost
        {
            boss_wins += 1;
            counter_boss_wins += usize::from(was_counter);
        }
    }
    if !matches!(game.stage, Stage::Won | Stage::Lost) {
        return Err(format!("expanded build sim exceeded budget on {:?} {}", hero, difficulty.id()));
    }
    game.audit_campaign_authority()?;
    game.verify_action_replay()?;
    Ok(ExpandedBuildSimOutcome {
        game,
        actions,
        reward_screens,
        focus_offer_screens,
        focus_picks,
        boss_battles,
        boss_wins,
        boss_core_ready,
        counter_boss_battles,
        counter_boss_wins,
    })
}

fn collect_expanded_builds(samples: usize) -> Result<Vec<ExpandedBuildCell>, String> {
    use three_kingdoms_spire_reborn::Difficulty;
    let mut cells = Vec::with_capacity(PLANS.len() * Difficulty::ALL.len());
    for (plan_idx, plan) in PLANS.iter().enumerate() {
        for difficulty in Difficulty::ALL {
            let mut cell = ExpandedBuildCell {
                id: plan.id,
                hero: plan.hero,
                difficulty,
                samples,
                standard_wins: 0,
                focused_wins: 0,
                standard_floor_sum: 0,
                focused_floor_sum: 0,
                core_distinct_sum: 0,
                support_distinct_sum: 0,
                complete_core: 0,
                complete_support: 0,
                boss_battles: 0,
                boss_wins: 0,
                boss_core_ready: 0,
                counter_boss_battles: 0,
                counter_boss_wins: 0,
                replay_verified_runs: 0,
                reward_screens: 0,
                focus_offer_screens: 0,
                focus_picks: 0,
                max_deck_cards: 0,
                max_actions: 0,
            };
            for trial in 0..samples {
                let seed = 700_000_u64 + (plan_idx as u64) * 1000 + trial as u64;
                let std_out = simulate_expanded_with_focus(plan.hero, difficulty, seed, None)?;
                let foc_out = simulate_expanded_with_focus(plan.hero, difficulty, seed, Some(plan))?;
                let core_got = plan.acquired(&foc_out.game.deck);
                let support_got = plan.support.iter()
                    .filter(|id| foc_out.game.deck.iter().any(|c| c.id == **id))
                    .count();
                cell.standard_wins += usize::from(std_out.game.stage == Stage::Won);
                cell.focused_wins += usize::from(foc_out.game.stage == Stage::Won);
                cell.standard_floor_sum += u64::from(std_out.game.floor);
                cell.focused_floor_sum += u64::from(foc_out.game.floor);
                cell.core_distinct_sum += core_got;
                cell.support_distinct_sum += support_got;
                cell.complete_core += usize::from(core_got == plan.core.len());
                cell.complete_support += usize::from(support_got == plan.support.len());
                cell.boss_battles += foc_out.boss_battles;
                cell.boss_wins += foc_out.boss_wins;
                cell.boss_core_ready += foc_out.boss_core_ready;
                cell.counter_boss_battles += foc_out.counter_boss_battles;
                cell.counter_boss_wins += foc_out.counter_boss_wins;
                cell.replay_verified_runs += 1;
                cell.reward_screens += foc_out.reward_screens;
                cell.focus_offer_screens += foc_out.focus_offer_screens;
                cell.focus_picks += foc_out.focus_picks;
                cell.max_deck_cards = cell.max_deck_cards
                    .max(std_out.game.deck.len())
                    .max(foc_out.game.deck.len());
                cell.max_actions = cell.max_actions.max(std_out.actions).max(foc_out.actions);
            }
            cells.push(cell);
        }
    }
    Ok(cells)
}

fn expanded_builds_report(samples: usize, output: Option<&str>) -> Result<(), String> {
    let cells = collect_expanded_builds(samples)?;
    let report = serde_json::to_string_pretty(&serde_json::json!({
        "schema_version": 1,
        "samples_per_cell": samples,
        "cells": cells,
    })).map_err(|e| e.to_string())?;
    if let Some(path) = output {
        fs::write(path, &report).map_err(|e| format!("write {path}: {e}"))?;
    } else {
        println!("{report}");
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn expanded_builds_and_per_act_economy_bounds() {
    use three_kingdoms_spire_reborn::Difficulty;
    for plan in PLANS {
        let out = simulate_expanded_with_focus(plan.hero, Difficulty::Warlords, 700_000, Some(plan)).unwrap();
        assert!(matches!(out.game.stage, Stage::Won | Stage::Lost));
        assert!(out.actions < MAX_ACTIONS && out.game.deck.len() <= 55);
        assert!(out.focus_picks <= out.focus_offer_screens && out.focus_offer_screens <= out.reward_screens);
    }
    for difficulty in Difficulty::ALL {
        let out = simulate_expanded_with_focus(Hero::GuanYu, difficulty, 700_000, Some(&PLANS[0])).unwrap();
        assert!(matches!(out.game.stage, Stage::Won | Stage::Lost));
    }
}

#[cfg(test)]
#[test]
fn expanded_campaign_supports_both_variants_across_all_ten_bosses_and_echo_density() {
    use three_kingdoms_spire_reborn::Difficulty;
    let mut seen_bosses = std::collections::BTreeSet::new();
    for (act, expected) in [
        (0u8, ["zhangjiao", "sunjian"]),
        (1u8, ["dongzhuo", "yuanshao"]),
        (2u8, ["simayi", "zuoci"]),
        (3u8, ["luxun", "lumeng"]),
        (4u8, ["jiangwei", "dengai"]),
    ] {
        for (branch, boss_id) in expected.iter().enumerate() {
            let mut g = Game::new_with_campaign_difficulty(
                Hero::CaoCao, 77, Difficulty::Warlords, format!("boss-variant-{act}-{branch}"),
            );
            let b = branch as u8;
            let [a1, a2, a3, a4, a5] = [b; 5];
            g.apply(Action::SetExpandedVariants { act1: a1, act2: a2, act3: a3, act4: a4, act5: a5 }).unwrap();
            g.floor = act * 8 + 7;
            g.stage = Stage::Map;
            g.map_lane = None;
            g.route = vec![Node::Boss];
            g.route_lanes = vec![0];
            g.apply(Action::Choose { index: 1 }).unwrap();
            let actual_id = g.battle.as_ref().and_then(|bt| bt.foes.first()).map(|f| f.id.clone()).unwrap();
            assert_eq!(actual_id, *boss_id);
            seen_bosses.insert(actual_id);
        }
    }
    assert_eq!(seen_bosses.len(), 10);
}

#[cfg(test)]
#[test]
fn tactical_policy_protects_convoy_skips_bloat_and_spends_market_gold() {
    let mut bloated = Game::new_with_campaign_difficulty(
        Hero::CaoCao, 7, three_kingdoms_spire_reborn::Difficulty::Warlords, "tactical-bloat".into(),
    );
    while bloated.deck.len() < 26 {
        bloated.deck.push(Card::new("strike"));
    }
    bloated.stage = Stage::Reward;
    bloated.rewards = vec![Card::new("strike"), Card::new("defend"), Card::new("slash")];
    assert_eq!(tactical_reward(&bloated), 0, "bloated starter deck should skip weak common cards");

    let mut market = Game::new_with_campaign_difficulty(
        Hero::CaoCao, 7, three_kingdoms_spire_reborn::Difficulty::Warlords, "tactical-market".into(),
    );
    market.stage = Stage::Market;
    market.gold = 260;
    market.shop_relic = None;
    let first = tactical_action(&market).unwrap();
    assert!(matches!(first, Action::Choose { index: 7 | 5 | 6 | 1..=3 }), "expected active Market purchase/upgrade/trim, got {first:?}");
}

fn expanded_matrix_report(samples: usize, output: Option<&str>) -> Result<(), String> {
    let (runs, cells) = collect_expanded_matrix(samples);
    let integrity = expanded_matrix_integrity(&runs);
    let report = serde_json::to_string_pretty(&serde_json::json!({
        "schema_version": 5,
        "samples_per_cell_per_policy": samples,
        "seed_formula": "247510 + trial",
        "normal_starter_deck": true,
        "integrity_verified": integrity.is_ok(),
        "cells": cells,
        "runs": runs,
    })).map_err(|e| e.to_string())?;
    // Preserve diagnostic rows on failure, but never return success to automation.
    if let Some(path) = output {
        fs::write(path, report).map_err(|e| format!("write {path}: {e}"))?;
    } else {
        println!("{report}");
    }
    integrity
}

fn main() {
    let arguments=std::env::args().collect::<Vec<_>>();
    if arguments.get(1).is_some_and(|arg| arg == "--expanded-matrix") {
        let samples = arguments.get(2).and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2).clamp(1, 16);
        expanded_matrix_report(samples, arguments.get(3).map(String::as_str))
            .unwrap_or_else(|e| panic!("expanded matrix report failed: {e}"));
        return;
    }
    if arguments.get(1).is_some_and(|arg| arg == "--expanded-builds") {
        let samples = arguments.get(2).and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1).clamp(1, 16);
        expanded_builds_report(samples, arguments.get(3).map(String::as_str))
            .unwrap_or_else(|e| panic!("expanded builds report failed: {e}"));
        return;
    }
    if arguments.get(1).is_some_and(|arg|arg=="--natural-builds") {
        let samples=arguments.get(2).and_then(|s|s.parse::<usize>().ok())
            .unwrap_or(1).clamp(1,20);
        natural_plan_report::run_natural_plan_report(
            samples, arguments.get(3).map(String::as_str)
        ).unwrap_or_else(|e|panic!("natural 15-plan study failed: {e}"));
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
    fn paired_win_discordance_counts_only_one_sided_victories() {
        assert_eq!(discordant_wins(false,false),(0,0));
        assert_eq!(discordant_wins(true,true),(0,0));
        assert_eq!(discordant_wins(true,false),(1,0));
        assert_eq!(discordant_wins(false,true),(0,1));
    }
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
    fn counter_battle_denominators_are_battles_not_turns_or_run_flags() {
        let encounters = vec![
            EncounterOutcome {
                foes: vec!["boss".into(), "minion".into()],
                survived: true, hero_hp_damage: 12, turns_completed: 4,
            },
            EncounterOutcome {
                foes: vec!["boss".into()], survived: false,
                hero_hp_damage: 17, turns_completed: 3,
            },
            EncounterOutcome {
                foes: vec!["minion".into()], survived: true,
                hero_hp_damage: 2, turns_completed: 1,
            },
        ];
        let boss = count_counter_encounters(&encounters, "boss");
        assert_eq!((boss.encounters, boss.survived, boss.hero_hp_damage,
            boss.turns_completed), (2, 1, 29, 7));
        let minion = count_counter_encounters(&encounters, "minion");
        assert_eq!((minion.encounters, minion.survived, minion.hero_hp_damage),
            (2, 2, 14));
        assert_eq!(count_counter_encounters(&encounters, "not_present").encounters, 0);
    }

    #[test]
    fn real_encounter_outcomes_are_repeatable_and_have_unique_foe_ids() {
        let first = simulate_policy(Hero::GuanYu, 8793, 2, Policy::Tactical).unwrap();
        let second = simulate_policy(Hero::GuanYu, 8793, 2, Policy::Tactical).unwrap();
        assert_eq!(first.encounter_outcomes, second.encounter_outcomes);
        assert!(!first.encounter_outcomes.is_empty());
        assert!(first.encounter_outcomes.iter().all(|sample| {
            !sample.foes.is_empty() && sample.foes.windows(2).all(|two| two[0] < two[1])
        }));
        assert_eq!(first.encounter_outcomes.len() as u64,
            first.telemetry.battles_won + first.telemetry.battles_lost);
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
