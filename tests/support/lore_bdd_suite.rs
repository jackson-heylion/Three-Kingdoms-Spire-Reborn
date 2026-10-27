//! Execute the literary Gherkin files through the same strict BDD interpreter.
use super::run_feature;

#[test]
fn every_literary_feature_is_executed_not_just_stored() {
    let documents: &[(&str, &str)] = &[
        ("039_041", include_str!("../../features/lore_ch039_041.feature")),
        ("042_044", include_str!("../../features/lore_ch042_044.feature")),
        ("045_047", include_str!("../../features/lore_ch045_047.feature")),
        ("048_050", include_str!("../../features/lore_ch048_050.feature")),
        ("051_053", include_str!("../../features/lore_ch051_053.feature")),
        ("054_056", include_str!("../../features/lore_ch054_056.feature")),
        ("057_059", include_str!("../../features/lore_ch057_059.feature")),
        ("060_062", include_str!("../../features/lore_ch060_062.feature")),
        ("063_065", include_str!("../../features/lore_ch063_065.feature")),
        ("066_068", include_str!("../../features/lore_ch066_068.feature")),
        ("069_071", include_str!("../../features/lore_ch069_071.feature")),
        ("072_074", include_str!("../../features/lore_ch072_074.feature")),
        ("075_077", include_str!("../../features/lore_ch075_077.feature")),
        ("078_080", include_str!("../../features/lore_ch078_080.feature")),
        ("081_083", include_str!("../../features/lore_ch081_083.feature")),
        ("084_086", include_str!("../../features/lore_ch084_086.feature")),
        ("087_089", include_str!("../../features/lore_ch087_089.feature")),
        ("090_092", include_str!("../../features/lore_ch090_092.feature")),
        ("093_095", include_str!("../../features/lore_ch093_095.feature")),
        ("096_098", include_str!("../../features/lore_ch096_098.feature")),
        ("099_101", include_str!("../../features/lore_ch099_101.feature")),
        ("102_104", include_str!("../../features/lore_ch102_104.feature")),
        ("105_107", include_str!("../../features/lore_ch105_107.feature")),
        ("108_110", include_str!("../../features/lore_ch108_110.feature")),
        ("111_113", include_str!("../../features/lore_ch111_113.feature")),
        ("114_116", include_str!("../../features/lore_ch114_116.feature")),
    ];
    let mut scenarios = 0;
    for (name, source) in documents {
        scenarios += run_feature(source, name).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    assert!(scenarios >= 100, "literary BDD coverage regressed: {scenarios}");
}
