// Keep source scenes in small, ordered modules. Concatenation is compile-time,
// so public BEATS remains an immutable slice and saved scene IDs never change.
include!("lore_beats_part1.rs");
include!("lore_beats_part2.rs");
include!("lore_beats_part3.rs");
include!("lore_beats_part4.rs");
include!("lore_beats_part5.rs");
include!("lore_beats_part6.rs");
include!("lore_beats_part7.rs");
include!("lore_beats_part8.rs");
include!("lore_beats_part9.rs");
include!("lore_beats_part10.rs");
include!("lore_beats_part11.rs");
include!("lore_beats_part12.rs");
include!("lore_beats_part13.rs");
include!("lore_beats_part14.rs");

const fn collect_beats<const N: usize>(parts: &[&[Beat]]) -> [Beat; N] {
    let mut result = [parts[0][0]; N];
    let mut position = 0;
    let mut p = 0;
    while p < parts.len() {
        let mut j = 0;
        while j < parts[p].len() {
            result[position] = parts[p][j];
            position += 1;
            j += 1;
        }
        p += 1;
    }
    assert!(position == N);
    result
}
const BEATS_COUNT: usize =
    BEATS_PART_1.len() + BEATS_PART_2.len() + BEATS_PART_3.len() + BEATS_PART_4.len() + BEATS_PART_5.len() + BEATS_PART_6.len() + BEATS_PART_7.len() + BEATS_PART_8.len() + BEATS_PART_9.len() + BEATS_PART_10.len() + BEATS_PART_11.len() + BEATS_PART_12.len() + BEATS_PART_13.len() + BEATS_PART_14.len();
pub const BEATS: &[Beat] = &collect_beats::<BEATS_COUNT>(&[
    BEATS_PART_1, BEATS_PART_2, BEATS_PART_3, BEATS_PART_4, BEATS_PART_5, BEATS_PART_6, BEATS_PART_7, BEATS_PART_8, BEATS_PART_9, BEATS_PART_10, BEATS_PART_11, BEATS_PART_12, BEATS_PART_13, BEATS_PART_14,
]);
