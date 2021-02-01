use core::cmp::{max, min};
use std::collections::HashMap;
use std::io::prelude::*;

use anyhow::Result;

use crate::{turtle_action::*};
use crate::{turtle_rotation::*};
use crate::location_state::LocationState;
use crate::vec3::*;

// Guesses the state of turtle by the recorded executed commands.
pub type Coord = Vec3::<i32>;

#[derive(Clone)]
pub enum StateSerializationPolicy {
    /// Load state upon initialization, and save after each modification.
    /// String basedirectory,
    LoadAndSave{load_dir: String, save_dir: String},
    /// Load state upon initialization, never save
    LoadOnly{load_dir: String},
    /// Start from clean slate, and save after each modification
    SaveOnly{save_dir: String},
    /// Don't load anything, forget everything.
    None
}


pub struct TurtleState {
    pub location: LocationState,
    pub world: WorldState,
    pub history: ActionHistory
}



impl TurtleState {
    pub fn new(id: String, serialization: StateSerializationPolicy) -> Self {
        TurtleState{
            location: LocationState::new(),
            world: WorldState::new(id, serialization.clone()),
            history: ActionHistory::new()
        }
    }

    pub fn from(location: LocationState, world: WorldState) -> Self {
        TurtleState {
            location,
            world,
            history: ActionHistory::new()
        }
    }

    pub fn update(&mut self, action: &TurtleAction, result: &TurtleActionReturn) {
        self.location.update(action, result);
        self.world.update(action, result, &self.location);
        self.history.update(action, result);
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Unknown,
    Air,
    AirOrGravityBlock,
    Block
}

impl Block {
    pub fn to_ascii(&self) -> char {
        match self {
            Block::Unknown => ' ',
            Block::Air => '.',
            Block::Block => '█',
            Block::AirOrGravityBlock => '^'
        }
    }

    pub fn from(c: char) -> Self {
        match c {
            ' ' => Block::Unknown,
            '.' => Block::Air,
            '█' => Block::Block,
            '^' => Block::AirOrGravityBlock,
            _ => panic!()
        }
    }
}


pub struct WorldState {
    pub state: HashMap<Coord, Block>,
    id: String,
    ser_policy: StateSerializationPolicy
}

impl WorldState {
    pub fn is_obstructed(&self, coord: &Vec3<i32>) -> Option<bool> {
        if let Some(block) = self.state.get(&coord) {
            match block {
                Block::Unknown|
                Block::AirOrGravityBlock => Option::None,
                Block::Air => Some(false),
                Block::Block => Some(true)
            }
        } else {
            Option::None
        }
    }
}

pub fn dimensions<'a>(iter: impl Iterator<Item= &'a Coord>) -> (Coord, Coord) {
    let (mut x_min, mut x_max): (i32, i32) = (i32::MAX, i32::MIN);
    let (mut y_min, mut y_max): (i32, i32) = (i32::MAX, i32::MIN);
    let (mut z_min, mut z_max): (i32, i32) = (i32::MAX, i32::MIN);
    for coord in iter {
        x_min = min(coord.0, x_min);
        x_max = max(coord.0, x_max);
        y_min = min(coord.1, y_min);
        y_max = max(coord.1, y_max);
        z_min = min(coord.2, z_min);
        z_max = max(coord.2, z_max);
    }
    (Vec3::<i32>(x_min, y_min, z_min), Vec3::<i32>(x_max, y_max, z_max))
}

impl WorldState {
    pub fn new(id: String, ser_policy: StateSerializationPolicy) -> Self {
        let state = WorldState::deserialize_or_empty(&id, &ser_policy);
        
        WorldState {
            state,
            id,
            ser_policy
        }
    }


    fn state_filepath(dir: &str, id: &str, create: bool) -> String {
        let basedir = format!("{}/{}", dir, id);
        if create {
            std::fs::create_dir_all(&basedir).unwrap();
        }
        format!("{}/state.txt", &basedir)
    }

    pub fn update_all(&mut self, blocks: HashMap<Coord, Block>) {
        for (coord, block) in blocks {
            self.update_at(coord, block);
        }
    }

    fn try_serialize(&self)  {
        match &self.ser_policy {
            StateSerializationPolicy::LoadAndSave {  save_dir, .. }|
            StateSerializationPolicy::SaveOnly { save_dir } => {
                let path = WorldState::state_filepath(save_dir.as_str(), &self.id, true);
                self.serialize(path.as_str()).unwrap();
            }
            StateSerializationPolicy::LoadOnly {..}|
            StateSerializationPolicy::None => {}
        };
    }

    fn deserialize_or_empty(id: &str, ser_policy: &StateSerializationPolicy) -> HashMap<Coord, Block> {
        match ser_policy {
            StateSerializationPolicy::LoadAndSave { load_dir, ..}|
            StateSerializationPolicy::LoadOnly { load_dir} => {
                let state_result = deserialize_worldstate(load_dir, id);
                match state_result {
                    Ok(state) => state,
                    Err(_) => HashMap::new()
                }
            },
            StateSerializationPolicy::SaveOnly { .. }|
            StateSerializationPolicy::None => { HashMap::new()}
        }
    }

    fn serialize(&self, path: &str) -> Result<()> {
        // let path = WorldState::state_filepath(&self.id);
        let (minv, maxv) = dimensions(self.state.keys());
        
        let mut file = std::fs::File::create(path)?;
        write!(&mut file, "{}\n", 1)?;
        for y in minv.1..=maxv.1 {
            let start = Vec3::<i32>(minv.0, y, minv.2);
            let start_str = serde_json::to_string(&start)?;
            let end = Vec3::<i32>(maxv.0, y, maxv.2);
            let end_str = serde_json::to_string(&end)?;
            write!(&mut file, "{}\n{}\n", start_str, end_str)?;
            file.write_all(self.to_ascii(y).as_bytes())?;
        }
        Ok(())

    }

    fn update_at(&mut self, loc_absolute: Coord, block: Block) {
        let previous = self.state.insert(loc_absolute, block.clone());
        match previous {
            Some(ref oldblock) if oldblock==&block => {}, // do nothing
            _ => self.try_serialize()
        };
    }

    pub fn get(&self, loc_absolute: &Coord) -> Block {
        self.state.get(loc_absolute).unwrap_or(&Block::Unknown).clone()
    }

    fn is_solid_above(&self, loc: &Coord) -> bool {
        let above = loc + &AxisDirection::AD_YP;
        match self.state.get(&above) {
            Some(block) if matches!(block, Block::Unknown|Block::AirOrGravityBlock) => false, // don't know
            Some(block) if matches!(block, Block::Block) => true,
            Some(_) => {
                self.is_solid_above(&above)
            },
            None => false
        }
    }

    pub fn update(&mut self, action: &TurtleAction, result: &TurtleActionReturn, loc: &LocationState) {
        
        if let Some(loc_absolute) = loc.loc_absolute.clone() {
            match (action, result) {
                (TurtleAction::Move{direction}, TurtleActionReturn::Success) 
                    if matches!(direction, RelativeDirection::Forward|RelativeDirection::Backward|RelativeDirection::Up) => {
                        self.update_at(loc_absolute, Block::Air);
                        
                    }
                (TurtleAction::Move{direction}, TurtleActionReturn::Success)
                    if matches!(direction, RelativeDirection::Down) => {
                        let is_block_above = self.is_solid_above(&loc_absolute);
                        if is_block_above {
                            self.update_at(loc_absolute, Block::Air);
                        } else {
                            self.update_at(loc_absolute, Block::AirOrGravityBlock);
                        }
                    },
                (TurtleAction::Move{direction}, TurtleActionReturn::Failure(_reason)) => {
                    let unit_dir = loc.get_dest_direction_absolute(&direction).unwrap(); // has to exist since we are in absolute
                    let dest = &loc_absolute + &unit_dir;
                    self.update_at(dest, Block::Block);
                },
                (TurtleAction::Detect{direction}, TurtleActionReturn::Boolean(value)) => {
                    let dest_loc = loc.get_dest_position_absolute(direction);
                    let block = if *value {
                        Block::Block
                    } else {
                        Block::Air
                    };
                    self.update_at(dest_loc.unwrap(), block);
                },
                (TurtleAction::Inspect{direction}, TurtleActionReturn::Failure(FailureReason::NoBlockToInspect)) => {

                    let dest_loc = loc.get_dest_position_absolute(direction);
                    self.update_at(dest_loc.unwrap(), Block::Air);
                }
                _ => {}
            }
        }
    }

    pub fn to_ascii(&self, layer: i32) -> String {
        let mut result: String = String::new();
        let (minv, maxv) = dimensions(self.state.keys());

        for x in ((minv.0)..=(maxv.0)).rev() {
            for z in minv.2..=maxv.2 {
                let key  = Vec3::<i32>(x, layer, z);
                let value = self.state.get(&key);
                let block = match value {
                    Some(x) => x,
                    None => &Block::Unknown
                };
                let c = block.to_ascii();
                result.push(c);

            }
            result.push('\n');
        }
        result
    }


}


pub fn deserialize_worldstate(state_dir: &str, id: &str) -> Result<HashMap<Vec3<i32>, Block>> {
    let path = WorldState::state_filepath(state_dir, id, false);
    let mut result: HashMap<Vec3<i32>, Block> = HashMap::new();
    // let file = std::fs::File::open(path)?;
    println!("Opening path {}", &path);
    let contents = std::fs::read_to_string(&path)?;
    let lines: Vec<&str> = contents.split('\n').collect();
    let version = lines.first().expect("Illegal file");
    assert_eq!(&"1", version);
    let mut iter = lines[1..].iter();
    while let Some(mut line) = iter.next() {
        if line.trim() == "" {
            break;
        }
        // println!("Line: {}", line);
        let minv: Coord = serde_json::from_str(line)?;
        line = iter.next().unwrap();
        // println!("Line: {}", line);
        let maxv: Coord = serde_json::from_str(line)?;
        let y = minv.1;
        for x in (minv.0..=maxv.0).rev() {

            line = iter.next().unwrap();
            // println!("x={}, Line: {}", x, line);

            let mut citer = line.chars();
            for z in minv.2..=maxv.2 {

                let key = Vec3::<i32>(x, y, z);
                let val = citer.next()
                    .expect(format!("Coordinte {:?} did not exist in {}", &key, &path).as_str());
                result.insert(key, Block::from(val));
            }
        }
    }

    Ok(result)
}

pub struct ActionHistory {
    history: Vec<(TurtleAction, TurtleActionReturn)>
}

impl ActionHistory  {
    pub fn new() -> Self {
        ActionHistory {
            history: vec![]
        }
    }

    pub fn move_steps_len(&self) -> usize {
        self.history.iter().filter(|(action,_)| matches!(action, TurtleAction::Move{..}|TurtleAction::Turn{..})).count()
    }

    pub fn print_move_steps(&self) {
        let result = self.history.iter().map(|(x,_)| x).filter(|action| matches!(action, TurtleAction::Move{..}|TurtleAction::Turn{..}))
            .map(|action| format!("{:?}", action)).collect::<String>();
        println!("{}", result);
    }

    pub fn update(&mut self, action: &TurtleAction, response: &TurtleActionReturn) {
        self.history.push((action.clone(), response.clone()));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_world_state_loading() {
        WorldState::new("0".to_string(), StateSerializationPolicy::LoadOnly{load_dir:"state".to_string()});
        
    }
}