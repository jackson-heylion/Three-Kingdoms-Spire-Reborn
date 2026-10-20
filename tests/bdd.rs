//! Executable BDD feature scenarios for story, combat and save portability.
//! Intentionally supports a small strict Gherkin subset (Feature/Scenario/Given/When/Then/And).
//! Every .feature step must be recognized; unknown steps FAIL instead of being silently skipped.
//! No additional runtime/test dependency is required for CLI, macOS native, or WASM builds.
use three_kingdoms_spire_reborn::{
    Action, BossMove, Card, Foe, Game, Intent, Node, Stage, content::{Hero, enemy_hp}, lore,
    combat::{EventKind, PowerId},
    progression::Profile,
    persistence::{SaveDirectoryLock, apply_and_persist, replace_run_and_persist, atomic_write, restore_primary_from_backup},
};

#[derive(Default)]
struct World {
    game: Option<Game>,
    before: Option<String>,
    error: Option<String>,
    foe_hp: Option<i32>,
    counter_gold: Option<i32>,
    event_cursor: Option<u64>,
    lock_dir: Option<std::path::PathBuf>,
    save_lock: Option<SaveDirectoryLock>,
    profile: Option<Profile>,
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
        if step == "an isolated save directory" {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?.as_nanos();
            self.lock_dir = Some(std::env::temp_dir().join(format!(
                "tks-bdd-single-writer-{}-{nanos}", std::process::id()
            )));
        } else if step == "I assign a recoverable start receipt" {
            let game = self.game_mut()?;
            game.run_id = "bdd-recovered-run".into();
            game.profile_start_receipt = true;
            self.profile = Some(Profile::new());
        } else if step == "I mark the campaign as a completed victory" {
            let game = self.game_mut()?;
            game.floor = 21;
            game.stage = Stage::Won;
        } else if step == "I reconcile the campaign from its saved snapshot" {
            let snapshot = self.game()?.to_json()?;
            let game = Game::from_json(&snapshot)?;
            let profile = self.profile.get_or_insert_with(Profile::new);
            profile.on_start_once(&game)?;
            if matches!(game.stage, Stage::Won | Stage::Lost) {
                profile.record(&game, game.run_id.clone())?;
            }
            self.game = Some(game);
        } else if step == "I reload and reconcile the same receipt" {
            let profile = self.profile.as_ref().ok_or("missing chronicle")?.to_json()?;
            self.profile = Some(Profile::from_json(&profile)?);
            let snapshot = self.game()?.to_json()?;
            let game = Game::from_json(&snapshot)?;
            let profile = self.profile.as_mut().ok_or("missing chronicle")?;
            profile.on_start_once(&game)?;
            if matches!(game.stage, Stage::Won | Stage::Lost) {
                profile.record(&game, game.run_id.clone())?;
            }
        } else if step == "exactly one permanent campaign victory is recorded" {
            let profile = self.profile.as_ref().ok_or("missing chronicle")?;
            let game = self.game()?;
            if profile.wins != 1 || profile.losses != 0 || profile.records.len() != 1
                || !profile.recorded_results.contains(&game.run_id)
                || profile.next_ascension(game.hero) != 1
            {
                return Err("terminal victory was not reconciled exactly once".into());
            }
        } else if step == "exactly one campaign start is recorded" {
            let profile = self.profile.as_ref().ok_or("missing chronicle")?;
            let game = self.game()?;
            if profile.started != 1
                || !profile.started_run_ids.contains(&game.run_id)
                || !profile.achievements.contains("first_step")
            {
                return Err("recoverable campaign was not counted exactly once".into());
            }
        } else if step == "I write the initial campaign snapshot" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            atomic_write(&dir.join("save.json"), &self.game()?.to_json()?)
                .map_err(|error| error.to_string())?;
        } else if step == "I attempt a persisted new campaign while disk write is blocked" {
            let path = self.lock_dir.as_ref().ok_or("missing isolated save directory")?
                .join("save.json");
            self.error = replace_run_and_persist(
                &mut self.game,
                Game::new(Hero::CaoCao, 96114),
                |candidate| atomic_write(&path, &candidate.to_json()?)
                    .map_err(|error| error.to_string()),
            ).err();
            if self.error.is_none() {
                return Err("replaced the campaign despite a failed save".into());
            }
        } else if step == "I retry the persisted new campaign" {
            let path = self.lock_dir.as_ref().ok_or("missing isolated save directory")?
                .join("save.json");
            replace_run_and_persist(
                &mut self.game,
                Game::new(Hero::CaoCao, 96114),
                |candidate| atomic_write(&path, &candidate.to_json()?)
                    .map_err(|error| error.to_string()),
            )?;
            self.error = None;
        } else if step == "the replacement campaign seed is 96114" {
            if self.game()?.seed != 96114 {
                return Err("new campaign not committed after successful retry".into());
            }
        } else if step == "I attempt a persisted choice while disk write is blocked" {
            let path = self.lock_dir.as_ref().ok_or("missing isolated save directory")?
                .join("save.json");
            self.error = apply_and_persist(
                self.game.as_mut().ok_or("missing game")?,
                Action::Choose { index: 1 },
                |candidate| atomic_write(&path, &candidate.to_json()?)
                    .map_err(|error| error.to_string()),
            ).err();
            if self.error.is_none() {
                return Err("expected failed persist but action committed".into());
            }
        } else if step == "I unblock save replacement" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            std::fs::remove_dir(dir.join("save.bak")).map_err(|error| error.to_string())?;
        } else if step == "I retry the persisted choice" {
            let path = self.lock_dir.as_ref().ok_or("missing isolated save directory")?
                .join("save.json");
            apply_and_persist(
                self.game.as_mut().ok_or("missing game")?,
                Action::Choose { index: 1 },
                |candidate| atomic_write(&path, &candidate.to_json()?)
                    .map_err(|error| error.to_string()),
            )?;
            self.error = None;
        } else if step == "the disk snapshot matches the current campaign" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            let raw = std::fs::read_to_string(dir.join("save.json"))
                .map_err(|error| error.to_string())?;
            if raw != self.game()?.to_json()? {
                return Err("disk save diverged from authoritative game".into());
            }
            Game::from_json(&raw)?.verify_action_replay()?;
        } else if step == "the campaign revision is exactly one" {
            if self.game()?.revision != 1 {
                return Err(format!("expected one committed action; got {}", self.game()?.revision));
            }
        } else if step == "I write two atomic save generations" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            atomic_write(&dir.join("save.json"), r#"{"generation":1}"#)
                .map_err(|e|e.to_string())?;
            atomic_write(&dir.join("save.json"), r#"{"generation":2}"#)
                .map_err(|e|e.to_string())?;
        } else if step == "the primary is generation 2 and the backup is generation 1" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            if std::fs::read_to_string(dir.join("save.json")).map_err(|e|e.to_string())?
                != r#"{"generation":2}"#
                || std::fs::read_to_string(dir.join("save.bak")).map_err(|e|e.to_string())?
                    != r#"{"generation":1}"#
            { return Err("a save generation is missing or torn".into()); }
        } else if step == "I corrupt the primary and restore the verified backup" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            let path = dir.join("save.json");
            std::fs::write(&path, "{broken").map_err(|e|e.to_string())?;
            let verified = std::fs::read_to_string(dir.join("save.bak"))
                .map_err(|e|e.to_string())?;
            if verified != r#"{"generation":1}"# {
                return Err("the source backup was not verified".into());
            }
            std::fs::rename(&path, path.with_extension("corrupt"))
                .map_err(|e|e.to_string())?;
            restore_primary_from_backup(&path, &verified).map_err(|e|e.to_string())?;
        } else if step == "the primary and backup both contain intact generation 1" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            if std::fs::read_to_string(dir.join("save.json")).map_err(|e|e.to_string())?
                    != r#"{"generation":1}"#
                || std::fs::read_to_string(dir.join("save.bak")).map_err(|e|e.to_string())?
                    != r#"{"generation":1}"#
                || std::fs::read_to_string(dir.join("save.corrupt")).map_err(|e|e.to_string())?
                    != "{broken"
            { return Err("recovery overwrote backup or lost the broken primary".into()); }
        } else if step == "I write a first atomic save" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            atomic_write(&dir.join("save.json"), "first").map_err(|e|e.to_string())?;
        } else if step == "I block backup replacement with a directory" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            std::fs::create_dir(dir.join("save.bak")).map_err(|e|e.to_string())?;
        } else if step == "a second atomic save is refused" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            if atomic_write(&dir.join("save.json"), "second").is_ok() {
                return Err("overwrote a directory used as the backup target".into());
            }
        } else if step == "the previous primary is intact and no staged files remain" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            if std::fs::read_to_string(dir.join("save.json")).map_err(|e|e.to_string())?
                != "first" || std::fs::read_dir(dir).map_err(|e|e.to_string())?
                    .any(|e|e.is_ok_and(|e|e.file_name().to_string_lossy().contains(".tmp.")))
            { return Err("failed save modified primary or leaked temporary file".into()); }
        } else if step == "no staged files remain in the save directory" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            if std::fs::read_dir(dir).map_err(|e|e.to_string())?
                .any(|e|e.is_ok_and(|e|e.file_name().to_string_lossy().contains(".tmp.")))
            { return Err("temporary save leaked".into()); }
        } else if step == "I start a writer in that directory" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            self.save_lock = Some(SaveDirectoryLock::acquire(dir).map_err(|e| e.to_string())?);
        } else if step == "another writer is refused without modifying the save" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            let before = std::fs::read(dir.join("save.json")).ok();
            let error = SaveDirectoryLock::acquire(dir).err()
                .ok_or("second writer unexpectedly acquired live save directory")?;
            if error.kind() != std::io::ErrorKind::WouldBlock {
                return Err(format!("wrong lock error: {error}"));
            }
            if std::fs::read(dir.join("save.json")).ok() != before {
                return Err("lock rejection changed the save".into());
            }
        } else if step == "I close the first writer" {
            self.save_lock = None;
        } else if step == "a new writer can take the same directory lock" {
            let dir = self.lock_dir.as_ref().ok_or("missing isolated save directory")?;
            let guard = SaveDirectoryLock::acquire(dir).map_err(|e| e.to_string())?;
            drop(guard);
        } else if let Some(prefix) = step.strip_prefix("a custom \"") {
            let hero=prefix.split('"').next().ok_or("missing custom hero")?;
            let seed:u64=step.split("with seed ").nth(1).ok_or("missing seed")?
                .parse().map_err(|_|"bad custom seed")?;
            self.game=Some(Game::new_with_mode(Hero::parse(hero).ok_or("unknown hero")?,
                seed,0,three_kingdoms_spire_reborn::RunMode::Custom));
            self.before=None;
            self.error=None;
        } else if let Some(prefix) = step.strip_prefix("a \"") {
            let hero = prefix.split('"').next().ok_or("missing hero")?;
            let seed: u64 = step.split("with seed ").nth(1).ok_or("missing seed")?
                .parse().map_err(|_| "bad seed")?;
            let hero = Hero::parse(hero).ok_or("unknown hero")?;
            self.game = Some(Game::new(hero, seed));
            self.before = None;
            self.error = None;
            self.foe_hp = None;
            self.counter_gold = None;
            self.event_cursor = None;
            self.profile = None;
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
        } else if step == "I forge a negative-gold save document" {
            let original = self.game()?.to_json()?;
            let mut document: serde_json::Value = serde_json::from_str(&original)
                .map_err(|e| e.to_string())?;
            document["gold"] = serde_json::json!(-1);
            self.error = Game::from_json(&document.to_string()).err();
            self.before = Some(original);
        } else if step == "I register a one-shot weak reduction listener" {
            use three_kingdoms_spire_reborn::event_stack::{Listener,Response};
            self.game_mut()?.apply(Action::RegisterReaction {
                listener:Listener::once("gherkin:weak","weak",Response::Reduce(1))
            })?;
        } else if step == "I attempt to register a listener outside custom mode" {
            use three_kingdoms_spire_reborn::event_stack::{Listener,Response};
            self.before=Some(self.game()?.to_json()?);
            self.error=self.game_mut()?.apply(Action::RegisterReaction {
                listener:Listener::once("forbidden","weak",Response::Cancel)
            }).err();
            if self.error.is_none(){return Err("ranked listener unexpectedly accepted".into());}
        } else if step == "the listener and complete action replay agree after save" {
            let game=self.game()?;
            if game.battle.as_ref().ok_or("not in battle")?.reactions.listeners.len()!=1 {
                return Err("registered listener missing".into());
            }
            game.verify_action_replay()?;
            game.reconstruct_event_history()?;
            let json=game.to_json()?;
            if Game::from_json(&json)?.to_json()?!=json {
                return Err("listener changed after save/load".into());
            }
        } else if step == "I enter the first battle for save validation" {
            self.game_mut()?.apply(Action::Choose { index: 1 })?;
            if self.game()?.stage != Stage::Combat {
                return Err("test fixture never entered combat".into());
            }
        } else if step == "I forge a zero-health-cap enemy save document" {
            let original = self.game()?.to_json()?;
            let mut document: serde_json::Value = serde_json::from_str(&original)
                .map_err(|e| e.to_string())?;
            document["battle"]["foes"][0]["max_hp"] = serde_json::json!(0);
            self.error = Game::from_json(&document.to_string()).err();
            self.before = Some(original);
        } else if step.starts_with("the forged save is rejected as \"") {
            let expected = quoted(step)?;
            if !self.error.as_deref().is_some_and(|error| error.contains(expected)) {
                return Err(format!("expected invalid save error containing {expected:?}, got {:?}", self.error));
            }
        } else if step.starts_with("I stage a market offering \"") {
            let id = quoted(step)?;
            if three_kingdoms_spire_reborn::content::card_spec(id).is_none() {
                return Err(format!("unknown market card {id}"));
            }
            let game = self.game_mut()?;
            game.stage = Stage::Market;
            game.last_node = Some(Node::Market);
            game.market = vec![Card::new(id)];
            game.gold = 90;
        } else if step == "I reduce market gold to 44" {
            self.game_mut()?.gold = 44;
        } else if step.starts_with("I attempt to buy market slot ") {
            let index:usize = parse_after(step, "I attempt to buy market slot ")?
                .parse().map_err(|_| "bad market slot")?;
            self.error = self.game_mut()?.apply(Action::Choose { index }).err();
            if self.error.is_none() { return Err("invalid market purchase succeeded".into()); }
        } else if step == "I buy market slot 1" {
            self.game_mut()?.apply(Action::Choose { index: 1 })?;
            self.error = None;
        } else if step == "market gold is 45 and the shelf is empty" {
            let game = self.game()?;
            if game.gold != 45 || !game.market.is_empty() {
                return Err("market did not charge 45 exactly and consume the only stock".into());
            }
        } else if step.starts_with("my permanent deck has exactly one \"") {
            let id = quoted(step)?;
            let count = self.game()?.deck.iter().filter(|card| card.id == id).count();
            if count != 1 { return Err(format!("expected one permanent {id}, got {count}")); }
        } else if step == "I leave the market" {
            self.game_mut()?.apply(Action::Choose { index: 0 })?;
        } else if step.starts_with("I prepare potion \"") {
            let id = quoted(step)?;
            if !three_kingdoms_spire_reborn::items::POTIONS.iter().any(|(valid, _)| *valid == id) {
                return Err(format!("unknown potion fixture: {id}"));
            }
            let game = self.game_mut()?;
            game.potion_bag = vec![id.to_owned()];
            game.potions = 1;
        } else if step.starts_with("I use potion ") {
            let value = parse_after(step, "I use potion ")?;
            let (slot, target) = value.split_once(" against enemy ")
                .ok_or("missing potion target")?;
            let slot = slot.parse::<usize>().map_err(|_| "invalid potion slot")?;
            let target = target.parse::<usize>().map_err(|_| "invalid potion target")?;
            self.game_mut()?.apply(Action::UsePotionAt { slot, target })?;
        } else if step.starts_with("I attempt to use potion ") {
            let value = parse_after(step, "I attempt to use potion ")?;
            let (slot, target) = value.split_once(" against enemy ")
                .ok_or("missing potion target")?;
            let slot = slot.parse::<usize>().map_err(|_| "invalid potion slot")?;
            let target = target.parse::<usize>().map_err(|_| "invalid potion target")?;
            self.error = self.game_mut()?.apply(Action::UsePotionAt { slot, target }).err();
            if self.error.is_none() { return Err("invalid potion unexpectedly succeeded".into()); }
        } else if step.starts_with("I set doctrine \"") {
            let id=quoted(step)?.to_owned();
            self.game_mut()?.apply(Action::SetDoctrine{id})?;
        } else if step.starts_with("I attempt doctrine \"") {
            let id=quoted(step)?.to_owned();
            self.error=self.game_mut()?.apply(Action::SetDoctrine{id}).err();
            if self.error.is_none() {return Err("expected rejected doctrine change".into());}
        } else if step.starts_with("the active doctrine is \"") {
            let desired=quoted(step)?;
            let actual=self.game()?.effective_doctrine().map(|d|d.id());
            if actual!=Some(desired) {return Err(format!("wanted {desired}, got {actual:?}"));}
        } else if step == "the active doctrine is automatic and not yet inferred" {
            let game=self.game()?;
            if game.doctrine.is_some() || game.effective_doctrine().is_some() {
                return Err("doctrine inferred too early or preference persisted".into());
            }
        } else if step.starts_with("I add permanent card \"") {
            let id=quoted(step)?;
            if three_kingdoms_spire_reborn::content::card_spec(id).is_none() {
                return Err(format!("unknown added card {id}"));
            }
            self.game_mut()?.deck.push(Card::new(id));
        } else if step == "the reward screen has three distinct legal cards" {
            let game=self.game()?;
            if game.stage!=Stage::Reward || game.rewards.len()!=3 {
                return Err("expected three-card reward screen".into());
            }
            for (idx,card) in game.rewards.iter().enumerate() {
                if three_kingdoms_spire_reborn::content::card_spec(&card.id).is_none()
                    || game.rewards.iter().skip(idx+1).any(|other|other.id==card.id)
                {return Err("duplicate or nonexistent reward card".into());}
            }
        } else if step == "the map has valid explicit connected routes" {
            let map=self.game()?.act_map.as_ref().ok_or("missing act map")?;
            if !map.is_valid() || map.edges.len()!=three_kingdoms_spire_reborn::map::ROWS-1 {
                return Err("act map is disconnected or still uses legacy topology".into());
            }
        } else if step.starts_with("the full action replay verifies ") {
            let count:usize=parse_after(step,"the full action replay verifies ")?
                .trim_end_matches(" steps").parse().map_err(|_|"invalid replay count")?;
            let actual=self.game()?.verify_action_replay()?;
            if actual!=count {return Err(format!("expected replay count {count}, got {actual}"));}
        } else if step == "I tamper with non-action gold" {
            self.game_mut()?.gold+=1;
        } else if step == "the full replay rejects the drift" {
            if self.game()?.verify_action_replay().is_ok() {
                return Err("mutated run unexpectedly passed seed/action replay".into());
            }
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
        } else if step.starts_with("the last card offers secondary choice ") {
            let suffix = parse_after(step,"the last card offers secondary choice ")?;
            let number = suffix.split_whitespace().next().ok_or("missing choice number")?
                .parse::<usize>().map_err(|_|"bad choice number")?;
            let id = quoted(step)?;
            let game = self.game()?;
            let hand_number = game.battle.as_ref().ok_or("not in combat")?.hand.len();
            let choices = game.secondary_choices(hand_number)?
                .ok_or("hand card has no secondary choices")?;
            if !choices.iter().any(|(n,card)| *n==number && card.id==id) {
                return Err(format!("choice {number} {id} missing: {choices:?}"));
            }
        } else if step.starts_with("the last card excludes secondary choice ") {
            let number = parse_after(step,"the last card excludes secondary choice ")?
                .parse::<usize>().map_err(|_|"bad excluded choice number")?;
            let game = self.game()?;
            let hand_number = game.battle.as_ref().ok_or("not in combat")?.hand.len();
            let choices = game.secondary_choices(hand_number)?
                .ok_or("hand card has no secondary choices")?;
            if choices.iter().any(|(n,_)|*n==number) {
                return Err(format!("choice {number} should be filtered: {choices:?}"));
            }
        } else if step.starts_with("I deploy boss \"") {
            let id = quoted(step)?;
            let (floor, branch) = match id {
                "zhangjiao" => (7,0), "sunjian" => (7,1),
                "dongzhuo" => (14,0), "yuanshao" => (14,1),
                "simayi" => (21,0), "zuoci" => (21,1),
                _ => return Err(format!("unknown boss {id}")),
            };
            let g = self.game_mut()?;
            if g.stage != Stage::Map {return Err("boss encounter must start from map".into());}
            g.floor=floor-1;
            g.variants=Some([branch;3]);
            g.route=vec![Node::Boss];
            g.route_lanes=vec![0];
            g.apply(Action::Choose {index:1})?;
            if g.battle.as_ref().ok_or("missing boss battle")?.foes[0].id != id {
                return Err("staged boss does not match selected variant".into());
            }
        } else if step == "I prepare lethal counterplay against the boss" {
            let gold = self.game()?.gold;
            self.counter_gold = Some(gold);
            let b = self.game_mut()?.battle.as_mut().ok_or("not in combat")?;
            if b.foes.len() != 1 || !matches!(b.foes[0].intent,
                Intent::Boss(BossMove::Thunder(_) | BossMove::TyrantLevy(_))) {
                return Err("expected a telegraphed thunder or levy boss".into());
            }
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
            b.foes[0].phase = 1; // Do not trigger unrelated half-health transitions.
            b.thorns = 5;
            b.block = 0;
            b.weak = 0;
            b.troops_hp = 0;
            // Keep combat open so ordered foe-death events remain inspectable.
            b.foes.push(Foe { id: "bandit".into(), hp: 90, max_hp: 90,
                block: 0, strength: 0, poison: 0, vulnerable: 0, weak: 0,
                intent: Intent::Guard(0), phase: 0 });
        } else if step == "I prepare a last-foe thorn counter at turn 44"
            || step == "I prepare a mutual-lethal thorn counter at turn 44" {
            let lethal_hero_hit = step.contains("mutual-lethal");
            let game = self.game_mut()?;
            game.hp = 1;
            let b = game.battle.as_mut().ok_or("not in combat")?;
            if b.foes.len() != 1 { return Err("expected exactly one living foe".into()); }
            b.turn = 44;
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
            b.foes[0].intent = Intent::Attack(if lethal_hero_hit { 10 } else { 0 });
            b.thorns = 5;
            b.block = 0;
            b.troops_hp = 0;
        } else if step == "the counterattack wins before supply exhaustion" {
            let game = self.game()?;
            if game.stage != Stage::Reward || game.hp != 1 || game.battle.is_some()
                || game.telemetry.totals.battles_won != 1
                || game.telemetry.totals.battles_lost != 0
                || game.telemetry.recent_turns.last().is_none_or(|t| t.turn != 44 || t.completed)
                || !game.log.last().is_some_and(|line| line.contains("反击清场"))
            { return Err("final thorn kill did not settle before attrition/new turn".into()); }
        } else if step == "the simultaneous lethal counterattack is a loss" {
            let game = self.game()?;
            if game.stage != Stage::Lost || game.hp != 0
                || game.telemetry.totals.battles_won != 0
                || game.telemetry.totals.battles_lost != 1
            { return Err("mutual lethal counterattack incorrectly awarded a win".into()); }
        } else if step == "the boss was defeated by thorns" {
            let battle = self.game()?.battle.as_ref().ok_or("combat ended before audit")?;
            if battle.foes[0].hp != 0 || !battle.events.iter().any(|event|
                event.kind == EventKind::FoeDefeated && event.source == "thorns"
                && event.target.as_deref() == Some("foe:0")) {
                return Err("boss death via thorns was not recorded".into());
            }
        } else if step == "the hero remains unweakened" {
            if self.game()?.battle.as_ref().ok_or("not in combat")?.weak != 0 {
                return Err("posthumous thunder applied weakness".into());
            }
        } else if step == "the slain tyrant did not steal gold" {
            let gold = self.counter_gold.ok_or("counterplay gold not remembered")?;
            if self.game()?.gold != gold { return Err("posthumous levy stole gold".into()); }
        } else if step == "I clear my guard" {
            self.game_mut()?.battle.as_mut().ok_or("not in combat")?.block=0;
        } else if step == "I wound the boss by 8 health" {
            let boss = &mut self.game_mut()?.battle.as_mut().ok_or("not in combat")?.foes[0];
            boss.hp=(boss.hp-8).max(1);
        } else if step == "I play four free cards" {
            let game=self.game_mut()?;
            let b=game.battle.as_mut().ok_or("not in combat")?;
            b.hand=vec![Card::new("guji");4];
            for _ in 0..4 {
                game.apply(Action::Play {index:1,target:1})?;
            }
        } else if step == "I stage coordinated officers" {
            let b=self.game_mut()?.battle.as_mut().ok_or("not in combat")?;
            let hp=enemy_hp("officer");
            b.foes=vec![
                Foe {id:"officer".into(),hp,max_hp:hp,block:0,strength:0,poison:0,
                    vulnerable:0,weak:0,intent:Intent::Assist(4),phase:0},
                Foe {id:"bandit".into(),hp:32,max_hp:32,block:0,strength:0,poison:0,
                    vulnerable:0,weak:0,intent:Intent::Attack(1),phase:0},
            ];
        } else if step.starts_with("the first foe telegraphs \"") {
            let wanted=quoted(step)?;
            let battle=self.game()?.battle.as_ref().ok_or("not in combat")?;
            let description=battle.foes[0].intent.label();
            if !description.contains(wanted) {
                return Err(format!("expected boss telegraph {wanted}, got {description}"));
            }
        } else if step.starts_with("the first foe has at least ") && step.ends_with(" guard") {
            let threshold: i32 = parse_after(step,"the first foe has at least ")?
                .split_whitespace().next().ok_or("no guard threshold")?
                .parse().map_err(|_|"invalid guard threshold")?;
            let actual=self.game()?.battle.as_ref().ok_or("not in combat")?.foes[0].block;
            if actual<threshold {return Err(format!("enemy guard {actual} below {threshold}"));}
        } else if step.starts_with("the second foe has at least ") && step.ends_with(" guard") {
            let threshold: i32 = parse_after(step,"the second foe has at least ")?
                .split_whitespace().next().ok_or("no guard threshold")?
                .parse().map_err(|_|"invalid guard threshold")?;
            let actual=self.game()?.battle.as_ref().ok_or("not in combat")?
                .foes.get(1).ok_or("second foe absent")?.block;
            if actual<threshold {return Err(format!("second foe guard {actual} below {threshold}"));}
        } else if step == "the hero is weakened" {
            if self.game()?.battle.as_ref().ok_or("not in combat")?.weak == 0 {
                return Err("boss failed to inflict lasting weakness".into());
            }
        } else if step.starts_with("hero gold is at most ") {
            let maximum: i32=parse_after(step,"hero gold is at most ")?
                .parse().map_err(|_|"invalid gold assertion")?;
            if self.game()?.gold>maximum {
                return Err(format!("gold {} exceeds {maximum}",self.game()?.gold));
            }
        } else if step == "boss health has increased" {
            let before=self.foe_hp.ok_or("boss health not remembered")?;
            let after=self.game()?.battle.as_ref().ok_or("not in combat")?.foes[0].hp;
            if after<=before{return Err(format!("expected boss recovery, got {before} -> {after}"));}
        } else if step == "the first enemy telegraphs calculated sabotage" {
            let intent=&self.game()?.battle.as_ref().ok_or("not in combat")?.foes[0].intent;
            if !matches!(intent,Intent::Boss(BossMove::HiddenScheme(_))) {
                return Err(format!("expected anti-spam hidden scheme, got {intent:?}"));
            }
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
        } else if step == "I duplicate the first enemy as a second combatant" {
            let b = self.game_mut()?.battle.as_mut().ok_or("not in combat")?;
            if b.foes.len() != 1 { return Err("expected a single enemy fixture".into()); }
            b.foes.push(b.foes[0].clone());
        } else if step == "the first enemy has one health and no guard" {
            let b = self.game_mut()?.battle.as_mut().ok_or("not in combat")?;
            b.foes[0].hp = 1;
            b.foes[0].block = 0;
        } else if step.starts_with("the first enemy has at least ") && step.ends_with(" poison") {
            let value = parse_after(step,"the first enemy has at least ")?
                .split_whitespace().next().ok_or("bad poison value")?
                .parse::<i32>().map_err(|_|"invalid poison threshold")?;
            let actual = self.game()?.battle.as_ref().ok_or("not in battle")?.foes[0].poison;
            if actual < value {return Err(format!("expected poison >= {value}, got {actual}")); }
        } else if step.starts_with("my hand contains \"") {
            let id = quoted(step)?;
            let b = self.game()?.battle.as_ref().ok_or("not in combat")?;
            if !b.hand.iter().any(|c|c.id==id) {return Err(format!("{id} was not retained")); }
        } else if step.starts_with("my exhaust pile contains \"") {
            let id = quoted(step)?;
            let b = self.game()?.battle.as_ref().ok_or("not in combat")?;
            if !b.exhausted.iter().any(|c|c.id==id) {
                return Err(format!("{id} was not exhausted at end turn"));
            }
        } else if step.starts_with("I replace hand with \"") {
            let id = quoted(step)?;
            let b = self.game_mut()?.battle.as_mut().ok_or("not in combat")?;
            b.hand = vec![Card::new(id)];
            b.energy = b.energy.max(3);
        } else if step.starts_with("I replace draw pile with \"") {
            let ids = quoted(step)?.split(',').collect::<Vec<_>>();
            let cards = ids.iter().map(|id| {
                if three_kingdoms_spire_reborn::content::card_spec(id).is_none() {
                    Err(format!("unknown draw fixture card {id}"))
                } else { Ok(Card::new(id)) }
            }).collect::<Result<Vec<_>,_>>()?;
            self.game_mut()?.battle.as_mut().ok_or("not in combat")?.draw = cards;
        } else if step.starts_with("I replace discard pile with \"") {
            let ids = quoted(step)?.split(',').collect::<Vec<_>>();
            let cards = ids.iter().map(|id| {
                if three_kingdoms_spire_reborn::content::card_spec(id).is_none() {
                    Err(format!("unknown discard fixture card {id}"))
                } else { Ok(Card::new(id)) }
            }).collect::<Result<Vec<_>,_>>()?;
            self.game_mut()?.battle.as_mut().ok_or("not in combat")?.discard = cards;
        } else if step.starts_with("I set combat energy to ") {
            let energy: i32 = parse_after(step,"I set combat energy to ")?
                .parse().map_err(|_|"invalid energy amount")?;
            if !(0..=99).contains(&energy) { return Err("energy out of fixture range".into()); }
            self.game_mut()?.battle.as_mut().ok_or("not in combat")?.energy = energy;
        } else if step == "I make the first enemy healthy with 200 HP" {
            let b = self.game_mut()?.battle.as_mut().ok_or("not in combat")?;
            b.foes[0].max_hp = 200;
            b.foes[0].hp = 200;
            b.foes[0].block = 0;
        } else if step.starts_with("battle troops attack equals ") {
            let expected:i32=parse_after(step,"battle troops attack equals ")?
                .parse().map_err(|_|"invalid troop attack")?;
            let actual=self.game()?.battle.as_ref().ok_or("not in combat")?.troops_attack;
            if actual!=expected {
                return Err(format!("troop attack {actual} differs from expected {expected}"));
            }
        } else if step.starts_with("battle energy equals ") {
            let expected: i32 = parse_after(step,"battle energy equals ")?
                .parse().map_err(|_|"bad energy assertion")?;
            let actual = self.game()?.battle.as_ref().ok_or("not in combat")?.energy;
            if actual != expected { return Err(format!("expected {expected} energy, got {actual}")); }
        } else if step == "I play the last card selecting 2" {
            let game = self.game_mut()?;
            let index = game.battle.as_ref().ok_or("not in combat")?.hand.len();
            game.apply(Action::PlaySelected {index,target:1,selection:2})?;
        } else if step == "I attempt to play the last card selecting 99" {
            let game = self.game_mut()?;
            let index = game.battle.as_ref().ok_or("not in combat")?.hand.len();
            self.error = game.apply(Action::PlaySelected {index,target:1,selection:99}).err();
            if self.error.is_none() { return Err("invalid pile choice was accepted".into()); }
        } else if step.starts_with("my hand has a temporary \"") {
            let id = quoted(step)?;
            let b = self.game()?.battle.as_ref().ok_or("not in combat")?;
            if !b.hand.iter().any(|card|card.id==id && card.temporary) {
                return Err(format!("expected temporary {id} in hand"));
            }
        } else if step.starts_with("my exhaust pile has a temporary \"") {
            let id = quoted(step)?;
            let b = self.game()?.battle.as_ref().ok_or("not in combat")?;
            if !b.exhausted.iter().any(|card|card.id==id && card.temporary) {
                return Err(format!("expected temporary {id} exhausted"));
            }
        } else if step == "I set command to three" {
            self.game_mut()?.battle.as_mut().ok_or("not in combat")?.command=3;
        } else if step == "the previous turn telemetry has completed turn one" {
            let g=self.game()?;
            let sample=g.telemetry.recent_turns.last().ok_or("missing turn telemetry")?;
            if !sample.completed || sample.turn!=1 || sample.floor!=1 {
                return Err(format!("unexpected turn telemetry: {sample:?}"));
            }
        } else if step == "run troop attacks are at least one" {
            if self.game()?.telemetry.totals.troop_attacks==0 {
                return Err("missing actual troop attack telemetry".into());
            }
        } else if step == "run command energy refunds equal one" {
            let m=&self.game()?.telemetry.totals;
            if m.command_energy!=1 || m.command_procs!=1 || m.command_block!=5 {
                return Err(format!("unexpected command refund counters: {m:?}"));
            }
        } else if step == "run Sun discards equal one" {
            let m=&self.game()?.telemetry.totals;
            if m.sun_discards!=1 || m.sun_discard_block!=2 {
                return Err(format!("unexpected Sun discard counters: {m:?}"));
            }
        } else if step == "run Sun combo triggers equal one" {
            let m=&self.game()?.telemetry.totals;
            if m.sun_combo_procs!=1 || m.sun_combo_block!=3 {
                return Err(format!("unexpected Sun combo counters: {m:?}"));
            }
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
        } else if step == "the first enemy health is unchanged" {
            let previous = self.foe_hp.ok_or("no remembered enemy health")?;
            let current = self.game()?.battle.as_ref().ok_or("not in combat")?.foes[0].hp;
            if previous != current {
                return Err(format!("expected unchanged health {previous}, got {current}"));
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
                "wisdom_on_draw" => PowerId::WisdomOnDraw,
                "toxic_reversal" => PowerId::ToxicReversal,
                "victory_drum" => PowerId::VictoryDrum,
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
                "card_resolved" => EventKind::CardResolved,
                "card_drawn" => EventKind::CardDrawn,
                "card_exhausted" => EventKind::CardExhausted,
                "status_applied" => EventKind::StatusApplied,
                "foe_defeated" => EventKind::FoeDefeated,
                "potion_used" => EventKind::PotionUsed,
                "card_tutored" => EventKind::CardTutored,
                "card_copied" => EventKind::CardCopied,
                "card_generated" => EventKind::CardGenerated,
                "enemy_technique" => EventKind::EnemyTechnique,
                _ => return Err(format!("unknown event kind {kind}")),
            };
            let count = self.game()?.battle.as_ref().ok_or("not in combat")?
                .events.iter().filter(|evt|evt.kind == event && evt.source == source).count();
            if count == 0 { return Err(format!("event {kind}/{source} absent")); }
            if step.ends_with("occurs exactly once") && count != 1 {
                return Err(format!("expected exactly one event {kind}/{source}, observed {count}"));
            }
            if step.ends_with("occurs exactly twice") && count != 2 {
                return Err(format!("expected exactly two events {kind}/{source}, observed {count}"));
            }
        } else if step == "I remember the latest combat event cursor" {
            self.event_cursor = Some(self.game()?.battle.as_ref()
                .ok_or("not in combat")?.event_seq);
        } else if step == "the combat event cursor returns only newer events" {
            let cursor = self.event_cursor.ok_or("no remembered cursor")?;
            let page = self.game()?.battle.as_ref().ok_or("not in combat")?
                .events_after(cursor, 80);
            if page.reset || page.missed > 0 || page.entries.is_empty()
                || page.entries.iter().any(|e| e.seq <= cursor)
                || page.entries.last().is_some_and(|e| e.seq != page.next_cursor)
            {
                return Err("event poll duplicated, skipped or lost committed combat events".into());
            }
        } else if step == "an overlarge event cursor signals reset" {
            let page = self.game()?.battle.as_ref().ok_or("not in combat")?
                .events_after(u64::MAX, 30);
            if !page.reset || !page.entries.is_empty() {
                return Err("old-battle cursor did not report reset".into());
            }
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
        } else if step.starts_with("my deck contains \"") {
            let card = quoted(step)?;
            if !self.game()?.deck.iter().any(|existing|existing.id == card) {
                return Err(format!("expected narrative reward card {card} in deck"));
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
        } else if step.starts_with("act ") && step.contains(" does not offer the lore scene ") {
            let number:u8=step.split_whitespace().nth(1).ok_or("no act number")?
                .parse().map_err(|_|"bad act")?;
            let forbidden=quoted(step)?;
            let game=self.game()?;
            if lore::next_beat(game.hero,number,&game.lore_flags,&game.lore_seen)
                .is_some_and(|b|b.id==forbidden) {
                return Err(format!("chapter {forbidden} was available without its prerequisite"));
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
                "energy" => battle.energy,
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

impl Drop for World {
    fn drop(&mut self) {
        self.save_lock = None;
        if let Some(dir) = self.lock_dir.as_ref() {
            let _ = std::fs::remove_dir_all(dir);
        }
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
        ("features/advanced_combat.feature", include_str!("../features/advanced_combat.feature")),
        ("features/potion_triggers.feature", include_str!("../features/potion_triggers.feature")),
        ("features/market_purchases.feature", include_str!("../features/market_purchases.feature")),
        ("features/advanced_cards.feature", include_str!("../features/advanced_cards.feature")),
        ("features/boss_tactics.feature", include_str!("../features/boss_tactics.feature")),
        ("features/balance_guards.feature", include_str!("../features/balance_guards.feature")),
        ("features/combat_telemetry.feature", include_str!("../features/combat_telemetry.feature")),
        ("features/doctrine_drafting.feature", include_str!("../features/doctrine_drafting.feature")),
        ("features/p1_replay_map.feature", include_str!("../features/p1_replay_map.feature")),
        ("features/lore.feature", include_str!("../features/lore.feature")),
        ("features/save_resume.feature", include_str!("../features/save_resume.feature")),
        ("features/durable_save.feature", include_str!("../features/durable_save.feature")),
        ("features/save_integrity.feature", include_str!("../features/save_integrity.feature")),
        ("features/action_persistence.feature", include_str!("../features/action_persistence.feature")),
        ("features/profile_reconciliation.feature", include_str!("../features/profile_reconciliation.feature")),
        ("features/novel_provenance.feature", include_str!("../features/novel_provenance.feature")),
        ("features/dynamic_reactions.feature", include_str!("../features/dynamic_reactions.feature")),
    ];
    let scenarios: usize = documents.iter().map(|(name,text)|
        run_feature(text,name).unwrap_or_else(|error|panic!("{error}"))).sum();
    assert!(scenarios >= 68, "BDD suite lost scenarios: {scenarios}");
    println!("Verified {scenarios} executable Given/When/Then acceptance scenarios");
}

#[test]
fn unsupported_bdd_steps_fail_loudly() {
    let invalid = "Feature: test\n  Scenario: Unknown behavior\n    Given something undefined\n";
    assert!(run_feature(invalid,"invalid.feature").unwrap_err().contains("UNIMPLEMENTED"));
}
