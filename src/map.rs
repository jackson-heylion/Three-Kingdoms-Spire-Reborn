//! Seven-row, three-lane connected encounter graph for each Three Kingdoms act.
//! Generated once from the game's deterministic RNG and persisted with the save.
use serde::{Deserialize, Serialize};
use crate::Node;

pub const LANES: u8 = 3;
pub const ROWS: usize = 7;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActMap {
    pub act: u8,
    pub rows: Vec<[Node; 3]>,
}

impl ActMap {
    pub fn generate(mut random: impl FnMut(usize) -> usize, act: u8) -> Self {
        let mut rows = Vec::with_capacity(ROWS);
        rows.push([Node::Battle, Node::Story, Node::Battle]);
        for row in 1..6 {
            let mut choices = [Node::Battle; 3];
            for node in &mut choices {
                let roll = random(100);
                *node = if row >= 2 && roll < 13 {
                    Node::Elite
                } else if roll < 25 {
                    Node::Camp
                } else if roll < 38 {
                    Node::Market
                } else if roll < 62 {
                    Node::Story
                } else {
                    Node::Battle
                };
            }
            if row == 2 { choices[2] = Node::Elite; }
            if row == 5 { choices[1] = Node::Camp; }
            rows.push(choices);
        }
        rows.push([Node::Boss; 3]);
        Self { act, rows }
    }

    pub fn routes(&self, row: usize, previous_lane: Option<u8>) -> Vec<(u8,Node)> {
        let Some(nodes) = self.rows.get(row) else { return Vec::new(); };
        (0..LANES).filter(|lane| previous_lane.is_none_or(|prior|
            (i32::from(*lane) - i32::from(prior)).abs() <= 1))
            .map(|lane| (lane,nodes[usize::from(lane)])).collect()
    }

    pub fn is_valid(&self) -> bool {
        self.act < 3 && self.rows.len() == ROWS
            && self.rows[0][0] == Node::Battle
            && self.rows[6].iter().all(|n| *n == Node::Boss)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_generated_act_has_reachable_boss_routes() {
        let mut value = 0_usize;
        for act in 0..3 {
            let map = ActMap::generate(|bound| {
                value = value.wrapping_mul(1664525).wrapping_add(1013904223);
                value % bound
            }, act);
            assert!(map.is_valid());
            for first in 0..LANES {
                let mut previous = Some(first);
                for row in 1..ROWS {
                    let options = map.routes(row,previous);
                    assert!(!options.is_empty());
                    previous = Some(options[0].0);
                }
                assert!(map.routes(6,previous).iter().all(|(_,n)| *n==Node::Boss));
            }
        }
    }

    #[test]
    fn graph_only_exposes_adjacent_lanes() {
        let map = ActMap::generate(|_| 45,0);
        assert_eq!(map.routes(1,Some(0)).len(),2);
        assert_eq!(map.routes(1,Some(1)).len(),3);
        assert_eq!(map.routes(1,Some(2)).len(),2);
        assert_eq!(map.routes(0,None).len(),3);
    }
}
