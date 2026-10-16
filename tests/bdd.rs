//! Executable BDD feature scenarios for story, combat and save portability.
//! Intentionally supports a small strict Gherkin subset (Feature/Scenario/Given/When/Then/And).
//! Every .feature step must be recognized; unknown steps FAIL instead of being silently skipped.
//! No additional runtime/test dependency is required for CLI, macOS native, or WASM builds.
use three_kingdoms_spire_reborn::{
    Action, Card, Game, Stage, content::Hero, lore,
    combat::{EventKind, PowerId},
};

#[derive(Default)]
struct World {
    game: Option<Game>,
    before: Option<String>,
    error: Option<String>,
    foe_hp: Option<i32>,
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
            self.foe_hp = None;
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
        } else if step == "I travel to a story node" {
            let game = self.game_mut()?;
            if game.stage != Stage::Map { return Err("must be on map before story".into()); }
            let index = game.route.iter().position(|node|
                *node == three_kingdoms_spire_reborn::Node::Story)
                .ok_or("no reachable story lane")? + 1;
            game.apply(Action::Choose { index })?;
            if game.stage != Stage::Story { return Err("story node failed to enter event".into()); }
        } else if step.starts_with("the current literary chapter is \"") {
            let id = quoted(step)?;
            if self.game()?.lore_current.as_deref() != Some(id) {
                return Err(format!("expected novel beat {id}, got {:?}",self.game()?.lore_current));
            }
        } else if step == "I start a battle" {
            let game = self.game_mut()?;
            if game.stage != Stage::Map { return Err("start battle requires Map".into()); }
            let index = game.route.iter().position(|node|
                *node == three_kingdoms_spire_reborn::Node::Battle)
                .ok_or("first map has no combat")? + 1;
            game.apply(Action::Choose { index })?;
        } else if step.starts_with("I put card \"") && step.ends_with(" in hand") {
            let id = quoted(step)?;
            if three_kingdoms_spire_reborn::content::card_spec(id).is_none() {
                return Err(format!("unknown test card {id}"));
            }
            let b = self.game_mut()?.battle.as_mut().ok_or("not in combat")?;
            b.hand.push(Card::new(id));
            b.energy = b.energy.max(10);
        } else if step == "I play the last card" {
            let game = self.game_mut()?;
            let idx = game.battle.as_ref().ok_or("no battle")?.hand.len();
            game.apply(Action::Play { index: idx, target: 1 })?;
        } else if step == "I gain one manual discard token" {
            self.game_mut()?.battle.as_mut().ok_or("not in combat")?.discard_tokens += 1;
        } else if step == "I remember the first enemy health" {
            self.foe_hp = Some(self.game()?.battle.as_ref().ok_or("not in battle")?.foes[0].hp);
        } else if step.starts_with("the first enemy loses at least ") {
            let expected: i32 = parse_after(step,"the first enemy loses at least ")?
                .parse().map_err(|_|"bad enemy damage expected")?;
            let prior = self.foe_hp.ok_or("prior foe health not remembered")?;
            let remaining = self.game()?.battle.as_ref().ok_or("not in combat")?.foes[0].hp;
            if prior-remaining < expected {
                return Err(format!("expected damage >= {expected}, got {}",prior-remaining));
            }
        } else if step == "I manually discard the last card" {
            let game = self.game_mut()?;
            let idx = game.battle.as_ref().ok_or("no battle")?.hand.len();
            game.apply(Action::Discard { index: idx })?;
        } else if step.starts_with("the power \"") && step.contains(" has at least ") {
            let id = quoted(step)?;
            let power = match id {
                "oath_guard" => PowerId::OathGuard,
                "thought_spring" => PowerId::ThoughtSpring,
                "arrows_on_discard" => PowerId::ArrowsOnDiscard,
                "granary_support" => PowerId::GranarySupport,
                "blood_courage" => PowerId::BloodCourage,
                _ => return Err(format!("unrecognized power id {id}")),
            };
            let expected: i32 = step.split(" has at least ").nth(1)
                .and_then(|suffix| suffix.split_whitespace().next()).ok_or("missing power threshold")?
                .parse().map_err(|_|"invalid power count")?;
            let actual = self.game()?.battle.as_ref().ok_or("not in combat")?
                .powers.iter().find(|stack|stack.id==power).map_or(0,|stack|stack.stacks);
            if actual < expected { return Err(format!("power {id} stacks {actual} below {expected}")); }
        } else if step.starts_with("combat event \"") && step.contains(" from \"") {
            let pieces = step.split('"').collect::<Vec<_>>();
            let kind = *pieces.get(1).ok_or("missing event type")?;
            let source = *pieces.get(3).ok_or("missing event source")?;
            let event = match kind {
                "card_played" => EventKind::CardPlayed,
                "card_discarded" => EventKind::CardDiscarded,
                "damage_dealt" => EventKind::DamageDealt,
                "damage_received" => EventKind::DamageReceived,
                "power_gained" => EventKind::PowerGained,
                "power_triggered" => EventKind::PowerTriggered,
                "turn_started" => EventKind::TurnStarted,
                "turn_ended" => EventKind::TurnEnded,
                _ => return Err(format!("unknown event kind {kind}")),
            };
            let found = self.game()?.battle.as_ref().ok_or("not in combat")?
                .events.iter().any(|evt|evt.kind == event && evt.source == source);
            if !found { return Err(format!("event {kind}/{source} absent")); }
        } else if step == "the event sequence strictly increases" {
            let events = &self.game()?.battle.as_ref().ok_or("not in combat")?.events;
            if !events.windows(2).all(|w|w[0].seq<w[1].seq) {
                return Err("events were reordered or duplicated".into());
            }
        } else if step == "I end turn" {
            self.game_mut()?.apply(Action::EndTurn)?;
        } else if step == "the action is rejected" {
            if self.error.is_none() { return Err("no rejected action was recorded".into()); }
        } else if step == "the snapshot is unchanged" {
            let previous = self.before.as_deref().ok_or("no snapshot remembered")?;
            if self.game()?.to_json()? != previous {
                return Err("invalid action mutated the game".into());
            }
        } else if step.starts_with("the flag \"") && step.ends_with(" was earned earlier") {
            let key = quoted(step)?;
            if !lore::BEATS.iter().any(|beat| beat.flag == key) {
                return Err(format!("unknown history flag: {key}"));
            }
            if !self.game()?.lore_flags.iter().any(|old| old == key) {
                self.game_mut()?.lore_flags.push(key.into());
            }
        } else if step.starts_with("the hidden conclusion \"") {
            let id = quoted(step)?;
            if !self.game()?.easter_unlocked.iter().any(|egg| egg == id) {
                return Err(format!("hidden storyline conclusion {id} was not unlocked"));
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
        } else if step == "novel index contains all 120 numbered chapters" {
            let mut chapters = Vec::new();
            for raw in include_str!("../data/novel_chapters.tsv").lines() {
                if raw.starts_with('#') || raw.starts_with("chapter") || raw.is_empty() { continue; }
                let columns = raw.split('\t').collect::<Vec<_>>();
                if columns.len() != 3 { return Err(format!("bad novel index row: {raw}")); }
                let number = columns[0].parse::<usize>().map_err(|_| "invalid chapter number")?;
                if columns[1].is_empty() || !columns[2].contains("三國演義/第") {
                    return Err(format!("bad source for chapter {number}"));
                }
                chapters.push(number);
            }
            if chapters != (1..=120).collect::<Vec<_>>() {
                return Err(format!("expected contiguous 120 chapters, got {}",chapters.len()));
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
        ("features/combat_powers.feature", include_str!("../features/combat_powers.feature")),
        ("features/lore.feature", include_str!("../features/lore.feature")),
        ("features/save_resume.feature", include_str!("../features/save_resume.feature")),
        ("features/novel_provenance.feature", include_str!("../features/novel_provenance.feature")),
    ];
    let scenarios: usize = documents.iter().map(|(name,text)|
        run_feature(text,name).unwrap_or_else(|error|panic!("{error}"))).sum();
    assert!(scenarios >= 22, "BDD suite lost scenarios: {scenarios}");
    println!("Verified {scenarios} executable Given/When/Then acceptance scenarios");
}

#[test]
fn unsupported_bdd_steps_fail_loudly() {
    let invalid = "Feature: test\n  Scenario: Unknown behavior\n    Given something undefined\n";
    assert!(run_feature(invalid,"invalid.feature").unwrap_err().contains("UNIMPLEMENTED"));
}
