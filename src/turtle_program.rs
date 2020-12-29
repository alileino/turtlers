use crate::turtle_action::*;
use anyhow::{anyhow, Result, Error};
use serde_json;
use serde_derive::{Deserialize, Serialize};
extern crate rand;
use rand::{Rng, seq::SliceRandom, seq::SliceChooseIter};

pub trait TurtleProgram {
    // fn init(&mut self, start_args: serde_json::Value) -> Result<()>;
    fn next(&mut self) -> Result<TurtleAction>;
    fn progress(&self) -> (u32, u32); // Represents a fraction of progress
    fn name(&self) -> &str;
    fn update(&mut self, msg: &TurtleResponseMsg);
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

    fn update(&mut self, _msg: &TurtleResponseMsg)  {
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

    fn update(&mut self, _msg: &TurtleResponseMsg) {
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
            "random" => Box::new(RandomProgram::new()),
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
pub struct RandomProgram {
    actions: [TurtleAction;66]
}
impl RandomProgram {
    pub fn new() -> Self {
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
            inventory::transfer_to(1), inventory::transfer_to(2),inventory::transfer_to(3), inventory::transfer_to(16)
            ];
        RandomProgram{actions:actions}
    }
}


impl TurtleProgram for RandomProgram {
    fn next(&mut self) -> Result<TurtleAction> {
        let mut rng = rand::thread_rng();

        let action = self.actions.choose(&mut rng).unwrap();
        match action {
            TurtleAction::Drop{..} => self.next(),
            _ => Ok(action.clone())
        }
        // Ok(action.clone())

    }

    fn progress(&self) -> (u32, u32) {
        (0,1)
    }

    fn name(&self) -> &str {
        "random"
    }

    fn update(&mut self, _: &TurtleResponseMsg) {
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