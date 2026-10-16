//! macOS terminal frontend. All rules remain in the library.
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use three_kingdoms_spire_reborn::{
    Action, Card, Game, RunMode, Stage,
    content::{Hero, card_spec},
    items::potion_name,
    progression::Profile,
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
    profile: Profile,
    theme: Theme,
    dir: PathBuf,
    use_ansi: bool,
    message: String,
}
fn save_path(dir: &std::path::Path) -> PathBuf { dir.join("save.json") }
fn prefs_path(dir: &std::path::Path) -> PathBuf { dir.join("settings.json") }
fn profile_path(dir: &std::path::Path) -> PathBuf { dir.join("chronicle.json") }
fn atomic_write(path: &std::path::Path, data: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&temporary, data)?;
    fs::rename(temporary, path)?;
    Ok(())
}
fn new_run_id() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("cli-{}-{nanos}",std::process::id())
}
fn default_dir() -> PathBuf {
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    home.join("Library/Application Support/ThreeKingdomsSpireReborn")
}
fn card_label(card: &Card) -> String {
    match card_spec(&card.id) {
        Some(spec) => format!("{} [费{}] {}{}{}{}",
            card.name(),
            if card.enchantment.as_deref() == Some("迅捷") { (spec.cost - 1).max(0) } else { spec.cost },
            if spec.retain { "[保留] " } else { "" },
            if spec.ethereal { "[虚无] " } else { "" },
            spec.description,
            if card.id == "quest_tulu" { format!("，进度{}/3", card.quest_progress) } else { String::new() }),
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

    fn save_profile(&self) -> Result<(), String> {
        let json = self.profile.to_json()?;
        atomic_write(&profile_path(&self.dir), &json).map_err(|e| e.to_string())
    }
    fn track_action(&mut self) -> Result<(), String> {
        let g = self.game.as_ref().ok_or("请先开局")?;
        self.profile.observe(g);
        if matches!(g.stage, Stage::Won | Stage::Lost) {
            let key = if g.run_id.is_empty() {
                format!("legacy:{}:{}:{}:{}:{:?}",g.hero.name(),g.seed,g.floor,g.revision,g.stage)
            } else { g.run_id.clone() };
            self.profile.record(g,key)?;
        }
        self.save_profile()
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
            println!(" 模式：{:?}  飞升难度：{}  已完成战役：{}  存档协议：V{}",
                g.mode, g.ascension, g.victories, g.version);
            println!(" 章节：{}；变体：{}",
                g.act_name(),
                g.variants.map_or("随种子".into(),|choices| format!("{}/{}/{}",choices[0],choices[1],choices[2])));
            if !g.boons.is_empty() {
                println!(" 先贤赐福：{}",g.boons.iter().map(|id|
                    three_kingdoms_spire_reborn::items::boon_name(id)).collect::<Vec<_>>().join("、"));
            }
            println!(" 战功簿：{}局 {}胜 {}负；最高层{}；最佳得分{}",
                self.profile.started, self.profile.wins, self.profile.losses,
                self.profile.best_floor, self.profile.best_score);
            for (i,p) in g.potion_bag.iter().enumerate() {
                println!("   药剂{}：{}", i + 1, potion_name(p));
            }
            println!("────────────────────────────────────────────────────");
            match g.stage {
                Stage::Map => {
                    println!(" 【征战路线】选择下一站：");
                    for (i, node) in g.route.iter().enumerate() {
                        let lane = g.route_lanes.get(i).map_or(i + 1, |n| usize::from(*n)+1);
                        println!("   {}. {}（路径{}）", i + 1, node.name(),lane);
                    }
                    println!(" 输入 map 查看本幕七层完整路线图");
                    println!(" 输入 choose <编号>");
                }
                Stage::Combat => {
                    let b = g.battle.as_ref().expect("stage validated");
                    println!(" 【战斗】第{}回合  军令：{}  护甲：{}  武勇：{}",
                        b.turn, b.energy, b.block, b.strength);
                    for (i, foe) in b.foes.iter().enumerate() {
                        println!("   敌{}：{}  {}/{}体力  甲{}  毒{}  意图：{}{}",
                            i + 1, foe.name(), foe.hp, foe.max_hp, foe.block, foe.poison,
                            foe.intent.label(), if foe.hp == 0 { " [阵亡]" } else if foe.phase > 0 { " [二阶段]" } else { "" });
                    }
                    println!(" 【职业资源】战意{} 谋略{} 连段{} 统御{} 怒气{} 反击{}",
                        b.momentum,b.scheme,b.combo,b.command,b.fury,b.thorns);
                    if b.troops_hp > 0 {
                        println!(" 【亲兵】体力{} 自动攻击{} / 回合",b.troops_hp,b.troops_attack);
                    }
                    if b.discard_tokens > 0 {
                        println!(" 【弃换】剩余{}次，discard N 弃换手牌并抽1张",b.discard_tokens);
                    }
                    if !b.powers.is_empty() {
                        println!(" 【持续军略】{}", b.powers.iter().map(|p|
                            format!("{}×{}",p.id.name(),p.stacks)).collect::<Vec<_>>().join("、"));
                    }
                    if let Some(event) = b.events.last() {
                        println!(" 【战斗结算】#{:03} {:?} 来源={} 数值={}",
                            event.seq,event.kind,event.source,event.amount);
                    }
                    println!(" 【手牌】抽牌堆{} / 弃牌堆{} / 消耗{}",
                        b.draw.len(), b.discard.len(), b.exhausted.len());
                    for (i, card) in b.hand.iter().enumerate() {
                        println!("   {}. {}", i + 1, card_label(card));
                    }
                    println!(" 输入 play <手牌编号> [敌方编号] / end / use [药剂编号] [敌方编号]");
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
                    println!("   4. 随机药剂 / 55金");
                    println!("   5. 遗物 {} / 155金",
                        g.shop_relic.as_deref().unwrap_or("已售罄"));
                    println!("   6. 移除诅咒或普通突刺 / 75金");
                    println!(" 输入 choose <编号> 购买，remove <牌组序号> 定向移牌，choose 0 离开");
                }
                Stage::Camp => {
                    println!(" 【营寨】choose 1 休整；choose 2 升级首张牌；choose 3 操练+2生命上限");
                    println!(" upgrade N 定向升级；enchant N 迅捷|锐锋|护身|明悟（35金）");
                }
                Stage::Story => {
                    println!(" 【驿道奇遇】{}", g.lore_hint());
                }
                Stage::Ancient => {
                    println!(" 【先贤赐福】此役平定，选择一项长期军略：");
                    for (i,boon) in g.boon_choices.iter().enumerate() {
                        println!("   {}. {}", i+1,three_kingdoms_spire_reborn::items::boon_name(boon));
                    }
                    println!(" 输入 choose <编号>");
                }
                Stage::Won => println!(" 【胜利】三幕皆定，江山已平！输入 new 开始新征程。"),
                Stage::Lost => println!(" 【败北】胜败乃兵家常事。输入 new 开始新征程。"),
            }
            println!("────────────────────────────────────────────────────");
            for msg in g.log.iter().rev().take(3).rev() { println!(" > {msg}"); }
        } else {
            println!(" 尚无存档。输入 new [guanyu|zhuge|sun|cao|lubu] [随机种子] [飞升0-10] 开始。");
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
            "help" | "h" => Ok("new [武将] [seed] [已解锁难度]，custom [武将] [seed] [0-10] 自由模式；variant 0|1 0|1 0|1 在出征前指定三幕变体；daily YYYY-MM-DD；load/save；choose N；play N [目标]；discard N 弃换；end；use [药剂号] [目标]；upgrade N；enchant N 迅捷|锐锋|护身|明悟；remove N；deck；map 查看连通路线；relics；lore 看纪事；bonds 看人物羁绊；chapter 1-120 回目索引；powers 持续能力；events 最近战斗结算；chronicle；theme dark|light；status；quit。".into()),
            "new" | "custom" => {
                let mode = if args[0] == "custom" {RunMode::Custom} else {RunMode::Standard};
                let hero = args.get(1).and_then(|s| Hero::parse(s)).unwrap_or(Hero::GuanYu);
                let seed = args.get(2).and_then(|s| s.parse::<u64>().ok()).unwrap_or_else(|| {
                    SystemTime::now().duration_since(UNIX_EPOCH).map_or(1, |t| t.as_secs().max(1))
                });
                let ascension = args.get(3).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0).min(10);
                let unlocked = self.profile.next_ascension(hero);
                if mode == RunMode::Standard && ascension > unlocked {
                    Err(format!("{}目前标准战役仅解锁飞升0~{unlocked}；通关该武将当前难度可解锁下一级。测试自由档请用 custom。",hero.name()))
                } else {
                    self.game = Some(Game::new_with_identity(hero,seed,ascension,mode,new_run_id()));
                    let g = self.game.as_ref().expect("just created");
                    self.profile.on_start(g);
                    self.save().and_then(|_| self.save_profile())
                        .map(|_| format!("以 {} 开始{:?}征程，种子 {seed}，难度 {ascension}。", hero.name(),mode))
                }
            }
            "daily" => {
                match args.get(1).copied() {
                    Some(date) if date.len() == 10 && date.chars().enumerate()
                        .all(|(i,c)| if i==4||i==7 { c=='-' } else { c.is_ascii_digit() }) => {
                        let seed = date.bytes().fold(0xcbf29ce484222325_u64, |h,c|
                            (h ^ u64::from(c)).wrapping_mul(0x100000001b3));
                        let heroes = [Hero::GuanYu,Hero::ZhugeLiang,Hero::SunShangxiang,Hero::CaoCao,Hero::LuBu];
                        let hero = heroes[(seed % heroes.len() as u64) as usize];
                        self.game = Some(Game::new_with_identity(hero,seed,3,RunMode::Daily,new_run_id()));
                        let g = self.game.as_ref().expect("just created");
                        self.profile.on_start(g);
                        self.save().and_then(|_| self.save_profile())
                            .map(|_| format!("每日挑战 {date}：{}，难度3，种子 {seed}",hero.name()))
                    }
                    _ => Err("每日挑战请输入 YYYY-MM-DD（如 daily 2026-10-16）".into())
                }
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
            "map" => Ok(self.game.as_ref().map_or("尚未开局".into(),|g| {
                g.act_map.as_ref().map_or("当前存档没有已生成的地图".into(),|graph| {
                    graph.rows.iter().enumerate().map(|(r,row)| {
                        let cells = row.iter().enumerate().map(|(lane,n)| {
                            let current = g.floor == graph.act*7 + r as u8 + 1
                                && g.map_lane == Some(lane as u8);
                            format!("{}{}{}",if current {"["} else {""},
                                n.name(),if current {"]"} else {""})
                        }).collect::<Vec<_>>().join("  ─  ");
                        format!("第{:02}层  {}",graph.act*7+r as u8+1,cells)
                    }).collect::<Vec<_>>().join("\n")
                })
            })),
            "deck" => Ok(self.game.as_ref().map_or("尚未开局".into(), |g| {
                g.deck.iter().enumerate().map(|(i,c)| format!("{}. {}", i+1, card_label(c))).collect::<Vec<_>>().join("\n")
            })),
            "chronicle" => Ok(format!("战役{}次，胜利{}次，失败{}次，最高层{}，最高分{}，卡牌图鉴{}，敌军图鉴{}，遗物图鉴{}。\n{}",
                self.profile.started,self.profile.wins,self.profile.losses,self.profile.best_floor,
                self.profile.best_score,self.profile.seen_cards.len(),self.profile.seen_enemies.len(),
                self.profile.discovered_relics.len(),
                self.profile.records.iter().rev().take(5).map(|r| format!("{}: {} {}层 评分{} 难度{}",
                    if r.won {"胜"} else {"败"},r.hero.name(),r.floor,r.score,r.ascension)).collect::<Vec<_>>().join("\n")
            )),
            "bonds" | "fate" => Ok(self.game.as_ref().map_or("尚未开局".into(), |g| {
                let records = g.active_bonds();
                if records.is_empty() {
                    "暂无已结缘人物；在演义奇遇中选择守诺，后续羁绊才会生效。".into()
                } else {
                    records.iter().map(|bond| format!("{} ↔ {}【{}】（原著第{}回）{}",
                        g.hero.name(),bond.companion,bond.relation.name(),bond.chapter,bond.note))
                        .collect::<Vec<_>>().join("\n")
                }
            })),
            "lore" => Ok(self.game.as_ref().map_or("尚未开局".into(), |g| {
                let mut out = format!("演义纪事：已见{}则，已结{}段因果，人物羁绊{}项。\n",
                    g.lore_seen.len(),g.lore_flags.len(),g.active_bonds().len());
                for id in &g.lore_seen {
                    if let Some(b) = three_kingdoms_spire_reborn::lore::beat(id) {
                        let chosen = if g.lore_flags.iter().any(|flag| flag == b.flag) {"立约"} else {"未立约"};
                        out.push_str(&format!("第{:03}回 {}【{}】{}\n",b.chapter,b.title,chosen,
                            three_kingdoms_spire_reborn::lore::provenance_url(b.chapter)));
                    }
                }
                let hidden = three_kingdoms_spire_reborn::lore::BEATS.iter()
                    .filter(|b| b.prerequisite.is_some_and(|key| g.lore_flags.iter().any(|flag| flag == key))
                        && !g.lore_seen.iter().any(|seen| seen == b.id)).count();
                out.push_str(&format!("尚有{hidden}项未揭开的因果（不剧透）\n"));
                for id in &g.easter_unlocked {
                    if let Some(egg) = three_kingdoms_spire_reborn::lore::EASTER_EGGS.iter().find(|e| e.id == id) {
                        out.push_str(&format!("隐藏结局【{}】— {}\n",egg.name,egg.hint));
                    }
                }
                out
            })),
            "chapter" => match args.get(1).and_then(|raw| raw.parse::<usize>().ok()) {
                Some(index) if (1..=120).contains(&index) => {
                    let line = include_str!("../data/novel_chapters.tsv").lines()
                        .filter(|line| line.chars().next().is_some_and(|ch| ch.is_ascii_digit()))
                        .nth(index-1).ok_or_else(|| "章节索引不完整".to_string());
                    line.map(|raw| {
                        let parts: Vec<_> = raw.split('\t').collect();
                        let events = three_kingdoms_spire_reborn::lore::BEATS.iter()
                            .filter(|beat| usize::from(beat.chapter) == index)
                            .map(|beat| beat.title).collect::<Vec<_>>();
                        format!("第{index:03}回：{}\n原典：{}\n本游戏关联的演义故事：{}",
                            parts.get(1).unwrap_or(&""),parts.get(2).unwrap_or(&""),
                            if events.is_empty() { "尚无已编写场景".into() } else { events.join("、") })
                    })
                }
                _ => Err("用法：chapter 1～120 查看原著回目索引和对应游戏章节".into()),
            },
            "powers" => Ok(self.game.as_ref().and_then(|g|g.battle.as_ref())
                .map_or("当前没有战斗中的持续能力。".into(),|b| {
                    if b.powers.is_empty() { "尚未激活持续军略。".into() }
                    else { b.powers.iter().map(|p|
                        format!("{}×{}：{}",p.id.name(),p.stacks,p.id.description()))
                        .collect::<Vec<_>>().join("\n") }
                })),
            "events" => Ok(self.game.as_ref().and_then(|g|g.battle.as_ref())
                .map_or("当前没有可用的战斗事件记录。".into(),|b| {
                    if b.events.is_empty() { "尚无事件记录。".into() }
                    else { b.events.iter().rev().take(30).rev().map(|e|
                        format!("#{} {:?} 来源:{} 结算:{} 目标:{}",
                            e.seq,e.kind,e.source,e.amount,e.target.as_deref().unwrap_or("-")))
                        .collect::<Vec<_>>().join("\n") }
                })),
            "relics" => Ok(self.game.as_ref().map_or("尚未开局".into(), |g| {
                g.relics.iter().map(|r| format!("{}：{}",r, three_kingdoms_spire_reborn::items::relic_name(r))).collect::<Vec<_>>().join("\n")
            })),
            "choose" | "c" | "play" | "p" | "end" | "use" | "upgrade" | "remove" | "enchant" | "discard" | "variant" => {
                let action = match args[0] {
                    "choose" | "c" => parsed(1, 0).map(|index| Action::Choose { index }),
                    "play" | "p" => parsed(1, 0).and_then(|index| parsed(2, 1)
                        .map(|target| Action::Play { index, target })),
                    "end" => Ok(Action::EndTurn),
                    "upgrade" => parsed(1, 0).map(|index| Action::Upgrade { index }),
                    "remove" => parsed(1, 0).map(|index| Action::Remove { index }),
                    "discard" => parsed(1, 0).map(|index| Action::Discard { index }),
                    "variant" => parsed(1, 9).and_then(|act1| parsed(2, 9)
                        .and_then(|act2| parsed(3, 9).map(|act3|
                            Action::SetVariants { act1: act1 as u8, act2: act2 as u8, act3: act3 as u8 }))),
                    "enchant" => parsed(1, 0).and_then(|index|
                        args.get(2).map_or(Err("请指定附魔类型".into()),|kind|
                            Ok(Action::Enchant { index, kind: (*kind).to_owned() }))),
                    _ => parsed(1, 1).and_then(|slot| parsed(2, 1)
                        .map(|target| Action::UsePotionAt { slot, target })),
                };
                action.and_then(|action| {
                    let messages = self.game.as_mut().ok_or("请先输入 new 开局")?.apply(action)?;
                    self.save()?;
                    self.track_action()?;
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
    let profile = fs::read_to_string(profile_path(&dir)).ok()
        .and_then(|s| Profile::from_json(&s).ok())
        .unwrap_or_default();
    let mut app = App {
        game: None, profile, theme, dir,
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
