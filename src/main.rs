
use tungstenite::WebSocket;
use anyhow::{anyhow, Result};
use serde_json::{self};
use serde_derive::{Deserialize, Serialize};
use turtlers::turtle_action::*;
use turtlers::turtle_program::*;
use turtlers::turtle::*;
use turtlers::vec3::Vec3;
use std::{net::{TcpListener, TcpStream}, thread::{self, spawn}, time};

use tungstenite::{accept, handshake::HandshakeRole, HandshakeError, Message};
use tungstenite as tung;

fn must_not_block<Role: HandshakeRole>(err: HandshakeError<Role>) -> tung::Error {
    match err {
        HandshakeError::Interrupted(_) => panic!("Bug: blocking socket would block"),
        HandshakeError::Failure(f) => f
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


fn parse_response(turtle: &Turtle, response: &TurtleResponseMsg) -> Result<TurtleActionReturn> {
    let last_action = turtle.last_action.as_ref().unwrap(); // If None, our program is initializing out of order
    let result = &response.result;

    match last_action {
        TurtleAction::Move{..}|
        TurtleAction::Turn{..}|
        TurtleAction::Place{..}|
        TurtleAction::Dig{..}|
        TurtleAction::Attack{..}|
        TurtleAction::Suck{..}|
        TurtleAction::Drop{..}|
        TurtleAction::TransferTo{..} => {
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
        TurtleAction::ItemDetail{..} => {
            let detail_result = result["1"].as_object();
            match detail_result {
                Some(x) => Ok(TurtleActionReturn::DetailSuccess(x.to_owned())),
                None => Ok(TurtleActionReturn::Failure(FailureReason::SlotIsEmpty))
            }
        },
        TurtleAction::Detect{..}|
        TurtleAction::Compare{..}|
        TurtleAction::Select{..}|
        TurtleAction::CompareTo{..} => {
            let is_block = result["1"].as_bool().unwrap();
            Ok(TurtleActionReturn::Boolean(is_block))
        },
        TurtleAction::ItemCount{..}|
        TurtleAction::ItemSpace{..} => {
            let num = result["1"].as_u64().unwrap();
            Ok(TurtleActionReturn::Number(num as u32))
        },
        TurtleAction::GpsLocate{timeout_ms: _, debug} => {
            if *debug {
                panic!();
            } else {
                if result["n"].as_u64().unwrap() == 1 {
                    Ok(TurtleActionReturn::Failure(FailureReason::GpsLocateFailure))
                } else {
                    let x = result["1"].as_i64().unwrap();
                    let y = result["2"].as_i64().unwrap();
                    let z = result["3"].as_i64().unwrap();
                    Ok(TurtleActionReturn::Coordinate(Vec3::<i32>(x as i32, y as i32, z as i32)))
                }
            }
        }

        _ => panic!()
    }

}


fn execute_message(turtle: &mut Turtle, msg: &str) -> Result<()> { // later, be more specific

    let msg_wtype: UnknownMsg = serde_json::from_str(msg)?;

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
            turtle.update(&resp);
            
        },
        x => return Err(anyhow!("Invalid msgtype received when program is finished: {}", x))
    };
    Ok(())
}

fn next_response(turtle: &mut Turtle) -> Result<String> {
    let action = turtle.next()?;
    let action_str = serde_json::to_string(&action.to_api_call())?;

    Ok(action_str)
}

fn handle_client_loop(mut socket: &mut WebSocket<TcpStream>, initialization_msg: &str) -> Result<()> {
    println!("Received initialization msg {}", initialization_msg);
    let mut turtle = create_turtle(&initialization_msg)?;
    println!("Successfully initialized turtle {}", turtle.id);
    turtle_ok(&mut socket)?;
    loop {
        match socket.read_message()? {
            Message::Text(x) => {
                execute_message(&mut turtle, &x.as_str())?;
                let response = next_response(&mut turtle)?;
                socket.write_message(Message::Text(response))?;
            },
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) => {
                println!("Ping/pong/close")
            },
            Message::Binary(_) => panic!("Binary message received!")
        }
        thread::sleep(time::Duration::from_millis(1));
    }
}

fn accept_client(stream: TcpStream) -> Result<()> {
    let mut socket = accept(stream).map_err(must_not_block)?;
    println!("Waiting for initialization");
    match socket.read_message()? {
        Message::Text(x) => {
            handle_client_loop(&mut socket, x.as_str())
        },
        _ => {
            Err(anyhow!("Invalid handshake"))
        }
    }

}


fn main() {
    
    let listener = TcpListener::bind("25.75.103.40:80").unwrap();

    for stream in listener.incoming() {
        let h = spawn(move || match stream {
            Ok(stream) => {
                if let Err(err) = accept_client(stream) {
                    match err {
                        e => println!("Client error: {}", e),
                    }
                }
            },
            Err(ref e) => println!("Error accepting stream: {}", e),
        });
        match h.join() {
            Ok(_) => {},
            Err(_) => return
        }
    }
}



#[cfg(test)]
mod test {

}