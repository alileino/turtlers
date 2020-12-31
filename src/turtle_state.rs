
use crate::{turtle_action::*};
use std::{ops::Index};
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
#[derive(Debug, Clone)]
pub enum Rotation {
    Y0,
    Y90,
    Y180,
    Y270
}

impl Rotation {
    
    const ALL: [Rotation;4] = [Rotation::Y0, Rotation::Y90, Rotation::Y180, Rotation::Y270];

    const ROT_0: (Coord, Coord, Coord) = (Vec3::<i32>(1,0,0), Vec3::<i32>(0,1,0), Vec3::<i32>(0,0,1));
    const ROT_Y90: (Coord, Coord, Coord) = (Vec3::<i32>(0,0,1), Vec3::<i32>(0,1,0), Vec3::<i32>(-1,0,0));
    const ROT_Y180: (Coord, Coord, Coord) = (Vec3::<i32>(-1, 0, 0), Vec3::<i32>(0,1,0), Vec3::<i32>(0,0,-1));
    const ROT_Y270: (Coord, Coord, Coord) = (Vec3::<i32>(0, 0, -1), Vec3::<i32>(0,1,0), Vec3::<i32>(1,0,0));

    pub fn apply_to(&self, vec: &Coord) -> Coord {
        let (x, _y, z) = match self {
            Rotation::Y0 => Rotation::ROT_0,
            Rotation::Y90 => Rotation::ROT_Y90,
            Rotation::Y180 => Rotation::ROT_Y180,
            Rotation::Y270 => Rotation::ROT_Y270
        };
        
        Vec3::<i32>(x.0*vec.0 + x.2*vec.2, vec.1, z.0*vec.0 + z.2*vec.2)
    }

    pub fn find_rotation(src: &Coord, dst: &Coord) -> Self {
        for rot in Rotation::ALL.iter() {
            if &rot.apply_to(src) == dst {
                return rot.clone();
            }
        }
        panic!()
    }
}

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


//  Two different measurements guarantee the orientation of the state
#[derive(Debug, Clone)]
pub enum LocationMode {
    Relative(Option<(Coord, Coord)>), // relative pos1, absolute pos1
    Absolute((Coord, Rotation)) // difference of new relative pos2 and relative pos1, and same for absolute position
}

 

#[derive(Debug)]
pub struct LocationState {
    pub loc: Coord, // Relative location
    pub loc_absolute: Option<Coord>, // Absolute, requires two GPS measurements from different locations
    direction: AxisDirection,
    pub location_precision: LocationMode
}


impl LocationState {
    const DEFAULT_DIRECTION: AxisDirection = AxisDirection::Xp;
    pub fn new() -> Self {
        LocationState {
            loc: Vec3::zero(), 
            direction: LocationState::DEFAULT_DIRECTION, 
            loc_absolute: None,
            location_precision: LocationMode::Relative(None)
        }
    }

    fn update_gps(&mut self, loc: &Vec3<i32>) {

        match &self.location_precision {
            LocationMode::Relative(None) => {
                self.location_precision = LocationMode::Relative(Some((self.loc.clone(), loc.clone())));
            },
            LocationMode::Relative(Some(loc1)) => {
                let rel_diff = &self.loc-&loc1.0; // cur relative - old relative
                if rel_diff.0 == 0 && rel_diff.2 == 0 { // Can't determine rotation with no x or z offsets
                    return;
                }
                let abs_diff = loc-&loc1.1; // cur absolute - old absolute
                let rotation = Rotation::find_rotation(&rel_diff, &abs_diff);
                println!("Found rotation {:?} and offset {:?}", rotation, loc1.1);
                self.location_precision = LocationMode::Absolute((loc1.1.to_owned(), rotation));

            },
            LocationMode::Absolute(_) => {
                if self.loc_absolute.as_ref().unwrap()!= loc {
                    panic!("New gps measurement {:?} differs from calculated value of {:?}", loc, self.loc_absolute);
                }
            }
        }

    }

    fn update_absolute_location(&mut self) {
        if let LocationMode::Absolute((base, rot)) = &self.location_precision {
            
            let loc_wrot = rot.apply_to(&self.loc);
            let loc_woffset = &loc_wrot + base;
            self.loc_absolute = Some(loc_woffset);
        }
    }
    pub fn update(&mut self, action: &TurtleAction, result: &TurtleActionReturn) {
        match action {
            TurtleAction::Move {direction} => {
                if *result != TurtleActionReturn::Success {
                    return;
                }
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
                if *result != TurtleActionReturn::Success {
                    return;
                }
                let new_dir = match direction {
                    RelativeDirection::Left => self.direction.rotate_left(),
                    RelativeDirection::Right => self.direction.rotate_right(),
                    _ => panic!()
                };
                self.direction = new_dir;
            },
            TurtleAction::GpsLocate{..} => {
                if let TurtleActionReturn::Coordinate(location) = &*result {
                    self.update_gps(location);
                }
            }
            _ => panic!()
        }
        self.update_absolute_location();
    }
}

impl Index<usize> for LocationState {
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
        let mut state =  LocationState::new();
    
        assert_eq!(AxisDirection::Xp, state.direction);
        let move_action = go::forward();
        state.update(&move_action, &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::new(1,0,0), state.loc);
        state.update(&go::backward(), &TurtleActionReturn::Success);
        assert_eq!(Coord::zero(), state.loc);
        state.update(&go::up(), &TurtleActionReturn::Success);
        assert_eq!(Coord::new(0,1,0), state.loc);
        state.update(&go::down(), &TurtleActionReturn::Success);
        assert_eq!(Coord::zero(), state.loc);
    }
    #[test]
    fn test_turn_move() {
        let mut state = LocationState::new();
        state.update(&turn::left(), &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::Zm, state.direction);
        state.update(&go::forward(), &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::AD_ZM, state.loc);
        state.update(&go::backward(), &TurtleActionReturn::Success);
        assert_eq!(Coord::zero(), state.loc);
        state.update(&turn::right(), &TurtleActionReturn::Success);
        state.update(&turn::right(), &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::Zp, state.direction);
        state.update(&go::forward(), &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::AD_ZP, state.loc); 
    }

    #[test]
    fn test_circle_turns() {
        let mut state = LocationState::new();
        /*  Before each iteration      after
                21                      14
                34                      23
        */
        for _ in 0..4 {
            state.update(&turn::left(), &TurtleActionReturn::Success);
            state.update(&go::forward(), &TurtleActionReturn::Success);
        }
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::zero(), state.loc);

        for _ in 0..4 {
            state.update(&turn::right(), &TurtleActionReturn::Success);
            state.update(&go::forward(), &TurtleActionReturn::Success);
        }
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::zero(), state.loc);

    }
}