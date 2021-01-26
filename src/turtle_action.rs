use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use crate::{turtle_rotation::*};
use crate::{vec3::Vec3};
/*
1. Generate as many actions as can be generated
2. Send actions
3. Notify how many were processed, which generates more actions


Actions could be enum variants. They can be converted to turtle calls, and they can have structure-like prperties
*/

trait TurtleActionSerializable {
    fn to_api_call(&self) -> serde_json::Value;
}


#[derive(Serialize, Deserialize)]
pub struct TurtleApiCall {
    cmd: String,
    arg1: serde_json::Value,
    arg2: serde_json::Value
}

impl TurtleApiCall {
    fn new(cmd: &str) -> Self {
        TurtleApiCall{cmd: cmd.to_string(), arg1: Value::Null, arg2: Value::Null}
    }

    fn new_wargs(cmd: &str, arg1: Value, arg2: Value) -> Self {
        TurtleApiCall{cmd: cmd.to_string(), arg1: arg1, arg2: arg2}
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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
    GpsLocate {timeout_ms: u32, debug: bool},
    Stop
}


// impl TurtleAction {
//     pub fn to_symbol(&self) -> char {
//         match self {
//             TurtleAction::Move{direction: RelativeDirection::Forward} => '\u{2190}',
//             TurtleAction::Move{direction: RelativeDirection::Backward} => '\u{2190}',
//             TurtleAction::Move{direction: RelativeDirection::Up} => '\u{2190}',
//             _ => todo!()
//         }
//
//     }
// }


pub mod go {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Move{direction:RelativeDirection::Forward}}
    pub const fn backward() -> TurtleAction {TurtleAction::Move{direction:RelativeDirection::Backward}}
    pub const fn up() -> TurtleAction {TurtleAction::Move{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Move{direction:RelativeDirection::Down}}
}

pub mod turn {
    use super::*;
    pub const fn left() -> TurtleAction {TurtleAction::Turn{direction:RelativeDirection::Left}}
    pub const fn right() -> TurtleAction {TurtleAction::Turn{direction:RelativeDirection::Right}}
}

pub mod dig {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Dig{direction:RelativeDirection::Forward}}
    pub const fn up() -> TurtleAction {TurtleAction::Dig{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Dig{direction:RelativeDirection::Down}}
}

pub mod inspect {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Inspect{direction:RelativeDirection::Forward}}
    pub const fn up() -> TurtleAction {TurtleAction::Inspect{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Inspect{direction:RelativeDirection::Down}}
}

pub mod detect {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Detect{direction:RelativeDirection::Forward}}
    pub const fn up() -> TurtleAction {TurtleAction::Detect{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Detect{direction:RelativeDirection::Down}}
}

pub mod place {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Place{direction:RelativeDirection::Forward}}
    pub const fn up() -> TurtleAction {TurtleAction::Place{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Place{direction:RelativeDirection::Down}}
}

pub mod compare {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Compare{direction:RelativeDirection::Forward}}
    pub const fn up() -> TurtleAction {TurtleAction::Compare{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Compare{direction:RelativeDirection::Down}}
}

pub mod attack {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Attack{direction:RelativeDirection::Forward}}
    pub const fn up() -> TurtleAction {TurtleAction::Attack{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Attack{direction:RelativeDirection::Down}}
}

pub mod suck {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Suck{direction:RelativeDirection::Forward}}
    pub const fn up() -> TurtleAction {TurtleAction::Suck{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Suck{direction:RelativeDirection::Down}}
}

pub mod drop {
    use super::*;
    pub const fn forward() -> TurtleAction {TurtleAction::Drop{direction:RelativeDirection::Forward}}
    pub const fn up() -> TurtleAction {TurtleAction::Drop{direction:RelativeDirection::Up}}
    pub const fn down() -> TurtleAction {TurtleAction::Drop{direction:RelativeDirection::Down}}
}
pub mod inventory {
    use super::*;
    pub const fn select(slot: u8) -> TurtleAction {
        TurtleAction::Select{slot }
    }

    pub const fn count(slot: u8) -> TurtleAction {
        TurtleAction::ItemCount{slot }
    }

    pub const fn space(slot: u8) -> TurtleAction {
        TurtleAction::ItemSpace{slot }
    }

    pub const fn detail(slot: u8) -> TurtleAction {
        TurtleAction::ItemDetail{slot }
    }

    pub const fn transfer_to(slot: u8) -> TurtleAction {
        TurtleAction::TransferTo{slot }
    }

    pub const fn compare_to(slot: u8) -> TurtleAction {
        TurtleAction::CompareTo{slot }
    }
}

pub mod gps {
    use super::*;
    pub const fn locate() -> TurtleAction {
        TurtleAction::GpsLocate{timeout_ms: 2000u32, debug:false}
    }
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
            1..=16 => TurtleApiCall::new_wargs(format!("turtle.{}", name).as_str(), Value::from(*slot), Value::Null),
            _ => panic!(format!("Slot index out of range: {}, should be [1, 16]", slot))
        }
    }

    fn gps_call(timeout_ms: &u32, debug: &bool) -> TurtleApiCall {
        TurtleApiCall::new_wargs("gps.locate", Value::from((*timeout_ms as f64) / 1000f64), Value::from(*debug))
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
            TurtleAction::Stop => TurtleApiCall::new("stop"),
            TurtleAction::GpsLocate {timeout_ms, debug} => TurtleAction::gps_call(timeout_ms, debug)
        }
    }
}


#[derive(PartialEq, Debug, Clone)]
pub enum FailureReason {
    MovementObstructed, // move
    NoBlockToInspect, // inspect
    NoItemsToPlace, // place
    CanNotPlaceItemHere, // place
    CanNotPlaceBlockHere, // place
    NothingToDigHere,   // dig
    NothingToAttackHere, // attack
    NoItemsToTake, // suck
    NoItemsToDrop, // drop
    SlotIsEmpty, // itemDetail
    NoSpaceForItems, // transferTo
    UnbreakableBlockDetected, // dig
    GpsLocateFailure
}

#[derive(PartialEq, Debug, Clone)]
pub enum TurtleActionReturn {
    Success,
    Failure(FailureReason),
    InspectSuccess(String, serde_json::Map<String, Value>),
    DetailSuccess(serde_json::Map<String, Value>),
    Boolean(bool),
    Number(u32),
    Coordinate(Vec3<i32>)
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
        "No space for items" => FailureReason::NoSpaceForItems,
        "Unbreakable block detected" => FailureReason::UnbreakableBlockDetected,
        _ => panic!(format!("Unknown reason {}", reason))
    }
}