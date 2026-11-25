//! A compact, Unicode-cell-aware terminal HUD. The plain CLI and Game are untouched.
//! Color is presentation-only and never enters persisted state or user commands.
use super::{App, Stage, Theme};
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
enum Tone { Normal, Muted, Accent, Good, Warning, Danger, Focus }
fn color(app: &App, text: &str, tone: Tone) -> String {
    if !app.use_ansi || matches!(tone, Tone::Normal) { return text.to_owned(); }
    let code = match app.theme {
        Theme::Dark => match tone {
            Tone::Normal => "97", Tone::Muted => "90", Tone::Accent => "1;96",
            Tone::Good => "1;92", Tone::Warning => "1;93",
            Tone::Danger => "1;91", Tone::Focus => "1;30;106",
        },
        Theme::Light => match tone {
            Tone::Normal => "30", Tone::Muted => "90", Tone::Accent => "1;34",
            Tone::Good => "1;32", Tone::Warning => "1;33",
            Tone::Danger => "1;31", Tone::Focus => "1;97;44",
        },
        Theme::HighContrast => match tone {
            Tone::Normal => "1;97", Tone::Muted => "1;37", Tone::Accent => "1;96",
            Tone::Good => "1;92", Tone::Warning => "1;93",
            Tone::Danger => "1;91", Tone::Focus => "1;30;103",
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

fn wrap_prefixed_lines(
    out: &mut Vec<String>, app: &App, prefix: &str, text: &str, columns: usize, tone: Tone, max_lines: usize,
) {
    let wrapped = report_lines(&format!("{prefix}{text}"), columns);
    let shown = wrapped.len().min(max_lines.max(1));
    for (idx, chunk) in wrapped.iter().take(shown).enumerate() {
        let mut rendered = chunk.clone();
        if idx + 1 == shown && wrapped.len() > shown {
            rendered = fit(&format!("{rendered}…"), columns);
        }
        line(out, app, &rendered, columns, tone);
    }
}

/// Wrap Unicode text without dropping the rest of a long archive line.
pub(super) fn report_lines(text: &str, columns: usize) -> Vec<String> {
    let columns = columns.clamp(28, 120);
    let mut result = Vec::new();
    for raw in text.lines() {
        let mut line = String::new();
        let mut cells = 0;
        for ch in raw.chars().filter(|ch| !ch.is_control()) {
            let width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if cells + width > columns {
                result.push(std::mem::take(&mut line));
                cells = 0;
            }
            line.push(ch);
            cells += width;
        }
        result.push(line);
    }
    if result.is_empty() { result.push("暂无档案内容".into()); }
    result
}

pub(super) fn report_page_size(rows: usize) -> usize {
    rows.max(16) - 3
}

pub(super) fn render_report(app: &App, columns: usize, rows: usize,
    lines: &[String], offset: usize) -> String
{
    let columns = columns.clamp(28, 120);
    let page = report_page_size(rows);
    let start = offset.min(lines.len().saturating_sub(page));
    let end = start.saturating_add(page).min(lines.len());
    let mut out = Vec::new();
    heading(&mut out, app, "档案与详情（只读）", columns);
    for content in &lines[start..end] {
        line(&mut out, app, content, columns, Tone::Normal);
    }
    line(&mut out, app, &format!("{}–{} / {} 行",
        start + usize::from(!lines.is_empty()), end, lines.len()), columns, Tone::Muted);
    line(&mut out, app, "↑↓ / jk 滚动  fb 翻页  q/Enter 返回", columns, Tone::Accent);
    out.join("\n")
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
        let act = (g.floor.saturating_sub(1) / g.campaign_rows() + 1)
            .min(g.campaign_max_floor() / g.campaign_rows());
        let hero_label = g.difficulty.map_or_else(|| g.hero.name().to_owned(),
            |tier| format!("{}·{}", g.hero.name(), tier.name()));
        let hp_tone = if g.hp * 4 <= g.max_hp { Tone::Danger }
            else if g.hp * 2 <= g.max_hp { Tone::Warning } else { Tone::Good };
        if columns < 48 {
            line(&mut out, app, &format!("{hero_label} · 第{act}幕·{}层", g.floor),
                columns, Tone::Accent);
            pieces(&mut out, app, columns, &[
                (&format!("生命 {}/{}  ", g.hp, g.max_hp), hp_tone),
                (&format!("金 {}  药 {}", g.gold, g.potions), Tone::Muted),
            ]);
        } else {
            pieces(&mut out, app, columns, &[
                (&format!("{hero_label}  第{act}幕·{}层  ", g.floor), Tone::Accent),
                (&format!("生命 {}/{}  ", g.hp, g.max_hp), hp_tone),
                (&format!("金 {}  药 {}", g.gold, g.potions), Tone::Muted),
            ]);
        }
        if details && rows >= 26 {
            let mode_label = g.difficulty.map_or_else(
                || format!("{:?} A{}", g.mode, g.ascension),
                |tier| format!("新制【{}】", tier.name()),
            );
            line(&mut out, app, &format!("{mode_label} · {} · 战法：{}",
                g.act_name(), g.doctrine.map_or("自动", |d| d.label())), columns, Tone::Muted);
            if let Some(cap) = g.victory_recovery_cap() {
                line(&mut out, app, &format!("本幕战后被动回血上限 {cap}"), columns, Tone::Muted);
            }
        }
        if let Some(b) = g.battle.as_ref().filter(|_| g.stage == Stage::Combat) {
            combat = true;
            pieces(&mut out, app, columns, &[
                (&format!("军令 {}  ", b.energy), Tone::Accent),
                (&format!("护甲 {}  ", b.block), Tone::Good),
                (&format!("回合 {}  ", b.turn), Tone::Normal),
                (&format!("武勇 {}  虚弱 {}  易伤 {}", b.strength, b.weak, b.vulnerable),
                    if b.weak > 0 || b.vulnerable > 0 { Tone::Warning } else { Tone::Muted }),
            ]);
            if let Some(objective) = g.tactical_objective() {
                let budget = if details { 6 } else if rows >= 28 { 3 } else { 1 };
                wrap_prefixed_lines(&mut out, app, "关卡目标：", &objective, columns, Tone::Warning, budget);
            }
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
                    else if active { Tone::Warning } else { Tone::Danger });
                if out.len() + 6 < rows && let Some(warning) = g.phase_transition_hint(i) {
                    line(&mut out, app, &format!("变阵预告：{warning}"), columns, Tone::Warning);
                }
            }
            // Surface only relevant resources by default; diagnostics live in detail mode.
            let resources = [
                ("战意", b.momentum), ("谋略", b.scheme), ("连段", b.combo),
                ("统御", b.command), ("怒气", b.fury), ("反击", b.thorns),
            ].into_iter().filter(|(_, value)| details || *value > 0)
                .map(|(name, value)| format!("{name} {value}"))
                .collect::<Vec<_>>();
            if !resources.is_empty() {
                line(&mut out, app, &resources.join("  "), columns, Tone::Accent);
            }
            if b.thorns > 0 {
                line(&mut out, app, &format!("反击 {} ·可中断后续战技", b.thorns),
                    columns, Tone::Warning);
            }
            if b.wildfire > 0 {
                line(&mut out, app,
                    &format!("焚营火势 {}层 ·回合末无视护甲损失最多{}生命 ·防御牌可扑灭",
                        b.wildfire, u32::from(b.wildfire) * 2),
                    columns, Tone::Danger);
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
                if g.in_council() {
                    if let Some(header) = g.lore_hint().lines().next() {
                        wrap_prefixed_lines(&mut out, app, "", header, columns, Tone::Warning, 2);
                    }
                } else if let Some(beat) = g.lore_current.as_deref().and_then(three_kingdoms_spire_reborn::lore::beat) {
                    let budget = if details { 8 } else if rows >= 24 { 4 } else { 2 };
                    wrap_prefixed_lines(&mut out, app,
                        &format!("第{:03}回·{}（{}）：", beat.chapter, beat.title, beat.character),
                        beat.premise, columns, Tone::Normal, budget);
                } else if let Some((title, _)) = g.story_options().split_once('：') {
                    line(&mut out, app, &format!("当前事件：{title}"), columns, Tone::Normal);
                }
            }
            if g.stage == Stage::Map
                && let Some(index) = options.get(cursor).and_then(|(_, command)|
                    command.strip_prefix("choose ")).and_then(|n| n.parse::<usize>().ok())
                && let Some(node) = index.checked_sub(1).and_then(|i| g.route.get(i))
                && let Some(hint) = g.route_objective_hint(*node)
            {
                let budget = if details { 5 } else if rows >= 24 { 3 } else { 1 };
                wrap_prefixed_lines(&mut out, app, "所选路线风险：", &hint, columns, Tone::Warning, budget);
            }
        }
    } else {
        line(&mut out, app, "尚未出征 · 选择武将开始征程", columns, Tone::Muted);
    }
    let selected = options.get(cursor);
    let description = selected.and_then(|(label, _)| focus_description(label, combat));
    let show_desc = !details && description.is_some() && rows >= 23;
    let show_log = rows >= 28 && app.game.as_ref().is_some();
    let show_feedback = !app.message.is_empty();
    let reserved = 1 + usize::from(show_desc) + usize::from(show_log)
        + usize::from(show_feedback);
    // Keep at least one decision on-screen, including at 28×16.
    let space = rows.saturating_sub(out.len() + reserved + 1);
    let (start, end) = window(options.len(), cursor, space);
    let menu_title = if combat {
        if options.iter().any(|(_, command)| command == "@cancel") {
            format!("指定副选牌  {}/{}", cursor + 1, options.len())
        } else {
            format!("手牌与行动  {}/{}", cursor + 1, options.len())
        }
    } else if options.iter().any(|(_, command)| command.starts_with("campaign ")) {
        format!("新制难度（三档）  {}/{}", cursor + 1, options.len())
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
    if show_desc {
        line(&mut out, app, &format!("选中说明 · {}", description.unwrap_or_default()),
            columns, Tone::Muted);
    }
    if show_log && let Some(g) = &app.game {
        line(&mut out, app, &format!("最近战报 · {}", g.log.last()
            .map_or("暂无战报", String::as_str)), columns, Tone::Muted);
    }
    if show_feedback {
        let tone = if app.message.starts_with("错误") || app.message.starts_with("存档未能") {
            Tone::Danger
        } else { Tone::Good };
        line(&mut out, app, &format!("操作反馈 · {}", app.message), columns, tone);
    }
    if out.len() >= rows { out.truncate(rows - 1); }
    let shortcuts = if columns < 52 {
        "↑↓ 选择  Enter确认  v详情  :命令"
    } else {
        "↑↓/jk 选  Enter确认  1-9快捷  t目标  v详情  :命令  ?帮助  q退出"
    };
    line(&mut out, app, shortcuts, columns, Tone::Muted);
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Profile,Theme,tui};
    use three_kingdoms_spire_reborn::{Game,Action,content::Hero};
    fn app(game:Option<Game>)->App {
        App{game,profile:Profile::new(),theme:Theme::Dark,
            dir:std::path::PathBuf::new(),use_ansi:false,message:String::new()}
    }
    #[test]
    fn lethal_counterplay_is_visible_without_affecting_commands() {
        let mut game = Game::new(Hero::GuanYu, 93204);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.battle.as_mut().unwrap().thorns = 5;
        let app = app(Some(game));
        let commands = tui::choices(&app, 1);
        let screen = render(&app, 80, 34, 1, &commands, 0);
        assert!(screen.contains("反击 5 ·可中断后续战技"));
        assert!(commands.iter().any(|(_, action)| action == "end"));
    }
    #[test]
    fn constrained_screens_fit_both_dimensions_and_scroll_to_selected_action(){
        let mut game=Game::new(Hero::GuanYu,83);
        game.apply(Action::Choose{index:1}).unwrap();
        let app=app(Some(game));
        let choices=tui::choices(&app,1);
        for (cols,rows) in [(28,16),(40,24),(80,24),(120,38)] {
            let content=render(&app,cols,rows,1,&choices,choices.len()-1);
            assert!(content.lines().count()<=rows,"{cols}x{rows}: {content}");
            assert!(content.lines().all(|s|width(s)<=cols),"{cols}x{rows}: {content}");
            // New utility entries can follow End Turn; validate focus/scroll for both.
            assert!(content.contains("查看战斗事件审计"),"{content}");
            let end_index = choices.iter().position(|(_,cmd)|cmd == "end").unwrap();
            let end_content = render(&app,cols,rows,1,&choices,end_index);
            assert!(end_content.contains("结束本回合"),"{end_content}");
            assert!(content.contains("敌军")&&content.contains("军令"),"{content}");
        }
    }
    #[test]
    fn secondary_card_submenu_has_a_distinct_keyboard_focus_heading() {
        let mut game = Game::new(Hero::ZhugeLiang, 983);
        game.apply(Action::Choose { index: 1 }).unwrap();
        let app = app(Some(game));
        let menu = vec![
            ("牌堆第2张：突刺".into(), "play 1 1 2".into()),
            ("取消选牌".into(), "@cancel".into()),
        ];
        let screen = render(&app,80,34,1,&menu,0);
        assert!(screen.contains("指定副选牌"),"{screen}");
        assert!(screen.contains("取消选牌"),"{screen}");
    }
    #[test]
    fn action_menu_is_only_hand_and_no_duplicate_targets(){
        let mut game=Game::new(Hero::GuanYu,16);
        game.apply(Action::Choose{index:1}).unwrap();
        let app=app(Some(game));
        let menu=tui::choices(&app,1);
        assert!(menu.iter().any(|(label,_)|label.starts_with('[')&&label.contains("突刺")));
        assert!(!menu.iter().any(|(label,_)|label.contains("目标敌军")));
        let screen=render(&app,80,34,1,&menu,0);
        assert!(screen.contains("手牌与行动"));
        assert_eq!(screen.matches("突刺").count(),menu.iter().filter(|(label,_)|label.contains("突刺")).count());
    }
    #[test]
    fn save_feedback_has_distinct_panel(){
        let mut game=Game::new(Hero::CaoCao,16);
        game.apply(Action::Choose{index:1}).unwrap();
        let mut app=app(Some(game));
        app.message="存档读取成功".into();
        let menu=tui::choices(&app,1);
        let screen=render(&app,80,34,1,&menu,0);
        assert!(screen.contains("最近战报"));
        assert!(screen.contains("操作反馈"));
        assert!(screen.contains("存档读取成功"));
    }
    #[test]
    fn scroll_window_keeps_cursor_visible(){
        for cursor in 0..30 {
            let (a,b)=window(30,cursor,5);
            assert!(a<=cursor&&cursor<b);
            assert_eq!(b-a,5);
        }
    }
    #[test]
    fn log_control_characters_are_sanitized(){
        let line=fit("张角\x1b[2J诡术\n奇谋",8);
        assert!(!line.contains('\x1b')&&!line.contains('\n'));
        assert!(width(&line)<=8);
    }
}

#[cfg(test)]
mod compact_hud_tests {
    use super::*;
    use crate::{Profile, Theme, tui};
    use three_kingdoms_spire_reborn::{Action, Game, content::Hero};
    fn app() -> App {
        let mut game = Game::new(Hero::GuanYu, 83);
        game.apply(Action::Choose { index: 1 }).unwrap();
        App { game: Some(game), profile: Profile::new(), theme: Theme::Dark,
            dir: std::path::PathBuf::new(), use_ansi: false, message: String::new() }
    }
    #[test]
    fn compact_defers_descriptions_and_keeps_menu_focus() {
        let app = app();
        let choices = tui::choices(&app, 1);
        let description = choices[0].0.split_once(" · ").unwrap().1;
        let compact = render_mode(&app, 80, 24, 1, &choices, 0, false);
        let expanded = render_mode(&app, 80, 34, 1, &choices, 0, true);
        assert!(compact.contains("选中说明"));
        assert_eq!(compact.matches(description).count(), 1);
        assert!(expanded.contains(description));
        assert!(compact.contains("▶"));
        assert!(compact.contains("军令"));
    }
    #[test]
    fn ansi_roles_are_optional_and_theme_aware() {
        let mut app = app();
        let options = tui::choices(&app, 1);
        let plain = render(&app, 80, 24, 1, &options, 0);
        assert!(!plain.contains("\x1b["));
        app.use_ansi = true;
        for theme in [Theme::Dark, Theme::Light, Theme::HighContrast] {
            app.theme = theme;
            let screen = render(&app, 80, 24, 1, &options, 0);
            assert!(screen.contains("\x1b["));
            assert!(screen.contains("▶"));
            assert!(screen.contains("生命"));
        }
    }
    #[test]
    fn narrow_and_short_viewports_keep_active_choice_visible() {
        let app = app();
        let choices = tui::choices(&app, 1);
        for (cols, rows) in [(28, 16), (40, 20), (80, 24), (120, 38)] {
            for details in [false, true] {
                let screen = render_mode(&app, cols, rows, 1, &choices, choices.len()-1, details);
                assert!(screen.lines().count() <= rows, "{cols}x{rows} {screen}");
                assert!(screen.lines().all(|line| width(line) <= cols), "{screen}");
                assert!(screen.contains("查看战斗事件审计"), "{screen}");
            }
        }
    }
}


#[cfg(test)]
mod expanded_objective_hud_regression {
    use super::*;
    use crate::{Profile, Theme, tui};
    use three_kingdoms_spire_reborn::{Action, Difficulty, Game, content::Hero};

    #[test]
    fn map_and_combat_hud_show_authoritative_objective_without_mutation() {
        let game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 16, Difficulty::Warlords, "hud-objective".into(),
        );
        let mut app = App { game: Some(game), profile: Profile::new(), theme: Theme::Dark,
            dir: std::path::PathBuf::new(), use_ansi: false, message: String::new() };
        let before = app.game.as_ref().unwrap().to_json().unwrap();
        let menu = tui::choices(&app, 1);
        let screen = render(&app, 100, 30, 1, &menu, 0);
        assert!(screen.contains("所选路线风险") && screen.contains("突围"));
        assert!(screen.contains("群雄逐鹿"));
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(), before);
        app.game.as_mut().unwrap().apply(Action::Choose { index: 1 }).unwrap();
        let before = app.game.as_ref().unwrap().to_json().unwrap();
        let menu = tui::choices(&app, 1);
        let screen = render(&app, 100, 30, 1, &menu, 0);
        assert!(screen.contains("关卡目标") && screen.contains("突围"));
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(), before);
    }
}


#[cfg(test)]
mod report_view_tests {
    use super::*;
    use crate::{Profile, Theme};

    #[test]
    fn long_cjk_archive_wraps_without_losing_text_and_scrolls_to_end() {
        let text = "街亭守水责任跨幕兑现，保留真实战场后果。".repeat(45);
        let lines = report_lines(&text, 28);
        assert_eq!(lines.concat(), text);
        assert!(lines.iter().all(|line| width(line) <= 28));
        let app = App { game: None, profile: Profile::new(), theme: Theme::Dark,
            dir: std::path::PathBuf::new(), use_ansi: false, message: String::new() };
        let first = render_report(&app, 28, 16, &lines, 0);
        let last = render_report(&app, 28, 16, &lines, usize::MAX);
        assert!(first.contains("档案与详情"));
        assert_ne!(first, last);
        assert!(last.contains(lines.last().unwrap()));
        for screen in [first, last] {
            assert!(screen.lines().count() <= 16);
            assert!(screen.lines().all(|line| width(line) <= 28));
            assert!(!screen.ends_with('\n'), "report must not scroll bottom row off");
        }
    }

    #[test]
    fn story_and_details_hud_surface_literary_context_without_legacy_ascension() {
        use crate::tui;
        use three_kingdoms_spire_reborn::{Difficulty, Game, content::Hero};
        let mut game = Game::new_with_campaign_difficulty(
            Hero::GuanYu, 16, Difficulty::Conqueror, "hud-story-details".into(),
        );
        game.stage = Stage::Story;
        game.lore_current = Some("taoyuan".into());
        let app_story = App { game: Some(game), profile: Profile::new(), theme: Theme::Dark,
            dir: std::path::PathBuf::new(), use_ansi: false, message: String::new() };
        let menu = tui::choices(&app_story, 1);
        let screen = render_mode(&app_story, 90, 32, 1, &menu, 0, true);
        assert!(screen.contains("第001回·桃园残誓"), "{screen}");
        assert!(screen.contains("新制【乱世枭雄】"), "{screen}");
        assert!(!screen.contains("A0"), "{screen}");
        assert!(!screen.ends_with('\n'));
    }
}
