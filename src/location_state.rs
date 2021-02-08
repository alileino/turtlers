use std::ops::Index;

use crate::turtle_action::{TurtleAction, TurtleActionReturn};
use crate::turtle_rotation::{AxisDirection, get_dest_axisdirection, RelativeDirection, Rotation};
use crate::turtle_state::Coord;
use crate::vec3::Vec3;

//  Two different measurements guarantee the orientation of the state
#[derive(Debug, Clone, PartialEq)]
pub enum LocationMode {
    Relative(Option<(Coord, Coord)>), // relative pos1, absolute pos1
    Absolute((Coord, Rotation)) // difference of new relative pos2 and relative pos1, and same for absolute position
}


#[derive(Debug, Clone)]
pub struct LocationState {
    pub loc: Coord, // Relative location
    pub loc_absolute: Option<Coord>, // Absolute, requires two GPS measurements from different locations
    pub direction: AxisDirection,
    pub direction_absolute: AxisDirection,
    pub location_precision: LocationMode,
    pub history: Vec<(Coord, AxisDirection)>
}

impl LocationState {
    pub const DEFAULT_DIRECTION: AxisDirection = AxisDirection::Xp;
    pub fn new() -> Self {
        LocationState {
            loc: Vec3::zero(),
            direction: LocationState::DEFAULT_DIRECTION,
            direction_absolute: LocationState::DEFAULT_DIRECTION,
            loc_absolute: None,
            location_precision: LocationMode::Relative(None),
            history: vec![]
        }
    }

    fn update_gps(&mut self, new_absolute: &Vec3<i32>) {
        println!("Updating gps {:?}", self.location_precision);
        match &self.location_precision {
            LocationMode::Relative(None) => {
                self.location_precision = LocationMode::Relative(Some((self.loc.clone(), new_absolute.clone())));
            },
            LocationMode::Relative(Some(old_loc)) => {
                let (old_rel, old_abs) = old_loc;
                let rel_diff = &self.loc-&old_rel; // cur relative - old relative
                if rel_diff.0 == 0 && rel_diff.2 == 0 { // Can't determine rotation with no x or z offsets
                    return;
                }
                let abs_diff = new_absolute-&old_abs; // cur absolute - old absolute
                let rotation = Rotation::find_rotation(&rel_diff, &abs_diff);
                let new_offset = old_abs - &rotation.apply_to(old_rel);
                // println!("Found rotation {:?} and offset {:?} from {:?} arriving at {:?}", rotation, new_offset, self.loc, new_absolute);
                // println!("Relative coords: old: {:?} cur: {:?} diff: {:?}", old_loc.0, self.loc, rel_diff);
                // println!("Absolute coords: old: {:?} cur: {:?} diff: {:?}", old_loc.1, new_absolute, abs_diff);


                let mut new_history = vec![];
                for (rel_coord, rel_dir) in &self.history {
                    let loc_wrot = rotation.apply_to(&rel_coord);
                    let loc_woffset = &loc_wrot + &new_offset;
                    let axis_rotated = AxisDirection::from(&rotation.apply_to(&rel_dir.to_unit_vector()));
                    new_history.push((loc_woffset, axis_rotated));
                }
                self.history = new_history;
                self.location_precision = LocationMode::Absolute((new_offset, rotation));

            },
            LocationMode::Absolute(_) => {
                if self.loc_absolute.as_ref().unwrap() != new_absolute {
                    panic!("New gps measurement {:?} differs from calculated value of {:?}", new_absolute, self.loc_absolute);
                } else {
                    // println!("GPS {:?} == {:?}", new_absolute, self.loc_absolute.as_ref().unwrap());
                }
            }
        }
        println!("Update done {:?}", self.location_precision);

    }

    fn update_absolute_location(&mut self) {
        if let LocationMode::Absolute((base, rot)) = &self.location_precision {

            let loc_wrot = rot.apply_to(&self.loc);
            let loc_woffset = &loc_wrot + base;
            self.loc_absolute = Some(loc_woffset.clone());
            println!("Rotating {:?}, woffset {:?}", self.loc, loc_woffset);
            let unit = self.direction.to_unit_vector();
            let unit_rotated = rot.apply_to(&unit);
            let as_axis_dir = AxisDirection::from(&unit_rotated);

            println!("{:?} {:?} {:?}", unit, unit_rotated, as_axis_dir);
            self.direction_absolute = as_axis_dir;
            let latest = (loc_woffset, self.direction_absolute.clone());
            if Some(&latest) != self.history.last() {
                self.history.push(latest);
            }
        } else {
            let latest = (self.loc.clone(), self.direction.clone());
            if Some(&latest) != self.history.last() {
                self.history.push((self.loc.clone(), self.direction.clone()));
            }
        }
    }

    pub fn get_path_absolute(&self) -> Vec<(AxisDirection, Rotation)> {
        let mut result = vec![];
        for i in 1..self.history.len() {
            let first = &self.history[i-1];
            let second = &self.history[i];
            let delta_coord = &second.0- &first.0;
            let delta = AxisDirection::from(&delta_coord);
            let delta_dir = AxisDirection::dot(&second.1, &first.1);
            result.push((delta, delta_dir));
        }
        result
    }

    pub fn print_history(&self) {
        let history = self.get_path_absolute();

        let mut result = vec![];
        for i in 0..history.len() {
            let (movement_dir, rot) = &history[i];
            let (_position, axis_dir) = &self.history[i];
            // println!("{:?} {:?} {:?}", _position, axis_dir, axis_dir.to_unit_vector());
            let c = match (movement_dir, rot) {
                (AxisDirection::None, rotation) => {
                    match rotation {
                        Rotation::Y90 => '\u{21B7}',
                        Rotation::Y270 => '\u{21B6}',
                        _ => panic!()
                    }
                },
                (dir, Rotation::Y0) => {
                    match (dir, axis_dir) {
                        (AxisDirection::Xp, AxisDirection::Xp) => '\u{2191}',
                        (AxisDirection::Xp, AxisDirection::Xm) => '\u{21E1}',
                        (AxisDirection::Xm, AxisDirection::Xm) => '\u{2193}',
                        (AxisDirection::Xm, AxisDirection::Xp) => '\u{21E3}',
                        (AxisDirection::Zp, AxisDirection::Zp) => '\u{2192}',
                        (AxisDirection::Zp, AxisDirection::Zm) => '\u{21E2}',
                        (AxisDirection::Zm, AxisDirection::Zm) => '\u{2190}',
                        (AxisDirection::Zm, AxisDirection::Zp) => '\u{21E0}',
                        (AxisDirection::Yp, _) => '\u{219F}',
                        (AxisDirection::Ym, _) => '\u{21A1}',
                        _ => panic!()
                    }
                },
                (_, _) => panic!()
            };

            result.push(c);
        }
        let as_string = result.iter().collect::<String>();
        println!("{}, {}", history.len(), as_string);
    }


    pub fn get_dest_direction_local(&self, move_direction: &RelativeDirection) -> Coord {
        get_dest_axisdirection(&self.direction, move_direction)
    }

    pub fn get_dest_direction_absolute(&self, move_direction: &RelativeDirection) -> Option<Coord> {

        if self.loc_absolute.is_none() {
            None
        } else {
            println!("GetDestDirAbsolute {:?} {:?}", self.direction_absolute, move_direction);
            let unit_dir = get_dest_axisdirection(&self.direction_absolute, move_direction);
            Some(unit_dir)
        }
    }

    pub fn get_dest_position_absolute(&self, move_direction: &RelativeDirection) -> Option<Coord> {
        if self.loc_absolute.is_none() {
            None
        } else {
            Some(self.loc_absolute.as_ref().unwrap() + self.get_dest_direction_absolute(move_direction).as_ref().unwrap())
        }
    }



    pub fn update(&mut self, action: &TurtleAction, result: &TurtleActionReturn) {
        match action {
            TurtleAction::Move {direction} => {
                if *result != TurtleActionReturn::Success {
                    return;
                }
                let unit_dir = self.get_dest_direction_local(&direction);
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
            TurtleAction::Detect{..} => {}, // Does not affect movement
            TurtleAction::Inspect{..} => {},
            _ => todo!("Not implemented: {:?}", action)
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
    use crate::turtle_action::{turn, go, gps};
    use crate::world_simulator::Runner;
    use crate::turtle_program::InitGpsProgram;

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

    #[test]
    fn test_gps() {
        let mut runner = Runner::make_world_unknown_loc_unknown("test_box", Coord::new(0,0,0), AxisDirection::Zm);
        let gps_program = InitGpsProgram::new();
        assert_eq!(0, runner.location().get_path_absolute().len());
        assert_eq!(AxisDirection::Zm, runner.shadow_location().direction_absolute);
        if let LocationMode::Absolute((offset, rotation)) = &runner.shadow_location().location_precision {
            assert_eq!(&Coord::zero(), offset);
            assert_eq!(&Rotation::Y270, rotation);
        } else {
            assert!(false);
        }
        runner.execute_action(&gps::locate());
        runner.execute_action(&gps::locate());
        assert_eq!(AxisDirection::Zm, runner.shadow_location().direction_absolute);
        runner.run(Box::new(gps_program));
        runner.location().print_history();
        println!("{:?}", runner.location().history);
        assert_eq!(2, runner.location().get_path_absolute().len());
        if let LocationMode::Absolute((offset, rotation)) = &runner.location().location_precision {
            assert_eq!(&Coord::zero(), offset);
            assert_eq!(&Rotation::Y270, rotation);
        } else {
            assert!(false);
        }
        assert_eq!(AxisDirection::Zm, runner.location().direction_absolute);


    }
    // Goal of this test is to ensure that
    // 1. GPS is initialized correctly
    // 2. History shows correct path after initialization.
    // We need GPS to use history, so there's no way around doing both of these.
    #[test]
    fn test_gps_and_history() {
        let mut runner = Runner::make_world_unknown_loc_unknown("test_box", Coord::new(0,0,0), AxisDirection::Zm);
        let gps_program = InitGpsProgram::new();
        assert_eq!(0, runner.location().get_path_absolute().len());
        runner.run(Box::new(gps_program));
        runner.location().print_history();
        assert_eq!(2, runner.location().get_path_absolute().len());
        let history = runner.location().get_path_absolute();
        let (first_dir, first_rot) = history.first().unwrap();

        assert_eq!(&AxisDirection::Zm, first_dir);
        assert_eq!(&Rotation::Y0, first_rot);

    }

}