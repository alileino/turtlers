

use core::cmp::{min, max};
use std::collections::{HashMap};
use crate::{turtle_action::*};
use std::{ops::Index};
use crate::vec3::*;
use anyhow::{Result};
use std::io::prelude::*;
use crate::{turtle_rotation::*};
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
    pub world: WorldState
}



impl TurtleState {
    pub fn new(id: String, serialization: StateSerializationPolicy) -> Self {
        TurtleState{
            location: LocationState::new(),
            world: WorldState::new(id, serialization.clone())
        }
    }

    pub fn from(location: LocationState, world: WorldState) -> Self {
        TurtleState {
            location,
            world
        }
    }

    pub fn update(&mut self, action: &TurtleAction, result: &TurtleActionReturn) {
        self.location.update(action, result);
        self.world.update(action, result, &self.location);
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

//  Two different measurements guarantee the orientation of the state
#[derive(Debug, Clone, PartialEq)]
pub enum LocationMode {
    Relative(Option<(Coord, Coord)>), // relative pos1, absolute pos1
    Absolute((Coord, Rotation)) // difference of new relative pos2 and relative pos1, and same for absolute position
}

 

#[derive(Debug, Clone)]
pub struct LocationState {
    pub loc: Coord, // Relative location
    pub loc_absolute: Option<Coord>, // Absolute, requires two GPS measurements from different locations
    pub direction: AxisDirection,
    pub direction_absolute: AxisDirection,
    pub location_precision: LocationMode
}

impl LocationState {
    pub const DEFAULT_DIRECTION: AxisDirection = AxisDirection::Xp;
    pub fn new() -> Self {
        LocationState {
            loc: Vec3::zero(), 
            direction: LocationState::DEFAULT_DIRECTION,
            direction_absolute: LocationState::DEFAULT_DIRECTION,
            loc_absolute: None,
            location_precision: LocationMode::Relative(None)
        }
    }

    fn update_gps(&mut self, new_absolute: &Vec3<i32>) {

        match &self.location_precision {
            LocationMode::Relative(None) => {
                self.location_precision = LocationMode::Relative(Some((self.loc.clone(), new_absolute.clone())));
            },
            LocationMode::Relative(Some(old_loc)) => {
                let (old_rel, old_abs) = old_loc;
                let rel_diff = &self.loc-&old_rel; // cur relative - old relative
                if rel_diff.0 == 0 && rel_diff.2 == 0 { // Can't determine rotation with no x or z offsets
                    return;
                }
                let abs_diff = new_absolute-&old_abs; // cur absolute - old absolute
                let rotation = Rotation::find_rotation(&rel_diff, &abs_diff);
                let new_offset = old_abs - &rotation.apply_to(old_rel);
                println!("Found rotation {:?} and offset {:?} from {:?} arriving at {:?}", rotation, new_offset, self.loc, new_absolute);
                println!("Relative coords: old: {:?} cur: {:?} diff: {:?}", old_loc.0, self.loc, rel_diff);
                println!("Absolute coords: old: {:?} cur: {:?} diff: {:?}", old_loc.1, new_absolute, abs_diff);
                self.location_precision = LocationMode::Absolute((new_offset, rotation));

                self.update_absolute_location();
                self.update_gps(&new_absolute);

            },
            LocationMode::Absolute(_) => {
                if self.loc_absolute.as_ref().unwrap() != new_absolute {
                    panic!("New gps measurement {:?} differs from calculated value of {:?}", new_absolute, self.loc_absolute);
                } else {
                    // println!("GPS {:?} == {:?}", new_absolute, self.loc_absolute.as_ref().unwrap());
                }
            }
        }

    }

    fn update_absolute_location(&mut self) {
        if let LocationMode::Absolute((base, rot)) = &self.location_precision {
            
            let loc_wrot = rot.apply_to(&self.loc);
            let loc_woffset = &loc_wrot + base;
            self.loc_absolute = Some(loc_woffset);
            self.direction_absolute = AxisDirection::from(&rot.apply_to(&self.direction.to_unit_vector()));
        }
    }


    pub fn get_dest_direction_local(&self, move_direction: &RelativeDirection) -> Coord {
        get_dest_axisdirection(&self.direction, move_direction)
    }

    pub fn get_dest_direction_absolute(&self, move_direction: &RelativeDirection) -> Option<Coord> {

        if self.loc_absolute.is_none() {
            None
        } else {
            let unit_dir = get_dest_axisdirection(&self.direction_absolute, move_direction);
            Some(unit_dir)

        }
        // let unit_dir = self.get_dest_direction_local(move_direction);
        // if let LocationMode::Absolute((_base, rot)) = &self.location_precision {
        //     Some(rot.apply_to(&unit_dir))
        // } else {
        //     None
        // }
    }

    pub fn get_dest_position_absolute(&self, move_direction: &RelativeDirection) -> Option<Coord> {
        if self.loc_absolute.is_none() {
            None
        } else {
            Some(self.loc_absolute.as_ref().unwrap() + self.get_dest_direction_absolute(move_direction).as_ref().unwrap())
        }
    }



    pub fn update(&mut self, action: &TurtleAction, result: &TurtleActionReturn) {
        match action {
            TurtleAction::Move {direction} => {
                if *result != TurtleActionReturn::Success {
                    return;
                }
                let unit_dir = self.get_dest_direction_local(&direction);
                self.loc += &unit_dir;

            },
            TurtleAction::Turn {direction} => {
                if *result != TurtleActionReturn::Success {
                    return;
                }
                let new_dir = match direction {
                    RelativeDirection::Left => self.direction.rotate_left(),
                    RelativeDirection::Right => self.direction.rotate_right(),
                    _ => panic!()
                };
                self.direction = new_dir;
            },
            TurtleAction::GpsLocate{..} => {
                if let TurtleActionReturn::Coordinate(location) = &*result {
                    self.update_gps(location);
                }
            }
            TurtleAction::Detect{..} => {}, // Does not affect movement
            TurtleAction::Inspect{..} => {},
            _ => todo!("Not implemented: {:?}", action)
        }
        self.update_absolute_location();
    }
}

impl Index<usize> for LocationState {
    type Output = i32;
    fn index<'a>(&'a self, i: usize) -> &'a i32 {
        &self.loc[i]
    }
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
        0
    }

    pub fn update(&mut self, action: &TurtleAction, response: &TurtleActionReturn) {
        self.history.push((action.clone(), response.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::ser::State;

    #[test]
    fn test_single_move() {
        let mut state =  LocationState::new();
    
        assert_eq!(AxisDirection::Xp, state.direction);
        let move_action = go::forward();
        state.update(&move_action, &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::new(1,0,0), state.loc);
        state.update(&go::backward(), &TurtleActionReturn::Success);
        assert_eq!(Coord::zero(), state.loc);
        state.update(&go::up(), &TurtleActionReturn::Success);
        assert_eq!(Coord::new(0,1,0), state.loc);
        state.update(&go::down(), &TurtleActionReturn::Success);
        assert_eq!(Coord::zero(), state.loc);
    }
    #[test]
    fn test_turn_move() {
        let mut state = LocationState::new();
        state.update(&turn::left(), &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::Zm, state.direction);
        state.update(&go::forward(), &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::AD_ZM, state.loc);
        state.update(&go::backward(), &TurtleActionReturn::Success);
        assert_eq!(Coord::zero(), state.loc);
        state.update(&turn::right(), &TurtleActionReturn::Success);
        state.update(&turn::right(), &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::Zp, state.direction);
        state.update(&go::forward(), &TurtleActionReturn::Success);
        assert_eq!(AxisDirection::AD_ZP, state.loc); 
    }

    #[test]
    fn test_circle_turns() {
        let mut state = LocationState::new();
        /*  Before each iteration      after
                21                      14
                34                      23
        */
        for _ in 0..4 {
            state.update(&turn::left(), &TurtleActionReturn::Success);
            state.update(&go::forward(), &TurtleActionReturn::Success);
        }
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::zero(), state.loc);

        for _ in 0..4 {
            state.update(&turn::right(), &TurtleActionReturn::Success);
            state.update(&go::forward(), &TurtleActionReturn::Success);
        }
        assert_eq!(AxisDirection::Xp, state.direction);
        assert_eq!(Coord::zero(), state.loc);

    }

    #[test]
    fn test_world_state_loading() {
        let mut state = WorldState::new("0".to_string(), StateSerializationPolicy::LoadOnly{load_dir:"state".to_string()});
        
    }
}