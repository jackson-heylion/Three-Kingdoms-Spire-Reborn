//! macOS terminal frontend. All rules remain in the library.
mod tui;
mod hud;
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
    persistence::{SaveDirectoryLock, atomic_write, restore_primary_from_backup},
    challenges::ChallengeModifier,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Theme { Dark, Light, HighContrast }
impl Theme {
    fn name(self) -> &'static str { match self { Self::Dark => "暗色", Self::Light => "亮色", Self::HighContrast => "高对比度" } }
    fn style(self) -> &'static str {
        match self { Self::Dark => "\x1b[97;40m", Self::Light => "\x1b[30;107m", Self::HighContrast => "\x1b[1;97;40m" }
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
fn new_run_id() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("cli-{}-{nanos}",std::process::id())
}
fn default_dir() -> PathBuf {
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    #[cfg(target_os = "macos")]
    { home.join("Library/Application Support/ThreeKingdomsSpireReborn") }
    #[cfg(not(target_os = "macos"))]
    {
        // Prefer XDG on Linux; never silently strand saves written by earlier
        // releases at the former macOS-style path.
        let legacy = home.join("Library/Application Support/ThreeKingdomsSpireReborn");
        let parent = env::var_os("XDG_DATA_HOME").map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| home.join(".local/share"));
        let modern = parent.join("ThreeKingdomsSpireReborn");
        if !modern.exists() && legacy.is_dir() { legacy } else { modern }
    }
}
fn card_label(card: &Card) -> String {
    match card_spec(&card.id) {
        Some(spec) => format!("{} [费{}] {}{}{}{}",
            card.name(),
            if spec.cost == -1 { "X".to_string() }
            else if card.enchantment.as_deref() == Some("迅捷") { (spec.cost - 1).max(0).to_string() }
            else { spec.cost.to_string() },
            if spec.retain { "[保留] " } else { "" },
            if spec.ethereal { "[虚无] " } else { "" },
            spec.description,
            if card.id == "quest_tulu" { format!("，进度{}/3", card.quest_progress) } else { String::new() }),
        None => format!("未知卡牌 {}", card.id),
    }
}
impl App {
    fn load(&mut self) -> Result<(), String> {
        let path=save_path(&self.dir);
        let current=fs::read_to_string(&path)
            .map_err(|e|e.to_string()).and_then(|raw|Game::from_json(&raw));
        match current {
            Ok(game)=>{
                self.game=Some(game);
                self.message="读取进度成功，战斗牌堆和随机状态已恢复。".into();
                Ok(())
            }
            Err(original_error)=>{
                let backup=path.with_extension("bak");
                let fallback=fs::read_to_string(&backup)
                    .map_err(|e|e.to_string())
                    .and_then(|raw|Game::from_json(&raw).map(|game| (game,raw)));
                match fallback {
                    Ok((game,verified_raw))=>{
                        // Preserve the damaged source for manual inspection;
                        // do not overwrite the last verified good backup.
                        if path.exists() {
                            fs::rename(&path,path.with_extension("corrupt"))
                                .map_err(|e|format!("无法保留受损存档: {e}"))?;
                        }
                        restore_primary_from_backup(&path,&verified_raw)
                            .map_err(|e|format!("备份有效，但恢复到主路径失败: {e}"))?;
                        self.game=Some(game);
                        self.message="主存档异常，已从上一次有效 .bak 备份恢复；损坏原文件已保留为 .corrupt。".into();
                        Ok(())
                    }
                    Err(backup_error)=>Err(format!("主存档错误：{original_error}；备份恢复失败：{backup_error}"))
                }
            }
        }
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
            println!(" {}",g.doctrine_status());
            if !g.modifiers.is_empty() {
                println!(" 【自由挑战】{}", g.modifiers.iter().map(|m| m.name())
                    .collect::<Vec<_>>().join("、"));
            }
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
                    let next_row=usize::from(g.floor%7);
                    for (i, node) in g.route.iter().enumerate() {
                        let lane = g.route_lanes.get(i).map_or(i as u8,|n|*n);
                        let (terrain,risk)=g.act_map.as_ref().map_or(
                            ("平道".to_string(),None),|map|(
                                map.terrain_at(next_row,lane).label().to_owned(),
                                Some(map.score_route(next_row,lane,g.hp,g.max_hp,g.gold))));
                        println!("   {}. {}（道{} / {} / 三层路线评分{}）",
                            i+1,node.name(),lane+1,terrain,risk.map_or("-".into(),|score|score.to_string()));
                    }
                    println!(" 输入 map 查看本幕七层完整路线图");
                    println!(" 输入 choose <编号>");
                }
                Stage::Combat => {
                    let b = g.battle.as_ref().expect("stage validated");
                    println!(" 【战斗】第{}回合  军令：{}  护甲：{}  武勇：{}",
                        b.turn, b.energy, b.block, b.strength);
                    if b.thorns > 0 {
                        println!(" 【反击】每次受击反伤{}；击杀施术者可中断尚未生效的后续战技。", b.thorns);
                    }
                    if b.turn>=35 {
                        let notice=if b.turn<40 {
                            format!("距军粮枯竭还剩{}个回合",40-b.turn)
                        } else {
                            format!("军粮枯竭：本回合结束时无视护甲失去{}体力，之后每4回合增加1点",
                                1+(b.turn-40)/4)
                        };
                        println!(" 【持久战警报】{notice}");
                    }
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
                    println!(" 输入 play <手牌编号> [敌方编号] [副选牌编号] / selections <手牌编号> 查看合法副牌 / piles / end / use");
                }
                Stage::Reward => {
                    println!(" 【战利品】");
                    for (i, card) in g.rewards.iter().enumerate() {
                        println!("   {}. {}", i + 1, card_label(card));
                    }
                    println!(" 输入 choose <编号>（0 跳过）");
                }
                Stage::Market => {
                    println!(" 【军市】金：{}（购买卡牌将永久加入牌组）", g.gold);
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
                    if g.modifiers.contains(&ChallengeModifier::NoRest) {
                        println!(" 【营寨·行军不息】休整被禁用；choose 2 升级；choose 3 操练+2生命上限");
                    } else {
                        println!(" 【营寨】choose 1 休整；choose 2 升级首张牌；choose 3 操练+2生命上限");
                    }
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
            "quit" | "exit" | "q" => {
                if self.game.is_none() { return false; }
                match self.save() {
                    Ok(()) => return false,
                    Err(error) => Err(format!(
                        "退出前保存失败：{error}；游戏仍在运行，请修复存档目录后重试 save 或 quit"
                    )),
                }
            }
            "help" | "h" => Ok("new [武将] [seed] [已解锁难度]，custom [武将] [seed] [0-10] 自由模式；variant 0|1 0|1 0|1 在出征前指定三幕变体；daily YYYY-MM-DD；load/save；choose N；play N [目标] [副选牌编号]；selections N 查可选副牌；discard N 弃换；end；use [药剂号] [目标]（火/毒指定存活目标，弱化影响全体） ；upgrade N；enchant N 迅捷|锐锋|护身|明悟；remove N；deck；map 查看路径与评分；inventory 背包；achievements 成就；codex 图鉴；relics；lore 看纪事；bonds 看人物羁绊；chapter 1-120 回目索引；powers 持续能力；events [游标] [条数1-80] 分页审计当前战斗事件；metrics 战役统计；verify 回放检验；history 最近操作；doctrine 查看流派 / doctrine auto|代码 选择；chronicle；timeline 战功时间线；modifier 查看挑战 / modifier 代码 on|off；theme dark|light|highcontrast；status；quit。".into()),
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
                Some("highcontrast") => { self.theme = Theme::HighContrast; self.preferences().map(|_| "高对比度主题已保存。".into()) }
                _ => Err("用法：theme light|dark|highcontrast".into()),
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
                            let ground=graph.terrain_at(r,lane as u8);
                            if ground==three_kingdoms_spire_reborn::map::Terrain::Closed {
                                return "× 断道".into();
                            }
                            format!("{}{}·{}{}",if current {"["} else {""},
                                n.name(),ground.label(),if current {"]"} else {""})
                        }).collect::<Vec<_>>().join("  ─  ");
                        format!("第{:02}层  {}",graph.act*7+r as u8+1,cells)
                    }).collect::<Vec<_>>().join("\n")
                })
            })),
            "piles" => Ok(self.game.as_ref().and_then(|g| g.battle.as_ref())
                .map_or("当前没有战斗中的可选牌堆。".into(), |b| {
                    format!("【抽牌堆·顶牌优先】\n{}\n【弃牌堆·最新优先】\n{}",
                        b.draw.iter().rev().enumerate()
                            .map(|(i,c)|format!("{}. {}",i+1,card_label(c)))
                            .collect::<Vec<_>>().join("\n"),
                        b.discard.iter().rev().enumerate()
                            .map(|(i,c)|format!("{}. {}",i+1,card_label(c)))
                            .collect::<Vec<_>>().join("\n"))
                })),
            "doctrine" if args.len()==1 => Ok(self.game.as_ref()
                .map_or("请先输入 new 开局".into(),|g|g.doctrine_status())),
            "verify" => self.game.as_ref().ok_or_else(||"请先开局".to_owned())
                .and_then(|game|game.verify_action_replay()
                    .map(|count|format!("全战役操作回放校验通过，共{count}步；随机数、地图、卡牌、战斗事件和剧情结果一致。"))),
            "history" => Ok(self.game.as_ref().map_or("尚未开局".into(),|g|{
                g.action_history.as_ref().map_or("旧存档缺少完整操作历史".into(),|actions|
                    format!("累计{}次成功操作。最近15步：\n{}",actions.len(),
                        actions.iter().enumerate().rev().take(15).collect::<Vec<_>>()
                        .iter().rev().map(|(idx,a)|format!("{}. {a:?}",idx+1))
                        .collect::<Vec<_>>().join("\n")))
            })),
            "inventory" | "bag" => Ok(self.game.as_ref().map_or("请先开局".into(),|g|{
                let equipment=if g.relics.is_empty(){"无".into()}else{g.relics.iter().map(|r|
                    format!("{r}：{}",three_kingdoms_spire_reborn::items::relic_name(r)))
                    .collect::<Vec<_>>().join("\n")};
                let potions=if g.potion_bag.is_empty(){"无".into()}else{g.potion_bag.iter()
                    .enumerate().map(|(i,p)|format!("{}. {}",i+1,potion_name(p)))
                    .collect::<Vec<_>>().join("\n")};
                let boons=if g.boons.is_empty(){"无".into()}else{g.boons.iter().map(|b|
                    three_kingdoms_spire_reborn::items::boon_name(b)).collect::<Vec<_>>().join("\n")};
                format!("【全局背包】体力{}/{}，金币{}。\n【遗物】\n{}\n【药剂】\n{}\n【先贤赐福】\n{}",
                    g.hp,g.max_hp,g.gold,equipment,potions,boons)
            })),
            "achievements" | "trophies" => Ok({
                let lines=self.profile.achievement_progress().into_iter()
                    .map(|(_,name,desc,won)|format!("{} {} —— {}",if won {"[已达成]"}else{"[未达成]"},name,desc))
                    .collect::<Vec<_>>().join("\n");
                format!("【战功成就】已解锁 {}/{}\n{}",
                    self.profile.achievements.len(),self.profile.achievement_progress().len(),lines)
            }),
            "timeline" => Ok({
                let milestones = [
                    ("沙场初胜", self.profile.achievements.contains("first_battle")),
                    ("天下归心", self.profile.achievements.contains("first_clear")),
                    ("三次凯旋", self.profile.wins >= 3),
                ];
                let progress = milestones.iter().enumerate().map(|(index, (name, won))|
                    format!("{}. {} {}", index + 1, if *won {"[已完成]"} else {"[未完成]"}, name))
                    .collect::<Vec<_>>().join("\n");
                let trophies = ChallengeModifier::ALL.iter().map(|modifier| {
                    let unlocked = self.profile.can_use_modifier(*modifier);
                    let collected = self.profile.challenge_trophies.contains(modifier.id());
                    format!("{}【{}】{}：{}", if collected {"[已收藏]"} else if unlocked {"[可挑战]"} else {"[未解锁]"},
                        modifier.name(), modifier.id(), modifier.description())
                }).collect::<Vec<_>>().join("\n");
                format!("【战役时间线】\n{progress}\n【永久挑战藏品】\n{trophies}")
            }),
            "modifier" | "modifiers" if args.len() == 1 => Ok({
                let active = self.game.as_ref().map_or("无", |game| if game.modifiers.is_empty() {"无"} else {"已启用"});
                let options = ChallengeModifier::ALL.iter().map(|modifier| {
                    let status = if self.profile.can_use_modifier(*modifier) {"可用"} else {"未解锁"};
                    let in_use = self.game.as_ref().is_some_and(|game| game.modifiers.contains(modifier));
                    format!("{} {}【{}】—— {}{}", status, modifier.name(), modifier.id(), modifier.description(),
                        if in_use {" [启用中]"} else {""})
                }).collect::<Vec<_>>().join("\n");
                format!("【自由挑战修饰符】当前：{active}\n{options}\n用法：custom 开局后输入 modifier 代码 on|off（仅第一层之前）；标准/每日模式不可使用。")
            }),
            "codex" => Ok({
                let cards=self.profile.seen_cards.iter().map(|id|{
                    let card=Card::new(id);
                    format!("{} ({id})：{}", card.name(), card_spec(id).map_or("未知描述", |spec| spec.description))
                }).collect::<Vec<_>>().join("、");
                let foes=self.profile.seen_enemies.iter().map(|id|
                    three_kingdoms_spire_reborn::content::enemy_name(id)
                ).collect::<Vec<_>>().join("、");
                let relics=self.profile.discovered_relics.iter().map(|id|
                    format!("{}：{}", id, three_kingdoms_spire_reborn::items::relic_name(id)))
                    .collect::<Vec<_>>().join("\n");
                format!("【图鉴】卡牌 {}/{}：{}\n敌军 {}/{}：{}\n遗物 {}/{}：{}",
                    self.profile.seen_cards.len(),
                    three_kingdoms_spire_reborn::content::card_count(),cards,
                    self.profile.seen_enemies.len(),
                    three_kingdoms_spire_reborn::content::ENEMY_IDS.len(),foes,
                    self.profile.discovered_relics.len(),
                    three_kingdoms_spire_reborn::items::RELICS.len(),relics)
            }),
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
            "metrics" => Ok(self.game.as_ref().map_or("尚未开局".into(),|g| {
                let c=&g.telemetry.totals;
                let recent=g.telemetry.recent_turns.iter().rev().take(8).rev()
                    .map(|t| format!("第{}层/回合{}{}：亲兵攻击{}、实际伤害{}、护卫承伤{}、统御触发{}、孙尚香弃换{}、连段{}、英雄生命损失{}",
                        t.floor,t.turn,if t.completed {""} else {"（未结束）"},
                        t.counters.troop_attacks,t.counters.troop_damage,t.counters.troop_absorbed,
                        t.counters.command_procs,t.counters.sun_discards,t.counters.sun_combo_procs,
                        t.counters.hero_hp_damage))
                    .collect::<Vec<_>>().join("\n");
                format!("战役累计：遭遇{}／胜{}／败{}；已结算{}回合；出牌{}；英雄生命损失{}。\n                    亲兵：行动{}、实际输出{}、吸收伤害{}。\n                    统御：触发{}、额外军令{}、护甲{}。\n                    孙尚香：主动弃换{}次／护甲{}，三连触发{}次／护甲{}／实际额外抽牌{}张。\n最近回合：\n{}",
                    c.battles_started,c.battles_won,c.battles_lost,c.turns_completed,
                    c.cards_played,c.hero_hp_damage,c.troop_attacks,c.troop_damage,
                    c.troop_absorbed,c.command_procs,c.command_energy,c.command_block,
                    c.sun_discards,c.sun_discard_block,c.sun_combo_procs,c.sun_combo_block,
                    c.sun_combo_draws,if recent.is_empty(){"暂无回合记录"}else{&recent})
            })),
            "events" => (|| -> Result<String, String> {
                if args.len() > 3 {
                    return Err("用法：events [游标] [条数1-80]".into());
                }
                let cursor = args.get(1).map(|value| value.parse::<u64>()
                    .map_err(|_| "事件游标必须为非负整数".to_owned())).transpose()?;
                let limit = args.get(2).map(|value| value.parse::<usize>()
                    .map_err(|_| "条数必须为1-80".to_owned())).transpose()?.unwrap_or(30);
                if !(1..=three_kingdoms_spire_reborn::combat::MAX_RECENT_EVENTS).contains(&limit) {
                    return Err("条数必须为1-80".into());
                }
                let Some((floor, b)) = self.game.as_ref()
                    .and_then(|g| g.battle.as_ref().map(|battle| (g.floor, battle))) else {
                    return Ok("当前没有可用的战斗事件记录。".into());
                };
                // With no cursor, preserve the previous latest-30 default.
                let after = cursor.unwrap_or_else(|| {
                    if b.events.len() > limit { b.events[b.events.len() - limit - 1].seq }
                    else { 0 }
                });
                let page = b.events_after(after, limit);
                let mut lines = vec![format!(
                    "第{floor}层当前战斗事件：最早保留#{}，最新#{}（最多保留80条；游标仅限本场战斗）",
                    page.oldest_available, page.latest
                )];
                if page.reset {
                    lines.push("警告：游标超过当前战斗最新序号，可能已经进入新战斗；请从0重新查询。".into());
                }
                if page.missed > 0 {
                    lines.push(format!("警告：游标之后已有{}条事件被裁剪，无法补读。", page.missed));
                } else if cursor.is_none() && page.oldest_available > 1 {
                    lines.push("提示：更早的事件已从内存与存档中裁剪。".into());
                }
                if page.entries.is_empty() {
                    lines.push("此游标之后没有可读取的事件。".into());
                } else {
                    lines.extend(page.entries.iter().map(|event| format!(
                        "#{} {:?} 来源:{} 结算:{} 目标:{}",
                        event.seq, event.kind, event.source, event.amount,
                        event.target.as_deref().unwrap_or("-")
                    )));
                }
                lines.push(format!("下一游标：{}；{}",
                    page.next_cursor, if page.has_more { "仍有后续事件" } else { "已到当前末尾" }));
                Ok(lines.join("\n"))
            })(),
            "selections" => (|| -> Result<String, String> {
                if args.len() != 2 {
                    return Err("用法：selections <手牌编号>（从牌堆顶部开始按1编号）".into());
                }
                let index = parsed(1,0)?;
                let candidates = self.game.as_ref().ok_or("请先开局")?
                    .secondary_choices(index)?
                    .ok_or("这张卡不需要指定副选牌")?;
                if candidates.is_empty() {
                    return Ok("当前没有符合此卡条件的副选牌；打出将不会检索或复制。".into());
                }
                let lines = candidates.iter().map(|(number, card)|
                    format!("{number}. {}", card_label(card))).collect::<Vec<_>>().join("\n");
                Ok(format!("手牌{index}合法副选牌（从牌堆顶开始编号）：\n{lines}\n用法：play {index} <敌方编号> <副选牌编号>"))
            })(),
            "relics" => Ok(self.game.as_ref().map_or("尚未开局".into(), |g| {
                g.relics.iter().map(|r| format!("{}：{}",r, three_kingdoms_spire_reborn::items::relic_name(r))).collect::<Vec<_>>().join("\n")
            })),
            "choose" | "c" | "play" | "p" | "end" | "use" | "upgrade" | "remove" | "enchant" | "discard" | "variant" | "doctrine" | "modifier" | "modifiers" => {
                let action = match args[0] {
                    "choose" | "c" => parsed(1, 0).map(|index| Action::Choose { index }),
                    "play" | "p" => parsed(1, 0).and_then(|index| parsed(2, 1)
                        .and_then(|target| args.get(3).map_or(Ok(Action::Play { index,target }), |_| {
                            parsed(3, 0).map(|selection| Action::PlaySelected { index,target,selection })
                        }))),
                    "end" => Ok(Action::EndTurn),
                    "upgrade" => parsed(1, 0).map(|index| Action::Upgrade { index }),
                    "remove" => parsed(1, 0).map(|index| Action::Remove { index }),
                    "discard" => parsed(1, 0).map(|index| Action::Discard { index }),
                    "modifier" | "modifiers" => match (args.get(1), args.get(2).copied().unwrap_or("on")) {
                        (Some(id), state @ ("on" | "off")) =>
                            ChallengeModifier::parse(id).ok_or_else(||"未知挑战代码".to_owned())
                                .and_then(|modifier| {
                                    if state == "on" && !self.profile.can_use_modifier(modifier) {
                                        Err(format!("挑战【{}】尚未通过时间线解锁；输入 timeline 查看条件", modifier.name()))
                                    } else {
                                        Ok(Action::SetModifier { id: modifier.id().into(), enabled: state == "on" })
                                    }
                                }),
                        _ => Err("用法：modifier no_rest|iron_legion|thin_supplies on|off".into()),
                    },
                    "doctrine" => args.get(1)
                        .map_or(Err("输入 doctrine 查看可选战法".into()),
                            |id|Ok(Action::SetDoctrine{id:(*id).into()})),
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
        println!("三国登塔录 TKS — macOS/Linux Rust terminal roguelike\n用法：tks [--tui] [--plain] [--save-dir <目录>]\n--tui 启用上下键菜单，:输入指令；--plain 适用于脚本管道。\n游戏内输入 help 查看指令。");
        return;
    }
    let mut dir = default_dir();
    if let Some(i) = args.iter().position(|a| a == "--save-dir")
        && let Some(value) = args.get(i + 1)
    {
        dir = PathBuf::from(value);
    }
    // Acquire before reading any save, profile or preferences. Keep for the
    // entire CLI/TUI session, including recovery, autosaves and quit.
    let _save_lock = match SaveDirectoryLock::acquire(&dir) {
        Ok(lock) => lock,
        Err(error) => {
            eprintln!("无法启动：存档目录 {} 不可用：{error}", dir.display());
            std::process::exit(2);
        }
    };
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
    if (save_path(&app.dir).exists() || save_path(&app.dir).with_extension("bak").exists())
        && let Err(err) = app.load()
    {
        app.message = format!("存档未能加载：{err}；可输入 new 开新局。");
    }
    if args.iter().any(|arg|arg=="--tui") {
        if let Err(err)=tui::run(&mut app) {
            eprintln!("TUI 启动失败：{err}；请使用默认行命令模式或 --plain");
        } else {
            return;
        }
    }
    loop {
        app.draw();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                if app.game.is_some() {
                    if let Err(error) = app.save() {
                        eprintln!("输入结束时自动存档失败：{error}；未保存进度不能保证恢复");
                        std::process::exit(2);
                    }
                }
                break;
            }
            Ok(_) => if !app.command(input.trim()) { break; },
            Err(e) => { eprintln!("输入错误：{e}"); break; }
        }
    }
}

#[cfg(test)]
mod secondary_choice_cli_tests {
    use super::*;
    use three_kingdoms_spire_reborn::content::Hero;

    #[test]
    fn failed_quit_does_not_silently_discard_game_and_retry_succeeds() {
        let directory = std::env::temp_dir().join(format!(
            "tks-quit-save-failure-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&directory).unwrap();
        let mut app = App {
            game: Some(Game::new(Hero::GuanYu, 96012)),
            profile: Profile::new(),
            theme: Theme::Dark,
            dir: directory.clone(),
            use_ansi: false,
            message: String::new(),
        };
        assert!(app.command("save"));
        let original = fs::read_to_string(save_path(&directory)).unwrap();
        fs::create_dir(directory.join("save.bak")).unwrap();
        assert!(app.command("quit"), "a failed save must refuse a normal quit");
        assert!(app.message.contains("退出前保存失败"), "{}", app.message);
        assert_eq!(fs::read_to_string(save_path(&directory)).unwrap(), original);
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(), original);
        fs::remove_dir(directory.join("save.bak")).unwrap();
        assert!(!app.command("quit"), "a retried save should allow exit");
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn cli_secondary_choice_listing_is_read_only_and_uses_authoritative_order() {
        let mut game = Game::new(Hero::ZhugeLiang,983);
        game.apply(Action::Choose { index: 1 }).unwrap();
        let battle = game.battle.as_mut().unwrap();
        battle.hand = vec![Card::new("xingpan_tutor")];
        battle.draw = vec![Card::new("defend"), Card::new("strike"), Card::new("qinglong")];
        let before = game.to_json().unwrap();
        let mut app = App {
            game: Some(game), profile: Profile::new(), theme: Theme::Dark,
            dir: PathBuf::new(), use_ansi: false, message: String::new(),
        };
        assert!(app.command("selections 1"));
        assert!(app.message.contains("1. 青龙"));
        assert!(app.message.contains("2. 突刺"));
        assert!(!app.message.contains("3. "));
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(),before);
        assert!(app.command("selections 0"));
        assert!(app.message.starts_with("错误："));
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(),before);
    }
}
