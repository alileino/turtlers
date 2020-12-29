
use tungstenite::WebSocket;
// #[allow(unused_imports)]
// use std::{fs, io::prelude::*};
// use thiserror::Error;
#[allow(unused_imports)]
use anyhow::{anyhow, Result, Error};
use serde_json::{self, Value};
use serde_derive::{Deserialize, Serialize};
pub mod turtle_action;
pub mod turtle_state;
pub mod turtle_program;
pub mod vec3;
use turtle_action::*;
use turtle_program::*;
use std::{
    net::{TcpListener, TcpStream},
    thread::spawn
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

pub struct Turtle {
    id: String,
    program: Box<dyn TurtleProgram>,
    last_action: Option<TurtleAction>
}

enum ProgramState {
    Finished,
    _Waiting(f64), // waiting for turtle to report back
    HasInstructions(f64) // has instructions that can be delivered to turtle
}

impl Turtle {
    pub fn new(name: String) -> Self {
        Turtle {
            id: name, 
            program: Box::new(NoProgram{}),
            last_action: None
        }
    }

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

    fn record(&mut self, action: TurtleAction) {
        self.last_action = Some(action);
    }
}

#[derive(Serialize, Deserialize)]
struct InitMsg {
    id: String
}



#[derive(Serialize, Deserialize)]
struct UnknownMsg {
    msgtype: String
}

pub fn create_turtle(initialization_msg: &str) -> Result<Turtle> {
    let v: InitMsg = serde_json::from_str(initialization_msg)?;
    
    let turtle = Turtle::new(v.id);
    Ok(turtle)
    // Turtle {id}
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

pub enum FailureReason {
    MovementObstructed,
    NoBlockToInspect,
    NoItemsToPlace,
    CanNotPlaceItemHere,
    CanNotPlaceBlockHere,
    NothingToDigHere,
    NothingToAttackHere,
    NoItemsToTake,
    NoItemsToDrop,
}

pub fn parse_failure_reason(reason: &str) -> FailureReason {
    match reason {
        "Movement obstructed" => FailureReason::MovementObstructed,
        "No block to inspect" => FailureReason::NoBlockToInspect,
        "No items to place" => FailureReason::NoItemsToPlace,
        "Cannot place item here" => FailureReason::CanNotPlaceItemHere,
        "Cannot place block here" => FailureReason::CanNotPlaceBlockHere,
        "Nothing to dig here" => FailureReason::NothingToDigHere,
        "Nothing to attack here" => FailureReason::NothingToAttackHere,
        "No items to take" => FailureReason::NoItemsToTake,
        "No items to drop" => FailureReason::NoItemsToDrop,
        _ => panic!(format!("Unknown reason {}", reason))
    }
}

pub enum TurtleActionReturn {
    Success,
    Failure(FailureReason),
    InspectSuccess(String, serde_json::Map<String, Value>),
    Boolean(bool)
}

fn parse_response(turtle: &Turtle, response: &TurtleResponseMsg) -> Result<TurtleActionReturn> {
    let last_action = &turtle.last_action;
    let result = &response.result;
    match last_action {
        Some(action) => {
            match action {
                TurtleAction::Move{..}|
                TurtleAction::Turn{..}|
                TurtleAction::Place{..}|
                TurtleAction::Dig{..}|
                TurtleAction::Attack{..}|
                TurtleAction::Suck{..}|
                TurtleAction::Drop{..} => {
                    let success = result["1"].as_bool().unwrap();
                    if success {
                        Ok(TurtleActionReturn::Success)
                    } else {
                        let reason = parse_failure_reason(result["2"].as_str().unwrap());
                        
                        Ok(TurtleActionReturn::Failure(reason))
                    }
                },
                TurtleAction::Inspect{..} => {
                    let success = result["1"].as_bool().unwrap();
                    if success {
                        let inspect_result = &result["2"];
                        let block_name = inspect_result["name"].to_string();
                        let state_table = inspect_result["state"].as_object();
                        let final_table = match state_table {
                            Some(x) => x.to_owned(),
                            None => serde_json::Map::new()
                        };
                        
                        Ok(TurtleActionReturn::InspectSuccess(block_name, final_table))
                        
                    } else {
                        let reason = parse_failure_reason(result["2"].as_str().unwrap());
                        Ok(TurtleActionReturn::Failure(reason))
                    }
                },
                TurtleAction::Detect{..}|
                TurtleAction::Compare{..}|
                TurtleAction::Select{..} => {
                    let is_block = result["1"].as_bool().unwrap();
                    Ok(TurtleActionReturn::Boolean(is_block))
                },
                _ => panic!()
            }
        },
        None => panic!()
    }
}



fn execute_message(turtle: &mut Turtle, msg: &str)-> Result<String> { // later, be more specific
    // let msg_value: serde_json::Value = serde_json::from_str(msg)?;
    let msg_wtype: UnknownMsg = serde_json::from_str(msg)?;
    println!("{}", msg);
    

    match msg_wtype.msgtype.as_str() {
        "start" => {
            println!("{}", msg);
            let create_program_msg: StartProgramMsg = serde_json::from_str(msg)?;
            let program = create_program(&create_program_msg)?;
            turtle.set_program(program);

        },
        "response" => {
            let resp_msg: TurtleResponseMsg = serde_json::from_str(&msg)?;
            let resp = parse_response(&turtle, &resp_msg)?;
            // turtle.program.update(&resp_msg);
            
        },
        x => return Err(anyhow!("Invalid msgtype received when program is finished: {}", x))
    };
    
    let action = match turtle.program_state() {
        ProgramState::HasInstructions(_) => turtle.program.next()?,
        ProgramState::Finished => TurtleAction::Stop,
        _ => panic!()
    };
    let action_str = serde_json::to_string(&action.to_api_call())?;
    println!("{}", action_str);
    turtle.record(action);
    Ok(action_str)
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

}