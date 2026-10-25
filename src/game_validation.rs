//! Save and live-state invariants, extracted from the oversized rules engine.
use crate::*;

impl Game {
    pub fn validate(&self) -> Result<(), String> {
        if self.hp < 0 || self.hp > self.max_hp || self.max_hp <= 0 || self.gold < 0 || self.floor > MAX_FLOOR
            || self.ascension > 10 || self.potions > 3 || self.potion_bag.len() > 3
            || usize::from(self.potions) != self.potion_bag.len()
            || self.potion_bag.iter().any(|id| !items::POTIONS.iter().any(|(valid,_)| valid == id))
        {
            return Err("存档属性无效".into());
        }
        if self.action_history.as_ref().is_some_and(|actions|actions.len()>50_000) {
            return Err("操作日志长度超限".into());
        }
        if self.rng_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("rng journal exceeds action limit".into());
        }
        if self.resource_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("resource journal exceeds action limit".into());
        }
        if self.battle_resource_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("combat resource journal exceeds action limit".into());
        }
        if self.foe_board_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("enemy board journal exceeds action limit".into());
        }
        if self.card_zone_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("card zone journal exceeds action limit".into());
        }
        if self.combat_effect_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("combat effect journal exceeds action limit".into());
        }
        if self.battle_runtime_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("battle runtime journal exceeds action limit".into());
        }
        if self.campaign_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("campaign journal exceeds action limit".into());
        }
        if self.telemetry_history.as_ref().is_some_and(|events| events.len()>50_001) {
            return Err("telemetry journal exceeds action limit".into());
        }
        if self.profile_start_receipt && self.run_id.is_empty() {
            return Err("可恢复战功凭证缺少唯一战役ID".into());
        }
        if (self.stage == Stage::Combat) != self.battle.is_some() {
            return Err("存档战斗阶段与战斗快照不一致".into());
        }
        if (self.mode != RunMode::Custom && !self.modifiers.is_empty())
            || self.modifiers.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err("挑战修饰符仅适用于自由模式，并且不能重复或乱序".into());
        }
        if self.doctrine.is_some_and(|focus| focus.hero() != self.hero) {
            return Err("当前战法与武将不匹配".into());
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
        if self.easter_unlocked.iter().any(|id| !lore::EASTER_EGGS.iter().any(|egg| egg.id == id))
        { return Err("未知演义彩蛋".into()); }
        if self.lore_flags.iter().any(|flag| !lore::BEATS.iter().any(|b| b.flag == flag))
            || self.lore_seen.iter().any(|id| lore::beat(id).is_none())
            || self.lore_current.as_deref().is_some_and(|id| lore::beat(id).is_none())
        { return Err("无效人物羁绊与剧情存档".into()); }
        if self.lore_current.is_some() && self.stage != Stage::Story {
            return Err("仅奇遇场景可拥有当前演义剧情".into());
        }
        for boon in self.boons.iter().chain(self.boon_choices.iter()) {
            if !items::BOONS.iter().any(|(id,_)| id == boon) { return Err(format!("未知赐福: {boon}")); }
        }
        for c in self.deck.iter().chain(self.rewards.iter()).chain(self.market.iter()) {
            if c.temporary { return Err("永久卡牌不能携带临时标志".into()); }
            if card_spec(&c.id).is_none() { return Err(format!("未知卡牌: {}", c.id)); }
            if c.enchantment.as_deref().is_some_and(|id|
                !matches!(id, "迅捷" | "锐锋" | "护身" | "明悟")) {
                return Err(format!("未知附魔: {:?}", c.enchantment));
            }
        }
        if self.telemetry.recent_turns.len() > MAX_RECENT_TURN_METRICS
            || self.telemetry.recent_turns.iter().any(|entry|
                entry.floor > MAX_FLOOR || entry.hero_hp_after < 0 || entry.troop_hp_after < 0)
        { return Err("战斗统计记录无效".into()); }
        if let Some(history)=&self.event_history {
            if history.len()>1_000_000 {return Err("event history exceeds size limit".into());}
            event_stack::verify_record(history)?;
        }
        if let Some(b) = &self.battle {
            if let Some(archive)=&self.event_history {
                // A complete archive must agree byte-for-byte with any retained
                // current-battle audit window; sequence numbers alone are insufficient.
                if !b.events.is_empty() {
                    let start=archive.len().checked_sub(b.events.len())
                        .ok_or("event archive shorter than battle window")?;
                    if !archive[start..].iter().zip(&b.events).all(|(entry,event)|
                        entry.encounter==self.telemetry.totals.battles_started
                        && entry.floor==self.floor && entry.event==*event) {
                        return Err("battle window differs from event archive".into());
                    }
                }
            }
            if !b.reactions.valid() {return Err("invalid listener stack".into());}
            if self.event_history.as_ref().is_some_and(|history|
                b.event_seq>0 && history.last().is_none_or(|e|
                    e.encounter!=self.telemetry.totals.battles_started
                    || e.event.seq!=b.event_seq))
            {return Err("current battle and event archive disagree".into());}
            for c in b.draw.iter().chain(b.hand.iter()).chain(b.discard.iter()).chain(b.exhausted.iter()) {
                if card_spec(&c.id).is_none() { return Err(format!("未知战斗卡牌: {}", c.id)); }
            }
            if b.energy < 0 || b.block < 0 || b.troops_hp < 0 || b.troops_attack < 0
                || b.discard_tokens < 0 || b.foes.is_empty() || b.foes.len() > 4
                || b.next_turn_energy_penalty > 2
                || b.powers.len() > 8 || b.powers.iter().any(|power| power.stacks <= 0 || power.stacks > 99)
                || b.powers.iter().enumerate().any(|(idx,p)| b.powers[..idx].iter().any(|prior| prior.id == p.id))
                || b.events.len() > MAX_RECENT_EVENTS
                || b.events.iter().any(|event| event.seq == 0 || event.seq > b.event_seq)
                || b.events.windows(2).any(|pair| pair[0].seq.checked_add(1) != Some(pair[1].seq))
                || b.events.last().is_some_and(|event| event.seq != b.event_seq)
                || b.momentum < 0 || b.scheme < 0 || b.combo < 0 || b.command < 0 || b.fury < 0
                || b.thorns < 0 || b.foes.iter().any(|f| f.max_hp <= 0 || f.hp < 0 || f.hp > f.max_hp || f.block < 0 || f.poison < 0 || f.phase > 1)
            { return Err("无效战斗数据".into()); }
        }
        Ok(())
    }
}
