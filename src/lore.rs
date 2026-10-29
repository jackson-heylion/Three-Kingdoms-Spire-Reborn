//! Three Kingdoms narrative adaptation, based on the 120-chapter Mao edition.
//! The chapter references are literary provenance, not a claim of historical accuracy.
//! Story text and rewards are original game writing, not verbatim source passages.
use crate::content::Hero;

#[derive(Clone, Copy, Debug)]
pub struct Beat {
    pub id: &'static str,
    pub title: &'static str,
    pub chapter: u8,
    pub act: u8,
    pub hero: Option<Hero>,
    pub prerequisite: Option<&'static str>,
    pub flag: &'static str,
    pub character: &'static str,
    pub premise: &'static str,
    pub vow: &'static str,
    pub bargain: &'static str,
    pub reward_card: &'static str,
    pub reward_gold: i32,
    pub reward_heal: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Sworn,
    Family,
    Mentor,
    Friend,
    Rival,
    Patron,
    Betrayal,
    Alliance,
    Inherited,
}

impl Relation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Sworn => "结义", Self::Family => "宗亲", Self::Mentor => "师徒",
            Self::Friend => "知己", Self::Rival => "宿敌", Self::Patron => "知遇",
            Self::Betrayal => "背盟", Self::Alliance => "联姻/同盟", Self::Inherited => "传承",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bond {
    pub protagonist: Hero,
    pub companion: &'static str,
    pub relation: Relation,
    pub chapter: u8,
    pub flag: &'static str,
    pub note: &'static str,
}

include!("lore_bonds.rs");

include!("lore_beats.rs");

#[derive(Clone, Copy, Debug)]
pub struct EasterEgg {
    pub id: &'static str,
    pub name: &'static str,
    pub hero: Option<Hero>,
    pub chapters: &'static [u8],
    pub flags: &'static [&'static str],
    pub reward_gold: i32,
    pub reward_heal: i32,
    pub hint: &'static str,
}

/// Hidden conclusions are only awarded after the *entire* flag chain.
/// Alternate monetary choices never plant a prerequisite, so they cannot
/// claim the same narrative ending accidentally.
const LEGACY_EGGS: &[EasterEgg] = include!("lore_eggs.rs");
const NEW_EGGS: &[EasterEgg] = include!("lore_eggs_part2.rs");
const CHAPTER_054_056_EGGS: &[EasterEgg] = include!("lore_eggs_part3.rs");
const CHAPTER_057_059_EGGS: &[EasterEgg] = include!("lore_eggs_part4.rs");
const CHAPTER_060_062_EGGS: &[EasterEgg] = include!("lore_eggs_part5.rs");
const CHAPTER_063_065_EGGS: &[EasterEgg] = include!("lore_eggs_part6.rs");
const CHAPTER_066_068_EGGS: &[EasterEgg] = include!("lore_eggs_part7.rs");
const CHAPTER_069_071_EGGS: &[EasterEgg] = include!("lore_eggs_part8.rs");
const CHAPTER_072_074_EGGS: &[EasterEgg] = include!("lore_eggs_part9.rs");
const CHAPTER_075_077_EGGS: &[EasterEgg] = include!("lore_eggs_part10.rs");
const CHAPTER_078_080_EGGS: &[EasterEgg] = include!("lore_eggs_part11.rs");
const CHAPTER_081_083_EGGS: &[EasterEgg] = include!("lore_eggs_part12.rs");
const CHAPTER_090_092_EGGS: &[EasterEgg] = include!("lore_eggs_part13.rs");
const CHAPTER_093_095_EGGS: &[EasterEgg] = include!("lore_eggs_part14.rs");
const CHAPTER_096_098_EGGS: &[EasterEgg] = include!("lore_eggs_part15.rs");
const CHAPTER_099_101_EGGS: &[EasterEgg] = include!("lore_eggs_part16.rs");
const CHAPTER_102_104_EGGS: &[EasterEgg] = include!("lore_eggs_part17.rs");
const CHAPTER_105_107_EGGS: &[EasterEgg] = include!("lore_eggs_part18.rs");
const CHAPTER_108_110_EGGS: &[EasterEgg] = include!("lore_eggs_part19.rs");
const CHAPTER_111_113_EGGS: &[EasterEgg] = include!("lore_eggs_part20.rs");
const CHAPTER_114_116_EGGS: &[EasterEgg] = include!("lore_eggs_part21.rs");
const CHAPTER_117_119_EGGS: &[EasterEgg] = include!("lore_eggs_part22.rs");
const CHAPTER_011_120_EGGS: &[EasterEgg] = include!("lore_eggs_part23.rs");
const CHAPTER_004_006_EGGS: &[EasterEgg] = include!("lore_eggs_part004_006.rs");
const fn collect_eggs<const N: usize>(parts: &[&[EasterEgg]]) -> [EasterEgg; N] {
    let mut out = [parts[0][0]; N];
    let mut pos = 0;
    let mut i = 0;
    while i < parts.len() {
        let mut j = 0;
        while j < parts[i].len() {
            out[pos] = parts[i][j];
            pos += 1;
            j += 1;
        }
        i += 1;
    }
    assert!(pos == N);
    out
}
pub const EASTER_EGGS: &[EasterEgg] =
    &collect_eggs::<{ LEGACY_EGGS.len() + NEW_EGGS.len() + CHAPTER_054_056_EGGS.len() + CHAPTER_057_059_EGGS.len() + CHAPTER_060_062_EGGS.len() + CHAPTER_063_065_EGGS.len() + CHAPTER_066_068_EGGS.len() + CHAPTER_069_071_EGGS.len() + CHAPTER_072_074_EGGS.len() + CHAPTER_075_077_EGGS.len() + CHAPTER_078_080_EGGS.len() + CHAPTER_081_083_EGGS.len() + CHAPTER_090_092_EGGS.len() + CHAPTER_093_095_EGGS.len() + CHAPTER_096_098_EGGS.len() + CHAPTER_099_101_EGGS.len() + CHAPTER_102_104_EGGS.len() + CHAPTER_105_107_EGGS.len() + CHAPTER_108_110_EGGS.len() + CHAPTER_111_113_EGGS.len() + CHAPTER_114_116_EGGS.len() + CHAPTER_004_006_EGGS.len() + CHAPTER_117_119_EGGS.len() + CHAPTER_011_120_EGGS.len() }>(&[LEGACY_EGGS, NEW_EGGS, CHAPTER_054_056_EGGS, CHAPTER_057_059_EGGS, CHAPTER_060_062_EGGS, CHAPTER_063_065_EGGS, CHAPTER_066_068_EGGS, CHAPTER_069_071_EGGS, CHAPTER_072_074_EGGS, CHAPTER_075_077_EGGS, CHAPTER_078_080_EGGS, CHAPTER_081_083_EGGS, CHAPTER_090_092_EGGS, CHAPTER_093_095_EGGS, CHAPTER_096_098_EGGS, CHAPTER_099_101_EGGS, CHAPTER_102_104_EGGS, CHAPTER_105_107_EGGS, CHAPTER_108_110_EGGS, CHAPTER_111_113_EGGS, CHAPTER_114_116_EGGS, CHAPTER_004_006_EGGS, CHAPTER_117_119_EGGS, CHAPTER_011_120_EGGS]);

pub fn newly_completed_eggs(
    hero: Hero, flags: &[String], already: &[String]
) -> Vec<&'static EasterEgg> {
    EASTER_EGGS.iter().filter(|e|
        e.hero.is_none_or(|owner| owner == hero)
            && !already.iter().any(|unlocked| unlocked == e.id)
            && e.flags.iter().all(|flag| flags.iter().any(|known| known == flag))
    ).collect()
}

pub fn beat(id: &str) -> Option<&'static Beat> { BEATS.iter().find(|b| b.id == id) }

include!("lore_selection.rs");

pub fn bonds_for(hero: Hero, flags: &[String]) -> Vec<&'static Bond> {
    BONDS.iter().filter(|b| b.protagonist == hero && flags.iter().any(|s| s == b.flag)).collect()
}

pub fn provenance_url(chapter: u8) -> String {
    format!("https://zh.wikisource.org/wiki/三國演義/第{chapter:03}回")
}

#[cfg(test)]
include!("lore_tests.rs");
