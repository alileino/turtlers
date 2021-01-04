
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use crate::{turtle_action::TurtleAction, vec3::Vec3};
use crate::{turtle_rotation::*};
use crate::{turtle_state::*};


#[derive(PartialEq, Eq, Hash)]
struct Node {
    loc: Vec3<i32>,
    dir: AxisDirection
}


fn generate_neighbors(node: &Node) -> Vec<Node> {
    let mut result = vec![];
    let turn_left = node.dir.rotate_left();
    let turn_right = node.dir.rotate_right();
    result.push(Node{loc: node.loc.clone(), dir: turn_left});
    result.push(Node{loc: node.loc.clone(), dir: turn_right});
    for rel_dir in &[RelativeDirection::Forward, RelativeDirection::Backward, RelativeDirection::Up, RelativeDirection::Down] {
        let loc = get_dest_axisdirection(&node.dir, rel_dir);
        result.push(Node{loc:loc, dir:node.dir.clone()});
    }

    
    result
}

fn manhattan(start: &Node, end: &Node) -> f64 {
    1.0
}
pub struct RTAStar {
    h: HashMap<Node, f64>,
    goal: Node
}


impl RTAStar {
    pub fn new(goal_loc: Vec3<i32>, goal_dir: AxisDirection) -> Self {
        let goal = Node{loc: goal_loc, dir: goal_dir};
        RTAStar {
            h: HashMap::new(),
            goal: goal
        }
    }

    fn next_node(&mut self, state: &TurtleState) -> Node {
        let loc = state.location.loc_absolute.as_ref().unwrap();
        let dir = &state.location.direction_absolute;
        
        let cur_node = Node {loc: loc.clone(), dir:dir.clone()};
        let mut successors = generate_neighbors(&cur_node);
        let mut costs: Vec<f64> = vec![];
        for node in &successors {
            let cost = match self.h.get(&node) {
                Some(h) => {
                    1f64 + h
                },
                None => {
                    1f64+manhattan(&node, &self.goal)
                }
            };
            costs.push(cost);
        }
        let min_cost = costs.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let larger_than_min= costs.iter().filter(|f| *f > &min_cost).fold(f64::INFINITY, |a, &b| a.min(b));
        let min_target = if larger_than_min.is_infinite() {
            larger_than_min
        } else {
            min_cost
        };
        let index = costs.iter().position(|f| f==&min_cost).unwrap();
        let best_succ = successors.remove(index);
        self.h.insert(cur_node, min_target);


        best_succ

    }

}
