//! Optional full-keyboard terminal menu for macOS/Linux without platform crates.
//! Raw mode is scoped by RAII, and the existing plain line CLI remains supported.
use super::{App, Stage, card_label};
#[path = "tui_layout.rs"]
mod layout;
use three_kingdoms_spire_reborn::challenges::ChallengeModifier;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use std::{fs::File,io::{self, Read, Write, IsTerminal},process::{Command,Stdio}};

struct RawTerminal { original: String }
impl RawTerminal {
    fn stty(args:&[&str])->io::Result<std::process::Output> {
        let tty=File::open("/dev/tty")?;
        Command::new("stty").args(args).stdin(Stdio::from(tty)).output()
    }
    fn new()->io::Result<Self> {
        let initial=Self::stty(&["-g"])?;
        if !initial.status.success() {return Err(io::Error::other("stty -g failed"));}
        let old=String::from_utf8(initial.stdout)
            .map_err(|e|io::Error::other(e.to_string()))?.trim().to_owned();
        if old.is_empty() {return Err(io::Error::other("could not read terminal mode"));}
        let result=Self::stty(&["-icanon","-echo","min","1","time","0"])?;
        if !result.status.success() {return Err(io::Error::other("could not set raw terminal"));}
        Ok(Self{original:old})
    }
}
impl Drop for RawTerminal {
    fn drop(&mut self) {let _=Self::stty(&[self.original.as_str()]);}
}
pub(super) fn choices(app:&App,target:usize)->Vec<(String,String)> {
    let Some(game)=app.game.as_ref() else {
        return vec![
            ("关羽·青龙偃月".into(),"new guanyu".into()),
            ("诸葛亮·八卦谋略".into(),"new zhuge".into()),
            ("孙尚香·连弩追击".into(),"new sun".into()),
            ("曹操·亲兵统御".into(),"new cao".into()),
            ("吕布·无双战魂".into(),"new lubu".into()),
        ];
    };
    match game.stage {
        Stage::Map=>game.route.iter().enumerate()
            .map(|(idx,node)|{
                let lane=game.route_lanes.get(idx).copied().unwrap_or(idx as u8);
                let row=usize::from(game.floor%7);
                let details=game.act_map.as_ref().map(|map|format!(
                    " · {} · 三层评分{}",
                    map.terrain_at(row,lane).label(),
                    map.score_route(row,lane,game.hp,game.max_hp,game.gold)));
                (format!("{} · 道{}{}",node.name(),lane+1,details.unwrap_or_default()),
                    format!("choose {}",idx+1))
            }).collect(),
        Stage::Combat=>{
            let mut options=game.battle.as_ref().map_or(vec![],|b|
                b.hand.iter().enumerate()
                    .map(|(index,card)|(format!("{}（目标敌军{}）",card_label(card),target),
                        format!("play {} {target}",index+1))).collect::<Vec<_>>());
            options.push(("结束本回合".into(),"end".into()));
            for (index,potion) in game.potion_bag.iter().enumerate() {
                options.push((format!("药剂{}：{}",index+1,
                    three_kingdoms_spire_reborn::items::potion_name(potion)),
                    format!("use {} {target}",index+1)));
            }
            options
        }
        Stage::Reward=> {
            let mut options=game.rewards.iter().enumerate()
                .map(|(idx,card)|(format!("选择 {}",card_label(card)),format!("choose {}",idx+1)))
                .collect::<Vec<_>>();
            options.push(("放弃奖励".into(),"choose 0".into()));
            options
        }
        Stage::Camp=>{
            let mut options = Vec::new();
            if !game.modifiers.contains(&ChallengeModifier::NoRest) {
                options.push(("扎营休整".into(), "choose 1".into()));
            }
            options.push(("强化牌组首张可升级卡".into(), "choose 2".into()));
            options.push(("操练体魄：生命上限+2".into(), "choose 3".into()));
            options
        },
        Stage::Market=> {
            let mut offers=game.market.iter().enumerate()
                .map(|(index,card)|(format!("购买 {} / {} 金",card_label(card),45+15*index),
                    format!("choose {}",index+1))).collect::<Vec<_>>();
            offers.extend([
                ("随机药剂 / 55 金".into(),"choose 4".into()),
                ("遗物 / 155 金".into(),"choose 5".into()),
                ("移除基础牌 / 75 金".into(),"choose 6".into()),
                ("离开军市".into(),"choose 0".into()),
            ]);
            offers
        }
        Stage::Story=>vec![
            ("选择奇遇结果1".into(),"choose 1".into()),
            ("选择奇遇结果2".into(),"choose 2".into()),
            ("选择奇遇结果3".into(),"choose 3".into()),
        ],
        Stage::Ancient=>game.boon_choices.iter().enumerate()
            .map(|(idx,id)|(format!("先贤赐福：{}",three_kingdoms_spire_reborn::items::boon_name(id)),
                format!("choose {}",idx+1))).collect(),
        Stage::Won|Stage::Lost=>vec![
            ("用关羽开启新征程".into(),"new guanyu".into()),
            ("打开战功簿".into(),"chronicle".into()),
        ],
    }
}
/// Truncate by display cells rather than bytes; never split a Han character.
fn fitting_label(label: &str, available: usize) -> String {
    if available==0 {return String::new();}
    let clean: String=label.chars().map(|ch|if ch.is_control(){' '}else{ch}).collect();
    if UnicodeWidthStr::width(clean.as_str())<=available {return clean;}
    if available==1 {return "…".into();}
    let mut out=String::new();
    let mut width=0;
    for ch in clean.chars() {
        let cells=UnicodeWidthChar::width(ch).unwrap_or(0);
        if width+cells>available-1 {break;}
        out.push(ch);
        width+=cells;
    }
    out.push('…');
    out
}
fn current_size()->(usize,usize) {
    RawTerminal::stty(&["size"]).ok()
        .and_then(|out|String::from_utf8(out.stdout).ok())
        .and_then(|raw|{
            let mut parts=raw.split_whitespace();
            Some((parts.next()?.parse::<usize>().ok()?,
                  parts.next()?.parse::<usize>().ok()?))
        })
        .map_or((24,80),|(rows,cols)|(rows.max(1),cols.max(1)))
}
fn read_byte()->io::Result<u8> {
    let mut byte=[0_u8;1];
    io::stdin().read_exact(&mut byte)?;
    Ok(byte[0])
}
fn typed_command()->io::Result<String> {
    let (rows,_)=current_size();
    print!("\x1b[?25h\x1b[{rows};1H\x1b[2K: ");
    io::stdout().flush()?;
    let mut value=String::new();
    let mut pending_utf8=Vec::new();
    loop {
        match read_byte()? {
            b'\r'|b'\n'=>{print!("\x1b[?25l");return Ok(value);}
            27=>{print!("\x1b[?25l");return Ok(String::new());}
            8|127=>{
                if !pending_utf8.is_empty() { pending_utf8.clear(); }
                else if let Some(ch)=value.pop() {
                    for _ in 0..UnicodeWidthChar::width(ch).unwrap_or(1).max(1) {
                        print!("\x08 \x08");
                    }
                }
            }
            byte if (byte.is_ascii_graphic()||byte==b' ') && value.len()<300 =>{
                pending_utf8.clear();
                value.push(char::from(byte));
                print!("{}",char::from(byte));
            }
            byte if byte >= 0x80 && value.len()<300 => {
                pending_utf8.push(byte);
                match std::str::from_utf8(&pending_utf8) {
                    Ok(utf8) => {
                        let complete=utf8.to_owned();
                        pending_utf8.clear();
                        value.push_str(&complete);
                        print!("{complete}");
                    }
                    Err(err) if err.error_len().is_some() || pending_utf8.len()>=4 => pending_utf8.clear(),
                    Err(_) => {}
                }
            }
            _=>{}
        }
        io::stdout().flush()?;
    }
}
struct AlternateScreen;
impl AlternateScreen {
    fn enter()->io::Result<Self> {
        print!("\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H");
        io::stdout().flush()?;
        Ok(Self)
    }
}
impl Drop for AlternateScreen {
    fn drop(&mut self) {
        print!("\x1b[0m\x1b[?25h\x1b[?1049l");
        let _=io::stdout().flush();
    }
}
pub(super) fn run(app:&mut App)->Result<(),String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("交互式TUI需要真实终端；管道脚本请使用 --plain".into());
    }
    let _raw=RawTerminal::new().map_err(|e|format!("无法启用键盘操作: {e}"))?;
    let _screen=AlternateScreen::enter().map_err(|e|e.to_string())?;
    let mut cursor=0usize;
    let mut target=1usize;
    loop {
        let options=choices(app,target);
        if options.is_empty() {cursor=0;}
        else {cursor=cursor.min(options.len()-1);}
        let (rows,columns)=current_size();
        let frame=layout::render_frame(app,target,cursor,&options,rows,columns);
        print!("\x1b[H\x1b[2J{frame}");
        io::stdout().flush().map_err(|e|e.to_string())?;
        let key=read_byte().map_err(|e|e.to_string())?;
        let decision=match key {
            b'j'=>{cursor=cursor.saturating_add(1).min(options.len().saturating_sub(1));None}
            b'k'=>{cursor=cursor.saturating_sub(1);None}
            b'\r'|b'\n'=>options.get(cursor).map(|(_,cmd)|cmd.clone()),
            b'1'..=b'9'=>options.get(usize::from(key-b'1')).map(|(_,cmd)|cmd.clone()),
            b't'=>{
                if let Some(battle)=app.game.as_ref().and_then(|g|g.battle.as_ref()) {
                    let living=battle.foes.iter().enumerate().filter(|(_,f)|f.hp>0)
                        .map(|(i,_)|i+1).collect::<Vec<_>>();
                    let position=living.iter().position(|id|*id==target).unwrap_or(0);
                    target=living.get((position+1)%living.len().max(1)).copied().unwrap_or(1);
                }
                None
            }
            b'?'=>Some("help".into()),
            b':' => {
                let command=typed_command().map_err(|e|e.to_string())?;
                if command.is_empty(){None}else{Some(command)}
            }
            27=>{
                let second=read_byte().map_err(|e|e.to_string())?;
                if second==b'[' {
                    let third=read_byte().map_err(|e|e.to_string())?;
                    if third==b'A' {cursor=cursor.saturating_sub(1);}
                    if third==b'B' {cursor=cursor.saturating_add(1).min(options.len().saturating_sub(1));}
                }
                None
            }
            b'q'=>{let _=app.save();return Ok(());}
            _=>None,
        };
        if let Some(command)=decision {
            let previous=app.game.as_ref().map(|g|g.stage);
            if !app.command(&command) {return Ok(());}
            if app.game.as_ref().map(|g|g.stage)!=previous {cursor=0;}
            if !matches!(app.game.as_ref().map(|g|g.stage),Some(Stage::Combat)) {target=1;}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Profile,Theme};
    use three_kingdoms_spire_reborn::{Game,content::Hero};
    fn app(game:Option<Game>)->App {
        App{game,profile:Profile::new(),theme:Theme::Dark,
            dir:std::path::PathBuf::new(),use_ansi:false,message:String::new()}
    }
    #[test]
    fn keyboard_menu_is_stage_aware_and_targeted() {
        let no_game=app(None);
        assert_eq!(choices(&no_game,1).len(),5);
        let mut game=Game::new(Hero::SunShangxiang,814);
        let menu=choices(&app(Some(game.clone())),1);
        assert_eq!(menu.len(),game.route.len());
        assert!(menu[0].1.starts_with("choose "));
        game.apply(three_kingdoms_spire_reborn::Action::Choose{index:1}).unwrap();
        let battle=choices(&app(Some(game)),2);
        assert!(battle.iter().any(|(_,cmd)|cmd=="end"));
        assert!(battle.iter().any(|(_,cmd)|cmd=="play 1 2"));
    }
}

#[cfg(test)]
mod p1_accessibility_tests {
    use super::*;
    use three_kingdoms_spire_reborn::{Action, Game, RunMode, content::Hero};

    #[test]
    fn narrow_tty_clips_cjk_without_splitting_codepoints() {
        let label = "大战·孙尚香出征";
        let short = fitting_label(label, 8);
        assert!(UnicodeWidthStr::width(short.as_str()) <= 8);
        assert!(short.ends_with('…'));
        assert_eq!(fitting_label(label, 80), label);
    }

    #[test]
    fn disabled_camp_rest_is_not_selectable_by_keyboard() {
        let mut game = Game::new_with_mode(Hero::GuanYu, 77, 0, RunMode::Custom);
        game.apply(Action::SetModifier { id: "no_rest".into(), enabled: true }).unwrap();
        game.stage = Stage::Camp;
        let app = App {
            game: Some(game),
            profile: crate::Profile::new(),
            theme: crate::Theme::Dark,
            dir: std::path::PathBuf::new(),
            use_ansi: false,
            message: String::new(),
        };
        let options = choices(&app, 1);
        assert!(!options.iter().any(|(_, action)|action == "choose 1"));
        assert!(options.iter().any(|(_, action)|action == "choose 2"));
    }
}
