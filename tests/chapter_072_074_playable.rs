//! 072–074: source-fixed scenes, atomic choices, exclusive outcomes, V2 and legacy priority.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, BEATS};
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut g = Game::new(hero, 7274);
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
    g.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    g
}
fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(g: &Game, id: &str) -> usize {
    g.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn source_revisions_are_fixed_and_scenes_have_correct_dependencies() {
    for (n, revision, doc) in [
        (72, "2568036", include_str!("../docs/CHAPTER_072_READING.md")),
        (73, "2568035", include_str!("../docs/CHAPTER_073_READING.md")),
        (74, "2568037", include_str!("../docs/CHAPTER_074_READING.md")),
    ] {
        assert!(doc.contains(&format!("oldid={revision}")) && doc.contains("单底本"), "{n}");
        assert!(doc.contains("回末"), "{n}");
    }
    let history = include_str!("../docs/CHAPTERS_072_074_HISTORY_SPOTCHECK.md");
    for revision in ["2583378", "2583383", "2579680"] {
        assert!(history.contains(revision));
    }
    assert!(history.contains("陈寿") && history.contains("裴松之") && history.contains("0/120"));
    for (id, chapter, prerequisite) in [
        ("xiegu_ration_inquest", 72, "hanshui_retreat_record"),
        ("hanzhong_route_review", 72, "dingjun_signal_record"),
        ("hanzhong_king_memorial", 73, "hanzhong_route_record"),
        ("jingxiang_border_warning", 73, "lukou_terms_record"),
        ("fancheng_flood_witness", 74, "jingxiang_border_record"),
        ("pangde_yujin_ledger", 74, "xiegu_ration_record"),
    ] {
        let b = beat(id).unwrap();
        assert_eq!(b.chapter, chapter);
        assert_eq!(b.prerequisite, Some(prerequisite));
    }
}

#[test]
fn cao_retreat_to_flood_reward_is_atomic_exclusive_and_idempotent() {
    let flags = ["guandu_grain", "hanshui_retreat_record"];
    let mut g = scene(Hero::CaoCao, "xiegu_ration_inquest", &flags);
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "pangde_yujin_ledger");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "xiegu_fancheng_command_debt"), 1);
    let saved = g.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    assert_eq!(restored.to_json().unwrap(), saved);
    enter(&mut restored, "pangde_yujin_ledger");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "xiegu_fancheng_command_debt"), 1);
    let mut refusal = scene(Hero::CaoCao, "xiegu_ration_inquest", &flags);
    refusal.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refusal, "pangde_yujin_ledger");
    refusal.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refusal, "xiegu_fancheng_command_debt"), 0);
}

#[test]
fn zhuge_and_guan_cross_act_debts_need_the_full_flag_chain() {
    let mut z = scene(Hero::ZhugeLiang, "hanzhong_route_review", &["redcliff_wind", "dingjun_signal_record"]);
    z.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut z, "hanzhong_king_memorial");
    z.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&z, "hanzhong_king_route_debt"), 1);
    let mut g = scene(Hero::GuanYu, "jingxiang_border_warning", &["anxi_seal", "lukou_terms_record"]);
    g.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut g, "fancheng_flood_witness");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g, "jingzhou_flood_warning_debt"), 1);
    let mut refused = scene(Hero::GuanYu, "jingxiang_border_warning", &["anxi_seal", "lukou_terms_record"]);
    refused.apply(Action::Choose { index: 2 }).unwrap();
    enter(&mut refused, "fancheng_flood_witness");
    refused.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&refused, "jingzhou_flood_warning_debt"), 0);
}

#[test]
fn legacy_act_two_routes_win_and_new_optional_scenes_remain_selectable() {
    for (hero, flags, old, new) in [
        (Hero::CaoCao, vec!["guandu_grain", "hanshui_retreat_record"], "liaodong", "xiegu_ration_inquest"),
        (Hero::ZhugeLiang, vec!["redcliff_wind", "dingjun_signal_record"], "wuzhang", "hanzhong_route_review"),
        (Hero::GuanYu, vec!["anxi_seal", "jingzhou_warning", "lukou_terms_record"], "jingzhou", "jingxiang_border_warning"),
    ] {
        let flags: Vec<String> = flags.into_iter().map(str::to_owned).collect();
        assert_eq!(next_beat(hero, 2, &flags, &[]).map(|b| b.id), Some(old));
        let seen: Vec<String> = BEATS.iter().filter(|b| b.id != new).map(|b| b.id.to_owned()).collect();
        assert_eq!(next_beat(hero, 2, &flags, &seen).map(|b| b.id), Some(new));
    }
}
