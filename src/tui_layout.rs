//! A bounded, readable TUI frame. Kept separate from the line-oriented CLI.
use super::{App, Stage, fitting_label};
use unicode_width::UnicodeWidthStr;

fn health_bar(value:i32, max:i32, cells:usize)->String {
    let filled=if max<=0 {0} else {
        (value.max(0) as usize).saturating_mul(cells)
            .checked_div(max as usize).unwrap_or(0).min(cells)
    };
    format!("{}{}", "█".repeat(filled), "░".repeat(cells-filled))
}
fn stage_name(stage:Stage)->&'static str {
    match stage {
        Stage::Map=>"行军路线", Stage::Combat=>"战斗",
        Stage::Reward=>"战利品", Stage::Camp=>"营寨",
        Stage::Market=>"军市", Stage::Story=>"奇遇",
        Stage::Ancient=>"先贤赐福", Stage::Won=>"凯旋", Stage::Lost=>"战败",
    }
}
fn context_lines(app:&App,target:usize)->Vec<String> {
    let Some(g)=&app.game else {
        return vec!["选择一名武将，开始你的三幕征程。".into(),
            "支持方向键、数字快捷键与冒号命令。".into()];
    };
    match g.stage {
        Stage::Combat => {
            let Some(b)=&g.battle else {return vec!["战斗数据暂不可用".into()]};
            let mut lines=vec![format!(
                "第{}回合   军令 {}   护甲 {}   武勇 {}   手牌 {}   目标 敌{}",
                b.turn,b.energy,b.block,b.strength,b.hand.len(),target)];
            for (idx,foe) in b.foes.iter().enumerate() {
                let marker=if idx+1==target && foe.hp>0 {"▶"} else {" "};
                let state=if foe.hp<=0 {"阵亡"} else if foe.phase>0 {"二阶段"} else {"存活"};
                lines.push(format!(
                    "{} 敌{} {} [{}] {}/{}  甲{} 毒{}  {} · {}",
                    marker,idx+1,foe.name(),health_bar(foe.hp,foe.max_hp,10),
                    foe.hp,foe.max_hp,foe.block,foe.poison,foe.intent.label(),state));
            }
            lines.push(format!(
                "战意 {} · 谋略 {} · 连段 {} · 统御 {} · 怒气 {} · 反击 {}",
                b.momentum,b.scheme,b.combo,b.command,b.fury,b.thorns));
            lines.push(format!(
                "牌堆：抽 {} / 弃 {} / 耗 {}   亲兵 {}   弃换 {}",
                b.draw.len(),b.discard.len(),b.exhausted.len(),
                b.troops_hp,b.discard_tokens));
            if !b.powers.is_empty() {
                lines.push(format!("持续军略：{}",b.powers.iter()
                    .map(|p|format!("{}×{}",p.id.name(),p.stacks))
                    .collect::<Vec<_>>().join(" · ")));
            }
            if b.turn>=35 {
                lines.push(if b.turn<40 {
                    format!("警告：距军粮枯竭还有{}回合",40-b.turn)
                } else {
                    format!("警告：本回合粮尽将无视护甲失去{}体力",
                        1+(b.turn-40)/4)
                });
            }
            lines
        }
        Stage::Map=>vec![
            format!("章节：{}   可选路线：{}条",g.act_name(),g.route.len()),
            "查看全幕路线：输入 :map  |  三层评分用于路线风险收益参考".into(),
        ],
        Stage::Story=>vec![format!("奇遇：{}",g.lore_hint())],
        Stage::Market=>vec![format!("当前金币：{}   选购后自动保存",g.gold)],
        Stage::Camp=>vec!["休整、强化或操练；定向强化可输入 :upgrade N".into()],
        Stage::Reward=>vec!["选择一张战利品，或跳过奖励。".into()],
        Stage::Ancient=>vec!["选择一项先贤赐福，效果持续至本局结束。".into()],
        Stage::Won=>vec!["天下平定。可开始新战役或查看战功簿。".into()],
        Stage::Lost=>vec!["征途结束。可重新选择武将。".into()],
    }
}
struct ScreenRow { text:String, selected:bool }
impl ScreenRow {
    fn normal(text:impl Into<String>)->Self { Self{text:text.into(),selected:false} }
    fn selected(text:impl Into<String>)->Self { Self{text:text.into(),selected:true} }
}
/// Pure frame composer: no IO, no game mutation. Every line is bounded by the
/// actual terminal cell width, and the active option stays in the viewport.
pub(super) fn render_frame(app:&App,target:usize,cursor:usize,options:&[(String,String)],
    rows:usize,columns:usize)->String {
    let height=rows.max(1);
    let width=columns.max(1);
    let line="─".repeat(width);
    let mut output=Vec::<ScreenRow>::new();
    output.push(ScreenRow::normal(format!("╭─ 三国登塔录 / THREE KINGDOMS SPIRE {}", "─".repeat(width.saturating_sub(34)))));
    if let Some(g)=&app.game {
        let hp=health_bar(g.hp,g.max_hp,10);
        output.push(ScreenRow::normal(format!(
            "{}  体力 [{}] {}/{}   金 {}   药 {}",
            g.hero.name(),hp,g.hp,g.max_hp,g.gold,g.potions)));
        output.push(ScreenRow::normal(format!(
            "{} · 第{}/21层 · {} · 飞升{} · 存档#{}",
            g.act_name(),g.floor,stage_name(g.stage),g.ascension,g.revision)));
    } else {
        output.push(ScreenRow::normal("新征程 · 选择武将"));
    }
    output.push(ScreenRow::normal(line.clone()));

    // Reserve space for actions and controls on small terminals.
    let footer_budget=if height>=14 {4} else {2};
    let context_budget=height.saturating_sub(output.len()+footer_budget+3);
    for detail in context_lines(app,target).into_iter().take(context_budget) {
        output.push(ScreenRow::normal(detail));
    }
    if output.len()+3<height {
        output.push(ScreenRow::normal(line.clone()));
    }
    output.push(ScreenRow::normal(format!(
        "可执行操作  {} 项   (↑↓ 选择 / 回车执行)",options.len())));
    let option_budget=height.saturating_sub(output.len()+footer_budget).max(1);
    let active=cursor.min(options.len().saturating_sub(1));
    let offset=active.saturating_sub(option_budget/2)
        .min(options.len().saturating_sub(option_budget));
    if options.is_empty() {
        output.push(ScreenRow::normal("暂无可用操作；按 : 输入命令。"));
    } else {
        for (idx,(label,_)) in options.iter().enumerate().skip(offset).take(option_budget) {
            let item=format!("{} {:>2}. {}",if idx==active {"▶"} else {" "},idx+1,label);
            output.push(if idx==active {
                ScreenRow::selected(item)
            } else {
                ScreenRow::normal(item)
            });
        }
    }
    if height>=14 {
        let selected=options.get(active).map_or("无",|(label,_)|label.as_str());
        output.push(ScreenRow::normal(format!("当前选择：{selected}")));
        let message=app.message.lines().next().unwrap_or("");
        output.push(ScreenRow::normal(format!("消息：{}",if message.is_empty() {"—"} else {message})));
    }
    output.push(ScreenRow::normal(line));
    output.push(ScreenRow::normal(if width<65 {
        "↑↓/jk 选  Enter 执行  t 目标  : 命令  ? 帮助  q 退出".into()
    } else {
        "↑↓/jk 选择  Enter 执行  1-9 快选  t 切目标  : 高级命令  ? 帮助  q 退出".into()
    }));
    output.truncate(height);
    let mut frame=String::new();
    if app.use_ansi {frame.push_str(app.theme.style());}
    for (idx,row) in output.iter().enumerate() {
        if idx>0 {frame.push_str("\r\n");}
        let text=fitting_label(&row.text,width);
        let padding=width.saturating_sub(UnicodeWidthStr::width(text.as_str()));
        if row.selected && app.use_ansi {frame.push_str("\x1b[7m");}
        frame.push_str(&text);
        frame.push_str(&" ".repeat(padding));
        if row.selected && app.use_ansi {
            frame.push_str("\x1b[0m");
            frame.push_str(app.theme.style());
        }
    }
    if app.use_ansi {frame.push_str("\x1b[0m");}
    frame
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Profile,Theme};
    use three_kingdoms_spire_reborn::{Game,Action,content::Hero};
    fn app(game:Option<Game>)->App {
        App{game,profile:Profile::new(),theme:Theme::Dark,
            dir:std::path::PathBuf::new(),use_ansi:false,message:String::new()}
    }
    #[test]
    fn narrow_and_tall_frames_never_overflow_terminal_cells() {
        let mut game=Game::new(Hero::GuanYu,127);
        game.apply(Action::Choose{index:1}).unwrap();
        let app=app(Some(game));
        let options=super::super::choices(&app,1);
        for (rows,cols) in [(16,32),(24,60),(28,80),(42,120)] {
            for cursor in [0,3,options.len().saturating_sub(1)] {
                let frame=render_frame(&app,1,cursor,&options,rows,cols);
                assert!(frame.lines().count()<=rows,"rows={rows}");
                assert!(frame.lines().all(|line|UnicodeWidthStr::width(line)<=cols),
                    "columns={cols}");
                assert!(frame.contains(&format!("{:>2}. ",cursor.min(options.len()-1)+1)),
                    "selected item scrolled away at {rows}x{cols}");
            }
        }
    }
    #[test]
    fn selected_target_and_messages_are_visible_without_plain_cli_duplication() {
        let mut game=Game::new(Hero::GuanYu,102);
        game.apply(Action::Choose{index:1}).unwrap();
        let mut app=app(Some(game));
        app.message="本回合操作成功".into();
        let options=super::super::choices(&app,1);
        let frame=render_frame(&app,1,0,&options,24,80);
        assert!(frame.contains("目标 敌1"));
        assert!(frame.contains("本回合操作成功"));
        assert!(frame.contains("可执行操作"));
        assert!(!frame.contains("三国登塔录 >"));
    }
    #[test]
    fn tiny_width_still_clips_cjk_safely() {
        let app=app(None);
        let options=super::super::choices(&app,1);
        let frame=render_frame(&app,1,4,&options,12,20);
        assert!(frame.lines().all(|line|UnicodeWidthStr::width(line)<=20));
    }
}
