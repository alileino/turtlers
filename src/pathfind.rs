
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
    for rel_dir in &[RelativeDirection::Forward, RelativeDirection::Backward, RelativeDirection::Up, RelativeDirection::Down] {
        let loc = get_dest_axisdirection(&node.dir, rel_dir);
        result.push(Node{loc:loc, dir:node.dir.clone()});
    }

    
    result
}

struct RTAStar {
    
}