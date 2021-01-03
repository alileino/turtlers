
use crate::{vec3::Vec3};
use crate::{turtle_rotation::*};


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
    // let loc_forward = 
    result
}

struct RTAStar {
    
}