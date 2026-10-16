//! Unicode-width-aware, read-only interactive battle HUD.
//! The original plain command interface and deterministic game core are unchanged.
use super::{App,Stage};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn width(text:&str)->usize {UnicodeWidthStr::width(text)}
fn fit(text:&str,limit:usize)->String {
    if limit==0 {return String::new();}
    let safe:String=text.chars().filter(|ch|!ch.is_control()).collect();
    if width(&safe)<=limit {return safe;}
    let mut result=String::new();
    let mut used=0;
    for ch in safe.chars() {
        let cells=UnicodeWidthChar::width(ch).unwrap_or(0);
        if used+cells>limit.saturating_sub(1) {break;}
        result.push(ch);
        used+=cells;
    }
    result.push('…');
    result
}
fn panel(out:&mut Vec<String>,title:&str,body:&[String],columns:usize) {
    let n=columns.max(8);
    let header=fit(title,n-5);
    out.push(format!("┌─{}{}┐",header,"─".repeat(n-3-width(&header))));
    let inner=n-4;
    for row in body {
        let s=fit(row,inner);
        out.push(format!("│ {}{} │",s," ".repeat(inner-width(&s))));
    }
    out.push(format!("└{}┘","─".repeat(n-2)));
}
fn window(total:usize,cursor:usize,available:usize)->(usize,usize) {
    if total==0 {return (0,0);}
    let count=available.max(1).min(total);
    let start=cursor.saturating_sub(count/2).min(total-count);
    (start,start+count)
}
pub(super) fn render(app:&App,columns:usize,rows:usize,target:usize,
    options:&[(String,String)],cursor:usize)->String
{
    let columns=columns.clamp(28,120);
    let rows=rows.max(16);
    let mut out=Vec::<String>::new();
    let title=format!("三国登塔录 / TKS · {} · 自动保存",app.theme.name());
    if let Some(g)=&app.game {
        let act=(g.floor.saturating_sub(1)/7+1).min(3);
        let status=vec![
            format!("{}  体力 {}/{}  金 {}  药 {}  第{}幕·第{}层",
                g.hero.name(),g.hp,g.max_hp,g.gold,g.potions,act,g.floor),
            format!("{:?} A{}  ·  {}  ·  战法：{}",
                g.mode,g.ascension,g.act_name(),
                g.doctrine.map_or("自动",|doctrine|doctrine.label())),
        ];
        panel(&mut out,&title,&status,columns);
        if let Some(b)=g.battle.as_ref().filter(|_|g.stage==Stage::Combat) {
            let foes=b.foes.iter().enumerate().map(|(i,e)|format!(
                "{} {}. {}  {}/{}  甲{} 毒{}  意图：{}{}",
                if i+1==target && e.hp>0 {"▶"} else {" "},i+1,e.name(),
                e.hp,e.max_hp,e.block,e.poison,e.intent.label(),
                if e.hp==0 {" [阵亡]"}else if e.phase>0 {" [二阶段]"}else{""}
            )).collect::<Vec<_>>();
            panel(&mut out,&format!("敌军 · 当前目标 {}（t 切换）",target),&foes,columns);
            let mut stats=vec![
                format!("第{}回合  军令 {}  护甲 {}  武勇 {}  虚弱 {}  易伤 {}",
                    b.turn,b.energy,b.block,b.strength,b.weak,b.vulnerable),
                format!("战意 {}  谋略 {}  连段 {}  统御 {}  怒气 {}  反击 {}",
                    b.momentum,b.scheme,b.combo,b.command,b.fury,b.thorns),
            ];
            if b.troops_hp>0 {stats.push(format!("亲兵 {} HP · 每回合攻击 {}",b.troops_hp,b.troops_attack));}
            if !b.powers.is_empty() && rows>=32 {
                stats.push(format!("持续军略：{}",b.powers.iter()
                    .map(|p|format!("{}×{}",p.id.name(),p.stacks))
                    .collect::<Vec<_>>().join("、")));
            }
            if b.turn>=35 {stats.push(if b.turn<40 {
                format!("军粮警报：距损耗还剩{}回合",40-b.turn)
            }else{
                format!("军粮枯竭：回合结束失去{}生命",1+(b.turn-40)/4)
            });}
            panel(&mut out,"我方状态",&stats,columns);
        }else {
            let hint=match g.stage {
                Stage::Map=>"选择下一站；:map 查看七层路线",
                Stage::Reward=>"选择战利品牌，或放弃奖励",
                Stage::Camp=>"休整、升级或操练；:upgrade N 可定向升级",
                Stage::Market=>"购买卡牌、药剂和遗物；:remove N 可移牌",
                Stage::Story=>"驿道奇遇：选择剧情结果",
                Stage::Ancient=>"选择长期先贤赐福",
                Stage::Won=>"三幕皆定，江山已平",
                Stage::Lost=>"征程结束，可以重新出征",
                Stage::Combat=>"正在战斗",
            };
            panel(&mut out,"当前场景",&[hint.into()],columns);
        }
        let report=rows>=23;
        let response=!app.message.is_empty();
        let reserved=usize::from(report)*3+usize::from(response)*3+1;
        let available=rows.saturating_sub(out.len()+reserved+2).max(1);
        let (start,end)=window(options.len(),cursor,available);
        let mut menu=options.iter().enumerate().take(end).skip(start).map(|(i,(label,_))|
            format!("{} {:>2}. {}",if i==cursor {"▶"}else{" "},i+1,label)
        ).collect::<Vec<_>>();
        let head=if let Some(b)=g.battle.as_ref().filter(|_|g.stage==Stage::Combat) {
            format!("手牌与行动 · 抽{} 弃{} 耗{}  [{}-{} / {}]",
                b.draw.len(),b.discard.len(),b.exhausted.len(),start+1,end,options.len())
        } else {
            format!("行动选择 [{}-{} / {}]",start+usize::from(!options.is_empty()),end,options.len())
        };
        if menu.is_empty() {menu.push("没有可选操作；按 : 输入命令".into());}
        panel(&mut out,&head,&menu,columns);
        if report {
            panel(&mut out,"最近战报",
                &[g.log.last().cloned().unwrap_or_else(||"暂无战报".into())],columns);
        }
        if response {panel(&mut out,"操作反馈",&[app.message.clone()],columns);}
    }else{
        panel(&mut out,&title,&["尚未出征 · 选择一名武将开始新战役".into()],columns);
        let reserved=1+usize::from(!app.message.is_empty())*3;
        let (start,end)=window(options.len(),cursor,rows.saturating_sub(out.len()+reserved+2));
        let menu=options.iter().enumerate().take(end).skip(start)
            .map(|(i,(label,_))|format!("{} {:>2}. {}",
                if i==cursor {"▶"}else{" "},i+1,label)).collect::<Vec<_>>();
        panel(&mut out,"选将",&menu,columns);
        if !app.message.is_empty(){panel(&mut out,"操作反馈",&[app.message.clone()],columns);}
    }
    let help=fit("↑↓/jk 选择  Enter确认  1-9快捷  t目标  :命令  ?帮助  q退出",columns);
    if out.len()>=rows {out.truncate(rows-1);}
    out.push(help);
    format!("{}\n",out.join("\n"))
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
    fn constrained_screens_fit_both_dimensions_and_scroll_to_selected_action(){
        let mut game=Game::new(Hero::GuanYu,83);
        game.apply(Action::Choose{index:1}).unwrap();
        let app=app(Some(game));
        let choices=tui::choices(&app,1);
        for (cols,rows) in [(28,16),(40,24),(80,24),(120,38)] {
            let content=render(&app,cols,rows,1,&choices,choices.len()-1);
            assert!(content.lines().count()<=rows,"{cols}x{rows}: {content}");
            assert!(content.lines().all(|s|width(s)<=cols),"{cols}x{rows}: {content}");
            assert!(content.contains("结束本回合"),"{content}");
            assert!(content.contains("敌军")&&content.contains("军令"),"{content}");
        }
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
