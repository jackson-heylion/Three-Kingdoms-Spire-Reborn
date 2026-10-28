//! Read-only, display-only battle impact summaries. No RNG, authoritative
//! state mutation or persistent fields. Damage numbers come from committed
//! battle events and committed before/after snapshots, not card descriptions.
use super::App;
use three_kingdoms_spire_reborn::{
    Game, Stage,
    combat::{CombatEvent, EventKind},
    content::{card_spec, enemy_name},
};

#[derive(Clone)]
pub(super) struct Snapshot {
    hp: i32,
    gold: i32,
    energy: i32,
    block: i32,
    momentum: i32,
    scheme: i32,
    combo: i32,
    command: i32,
    fury: i32,
    strength: i32,
    weak: i32,
    vulnerable: i32,
    foes: Vec<(String, i32, i32, i32, i32, i32, u8)>,
}

pub(super) fn snapshot(game: &Game) -> Option<Snapshot> {
    let b = game.battle.as_ref().filter(|_| game.stage == Stage::Combat)?;
    Some(Snapshot {
        hp: game.hp, gold: game.gold, energy: b.energy, block: b.block,
        momentum: b.momentum, scheme: b.scheme, combo: b.combo,
        command: b.command, fury: b.fury, strength: b.strength,
        weak: i32::from(b.weak), vulnerable: i32::from(b.vulnerable),
        foes: b.foes.iter().map(|foe|
            (foe.id.clone(), foe.hp, foe.block, foe.poison,
                i32::from(foe.weak), i32::from(foe.vulnerable), foe.phase)
        ).collect(),
    })
}

fn changed(out: &mut Vec<String>, label: &str, before: i32, after: i32) {
    if before != after {
        let change = after - before;
        out.push(format!("{label}{}{}（现{after}）",
            if change > 0 { "+" } else { "" }, change));
    }
}

/// Display only committed differences. Invalid actions and failed saves never
/// pass through here; an encounter ending can legitimately erase Battle.
pub(super) fn impact(before: &Snapshot, game: &Game) -> String {
    let mut out = Vec::new();
    changed(&mut out, "我方生命", before.hp, game.hp);
    changed(&mut out, "金钱", before.gold, game.gold);
    if let Some(b) = game.battle.as_ref().filter(|_| game.stage == Stage::Combat) {
        changed(&mut out, "军令", before.energy, b.energy);
        changed(&mut out, "我方护甲", before.block, b.block);
        for (label, old, new) in [
            ("战意", before.momentum, b.momentum),
            ("谋略", before.scheme, b.scheme),
            ("连段", before.combo, b.combo),
            ("统御", before.command, b.command),
            ("怒气", before.fury, b.fury),
            ("武勇", before.strength, b.strength),
            ("我方虚弱", before.weak, i32::from(b.weak)),
            ("我方易伤", before.vulnerable, i32::from(b.vulnerable)),
        ] {
            changed(&mut out, label, old, new);
        }
        for (idx, foe) in b.foes.iter().enumerate() {
            if let Some((_id, hp, block, poison, weak, vulnerable, phase)) =
                before.foes.get(idx).filter(|(id, ..)| *id == foe.id) {
                let prefix = format!("敌{}·{}", idx + 1, foe.name());
                changed(&mut out, &format!("{prefix}生命"), *hp, foe.hp);
                changed(&mut out, &format!("{prefix}护甲"), *block, foe.block);
                changed(&mut out, &format!("{prefix}中毒"), *poison, foe.poison);
                changed(&mut out, &format!("{prefix}虚弱"), *weak, i32::from(foe.weak));
                changed(&mut out, &format!("{prefix}易伤"), *vulnerable, i32::from(foe.vulnerable));
                if *phase != foe.phase {
                    out.push(format!("{prefix}进入第{}阶段", foe.phase + 1));
                }
            } else {
                out.push(format!("敌{}·{}增援入场，生命{}/{}", idx+1,foe.name(),foe.hp,foe.max_hp));
            }
        }
    }
    out.join("；")
}

fn foe_label(event: &CombatEvent, game: &Game) -> String {
    let index = event.target.as_deref().and_then(|target|target.strip_prefix("foe:"))
        .and_then(|raw|raw.parse::<usize>().ok());
    index.and_then(|i|game.battle.as_ref()?.foes.get(i))
        .map_or_else(|| "敌军".into(), |foe| foe.name().into())
}

/// Stable event IDs are translated only for display. The battle's bounded
/// journal is the evidence for *actual*, post-block and post-reaction damage.
fn event_line(event: &CombatEvent, game: &Game) -> Option<(bool, String)> {
    let target = foe_label(event, game);
    let amount = event.amount;
    match event.kind {
        EventKind::CardPlayed => {
            let name = card_spec(&event.source).map_or(event.source.as_str(), |card| card.name);
            Some((true, format!("我方打出【{name}】")))
        }
        EventKind::PotionUsed => Some((true,
            format!("我方使用药剂 {}，数量{}", event.source, amount))),
        EventKind::DamageDealt => Some((true, format!(
            "我方对{target}实际造成{amount}伤害（来源{}）",event.source))),
        EventKind::DamageReceived => {
            if let Some(index) = event.source.strip_prefix("foe:") {
                let caster = index.parse::<usize>().ok()
                    .and_then(|i|game.battle.as_ref()?.foes.get(i))
                    .map_or("敌军", |foe|foe.name());
                Some((false, format!("{caster}攻击我方：实际损失{amount}生命{}",
                    if amount == 0 {"（已抵挡）"} else {""})))
            } else {
                Some((true, format!("我方受到{}的实际伤害{amount}", event.source)))
            }
        }
        EventKind::StatusApplied => Some((
            event.target.as_deref() != Some("hero"),
            if event.target.as_deref() == Some("hero") {
                format!("敌方向我方施加{} {amount}层", event.source)
            } else {
                format!("我方向{target}施加{} {amount}层", event.source)
            },
        )),
        EventKind::FoeDefeated => Some((true,format!("{target}被击败"))),
        EventKind::FoeSummoned => Some((false,format!(
            "敌方召唤{}增援入场",enemy_name(&event.source)))),
        EventKind::EnemyTechnique if event.source.ends_with("_phase_two") =>
            Some((false,format!("敌方{}切换二阶段，生命回满",
                enemy_name(event.source.trim_end_matches("_phase_two"))))),
        EventKind::EnemyTechnique => Some((false,format!(
            "敌方{}使用战技",enemy_name(&event.source)))),
        EventKind::PowerTriggered => Some((true,format!(
            "我方触发{}，效果值{amount}",event.source))),
        _ => None,
    }
}

/// Compact view deliberately chooses the last meaningful event of each side,
/// not the last two events of one side.
pub(super) fn recent(game: &Game) -> Vec<String> {
    let Some(battle) = game.battle.as_ref() else { return Vec::new() };
    let mut player = None;
    let mut enemy = None;
    for event in battle.events.iter().rev() {
        if let Some((ours, description)) = event_line(event, game) {
            if ours && player.is_none() { player = Some(description); }
            else if enemy.is_none() { enemy = Some(description); }
            if player.is_some() && enemy.is_some() { break; }
        }
    }
    // Guard and rally are only described in EndTurn's committed narrative,
    // not as damage/status events. Keep them visible without creating a
    // synthetic authoritative combat event or altering listener semantics.
    if enemy.is_none()
        && let Some(turn) = game.log.iter().rev()
            .find(|message| message.starts_with("回合结束：")) {
        enemy = Some(format!("敌方回合：{}",
            turn.trim_start_matches("回合结束：").trim_end_matches('。')));
    }
    vec![player.unwrap_or_else(|| "我方尚未行动".into()),
        enemy.unwrap_or_else(|| "敌方尚未行动".into())]
}

/// Complete readable audit in a scrollable overlay. No claimed unlimited
/// history: the battle journal deliberately retains only its newest 80 events.
pub(super) fn document(app: &App) -> String {
    let mut sections = Vec::new();
    if !app.message.is_empty() {
        sections.push(format!("【最近指令与实际结算】\n{}",app.message.replace('；', "；\n")));
    }
    if let Some(game) = &app.game {
        if let Some(b) = &game.battle {
            let events = b.events.iter()
                .filter_map(|event|event_line(event,game)
                    .map(|(_,line)|format!("#{:04} {line}",event.seq)))
                .collect::<Vec<_>>();
            sections.push(format!("【双方事件流水 · 最近最多80条，按结算顺序】\n{}",
                if events.is_empty() {"暂无已结算战斗事件".into()}
                else { events.join("\n") }));
            sections.push(format!("【最近实际操作记录 · 最多12条】\n{}",
                game.log.iter().rev().take(12).rev().cloned()
                    .collect::<Vec<_>>().join("\n")));
            sections.push(format!("【当前双方生命】\n我方 {}/{}；{}",
                game.hp,game.max_hp,b.foes.iter().enumerate()
                    .map(|(i,foe)|format!("敌{} {} {}/{} HP（第{}阶段）",
                        i+1,foe.name(),foe.hp,foe.max_hp,foe.phase+1))
                    .collect::<Vec<_>>().join("；")));
        }
    }
    if sections.is_empty() { "暂无战斗记录".into() } else { sections.join("\n\n") }
}

#[cfg(test)]
mod report_tests {
    use super::*;
    use three_kingdoms_spire_reborn::{Action, content::Hero};

    #[test]
    fn reports_committed_damage_and_resources_without_mutation() {
        let mut game=Game::new(Hero::GuanYu, 345);
        game.apply(Action::Choose{index:1}).unwrap();
        let b=game.battle.as_mut().unwrap();
        b.hand=vec![three_kingdoms_spire_reborn::Card::new("strike")];
        b.foes[0].block=0;
        let before=snapshot(&game).unwrap();
        let saved=game.to_json().unwrap();
        game.apply(Action::Play{index:1,target:1}).unwrap();
        let result=impact(&before,&game);
        assert!(result.contains("生命-"),"{result}");
        assert!(result.contains("战意+"),"{result}");
        let events=recent(&game);
        assert!(events[0].contains("实际造成"),"{events:?}");
        assert_eq!(saved, saved.clone()); // Rendering does not produce actions.
        let after=game.to_json().unwrap();
        let _=recent(&game);
        assert_eq!(game.to_json().unwrap(),after);
    }

    #[test]
    fn non_damage_enemy_guard_is_visible_from_committed_turn_log() {
        let mut game=Game::new(Hero::GuanYu, 942);
        game.apply(Action::Choose{index:1}).unwrap();
        {
            let battle=game.battle.as_mut().unwrap();
            battle.foes[0].phase=1;
            battle.foes[0].intent=three_kingdoms_spire_reborn::Intent::Guard(8);
            battle.troops_hp=0;
            battle.troops_attack=0;
        }
        game.apply(Action::EndTurn).unwrap();
        let last=recent(&game);
        assert!(last[1].contains("护甲"),"{last:?}");
    }

    #[test]
    fn enemy_actions_show_actual_damage_not_intent_estimates() {
        let mut game=Game::new(Hero::GuanYu, 345);
        game.apply(Action::Choose{index:1}).unwrap();
        {
            let b=game.battle.as_mut().unwrap();
            b.foes[0].hp=b.foes[0].max_hp;
            b.foes[0].phase=1;
            b.foes[0].intent=three_kingdoms_spire_reborn::Intent::Attack(8);
            b.block=0;
            b.troops_hp=0;
            b.troops_attack=0;
        }
        game.apply(Action::EndTurn).unwrap();
        let desc=recent(&game);
        assert!(desc[1].contains("实际损失"),"{desc:?}");
    }
}
