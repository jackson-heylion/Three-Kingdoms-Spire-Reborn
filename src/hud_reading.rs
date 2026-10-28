//! A read-only, scrollable full-text view for long TUI content.
//! Wrap at Unicode display-cell boundaries; never truncate the underlying narrative.
use super::{App, Stage};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn width(text: &str) -> usize { UnicodeWidthStr::width(text) }

pub(super) fn wrapped(text: &str, columns: usize) -> Vec<String> {
    let columns = columns.max(1);
    let mut output = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        let mut used = 0;
        for ch in paragraph.chars().filter(|ch| !ch.is_control()) {
            let cells = UnicodeWidthChar::width(ch).unwrap_or(0);
            if cells > columns { continue; }
            if used + cells > columns && !current.is_empty() {
                output.push(std::mem::take(&mut current));
                used = 0;
            }
            current.push(ch);
            used += cells;
        }
        output.push(current);
    }
    output
}

/// A limited preview announces that the remainder exists, instead of silently
/// cutting a sentence. All text is available with the 'i' reading overlay.
pub(super) fn preview(text: &str, columns: usize, limit: usize) -> Vec<String> {
    if limit == 0 { return Vec::new(); }
    let lines = wrapped(text, columns);
    if lines.len() <= limit { return lines; }
    let mut shown = lines.into_iter().take(limit).collect::<Vec<_>>();
    let marker = "… i全文";
    if let Some(last) = shown.last_mut() {
        while width(last) + width(marker) > columns && !last.is_empty() {
            last.pop(); // Remove a Unicode scalar, never a UTF-8 byte.
        }
        last.push_str(marker);
    }
    shown
}

/// Preserve all scene text, selected menu wording and long command feedback.
pub(super) fn document(app: &App, options: &[(String, String)], cursor: usize) -> String {
    let mut sections = Vec::new();
    if let Some(game) = &app.game
        && game.stage == Stage::Story {
        sections.push(format!("【当前演义剧情】\n{}", game.lore_hint()));
    }
    if let Some((label, _)) = options.get(cursor) {
        sections.push(format!("【当前选项】\n{label}"));
    }
    if !app.message.is_empty() {
        sections.push(format!("【操作反馈】\n{}", app.message));
    }
    if let Some(game) = &app.game {
        let logs = game.log.iter().rev().take(3).rev().cloned().collect::<Vec<_>>();
        if !logs.is_empty() {
            sections.push(format!("【最近战报】\n{}", logs.join("\n")));
        }
    }
    if sections.is_empty() {
        sections.push("暂无可阅读内容。请选择一个行动，或通过 :lore 查看剧情记录。".into());
    }
    sections.join("\n\n")
}

pub(super) fn lines(app: &App, columns: usize,
    options: &[(String, String)], cursor: usize) -> Vec<String> {
    wrapped(&document(app, options, cursor), columns.clamp(28, 120))
}

pub(super) fn max_offset(app: &App, columns: usize, rows: usize,
    options: &[(String, String)], cursor: usize) -> usize {
    lines(app, columns, options, cursor).len()
        .saturating_sub(rows.max(16).saturating_sub(3))
}

pub(super) fn render(app: &App, columns: usize, rows: usize,
    options: &[(String, String)], cursor: usize, offset: usize) -> String {
    let columns = columns.clamp(28, 120);
    let rows = rows.max(16);
    let body = lines(app, columns, options, cursor);
    let visible = rows - 3;
    let start = offset.min(body.len().saturating_sub(visible));
    let end = (start + visible).min(body.len());
    let mut screen = vec![
        "三国登塔录 · 全文阅读（只读）".to_owned(),
        "全文模式不执行行动，也不改变存档".to_owned(),
    ];
    screen.extend(body[start..end].iter().cloned());
    let footer = if columns < 50 {
        format!("i/q返回 jk滚动 n/p翻页 {}/{}", end, body.len())
    } else {
        format!("i/Enter/q返回 ↑↓/jk滚动 n/p翻页 · 第{}—{}行 / 共{}行",
            if body.is_empty() {0} else {start + 1}, end, body.len())
    };
    screen.push(wrapped(&footer, columns).into_iter().next().unwrap_or_default());
    format!("{}\n", screen.join("\n"))
}
