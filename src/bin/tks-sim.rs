//! Deterministic headless simulation for balancing and softlock discovery.
//! Run: cargo run --release --bin tks-sim -- 250
use three_kingdoms_spire_reborn::{
    Action, Game, Stage,
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
                    let cost = if card.enchantment.as_deref() == Some("迅捷") {
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

fn simulate(hero: Hero, seed: u64, ascension: u8) -> Result<Outcome, String> {
    let mut game = Game::new_with_ascension(hero, seed, ascension);
    for count in 0..MAX_ACTIONS {
        if matches!(game.stage,Stage::Won | Stage::Lost) {
            return Ok(Outcome {
                hero, seed, floor: game.floor, won: game.stage == Stage::Won,
                actions: count, snapshot: game.to_json()?,
            });
        }
        let action = preferred_action(&game)?;
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

fn main() {
    let count = std::env::args().nth(1)
        .and_then(|raw| raw.parse::<usize>().ok()).unwrap_or(100).min(20_000);
    let mut victories = 0_u64;
    let mut deepest = 0_u8;
    let mut total_actions = 0_u64;
    for n in 0..count {
        let hero = HEROES[n % HEROES.len()];
        let seed = (n as u64 + 1).wrapping_mul(0x9E3779B97F4A7C15);
        let difficulty = ((n/5) % 11) as u8;
        match simulate(hero,seed,difficulty) {
            Ok(outcome) => {
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
    println!("Simulation OK: {} runs, {} wins, maximum floor {}, {} actions",
        count,victories,deepest,total_actions);
}

#[cfg(test)]
mod tests {
    use super::*;
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
