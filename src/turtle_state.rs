
use crate::turtle_action::*;
use std::ops::Index;
use crate::vec3::*;

// Guesses the state of turtle by the recorded executed commands.

enum AxisDirection {
    Xp,
    Xm,
    Zp,
    Zm
}
// const AD_XP: Vec3<i32> = Vec3::<i32>::from_array([1i32,0,0]);

impl AxisDirection {
    

    fn to_unit_vector(&self) -> [i32;3] {
        match self {
            AxisDirection::Xp => [1, 0, 0],
            AxisDirection::Xm => [-1, 0, 0],
            AxisDirection::Zp => [0, 0, 1],
            AxisDirection::Zm => [0, 0, -1]
        }
    }
}



struct TurtleState {
    loc: [i32; 3],
    direction: AxisDirection
}

impl TurtleState {

    fn update(&mut self, action: &TurtleAction) {
        match action {
            TurtleAction::Move {direction} => {
                match direction {
                    RelativeDirection::Up => self.loc[1] += 1,
                    RelativeDirection::Down => self.loc[1] -= 1,
                    // RelativeDirection::Forward => 
                    _ => panic!()
                }
            },
            _ => panic!()
        }
    }
}

impl Index<usize> for TurtleState {
    type Output = i32;
    fn index<'a>(&'a self, i: usize) -> &'a i32 {
        &self.loc[i]
    }
}