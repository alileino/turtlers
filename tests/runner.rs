
use turtlers::turtle_state::*;
use turtlers::turtle::*;
use anyhow::Result;
use turtlers::turtle_program::{TurtleProgram};
use turtlers::turtle_action::{TurtleAction, TurtleActionReturn};
use turtlers::turtle_rotation::AxisDirection;
use std::borrow::Borrow;

pub struct Runner {
    pub turtle: Turtle,
    shadow_state: TurtleState
    // ,shadow_location: Option<LocationState>
}



impl Runner {
    const TEST_STATE_DIR: &'static str = "tests/state";

    /// if load_state=true, the state is immediately loaded to WorldState. Otherwise it will only
    /// be used when executing commands, so that it will eventually be revealed.
    pub fn new(state_name: &str, load_state: bool, start_location: (Coord, AxisDirection), start_location_known: bool) -> Self {
        let load_from_test_policy = StateSerializationPolicy::LoadOnly {load_dir: Runner::TEST_STATE_DIR.to_string()};
        let ser_policy =
            if load_state {
                load_from_test_policy.clone()
            } else {
                StateSerializationPolicy::None
        };

        let shadow_wstate = WorldState::new(state_name.to_string(), load_from_test_policy);

        let rotation = AxisDirection::dot(&LocationState::DEFAULT_DIRECTION, &start_location.1);
        let mut shadow_loc = LocationState {
            location_precision: LocationMode::Absolute((start_location.0.clone(), rotation)),
            loc: Coord::zero(),
            direction: LocationState::DEFAULT_DIRECTION,
            loc_absolute: Some(start_location.0.clone()),
            direction_absolute: start_location.1
        };


        let location = if start_location_known {
            shadow_loc.clone()
        } else {
            LocationState::new()
        };
        let world = WorldState::new(state_name.to_string(), ser_policy);
        let state = TurtleState::from(location, world);
        let turtle = Turtle::from(state_name.to_string(), state);

        let shadow_state = TurtleState::from(shadow_loc, shadow_wstate);


        let mut runner = Runner {
            turtle,
            shadow_state,
        };

        runner
    }

    pub fn make_world_unknown_loc_known(state_name: &str, start_loc: Coord, start_axis: AxisDirection) -> Self {
        Self::new(
            state_name,
            false,
            (start_loc, start_axis),
            true
        )
    }

    pub fn make_world_unknown_loc_known_originxp(state_name: &str) -> Self {
        Self::make_world_unknown_loc_known(state_name, Coord::zero(), AxisDirection::Xp)
    }

    pub fn make_world_known_loc_known(state_name: &str, start_loc: Coord, start_axis: AxisDirection) -> Self {
        Self::new(
            state_name,
            true,
            (start_loc, start_axis),
            true
        )
    }

    pub fn make_world_known_loc_known_originxp(state_name: &str) -> Self {
        Self::make_world_known_loc_known(state_name, Coord::zero(), AxisDirection::Xp)
    }

    pub fn make_world_unknown_loc_unknown(state_name: &str, start_loc: Coord, start_axis: AxisDirection) -> Self {
        Self::new(
            state_name,
            false,
            (start_loc, start_axis),
            false
        )
    }

    pub fn make_world_unknown_loc_unknown_originxp(state_name: &str) -> Self {
        Self::make_world_unknown_loc_unknown(state_name, Coord::zero(), AxisDirection::Xp)
    }

    pub fn execute_action(&self, action: &TurtleAction) -> TurtleActionReturn {
        match action {
            TurtleAction::Turn { .. } => TurtleActionReturn::Success,
            TurtleAction::Move { direction } => {
                let unit_dir = self.location().get_dest_direction_local(&direction);
                // let dest_loc =
                TurtleActionReturn::Success
            },
            TurtleAction::Dig { .. } => {todo!()},
            TurtleAction::Detect { .. } => {todo!()},
            TurtleAction::Place { .. } => {todo!()},
            TurtleAction::Drop { .. } => {todo!()},
            TurtleAction::Attack { .. } => {todo!()},
            TurtleAction::Suck { .. } => {todo!()},
            TurtleAction::Inspect { .. } => {todo!()},
            TurtleAction::Compare { .. } => {todo!()},
            TurtleAction::Select { .. } => {todo!()},
            TurtleAction::ItemCount { .. } => {todo!()},
            TurtleAction::ItemSpace { .. } => {todo!()},
            TurtleAction::ItemDetail { .. } => {todo!()},
            TurtleAction::TransferTo { .. } => {todo!()},
            TurtleAction::CompareTo { .. } => {todo!()},
            TurtleAction::GpsLocate { .. } => {todo!()},
            TurtleAction::Stop => panic!()
        };
        TurtleActionReturn::Success
    }

    pub fn start(&mut self, program: Box<dyn TurtleProgram>) {
        self.turtle.set_program(program);
        let action = self.turtle.next().unwrap_or(&TurtleAction::Stop).clone();
        while action != TurtleAction::Stop {

            let response = self.execute_action(&action);
            self.turtle.update(&response);

        }
    }

    pub fn world(&self) -> &WorldState {
        &self.turtle.state.world
    }

    pub fn location(&self) -> &LocationState {
        &self.turtle.state.location
    }

    pub fn shadow_world(&self) -> &WorldState {
        &self.shadow_state.world
    }

    pub fn shadow_location(&self) -> &LocationState {
        &self.shadow_state.location
    }





    // fn load_state(&mut self, id: &str) -> Result<()>{
    //     let state = deserialize_worldstate()?;
    //     self.turtle.state.world.update_all(state);
    //     Ok(())
    // }
}




#[cfg(test)]
mod tests {
    use super::*;
    use turtlers::turtle_action::go;
    use turtlers::turtle_program::FromActionsProgram;

    #[test]
    fn runner_known_world_and_loc() {
        let start_loc = Coord::new(1,2,3);
        let runner =  Runner::make_world_known_loc_known("test_box", start_loc.clone(), AxisDirection::Zp);
        assert_eq!(49, runner.world().state.len());
        assert_eq!(49, runner.shadow_world().state.len());
        assert_eq!(&start_loc, runner.shadow_location().loc_absolute.as_ref().unwrap());
        assert_eq!(&start_loc, runner.location().loc_absolute.as_ref().unwrap());
        assert_eq!(&AxisDirection::Zp, &runner.location().direction_absolute);
    }

    #[test]
    fn runner_unknown_world_known_loc() {
        let start_loc = Coord::new(1,2,3);
        let runner =  Runner::make_world_unknown_loc_known("test_box", start_loc.clone(), AxisDirection::Zp);
        assert_eq!(0, runner.world().state.len());
        assert_eq!(49, runner.shadow_world().state.len());
        assert_eq!(&start_loc, runner.shadow_location().loc_absolute.as_ref().unwrap());
        assert_eq!(&start_loc, runner.location().loc_absolute.as_ref().unwrap());

    }

    #[test]
    fn runner_unknown_world_unknown_loc() {
        let start_loc = Coord::new(1,2,3);
        let runner =  Runner::make_world_unknown_loc_unknown("test_box", start_loc.clone(), AxisDirection::Zp);
        assert_eq!(0, runner.world().state.len());
        assert_eq!(49, runner.shadow_world().state.len());
        assert_eq!(&start_loc, runner.shadow_location().loc_absolute.as_ref().unwrap());
        assert!(runner.location().loc_absolute.is_none());
        assert_eq!(LocationMode::Relative(Option::None), runner.location().location_precision);
    }

    #[test]
    fn runner_state_wprogram() {

        let program = FromActionsProgram::from(
            &[go::forward()]
        );
        // let runner = Runner::

    }
}