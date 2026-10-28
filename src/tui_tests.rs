use super::*;

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
    fn market_keyboard_menu_identifies_permanent_card_acquisition() {
        let mut game = Game::new(Hero::GuanYu, 93403);
        game.stage = Stage::Market;
        game.market = vec![Card::new("huogong")];
        let choices = choices(&app(Some(game)), 1);
        assert!(choices.iter().any(|(label, command)|
            label.contains("购入牌组") && label.contains("火攻")
                && command == "choose 1"));
    }
    #[test]
    fn secondary_keyboard_menu_uses_top_first_numbers_and_can_cancel_without_mutation() {
        let mut game = Game::new(Hero::ZhugeLiang, 983);
        game.apply(three_kingdoms_spire_reborn::Action::Choose { index: 1 }).unwrap();
        {
            let battle = game.battle.as_mut().unwrap();
            battle.hand = vec![Card::new("xingpan_tutor")];
            battle.draw = vec![Card::new("defend"), Card::new("strike"), Card::new("qinglong")];
        }
        let before = game.to_json().unwrap();
        let app = app(Some(game));
        let command = "play 1 2";
        assert_eq!(selected_hand(&app,command), Some(1));
        let menu = secondary_menu(&app,1,2);
        assert_eq!(menu[0].1, "play 1 2 1");
        assert_eq!(menu[1].1, "play 1 2 2");
        assert!(menu.iter().all(|(_,command)|command != "play 1 2 3"));
        assert!(menu.iter().any(|(_,command)|command == "@cancel"));
        assert!(menu.iter().any(|(_,command)|command == "play 1 2"));
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(),before);
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
        assert!(battle.iter().any(|(_,cmd)|cmd=="events"));
        assert!(battle.iter().any(|(_,cmd)|cmd=="play 1 2"));
    }
}

#[cfg(test)]
mod p1_accessibility_tests {
    use super::*;
    use crate::{Profile, Theme};
    use three_kingdoms_spire_reborn::{Action, Game, RunMode, content::Hero};

    #[test]
    fn potion_menu_exposes_single_target_and_all_foe_scope() {
        let mut game = Game::new(Hero::GuanYu, 8184);
        game.apply(Action::Choose { index: 1 }).unwrap();
        game.potion_bag = vec!["poison".into(), "weak".into()];
        game.potions = 2;
        let menu = choices(&App { game: Some(game), profile: Profile::new(),
            theme: Theme::Dark, dir: std::path::PathBuf::new(),
            use_ansi: false, message: String::new() }, 2);
        assert!(menu.iter().any(|(label, command)|
            label.contains("→目标2") && command == "use 1 2"));
        assert!(menu.iter().any(|(label, command)|
            label.contains("→敌方全体") && command == "use 2 2"));
    }
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


#[cfg(test)]
mod narrative_choice_tests {
    use super::*;
    use crate::{Profile, Theme};
    use three_kingdoms_spire_reborn::{Game, content::Hero};

    #[test]
    fn authored_story_menu_exposes_real_choices_instead_of_numbers_only() {
        let mut game = Game::new(Hero::GuanYu, 42);
        game.stage = Stage::Story;
        game.lore_current = Some("taoyuan".into());
        let beat = three_kingdoms_spire_reborn::lore::beat("taoyuan").unwrap();
        let before = game.to_json().unwrap();
        let app = App { game: Some(game), profile: Profile::new(),
            theme: Theme::Dark, dir: std::path::PathBuf::new(),
            use_ansi: false, message: String::new() };
        let menu = choices(&app, 1);
        assert_eq!(menu.iter().map(|pair| pair.1.as_str()).collect::<Vec<_>>(),
            vec!["choose 1", "choose 2", "choose 3"]);
        assert!(menu[0].0.contains(beat.vow));
        assert!(menu[1].0.contains(beat.bargain));
        assert!(menu[2].0.contains("不介入"));
        assert_eq!(app.game.as_ref().unwrap().to_json().unwrap(), before);
    }
}
