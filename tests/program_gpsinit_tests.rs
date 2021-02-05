
use turtlers::world_simulator::Runner;
use turtlers::turtle_program::InitGpsProgram;
use turtlers::turtle_state::Coord;

#[cfg(test)]
mod tests {
    use super::*;
    use turtlers::turtle_rotation::AxisDirection;

    #[test]
    fn going_forward_works() {
        let mut runner = Runner::make_world_unknown_loc_unknown_originxp("test_box");
        let mut program = InitGpsProgram::new();
        assert_eq!(0, runner.world().state.len());
        assert_eq!(Option::None, runner.location().loc_absolute);
        runner.run(Box::new(program));
        assert_eq!(Option::Some(Coord::new(0,0,0)), runner.location().loc_absolute);
    }

    #[test]
    fn going_backward_works() {
        let mut runner = Runner::make_world_unknown_loc_unknown("test_box", Coord::new(2, 0, 0), AxisDirection::Xp);
        let mut program = InitGpsProgram::new();
        assert_eq!(0, runner.world().state.len());
        assert_eq!(Option::None, runner.location().loc_absolute);
        runner.run(Box::new(program));
        assert_eq!(Option::Some(Coord::new(2,0,0)), runner.location().loc_absolute);
    }
}