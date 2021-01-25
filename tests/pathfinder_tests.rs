mod runner;
use runner::Runner;
use turtlers::turtle_program::{InitGpsProgram, MultiProgram};
use turtlers::turtle_state::Coord;

#[cfg(test)]
mod tests {
    use super::*;
    use turtlers::turtle_state::LocationMode;
    use turtlers::turtle_rotation::{AxisDirection, Rotation};
    use turtlers::turtle_program::PathfindingTestProgram;


    fn create_gps_and_pathfinder(loc: &Coord, dir: &AxisDirection) -> Box<MultiProgram> {
        let mut multi = MultiProgram::new(Box::new(InitGpsProgram::new()));
        let pathfinder = PathfindingTestProgram::new(loc.clone(), dir.clone());
        multi.add(Box::new(pathfinder));
        Box::new(multi)
    }

    #[test]
    fn multiprogram_finds_gps() {
        let mut multi = MultiProgram::new(Box::new(InitGpsProgram::new()));
        let mut runner = Runner::make_world_unknown_loc_unknown_originxp("test_box");
        runner.run(Box::new(multi));
        assert_eq!(LocationMode::Absolute((Coord::zero(), Rotation::Y0)), runner.location().location_precision)
    }

    #[test]
    fn pathfinder_easy_paths() {
        let coord = Coord::new(0,0,0);
        let mut program = create_gps_and_pathfinder(&coord, &AxisDirection::Xp);
        let mut runner = Runner::make_world_unknown_loc_unknown_originxp("test_box");
        runner.run(program);

    }
}