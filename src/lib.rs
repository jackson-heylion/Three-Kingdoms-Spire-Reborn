//! UI-free deterministic game engine. Safe to reuse from Swift/UniFFI and wasm.
//! Every authoritative mutation goes through Game::apply(Action).
pub mod content;

use content::{Effect, Hero, REWARD_POOL, NEUTRAL_POOL, hero_pool, card_spec, enemy_name};
use serde::{Deserialize, Serialize};

pub const SAVE_VERSION: u32 = 2;
pub const MAX_FLOOR: u8 = 21;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub upgraded: bool,
}
impl Card {
    pub fn new(id: &str) -> Self {
        Self { id: id.into(), upgraded: false }
    }
    pub fn name(&self) -> String {
        let base = card_spec(&self.id).map_or("未知牌", |s| s.name);
        format!("{base}{}", if self.upgraded { "+" } else { "" })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage { Map, Combat, Reward, Camp, Market, Story, Won, Lost }

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
pub enum Intent { Attack(i32), Guard(i32), Rally(i32) }
impl Intent {
    pub fn label(&self) -> String {
        match self {
            Self::Attack(n) => format!("进攻 {n}"), Self::Guard(n) => format!("防御 {n}"),
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
}

impl Game {
    pub fn new(hero: Hero, seed: u64) -> Self { Self::new_with_ascension(hero, seed, 0) }
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
        if self.hp < 0 || self.hp > self.max_hp || self.max_hp <= 0 || self.floor > MAX_FLOOR || self.ascension > 10 || self.potions > 3 || self.potion_bag.len() > 3 {
            return Err("存档属性无效".into());
        }
        if (self.stage == Stage::Combat) != self.battle.is_some() {
            return Err("存档战斗阶段与战斗快照不一致".into());
        }
        for c in self.deck.iter().chain(self.rewards.iter()).chain(self.market.iter()) {
            if card_spec(&c.id).is_none() { return Err(format!("未知卡牌: {}", c.id)); }
        }
        if let Some(b) = &self.battle {
            for c in b.draw.iter().chain(b.hand.iter()).chain(b.discard.iter()).chain(b.exhausted.iter()) {
                if card_spec(&c.id).is_none() { return Err(format!("未知战斗卡牌: {}", c.id)); }
            }
            if b.energy < 0 || b.block < 0 || b.foes.is_empty() || b.foes.len() > 4
                || b.momentum < 0 || b.scheme < 0 || b.combo < 0 || b.command < 0 || b.fury < 0
                || b.thorns < 0 || b.foes.iter().any(|f| f.hp < 0 || f.hp > f.max_hp || f.block < 0 || f.poison < 0)
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
        if next.is_multiple_of(7) {
            self.route = vec![Node::Boss];
            return;
        }
        let phases: &[Node] = if next % 7 == 1 {
            &[Node::Battle, Node::Story, Node::Camp]
        } else if next % 7 == 3 {
            &[Node::Battle, Node::Elite, Node::Market]
        } else if next % 7 == 5 {
            &[Node::Battle, Node::Camp, Node::Market]
        } else {
            &[Node::Battle, Node::Story, Node::Market, Node::Elite, Node::Camp]
        };
        // Consume the saved RNG only at floor transitions. This preserves
        // deterministic branching after resuming on another frontend.
        let offset = self.random(phases.len());
        self.route = (0..3).map(|i| phases[(offset + i) % phases.len()]).collect();
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
    fn generate_foes(&mut self, node: Node) -> Vec<Foe> {
        let act = (self.floor - 1) / 7;
        let ids: Vec<&str> = match node {
            Node::Boss => vec![match act { 0 => "zhangjiao", 1 => "dongzhuo", _ => "simayi" }],
            Node::Elite => vec!["elite"],
            _ => {
                let first = if act == 0 { "bandit" } else if act == 1 { "officer" } else { "shaman" };
                if self.floor > 2 && self.random(100) < 35 {
                    vec![first, "archer"]
                } else { vec![first] }
            }
        };
        ids.into_iter().map(|id| {
            let hp = match id {
                "zhangjiao" => 75, "dongzhuo" => 106, "simayi" => 142,
                "elite" => 58 + act as i32 * 14, "bandit" => 32,
                "archer" => 23 + act as i32 * 8,
                "officer" => 43, "shaman" => 53, _ => 30,
            };
            Foe {
                id: id.to_string(), hp, max_hp: hp, block: 0, strength: 0,
                poison: 0, vulnerable: 0, weak: 0, intent: Intent::Attack(6),
            }
        }).collect()
    }
    fn roll_intents(&mut self) {
        let n = self.battle.as_ref().map_or(0, |b| b.foes.len());
        for i in 0..n {
            let roll = self.random(100);
            let foe = &mut self.battle.as_mut().expect("battle").foes[i];
            if foe.hp <= 0 { continue; }
            let act = ((self.floor - 1) / 7) as i32;
            foe.intent = if roll < 65 {
                Intent::Attack(7 + act * 3 + foe.strength + if foe.id == "simayi" { 5 } else { 0 } + self.ascension as i32 / 3)
            } else if roll < 86 {
                Intent::Guard(6 + act * 4)
            } else {
                Intent::Rally(2)
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
            combo: 0, command: 0, fury: 0, thorns: 0, played: 0, turn: 1,
        });
        self.roll_intents();
        let extra_draw = usize::from(self.relics.iter().any(|r| r == "虎翼"));
        self.draw_cards(5 + extra_draw);
        if let Some(b) = &mut self.battle {
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
        self.refresh_route();
    }
    fn choose_route(&mut self, index: usize) -> Result<String, String> {
        let node = *self.route.get(index.checked_sub(1).ok_or("路线编号从1开始")?)
            .ok_or("无效路线编号")?;
        self.floor += 1;
        self.last_node = Some(node);
        let msg = format!("第{}层：{}", self.floor, node.name());
        match node {
            Node::Battle | Node::Elite | Node::Boss => self.start_battle(node),
            Node::Camp => self.stage = Stage::Camp,
            Node::Market => {
                self.market = self.pick_reward_cards();
                self.stage = Stage::Market;
            }
            Node::Story => {
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
    fn win_battle(&mut self) -> String {
        let node = self.last_node.unwrap_or(Node::Battle);
        self.battle = None;
        let payout = match node { Node::Boss => 100, Node::Elite => 60, _ => 24 + self.random(15) as i32 };
        self.gold += payout;
        if self.relics.iter().any(|r| r == "铜雀") { self.gold += 12; }
        if self.relics.iter().any(|r| r == "神医") {
            self.hp = (self.hp + 6).min(self.max_hp);
        }
        if node == Node::Elite || node == Node::Boss {
            self.grant_relic();
            if node == Node::Boss {
                self.max_hp += 5;
                self.hp = (self.hp + 5).min(self.max_hp);
            }
        }
        if self.potions < 3 && self.random(100) < 32 {
            let pool = ["healing", "fire", "shield", "energy", "poison", "strength", "insight"];
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
        if b.energy < spec.cost { return Err("军令不足".into()); }
        let requires_target = matches!(
            spec.effect, Effect::Hit(_) | Effect::HitDraw(_, _) | Effect::HitTwice(_)
                | Effect::Poison(_) | Effect::HitWeak(_, _) | Effect::HitVulnerable(_, _)
                | Effect::MomentumStrike(_) | Effect::SchemeStrike(_) | Effect::ComboStrike(_)
                | Effect::CommandStrike(_) | Effect::FuryStrike(_) | Effect::BloodPrice(_, _)
                | Effect::Leech(_) | Effect::HitPoison(_, _) | Effect::Pierce(_)
        );
        let target_idx = if requires_target { Some(self.target_index(target)?) } else { None };
        {
            let b = self.battle.as_mut().expect("battle");
            b.energy -= spec.cost;
            b.hand.remove(hand_idx);
        }
        let bonus = if card.upgraded { 3 } else { 0 };
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
                for i in 0..count {
                    if self.battle.as_ref().expect("battle").foes[i].hp > 0 {
                        self.damage_enemy(i, n + bonus);
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
            Effect::Unplayable => unreachable!(),
        }
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
            && self.battle.as_ref().expect("battle").combo.is_multiple_of(3) {
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
        let intents: Vec<(usize, Intent)> = self.battle.as_ref().expect("battle").foes.iter()
            .enumerate().filter(|(_, f)| f.hp > 0)
            .map(|(i, f)| (i, f.intent.clone())).collect();
        let mut messages = vec![];
        for (i, intent) in intents {
            let name = self.battle.as_ref().expect("battle").foes[i].name().to_string();
            match intent {
                Intent::Attack(power) => {
                    let b = self.battle.as_mut().expect("battle");
                    let foe = &b.foes[i];
                    let reduced = if foe.weak > 0 { power * 3 / 4 } else { power };
                    let modified = if b.vulnerable > 0 { reduced * 3 / 2 } else { reduced };
                    let blocked = modified.min(b.block);
                    b.block -= blocked;
                    let damage = (modified - blocked).max(0);
                    self.hp = (self.hp - damage).max(0);
                    if self.hero == Hero::LuBu { b.fury += damage / 2; }
                    if b.thorns > 0 { b.foes[i].hp = (b.foes[i].hp - b.thorns).max(0); }
                    messages.push(format!("{name}进攻，受到{damage}点伤害"));
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
            b.turn += 1;
            for foe in &mut b.foes {
                foe.vulnerable = foe.vulnerable.saturating_sub(1);
                foe.weak = foe.weak.saturating_sub(1);
                foe.block = 0;
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
        self.finish_scene();
        Ok(result)
    }
    fn choose_camp(&mut self, index: usize) -> Result<String, String> {
        let message = match index {
            1 => {
                let heal = (self.max_hp * 30 / 100).max(1);
                let actual = heal.min(self.max_hp - self.hp);
                self.hp += actual;
                format!("休整恢复{actual}点体力。")
            }
            2 => {
                let card = self.deck.iter_mut().find(|c| !c.upgraded && c.id != "cursed")
                    .ok_or("没有可升级的卡牌")?;
                card.upgraded = true;
                format!("磨砺兵器：升级【{}】。", card.name())
            }
            _ => return Err("营寨：choose 1 休整，choose 2 升级一张未升级卡牌".into()),
        };
        self.finish_scene();
        Ok(message)
    }
    fn choose_market(&mut self, index: usize) -> Result<String, String> {
        if index == 0 {
            self.finish_scene();
            return Ok("离开军市。".into());
        }
        let idx = index - 1;
        let cost = 45 + idx as i32 * 15;
        let card = self.market.get(idx).ok_or("无效商品编号")?.clone();
        if self.gold < cost { return Err(format!("金不足，需要{cost}。")); }
        self.gold -= cost;
        self.market.remove(idx);
        let msg = format!("花费{cost}金购买【{}】。", card.name());
        Ok(msg)
    }
    fn choose_story(&mut self, index: usize) -> Result<String, String> {
        let msg = match index {
            1 => {
                if self.gold < 15 { return Err("需要15金购得补给".into()); }
                self.gold -= 15;
                self.hp = (self.hp + 14).min(self.max_hp);
                "用15金向乡民购得粮草，回复14体力。".into()
            }
            2 => {
                self.deck.push(Card::new("huogong"));
                self.deck.push(Card::new("cursed"));
                "得古卷《火攻》，但军心涣散混入牌组。".into()
            }
            3 => {
                self.hp = (self.hp - 8).max(0);
                self.gold += 65;
                "冒险夺得军资65金，损失8体力。".into()
            }
            _ => return Err("奇遇：choose 1 粮草 / 2 古卷 / 3 劫粮".into()),
        };
        if self.hp == 0 { self.stage = Stage::Lost; } else { self.finish_scene(); }
        Ok(msg)
    }
    pub fn apply(&mut self, action: Action) -> Result<Vec<String>, String> {
        let msg = match action {
            Action::Choose { index } => match self.stage {
                Stage::Map => self.choose_route(index)?,
                Stage::Reward => self.choose_reward(index)?,
                Stage::Camp => self.choose_camp(index)?,
                Stage::Market => self.choose_market(index)?,
                Stage::Story => self.choose_story(index)?,
                _ => return Err("当前场景无法选择".into()),
            },
            Action::Play { index, target } => self.play(index, target)?,
            Action::EndTurn => self.end_turn()?,
            Action::UsePotion => {
                if self.stage != Stage::Combat { return Err("仅战斗中可使用药剂".into()); }
                if self.potions == 0 { return Err("没有药剂".into()); }
                self.potions -= 1;
                let healed = (self.max_hp - self.hp).min(16);
                self.hp += healed;
                format!("饮用军医药剂，回复{healed}体力。")
            }
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
            assert_eq!(game.route == vec![Node::Boss], floor.is_multiple_of(7));
        }
    }
    #[test]
    fn corrupt_snapshot_rejected() {
        let game = Game::new(Hero::ZhugeLiang, 2);
        let json = game.to_json().unwrap().replace("\"version\": 2", "\"version\": 99");
        assert!(Game::from_json(&json).is_err());
    }
}
