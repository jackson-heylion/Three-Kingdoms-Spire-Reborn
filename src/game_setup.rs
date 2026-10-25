//! Campaign creation and map regeneration, extracted from the large engine.
use crate::{Game, Card, Stage, RunMode, SAVE_VERSION, MAX_FLOOR, map,
    resource_events, rng_events, combat_resource_events, foe_board_events,
    card_zone_events, combat_effect_events, campaign_authority, CombatTelemetry};
use crate::content::Hero;

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
            rng_trace: None, dispatch_parent: None, event_parent_trace: None,
            rng_history: Some(vec![]),
            hero, hp: max_hp, max_hp, gold: 90, floor: 0, stage: Stage::Map,
            last_node: None, route: vec![], deck: deck.clone(), relics: vec!["军旗".into()],
            potions: 1, battle: None, action_history: Some(vec![]), event_history: Some(vec![]),
            resource_history: Some(vec![resource_events::ResourceEvent::Started {
                state: resource_events::Resources { hp: max_hp, max_hp, gold: 90 },
            }]), battle_resource_history: Some(vec![combat_resource_events::BattleResourceEvent::Started]),
            foe_board_history: Some(vec![foe_board_events::FoeBoardEvent::Started]),
            card_zone_history: Some(vec![card_zone_events::CardZoneEvent::Started { deck }]),
            combat_effect_history: Some(vec![combat_effect_events::CombatEffectEvent::Started]),
            campaign_history: None,
            modifiers: vec![], doctrine: None, telemetry: CombatTelemetry::default(), rewards: vec![], market: vec![],
            log: vec![format!("{}率部出征，征途由此开始。", hero.name())],
            ascension: ascension.min(10), event_id: 0,
            potion_bag: vec!["healing".into()], shop_relic: None,
            relic_history: vec!["军旗".into()], victories: 0,
            boons: vec![], boon_choices: vec![], variants: None,
            mode: RunMode::Standard, run_id: String::new(), profile_start_receipt: false,
            act_map: None, map_lane: None, route_lanes: Vec::new(),
            lore_flags: vec![], lore_seen: vec![], lore_current: None,
            easter_unlocked: Vec::new(),
        };
        game.refresh_route();
        // Map generation consumes RNG before revision zero is committed.
        game.rng_history = Some(vec![rng_events::RngEvent::Started {
            state: game.rng_state,
        }]);
        game.campaign_history = Some(vec![campaign_authority::CampaignEvent::Started {
            state: campaign_authority::CampaignState::current(&game),
        }]);
        game
    }
    pub(crate) fn refresh_route(&mut self) {
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
}
