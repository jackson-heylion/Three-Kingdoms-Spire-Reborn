// Deterministic scene ordering; extracted from lore.rs to keep modules small.
// A later optional retrospective must not steal the original Jingzhou entry
// from an old save that has already earned the Anxi responsibility flag.
/// For deterministic scenario selection: favor a hero-specific prerequisite payoff,
/// then the hero's introductory chapter, finally a wider shared historic clue.
/// It consumes no randomness; each encounter is used at most once per run.
pub fn next_beat(hero: Hero, act: u8, flags: &[String], seen: &[String]) -> Option<&'static Beat> {
    let eligible = |b: &&Beat| {
        b.act == act && !seen.iter().any(|id| id == b.id)
        && b.prerequisite.is_none_or(|key| flags.iter().any(|flag| flag == key))
        && b.hero.is_none_or(|required| required == hero)
    };
    BEATS.iter().filter(eligible).min_by_key(|b| {
        let is_hero = b.hero == Some(hero);
        let is_payoff = b.prerequisite.is_some();
        let flag = |f: &str| flags.iter().any(|x| x == f);
        let unseen = |id: &str| !seen.iter().any(|x| x == id);
        let protect_old_jingzhou = b.id == "hanjin_ferry"
            && flag("anxi_seal") && unseen("jingzhou");
        let protect_legacy = protect_old_jingzhou
            || (b.id == "changsha_mercy" && flag("anxi_seal") && unseen("jingzhou"))
            || (b.id == "nanjun_claim_ledger" && flag("redcliff_wind") && unseen("wuzhang"))
            || (b.id == "hefei_memorial" && flag("jiangdong_covenant") && unseen("yiling"))
            || (b.id == "nanxu_escape_witness" && flag("jiangdong_covenant") && unseen("yiling"))
            || ((b.id == "jingzhou_pledge_copy" || b.id == "jingzhou_false_passage")
                && flag("redcliff_wind") && unseen("wuzhang"))
            || (b.id == "tongque_dispatch_ledger" && flag("guandu_grain") && unseen("liaodong"))
            || (matches!(b.id, "mateng_case" | "tongguan_retreat" | "weinan_division" | "zhangsong_audience")
                && flag("guandu_grain") && unseen("liaodong"))
            || (b.id == "river_child_witness" && flag("jiangdong_covenant") && unseen("yiling"))
            || (matches!(b.id, "yizhou_map_audit" | "fu_pass_review")
                && flag("redcliff_wind") && unseen("wuzhang"));
        (if protect_legacy { 4 } else if is_hero && is_payoff { 0 } else if is_hero { 1 } else if is_payoff { 2 } else { 3 }, b.chapter)
    })
}


#[cfg(test)]
mod compatibility_tests {
    use super::*;
    #[test]
    fn optional_hanjin_never_forces_old_anxi_saves_past_jingzhou() {
        let flags = vec!["anxi_seal".into(), "xutian_guarded_emperor".into()];
        assert_eq!(next_beat(Hero::GuanYu, 2, &flags, &[]).map(|b| b.id), Some("jingzhou"));
        assert_eq!(next_beat(Hero::GuanYu, 2, &flags, &["jingzhou".into()]).map(|b| b.id), Some("hanjin_ferry"));
        assert_eq!(next_beat(Hero::GuanYu, 2, &["xutian_guarded_emperor".into()], &[]).map(|b| b.id), Some("hanjin_ferry"));
    }
}
