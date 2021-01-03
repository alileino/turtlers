

use core::cmp::{min, max};
use std::collections::{HashMap};
use crate::{turtle_action::*};
use std::{ops::Index};
use crate::vec3::*;
use anyhow::{Result};
use std::io::prelude::*;
use crate::{turtle_rotation::*};
// Guesses the state of turtle by the recorded executed commands.
type Coord = Vec3::<i32>;

pub struct TurtleState {
    location: LocationState,
    world: WorldState
}

impl TurtleState {
    pub fn new(id: String) -> Self {
        TurtleState{
            location: LocationState::new(),
            world: WorldState::new(id)
        }
    }

    pub fn update(&mut self, action: &TurtleAction, result: &TurtleActionReturn) {
        self.location.update(action, result);
        self.world.update(action, result, &self.location);
    }
}
#[derive(Clone, PartialEq, Eq)]
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
    state: HashMap<Coord, Block>,
    id: String
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
    pub fn new(id: String) -> Self {
        let dest_path = WorldState::format_save_dir(id.as_str());
        std::fs::create_dir_all(dest_path).unwrap();
        let state_fromfile = WorldState::deserialize(id.as_str());
        let state = match state_fromfile {
            Ok(state) => state,
            Err(e) => panic!(format!("{}, {:?}", e, e.source())) //HashMap::new()
        };
        
        WorldState {
            state: state,
            id: id
        }
    }

    fn format_save_dir(id: &str) -> String {
        format!("state/{}", id)
    }

    fn state_filepath(id: &str) -> String {
        let dir = WorldState::format_save_dir(id);
        format!("{}/state.txt", dir)
    }



    fn serialize(&self) -> Result<()> {
        let path = WorldState::state_filepath(&self.id);
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
            _ => self.serialize().unwrap()
        };
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

    
    fn deserialize(id: &str) -> Result<HashMap<Vec3<i32>, Block>> {
        let path = WorldState::state_filepath(id);
        let mut result: HashMap<Vec3<i32>, Block> = HashMap::new();
        // let file = std::fs::File::open(path)?;
        println!("Opening path {}", path);
        let contents = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = contents.split('\n').collect();
        let version = lines.first().expect("Illegal file");
        assert_eq!(&"1", version);
        println!("{:?}", lines);
        let mut iter = lines[1..].iter();
        while let Some(mut line) = iter.next() {
            if line.trim() == "" {
                break;
            }
            println!("Line: {}", line);
            let minv: Coord = serde_json::from_str(line)?;
            line = iter.next().unwrap();
            println!("Line: {}", line);
            let maxv: Coord = serde_json::from_str(line)?;
            let y = minv.1;
            for x in (minv.0..=maxv.0).rev() {
                
                line = iter.next().unwrap();
                println!("x={}, Line: {}", x, line);
                
                let mut citer = line.chars();
                for z in minv.2..=maxv.2 {
                    let val = citer.next().unwrap();
                    let key = Vec3::<i32>(x, y, z);        
                    result.insert(key, Block::from(val));
                }
            }
        }


        Ok(result)
    }

}



//  Two different measurements guarantee the orientation of the state
#[derive(Debug, Clone)]
pub enum LocationMode {
    Relative(Option<(Coord, Coord)>), // relative pos1, absolute pos1
    Absolute((Coord, Rotation)) // difference of new relative pos2 and relative pos1, and same for absolute position
}

 

#[derive(Debug)]
pub struct LocationState {
    pub loc: Coord, // Relative location
    pub loc_absolute: Option<Coord>, // Absolute, requires two GPS measurements from different locations
    direction: AxisDirection,
    pub location_precision: LocationMode
}

impl LocationState {
    const DEFAULT_DIRECTION: AxisDirection = AxisDirection::Xp;
    pub fn new() -> Self {
        LocationState {
            loc: Vec3::zero(), 
            direction: LocationState::DEFAULT_DIRECTION, 
            loc_absolute: None,
            location_precision: LocationMode::Relative(None)
        }
    }

    fn update_gps(&mut self, loc: &Vec3<i32>) {

        match &self.location_precision {
            LocationMode::Relative(None) => {
                self.location_precision = LocationMode::Relative(Some((self.loc.clone(), loc.clone())));
            },
            LocationMode::Relative(Some(loc1)) => {
                let rel_diff = &self.loc-&loc1.0; // cur relative - old relative
                if rel_diff.0 == 0 && rel_diff.2 == 0 { // Can't determine rotation with no x or z offsets
                    return;
                }
                let abs_diff = loc-&loc1.1; // cur absolute - old absolute
                let rotation = Rotation::find_rotation(&rel_diff, &abs_diff);
                println!("Found rotation {:?} and offset {:?}", rotation, loc1.1);
                self.location_precision = LocationMode::Absolute((loc1.1.to_owned(), rotation));

            },
            LocationMode::Absolute(_) => {
                if self.loc_absolute.as_ref().unwrap()!= loc {
                    panic!("New gps measurement {:?} differs from calculated value of {:?}", loc, self.loc_absolute);
                }
            }
        }

    }

    fn update_absolute_location(&mut self) {
        if let LocationMode::Absolute((base, rot)) = &self.location_precision {
            
            let loc_wrot = rot.apply_to(&self.loc);
            let loc_woffset = &loc_wrot + base;
            self.loc_absolute = Some(loc_woffset);
        }
    }


    pub fn get_dest_direction_local(&self, move_direction: &RelativeDirection) -> Coord {
        get_dest_axisdirection(&self.direction, move_direction)
    }

    pub fn get_dest_direction_absolute(&self, move_direction: &RelativeDirection) -> Option<Coord> {
        let unit_dir = self.get_dest_direction_local(move_direction);
        if let LocationMode::Absolute((_base, rot)) = &self.location_precision {
            Some(rot.apply_to(&unit_dir))
        } else {
            None
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
            _ => panic!()
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut state = WorldState::new("0".to_string());
        
    }
}