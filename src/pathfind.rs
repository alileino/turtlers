
use std::{cmp::min, collections::{HashMap}, thread};
use crate::{turtle_action::{TurtleAction, gps}, vec3::Vec3};
use crate::{turtle_rotation::*};
use crate::{turtle_state::*};
use std::time::Duration;
use anyhow::{anyhow, Result};

#[derive(PartialEq, Hash, Debug, Eq)]
struct Node {
    loc: Vec3<i32>,
    dir: AxisDirection
}




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

fn dist_heuristic(state: &WorldState, start: &Node, end: &Node, can_dig: bool, _cur_cost: u64) -> u64 {
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
    cost
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

    fn next_node(&mut self, state: &TurtleState) -> TurtleAction { 
        let loc = state.location.loc_absolute.as_ref().unwrap();
        let dir = &state.location.direction_absolute;
        
        let cur_node = Node {loc: loc.clone(), dir:dir.clone()};
        if cur_node == self.goal {
            return TurtleAction::Stop;
        }
        let mut successors = generate_neighbors(&cur_node);
        let mut costs: Vec<u64> = vec![];
        for node in &successors {
            let cost = match self.h.get(&node.0) {
                Some(h) => {
                    1 + h
                },
                None => {
                    1+dist_heuristic(&state.world, &node.0, &self.goal, false, 0)
                }
            };
            println!("{:?} {:?} {:?}", node.1, cost, self.h.get(&node.0).is_some());
            costs.push(cost);
            
        }
        
        // println!("Successors: {:?} Costs: {:?}", successors, costs);
        let min_cost = costs.iter().min().unwrap();
        let larger_than_min= costs.iter().filter(|f| *f > &min_cost).min();
        let min_target =  larger_than_min.unwrap_or(min_cost);
        let index = costs.iter().position(|f| f==min_cost).unwrap();
        let best_succ = successors.remove(index);
        println!("Was {:?} target {:?}, cost_there {:?} cost_here {:?}", cur_node, best_succ, min_cost, min_target);
        // println!("Costs: {:?}", self.h);

        self.h.insert(cur_node, *min_cost);
        // thread::sleep(Duration::from_millis(7500));
        best_succ.1
        

    }

    pub fn update(&mut self, state: &TurtleState)  {
        self.it += 1;
        if self.it % 2 == 1 {
            self.next = Some(gps::locate())
        } else {
            let next= self.next_node(&state);

            self.next = Some(next)
        }
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
