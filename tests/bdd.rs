//! Executable BDD feature scenarios for story, combat and save portability.
//! Intentionally supports a small strict Gherkin subset (Feature/Scenario/Given/When/Then/And).
//! Every .feature step must be recognized; unknown steps FAIL instead of being silently skipped.
//! No additional runtime/test dependency is required for CLI, macOS native, or WASM builds.
use three_kingdoms_spire_reborn::{Action, Game, Stage, content::Hero, lore};

#[derive(Default)]
struct World {
    game: Option<Game>,
    before: Option<String>,
    error: Option<String>,
}

fn quoted(step: &str) -> Result<&str, String> {
    step.split('"').nth(1).ok_or_else(|| format!("missing quoted argument: {step}"))
}
fn parse_after<'a>(step: &'a str, prefix: &str) -> Result<&'a str, String> {
    step.strip_prefix(prefix).ok_or_else(|| format!("unexpected step: {step}"))
}
impl World {
    fn game(&self) -> Result<&Game, String> {
        self.game.as_ref().ok_or_else(|| "missing initial Given campaign".into())
    }
    fn game_mut(&mut self) -> Result<&mut Game, String> {
        self.game.as_mut().ok_or_else(|| "missing initial Given campaign".into())
    }
    fn assert_case(&mut self, step: &str) -> Result<(), String> {
        if let Some(prefix) = step.strip_prefix("a \"") {
            let hero = prefix.split('"').next().ok_or("missing hero")?;
            let seed: u64 = step.split("with seed ").nth(1).ok_or("missing seed")?
                .parse().map_err(|_| "bad seed")?;
            let hero = Hero::parse(hero).ok_or("unknown hero")?;
            self.game = Some(Game::new(hero, seed));
            self.before = None;
            self.error = None;
        } else if step.starts_with("the lore scene \"") {
            let id = quoted(step)?;
            if lore::beat(id).is_none() { return Err(format!("unknown lore beat: {id}")); }
            let game = self.game_mut()?;
            game.stage = Stage::Story;
            game.lore_current = Some(id.into());
        } else if step.starts_with("I choose ") {
            let index = parse_after(step, "I choose ")?.parse::<usize>()
                .map_err(|_| "bad choice")?;
            self.game_mut()?.apply(Action::Choose { index })?;
            self.error = None;
        } else if step.starts_with("I attempt to choose ") {
            let index = parse_after(step, "I attempt to choose ")?.parse::<usize>()
                .map_err(|_| "bad attempted choice")?;
            let result = self.game_mut()?.apply(Action::Choose { index });
            self.error = result.err();
            if self.error.is_none() { return Err("expected rejected choice but command succeeded".into()); }
        } else if step == "I remember the current snapshot" {
            self.before = Some(self.game()?.to_json()?);
        } else if step == "I save and restore" {
            let snapshot = self.game()?.to_json()?;
            let game = Game::from_json(&snapshot)?;
            if snapshot != game.to_json()? {
                return Err("save roundtrip changed game state".into());
            }
            self.game = Some(game);
        } else if step == "I start a battle" {
            let game = self.game_mut()?;
            if game.stage != Stage::Map { return Err("start battle requires Map".into()); }
            let index = game.route.iter().position(|node|
                *node == three_kingdoms_spire_reborn::Node::Battle)
                .ok_or("first map has no combat")? + 1;
            game.apply(Action::Choose { index })?;
        } else if step == "I end turn" {
            self.game_mut()?.apply(Action::EndTurn)?;
        } else if step == "the action is rejected" {
            if self.error.is_none() { return Err("no rejected action was recorded".into()); }
        } else if step == "the snapshot is unchanged" {
            let previous = self.before.as_deref().ok_or("no snapshot remembered")?;
            if self.game()?.to_json()? != previous {
                return Err("invalid action mutated the game".into());
            }
        } else if step.starts_with("the flag \"") {
            let key = quoted(step)?;
            let present = self.game()?.lore_flags.iter().any(|flag| flag == key);
            if step.ends_with(" is present") && !present {
                return Err(format!("expected lore flag {key}"));
            }
            if step.ends_with(" is absent") && present {
                return Err(format!("forbidden lore flag {key}"));
            }
        } else if step.starts_with("the bond with \"") {
            let npc = quoted(step)?;
            let active = self.game()?.active_bonds().iter().any(|bond| bond.companion == npc);
            if step.ends_with(" is active") && !active {
                return Err(format!("expected active bond with {npc}"));
            }
            if step.ends_with(" is absent") && active {
                return Err(format!("unexpected active bond with {npc}"));
            }
        } else if step.starts_with("act ") && step.ends_with(" has no available lore scene") {
            let number: u8 = step.split_whitespace().nth(1).ok_or("no act number")?
                .parse().map_err(|_| "bad act")?;
            let game = self.game()?;
            if let Some(next) = lore::next_beat(game.hero,number,&game.lore_flags,&game.lore_seen) {
                return Err(format!("unexpected foreshadowed chapter: {}",next.id));
            }
        } else if step.starts_with("act ") && step.contains(" offers the lore scene ") {
            let number: u8 = step.split_whitespace().nth(1).ok_or("no act number")?
                .parse().map_err(|_| "bad act")?;
            let wanted = quoted(step)?;
            let game = self.game()?;
            let next = lore::next_beat(game.hero,number,&game.lore_flags,&game.lore_seen)
                .ok_or_else(||format!("expected {wanted}, but no beat eligible"))?;
            if next.id != wanted {
                return Err(format!("expected {wanted}, got {}",next.id));
            }
        } else if step.starts_with("battle ") && step.contains(" is at least ") {
            let chunks = step.split_whitespace().collect::<Vec<_>>();
            if chunks.len() != 6 { return Err(format!("invalid battle resource step: {step}")); }
            let expected: i32 = chunks[5].parse().map_err(|_| "bad threshold")?;
            let battle = self.game()?.battle.as_ref().ok_or("not in battle")?;
            let got = match chunks[1] {
                "guard" => battle.block,
                "scheme" => battle.scheme,
                "combo" => battle.combo,
                "command" => battle.command,
                "fury" => battle.fury,
                other => return Err(format!("unknown battle meter {other}")),
            };
            if got < expected { return Err(format!("{}={} below {expected}",chunks[1],got)); }
        } else if step == "the current scene is combat" {
            if self.game()?.stage != Stage::Combat {
                return Err(format!("expected combat, got {:?}",self.game()?.stage));
            }
        } else if step == "every literary scene references a chapter from 1 to 120" {
            if !lore::BEATS.iter().all(|beat| (1..=120).contains(&beat.chapter)) {
                return Err("invalid source chapter".into());
            }
        } else if step == "all narrative prerequisites have earlier setup chapters" {
            for beat in lore::BEATS {
                if let Some(required) = beat.prerequisite
                    && !lore::BEATS.iter().any(|earlier|
                        earlier.flag == required && earlier.chapter <= beat.chapter)
                {
                    return Err(format!("orphan or late foreshadowing for {}",beat.id));
                }
            }
        } else if step == "all character bonds refer to a playable literary scene" {
            for bond in lore::BONDS {
                if !lore::BEATS.iter().any(|beat| beat.flag == bond.flag) {
                    return Err(format!("relationship missing authored scene: {}",bond.companion));
                }
            }
        } else if step == "the literary catalog is explicitly distinguished from historical evidence" {
            let study = include_str!("../docs/NOVEL_STUDY.md");
            if !(study.contains("小说情节和历史真实分开标注")
                && study.contains("不是“全本读毕”")
                && study.contains("120回")) {
                return Err("source study must distinguish fiction, history, and audit scope".into());
            }
        } else {
            return Err(format!("UNIMPLEMENTED BDD STEP: {step}"));
        }
        Ok(())
    }
}

fn run_feature(source: &str, name: &str) -> Result<usize, String> {
    let mut world = World::default();
    let mut count = 0;
    let mut scenario = String::new();
    let mut steps = 0;
    for (line_index,line) in source.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("Feature:") { continue; }
        if let Some(title) = t.strip_prefix("Scenario:") {
            if !scenario.is_empty() && steps == 0 {
                return Err(format!("{name}: empty scenario {scenario}"));
            }
            scenario = title.trim().into();
            world = World::default();
            count += 1;
            steps = 0;
            continue;
        }
        // Gherkin allows free-form Feature descriptions before the first Scenario.
        if scenario.is_empty() { continue; }
        let step = ["Given ","When ","Then ","And ","But "].iter()
            .find_map(|prefix| t.strip_prefix(prefix))
            .ok_or_else(||format!("{name}:{}: unsupported Gherkin syntax: {t}",line_index+1))?;
        if scenario.is_empty() { return Err(format!("{name}: step outside Scenario")); }
        world.assert_case(step).map_err(|error|
            format!("{name}:{}: scenario \"{scenario}\"; step \"{step}\": {error}",line_index+1))?;
        steps += 1;
    }
    if count == 0 || steps == 0 { return Err(format!("{name}: no runnable scenario")); }
    Ok(count)
}

#[test]
fn gherkin_bdd_acceptance_suite() {
    let documents = [
        ("features/lore.feature", include_str!("../features/lore.feature")),
        ("features/save_resume.feature", include_str!("../features/save_resume.feature")),
        ("features/novel_provenance.feature", include_str!("../features/novel_provenance.feature")),
    ];
    let scenarios: usize = documents.iter().map(|(name,text)|
        run_feature(text,name).unwrap_or_else(|error|panic!("{error}"))).sum();
    assert!(scenarios >= 12, "BDD suite lost scenarios: {scenarios}");
    println!("Verified {scenarios} executable Given/When/Then acceptance scenarios");
}

#[test]
fn unsupported_bdd_steps_fail_loudly() {
    let invalid = "Feature: test\n  Scenario: Unknown behavior\n    Given something undefined\n";
    assert!(run_feature(invalid,"invalid.feature").unwrap_err().contains("UNIMPLEMENTED"));
}
