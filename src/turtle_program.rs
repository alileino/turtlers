use crate::{turtle_action::*, turtle_state::LocationState};
use anyhow::{anyhow, Result};
use serde_json;
use serde_derive::{Deserialize, Serialize};
extern crate rand;
use rand::{seq::SliceRandom};

pub trait TurtleProgram {
    fn next(&mut self) -> Result<TurtleAction>;
    fn progress(&self) -> (u32, u32); // Represents a fraction of progress
    fn name(&self) -> &str;
    fn update(&mut self, result: &TurtleActionReturn, action: &TurtleAction);
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

    fn update(&mut self, _result: &TurtleActionReturn, _action: &TurtleAction)  {
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

    fn update(&mut self, _result: &TurtleActionReturn, _action: &TurtleAction) {
        todo!()
    }
}

pub fn create_program(msg: &StartProgramMsg ) -> Result<Box<dyn TurtleProgram>> {
    let program_name = &msg.args[0];
    let args = &msg.args[1..];
    let boxed: Box<dyn TurtleProgram> = 
        match program_name.as_str() {
            "rotate" => Box::new(RotateProgram::new(args)?),
            "no" => Box::new(NoProgram{}),
            "random" => Box::new(RandomProgram::new(false, false)),
            "locatetest" => Box::new(LocationTestProgram::new()),
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
pub struct LocationTestProgram {
    state: LocationState,
    gps_initialized: bool,
    random: RandomProgram,
    cur_step: u32
}

impl LocationTestProgram {
    pub fn new() -> Self {
        let random = RandomProgram::new(false,true);
        let state = LocationState::new();
        LocationTestProgram{state: state, random:random, gps_initialized:false, cur_step:0}
    }
}

impl TurtleProgram for LocationTestProgram {
    fn next(&mut self) -> Result<TurtleAction> {
        if self.cur_step % 2 == 0 {
            Ok(gps::locate())
        } else {
            self.random.next()
        }
    }

    fn progress(&self) -> (u32, u32) {
        (0,1)
    }

    fn name(&self) -> &str {
        "locationtest"
    }

    fn update(&mut self, result: &TurtleActionReturn, action: &TurtleAction) {
        match action {
            TurtleAction::Move{..}|
            TurtleAction::Turn{..} => {
                self.state.update(action, result);
            },
            TurtleAction::GpsLocate{..} => {
                if let TurtleActionReturn::Coordinate(_) = &*result {
                    self.state.update(action, result);
                } else {
                    if self.state.loc_absolute.is_none() {
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
    only_move: bool
}
impl RandomProgram {
    pub fn new(enable_drop: bool, only_move: bool) -> Self {
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
        RandomProgram{actions:actions, drop:enable_drop, only_move: only_move}
    }
}


impl TurtleProgram for RandomProgram {
    fn next(&mut self) -> Result<TurtleAction> {
        let mut rng = rand::thread_rng();

        let action = self.actions.choose(&mut rng).unwrap();

        if self.only_move {
            match action {
                TurtleAction::Move{..}|
                TurtleAction::Turn{..} => Ok(action.clone()),
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

    fn update(&mut self, _result: &TurtleActionReturn, _action: &TurtleAction) {
        // Don't care
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