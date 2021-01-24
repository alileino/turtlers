
use std::{cmp::min, collections::{HashMap}};
use crate::{turtle_action::{TurtleAction, gps}, vec3::*};
use crate::{turtle_rotation::*};
use crate::{turtle_state::*};
use anyhow::{anyhow, Result};

#[derive(PartialEq, Hash, Debug, Eq)]
struct Node {
    loc: Vec3<i32>,
    dir: AxisDirection
}




// fn generate_neighbours_at(start_node: &Node, depth: usize) -> Vec<(Node, TurtleAction)> {
//     let mut next_open = vec![(start_node, )];
//     for i in 0..depth {
//         let mut result = vec![];
//         for node in next_open {
//             let turn_left = node.dir.rotate_left();
//             let turn_right = node.dir.rotate_right();
//             result.push(
//                 (Node{loc: node.loc.clone(), dir: turn_left},
//                 TurtleAction::Turn{direction:RelativeDirection::Left})
//             );
//             result.push(
//                 (Node{loc: node.loc.clone(), dir: turn_right},
//                 TurtleAction::Turn{direction:RelativeDirection::Right})
//             );
//
//             for rel_dir in &[RelativeDirection::Forward, RelativeDirection::Backward, RelativeDirection::Up, RelativeDirection::Down] {
//                 let loc_dir = get_dest_axisdirection(&node.dir, rel_dir);
//                 let loc = &node.loc + &loc_dir;
//                 result.push(
//                     (Node{loc:loc, dir:node.dir.clone()},
//                     TurtleAction::Move{direction:rel_dir.to_owned()})
//                 );
//             }
//         }
//         next_open = result;
//     }
//
//     next_open[0]
// }

fn generate_neighbors(node: &Node) -> Vec<(Node, TurtleAction)> {
    
    let mut result = vec![];
    let turn_left = node.dir.rotate_left();
    let turn_right = node.dir.rotate_right();
    result.push(
        (Node{loc: node.loc.clone(), dir: turn_left}, 
        TurtleAction::Turn{direction:RelativeDirection::Left})
    );
    result.push(
        (Node{loc: node.loc.clone(), dir: turn_right},
        TurtleAction::Turn{direction:RelativeDirection::Right})
    );
    for rel_dir in &[RelativeDirection::Forward, RelativeDirection::Backward, RelativeDirection::Up, RelativeDirection::Down] {
        let loc_dir = get_dest_axisdirection(&node.dir, rel_dir);
        let loc = &node.loc + &loc_dir;
        result.push(
            (Node{loc:loc, dir:node.dir.clone()},
            TurtleAction::Move{direction:rel_dir.to_owned()})
        );
    }

    result
}

// fn dist_heuristic(state: &WorldState, start: &Node, end: &Node, can_dig: bool, cur_cost: f64) -> f64 {
//     let mut current = start.clone();
//     let mut rotation_needed = Rotation::find_rotation(&start.loc, &end.loc);
//     let mut distance_needed = &end.loc-&start.loc;
//     let mut cost = 0f64;


//     // Heuristic takes first rotations which reward, and calculates cost according to:
//     // Turning = 1
//     // Air=1 (have to take a step)
//     // Block=2 if can_dig (have to dig and move = two steps), otherwise inf.
    // let neighbors = generate_neighbors(&current);
    // for neighbor in &neighbors {
    //     let new_dist = &end.loc-&neighbor.loc;

    // }

    // Heuristic takes first rotations which reward, and calculates cost according to:
    // Turning = 1
    // Air=1 (have to take a step)
    // Unknown=1 (have to be optimistic due to consistency)
    // Block=2 if can_dig (have to dig and move = two steps), otherwise inf.

// }

fn dist_heuristic(state: &WorldState, start: &Node, end: &Node, can_dig: bool, cur_cost: u64) -> u64 {
    let current = start.clone();
    let block = state.state.get(&current.loc).unwrap_or( &Block::Unknown);
    if !can_dig && block == &Block::Block {
         return 999999;
    }
    // let mut rotation_needed = Rotation::find_rotation(&
    let rotation_needed = AxisDirection::dot(&start.dir, &end.dir);
    let distance_needed = &end.loc-&start.loc;
    let rotation_cost = match rotation_needed {
        Rotation::Y0 => {

            // if straight ahead, (with Y offset allowed)
            if distance_needed.0 == 0 || distance_needed.2 == 0
            {
                let d = Vec3(min(distance_needed.0.abs(), 1), 0, min(distance_needed.2.abs(), 1));
                
                if d == Vec3::zero() ||  d == end.dir.to_unit_vector() || d == -end.dir.to_unit_vector() {
                    println!("0-cost Y0, goal at: {:?} with distance remaining {:?} at direction {:?} end_dir: {:?}", end.dir, distance_needed, d, end.dir.to_unit_vector());
                    0
                } else {
                    println!("NON0-cost Y0,  goal at: {:?} with distance remaining {:?} at direction {:?} end_dir: {:?}", end.dir, distance_needed, d, end.dir.to_unit_vector());
                    2
                }
            } else {
                println!("NON0-cost Y0 goal at: {:?} with distance remaining {:?} ", end.dir, distance_needed);
                2 // we need to turn at least twice more 
            }
            
        },
        Rotation::Y90|Rotation::Y270 => 1,
        Rotation::Y180 => 2
    }; // AT LEAST this amount of rotation. However:

    let cost = (distance_needed.abs_sum() + rotation_cost) as u64;
    cur_cost + cost
}

fn blocking_path(state: &WorldState, start: &Node, end: &Node) -> u64 {
    let mut current = start.loc.clone();
    let mut dist_needed = &end.loc-&start.loc;


    
    while dist_needed != Vec3::zero() {
        // for i in 0..3usize {
        //     if dist_needed.index(i) != &0 {

        //     }
        // }
        let mut non_block_found = false;
        for axis in &AxisDirection::ALL {
            let dir = axis.to_unit_vector();
            let dot = dist_needed.dot(&dir);
            if dot > 0 {
                let dest = &current + &dir;
                if let Some(_block) = state.state.get(&dest) {
                    
                    continue;
                }
                non_block_found = true;
                current = dest;
                dist_needed = &end.loc-&current;
                break;
            }
            
            
        }
        if !non_block_found {
            return 1;
        }
    }
    0
}


#[derive(Debug)]
pub struct RTAStar {
    h: HashMap<Node, u64>,
    goal: Node,
    next: Option<TurtleAction>,
    it: i32
}


impl RTAStar {
    pub fn new(goal_loc: Vec3<i32>, goal_dir: AxisDirection) -> Self {
        let goal = Node{loc: goal_loc, dir: goal_dir};
        RTAStar {
            h: HashMap::new(),
            goal: goal,
            next: None,
            it: 0
        }
    }

    fn select_successor(&self, nodes: &Vec<(Node, TurtleAction)>, costs: &Vec<u64>) -> usize {
        let min_cost = costs.iter().min().unwrap();
        let nodes_min_cost: Vec<usize> = (0..costs.len()).filter(|i| &costs[*i] == min_cost).collect();
        let nodes_not_turning: Vec<usize> = nodes_min_cost.iter().map(|i| *i).filter(|i| matches!((&nodes[*i]).1, TurtleAction::Move{..})).collect();
        if nodes_not_turning.len() > 0 {
            nodes_not_turning[0]
        } else {
            nodes_min_cost[0]
        }
    }

    fn get_second_cost(&self, costs: &Vec<u64>) -> u64 {
        let min_cost = costs.iter().min().unwrap();
        let nodes_min_cost: Vec<usize> = (0..costs.len()).filter(|i| &costs[*i] == min_cost).collect();
        if nodes_min_cost.len() > 1 {
            *min_cost
        } else {
            let larger_than_min= costs.iter().filter(|f| *f > &min_cost).min();
            *larger_than_min.unwrap_or(min_cost)
        }
    }

    fn next_node(&mut self, state: &TurtleState) -> TurtleAction { 
        let loc = state.location.loc_absolute.as_ref().unwrap();
        let dir = &state.location.direction_absolute;
        
        let cur_node = Node {loc: loc.clone(), dir:dir.clone()};
        if cur_node == self.goal {
            return TurtleAction::Stop;
        }
        // const SEARCH_DEPTH: usize = 3;
        let mut successors = generate_neighbors(&cur_node);
        let mut costs: Vec<u64> = vec![];
        
        for node in &successors {
            let cost = match self.h.get(&node.0) {
                Some(h) => {
                    1 + h
                },
                None => {
                    1+dist_heuristic(&state.world, &node.0, &self.goal, false, 0)
                     +blocking_path(&state.world, &node.0, &self.goal)
                }
            };
            println!("{:?} {:?} {:?}", node.1, cost, self.h.get(&node.0).is_some());
            costs.push(cost);
            
        }
        let index = self.select_successor(&successors, &costs);
        let best_succ = successors.remove(index);
        let second_cost = self.get_second_cost(&costs);
        // println!("Was {:?} target {:?}, cost_there {:?} cost_here {:?}", cur_node, best_succ, min_cost, second_cost);
        // println!("Costs: {:?}", self.h);

        self.h.insert(cur_node, second_cost);
        // thread::sleep(Duration::from_millis(7500));
        best_succ.1
        

    }

    pub fn update(&mut self, state: &TurtleState)  {
        let next= self.next_node(&state);
        self.next = Some(next);
    }

    pub fn next(&self) -> Result<TurtleAction> {
        let next = self.next.as_ref();
        if next.is_some() {
            Ok(next.unwrap().clone())
        } else {
            Err(anyhow!("No steps left in pathfinding!"))
        }

    }

}
