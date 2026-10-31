//! Three chapters' fixed Mao/Jiajing local witnesses; not full collation or map reachability.
use three_kingdoms_spire_reborn::content::Hero;
use three_kingdoms_spire_reborn::lore::beat;
use three_kingdoms_spire_reborn::{Action, Game, Stage};

fn enter(g: &mut Game, id: &str) {
    g.stage = Stage::Story;
    g.lore_current = Some(id.into());
}
fn count(xs: &[String], flag: &str) -> usize {
    xs.iter().filter(|s| s.as_str() == flag).count()
}

#[test]
fn fixed_local_witnesses_and_reward_contracts() {
    let doc = include_str!("../docs/CHAPTERS_043_045_JIAJING_VARIANTS.md");
    let audit = include_str!("../data/chapter_body_audit.tsv");
    for rev in ["2568006", "2568008", "2568007", "2554819", "2554825",
                "2554826", "2554798", "2554796", "2554749"] {
        assert!(doc.contains(rev), "missing pinned revision {rev}");
    }
    for anchor in ["田横", "马腾、韩遂", "马超、韩遂", "数千", "数万人",
                   "五六万", "三万", "原刻影印", "未验收"] {
        assert!(doc.contains(anchor), "missing local evidence {anchor}");
    }
    assert_eq!(audit.lines().skip(1).filter(|l| !l.is_empty()).count(), 120);
    for ch in 43..=45 {
        let line = audit.lines().find(|l| l.starts_with(&format!("{ch:03}\t")))
            .expect("missing chapter");
        assert!(line.contains("CHAPTERS_043_045_JIAJING_VARIANTS.md"));
        assert!(line.contains("正文通读/未完成异本对校"));
    }
    for (id, chapter, flag, gold, heal, edition) in [
        ("chaisang_envoy", 43, "chaisang_envoy_record", 18, 7, "嘉靖085"),
        ("jiangdong_council", 44, "jiangdong_council_record", 19, 8, "嘉靖088"),
        ("jiangdong_muster", 44, "jiangdong_muster_record", 18, 8, "嘉靖088"),
        ("jiangkou_inquest", 45, "jiangkou_inquest_record", 17, 7, "嘉靖089"),
    ] {
        let b = beat(id).expect("existing playable beat");
        assert_eq!((b.chapter, b.flag, b.reward_gold, b.reward_heal),
                   (chapter, flag, gold, heal));
        assert!(b.premise.contains(edition), "{id} missing version boundary");
    }
}

#[test]
fn zhuge_chaisang_to_council_choice_is_atomic_and_once_only() {
    let mut g = Game::new(Hero::ZhugeLiang, 43045);
    g.lore_flags.push("dangyang_refuge_account".into());
    enter(&mut g, "chaisang_envoy");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g.lore_flags, "chaisang_envoy_record"), 1);
    enter(&mut g, "jiangdong_council");
    g.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&g.lore_flags, "jiangdong_council_record"), 1);
    let mut restored = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut restored, "jiangdong_council");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "jiangdong_council_record"), 1);
    let mut refused = Game::new(Hero::ZhugeLiang, 43046);
    refused.lore_flags.push("chaisang_envoy_record".into());
    enter(&mut refused, "jiangdong_council");
    refused.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&refused.lore_flags, "jiangdong_council_record"), 0);
}

#[test]
fn cao_false_letter_refusal_preserves_old_routes_and_restore() {
    let mut g = Game::new(Hero::CaoCao, 45045);
    g.lore_flags.push("guandu_grain".into());
    enter(&mut g, "jiangkou_inquest");
    let before = g.to_json().unwrap();
    assert!(g.apply(Action::Choose { index: 99 }).is_err());
    assert_eq!(g.to_json().unwrap(), before);
    g.apply(Action::Choose { index: 2 }).unwrap();
    assert_eq!(count(&g.lore_flags, "jiangkou_inquest_record"), 0);
    let mut restored = Game::from_json(&g.to_json().unwrap()).unwrap();
    enter(&mut restored, "liaodong");
    restored.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&restored.lore_flags, "liaodong_letter"), 1);
    let mut yes = Game::new(Hero::CaoCao, 45046);
    yes.lore_flags.push("guandu_grain".into());
    enter(&mut yes, "jiangkou_inquest");
    yes.apply(Action::Choose { index: 1 }).unwrap();
    assert_eq!(count(&yes.lore_flags, "jiangkou_inquest_record"), 1);
}
