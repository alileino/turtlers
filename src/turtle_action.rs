use serde_derive::{Deserialize, Serialize};

/*
1. Generate as many actions as can be generated
2. Send actions
3. Notify how many were processed, which generates more actions


Actions could be enum variants. They can be converted to turtle calls, and they can have structure-like prperties
*/

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


#[derive(Serialize, Deserialize)]
pub struct TurtleApiCall {
    cmd: String,
    arg1: Option<String>
}

impl TurtleApiCall {
    fn new(cmd: &str) -> Self {
        TurtleApiCall{cmd: cmd.to_string(), arg1: None}
    }
    fn new_wargs(cmd: &str, arg1: String) -> Self {
        TurtleApiCall{cmd: cmd.to_string(), arg1: Some(arg1)}
    }
}

#[derive(Debug)]
pub enum TurtleAction {
    Turn {direction: RelativeDirection},
    Move {direction: RelativeDirection},
    Dig {direction: RelativeDirection},
    Detect {direction: RelativeDirection},
    Place {direction: RelativeDirection},
    Drop {direction: RelativeDirection},
    Attack {direction: RelativeDirection},
    Suck {direction: RelativeDirection},
    Inspect {direction: RelativeDirection},
    Compare {direction: RelativeDirection},
    Select {slot: u8}, // [1, 16]
    ItemCount {slot: u8},
    ItemSpace {slot: u8},
    ItemDetail {slot: u8}, // this also has a second parameter, "detailed", which supposedly is slower.
    TransferTo {slot: u8},
    CompareTo {slot: u8},
    Stop
}


impl TurtleAction {
    fn three_direction_call(name: &str, direction: &RelativeDirection) -> TurtleApiCall {
        let call = match direction {
            RelativeDirection::Forward => format!("turtle.{}", name),
            RelativeDirection::Up => format!("turtle.{}Up", name),
            RelativeDirection::Down => format!("turtle.{}Down", name),
            _ => panic!(format!("Unsupported {} direction {:?}", name, direction))
            };
        TurtleApiCall::new(call.as_str())
    }
    fn slot_call(name: &str, slot: &u8) -> TurtleApiCall {
        match slot {
            1..=16 => TurtleApiCall::new_wargs(format!("turtle.{}", name).as_str(), slot.to_string()),
            _ => panic!(format!("Slot index out of range: {}, should be [1, 16]", slot))
        }
    }
    
    pub fn to_api_call(&self) -> TurtleApiCall {
        match self {
            TurtleAction::Turn {direction} => {
            let call = match direction {
                    RelativeDirection::Right => "turtle.turnRight",
                    RelativeDirection::Left => "turtle.turnLeft",
                    _ => panic!(format!("Unsupported turn direction {:?}", direction))
                };
                TurtleApiCall::new(call)
            },
            TurtleAction::Move {direction } => {
                let call = match direction {
                    RelativeDirection::Forward => "turtle.forward",
                    RelativeDirection::Backward => "turtle.back",
                    RelativeDirection::Up => "turtle.up",
                    RelativeDirection::Down => "turtle.down",
                    _ => panic!(format!("Unsupported move direction {:?}", direction))
                };
                TurtleApiCall::new(call)
            },
            TurtleAction::Dig {direction} =>
                TurtleAction::three_direction_call("dig", direction),
            TurtleAction::Detect {direction} => 
                TurtleAction::three_direction_call("detect", direction),
            TurtleAction::Place {direction} => 
                TurtleAction::three_direction_call("place", direction),
            TurtleAction::Drop {direction} => 
                TurtleAction::three_direction_call("drop", direction),
            TurtleAction::Attack {direction} => 
                TurtleAction::three_direction_call("attack", direction),
            TurtleAction::Suck {direction} => 
                TurtleAction::three_direction_call("suck", direction),
            TurtleAction::Inspect {direction } =>
                TurtleAction::three_direction_call("inspect", direction),
            TurtleAction::Compare {direction } =>
                TurtleAction::three_direction_call("compare", direction),
            TurtleAction::Select {slot } => TurtleAction::slot_call("select", slot),
            TurtleAction::ItemCount {slot } => TurtleAction::slot_call("getItemCount", slot),
            TurtleAction::ItemSpace {slot } => TurtleAction::slot_call("getItemSpace", slot),
            TurtleAction::ItemDetail {slot } => TurtleAction::slot_call("getItemDetail", slot),
            TurtleAction::TransferTo {slot } => TurtleAction::slot_call("transferTo", slot),
            TurtleAction::CompareTo {slot } => TurtleAction::slot_call("compareTo", slot),
            TurtleAction::Stop => TurtleApiCall::new("stop")
        }
    }
}