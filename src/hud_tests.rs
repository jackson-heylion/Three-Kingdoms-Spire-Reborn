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
            full.replace(['\n', ' '], ""));
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

#[cfg(test)]
mod target_and_class_resource_tests {
    use super::*;
    use crate::{Profile, Theme, tui};
    use three_kingdoms_spire_reborn::{Action, Game, content::Hero};

    fn battle(hero: Hero) -> App {
        let mut game = Game::new(hero, 20261028);
        game.apply(Action::Choose { index: 1 }).unwrap();
        App { game: Some(game), profile: Profile::new(), theme: Theme::Light,
            dir: std::path::PathBuf::new(), use_ansi: false, message: String::new() }
    }

    #[test]
    fn primary_resource_is_always_visible_even_at_zero_before_enemies() {
        for (hero, label) in [
            (Hero::GuanYu, "战意"), (Hero::ZhugeLiang, "谋略"),
            (Hero::SunShangxiang, "连段"), (Hero::CaoCao, "统御"),
            (Hero::LuBu, "怒气"),
        ] {
            let mut app = battle(hero);
            let g = app.game.as_mut().unwrap();
            let b = g.battle.as_mut().unwrap();
            b.momentum = 0;
            b.scheme = 0;
            b.combo = 0;
            b.command = 0;
            b.fury = 0;
            let choices = tui::choices(&app, 1);
            let snapshot = app.game.as_ref().unwrap().to_json().unwrap();
            for (columns, rows) in [(28, 16), (40, 20), (80, 24), (120, 38)] {
                let output = render_mode(&app, columns, rows, 1, &choices, 0, false);
                let resource = format!("{label} 0");
                assert!(output.contains(&resource), "{hero:?}, {columns}x{rows}: {output}");
                assert!(output.find(&resource) < output.find("敌军"), "{output}");
                assert!(output.contains("军令") && output.contains("护甲"), "{output}");
                assert!(output.lines().count() <= rows, "{output}");
                assert!(output.lines().all(|line| width(line) <= columns), "{output}");
                assert!(output.contains("▶"), "{output}");
            }
            assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(), snapshot);
        }
    }

    #[test]
    fn active_secondary_meter_is_not_hidden_behind_foes() {
        let mut app = battle(Hero::GuanYu);
        let b = app.game.as_mut().unwrap().battle.as_mut().unwrap();
        b.momentum = 0;
        b.command = 2;
        let choices = tui::choices(&app, 1);
        let output = render_mode(&app, 40, 20, 1, &choices, 0, false);
        assert!(output.contains("战意 0") && output.contains("统御 2"), "{output}");
        assert!(output.find("统御 2") < output.find("敌军"), "{output}");
    }

    #[test]
    fn selected_enemy_uses_dark_orange_in_light_theme_not_bright_yellow() {
        let mut app = battle(Hero::GuanYu);
        let options = tui::choices(&app, 1);
        app.use_ansi = true;
        for (theme, code) in [
            (Theme::Light, "\x1b[1;38;2;145;64;0m"),
            (Theme::Dark, "\x1b[1;38;5;215m"),
            (Theme::HighContrast, "\x1b[1;97;44m"),
        ] {
            app.theme = theme;
            let screen = render_mode(&app, 80, 28, 1, &options, 0, false);
            let selected = screen.lines().find(|line| line.contains('▶') && line.contains("HP"))
                .expect("selected living enemy must be on screen");
            assert!(selected.starts_with(&format!("{code}▶")), "{theme:?} {selected}");
            if matches!(theme, Theme::Light) {
                assert!(!selected.contains("\x1b[1;33m"), "{selected}");
            }
        }
    }
}

#[cfg(test)]
mod player_enemy_report_tests {
    use super::*;
    use crate::{Profile,Theme,tui,combat_report,hud_reading};
    use three_kingdoms_spire_reborn::{Action,Game,content::Hero};

    #[test]
    fn visible_combat_panel_identifies_both_sides_and_readable_journal_is_lossless() {
        let mut game = Game::new(Hero::GuanYu, 123);
        game.apply(Action::Choose{index:1}).unwrap();
        {
            let b=game.battle.as_mut().unwrap();
            b.foes[0].phase=1;
            b.foes[0].intent=three_kingdoms_spire_reborn::Intent::Attack(5);
            b.troops_hp=0;
            b.troops_attack=0;
        }
        game.apply(Action::EndTurn).unwrap();
        let app=App{game:Some(game),profile:Profile::new(),theme:Theme::Dark,
            dir:std::path::PathBuf::new(),use_ansi:false,message:"回合结束".into()};
        let options=tui::choices(&app,1);
        let snapshot=app.game.as_ref().unwrap().to_json().unwrap();
        let view=render_mode(&app,80,30,1,&options,0,false);
        assert!(view.contains("双方行动"),"{view}");
        assert!(view.contains("敌方") || view.contains("实际损失"),"{view}");
        assert!(view.contains("l战报"),"{view}");
        let document=combat_report::document(&app);
        assert!(document.contains("双方事件流水"));
        assert!(document.contains("实际损失"),"{document}");
        let all=hud_reading::battle_lines(&app,28);
        assert_eq!(all.join(""),document.replace('\n',""));
        for (columns,rows) in [(28,16),(40,20),(80,30)] {
            let max=hud_reading::battle_max_offset(&app,columns,rows);
            for offset in [0,max,usize::MAX] {
                let screen=hud_reading::render_battle(&app,columns,rows,offset);
                assert!(screen.lines().count()<=rows,"{screen}");
                assert!(screen.lines().all(|line|width(line)<=columns),"{screen}");
            }
        }
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(),snapshot);
    }
}
