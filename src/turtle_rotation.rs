use crate::{vec3::Vec3};
type Coord = Vec3::<i32>;


#[derive(Debug, PartialEq, Clone)]
pub enum RelativeDirection {
    Forward,
    Backward,
    Right,
    Left,
    Down,
    Up
}

#[derive(PartialEq, Debug, Eq, Hash)]
pub enum AxisDirection {
    Xp,
    Xm,
    Zp,
    Zm
}

/*
    Xp
Zm      Zp
    Xm

*/
#[derive(Debug, Clone)]
pub enum Rotation {
    Y0,
    Y90,
    Y180,
    Y270
}

impl Rotation {
    
    const ALL: [Rotation;4] = [Rotation::Y0, Rotation::Y90, Rotation::Y180, Rotation::Y270];

    const ROT_0: (Coord, Coord, Coord) = (Vec3::<i32>(1,0,0), Vec3::<i32>(0,1,0), Vec3::<i32>(0,0,1));
    const ROT_Y90: (Coord, Coord, Coord) = (Vec3::<i32>(0,0,1), Vec3::<i32>(0,1,0), Vec3::<i32>(-1,0,0));
    const ROT_Y180: (Coord, Coord, Coord) = (Vec3::<i32>(-1, 0, 0), Vec3::<i32>(0,1,0), Vec3::<i32>(0,0,-1));
    const ROT_Y270: (Coord, Coord, Coord) = (Vec3::<i32>(0, 0, -1), Vec3::<i32>(0,1,0), Vec3::<i32>(1,0,0));

    pub fn apply_to(&self, vec: &Coord) -> Coord {
        let (x, _y, z) = match self {
            Rotation::Y0 => Rotation::ROT_0,
            Rotation::Y90 => Rotation::ROT_Y90,
            Rotation::Y180 => Rotation::ROT_Y180,
            Rotation::Y270 => Rotation::ROT_Y270
        };
        
        Vec3::<i32>(x.0*vec.0 + x.2*vec.2, vec.1, z.0*vec.0 + z.2*vec.2)
    }

    pub fn find_rotation(src: &Coord, dst: &Coord) -> Self {
        for rot in Rotation::ALL.iter() {
            if &rot.apply_to(src) == dst {
                return rot.clone();
            }
        }
        panic!()
    }
}

impl AxisDirection {
    pub const AD_XP: Vec3<i32> = Vec3::<i32>(1,0,0);
    pub const AD_XM: Vec3<i32> = Vec3::<i32>(-1,0,0);
    pub const AD_ZP: Vec3<i32> = Vec3::<i32>(0,0,1);
    pub const AD_ZM: Vec3<i32> = Vec3::<i32>(0,0,-1);
    pub const AD_YP: Vec3<i32> = Vec3::<i32>(0,1,0);
    pub const AD_YM: Vec3<i32> = Vec3::<i32>(0,-1,0);

    pub fn to_unit_vector(&self) -> Vec3<i32> {
        match self {
            AxisDirection::Xp => AxisDirection::AD_XP,
            AxisDirection::Xm => AxisDirection::AD_XM,
            AxisDirection::Zp => AxisDirection::AD_ZP,
            AxisDirection::Zm => AxisDirection::AD_ZM
        }
    }

    pub fn rotate_right(&self) -> AxisDirection {
        match self {
            AxisDirection::Xp => AxisDirection::Zp,
            AxisDirection::Zp => AxisDirection::Xm,
            AxisDirection::Xm => AxisDirection::Zm,
            AxisDirection::Zm => AxisDirection::Xp
        }
    }

    pub fn rotate_left(&self) -> AxisDirection {
        match self {
            AxisDirection::Xp => AxisDirection::Zm,
            AxisDirection::Zm => AxisDirection::Xm,
            AxisDirection::Xm => AxisDirection::Zp,
            AxisDirection::Zp => AxisDirection::Xp
        }
    }

}

pub fn get_dest_axisdirection(cur_axis: &AxisDirection, move_direction: &RelativeDirection) -> Coord {
    //Returns the destination location from given direction
    match move_direction {
        RelativeDirection::Up => AxisDirection::AD_YP,
        RelativeDirection::Down => AxisDirection::AD_YM,
        RelativeDirection::Forward => cur_axis.to_unit_vector(),
        RelativeDirection::Backward => (-cur_axis.to_unit_vector()),
        _ => panic!()
    }
}