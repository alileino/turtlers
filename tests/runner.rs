
use turtlers::turtle_state::*;
use turtlers::turtle::*;
use turtlers::turtle_program::{TurtleProgram};
use turtlers::turtle_action::{TurtleAction, TurtleActionReturn, FailureReason};
use turtlers::turtle_rotation::{AxisDirection};

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
        let shadow_loc = LocationState {
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


        let runner = Runner {
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

    pub fn simulate_action(&self, action: &TurtleAction) -> TurtleActionReturn {
        match action {
            TurtleAction::Turn { .. } => TurtleActionReturn::Success,
            TurtleAction::Move { direction } => {
                // Shadow location has to exist and be absolute
                let dest_loc = self.shadow_location().get_dest_position_absolute(direction).unwrap();

                let obstructed = self.shadow_world().is_obstructed(&dest_loc);
                println!("{:?} obstructed: {:?}", dest_loc, obstructed);
                match obstructed {
                    Some(true) => TurtleActionReturn::Failure(FailureReason::MovementObstructed),
                    Some(false) => TurtleActionReturn::Success,
                    None => panic!("Moved to a block which can't be simulated due to missing information.")
                }
            },
            TurtleAction::Dig { .. } => {todo!()},
            TurtleAction::Detect { direction } => {
                let dest_loc = self.shadow_location().get_dest_position_absolute(direction).unwrap();
                let obstructed = self.shadow_world().is_obstructed(&dest_loc);
                match obstructed {
                    Some(value) => TurtleActionReturn::Boolean(value),
                    None => panic!("Moved to a block which can't be simulated due to missing information.")
                }
            },
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
        }
    }

    pub fn set_program(&mut self, program: Box<dyn TurtleProgram>) {
        self.turtle.set_program(program);
    }

    pub fn execute_action(&mut self, action: &TurtleAction) -> TurtleActionReturn {
        if action != &TurtleAction::Stop {
            self.turtle.last_action = Some(action.clone());
            let response = self.simulate_action(&action);
            self.turtle.update( &response);
            self.shadow_state.update(&action, &response);
            response
        } else {
            TurtleActionReturn::Success
        }
    }
    pub fn execute_next(&mut self) -> (TurtleAction, TurtleActionReturn) {
        let action = self.turtle.next().unwrap_or(&TurtleAction::Stop).clone();
        let response = self.execute_action(&action);
        (action, response)
    }

    /// Runs an entire program from start to finish
    pub fn run(&mut self, program: Box<dyn TurtleProgram>) {
        self.set_program(program);
        loop {
            let (action, _response) = self.execute_next();
            if action == TurtleAction::Stop {
                break;
            }
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



}




#[cfg(test)]
mod tests {
    use super::*;
    use turtlers::turtle_action::{go, turn, detect};
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
    fn runner_state_detects_wall_going_forward() {

        let program = FromActionsProgram::from(
            &[go::forward()]
        );
        let mut runner = Runner::make_world_known_loc_known_originxp("test_box");
        assert_eq!(Coord::zero(), runner.location().loc);
        runner.run(Box::new(program));
        assert_eq!(Coord::new(1,0,0), runner.location().loc);
        assert_eq!(Coord::new(1,0,0), runner.shadow_location().loc);
        let program = FromActionsProgram::from(
            &[go::forward(), go::forward()]
        );
        runner.run(Box::new(program));
        assert_eq!(Coord::new(2,0,0), runner.location().loc);
        assert_eq!(Coord::new(2,0,0), runner.shadow_location().loc);
    }

    #[test]
    fn runner_state_detects_wall_while_turning() {
        let program = FromActionsProgram::from(
            &[turn::right(), go::forward(), go::forward()]
        );
        let mut runner = Runner::make_world_unknown_loc_known_originxp("test_box");
        runner.run(Box::new(program));
        assert_eq!(Coord::new(0,0,2), runner.shadow_location().loc);
        assert_eq!(Block::Unknown, runner.world().get(&Coord::new(0,0,3)));
        let response = runner.execute_action(&detect::forward());
        assert_eq!(response, TurtleActionReturn::Boolean(true));
        assert_eq!(Block::Block, runner.world().get(&Coord::new(0,0,3)));

    }
}