//! Chapters 036–038: independently pinned text, optional responsibility choices,
//! stable legacy routes, atomic refusal, save/restore and once-only cross-act rewards.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat};

fn at(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 36038);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|f| (*f).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn has(g: &Game, flag: &str) -> bool { g.lore_flags.iter().any(|f| f == flag) }
fn awards(g: &Game, id: &str) -> usize { g.easter_unlocked.iter().filter(|f| f.as_str() == id).count() }

#[test]
fn fixed_sources_and_literary_order_are_auditable() {
    for (ch, oldid, id) in [
        (36, "2568002", "fancheng_counsel"),
        (37, "2567998", "xushu_departure"),
        (38, "4256828", "longzhong_conditions"),
    ] {
        let doc = match ch {
            36 => include_str!("../docs/CHAPTER_036_READING.md"),
            37 => include_str!("../docs/CHAPTER_037_READING.md"),
            _ => include_str!("../docs/CHAPTER_038_READING.md"),
        };
        assert!(doc.contains(oldid) && doc.contains("单底本"));
        assert_eq!(beat(id).unwrap().chapter, ch);
    }
    assert_eq!(beat("jiangdong_hostage").unwrap().chapter, 38);
    assert_eq!(beat("fancheng_counsel").unwrap().prerequisite, Some("xinye_counsel_record"));
    assert_eq!(beat("xushu_departure").unwrap().prerequisite, Some("fancheng_counsel_record"));
    assert_eq!(beat("longzhong_conditions").unwrap().prerequisite, Some("longzhong_plan"));
    assert_eq!(beat("jiangdong_hostage").unwrap().prerequisite, Some("sun_succession_account"));
    assert!(beat("xushu_departure").unwrap().premise.contains("两访均未见孔明"));
    assert!(beat("jiangdong_hostage").unwrap().premise.contains("回末黄祖未死"));
    assert_eq!(next_beat(Hero::GuanYu, 2, &["xinye_counsel_record".into(), "anxi_seal".into()], &[]).map(|b| b.id), Some("fancheng_counsel"));
}

#[test]
fn guan_yu_fancheng_to_jingzhou_is_atomic_persistent_and_once_only() {
    let mut g = at(Hero::GuanYu, "fancheng_counsel", &["xinye_counsel_record", "anxi_seal"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "fancheng_counsel_record"));
    let saved = g.to_json().unwrap();
    let mut g = Game::from_json(&saved).unwrap();
    assert_eq!(g.to_json().unwrap(), saved);
    enter(&mut g, "xushu_departure");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "xushu_departure_record"));
    assert_eq!(awards(&g, "fancheng_xushu_jingzhou"), 0);
    enter(&mut g, "jingzhou");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "fancheng_xushu_jingzhou"), 1);
    enter(&mut g, "jingzhou");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "fancheng_xushu_jingzhou"), 1);
}

#[test]
fn longzhong_conditions_are_optional_and_pay_only_after_red_cliffs() {
    let mut g = at(Hero::ZhugeLiang, "longzhong", &[]);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "longzhong_plan"));
    assert_eq!(next_beat(Hero::ZhugeLiang, 1, &g.lore_flags, &[]).map(|b| b.id), Some("longzhong_conditions"));
    enter(&mut g, "longzhong_conditions");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 0 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "longzhong_conditions_record"));
    assert_eq!(awards(&g, "longzhong_strategy_debt"), 0);
    let mut g = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g, "redcliff");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "longzhong_strategy_debt"), 1);
    enter(&mut g, "redcliff");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "longzhong_strategy_debt"), 1);
}

#[test]
fn jiangdong_refusal_does_not_block_existing_wedding_or_yiling() {
    let mut g = at(Hero::SunShangxiang, "jiangdong_hostage", &["sun_succession_account"]);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!has(&g, "jiangdong_hostage_record"));
    enter(&mut g, "jiangdong_wedding");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "jiangdong_covenant"));
    enter(&mut g, "yiling");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "yiling_ashes"));
    assert_eq!(awards(&g, "jiangdong_guest_debt"), 0);
}

#[test]
fn jiangdong_hostage_choice_pays_after_existing_cross_act_events_once() {
    let mut g = at(Hero::SunShangxiang, "jiangdong_hostage", &["sun_succession_account"]);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&g, "jiangdong_hostage_record"));
    enter(&mut g, "jiangdong_wedding");
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "yiling");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "jiangdong_guest_debt"), 1);
    let mut g = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut g, "yiling");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(awards(&g, "jiangdong_guest_debt"), 1);
}

#[test]
fn legacy_saves_without_new_flags_still_reach_original_payoffs() {
    let mut guan = at(Hero::GuanYu, "jingzhou", &["anxi_seal"]);
    guan.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&guan, "jingzhou_warning"));
    assert_eq!(awards(&guan, "fancheng_xushu_jingzhou"), 0);
    let mut zhuge = at(Hero::ZhugeLiang, "redcliff", &["longzhong_plan"]);
    zhuge.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&zhuge, "redcliff_wind"));
    assert_eq!(awards(&zhuge, "longzhong_strategy_debt"), 0);
    let mut sun = at(Hero::SunShangxiang, "yiling", &["jiangdong_covenant"]);
    sun.apply(Action::Choose { index: 1 }).unwrap();
    assert!(has(&sun, "yiling_ashes"));
    assert_eq!(awards(&sun, "jiangdong_guest_debt"), 0);
}
