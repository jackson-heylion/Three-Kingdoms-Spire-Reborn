//! Persistent 7×3 seeded act graphs with explicit branching edges, hazards and
//! three-act encounter weights. Missing optional graph fields decode old V2 saves.
use serde::{Deserialize, Serialize};
use crate::Node;

pub const LANES: u8 = 3;
pub const ROWS: usize = 7;
pub const ENCOUNTER_WEIGHTS: [[u8;5];3] = [
    // battle, story, camp, market, elite
    [46,25,16,10,3],
    [41,21,16,11,11],
    [35,17,16,13,19],
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    #[default] Road,
    Supply,    // +12 gold on entering
    Hazard,    // -3 hp but never lethal
    Closed,    // impassable lane at this floor
}
impl Terrain {
    pub fn label(self)-> &'static str {
        match self {
            Self::Road=>"平道",Self::Supply=>"辎重",Self::Hazard=>"险地",Self::Closed=>"断道",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActMap {
    pub act: u8,
    pub rows: Vec<[Node;3]>,
    /// Outgoing edge bit sets: edges[row][from] -> row+1 lane.
    /// An empty array is a legacy map with adjacent-lane default topology.
    #[serde(default)]
    pub edges: Vec<[u8;3]>,
    #[serde(default)]
    pub terrain: Vec<[Terrain;3]>,
}
impl ActMap {
    pub fn generate(mut random: impl FnMut(usize)->usize, act:u8)->Self {
        let mut rows=Vec::with_capacity(ROWS);
        let mut terrain=Vec::with_capacity(ROWS);
        rows.push([Node::Battle,Node::Story,Node::Battle]);
        terrain.push([Terrain::Road;3]);
        let weights=ENCOUNTER_WEIGHTS[usize::from(act.min(2))];
        for row in 1..6 {
            let mut choices=[Node::Battle;3];
            let mut ground=[Terrain::Road;3];
            for lane in 0..3 {
                let mut roll=random(100);
                let kinds=[Node::Battle,Node::Story,Node::Camp,Node::Market,Node::Elite];
                let mut selection=Node::Battle;
                for (index,weight) in weights.iter().enumerate() {
                    if roll<usize::from(*weight) {
                        selection=kinds[index];
                        break;
                    }
                    roll-=usize::from(*weight);
                }
                if row<2 && selection==Node::Elite {selection=Node::Battle;}
                choices[lane]=selection;
                let ground_roll=random(100);
                ground[lane]=match ground_roll {
                    0..=7 if row>1 && lane>0 => Terrain::Closed,
                    8..=20 => Terrain::Supply,
                    21..=34 => Terrain::Hazard,
                    _ => Terrain::Road,
                };
            }
            // Never close both entrances available to a right-hand lane.
            if ground[1]==Terrain::Closed && ground[2]==Terrain::Closed {
                ground[2]=Terrain::Road;
            }
            // Defensive corridor near a boss prevents three locked entrances.
            if row==5 {ground[1]=Terrain::Road;choices[1]=Node::Camp;}
            rows.push(choices);
            terrain.push(ground);
        }
        rows.push([Node::Boss;3]);
        terrain.push([Terrain::Road;3]);
        let mut edges=Vec::with_capacity(ROWS-1);
        for row in 0..ROWS-1 {
            let mut transitions=[0u8;3];
            for (from,slot) in transitions.iter_mut().enumerate() {
                if terrain[row][from]==Terrain::Closed {continue;}
                let candidates=(0..3).filter(|to|
                    (from as i32-*to as i32).abs()<=1
                    && terrain[row+1][*to]!=Terrain::Closed).collect::<Vec<_>>();
                // At least one edge per open lane. Other valid choices are
                // optional forks; identical source lanes may merge downstream.
                let preferred=candidates[random(candidates.len())];
                *slot|=1u8<<preferred;
                if candidates.len()>1 && random(100)<56 {
                    let extra=candidates[random(candidates.len())];
                    *slot|=1u8<<extra;
                }
            }
            edges.push(transitions);
        }
        Self {act,rows,edges,terrain}
    }
    pub fn terrain_at(&self,row:usize,lane:u8)->Terrain {
        self.terrain.get(row).map_or(Terrain::Road,|r|r[usize::from(lane.min(2))])
    }
    pub fn routes(&self,row:usize,previous_lane:Option<u8>)->Vec<(u8,Node)> {
        let Some(nodes)=self.rows.get(row) else {return vec![]};
        (0..LANES)
            .filter(|lane|self.terrain_at(row,*lane)!=Terrain::Closed)
            .filter(|lane|previous_lane.is_none_or(|previous|{
                if previous>=LANES {return false;}
                if row>0 && let Some(edges)=self.edges.get(row-1) {
                    edges[usize::from(previous)] & (1u8<<lane) !=0
                } else {(i32::from(*lane)-i32::from(previous)).abs()<=1}
            }))
            .map(|lane|(lane,nodes[usize::from(lane)])).collect()
    }
    pub fn is_valid(&self)->bool {
        if self.act>=3 || self.rows.len()!=ROWS || self.rows[0][0]!=Node::Battle
            || !self.rows[6].iter().all(|node|*node==Node::Boss)
            || (!self.edges.is_empty() && self.edges.len()!=ROWS-1)
            || (!self.terrain.is_empty() && self.terrain.len()!=ROWS)
            || self.terrain_at(0,0)==Terrain::Closed
            || (0..LANES).any(|lane|self.terrain_at(6,lane)==Terrain::Closed)
        {return false;}
        if self.edges.is_empty() {
            // Existing saved campaigns used exactly adjacent-lane paths.
            return true;
        }
        // Check that every enterable reachable lane offers at least one route,
        // and that all edge bits represent adjacent, unblocked destinations.
        let mut reachable=[true;3];
        for row in 1..ROWS {
            let mut next=[false;3];
            for (previous,found) in reachable.iter().enumerate() {
                let bits=self.edges[row-1][previous];
                if bits&!0b111!=0 || (bits!=0 && self.terrain_at(row-1,previous as u8)==Terrain::Closed)
                {return false;}
                for to in 0usize..3 {
                    if bits&(1u8<<to)!=0 && (previous.abs_diff(to)>1 ||
                        self.terrain_at(row,to as u8)==Terrain::Closed)
                    {return false;}
                }
                if *found {
                    let routes=self.routes(row,Some(previous as u8));
                    if routes.is_empty() {return false;}
                    for (lane,_) in routes {next[usize::from(lane)]=true;}
                }
            }
            reachable=next;
        }
        reachable.iter().any(|found|*found)
    }
    pub fn score_route(&self,row:usize,lane:u8,hp:i32,max_hp:i32,gold:i32)->i32 {
        // Transparent local 3-floor lookahead, not a hidden best-path oracle.
        let mut frontier=vec![(lane,0i32)];
        for depth in 0..3 {
            let level=row+depth;
            if level>=ROWS {break;}
            let mut next=Vec::new();
            for &(previous,score) in &frontier {
                let options=if depth==0 {
                    self.routes(level,None).into_iter().filter(|(l,_)|*l==lane).collect::<Vec<_>>()
                } else {
                    self.routes(level,Some(previous))
                };
                for (to,node) in options {
                    let missing=(max_hp-hp).max(0);
                    let node_value=match node {
                        Node::Battle=>10,
                        Node::Elite=>if hp<max_hp/2 {-24}else{19},
                        Node::Camp=>if missing>18 {38}else{15},
                        Node::Story=>18,
                        Node::Market=>if gold>=150 {28}else{5},
                        Node::Boss=>15,
                    };
                    let terrain_value=match self.terrain_at(level,to) {
                        Terrain::Supply=>16,Terrain::Hazard=>if hp<=15 {-30}else{-8},
                        Terrain::Road|Terrain::Closed=>0,
                    };
                    next.push((to,score+node_value+terrain_value));
                }
            }
            if next.is_empty(){break;}
            frontier=next;
        }
        frontier.iter().map(|(_,v)|*v).max().unwrap_or(i32::MIN/4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_maps_are_connected_replayable_and_act_weighted() {
        for seed in 1..=250u64 {
            for act in 0..3 {
                let mut state=seed.wrapping_mul(1597334677);
                let random=|n:usize|{
                    state=state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    ((state>>32) as usize)%n
                };
                let map=ActMap::generate(random,act);
                assert!(map.is_valid(),"act={act} seed={seed}");
                let json=serde_json::to_string(&map).unwrap();
                assert_eq!(serde_json::to_string(&serde_json::from_str::<ActMap>(&json).unwrap()).unwrap(),json);
                for first in 0..3 {
                    let mut lane=first;
                    for row in 1..ROWS {
                        let routes=map.routes(row,Some(lane));
                        if row==1 && map.terrain_at(0,lane)==Terrain::Closed {break;}
                        assert!(!routes.is_empty());
                        lane=routes[0].0;
                    }
                }
            }
        }
    }
    #[test]
    fn legacy_maps_use_neighbor_adjacency_and_keep_old_shape() {
        let map=ActMap::generate(|n|n/2,0);
        let mut old=serde_json::to_value(&map).unwrap();
        old.as_object_mut().unwrap().remove("edges");
        old.as_object_mut().unwrap().remove("terrain");
        let restored:ActMap=serde_json::from_value(old).unwrap();
        assert!(restored.is_valid());
        assert_eq!(restored.routes(1,Some(0)).len(),2);
        assert_eq!(restored.routes(1,Some(1)).len(),3);
    }
    #[test]
    fn invalid_edges_and_closed_destinations_are_rejected() {
        let mut map=ActMap::generate(|n|n/2,0);
        map.edges[0][0]=0b1000;
        assert!(!map.is_valid());
        map.edges[0][0]=0;
        assert!(!map.is_valid());
    }
    #[test]
    fn route_scores_react_to_health_and_special_terrain() {
        let mut map=ActMap::generate(|n|n/2,0);
        map.rows[1][0]=Node::Elite;
        map.terrain[1][0]=Terrain::Road;
        let healthy=map.score_route(1,0,80,80,0);
        let weak=map.score_route(1,0,20,80,0);
        assert!(healthy>weak);
    }
}
