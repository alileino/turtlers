use crate::{turtle_action::*, turtle_rotation::AxisDirection, turtle_state::*, vec3::Vec3};
use anyhow::{anyhow, Result};
use pathfind::RTAStar;
use serde_json;
use serde_derive::{Deserialize, Serialize};
extern crate rand;
use rand::{seq::SliceRandom};
use crate::pathfind;
use crate::{turtle_rotation::RelativeDirection};
use std::collections::VecDeque;

pub trait TurtleProgram {
    fn next(&mut self) -> Result<TurtleAction>;
    fn progress(&self) -> (u32, u32); // Represents a fraction of progress
    fn name(&self) -> &str;
    fn update(&mut self, state: &TurtleState, action: &TurtleAction, result: &TurtleActionReturn);
    // Updates received before next() is called
    // fn pre_execution_update(&mut self, state: &TurtleState, action: &TurtleAction, result: &TurtleActionReturn) {}
}

pub enum ProgramState {
    Finished,
    _Waiting(f64), // waiting for turtle to report back
    HasInstructions(f64) // has instructions that can be delivered to turtle
}

pub(crate) fn program_state(program: &Box<dyn TurtleProgram>) -> ProgramState {
    let progress = program.progress();
    if progress.0 == progress.1 {
        return ProgramState::Finished;
    }
    let progress_f = (progress.1 as f64)/(progress.0 as f64);
    ProgramState::HasInstructions(progress_f)
}

#[derive(Serialize, Deserialize)]
pub struct StartProgramMsg {
    msgtype: String, // Should be "start"
    args: Vec<String>
}

// #[derive(Debug)]
// enum TurtleAction {
//     Rotate(RelativeDirection)
// }



#[derive(Debug)]
pub struct RotateProgram {
    steps: u32,
    steps_remaining: u32,
    direction: RelativeDirection
    // actions_remaining: Vec<TurtleAction>
}

impl RotateProgram {
    pub fn new(start_args: &[String]) -> Result<Self> {
        let steps_i32 = start_args[0].parse::<i32>()?;
        let direction = match steps_i32 > 0 {
            true => RelativeDirection::Right,
            false => RelativeDirection::Left
        };
        Ok(RotateProgram {steps: steps_i32.abs() as u32, steps_remaining:steps_i32 as u32, direction: direction})
    }
}

impl TurtleProgram for RotateProgram {
    // fn init(&mut self, start_args: serde_json::Value) -> Result<()> {
    //     self.steps = start_args["steps"].as_i64().unwrap_or(0) as i32;
    //     Ok(())
    // }
    fn progress(&self) -> (u32, u32) {
        return (self.steps-self.steps_remaining, self.steps)
    }

    fn name(&self)  -> &str {"rotate"}

    fn next(&mut self) -> Result<TurtleAction> {
        // self.actions_remaining.push(TurtleAction::Turn{direction: RelativeDirection::Left});
        assert!(self.steps_remaining > 0);
        Ok(go::forward())
        // Ok(TurtleAction::Turn{direction: RelativeDirection::Left})
    }

    fn update(&mut self, _state: &TurtleState, _action: &TurtleAction, _result: &TurtleActionReturn)  {
        self.steps_remaining -= 1;
    }
}


pub struct NoProgram {}
impl TurtleProgram for NoProgram {

    // fn init(&mut self, _: serde_json::Value) -> Result<()> {
    //     Err(anyhow!("Can't initialize NoProgram"))
    // }

    fn progress(&self) -> (u32, u32) {
        (1,1)
    }

    fn name(&self) -> &str {"noprogram"}

    fn next(&mut self) -> Result<TurtleAction> {
        Err(anyhow!("Can't initialize NoProgram"))
    }

    fn update(&mut self, _state: &TurtleState, _action: &TurtleAction, _result: &TurtleActionReturn) {

    }
}

pub fn create_program(msg: &StartProgramMsg ) -> Result<Box<dyn TurtleProgram>> {
    let program_name = &msg.args[0];
    let args = &msg.args[1..];
    let boxed: Box<dyn TurtleProgram> = 
        match program_name.as_str() {
            "rotate" => Box::new(RotateProgram::new(args)?),
            "no" => Box::new(NoProgram{}),
            "random" => Box::new(RandomProgram::new(false, false, false)),
            "locatetest" => Box::new(LocationTestProgram::new()),
            "pathfindtest" => Box::new(PathfindingTestProgram::new(Coord::zero(), AxisDirection::Xp)),
            "initgps" => Box::new(InitGpsProgram::new()),
            program => return Err(anyhow!("Invalid program: {}", program))
    };
    Ok(boxed)

    // match program_name.as_str() {
    //     "rotate" => Ok(Box::new(RotateProgram::new(args)?)),
    //     "no" => Ok(Box::new(NoProgram{})),
    //     program => Err(anyhow!("Invalid program: {}", program))
    // }
    
}


// Message that the lua client sends in response to a command it just tried to execute
#[derive(Serialize, Deserialize)]
pub struct TurtleResponseMsg {
    pub msgtype: String,
    pub result: serde_json::Value // the return value
}

// fn packtable_to_vec(table: serde_json::Value) -> Result<Vec<serde_json::Value>> {
//     let n = table["n"].to_string().parse::<usize>()?;
//     let mut result = vec![];
//     for i in 1..=n {
//         let value = &table[i.to_string()];
//         result.push(value);
//     }
//     Ok(result)
// }

#[derive(Debug)]
pub struct PathfindingTestProgram {
    pathfinder: RTAStar
}

impl PathfindingTestProgram {
    pub fn new(loc: Vec3::<i32>, dir: AxisDirection) -> Self {
        PathfindingTestProgram{
            pathfinder: RTAStar::new(loc, dir)
        }
    }
}

impl TurtleProgram for PathfindingTestProgram {
    fn next(&mut self) -> Result<TurtleAction> {
        println!("PATHFINDING");
        self.pathfinder.next()

    }

    fn progress(&self) -> (u32, u32) {
        (0, 1)
    }

    fn name(&self) -> &str {
        "pathfindtest"
    }

    fn update(&mut self, state: &TurtleState,  _action: &TurtleAction, _result: &TurtleActionReturn) {
        println!("UPDATE PATHFINDING");
        self.pathfinder.update(&state);
    }
}


#[derive(Debug)]
pub struct LocationTestProgram {
    // state: LocationState,
    gps_initialized: bool,
    random: RandomProgram,
    cur_step: u32
}

impl LocationTestProgram {
    pub fn new() -> Self {
        let random = RandomProgram::new(false,true, true);
        // let state = LocationState::new();
        LocationTestProgram{random:random, gps_initialized:false, cur_step:0}
    }
}

impl TurtleProgram for LocationTestProgram {
    fn next(&mut self) -> Result<TurtleAction> {
        if self.cur_step % 7 == 3 {
            Ok(gps::locate())
        } else {
            self.random.next()
        }
    }

    fn progress(&self) -> (u32, u32) {
        (0,1)
    }

    fn name(&self) -> &str {
        "locatetest"
    }

    fn update(&mut self, state: &TurtleState,  action: &TurtleAction, result: &TurtleActionReturn) {
        println!("{:?}", state.location.loc_absolute);
        match action {
            TurtleAction::Move{..}|
            TurtleAction::Turn{..} => {
                // self.state.update(action, result);
            },
            TurtleAction::GpsLocate{..} => {
                if let TurtleActionReturn::Coordinate(_) = &*result {
                    // self.state.update(action, result);
                } else {
                    if state.location.loc_absolute.is_none() {
                        // Execution was started out of gps range
                        panic!("Could not determine gps");
                    } else {
                        // Execution _ended up_ out of gps range
                        println!("Out of GPS range");
                    }
                }
            },
            _ => panic!()
        }
        self.cur_step += 1;
    }
}



#[derive(Debug)]
pub struct RandomProgram {
    actions: [TurtleAction;67],
    drop: bool,
    only_move: bool,
    horizontal: bool
}
impl RandomProgram {
    pub fn new(enable_drop: bool, only_move: bool, horizontal: bool) -> Self {
        let actions = [
            turn::left(), turn::right(),
            go::forward(), go::backward(),  go::up(), go::down(), 
            dig::forward(), dig::up(), dig::down(),
            detect::forward(), detect::down(), detect::up(),
            place::forward(), place::down(), place::up(),
            drop::forward(), drop::down(), drop::up(),
            attack::forward(), attack::down(), attack::up(),
            suck::forward(), suck::down(), suck::up(),
            inspect::forward(), inspect::down(), inspect::up(),
            compare::forward(), compare::down(), compare::up(),
            inventory::select(1), inventory::select(2), inventory::select(3), inventory::select(4),
            inventory::select(5), inventory::select(6), inventory::select(7), inventory::select(8),
            inventory::select(9), inventory::select(10), inventory::select(11), inventory::select(12),
            inventory::select(13), inventory::select(14), inventory::select(15), inventory::select(16),
            inventory::compare_to(1), inventory::compare_to(2), inventory::compare_to(3), inventory::compare_to(16),
            inventory::count(1), inventory::count(2), inventory::count(3), inventory::count(16),
            inventory::detail(1), inventory::detail(2), inventory::detail(3), inventory::detail(16),
            inventory::space(1), inventory::space(2), inventory::space(3), inventory::space(16), 
            inventory::transfer_to(1), inventory::transfer_to(2),inventory::transfer_to(3), inventory::transfer_to(16),
            gps::locate()
            ];
        RandomProgram{actions:actions, drop:enable_drop, only_move: only_move, horizontal: horizontal}
    }
}


impl TurtleProgram for RandomProgram {
    fn next(&mut self) -> Result<TurtleAction> {
        let mut rng = rand::thread_rng();

        let action = self.actions.choose(&mut rng).unwrap();
        
        if self.only_move {
            match action {
                TurtleAction::Move{direction} if self.horizontal && matches!(direction, RelativeDirection::Up|RelativeDirection::Down) => self.next(),
                TurtleAction::Move{..}|
                TurtleAction::Turn{..}|
                TurtleAction::GpsLocate{..} => Ok(action.clone()),
                _ => self.next()
            }
        } else {
            if !self.drop {
                match action {
                    TurtleAction::Drop{..} => self.next(),
                    _ => Ok(action.clone())
                }
            } else {
                Ok(action.clone())
            }
        }
    }

    fn progress(&self) -> (u32, u32) {
        (0,1)
    }

    fn name(&self) -> &str {
        "random"
    }

    fn update(&mut self, _state: &TurtleState, _action: &TurtleAction, _result: &TurtleActionReturn) {
        // Don't care
    }
}

/// Program that simply executes the predetermined actions
#[derive(Debug)]
pub struct FromActionsProgram {
    actions: Vec<TurtleAction>,
    index: usize
}

impl FromActionsProgram {
    pub fn new(actions: Vec<TurtleAction>) -> Self {
        FromActionsProgram {
            actions,
            index: 0
        }
    }

    pub fn from(actions: &[TurtleAction]) -> Self {
        Self::new(Vec::from(actions))
    }
}



impl TurtleProgram for FromActionsProgram {


    fn next(&mut self) -> Result<TurtleAction> {
        if self.index < self.actions.len() {
            Ok(self.actions[self.index].clone())
        } else {
            Err(anyhow!("next() called when the program is finished!"))
        }
    }

    fn progress(&self) -> (u32, u32) {
        (self.index as u32, self.actions.len() as u32)
    }

    fn name(&self) -> &str {
        "fromactions"
    }

    fn update(&mut self, _state: &TurtleState, _action: &TurtleAction, _result: &TurtleActionReturn) {
        self.index += 1;
    }
}

pub struct MultiProgram {
    programs: VecDeque<Box<dyn TurtleProgram>>,
    current: Box<dyn TurtleProgram>
}

impl MultiProgram {
    pub fn new(first: Box<dyn TurtleProgram>) -> Self {

        MultiProgram {
            programs: VecDeque::new(),
            current: first
        }
    }

    pub fn add(&mut self, program: Box<dyn TurtleProgram>)  {
        self.programs.push_back(program);
    }
}

impl TurtleProgram for MultiProgram {
    fn next(&mut self) -> Result<TurtleAction> {
        let mut from_current = self.current.next();
        while let Err(_) = from_current {

            let next_program = self.programs.pop_front();
            if next_program.is_none() {
                return Err(anyhow!("Out of programs"));
            }
            self.current = next_program.unwrap();
            from_current = self.current.next();
        };
        from_current
    }

    fn progress(&self) -> (u32, u32) {
        let current_progress =  self.current.progress();
        (current_progress.0, current_progress.1 + self.programs.len() as u32)
    }

    fn name(&self) -> &str {
        "multiprogram"
    }

    fn update(&mut self, state: &TurtleState, action: &TurtleAction, result: &TurtleActionReturn) {
        // let program = match program_state(&self.current) {
        //     ProgramState::Finished => {
        //         self.programs.front_mut().unwrap_or(&mut self.current)
        //
        //     }
        //     _ => &mut self.current
        // };
        self.current.update(state, action, result);
        for program in self.programs.iter_mut() {
            program.update(state, action, result);
        }
    }
}

// A simple program whose task is to initialize GPS and to return to the position it was in
#[derive(Debug)]
pub struct InitGpsProgram {
    strategy: usize,
    step_in_strategy: i32,
    has_gps: bool
}

impl InitGpsProgram {

    pub fn new() -> Self {
        InitGpsProgram {
            strategy: 0,
            step_in_strategy: -1,
            has_gps: false
        }
    }

    fn get_cur_strategy(&self) -> Vec<TurtleAction> {
        match self.strategy {
            0 => vec!(go::forward(), gps::locate(), go::backward()),
            1 => vec!(go::backward(), gps::locate(), go::forward()),
            2 => vec!(turn::right(), go::forward(), gps::locate(), go::backward(), turn::left()),
            _ => panic!()
        }
    }
}

impl TurtleProgram for InitGpsProgram {


    fn next(&mut self) -> Result<TurtleAction> {
        let strategy = self.get_cur_strategy();
        if self.step_in_strategy == -1 {
            Ok(gps::locate())
        } else if self.step_in_strategy as usize == strategy.len() {
            Err(anyhow!(""))
        }else {
            Ok(strategy[self.step_in_strategy as usize].clone())
        }
    }

    fn progress(&self) -> (u32, u32) {
        if self.step_in_strategy as usize == self.get_cur_strategy().len() {
            (1, 1)
        } else {
            (0, 1)
        }
    }

    fn name(&self) -> &str {
        "initgps"
    }

    fn update(&mut self, _state: &TurtleState, action: &TurtleAction, result: &TurtleActionReturn) {
        // println!("{:?} - {:?}", action, result);
        match (action, result) {
            (TurtleAction::Move{..}, TurtleActionReturn::Failure(..)) => {
                self.strategy += 1;
                self.step_in_strategy = -1
            },
            (TurtleAction::GpsLocate {..}, TurtleActionReturn::Failure(..)) => {
                panic!("No gps!")
            },
            (_, _) => {
                self.step_in_strategy += 1;
            }
        }
    }
}



#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_rotate_program_start() {
        let start_args = vec!["rotate".to_string(), "-4".to_string()];
        // let start_msg = StartProgramMsg{msgtype:"start".to_string(), args:start_args};
        // let program = create_program(&start_msg);
        // assert!(program.is_ok());
        let program = RotateProgram::new(&start_args[1..]).unwrap();
        assert_eq!(4, program.steps);
        assert_eq!(RelativeDirection::Left, program.direction);

  
    }

    // #[test]
    // fn test_lua_table_parsing() {
    //     let msg = r#"{"msgtype":"response","result":{"n":2,"1":false,"2":"Movement obstructed"},"success":true}"#;
    //     let msg_value: TurtleResponseMsg = serde_json::from_str(msg).unwrap();
    //     let as_vec = packtable_to_vec(msg_value.result).unwrap_or_default();
    //     // assert_eq!()
    // }
}