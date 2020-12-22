
use tungstenite::WebSocket;
// #[allow(unused_imports)]
// use std::{fs, io::prelude::*};
// use thiserror::Error;
#[allow(unused_imports)]
use anyhow::{anyhow, Result, Error};
use serde_json;
use serde_derive::{Deserialize, Serialize};

use std::{
    net::{TcpListener, TcpStream},
    thread::spawn,
};

use tungstenite::{accept, handshake::HandshakeRole, HandshakeError, Message};
use tungstenite as tung;
// #[derive(Error, Debug)]
// pub enum MyErrors {
//     #[error("unknown")]
//     Unknown,
// }

// #[macro_export]
// macro_rules! Err {
//     ($err:expr $(,)?) => {{
//         let error = $err;
//         Err(anyhow::anyhow!(error))
//     }};
// }

// macro_rules! unwrap_or_error {
//     ( $e:expr ) => {
//         match $e {
//             Ok(x) => x,
//             Err(_) => Err(_),
//         }
//     }
// }

fn must_not_block<Role: HandshakeRole>(err: HandshakeError<Role>) -> tung::Error {
    match err {
        HandshakeError::Interrupted(_) => panic!("Bug: blocking socket would block"),
        HandshakeError::Failure(f) => f
    }
}

trait TurtleProgram {
    // fn init(&mut self, start_args: serde_json::Value) -> Result<()>;
    fn next(&mut self) -> Result<TurtleAction>;
    fn progress(&self) -> (u32, u32); // Represents a fraction of progress
    fn name(&self) -> &str;
    fn update(&mut self, msg: &TurtleResponseMsg);
}

trait TurtleActionSerializable {
    fn to_api_call(&self) -> serde_json::Value;
}

#[derive(Debug)]
pub enum RelativeDirection {
    Forward,
    Backward,
    Right,
    Left,
    Down,
    Up
}

// #[derive(Debug)]
// enum TurtleAction {
//     Rotate(RelativeDirection)
// }

#[derive(Debug)]
struct RotateProgram {
    steps: u32,
    steps_remaining: u32,
    direction: RelativeDirection
    // actions_remaining: Vec<TurtleAction>
}

impl RotateProgram {
    fn new(start_args: &serde_json::Value) -> Self {
        let steps_i32 = start_args["steps"].as_u64().unwrap();
        let direction = match steps_i32 > 0 {
            true => RelativeDirection::Right,
            false => RelativeDirection::Left
        };
        RotateProgram {steps: steps_i32 as u32, steps_remaining:steps_i32 as u32, direction: direction}
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
        Ok(TurtleAction::Turn{direction: RelativeDirection::Left})
    }

    fn update(&mut self, msg: &TurtleResponseMsg)  {
        self.steps_remaining -= 1;
    }
}

/*
1. Generate as many actions as can be generated
2. Send actions
3. Notify how many were processed, which generates more actions


Actions could be enum variants. They can be converted to turtle calls, and they can have structure-like prperties
*/
#[derive(Serialize, Deserialize)]
struct TurtleApiCall {
    cmd: String
}

impl TurtleApiCall {
    fn new(cmd: &str) -> Self {
        TurtleApiCall{cmd: cmd.to_string()}
    }
}

#[derive(Debug)]
enum TurtleAction {
    Turn {direction: RelativeDirection},
    Move {direction: RelativeDirection},
    Stop
}


impl TurtleAction {
    fn to_api_call(&self) -> TurtleApiCall {
        
        match self {
            TurtleAction::Turn {direction} => {
            let call = match direction {
                    RelativeDirection::Right => "turtle.turnRight",
                    RelativeDirection::Left => "turtle.turnLeft",
                    x => panic!(format!("Unsupported api call {:?}", x))
                };
                TurtleApiCall::new(call)
            },
            TurtleAction::Move {direction} => {

                TurtleApiCall::new("")
            },
            TurtleAction::Stop =>  TurtleApiCall::new("stop")

        }
    }
}

struct NoProgram {}
impl TurtleProgram for NoProgram {

    // fn init(&mut self, _: serde_json::Value) -> Result<()> {
    //     Err(anyhow!("Can't initialize NoProgram"))
    // }

    fn progress(&self) -> (u32, u32) {
        (1,1)
    }

    fn name(&self) -> &str {"noprogram"}

    fn next(&mut self) -> Result<(TurtleAction)> {
        Err(anyhow!("Can't initialize NoProgram"))
    }

    fn update(&mut self, msg: &TurtleResponseMsg) {
        todo!()
    }
}


pub struct Turtle {
    id: String,
    program: Box<dyn TurtleProgram>
}

enum ProgramState {
    Finished,
    Waiting(f64), // waiting for turtle to report back
    HasInstructions(f64) // has instructions that can be delivered to turtle
}

impl Turtle {
    fn program_state(&self) -> ProgramState {
        let progress = self.program.progress();
        if progress.0 == progress.1 {
            return ProgramState::Finished;
        }
        let progress_f = (progress.1 as f64)/(progress.0 as f64);
        ProgramState::HasInstructions(progress_f)
    }

    fn set_program(&mut self, program: Box<dyn TurtleProgram>) {
        self.program = program;
        // self.program.start();
        println!("Set program to {}", self.program.name());
    }
}

#[derive(Serialize, Deserialize)]
struct InitMsg {
    id: String
}

#[derive(Serialize, Deserialize)]
struct StartProgramMsg {
    msgtype: String,
    program: String,
    args: serde_json::Value
}

// Message that the lua client sends in response to a command it just tried to execute
#[derive(Serialize, Deserialize)]
struct TurtleResponseMsg {
    msgtype: String,
    success: bool, // was the command run
    result: serde_json::Value // the return value
}

#[derive(Serialize, Deserialize)]
struct UnknownMsg {
    msgtype: String
}

pub fn create_turtle(initialization_msg: &str) -> Result<Turtle> {
    let v: InitMsg = serde_json::from_str(initialization_msg)?;
    
    let turtle = Turtle{id: v.id, program: Box::new(NoProgram{})};
    Ok(turtle)
    // Turtle {id}
}

fn create_program(msg: &StartProgramMsg ) -> Result<Box<dyn TurtleProgram>> {
    match msg.program.as_str() {
        "rotate" => Ok(Box::new(RotateProgram::new(&msg.args))),
        program => Err(anyhow!("Invalid program: {}", program))
    }
    
}



#[derive(Serialize)]
struct TurtleResult {
    result: String
}

fn turtle_ok(socket: &mut WebSocket<TcpStream>) -> Result<()> {
    let value = TurtleResult{result:"ok".to_string()};
    let contents = serde_json::to_string(&value)?;
    socket.write_message(Message::Text(contents))?;
    Ok(())
}



fn execute_message(turtle: &mut Turtle, msg: &str)-> Result<String> { // later, be more specific
    // let msg_value: serde_json::Value = serde_json::from_str(msg)?;
    let msg_wtype: UnknownMsg = serde_json::from_str(msg)?;

    

    match msg_wtype.msgtype.as_str() {
        "start" => {
            let create_program_msg: StartProgramMsg = serde_json::from_str(msg)?;
            let program = create_program(&create_program_msg)?;
            turtle.set_program(program);
            let action = turtle.program.next()?;
            let action_str = serde_json::to_string(&action.to_api_call())?;
            Ok(action_str)
        },
        "response" => {
            let resp_msg: TurtleResponseMsg = serde_json::from_str(&msg)?;
            turtle.program.update(&resp_msg);
            
            let action = match turtle.program_state() {
                ProgramState::HasInstructions(_) => turtle.program.next()?,
                ProgramState::Finished => TurtleAction::Stop,
                _ => panic!()
            };
            let action_str = serde_json::to_string(&action.to_api_call())?;
            Ok(action_str)

        },
        x => Err(anyhow!("Invalid msgtype received when program is finished: {}", x))
    }
    // let program = create_program(&msg_value)?;
    


    // let command: String = msg_value["command"].to_string();
    
    // match command {

    //     _ => Err(anyhow!("Invalid command: {}", command))
    // }
}

fn do_client_stuff(mut socket: &mut WebSocket<TcpStream>, initialization_msg: &str) -> Result<()> {
    println!("Received initialization msg {}", initialization_msg);
    let mut turtle = create_turtle(&initialization_msg)?;
    println!("Successfully initialized turtle {}", turtle.id);
    turtle_ok(&mut socket)?;
    loop {
        match socket.read_message()? {
            Message::Text(x) => {
                let response = execute_message(&mut turtle, &x.as_str())?;
                socket.write_message(Message::Text(response))?;
            },
            msg @ Message::Binary(_) => {
                socket.write_message(msg)?;
            },
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                println!("Ping/pong/close")
            }
        }
    }
}

fn handle_client(stream: TcpStream) -> Result<()> {
    let mut socket = accept(stream).map_err(must_not_block)?;
    println!("Waiting for initialization");
    match socket.read_message()? {
        Message::Text(x) => {
            do_client_stuff(&mut socket, x.as_str())
        },
        _ => {
            Err(anyhow!("Invalid handshake"))
        }
    }

}

fn main() {
    
    let listener = TcpListener::bind("25.75.103.40:80").unwrap();
    for stream in listener.incoming() {
        spawn(move || match stream {
            Ok(stream) => {
                if let Err(err) = handle_client(stream) {
                    match err {
                        e => println!("Client error: {}", e),
                    }
                }
            }
            Err(e) => println!("Error accepting stream: {}", e),
        });
    }

}



#[cfg(test)]
mod test {
    use super::*;
   
    #[test]
    fn ensure_time_within_tolerance()  {

    }
}