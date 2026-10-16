//! macOS terminal frontend. All rules remain in the library.
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use three_kingdoms_spire_reborn::{
    Action, Card, Game, Stage,
    content::{Hero, card_spec},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Theme { Dark, Light }
impl Theme {
    fn name(self) -> &'static str { match self { Self::Dark => "暗色", Self::Light => "亮色" } }
    fn style(self) -> &'static str {
        match self { Self::Dark => "\x1b[97;40m", Self::Light => "\x1b[30;107m" }
    }
}
#[derive(Serialize, Deserialize)]
struct Preferences { theme: Theme }
struct App {
    game: Option<Game>,
    theme: Theme,
    dir: PathBuf,
    use_ansi: bool,
    message: String,
}
fn save_path(dir: &std::path::Path) -> PathBuf { dir.join("save.json") }
fn prefs_path(dir: &std::path::Path) -> PathBuf { dir.join("settings.json") }
fn atomic_write(path: &std::path::Path, data: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&temporary, data)?;
    fs::rename(temporary, path)?;
    Ok(())
}
fn default_dir() -> PathBuf {
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join("Library/Application Support/ThreeKingdomsSpireReborn")
}
fn card_label(card: &Card) -> String {
    match card_spec(&card.id) {
        Some(spec) => format!("{} [费{}] {}", card.name(), spec.cost, spec.description),
        None => format!("未知卡牌 {}", card.id),
    }
}
impl App {
    fn load(&mut self) -> Result<(), String> {
        let raw = fs::read_to_string(save_path(&self.dir)).map_err(|e| e.to_string())?;
        self.game = Some(Game::from_json(&raw)?);
        self.message = "读取进度成功，战斗牌堆和随机状态已恢复。".into();
        Ok(())
    }
    fn save(&self) -> Result<(), String> {
        let game = self.game.as_ref().ok_or("尚未开局")?;
        atomic_write(&save_path(&self.dir), &game.to_json()?).map_err(|e| e.to_string())
    }
    fn preferences(&self) -> Result<(), String> {
        let data = serde_json::to_string_pretty(&Preferences { theme: self.theme })
            .map_err(|e| e.to_string())?;
        atomic_write(&prefs_path(&self.dir), &data).map_err(|e| e.to_string())
    }
    fn draw(&self) {
        if self.use_ansi { print!("\x1b[2J\x1b[H{}", self.theme.style()); }
        println!(" ╔══════════════════════════════════════════════════╗");
        println!(" ║               三国登塔录 / TKS                  ║");
        println!(" ╚══════════════════════════════════════════════════╝");
        println!(" 主题：{}  |  操作：help  |  进度自动保存", self.theme.name());
        println!("────────────────────────────────────────────────────");
        if let Some(g) = &self.game {
            println!(" 武将：{}  体力：{}/{}  金：{}  药剂：{}",
                g.hero.name(), g.hp, g.max_hp, g.gold, g.potions);
            println!(" 幕：{}  层：{}/21  牌组：{}  遗物：{}  存档修订：{}",
                (g.floor.saturating_sub(1) / 7 + 1).min(3), g.floor, g.deck.len(),
                g.relics.join("、"), g.revision);
            println!("────────────────────────────────────────────────────");
            match g.stage {
                Stage::Map => {
                    println!(" 【征战路线】选择下一站：");
                    for (i, node) in g.route.iter().enumerate() {
                        println!("   {}. {}", i + 1, node.name());
                    }
                    println!(" 输入 choose <编号>");
                }
                Stage::Combat => {
                    let b = g.battle.as_ref().expect("stage validated");
                    println!(" 【战斗】第{}回合  军令：{}  护甲：{}  武勇：{}",
                        b.turn, b.energy, b.block, b.strength);
                    for (i, foe) in b.foes.iter().enumerate() {
                        println!("   敌{}：{}  {}/{}体力  甲{}  毒{}  意图：{}{}",
                            i + 1, foe.name(), foe.hp, foe.max_hp, foe.block, foe.poison,
                            foe.intent.label(), if foe.hp == 0 { " [阵亡]" } else { "" });
                    }
                    println!(" 【手牌】抽牌堆{} / 弃牌堆{} / 消耗{}",
                        b.draw.len(), b.discard.len(), b.exhausted.len());
                    for (i, card) in b.hand.iter().enumerate() {
                        println!("   {}. {}", i + 1, card_label(card));
                    }
                    println!(" 输入 play <手牌编号> [敌方编号] / end / use");
                }
                Stage::Reward => {
                    println!(" 【战利品】");
                    for (i, card) in g.rewards.iter().enumerate() {
                        println!("   {}. {}", i + 1, card_label(card));
                    }
                    println!(" 输入 choose <编号>（0 跳过）");
                }
                Stage::Market => {
                    println!(" 【军市】金：{}", g.gold);
                    for (i, card) in g.market.iter().enumerate() {
                        println!("   {}. {} / {}金", i + 1, card_label(card), 45 + i * 15);
                    }
                    println!(" 输入 choose <编号> 购买，choose 0 离开");
                }
                Stage::Camp => {
                    println!(" 【营寨】choose 1 休整回复30%最大体力；choose 2 升级一张卡牌");
                }
                Stage::Story => {
                    println!(" 【驿道奇遇】choose 1 以15金补给；choose 2 取火攻古卷（附带诅咒）；choose 3 损失8体力夺65金");
                }
                Stage::Won => println!(" 【胜利】三幕皆定，江山已平！输入 new 开始新征程。"),
                Stage::Lost => println!(" 【败北】胜败乃兵家常事。输入 new 开始新征程。"),
            }
            println!("────────────────────────────────────────────────────");
            for msg in g.log.iter().rev().take(3).rev() { println!(" > {msg}"); }
        } else {
            println!(" 尚无存档。输入 new [guanyu|zhuge|sun|cao|lubu] [随机种子] 开始。");
        }
        if !self.message.is_empty() { println!(" 提示：{}", self.message); }
        if self.use_ansi { print!("\x1b[0m"); }
        print!("\n三国登塔录 > ");
        let _ = io::stdout().flush();
    }
    fn command(&mut self, raw: &str) -> bool {
        let args: Vec<&str> = raw.split_whitespace().collect();
        if args.is_empty() { return true; }
        let parsed = |idx: usize, default: usize| -> Result<usize, String> {
            match args.get(idx) {
                Some(value) => value.parse::<usize>().map_err(|_| "请输入有效非负整数".into()),
                None => Ok(default),
            }
        };
        let result: Result<String, String> = match args[0].to_ascii_lowercase().as_str() {
            "quit" | "exit" | "q" => { let _ = self.save(); return false; }
            "help" | "h" => Ok("new [武将] [seed] 新局；load 读档；save 保存；choose N 选择；play N [目标] 出牌；end 结束回合；use 饮药；theme dark|light 主题；status 状态；quit 退出。编号从1开始。".into()),
            "new" => {
                let hero = args.get(1).and_then(|s| Hero::parse(s)).unwrap_or(Hero::GuanYu);
                let seed = args.get(2).and_then(|s| s.parse::<u64>().ok()).unwrap_or_else(|| {
                    SystemTime::now().duration_since(UNIX_EPOCH).map_or(1, |t| t.as_secs().max(1))
                });
                self.game = Some(Game::new(hero, seed));
                self.save().map(|_| format!("以 {} 开始新征程，种子 {seed}。", hero.name()))
            }
            "load" => self.load().map(|_| "已读档。".into()),
            "save" => self.save().map(|_| "已保存。".into()),
            "theme" => match args.get(1).copied() {
                Some("light") => { self.theme = Theme::Light; self.preferences().map(|_| "亮色主题已保存。".into()) }
                Some("dark") => { self.theme = Theme::Dark; self.preferences().map(|_| "暗色主题已保存。".into()) }
                _ => Err("用法：theme light 或 theme dark".into()),
            },
            "status" => Ok(self.game.as_ref().map_or("尚未开局".into(), |g| {
                format!("{}，第{}层，体力{}/{}，{}金，{}张卡牌，版本{}。",
                    g.hero.name(), g.floor, g.hp, g.max_hp, g.gold, g.deck.len(), g.version)
            })),
            "choose" | "c" | "play" | "p" | "end" | "use" => {
                let action = match args[0] {
                    "choose" | "c" => parsed(1, 0).map(|index| Action::Choose { index }),
                    "play" | "p" => parsed(1, 0).and_then(|index| parsed(2, 1)
                        .map(|target| Action::Play { index, target })),
                    "end" => Ok(Action::EndTurn),
                    _ => Ok(Action::UsePotion),
                };
                action.and_then(|action| {
                    let messages = self.game.as_mut().ok_or("请先输入 new 开局")?.apply(action)?;
                    self.save()?;
                    Ok(messages.join("\n"))
                })
            }
            _ => Err("未知指令。输入 help 查看操作。".into()),
        };
        self.message = match result { Ok(msg) => msg, Err(err) => format!("错误：{err}") };
        true
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("三国登塔录 TKS — macOS Rust terminal roguelike\n用法：tks [--plain] [--save-dir <目录>]\n游戏内输入 help 查看指令。");
        return;
    }
    let mut dir = default_dir();
    if let Some(i) = args.iter().position(|a| a == "--save-dir")
        && let Some(value) = args.get(i + 1)
    {
        dir = PathBuf::from(value);
    }
    let theme = fs::read_to_string(prefs_path(&dir)).ok()
        .and_then(|s| serde_json::from_str::<Preferences>(&s).ok())
        .map_or(Theme::Dark, |settings| settings.theme);
    let mut app = App {
        game: None, theme, dir,
        use_ansi: io::stdout().is_terminal() && !args.iter().any(|a| a == "--plain"),
        message: String::new(),
    };
    if save_path(&app.dir).exists()
        && let Err(err) = app.load()
    {
        app.message = format!("存档未能加载：{err}；可输入 new 开新局。");
    }
    loop {
        app.draw();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => { let _ = app.save(); break; }
            Ok(_) => if !app.command(input.trim()) { break; },
            Err(e) => { eprintln!("输入错误：{e}"); break; }
        }
    }
}
