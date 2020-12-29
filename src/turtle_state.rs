
use crate::turtle_action::*;
use std::ops::Index;
use crate::vec3::*;

// Guesses the state of turtle by the recorded executed commands.
type Coord = Vec3::<i32>;
#[derive(PartialEq, Debug)]
pub enum AxisDirection {
    Xp,
    Xm,
    Zp,
    Zm
}

/*
    Xp
Zm      Zp
    Xm

*/

impl AxisDirection {
    const AD_XP: Vec3<i32> = Vec3::<i32>(1,0,0);
    const AD_XM: Vec3<i32> = Vec3::<i32>(-1,0,0);
    const AD_ZP: Vec3<i32> = Vec3::<i32>(0,0,1);
    const AD_ZM: Vec3<i32> = Vec3::<i32>(0,0,-1);
    const AD_YP: Vec3<i32> = Vec3::<i32>(0,1,0);
    const AD_YM: Vec3<i32> = Vec3::<i32>(0,-1,0);

    fn to_unit_vector(&self) -> Vec3<i32> {
        match self {
            AxisDirection::Xp => AxisDirection::AD_XP,
            AxisDirection::Xm => AxisDirection::AD_XM,
            AxisDirection::Zp => AxisDirection::AD_ZP,
            AxisDirection::Zm => AxisDirection::AD_ZM
        }
    }

    fn rotate_right(&self) -> AxisDirection {
        match self {
            AxisDirection::Xp => AxisDirection::Zp,
            AxisDirection::Zp => AxisDirection::Xm,
            AxisDirection::Xm => AxisDirection::Zm,
            AxisDirection::Zm => AxisDirection::Xp
        }
    }

    fn rotate_left(&self) -> AxisDirection {
        match self {
            AxisDirection::Xp => AxisDirection::Zm,
            AxisDirection::Zm => AxisDirection::Xm,
            AxisDirection::Xm => AxisDirection::Zp,
            AxisDirection::Zp => AxisDirection::Xp
        }
    }
}

#[derive(Debug)]
pub struct TurtleState {
    loc: Coord,
    direction: AxisDirection
}


impl TurtleState {
    const DEFAULT_DIRECTION: AxisDirection = AxisDirection::Xp;
    pub fn new() -> Self {
        TurtleState {loc: Vec3::zero(), direction: TurtleState::DEFAULT_DIRECTION}
    }
    pub fn update(&mut self, action: &TurtleAction) {
        match action {
            TurtleAction::Move {direction} => {
                let unit_dir = 
                match direction {
                    RelativeDirection::Up => AxisDirection::AD_YP,
                    RelativeDirection::Down => AxisDirection::AD_YM,
                    RelativeDirection::Forward => self.direction.to_unit_vector(),
                    RelativeDirection::Backward => (-self.direction.to_unit_vector()),
                    _ => panic!()
                };
                self.loc += &unit_dir;
            },
            TurtleAction::Turn {direction} => {
                let new_dir = match direction {
                    RelativeDirection::Left => self.direction.rotate_left(),
                    RelativeDirection::Right => self.direction.rotate_right(),
                    _ => panic!()
                };
                self.direction = new_dir;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_move() {
        let mut state =  TurtleState::new();
    
        assert_eq!(AxisDirection::Xp, state.direction);
        let move_action = go::forward();
        state.update(&move_action);
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::new(1,0,0), state.loc);
        state.update(&go::backward());
        assert_eq!(Coord::zero(), state.loc);
        state.update(&go::up());
        assert_eq!(Coord::new(0,1,0), state.loc);
        state.update(&go::down());
        assert_eq!(Coord::zero(), state.loc);
    }
    #[test]
    fn test_turn_move() {
        let mut state = TurtleState::new();
        state.update(&turn::left());
        assert_eq!(AxisDirection::Zm, state.direction);
        state.update(&go::forward());
        assert_eq!(AxisDirection::AD_ZM, state.loc);
        state.update(&go::backward());
        assert_eq!(Coord::zero(), state.loc);
        state.update(&turn::right());
        state.update(&turn::right());
        assert_eq!(AxisDirection::Zp, state.direction);
        state.update(&go::forward());
        assert_eq!(AxisDirection::AD_ZP, state.loc); 
    }

    #[test]
    fn test_circle_turns() {
        let mut state = TurtleState::new();
        /*  Before each iteration      after
                21                      14
                34                      23
        */
        for _ in 0..4 {
            state.update(&turn::left());
            state.update(&go::forward());
        }
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::zero(), state.loc);

        for _ in 0..4 {
            state.update(&turn::right());
            state.update(&go::forward());
        }
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::zero(), state.loc);

    }
}