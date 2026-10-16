//! UI-free deterministic game engine. Safe to reuse from Swift/UniFFI and wasm.
//! Every authoritative mutation goes through Game::apply(Action).
pub mod content;
pub mod items;
pub mod map;
pub mod progression;

use content::{Effect, Hero, REWARD_POOL, NEUTRAL_POOL, hero_pool, card_spec, enemy_name, enemy_hp};
use serde::{Deserialize, Serialize};

pub const SAVE_VERSION: u32 = 2;
pub const MAX_FLOOR: u8 = 21;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunMode {
    #[default] Standard,
    Daily,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub upgraded: bool,
    #[serde(default)] pub enchantment: Option<String>,
    #[serde(default)] pub quest_progress: u8,
}
impl Card {
    pub fn new(id: &str) -> Self {
        Self { id: id.into(), upgraded: false, enchantment: None, quest_progress: 0 }
    }
    pub fn name(&self) -> String {
        let base = card_spec(&self.id).map_or("未知牌", |s| s.name);
        format!("{base}{}{}", if self.upgraded { "+" } else { "" },
            self.enchantment.as_ref().map_or(String::new(), |kind| format!("·{kind}")))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage { Map, Combat, Reward, Camp, Market, Story, Ancient, Won, Lost }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Node { Battle, Elite, Camp, Market, Story, Boss }
impl Node {
    pub fn name(self) -> &'static str {
        match self {
            Self::Battle => "遭遇战", Self::Elite => "精英战", Self::Camp => "营寨",
            Self::Market => "军市", Self::Story => "奇遇", Self::Boss => "关底统帅",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    Attack(i32),
    MultiAttack(i32, u8),
    Hex(u8, u8),
    Guard(i32),
    Rally(i32),
}
impl Intent {
    pub fn label(&self) -> String {
        match self {
            Self::Attack(n) => format!("进攻 {n}"),
            Self::MultiAttack(n, hits) => format!("连击 {n}×{hits}"),
            Self::Hex(weak, vuln) => format!("诡计 虚弱{weak}/易伤{vuln}"),
            Self::Guard(n) => format!("防御 {n}"),
            Self::Rally(n) => format!("聚势 +{n}"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Foe {
    pub id: String,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub strength: i32,
    pub poison: i32,
    pub vulnerable: u8,
    #[serde(default)]
    pub weak: u8,
    pub intent: Intent,
    #[serde(default)] pub phase: u8,
}
impl Foe {
    pub fn name(&self) -> &'static str { enemy_name(&self.id) }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Battle {
    pub foes: Vec<Foe>,
    pub draw: Vec<Card>,
    pub hand: Vec<Card>,
    pub discard: Vec<Card>,
    pub exhausted: Vec<Card>,
    pub energy: i32,
    pub block: i32,
    pub strength: i32,
    pub weak: u8,
    #[serde(default)] pub vulnerable: u8,
    #[serde(default)] pub momentum: i32,
    #[serde(default)] pub scheme: i32,
    #[serde(default)] pub combo: i32,
    #[serde(default)] pub command: i32,
    #[serde(default)] pub fury: i32,
    #[serde(default)] pub thorns: i32,
    #[serde(default)] pub played: u32,
    #[serde(default)] pub troops_hp: i32,
    #[serde(default)] pub troops_attack: i32,
    #[serde(default)] pub discard_tokens: i32,
    pub turn: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub version: u32,
    pub revision: u64,
    pub seed: u64,
    pub rng_state: u64,
    pub hero: Hero,
    pub hp: i32,
    pub max_hp: i32,
    pub gold: i32,
    pub floor: u8,
    pub stage: Stage,
    pub last_node: Option<Node>,
    pub route: Vec<Node>,
    pub deck: Vec<Card>,
    pub relics: Vec<String>,
    pub potions: u8,
    pub battle: Option<Battle>,
    pub rewards: Vec<Card>,
    pub market: Vec<Card>,
    pub log: Vec<String>,
    #[serde(default)] pub ascension: u8,
    #[serde(default)] pub event_id: u8,
    #[serde(default)] pub potion_bag: Vec<String>,
    #[serde(default)] pub shop_relic: Option<String>,
    #[serde(default)] pub relic_history: Vec<String>,
    #[serde(default)] pub victories: u32,
    #[serde(default)] pub boons: Vec<String>,
    #[serde(default)] pub boon_choices: Vec<String>,
    #[serde(default)] pub variants: Option<[u8; 3]>,
    #[serde(default)] pub mode: RunMode,
    #[serde(default)] pub run_id: String,
    #[serde(default)] pub act_map: Option<map::ActMap>,
    #[serde(default)] pub map_lane: Option<u8>,
    #[serde(default)] pub route_lanes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Choose { index: usize },
    Play { index: usize, target: usize },
    EndTurn,
    UsePotion,
    UsePotionAt { slot: usize, target: usize },
    Upgrade { index: usize },
    Remove { index: usize },
    Enchant { index: usize, kind: String },
    Discard { index: usize },
    SetVariants { act1: u8, act2: u8, act3: u8 },
}

impl Game {
    pub fn new(hero: Hero, seed: u64) -> Self { Self::new_with_ascension(hero, seed, 0) }
    pub fn new_with_mode(hero: Hero, seed: u64, ascension: u8, mode: RunMode) -> Self {
        Self::new_with_identity(hero, seed, ascension, mode, String::new())
    }
    pub fn new_with_identity(hero: Hero, seed: u64, ascension: u8, mode: RunMode, run_id: String) -> Self {
        let mut game = Self::new_with_ascension(hero,seed,ascension);
        game.mode = mode;
        game.run_id = run_id;
        game
    }
    pub fn new_with_ascension(hero: Hero, seed: u64, ascension: u8) -> Self {
        let mut deck = vec![];
        for _ in 0..5 { deck.push(Card::new("strike")); }
        for _ in 0..4 { deck.push(Card::new("defend")); }
        deck.push(Card::new(hero.signature()));
        let max_hp = hero.max_hp();
        let mut game = Self {
            version: SAVE_VERSION, revision: 0, seed, rng_state: seed.max(1),
            hero, hp: max_hp, max_hp, gold: 90, floor: 0, stage: Stage::Map,
            last_node: None, route: vec![], deck, relics: vec!["军旗".into()],
            potions: 1, battle: None, rewards: vec![], market: vec![],
            log: vec![format!("{}率部出征，征途由此开始。", hero.name())],
            ascension: ascension.min(10), event_id: 0,
            potion_bag: vec!["healing".into()], shop_relic: None,
            relic_history: vec!["军旗".into()], victories: 0,
            boons: vec![], boon_choices: vec![], variants: None,
            mode: RunMode::Standard, run_id: String::new(),
            act_map: None, map_lane: None, route_lanes: Vec::new(),
        };
        game.refresh_route();
        game
    }
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        let mut g: Self = serde_json::from_str(raw).map_err(|e| format!("存档解析失败: {e}"))?;
        if g.version == 1 {
            g.version = SAVE_VERSION;
            if g.potion_bag.is_empty() {
                for _ in 0..g.potions { g.potion_bag.push("healing".into()); }
            }
            if g.relic_history.is_empty() { g.relic_history = g.relics.clone(); }
        }
        if g.version != SAVE_VERSION {
            return Err(format!("存档版本 {} 不兼容当前版本 {}", g.version, SAVE_VERSION));
        }
        g.validate()?;
        Ok(g)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.hp < 0 || self.hp > self.max_hp || self.max_hp <= 0 || self.floor > MAX_FLOOR
            || self.ascension > 10 || self.potions > 3 || self.potion_bag.len() > 3
            || usize::from(self.potions) != self.potion_bag.len()
            || self.potion_bag.iter().any(|id| !items::POTIONS.iter().any(|(valid,_)| valid == id))
        {
            return Err("存档属性无效".into());
        }
        if (self.stage == Stage::Combat) != self.battle.is_some() {
            return Err("存档战斗阶段与战斗快照不一致".into());
        }
        if self.variants.is_some_and(|settings| settings.iter().any(|n| *n > 1)) {
            return Err("章节变体必须为0或1".into());
        }
        if self.map_lane.is_some_and(|lane| lane >= map::LANES)
            || self.route_lanes.iter().any(|lane| *lane >= map::LANES)
            || self.act_map.as_ref().is_some_and(|graph| !graph.is_valid())
        { return Err("非法战役地图状态".into()); }
        if !self.route_lanes.is_empty() && self.route_lanes.len() != self.route.len() {
            return Err("地图显示路线与可达车道数量不一致".into());
        }
        for boon in self.boons.iter().chain(self.boon_choices.iter()) {
            if !items::BOONS.iter().any(|(id,_)| id == boon) { return Err(format!("未知赐福: {boon}")); }
        }
        for c in self.deck.iter().chain(self.rewards.iter()).chain(self.market.iter()) {
            if card_spec(&c.id).is_none() { return Err(format!("未知卡牌: {}", c.id)); }
            if c.enchantment.as_deref().is_some_and(|id|
                !matches!(id, "迅捷" | "锐锋" | "护身" | "明悟")) {
                return Err(format!("未知附魔: {:?}", c.enchantment));
            }
        }
        if let Some(b) = &self.battle {
            for c in b.draw.iter().chain(b.hand.iter()).chain(b.discard.iter()).chain(b.exhausted.iter()) {
                if card_spec(&c.id).is_none() { return Err(format!("未知战斗卡牌: {}", c.id)); }
            }
            if b.energy < 0 || b.block < 0 || b.troops_hp < 0 || b.troops_attack < 0
                || b.discard_tokens < 0 || b.foes.is_empty() || b.foes.len() > 4
                || b.momentum < 0 || b.scheme < 0 || b.combo < 0 || b.command < 0 || b.fury < 0
                || b.thorns < 0 || b.foes.iter().any(|f| f.hp < 0 || f.hp > f.max_hp || f.block < 0 || f.poison < 0 || f.phase > 1)
            { return Err("无效战斗数据".into()); }
        }
        Ok(())
    }
    fn random(&mut self, upper: usize) -> usize {
        if upper <= 1 { return 0; }
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) % upper as u64) as usize
    }
    fn shuffle(&mut self, cards: &mut [Card]) {
        for i in (1..cards.len()).rev() {
            let j = self.random(i + 1);
            cards.swap(i, j);
        }
    }
    fn refresh_route(&mut self) {
        let next = self.floor + 1;
        if next > MAX_FLOOR {
            self.route.clear();
            self.route_lanes.clear();
            return;
        }
        let act = self.floor / 7;
        if self.act_map.as_ref().is_none_or(|map| map.act != act) {
            let generated = map::ActMap::generate(|bound| self.random(bound),act);
            self.act_map = Some(generated);
            self.map_lane = None;
        }
        let row = usize::from((next - 1) % 7);
        let choices = self.act_map.as_ref().expect("generated").routes(row,self.map_lane);
        self.route_lanes = choices.iter().map(|(lane,_)| *lane).collect();
        self.route = choices.iter().map(|(_,node)| *node).collect();
    }
    fn pick_reward_cards(&mut self) -> Vec<Card> {
        let own = hero_pool(self.hero);
        let mut choices: Vec<Card> = vec![];
        while choices.len() < 3 {
            let roll = self.random(100);
            let id = if roll < 62 {
                own[self.random(own.len())]
            } else if roll < 85 {
                NEUTRAL_POOL[self.random(NEUTRAL_POOL.len())]
            } else {
                REWARD_POOL[self.random(REWARD_POOL.len())]
            };
            let card = Card::new(id);
            if !choices.iter().any(|c| c.id == card.id) { choices.push(card); }
        }
        choices
    }

    pub fn act_name(&self) -> &'static str {
        let act = self.floor.saturating_sub(1) / 7;
        let branch = self.act_variant(act);
        match (act.min(2), branch) {
            (0, 0) => "黄巾乱世", (0, _) => "江东烽烟",
            (1, 0) => "关中风云", (1, _) => "群雄会盟",
            (2, 0) => "冢虎之谋", _ => "奇门斗法",
        }
    }
    fn act_variant(&self, act: u8) -> u64 {
        self.variants.map_or(
            self.seed.rotate_right(act as u32 * 11) & 1,
            |v| u64::from(v[usize::from(act.min(2))]),
        )
    }
    fn generate_foes(&mut self, node: Node) -> Vec<Foe> {
        let act = self.floor.saturating_sub(1) / 7;
        let branch = self.act_variant(act);
        let ids: Vec<&str> = match node {
            Node::Boss => vec![match (act, branch) {
                (0,0) => "zhangjiao", (0,_) => "sunjian",
                (1,0) => "dongzhuo", (1,_) => "yuanshao",
                (2,0) => "simayi", _ => "zuoci",
            }],
            Node::Elite => {
                let elites = ["elite", "war_elephant", "royal_guard", "tiger_general", "sacred_beast"];
                vec![elites[self.random(elites.len())]]
            }
            _ => {
                let band = match (act, branch) {
                    (0,0) => ["bandit", "archer", "yellow_turban", "pyromancer", "spearman"],
                    (0,_) => ["archer", "cuirass", "sling", "raider", "crossbowman"],
                    (1,0) => ["officer", "raider", "wolf", "horse_archer", "cuirass"],
                    (1,_) => ["officer", "crossbowman", "tactician", "assassin", "iron_guard"],
                    (2,0) => ["shaman", "tomb_guard", "tactician", "witch", "panther"],
                    _ => ["shaman", "firelancer", "assassin", "witch", "iron_guard"],
                };
                let first = band[self.random(band.len())];
                if self.floor > 2 && self.random(100) < 40 {
                    let second = band[self.random(band.len())];
                    vec![first, second]
                } else { vec![first] }
            }
        };
        ids.into_iter().map(|id| {
            let hp = enemy_hp(id);
            Foe {
                id: id.to_string(), hp, max_hp: hp, block: 0, strength: 0,
                poison: 0, vulnerable: 0, weak: 0, intent: Intent::Attack(6), phase: 0,
            }
        }).collect()
    }
    fn roll_intents(&mut self) {
        let count = self.battle.as_ref().map_or(0, |b| b.foes.len());
        for i in 0..count {
            let roll = self.random(100);
            let b = self.battle.as_mut().expect("battle");
            let turn = b.turn;
            let foe = &mut b.foes[i];
            if foe.hp == 0 { continue; }
            let act = (self.floor.saturating_sub(1) / 7) as i32;
            let attack = 6 + act * 3 + foe.strength + self.ascension as i32 / 3
                + match foe.id.as_str() {
                    "dongzhuo" => 7, "simayi" => 6, "zuoci" => 5, "tiger_general" => 5,
                    "archer" | "assassin" => 1, _ => 0,
                };
            let spellcaster = matches!(foe.id.as_str(),
                "shaman" | "pyromancer" | "tactician" | "witch"
                | "zhangjiao" | "simayi" | "zuoci");
            let swift = matches!(foe.id.as_str(),
                "archer" | "crossbowman" | "horse_archer" | "assassin" | "panther");
            foe.intent = if foe.phase > 0 && matches!(foe.id.as_str(), "sunjian" | "dongzhuo") {
                Intent::MultiAttack((attack / 2).max(3), 2)
            } else if spellcaster && (roll < 26 || turn.is_multiple_of(4)) {
                Intent::Hex(1 + u8::from(act > 0), u8::from(act > 0))
            } else if swift && roll < 45 {
                Intent::MultiAttack((attack / 2).max(2), 2 + u8::from(act > 0))
            } else if roll < 68 {
                Intent::Attack(attack)
            } else if roll < 86 {
                Intent::Guard(6 + act * 4 + self.ascension as i32 / 2)
            } else {
                Intent::Rally(2 + i32::from(act > 0))
            };
        }
    }
    fn draw_cards(&mut self, count: usize) {
        for _ in 0..count {
            if self.battle.as_ref().expect("battle").hand.len() >= 10 { return; }
            if self.battle.as_ref().expect("battle").draw.is_empty() {
                let mut discard = std::mem::take(&mut self.battle.as_mut().expect("battle").discard);
                if discard.is_empty() { return; }
                self.shuffle(&mut discard);
                self.battle.as_mut().expect("battle").draw = discard;
            }
            let b = self.battle.as_mut().expect("battle");
            if let Some(card) = b.draw.pop() { b.hand.push(card); }
        }
    }
    fn start_battle(&mut self, node: Node) {
        let mut foes = self.generate_foes(node);
        for foe in &mut foes {
            let extra = (self.ascension as i32 / 2) * (1 + (self.floor as i32 / 7));
            foe.hp += extra;
            foe.max_hp += extra;
        }
        let mut draw = self.deck.clone();
        self.shuffle(&mut draw);
        self.battle = Some(Battle {
            foes, draw, hand: vec![], discard: vec![], exhausted: vec![],
            energy: 3, block: 0, strength: if self.hero == Hero::GuanYu { 1 } else { 0 },
            weak: 0, vulnerable: 0, momentum: 0, scheme: 0,
            combo: 0, command: 0, fury: 0, thorns: 0, played: 0,
            troops_hp: if self.hero == Hero::CaoCao { 12 } else { 0 },
            troops_attack: if self.hero == Hero::CaoCao { 3 } else { 0 },
            discard_tokens: 0, turn: 1,
        });
        self.roll_intents();
        let extra_draw = usize::from(self.relics.iter().any(|r| r == "虎翼"));
        let boon_draw = usize::from(self.boons.iter().any(|b| b == "strategist")) * 2;
        self.draw_cards(5 + extra_draw + boon_draw);
        if let Some(b) = &mut self.battle {
            if self.relics.iter().any(|r| r == "铁骑") { b.block += 8; }
            if self.boons.iter().any(|id| id == "veteran") { b.block += 6; }
            if self.boons.iter().any(|id| id == "war_drum") { b.energy += 1; }
            if self.boons.iter().any(|id| id == "blood_oath") { b.strength += 2; }
            if self.boons.iter().any(|id| id == "formation") { b.command += 2; }
            if self.relics.iter().any(|r| r == "璇玑") { b.energy += 1; }
            if self.relics.iter().any(|r| r == "军旗") { b.block += 4; }
            if self.relics.iter().any(|r| r == "虎符") { b.strength += 1; }
            if self.relics.iter().any(|r| r == "兵书") { b.scheme += 1; }
        }
        self.stage = Stage::Combat;
    }
    fn finish_scene(&mut self) {
        self.stage = Stage::Map;
        self.last_node = None;
        self.rewards.clear();
        self.market.clear();
        self.shop_relic = None;
        self.boon_choices.clear();
        self.refresh_route();
    }
    fn choose_route(&mut self, index: usize) -> Result<String, String> {
        let node = *self.route.get(index.checked_sub(1).ok_or("路线编号从1开始")?)
            .ok_or("无效路线编号")?;
        if let Some(lane) = self.route_lanes.get(index - 1) {
            self.map_lane = Some(*lane);
        }
        self.floor += 1;
        self.last_node = Some(node);
        let msg = format!("第{}层：{}", self.floor, node.name());
        match node {
            Node::Battle | Node::Elite | Node::Boss => self.start_battle(node),
            Node::Camp => self.stage = Stage::Camp,
            Node::Market => {
                self.market = self.pick_reward_cards();
                self.shop_relic = self.select_relic();
                self.stage = Stage::Market;
            }
            Node::Story => {
                for card in &mut self.deck {
                    if card.id == "quest_tulu" {
                        card.quest_progress = card.quest_progress.saturating_add(1);
                        if card.quest_progress >= 3 {
                            card.id = "quest_resolved".into();
                            card.quest_progress = 0;
                        }
                    }
                }
                self.event_id = self.random(12) as u8;
                self.stage = Stage::Story;
            },
        }
        Ok(msg)
    }
    fn target_index(&self, requested: usize) -> Result<usize, String> {
        let battle = self.battle.as_ref().ok_or("当前不在战斗中")?;
        let idx = requested.checked_sub(1).ok_or("目标编号从1开始")?;
        if battle.foes.get(idx).is_none_or(|f| f.hp <= 0) {
            Err("无效目标：敌军不存在或已阵亡".into())
        } else { Ok(idx) }
    }
    fn damage_enemy(&mut self, idx: usize, base: i32) {
        let b = self.battle.as_mut().expect("battle");
        let raw = base + b.strength
            + if self.hero == Hero::GuanYu { b.momentum / 3 } else { 0 }
            + if self.hero == Hero::LuBu { b.fury / 4 } else { 0 };
        let foe = &mut b.foes[idx];
        let raw = if b.weak > 0 { raw.max(0) * 3 / 4 } else { raw.max(0) };
        let damage = if foe.vulnerable > 0 { raw * 3 / 2 } else { raw };
        let absorbed = damage.min(foe.block);
        foe.block -= absorbed;
        foe.hp = (foe.hp - damage + absorbed).max(0);
    }
    fn defeated(&self) -> bool {
        self.battle.as_ref().is_some_and(|b| b.foes.iter().all(|f| f.hp <= 0))
    }

    fn select_relic(&mut self) -> Option<String> {
        let available: Vec<&str> = items::RELICS.iter().map(|(id, _)| *id)
            .filter(|id| !self.relic_history.iter().any(|r| r == *id)).collect();
        if available.is_empty() { None }
        else { Some(available[self.random(available.len())].to_string()) }
    }
    fn grant_relic(&mut self) {
        if let Some(id) = self.select_relic() {
            if id == "厚甲" { self.max_hp += 8; self.hp += 8; }
            self.relic_history.push(id.clone());
            self.relics.push(id);
        } else {
            self.gold += 50;
        }
    }
    fn win_battle(&mut self) -> String {
        let node = self.last_node.unwrap_or(Node::Battle);
        self.battle = None;
        let payout = match node { Node::Boss => 100, Node::Elite => 60, _ => 24 + self.random(15) as i32 };
        self.gold += payout;
        if self.relics.iter().any(|r| r == "铜雀") { self.gold += 12; }
        if self.relics.iter().any(|r| r == "神医") {
            self.hp = (self.hp + 6).min(self.max_hp);
        }
        if self.relics.iter().any(|r| r == "百草") {
            self.hp = (self.hp + 2).min(self.max_hp);
        }
        if self.boons.iter().any(|id| id == "renewal") {
            self.hp = (self.hp + 8).min(self.max_hp);
        }
        if node == Node::Elite || node == Node::Boss {
            self.grant_relic();
            if node == Node::Boss {
                self.max_hp += 5;
                self.hp = (self.hp + 5).min(self.max_hp);
            }
        }
        let chance = if self.relics.iter().any(|r| r == "药囊") { 60 } else { 32 };
        if self.potions < 3 && self.random(100) < chance {
            let pool = ["healing", "fire", "shield", "energy", "poison", "strength", "insight", "weak", "cleanse", "blood"];
            let potion = pool[self.random(pool.len())];
            self.potion_bag.push(potion.into());
            self.potions = self.potion_bag.len() as u8;
        }
        if self.floor >= MAX_FLOOR {
            self.victories = self.victories.saturating_add(1);
            self.stage = Stage::Won;
            return "三军凯旋！三幕征程全部通关。".into();
        }
        self.rewards = self.pick_reward_cards();
        self.stage = Stage::Reward;
        format!("战斗胜利！获得{payout}金，选择一张战利品，或输入 choose 0 跳过。")
    }
    fn play(&mut self, index: usize, target: usize) -> Result<String, String> {
        let b = self.battle.as_ref().ok_or("当前不在战斗中")?;
        let hand_idx = index.checked_sub(1).ok_or("手牌编号从1开始")?;
        let card = b.hand.get(hand_idx).ok_or("无效手牌编号")?.clone();
        let spec = card_spec(&card.id).ok_or("未知卡牌")?;
        if matches!(spec.effect, Effect::Unplayable) { return Err("这张牌无法打出".into()); }
        let cost = if card.enchantment.as_deref() == Some("迅捷") {
            (spec.cost - 1).max(0)
        } else { spec.cost };
        if b.energy < cost { return Err("军令不足".into()); }
        let requires_target = matches!(
            spec.effect, Effect::Hit(_) | Effect::HitDraw(_, _) | Effect::HitTwice(_)
                | Effect::Poison(_) | Effect::HitWeak(_, _) | Effect::HitVulnerable(_, _)
                | Effect::MomentumStrike(_) | Effect::SchemeStrike(_) | Effect::ComboStrike(_)
                | Effect::CommandStrike(_) | Effect::FuryStrike(_) | Effect::BloodPrice(_, _)
                | Effect::Leech(_) | Effect::HitPoison(_, _) | Effect::Pierce(_)
                | Effect::DiscardStrike(_)
        );
        let target_idx = if requires_target { Some(self.target_index(target)?) } else { None };
        {
            let b = self.battle.as_mut().expect("battle");
            b.energy -= cost;
            b.hand.remove(hand_idx);
        }
        let bonus = if card.upgraded { 3 } else { 0 }
            + if card.enchantment.as_deref() == Some("锐锋") { 3 } else { 0 };
        match spec.effect {
            Effect::Hit(n) => self.damage_enemy(target_idx.expect("target"), n + bonus),
            Effect::HitDraw(n, draw) => {
                self.damage_enemy(target_idx.expect("target"), n + bonus);
                self.draw_cards(draw);
            }
            Effect::HitTwice(n) => {
                for _ in 0..2 { self.damage_enemy(target_idx.expect("target"), n + bonus); }
            }
            Effect::All(n) => {
                let count = self.battle.as_ref().expect("battle").foes.len();
                let extra = if self.relics.iter().any(|r| r == "朱雀") { 3 } else { 0 }
                    + if self.boons.iter().any(|id| id == "flaming_sky") { 4 } else { 0 };
                for i in 0..count {
                    if self.battle.as_ref().expect("battle").foes[i].hp > 0 {
                        self.damage_enemy(i, n + bonus + extra);
                    }
                }
            }
            Effect::Guard(n) => self.battle.as_mut().expect("battle").block += n + bonus,
            Effect::GuardEnergy(n) => {
                let b = self.battle.as_mut().expect("battle");
                b.block += n + bonus;
                b.energy += 1;
            }
            Effect::Poison(n) => {
                self.battle.as_mut().expect("battle").foes[target_idx.expect("target")].poison += n + bonus;
            }
            Effect::Draw(n) => self.draw_cards(n + usize::from(card.upgraded)),
            Effect::Strength(n) => self.battle.as_mut().expect("battle").strength += n + i32::from(card.upgraded),
            Effect::Heal(n) => self.hp = (self.hp + n + bonus).min(self.max_hp),
            Effect::HitWeak(n, turns) => {
                let idx = target_idx.expect("target");
                self.damage_enemy(idx, n + bonus);
                self.battle.as_mut().expect("battle").foes[idx].weak += turns;
            }
            Effect::HitVulnerable(n, turns) => {
                let idx = target_idx.expect("target");
                self.damage_enemy(idx, n + bonus);
                self.battle.as_mut().expect("battle").foes[idx].vulnerable += turns;
            }
            Effect::WeakAll(turns) => {
                for foe in &mut self.battle.as_mut().expect("battle").foes {
                    if foe.hp > 0 { foe.weak += turns; }
                }
            }
            Effect::ApplyVulnerable(turns) => {
                for foe in &mut self.battle.as_mut().expect("battle").foes {
                    if foe.hp > 0 { foe.vulnerable += turns; }
                }
            }
            Effect::GainMomentum(n) => self.battle.as_mut().expect("battle").momentum += n + bonus,
            Effect::MomentumStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").momentum * 3;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
                self.battle.as_mut().expect("battle").momentum = 0;
            }
            Effect::Scheme(n) => self.battle.as_mut().expect("battle").scheme += n + bonus,
            Effect::SchemeStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").scheme * 4;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
                self.battle.as_mut().expect("battle").scheme = 0;
            }
            Effect::ComboStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").combo * 3;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
            }
            Effect::Command(n) => self.battle.as_mut().expect("battle").command += n + bonus,
            Effect::CommandStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").command * 3;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
                self.battle.as_mut().expect("battle").command = 0;
            }
            Effect::Fury(n) => self.battle.as_mut().expect("battle").fury += n + bonus,
            Effect::FuryStrike(n) => {
                let boost = self.battle.as_ref().expect("battle").fury * 3;
                self.damage_enemy(target_idx.expect("target"), n + bonus + boost);
                self.battle.as_mut().expect("battle").fury = 0;
            }
            Effect::BloodPrice(loss, power) => {
                self.hp = (self.hp - loss).max(1);
                self.battle.as_mut().expect("battle").fury += loss;
                self.damage_enemy(target_idx.expect("target"), power + bonus);
            }
            Effect::Thorns(n) => self.battle.as_mut().expect("battle").thorns += n + bonus,
            Effect::Leech(n) => {
                let idx = target_idx.expect("target");
                let before = self.battle.as_ref().expect("battle").foes[idx].hp;
                self.damage_enemy(idx, n + bonus);
                let after = self.battle.as_ref().expect("battle").foes[idx].hp;
                self.hp = (self.hp + (before - after) / 3).min(self.max_hp);
            }
            Effect::HitPoison(n, poison) => {
                let idx = target_idx.expect("target");
                self.damage_enemy(idx, n + bonus);
                self.battle.as_mut().expect("battle").foes[idx].poison += poison;
            }
            Effect::Pierce(n) => {
                let b = self.battle.as_mut().expect("battle");
                let foe = &mut b.foes[target_idx.expect("target")];
                foe.hp = (foe.hp - n - bonus).max(0);
            }
            Effect::DrawEnergy(n, energy) => {
                self.battle.as_mut().expect("battle").energy += energy;
                self.draw_cards(n + usize::from(card.upgraded));
            }
            Effect::Energy(n) => self.battle.as_mut().expect("battle").energy += n + i32::from(card.upgraded),
            Effect::Companion(amount) => {
                let b = self.battle.as_mut().expect("battle");
                b.troops_hp += amount + bonus;
                b.troops_attack += (amount / 4).max(1);
            }
            Effect::Tactics(amount) => {
                self.battle.as_mut().expect("battle").discard_tokens += amount;
            }
            Effect::DiscardStrike(amount) => {
                self.damage_enemy(target_idx.expect("target"), amount + bonus);
            }
            Effect::Unplayable => unreachable!(),
        }
        if card.enchantment.as_deref() == Some("护身") {
            self.battle.as_mut().expect("battle").block += 4;
        }
        if card.enchantment.as_deref() == Some("明悟") { self.draw_cards(1); }
        let attack = requires_target || matches!(spec.effect, Effect::All(_));
        {
            let b = self.battle.as_mut().expect("battle");
            b.played += 1;
            match self.hero {
                Hero::GuanYu if attack => b.momentum += 1,
                Hero::ZhugeLiang if !attack => b.scheme += 1,
                Hero::SunShangxiang if attack => b.combo += 1,
                Hero::CaoCao if !attack => b.command += 1,
                _ => {}
            }
        }
        if self.hero == Hero::ZhugeLiang && self.battle.as_ref().expect("battle").scheme >= 3 {
            self.battle.as_mut().expect("battle").scheme -= 3;
            self.battle.as_mut().expect("battle").energy += 1;
            self.draw_cards(1);
        }
        if self.hero == Hero::SunShangxiang && attack
            && self.battle.as_ref().expect("battle").combo % 3 == 0 {
            self.draw_cards(1);
        }
        if self.hero == Hero::CaoCao && self.battle.as_ref().expect("battle").command >= 3 {
            let b = self.battle.as_mut().expect("battle");
            b.command -= 3;
            b.block += 5;
            b.energy += 1;
        }
        if self.relics.iter().any(|r| r == "百战") && attack {
            self.battle.as_mut().expect("battle").strength += 1;
        }
        let msg = format!("使用了【{}】。", card.name());
        if spec.exhaust {
            self.battle.as_mut().expect("battle").exhausted.push(card);
        } else {
            self.battle.as_mut().expect("battle").discard.push(card);
        }
        if self.defeated() { Ok(format!("{msg} {}", self.win_battle())) } else { Ok(msg) }
    }
    fn discard_card(&mut self, index: usize) -> Result<String, String> {
        if self.stage != Stage::Combat { return Err("仅战斗中可弃换".into()); }
        let idx = index.checked_sub(1).ok_or("手牌编号从1开始")?;
        let battle = self.battle.as_ref().expect("battle");
        if battle.discard_tokens == 0 { return Err("没有弃换次数；需先打出换阵类卡牌".into()); }
        if idx >= battle.hand.len() { return Err("无效手牌编号".into()); }
        let card = self.battle.as_mut().expect("battle").hand.remove(idx);
        let auto = card.id == "feishi";
        let msg = if auto { format!("弃换【{}】：触发策应攻击", card.name()) }
                  else { format!("弃换【{}】", card.name()) };
        {
            let b = self.battle.as_mut().expect("battle");
            b.discard_tokens -= 1;
            b.discard.push(card);
            if self.hero == Hero::SunShangxiang { b.combo += 1; }
        }
        if auto {
            let target = self.battle.as_ref().expect("battle").foes
                .iter().position(|foe| foe.hp > 0);
            if let Some(target) = target { self.damage_enemy(target, 7); }
        }
        self.draw_cards(1);
        if self.defeated() { Ok(format!("{msg}。{}", self.win_battle())) }
        else { Ok(msg) }
    }
    fn troop_attack(&mut self) {
        let snapshot = self.battle.as_ref().expect("battle");
        if snapshot.troops_hp <= 0 || snapshot.troops_attack <= 0 { return; }
        if let Some(idx) = snapshot.foes.iter().position(|f| f.hp > 0) {
            // Companions do not inherit the hero's strength or consume command.
            let b = self.battle.as_mut().expect("battle");
            let f = &mut b.foes[idx];
            let damage = b.troops_attack;
            let absorbed = damage.min(f.block);
            f.block -= absorbed;
            f.hp = (f.hp - damage + absorbed).max(0);
        }
    }
    fn enemy_attack(&mut self, foe_idx: usize, power: i32) -> i32 {
        let b = self.battle.as_mut().expect("battle");
        let reduced = if b.foes[foe_idx].weak > 0 { power * 3 / 4 } else { power };
        let modified = if b.vulnerable > 0 { reduced * 3 / 2 } else { reduced };
        let blocked = modified.min(b.block);
        b.block -= blocked;
        let incoming = (modified - blocked).max(0);
        let protected = incoming.min(b.troops_hp);
        b.troops_hp -= protected;
        let damage = incoming - protected;
        self.hp = (self.hp - damage).max(0);
        if self.hero == Hero::LuBu { b.fury += damage / 2; }
        if b.thorns > 0 {
            b.foes[foe_idx].hp = (b.foes[foe_idx].hp - b.thorns).max(0);
        }
        damage
    }

    fn boss_phase_transitions(&mut self) -> Vec<String> {
        let changes: Vec<(usize,String)> = self.battle.as_ref().expect("battle").foes
            .iter().enumerate().filter(|(_,foe)|
                foe.hp > 0 && foe.phase == 0 && foe.hp * 2 <= foe.max_hp
                && matches!(foe.id.as_str(),"zhangjiao"|"sunjian"|"dongzhuo"|"yuanshao"|"simayi"|"zuoci"))
            .map(|(i,foe)| (i,foe.id.clone())).collect();
        let mut messages = vec![];
        for (idx,id) in changes {
            let b = self.battle.as_mut().expect("battle");
            b.foes[idx].phase = 1;
            match id.as_str() {
                "zhangjiao" | "yuanshao" | "zuoci" => {
                    let ally = match id.as_str() {
                        "zhangjiao" => "yellow_turban",
                        "yuanshao" => "officer", _ => "shaman",
                    };
                    if b.foes.len() < 4 {
                        let hp = enemy_hp(ally);
                        b.foes.push(Foe {
                            id: ally.into(), hp, max_hp: hp, block: 0, strength: 0,
                            poison: 0, vulnerable: 0, weak: 0,
                            intent: Intent::Guard(4), phase: 0,
                        });
                    }
                    if id == "zuoci" {
                        let foe = &mut b.foes[idx];
                        foe.hp = (foe.hp + 18).min(foe.max_hp);
                    }
                    messages.push(format!("{}进入第二阶段并呼叫援军",enemy_name(&id)));
                }
                "sunjian" => {
                    b.foes[idx].strength += 4;
                    b.foes[idx].block += 8;
                    messages.push("孙坚激发虎威，获得4武勇和8护甲".into());
                }
                "dongzhuo" => {
                    b.foes[idx].strength += 3;
                    b.weak += 2;
                    self.gold = (self.gold - 30).max(0);
                    messages.push("董卓暴虐：我军虚弱两回合并损失最多30金".into());
                }
                "simayi" => {
                    b.discard.push(Card::new("cursed"));
                    b.discard.push(Card::new("cursed"));
                    b.vulnerable += 2;
                    messages.push("司马懿奇袭：两张军心涣散混入弃牌堆，我军易伤两回合".into());
                }
                _ => {}
            }
        }
        messages
    }
    fn end_turn(&mut self) -> Result<String, String> {
        if self.stage != Stage::Combat { return Err("当前不在战斗中".into()); }
        {
            let b = self.battle.as_mut().expect("battle");
            b.discard.append(&mut b.hand);
            for foe in &mut b.foes {
                if foe.hp > 0 && foe.poison > 0 {
                    foe.hp = (foe.hp - foe.poison).max(0);
                    foe.poison = (foe.poison - 1).max(0);
                }
            }
        }
        if self.defeated() { return Ok(self.win_battle()); }
        self.troop_attack();
        if self.defeated() { return Ok(self.win_battle()); }
        let phase_messages = self.boss_phase_transitions();
        if let Some(b) = self.battle.as_mut() {
            for f in &mut b.foes { f.block = 0; }
        }
        let intents: Vec<(usize, Intent)> = self.battle.as_ref().expect("battle").foes.iter()
            .enumerate().filter(|(_, f)| f.hp > 0)
            .map(|(i, f)| (i, f.intent.clone())).collect();
        let mut messages = phase_messages;
        for (i, intent) in intents {
            if self.battle.as_ref().is_some_and(|b| b.foes[i].hp == 0) { continue; }
            let name = self.battle.as_ref().expect("battle").foes[i].name().to_string();

            match intent {
                Intent::Attack(power) => {
                    let damage = self.enemy_attack(i, power);
                    messages.push(format!("{name}进攻，受到{damage}点伤害"));
                }
                Intent::MultiAttack(power, hits) => {
                    let mut total = 0;
                    for _ in 0..hits {
                        if self.hp == 0 || self.battle.as_ref().expect("battle").foes[i].hp == 0 { break; }
                        total += self.enemy_attack(i, power);
                    }
                    messages.push(format!("{name}连续进攻{hits}次，受到{total}点伤害"));
                }
                Intent::Hex(weak, vulnerable) => {
                    let b = self.battle.as_mut().expect("battle");
                    b.weak += weak;
                    b.vulnerable += vulnerable;
                    messages.push(format!("{name}施放诡计：虚弱{weak}/易伤{vulnerable}"));
                }
                Intent::Guard(amount) => {
                    self.battle.as_mut().expect("battle").foes[i].block += amount;
                    messages.push(format!("{name}增加{amount}护甲"));
                }
                Intent::Rally(amount) => {
                    self.battle.as_mut().expect("battle").foes[i].strength += amount;
                    messages.push(format!("{name}聚势+{amount}"));
                }
            }
            if self.hp == 0 {
                self.stage = Stage::Lost;
                self.battle = None;
                return Ok(format!("{}。将星陨落，征途结束。", messages.join("；")));
            }
        }
        {
            let b = self.battle.as_mut().expect("battle");
            b.block = 0;
            b.weak = b.weak.saturating_sub(1);
            b.vulnerable = b.vulnerable.saturating_sub(1);
            b.energy = 3 + i32::from(self.relics.iter().any(|r| r == "玉玺"));
            b.combo = 0;
            b.discard_tokens = 0;
            b.turn += 1;
            for foe in &mut b.foes {
                foe.vulnerable = foe.vulnerable.saturating_sub(1);
                foe.weak = foe.weak.saturating_sub(1);
            }
        }
        self.roll_intents();
        self.draw_cards(5 + usize::from(self.relics.iter().any(|r| r == "虎翼")));
        if self.defeated() { return Ok(self.win_battle()); }
        Ok(format!("回合结束：{}。", messages.join("；")))
    }
    fn choose_reward(&mut self, index: usize) -> Result<String, String> {
        let result = if index == 0 {
            "放弃卡牌奖励。".into()
        } else {
            let card = self.rewards.get(index - 1).ok_or("无效奖励编号")?.clone();
            let msg = format!("加入卡牌【{}】。", card.name());
            self.deck.push(card);
            msg
        };
        if self.last_node == Some(Node::Boss) {
            self.boon_choices = self.choose_boons();
            self.stage = Stage::Ancient;
        } else { self.finish_scene(); }
        Ok(result)
    }
    fn choose_boons(&mut self) -> Vec<String> {
        let mut pool: Vec<&str> = items::BOONS.iter().map(|(id,_)| *id)
            .filter(|id| !self.boons.iter().any(|owned| owned == id)).collect();
        let mut picks = vec![];
        while !pool.is_empty() && picks.len() < 3 {
            let idx = self.random(pool.len());
            picks.push(pool.remove(idx).to_string());
        }
        picks
    }
    fn choose_boon(&mut self, index: usize) -> Result<String, String> {
        if self.stage != Stage::Ancient { return Err("当前不在先贤赐福场景".into()); }
        let idx = index.checked_sub(1).ok_or("赐福编号从1开始")?;
        let id = self.boon_choices.get(idx).ok_or("无效赐福编号")?.clone();
        match id.as_str() {
            "imperial_seal" => { self.max_hp += 12; self.hp += 12; }
            "blood_oath" => { self.max_hp = (self.max_hp - 8).max(1); self.hp = self.hp.min(self.max_hp).max(1); }
            "caravan" => self.gold += 140,
            "arsenal" => {
                let mut eligible: Vec<usize> = self.deck.iter().enumerate()
                    .filter(|(_,c)| !c.upgraded && c.id != "cursed" && c.id != "quest_tulu")
                    .map(|(i,_)| i).collect();
                for _ in 0..3 {
                    if eligible.is_empty() { break; }
                    let index = self.random(eligible.len());
                    let picked = eligible.remove(index);
                    self.deck[picked].upgraded = true;
                }
            }
            "herb_garden" => {
                while self.potion_bag.len() < 3 { self.potion_bag.push("healing".into()); }
                self.potions = self.potion_bag.len() as u8;
            }
            "founder" => self.grant_relic(),
            _ => {}
        }
        self.boons.push(id.clone());
        self.finish_scene();
        Ok(format!("获得先贤赐福：{}", items::boon_name(&id)))
    }

    fn enchant_card(&mut self, index: usize, kind: &str) -> Result<String, String> {
        if self.stage != Stage::Camp { return Err("附魔必须在营寨进行".into()); }
        if !matches!(kind, "迅捷" | "锐锋" | "护身" | "明悟") {
            return Err("附魔类型：迅捷 / 锐锋 / 护身 / 明悟".into());
        }
        let idx = index.checked_sub(1).ok_or("牌组编号从1开始")?;
        let card = self.deck.get(idx).ok_or("无效牌组编号")?;
        if card.enchantment.is_some() || matches!(card.id.as_str(), "cursed" | "quest_tulu") {
            return Err("此卡无法附魔".into());
        }
        if self.gold < 35 { return Err("附魔需要35金".into()); }
        self.gold -= 35;
        self.deck[idx].enchantment = Some(kind.to_owned());
        let message = format!("附魔【{}】，获得{kind}效果。", self.deck[idx].name());
        self.finish_scene();
        Ok(message)
    }
    fn upgrade_card(&mut self, index: usize) -> Result<String, String> {
        if self.stage != Stage::Camp { return Err("只能在营寨升级卡牌".into()); }
        let idx = index.checked_sub(1).ok_or("卡牌编号从1开始")?;
        let card = self.deck.get_mut(idx).ok_or("无效卡牌编号")?;
        if card.upgraded || card.id == "cursed" { return Err("此卡不可升级".into()); }
        card.upgraded = true;
        let msg = format!("升级【{}】。", card.name());
        self.finish_scene();
        Ok(msg)
    }
    fn remove_card(&mut self, index: usize) -> Result<String, String> {
        if self.stage != Stage::Market { return Err("只能在军市移除卡牌".into()); }
        if self.deck.len() <= 1 { return Err("至少保留一张卡牌".into()); }
        let idx = index.checked_sub(1).ok_or("卡牌编号从1开始")?;
        if idx >= self.deck.len() { return Err("无效卡牌编号".into()); }
        if self.gold < 75 { return Err("移牌需要75金".into()); }
        self.gold -= 75;
        let card = self.deck.remove(idx);
        Ok(format!("消耗75金移除【{}】。", card.name()))
    }
    fn choose_camp(&mut self, index: usize) -> Result<String, String> {
        let msg = match index {
            1 => {
                let extra = if self.relics.iter().any(|r| r == "军粮") { 8 } else { 0 };
                let heal = (self.max_hp * 30 / 100).max(1) + extra;
                let actual = heal.min(self.max_hp - self.hp);
                self.hp += actual;
                format!("休整回复{actual}点体力。")
            }
            2 => {
                let index = self.deck.iter().position(|c| !c.upgraded && c.id != "cursed")
                    .ok_or("没有可升级的卡牌")? + 1;
                return self.upgrade_card(index);
            }
            3 => {
                self.max_hp += 2; self.hp += 2;
                "操练兵卒：最大体力+2。".into()
            }
            _ => return Err("营寨：choose 1 休整、choose 2 升级首张可升级牌、choose 3 操练；升级指定牌用 upgrade N".into()),
        };
        self.finish_scene();
        Ok(msg)
    }
    fn choose_market(&mut self, index: usize) -> Result<String, String> {
        if index == 0 { self.finish_scene(); return Ok("离开军市。".into()); }
        if (1..=3).contains(&index) {
            let idx = index - 1;
            let cost = 45 + idx as i32 * 15;
            if self.gold < cost { return Err(format!("需要{cost}金")); }
            let card = self.market.get(idx).ok_or("该商品已售出")?.clone();
            self.gold -= cost;
            self.market.remove(idx);
            return Ok(format!("购得【{}】，花费{cost}金。", card.name()));
        }
        match index {
            4 => {
                if self.gold < 55 || self.potions >= 3 { return Err("药剂55金；最多携带3瓶".into()); }
                self.gold -= 55;
                let id = items::POTIONS[self.random(items::POTIONS.len())].0;
                self.potion_bag.push(id.to_owned());
                self.potions = self.potion_bag.len() as u8;
                Ok(format!("购得药剂：{}", items::potion_name(id)))
            }
            5 => {
                let id = self.shop_relic.as_ref().ok_or("遗物已售罄")?.clone();
                if self.gold < 155 { return Err("遗物价格155金".into()); }
                self.gold -= 155;
                if id == "厚甲" { self.max_hp += 8; self.hp += 8; }
                self.relics.push(id.clone()); self.relic_history.push(id.clone());
                self.shop_relic = None;
                Ok(format!("购得遗物【{}】：{}", id, items::relic_name(&id)))
            }
            6 => {
                let idx = self.deck.iter().position(|c| c.id == "cursed")
                    .or_else(|| self.deck.iter().position(|c| c.id == "strike"))
                    .ok_or("无可快捷移除卡牌，用 remove N 指定其他卡牌")? + 1;
                self.remove_card(idx)
            }
            _ => Err("军市选项不存在".into()),
        }
    }

    pub fn story_options(&self) -> &'static str {
        match self.event_id {
            0 => "驿站筹粮：1 购粮恢复 / 2 火攻古卷附诅咒 / 3 劫军资",
            1 => "破庙祭祀：1 损血得遗物 / 2 恢复 / 3 离开",
            2 => "渡江船工：1 购粮 / 2 损血赚金 / 3 专属牌",
            3 => "流民投靠：1 买药 / 2 获金 / 3 中立牌",
            4 => "军师论道：1 付费升级 / 2 专属牌 / 3 离开",
            5 => "古战场：1 损血得遗物 / 2 金币附诅咒 / 3 离开",
            6 => "雨夜守寨：1 恢复 / 2 损血增加最大生命 / 3 获金币",
            7 => "奇门阵：1 免费升级 / 2 专属牌附诅咒 / 3 离开",
            8 => "粮道纠纷：1 购粮 / 2 损血换金币 / 3 领药",
            9 => "铁匠相逢：1 付费升级 / 2 中立牌 / 3 离开",
            10 => "故旧重逢：1 恢复 / 2 增加最大生命 / 3 专属牌",
            _ => "星象异变：1 损血得遗物 / 2 获金币 / 3 恢复",
        }
    }
    fn choose_story(&mut self, index: usize) -> Result<String, String> {
        if !(1..=3).contains(&index) { return Err("奇遇请选 choose 1、2 或 3".into()); }
        let event = self.event_id;
        let cost = match (event,index) {
            (0,1) => 15, (2,1) => 25, (3,1) => 30,
            (4,1) => 40, (8,1) => 20, (9,1) => 40, _ => 0,
        };
        if self.gold < cost { return Err(format!("需要{cost}金")); }
        self.gold -= cost;
        let msg = match (event,index) {
            (0,1) | (2,1) | (8,1) => {
                self.hp = (self.hp + 18).min(self.max_hp); "购得粮草，回复18体力。"
            }
            (0,2) => {
                self.deck.push(Card::new("huogong")); self.deck.push(Card::new("cursed"));
                "获得火攻古卷，但沾染军心涣散。"
            }
            (0,3) => {
                self.hp = (self.hp - 8).max(0); self.gold += 65;
                "损失8体力，获得65金。"
            }
            (1,1) | (5,1) | (11,1) => {
                self.hp = (self.hp - if event == 1 { 6 } else { 10 }).max(0);
                self.grant_relic(); "献祭体力，获得遗物。"
            }
            (1,2) | (6,1) | (10,1) | (11,3) => {
                self.hp = (self.hp + 15).min(self.max_hp); "恢复15体力。"
            }
            (2,2) | (8,2) => {
                self.hp = (self.hp - 7).max(0); self.gold += 40;
                "失去7体力，得到40金。"
            }
            (4,2) | (7,2) => {
                self.deck.push(Card::new("quest_tulu"));
                if event == 7 { self.deck.push(Card::new("cursed")); }
                "得军情图录：再经历3次奇遇后蜕变为决断之牌。"
            }
            (2,3) | (10,3) => {
                let pool = hero_pool(self.hero);
                let picked = pool[self.random(pool.len())];
                self.deck.push(Card::new(picked));
                if event == 7 { self.deck.push(Card::new("cursed")); }
                "获得一张武将专属牌。"
            }
            (3,1) | (8,3) => {
                if self.potions < 3 { self.potion_bag.push("healing".into()); self.potions += 1; }
                else { self.gold += 15; }
                "获得药剂，满栏则转换为15金。"
            }
            (3,2) | (6,3) | (11,2) => {
                self.gold += if event == 11 {45} else {30}; "获得金币。"
            }
            (3,3) | (9,2) => {
                let picked = NEUTRAL_POOL[self.random(NEUTRAL_POOL.len())];
                self.deck.push(Card::new(picked));
                "获得一张中立牌。"
            }
            (4,1) | (7,1) | (9,1) => {
                if let Some(card) = self.deck.iter_mut().find(|c| !c.upgraded && c.id != "cursed") {
                    card.upgraded = true; "获得一次卡牌升级。"
                } else { self.gold += cost; "没有可升级卡牌，退回所付费用。" }
            }
            (5,2) => { self.gold += 60; self.deck.push(Card::new("cursed")); "拾得60金，但附上诅咒。" }
            (6,2) => { self.max_hp += 4; self.hp = (self.hp - 8).max(0); "最大体力加4，失去8体力。" }
            (10,2) => { self.max_hp += 3; self.hp += 3; "最大体力增加3。" }
            _ => "避开纷争，继续行军。",
        };
        if self.relics.iter().any(|r| r == "聚宝") && msg.contains("金") {
            self.gold += 10;
        }
        if self.hp == 0 { self.stage = Stage::Lost; } else { self.finish_scene(); }
        Ok(msg.into())
    }
    fn use_potion(&mut self, slot: usize, target: usize) -> Result<String, String> {
        if self.stage != Stage::Combat { return Err("只能战斗时使用药剂".into()); }
        let idx = slot.checked_sub(1).ok_or("药剂编号从1开始")?;
        let id = self.potion_bag.get(idx).ok_or("无效药剂编号")?.clone();
        let foe_index = if matches!(id.as_str(), "fire" | "poison") {
            Some(self.target_index(target)?)
        } else { None };
        self.potion_bag.remove(idx);
        self.potions = self.potion_bag.len() as u8;
        let label = items::potion_name(&id).to_string();
        match id.as_str() {
            "healing" => self.hp = (self.hp + 16).min(self.max_hp),
            "fire" => self.damage_enemy(foe_index.expect("target"), 22),
            "shield" => self.battle.as_mut().expect("battle").block += 20,
            "energy" => self.battle.as_mut().expect("battle").energy += 2,
            "poison" => self.battle.as_mut().expect("battle").foes[foe_index.expect("target")].poison += 12,
            "strength" => self.battle.as_mut().expect("battle").strength += 3,
            "insight" => self.draw_cards(3),
            "weak" => for foe in &mut self.battle.as_mut().expect("battle").foes {
                if foe.hp > 0 { foe.weak += 3; }
            },
            "cleanse" => {
                self.hp = (self.hp + 8).min(self.max_hp);
                let b = self.battle.as_mut().expect("battle");
                b.weak = 0; b.vulnerable = 0;
            }
            "blood" => {
                self.hp = (self.hp - 4).max(1);
                self.battle.as_mut().expect("battle").strength += 5;
            }
            _ => return Err("未知药剂".into()),
        }
        if self.relics.iter().any(|r| r == "青囊") {
            self.hp = (self.hp + 4).min(self.max_hp);
        }
        if self.defeated() { Ok(format!("{label}。{}", self.win_battle())) }
        else { Ok(format!("使用药剂：{label}。")) }
    }
    /// Atomic command boundary: rejected actions never mutate the live game, RNG,
    /// revision, deck or potion slots. Frontends can perform CAS on revision.
    pub fn apply(&mut self, action: Action) -> Result<Vec<String>, String> {
        let mut draft = self.clone();
        let messages = draft.apply_unchecked(action)?;
        draft.validate()?;
        *self = draft;
        Ok(messages)
    }
    fn apply_unchecked(&mut self, action: Action) -> Result<Vec<String>, String> {
        let msg = match action {
            Action::Choose { index } => match self.stage {
                Stage::Map => self.choose_route(index)?,
                Stage::Reward => self.choose_reward(index)?,
                Stage::Camp => self.choose_camp(index)?,
                Stage::Market => self.choose_market(index)?,
                Stage::Story => self.choose_story(index)?,
                Stage::Ancient => self.choose_boon(index)?,
                _ => return Err("当前场景无法选择".into()),
            },
            Action::Play { index, target } => self.play(index, target)?,
            Action::EndTurn => self.end_turn()?,
            Action::Upgrade { index } => self.upgrade_card(index)?,
            Action::Enchant { index, kind } => self.enchant_card(index, &kind)?,
            Action::Discard { index } => self.discard_card(index)?,
            Action::SetVariants { act1, act2, act3 } => {
                if self.floor != 0 || self.stage != Stage::Map {
                    return Err("只能在进入第一层前设置章节变体".into());
                }
                if [act1,act2,act3].iter().any(|n| *n > 1) {
                    return Err("每幕仅允许变体0或1".into());
                }
                self.variants = Some([act1,act2,act3]);
                format!("自定义章节已选：{act1}/{act2}/{act3}")
            },
            Action::Remove { index } => self.remove_card(index)?,
            Action::UsePotionAt { slot, target } => self.use_potion(slot, target)?,
            Action::UsePotion => self.use_potion(1, 1)?,
        };
        self.revision = self.revision.saturating_add(1);
        self.log.push(msg.clone());
        if self.log.len() > 12 { self.log.remove(0); }
        self.validate()?;
        Ok(vec![msg])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_survives_mid_combat() {
        let mut game = Game::new(Hero::GuanYu, 2026);
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(game.stage, Stage::Combat);
        let snapshot = game.to_json().unwrap();
        let mut restored = Game::from_json(&snapshot).unwrap();
        assert_eq!(restored.rng_state, game.rng_state);
        assert_eq!(restored.battle.as_ref().unwrap().hand.len(), game.battle.as_ref().unwrap().hand.len());
        assert_eq!(restored.to_json().unwrap(), snapshot);
        game.apply(Action::EndTurn).unwrap();
        restored.apply(Action::EndTurn).unwrap();
        assert_eq!(game.to_json().unwrap(), restored.to_json().unwrap());
    }
    #[test]
    fn invalid_actions_do_not_spend_energy_or_advance_revision() {
        let mut game = Game::new(Hero::LuBu, 8);
        game.apply(Action::Choose { index: 1 }).unwrap();
        let previous = game.to_json().unwrap();
        assert!(game.apply(Action::Play { index: 999, target: 1 }).is_err());
        assert_eq!(previous, game.to_json().unwrap());
    }
    #[test]
    fn map_has_three_boss_floors() {
        let mut game = Game::new(Hero::CaoCao, 7);
        for floor in 1..=MAX_FLOOR {
            game.floor = floor - 1;
            game.refresh_route();
            assert_eq!(game.route.iter().all(|node| *node == Node::Boss),
                floor.is_multiple_of(7));
        }
    }
    #[test]
    fn corrupt_snapshot_rejected() {
        let game = Game::new(Hero::ZhugeLiang, 2);
        let json = game.to_json().unwrap().replace("\"version\": 2", "\"version\": 99");
        assert!(Game::from_json(&json).is_err());
    }

    #[test]
    fn every_original_card_is_playable_and_serializable() {
        let classes = [Hero::GuanYu, Hero::ZhugeLiang, Hero::SunShangxiang, Hero::CaoCao, Hero::LuBu];
        let mut all = vec!["strike", "defend"];
        all.extend(REWARD_POOL.iter().copied());
        all.extend(NEUTRAL_POOL.iter().copied());
        for hero in classes { all.extend(hero_pool(hero).iter().copied()); }
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len() + 3, content::card_count());
        for id in all {
            let mut g = Game::new(Hero::GuanYu, 42);
            g.apply(Action::Choose { index: 1 }).unwrap();
            let b = g.battle.as_mut().unwrap();
            b.energy = 99;
            b.hand = vec![Card::new(id)];
            for foe in &mut b.foes { foe.max_hp = 10000; foe.hp = 10000; }
            assert!(g.apply(Action::Play { index: 1, target: 1 }).is_ok(), "unplayable card {id}");
            let json = g.to_json().unwrap();
            assert!(Game::from_json(&json).is_ok(), "invalid snapshot after {id}");
        }
    }
    #[test]
    fn each_hero_passive_has_a_distinct_mechanical_effect() {
        let entries = [
            (Hero::GuanYu, "strike"), (Hero::ZhugeLiang, "defend"),
            (Hero::SunShangxiang, "strike"), (Hero::CaoCao, "defend"),
            (Hero::LuBu, "xuezhai"),
        ];
        for (hero, card) in entries {
            let mut g = Game::new(hero, 321);
            g.apply(Action::Choose { index: 1 }).unwrap();
            let b = g.battle.as_mut().unwrap();
            b.energy = 99;
            b.hand = vec![Card::new(card)];
            g.apply(Action::Play { index: 1, target: 1 }).unwrap();
            let b = g.battle.as_ref().unwrap();
            assert!(match hero {
                Hero::GuanYu => b.momentum > 0,
                Hero::ZhugeLiang => b.scheme > 0,
                Hero::SunShangxiang => b.combo > 0,
                Hero::CaoCao => b.command > 0,
                Hero::LuBu => b.fury > 0,
            });
        }
    }
    #[test]
    fn v1_saves_migrate_without_losing_battle_state() {
        let mut g = Game::new(Hero::SunShangxiang, 51);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let mut raw: serde_json::Value = serde_json::from_str(&g.to_json().unwrap()).unwrap();
        raw["version"] = serde_json::json!(1);
        for field in ["ascension", "event_id", "potion_bag", "shop_relic", "relic_history", "victories"] {
            raw.as_object_mut().unwrap().remove(field);
        }
        for field in ["vulnerable", "momentum", "scheme", "combo", "command", "fury", "thorns", "played"] {
            raw["battle"].as_object_mut().unwrap().remove(field);
        }
        for enemy in raw["battle"]["foes"].as_array_mut().unwrap() {
            enemy.as_object_mut().unwrap().remove("weak");
        }
        let restored = Game::from_json(&serde_json::to_string(&raw).unwrap()).unwrap();
        assert_eq!(restored.version, SAVE_VERSION);
        assert_eq!(restored.battle.as_ref().unwrap().hand.len(), g.battle.as_ref().unwrap().hand.len());
        assert_eq!(restored.potion_bag, vec!["healing".to_string()]);
    }
    #[test]
    fn relics_are_unique_and_potions_are_consumed() {
        let mut g = Game::new(Hero::CaoCao, 771);
        for _ in 0..items::RELICS.len() { g.grant_relic(); }
        let unique: std::collections::HashSet<_> = g.relics.iter().collect();
        assert_eq!(unique.len(), g.relics.len());
        g.apply(Action::Choose { index: 1 }).unwrap();
        g.potion_bag = vec!["shield".into(), "poison".into()];
        g.potions = 2;
        g.apply(Action::UsePotionAt { slot: 1, target: 1 }).unwrap();
        assert!(g.battle.as_ref().unwrap().block >= 20);
        assert_eq!(g.potion_bag, vec!["poison".to_string()]);
        g.apply(Action::UsePotionAt { slot: 1, target: 1 }).unwrap();
        assert_eq!(g.potions, 0);
    }
    #[test]
    fn alternate_act_bosses_and_enemy_catalogue_are_valid() {
        for id in content::ENEMY_IDS {
            assert!(content::enemy_hp(id) > 0);
            assert_ne!(content::enemy_name(id), "无名敌军");
        }
        for seed in [2_u64, 3_u64] {
            let mut g = Game::new(Hero::GuanYu, seed);
            for floor in [7, 14, 21] {
                g.floor = floor;
                let foes = g.generate_foes(Node::Boss);
                assert_eq!(foes.len(), 1);
                assert!(foes[0].max_hp >= 75);
            }
        }
    }

    #[test]
    fn card_enchantments_reduce_energy_cost_and_survive_resume() {
        let mut g = Game::new(Hero::GuanYu, 33);
        g.stage = Stage::Camp;
        g.apply(Action::Enchant { index: 1, kind: "迅捷".into() }).unwrap();
        assert_eq!(g.gold, 55);
        assert_eq!(g.deck[0].enchantment.as_deref(), Some("迅捷"));
        let mut restored = Game::from_json(&g.to_json().unwrap()).unwrap();
        restored.apply(Action::Choose { index: 1 }).unwrap();
        let b = restored.battle.as_mut().unwrap();
        b.hand = vec![restored.deck[0].clone()];
        let prior_energy = b.energy;
        restored.apply(Action::Play { index: 1, target: 1 }).unwrap();
        assert_eq!(restored.battle.as_ref().unwrap().energy, prior_energy);
    }
    #[test]
    fn quest_transforms_after_three_story_nodes() {
        let mut g = Game::new(Hero::ZhugeLiang, 77);
        g.deck.push(Card::new("quest_tulu"));
        for progress in 1..=3 {
            g.route = vec![Node::Story];
            g.route_lanes = vec![0];
            g.apply(Action::Choose { index: 1 }).unwrap();
            if progress < 3 {
                assert_eq!(g.deck.last().unwrap().quest_progress, progress);
            }
            g.apply(Action::Choose { index: 3 }).unwrap();
            assert_ne!(g.stage, Stage::Combat);
        }
        assert_eq!(g.deck.last().unwrap().id, "quest_resolved");
    }

    #[test]
    fn cao_cao_troops_block_damage_and_attack() {
        let mut g = Game::new(Hero::CaoCao, 123);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let b = g.battle.as_ref().unwrap();
        assert_eq!(b.troops_hp, 12);
        assert_eq!(b.troops_attack, 3);
        let enemy_hp = b.foes[0].hp;
        g.troop_attack();
        assert_eq!(g.battle.as_ref().unwrap().foes[0].hp, enemy_hp - 3);
        let hp_before = g.hp;
        let dmg = g.enemy_attack(0, 8);
        assert_eq!(dmg, 0);
        assert_eq!(g.hp, hp_before);
        assert!(g.battle.as_ref().unwrap().troops_hp < 12);
    }
    #[test]
    fn sun_shangxiang_discard_trigger_and_resume() {
        let mut g = Game::new(Hero::SunShangxiang, 888);
        g.apply(Action::Choose { index: 1 }).unwrap();
        let b = g.battle.as_mut().unwrap();
        b.discard_tokens = 1;
        b.hand = vec![Card::new("feishi")];
        let before = b.foes[0].hp;
        g.apply(Action::Discard { index: 1 }).unwrap();
        assert!(g.battle.as_ref().is_some_and(|b| b.foes[0].hp < before));
        assert!(Game::from_json(&g.to_json().unwrap()).is_ok());
    }

    #[test]
    fn rejected_actions_rollback_all_mutations_including_rng() {
        let mut g = Game::new(Hero::SunShangxiang, 15);
        g.stage = Stage::Market;
        g.market = g.pick_reward_cards();
        g.gold = 0;
        let before = g.to_json().unwrap();
        assert!(g.apply(Action::Choose { index: 4 }).is_err());
        assert_eq!(g.to_json().unwrap(), before);
        g.stage = Stage::Combat;
        g.start_battle(Node::Battle);
        g.potion_bag = vec!["unknown".into()];
        g.potions = 1;
        let before = g.to_json().unwrap();
        assert!(g.apply(Action::UsePotionAt { slot: 1, target: 1 }).is_err());
        assert_eq!(g.to_json().unwrap(), before);
    }

    #[test]
    fn boss_reward_flows_into_ancient_choice_without_losing_save() {
        let mut g = Game::new(Hero::GuanYu, 444);
        g.floor = 7;
        g.last_node = Some(Node::Boss);
        g.stage = Stage::Combat;
        g.battle = None;
        let _ = g.win_battle();
        assert_eq!(g.stage, Stage::Reward);
        g.apply(Action::Choose { index: 0 }).unwrap();
        assert_eq!(g.stage, Stage::Ancient);
        assert_eq!(g.boon_choices.len(), 3);
        let snap = g.to_json().unwrap();
        let mut restored = Game::from_json(&snap).unwrap();
        restored.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(restored.stage, Stage::Map);
        assert_eq!(restored.boons.len(), 1);
    }

    #[test]
    fn act_variants_select_distinct_bosses_and_save_across_resume() {
        let mut g = Game::new(Hero::GuanYu, 1);
        g.apply(Action::SetVariants { act1: 1, act2: 1, act3: 1 }).unwrap();
        for (floor,id) in [(7,"sunjian"),(14,"yuanshao"),(21,"zuoci")] {
            g.floor = floor;
            assert_eq!(g.generate_foes(Node::Boss)[0].id,id);
        }
        let before = g.to_json().unwrap();
        let loaded = Game::from_json(&before).unwrap();
        assert_eq!(loaded.to_json().unwrap(), before);
        g.floor = 1;
        assert!(g.apply(Action::SetVariants { act1:0,act2:0,act3:0 }).is_err());
    }

    #[test]
    fn six_bosses_transition_once_and_preserve_summons_in_snapshot() {
        for (id,ally) in [
            ("zhangjiao",true),("sunjian",false),("dongzhuo",false),
            ("yuanshao",true),("simayi",false),("zuoci",true),
        ] {
            let mut g = Game::new(Hero::GuanYu, 88);
            g.floor = 7;
            g.start_battle(Node::Boss);
            let b = g.battle.as_mut().unwrap();
            let hp = enemy_hp(id);
            b.foes[0] = Foe {
                id: id.into(), hp: hp/2, max_hp: hp, block: 0,
                strength: 0, poison: 0, vulnerable: 0, weak: 0,
                intent: Intent::Guard(0), phase: 0,
            };
            assert!(!g.boss_phase_transitions().is_empty());
            assert_eq!(g.battle.as_ref().unwrap().foes[0].phase,1);
            assert_eq!(g.battle.as_ref().unwrap().foes.len() > 1,ally);
            assert!(g.boss_phase_transitions().is_empty());
            let snapshot = g.to_json().unwrap();
            assert_eq!(Game::from_json(&snapshot).unwrap().to_json().unwrap(),snapshot);
        }
    }

    #[test]
    fn connected_map_lanes_reject_nonadjacent_moves_and_resume() {
        let mut g = Game::new(Hero::GuanYu,111);
        assert_eq!(g.route.len(),3);
        assert!(g.act_map.as_ref().is_some_and(|m| m.is_valid()));
        assert_eq!(g.route_lanes,vec![0,1,2]);
        let next = g.act_map.as_ref().unwrap().rows[1];
        g.apply(Action::Choose { index: 1 }).unwrap();
        // Simulate completing the scene without modifying the next row topology.
        g.battle = None;
        g.finish_scene();
        assert_eq!(g.route_lanes,vec![0,1]);
        assert_eq!(g.route,vec![next[0],next[1]]);
        let json = g.to_json().unwrap();
        assert_eq!(Game::from_json(&json).unwrap().to_json().unwrap(),json);
    }
    #[test]
    fn targeted_camp_upgrade_and_shop_removal_work() {
        let mut g = Game::new(Hero::ZhugeLiang, 8);
        g.stage = Stage::Camp;
        g.apply(Action::Upgrade { index: 10 }).unwrap();
        assert!(g.deck[9].upgraded);
        g.stage = Stage::Market;
        g.apply(Action::Remove { index: 1 }).unwrap();
        assert_eq!(g.deck.len(), 9);
        assert_eq!(g.gold, 15);
    }
}
