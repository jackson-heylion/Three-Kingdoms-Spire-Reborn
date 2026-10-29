//! P1④: the final chapter is read, chapter 011 is playable, historical outcomes stay fixed.
use three_kingdoms_spire_reborn::{Action, Game, Stage};
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::{beat, next_beat, newly_completed_eggs, BEATS};

fn scene(hero: Hero, id: &str, flags: &[&str]) -> Game {
    let mut game = Game::new(hero, 111200);
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
    game.lore_flags = flags.iter().map(|s| (*s).into()).collect();
    game
}
fn enter(game: &mut Game, id: &str) {
    game.stage = Stage::Story;
    game.lore_current = Some(id.into());
}
fn count(game: &Game, id: &str) -> usize {
    game.easter_unlocked.iter().filter(|s| s.as_str() == id).count()
}

#[test]
fn fixed_final_body_has_full_evidence_but_not_variant_completion() {
    let doc = include_str!("../docs/CHAPTER_120_READING.md");
    assert!(doc.contains("oldid=2568085") && doc.contains("L177–203") && doc.contains("单底本全文通读"));
    let history = include_str!("../docs/CHAPTERS_011_120_HISTORY_SPOTCHECK.md");
    assert!(history.contains("四十三郡") && history.contains("八十三郡") && history.contains("0/120"));
    let lines: Vec<_> = include_str!("../data/chapter_body_audit.tsv").lines().collect();
    assert_eq!(lines.len(), 121);
    for (i, row) in lines.iter().enumerate().skip(1) {
        assert!(row.starts_with(&format!("{i:03}\t")));
        assert!(row.contains("正文通读/未完成异本对校"), "chapter {i}");
    }
    assert!(lines[120].contains("CHAPTER_120_READING.md"));
    assert!(lines[11].contains("beihai_xuzhou_relief_archive"));
    for (chapter, id) in [(11, "beihai_xuzhou_relief_archive"), (11, "xuzhou_yanzhou_dispatch_archive"),
        (11, "yanzhou_taishan_ambush_archive"), (120, "sunhao_lukang_memorial"),
        (120, "wu_stonecity_surrender_roll"), (120, "jin_yanghu_border_ledger"),
        (120, "jin_duyu_campaign_ledger"), (120, "jin_wangjun_chain_ledger"),
        (120, "jin_wu_family_settlement")] {
        assert_eq!(beat(id).unwrap().chapter, chapter);
    }
    let chapters: std::collections::BTreeSet<_> = BEATS.iter().map(|b| b.chapter).collect();
    assert_eq!(chapters.len(), 120, "all chapters now have source scenes");
}

#[test]
fn refused_choices_are_atomic_and_do_not_award_unearned_debts() {
    let mut game = scene(Hero::SunShangxiang, "sunhao_lukang_memorial",
        &["wu_relief_arrival_dispute_record"]);
    let before = game.to_json().unwrap();
    assert!(game.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(game.to_json().unwrap(), before);
    game.apply(Action::Choose { index: 1 }).unwrap();
    enter(&mut game, "wu_stonecity_surrender_roll");
    game.apply(Action::Choose { index: 2 }).unwrap();
    assert!(!game.lore_flags.iter().any(|s| s == "wu_stonecity_surrender_record"));
    assert_eq!(count(&game, "wu_sunhao_stonecity_debt"), 0);
    let saved = game.to_json().unwrap();
    assert_eq!(Game::from_json(&saved).unwrap().to_json().unwrap(), saved);
}

#[test]
fn wei_wu_and_shu_epilogue_rewards_are_one_time_after_v2_restore() {
    let mut wei = scene(Hero::CaoCao, "jin_yanghu_border_ledger", &["wei_jin_abdication_record"]);
    for id in ["jin_yanghu_border_ledger", "jin_duyu_campaign_ledger", "jin_wangjun_chain_ledger"] {
        enter(&mut wei, id);
        wei.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&wei, "jin_yanghu_wangjun_debt"), 1);
    let saved = wei.to_json().unwrap();
    let mut restored = Game::from_json(&saved).unwrap();
    enter(&mut restored, "jin_wangjun_chain_ledger");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored, "jin_yanghu_wangjun_debt"), 1);
    let mut wu = scene(Hero::SunShangxiang, "sunhao_lukang_memorial",
        &["wu_relief_arrival_dispute_record"]);
    for id in ["sunhao_lukang_memorial", "wu_stonecity_surrender_roll"] {
        enter(&mut wu, id);
        wu.apply(Action::Choose { index: 1 }).unwrap();
    }
    assert_eq!(count(&wu, "wu_sunhao_stonecity_debt"), 1);
    let mut shu = scene(Hero::ZhugeLiang, "jin_wu_family_settlement",
        &["jiangwei_last_courier_record"]);
    shu.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&shu, "jin_wu_civilian_debt"), 1);
}

#[test]
fn chapter_011_choices_require_real_accepted_flags_and_legacy_entries_survive() {
    for (hero, id, old, flag, reward) in [
        (Hero::GuanYu, "beihai_xuzhou_relief_archive", "taoyuan_oath",
            "xuzhou_gate_account", "beihai_xuzhou_civil_debt"),
        (Hero::CaoCao, "xuzhou_yanzhou_dispatch_archive", "xuzhou_debt",
            "puyang_supply", "xuzhou_yanzhou_ration_debt"),
        (Hero::LuBu, "yanzhou_taishan_ambush_archive", "changan_account",
            "xiaopei_refuge", "changan_yanzhou_xiaopei_debt"),
    ] {
        let mut game = scene(hero, id, &[old, flag]);
        assert_eq!(count(&game, reward), 0);
        game.apply(Action::Choose { index: 1 }).unwrap();
        assert_eq!(count(&game, reward), 1);
        let only_two: Vec<String> = [old, flag].iter().map(|s| (*s).into()).collect();
        assert!(!newly_completed_eggs(hero, &only_two, &[]).iter().any(|egg| egg.id == reward));
    }
    assert_eq!(next_beat(Hero::GuanYu, 0, &["taoyuan_oath".into()],
        &["taoyuan".into()]).map(|b| b.id), Some("anxi"));
    assert_eq!(next_beat(Hero::CaoCao, 0, &["xuzhou_debt".into()],
        &["sevenstar".into(), "xuzhou".into()]).map(|b| b.id), Some("puyang"));
    assert_eq!(next_beat(Hero::CaoCao, 0, &[],
        &["sevenstar".into(), "xuzhou".into()]).map(|b| b.id), Some("xudu"));
    assert_eq!(next_beat(Hero::LuBu, 0, &["changan_account".into()],
        &["redhare".into(), "pavilion".into(), "changan".into()]).map(|b| b.id), Some("xiaopei"));
    for (hero, act, id, required) in [
        (Hero::GuanYu, 0, "beihai_xuzhou_relief_archive", "taoyuan_oath"),
        (Hero::CaoCao, 0, "xuzhou_yanzhou_dispatch_archive", "xuzhou_debt"),
        (Hero::LuBu, 0, "yanzhou_taishan_ambush_archive", "changan_account"),
        (Hero::CaoCao, 2, "jin_yanghu_border_ledger", "wei_jin_abdication_record"),
    ] {
        let seen: Vec<String> = BEATS.iter().filter(|b| b.id != id).map(|b| b.id.into()).collect();
        assert_eq!(next_beat(hero, act, &[required.into()], &seen).map(|b| b.id), Some(id));
    }
    assert_eq!(next_beat(Hero::ZhugeLiang, 2,
        &["redcliff_wind".into(), "jiangwei_last_courier_record".into()], &[])
        .map(|b| b.id), Some("wuzhang"));
    assert_eq!(next_beat(Hero::CaoCao, 2,
        &["guandu_grain".into(), "wei_jin_abdication_record".into()], &[])
        .map(|b| b.id), Some("liaodong"));
    assert_eq!(next_beat(Hero::SunShangxiang, 2,
        &["jiangdong_covenant".into(), "wu_relief_arrival_dispute_record".into()], &[])
        .map(|b| b.id), Some("yiling"));
}
