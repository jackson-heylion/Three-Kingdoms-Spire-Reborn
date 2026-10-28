use super::*;

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
mod narrative_reading_tests {
    use super::*;
    use crate::{Profile, Theme, tui, hud_reading};
    use three_kingdoms_spire_reborn::{Game, content::Hero};

    fn story() -> App {
        let mut game = Game::new(Hero::GuanYu, 42);
        game.stage = Stage::Story;
        game.lore_current = Some("taoyuan".into());
        App { game: Some(game), profile: Profile::new(), theme: Theme::Dark,
            dir: std::path::PathBuf::new(), use_ansi: false, message: String::new() }
    }

    #[test]
    fn narrative_intro_visible_and_options_still_selectable_on_small_screen() {
        let app = story();
        let options = tui::choices(&app, 1);
        let before = app.game.as_ref().unwrap().to_json().unwrap();
        for (columns, rows) in [(28, 16), (40, 20), (80, 24)] {
            let rendered = render_mode(&app, columns, rows, 1, &options, 0, false);
            assert!(rendered.contains("演义残卷"), "{rendered}");
            assert!(rendered.contains("行动选择"), "{rendered}");
            assert!(rendered.contains("▶"), "{rendered}");
            assert!(rendered.lines().count() <= rows);
            assert!(rendered.lines().all(|line| width(line) <= columns), "{rendered}");
        }
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(), before);
    }

    #[test]
    fn long_chinese_text_is_scrollable_without_content_loss_or_state_changes() {
        let mut app = story();
        app.message = "操作反馈很长，包含中文标点和不同史料版本。".repeat(35);
        let options = tui::choices(&app, 1);
        let before = app.game.as_ref().unwrap().to_json().unwrap();
        let full = hud_reading::document(&app, &options, 0);
        assert!(full.contains(&app.message));
        assert!(full.contains("【当前演义剧情】"));
        let all_lines = hud_reading::lines(&app, 28, &options, 0);
        assert_eq!(all_lines.join("").replace(' ', ""),
            full.replace('\n', "").replace(' ', ""));
        let last = hud_reading::max_offset(&app, 28, 16, &options, 0);
        assert!(last > 0);
        for offset in [0, last / 2, last, usize::MAX] {
            let page = hud_reading::render(&app, 28, 16, &options, 0, offset);
            assert!(page.lines().count() <= 16, "{page}");
            assert!(page.lines().all(|line| width(line) <= 28), "{page}");
        }
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(), before);
    }

    #[test]
    fn preview_marks_hidden_remainder_and_handles_cjk_cell_width() {
        let long = "长篇剧情的因果证词和人物选择".repeat(8);
        let short = hud_reading::preview(&long, 28, 2);
        assert_eq!(short.len(), 2);
        assert!(short.last().unwrap().contains("i全文"));
        assert!(short.iter().all(|line| width(line) <= 28));
        assert_eq!(hud_reading::wrapped(&long, 28).join(""), long);
        let hostile = hud_reading::wrapped("A\x1b[2J董卓\r\n孙权", 7);
        assert!(hostile.iter().all(|line| !line.contains('\x1b') && width(line) <= 7));
    }
}
