//! A compact, Unicode-cell-aware terminal HUD. The plain CLI and Game are untouched.
//! Color is presentation-only and never enters persisted state or user commands.
use super::{App, Stage, Theme, hud_reading};
use three_kingdoms_spire_reborn::content::Hero;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn width(text: &str) -> usize { UnicodeWidthStr::width(text) }

fn fit(text: &str, limit: usize) -> String {
    if limit == 0 { return String::new(); }
    let safe: String = text.chars().filter(|ch| !ch.is_control()).collect();
    if width(&safe) <= limit { return safe; }
    let mut out = String::new();
    let mut used = 0;
    for ch in safe.chars() {
        let cells = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + cells > limit.saturating_sub(1) { break; }
        out.push(ch);
        used += cells;
    }
    out.push('…');
    out
}

fn window(total: usize, cursor: usize, available: usize) -> (usize, usize) {
    if total == 0 { return (0, 0); }
    let count = available.max(1).min(total);
    let start = cursor.saturating_sub(count / 2).min(total - count);
    (start, start + count)
}

#[derive(Clone, Copy)]
enum Tone { Normal, Muted, Accent, Good, Warning, Danger, Target, Focus }
fn color(app: &App, text: &str, tone: Tone) -> String {
    if !app.use_ansi || matches!(tone, Tone::Normal) { return text.to_owned(); }
    let code = match app.theme {
        Theme::Dark => match tone {
            Tone::Normal => "97", Tone::Muted => "90", Tone::Accent => "1;96",
            Tone::Good => "1;92", Tone::Warning => "1;93",
            Tone::Danger => "1;91", Tone::Target => "1;38;5;215",
            Tone::Focus => "1;30;106",
        },
        Theme::Light => match tone {
            Tone::Normal => "30", Tone::Muted => "90", Tone::Accent => "1;34",
            Tone::Good => "1;32", Tone::Warning => "1;33",
            Tone::Danger => "1;31", Tone::Target => "1;38;2;145;64;0",
            Tone::Focus => "1;97;44",
        },
        Theme::HighContrast => match tone {
            Tone::Normal => "1;97", Tone::Muted => "1;37", Tone::Accent => "1;96",
            Tone::Good => "1;92", Tone::Warning => "1;93",
            Tone::Danger => "1;91", Tone::Target => "1;97;44",
            Tone::Focus => "1;30;103",
        },
    };
    format!("\x1b[{code}m{text}{}", app.theme.style())
}
fn line(out: &mut Vec<String>, app: &App, text: &str, columns: usize, tone: Tone) {
    out.push(color(app, &fit(text, columns), tone));
}
fn pieces(out: &mut Vec<String>, app: &App, columns: usize, spans: &[(&str, Tone)]) {
    let mut result = String::new();
    let mut remaining = columns;
    for &(text, tone) in spans {
        if remaining == 0 { break; }
        let clipped = fit(text, remaining);
        remaining = remaining.saturating_sub(width(&clipped));
        result.push_str(&color(app, &clipped, tone));
    }
    out.push(result);
}
fn heading(out: &mut Vec<String>, app: &App, title: &str, columns: usize) {
    let line_text = format!("─ {title} {}", "─".repeat(columns.saturating_sub(width(title) + 4)));
    line(out, app, &line_text, columns, Tone::Muted);
}

/// Always show the current hero's core resource, including zero.
fn primary_resource(hero: Hero, momentum: i32, scheme: i32, combo: i32,
    command: i32, fury: i32) -> (&'static str, i32) {
    match hero {
        Hero::GuanYu => ("战意", momentum),
        Hero::ZhugeLiang => ("谋略", scheme),
        Hero::SunShangxiang => ("连段", combo),
        Hero::CaoCao => ("统御", command),
        Hero::LuBu => ("怒气", fury),
    }
}

/// Pack complete labeled values into Unicode-cell-aware rows before the foe list.
/// Never let an inactive resource disappear merely because its value is zero.
fn status_rows(out: &mut Vec<String>, app: &App, columns: usize,
    tokens: &[(String, Tone)]) {
    let mut spans: Vec<(&str, Tone)> = Vec::new();
    let mut used = 0usize;
    for (label, tone) in tokens {
        let cells = width(label);
        if !spans.is_empty() && used + 2 + cells > columns {
            pieces(out, app, columns, &spans);
            spans.clear();
            used = 0;
        }
        if !spans.is_empty() {
            spans.push(("  ", Tone::Normal));
            used += 2;
        }
        spans.push((label.as_str(), *tone));
        used += cells;
    }
    if !spans.is_empty() {
        pieces(out, app, columns, &spans);
    }
}

fn menu_label(label: &str, combat: bool, details: bool) -> &str {
    if combat && !details { label.split_once(" · ").map_or(label, |(name, _)| name) }
    else { label }
}
fn focus_description(label: &str, combat: bool) -> Option<&str> {
    if combat { label.split_once(" · ").map(|(_, description)| description) }
    else { None }
}

#[cfg(test)]
pub(super) fn render(app: &App, columns: usize, rows: usize, target: usize,
    options: &[(String, String)], cursor: usize) -> String {
    render_mode(app, columns, rows, target, options, cursor, false)
}

/// "v" toggles optional diagnostics; the default keeps decisions and threats above details.
pub(super) fn render_mode(app: &App, columns: usize, rows: usize, target: usize,
    options: &[(String, String)], cursor: usize, details: bool) -> String {
    let columns = columns.clamp(28, 120);
    let rows = rows.max(16);
    let mut out = Vec::new();
    let title = if details { "三国登塔录  ·  详细视图" } else { "三国登塔录  ·  战术视图" };
    pieces(&mut out, app, columns, &[
        (title, Tone::Accent),
        ("  /  自动保存", Tone::Muted),
    ]);

    let mut combat = false;
    if let Some(g) = &app.game {
        let act = (g.floor.saturating_sub(1) / 7 + 1).min(3);
        let hp_tone = if g.hp * 4 <= g.max_hp { Tone::Danger }
            else if g.hp * 2 <= g.max_hp { Tone::Warning } else { Tone::Good };
        if columns < 48 {
            line(&mut out, app, &format!("{} · 第{}幕·{}层", g.hero.name(), act, g.floor),
                columns, Tone::Accent);
            pieces(&mut out, app, columns, &[
                (&format!("生命 {}/{}  ", g.hp, g.max_hp), hp_tone),
                (&format!("金 {}  药 {}", g.gold, g.potions), Tone::Muted),
            ]);
        } else {
            pieces(&mut out, app, columns, &[
                (&format!("{}  第{}幕·{}层  ", g.hero.name(), act, g.floor), Tone::Accent),
                (&format!("生命 {}/{}  ", g.hp, g.max_hp), hp_tone),
                (&format!("金 {}  药 {}", g.gold, g.potions), Tone::Muted),
            ]);
        }
        if details && rows >= 26 {
            line(&mut out, app, &format!("{:?} A{} · {} · 战法：{}",
                g.mode, g.ascension, g.act_name(),
                g.doctrine.map_or("自动", |d| d.label())), columns, Tone::Muted);
        }
        if let Some(b) = g.battle.as_ref().filter(|_| g.stage == Stage::Combat) {
            combat = true;
            let (core_name, core_value) = primary_resource(g.hero,
                b.momentum, b.scheme, b.combo, b.command, b.fury);
            let mut stats = vec![
                (format!("军令 {}", b.energy), Tone::Accent),
                (format!("护甲 {}", b.block), Tone::Good),
                (format!("{core_name} {core_value}"), Tone::Accent),
            ];
            // Show active cross-class resources as well, not just the hero's main meter.
            for (name, value) in [
                ("战意", b.momentum), ("谋略", b.scheme), ("连段", b.combo),
                ("统御", b.command), ("怒气", b.fury), ("反击", b.thorns),
            ] {
                if name != core_name && value > 0 {
                    stats.push((format!("{name} {value}"), Tone::Accent));
                }
            }
            stats.extend([
                (format!("回合 {}", b.turn), Tone::Muted),
                (format!("武勇 {}", b.strength), Tone::Muted),
                (format!("虚弱 {}", b.weak),
                    if b.weak > 0 { Tone::Warning } else { Tone::Muted }),
                (format!("易伤 {}", b.vulnerable),
                    if b.vulnerable > 0 { Tone::Warning } else { Tone::Muted }),
            ]);
            status_rows(&mut out, app, columns, &stats);
            heading(&mut out, app, "敌军（t 切换目标）", columns);
            for (i, enemy) in b.foes.iter().enumerate() {
                if out.len() + 5 >= rows { break; }
                let active = i + 1 == target && enemy.hp > 0;
                let label = format!("{} {}. {}  {}/{} HP ·甲{} 毒{} ·{}{}",
                    if active { "▶" } else { " " }, i + 1, enemy.name(), enemy.hp,
                    enemy.max_hp, enemy.block, enemy.poison, enemy.intent.label(),
                    if enemy.hp == 0 { " [阵亡]" }
                    else if enemy.phase > 0 { " [二阶段]" } else { "" });
                line(&mut out, app, &label, columns,
                    if enemy.hp == 0 { Tone::Muted }
                    else if active { Tone::Target } else { Tone::Danger });
            }
            if b.thorns > 0 {
                line(&mut out, app, &format!("反击 {} ·可中断后续战技", b.thorns),
                    columns, Tone::Warning);
            }
            if b.turn >= 35 {
                let warning = if b.turn < 40 { format!("军粮告急 · 距损耗{}回合", 40 - b.turn) }
                else { format!("军粮枯竭 · 回合末损失{}生命", 1 + (b.turn - 40) / 4) };
                line(&mut out, app, &warning, columns, Tone::Danger);
            }
            if details && rows >= 30 {
                line(&mut out, app, &format!("牌堆 抽{} / 弃{} / 耗{} · 亲兵 {}",
                    b.draw.len(), b.discard.len(), b.exhausted.len(), b.troops_hp),
                    columns, Tone::Muted);
                if !b.powers.is_empty() {
                    line(&mut out, app, &format!("持续军略：{}", b.powers.iter()
                        .map(|p| format!("{}×{}", p.id.name(), p.stacks))
                        .collect::<Vec<_>>().join("、")), columns, Tone::Muted);
                }
            }
        } else {
            let scene = match g.stage {
                Stage::Map => "选下一站 · :map 看路线", Stage::Reward => "选择战利品",
                Stage::Camp => "扎营休整", Stage::Market => "军市交易",
                Stage::Story => "驿道奇遇", Stage::Ancient => "先贤赐福",
                Stage::Won => "征程告捷", Stage::Lost => "战役结束",
                Stage::Combat => "战斗中",
            };
            line(&mut out, app, scene, columns, Tone::Accent);
            if g.stage == Stage::Story {
                // Preserve the scene title and premise, even when the menu is compact.
                // The full narrative and choices remain scrollable with 'i'.
                let opening = g.lore_hint();
                let premise = opening.lines().next().unwrap_or_default();
                let limit = if rows >= 28 { 4 } else if rows >= 21 { 3 } else { 2 };
                for paragraph in hud_reading::preview(premise, columns, limit) {
                    line(&mut out, app, &paragraph, columns, Tone::Normal);
                }
            }
        }
    } else {
        line(&mut out, app, "尚未出征 · 选择武将开始征程", columns, Tone::Muted);
    }
    let selected = options.get(cursor);
    let description = selected.and_then(|(label, _)| {
        if combat {
            focus_description(label, true).map(|text| format!("选中说明 · {text}"))
        } else if width(label) > columns.saturating_sub(8) {
            Some(format!("当前选择 · {label}"))
        } else {
            None
        }
    });
    let show_desc = !details && description.is_some() && rows >= 23;
    let show_log = rows >= 28 && app.game.as_ref().is_some();
    let show_feedback = !app.message.is_empty();
    let desc_rows = if show_desc { if rows >= 30 { 2 } else { 1 } } else { 0 };
    let feedback_rows = if show_feedback { if rows >= 30 { 2 } else { 1 } } else { 0 };
    let reserved = 1 + desc_rows + usize::from(show_log) + feedback_rows;
    // Keep at least one decision on-screen, including at 28×16.
    let space = rows.saturating_sub(out.len() + reserved + 1);
    let (start, end) = window(options.len(), cursor, space);
    let menu_title = if combat {
        if options.iter().any(|(_, command)| command == "@cancel") {
            format!("指定副选牌  {}/{}", cursor + 1, options.len())
        } else {
            format!("手牌与行动  {}/{}", cursor + 1, options.len())
        }
    } else if app.game.is_some() {
        format!("行动选择  {}/{}", cursor + usize::from(!options.is_empty()), options.len())
    } else {
        format!("选将  {}/{}", cursor + usize::from(!options.is_empty()), options.len())
    };
    heading(&mut out, app, &menu_title, columns);
    if options.is_empty() {
        line(&mut out, app, "暂无选项 · 按 : 输入命令", columns, Tone::Muted);
    } else {
        for (i, (label, command)) in options.iter().enumerate().take(end).skip(start) {
            let text = format!("{} {:>2}. {}", if i == cursor { "▶" } else { " " },
                i + 1, menu_label(label, combat, details));
            let padded = fit(&text, columns);
            if i == cursor {
                // Always signal keyboard focus through both shape and contrast.
                let pad = columns.saturating_sub(width(&padded));
                out.push(color(app, &format!("{padded}{}", " ".repeat(pad)), Tone::Focus));
            } else {
                line(&mut out, app, &padded, columns,
                    if command == "end" { Tone::Accent }
                    else if command == "@cancel" { Tone::Warning }
                    else if label.starts_with("× ") { Tone::Muted }
                    else { Tone::Normal });
            }
        }
    }
    if let Some(description) = description.filter(|_| show_desc) {
        for text in hud_reading::preview(&description, columns, desc_rows) {
            line(&mut out, app, &text, columns, Tone::Muted);
        }
    }
    if show_log && let Some(g) = &app.game {
        line(&mut out, app, &format!("最近战报 · {}", g.log.last()
            .map_or("暂无战报", String::as_str)), columns, Tone::Muted);
    }
    if show_feedback {
        let tone = if app.message.starts_with("错误") || app.message.starts_with("存档未能") {
            Tone::Danger
        } else { Tone::Good };
        for text in hud_reading::preview(&format!("操作反馈 · {}", app.message),
            columns, feedback_rows) {
            line(&mut out, app, &text, columns, tone);
        }
    }
    if out.len() >= rows { out.truncate(rows - 1); }
    let shortcuts = if columns < 52 {
        "↑↓选择 Enter确认 i全文 :命令"
    } else {
        "↑↓/jk选 Enter确认 1-9快捷 i全文 t目标 v详情 :命令 ?帮助 q退出"
    };
    line(&mut out, app, shortcuts, columns, Tone::Muted);
    format!("{}\n", out.join("\n"))
}

#[cfg(test)]
#[path = "hud_tests.rs"]
mod hud_tests;
