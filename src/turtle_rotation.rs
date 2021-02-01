use crate::{vec3::Vec3};
type Coord = Vec3::<i32>;


#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum RelativeDirection {
    Forward,
    Backward,
    Right,
    Left,
    Down,
    Up
}

#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub enum AxisDirection {
    None,
    Xp,
    Xm,
    Zp,
    Zm,
    Yp,
    Ym
}

/*
    Xp
Zm      Zp
    Xm

*/
#[derive(Debug, Clone, PartialEq)]
pub enum Rotation {
    Y0,
    Y90,
    Y180,
    Y270
}

/*
    const ROT_0: (Coord, Coord, Coord) = (Vec3::<i32>(1,0,0), Vec3::<i32>(0,1,0), Vec3::<i32>(0,0,1));
    const ROT_Y90: (Coord, Coord, Coord) = (Vec3::<i32>(0,0,-1), Vec3::<i32>(0,1,0), Vec3::<i32>(1,0,0));
    const ROT_Y180: (Coord, Coord, Coord) = (Vec3::<i32>(-1, 0, 0), Vec3::<i32>(0,1,0), Vec3::<i32>(0,0,-1));
    const ROT_Y270: (Coord, Coord, Coord) = (Vec3::<i32>(0, 0, 1), Vec3::<i32>(0,1,0), Vec3::<i32>(-1,0,0)); */

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
            let rotated = &rot.apply_to(src);
            if  rotated.0 == dst.0 && rotated.2 == dst.2 {
                return rot.clone();
            }
        }
        panic!(format!("Could not rotate {:?} to {:?}", src, dst))
    }



}

impl AxisDirection {
    pub const AD_XP: Vec3<i32> = Vec3::<i32>(1,0,0);
    pub const AD_XM: Vec3<i32> = Vec3::<i32>(-1,0,0);
    pub const AD_ZP: Vec3<i32> = Vec3::<i32>(0,0,1);
    pub const AD_ZM: Vec3<i32> = Vec3::<i32>(0,0,-1);
    pub const AD_YP: Vec3<i32> = Vec3::<i32>(0,1,0);
    pub const AD_YM: Vec3<i32> = Vec3::<i32>(0,-1,0);

    pub const ALL: [AxisDirection;6] = [AxisDirection::Xp, AxisDirection::Zp, AxisDirection::Xm, AxisDirection::Zm, AxisDirection::Yp, AxisDirection::Ym];

    pub fn from(unit_vec: &Coord) -> Self {
        match unit_vec {
            &AxisDirection::AD_XP => AxisDirection::Xp,
            &AxisDirection::AD_XM => AxisDirection::Xm,
            &AxisDirection::AD_ZP => AxisDirection::Zp,
            &AxisDirection::AD_ZM => AxisDirection::Zm,
            &AxisDirection::AD_YP => AxisDirection::Yp,
            &AxisDirection::AD_YM => AxisDirection::Ym,
            _ => AxisDirection::None
        }
    }

    pub fn to_unit_vector(&self) -> Vec3<i32> {
        match self {
            AxisDirection::Xp => AxisDirection::AD_XP,
            AxisDirection::Xm => AxisDirection::AD_XM,
            AxisDirection::Zp => AxisDirection::AD_ZP,
            AxisDirection::Zm => AxisDirection::AD_ZM,
            AxisDirection::Yp => AxisDirection::AD_YP,
            AxisDirection::Ym => AxisDirection::AD_YM,
            AxisDirection::None => panic!("None direction is effectively zero vector, and can't be made into unit vector. ")
        }
    }

    pub fn rotate_right(&self) -> AxisDirection {
        match self {
            AxisDirection::Xp => AxisDirection::Zp,
            AxisDirection::Zp => AxisDirection::Xm,
            AxisDirection::Xm => AxisDirection::Zm,
            AxisDirection::Zm => AxisDirection::Xp,
            _ => panic!() // implement as identity if needed
        }
    }

    pub fn rotate_left(&self) -> AxisDirection {
        match self {
            AxisDirection::Xp => AxisDirection::Zm,
            AxisDirection::Zm => AxisDirection::Xm,
            AxisDirection::Xm => AxisDirection::Zp,
            AxisDirection::Zp => AxisDirection::Xp,
            _ => panic!() // implement as identity if needed
        }
    }

    pub fn dot(lhs: &AxisDirection, rhs: &AxisDirection) -> Rotation {
        let lhs_vec = lhs.to_unit_vector();
        let rhs_vec = rhs.to_unit_vector();
        if lhs==rhs {
            Rotation::Y0
        } else if lhs_vec == -rhs_vec {
            Rotation::Y180
        } else if &lhs.rotate_left() == rhs {
            Rotation::Y270
        } else if &lhs.rotate_right() == rhs {
            Rotation::Y90
        } else {
            panic!()
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

pub fn get_dest_pos(cur_pos: &Coord, cur_axis: &AxisDirection, move_direction: &RelativeDirection) -> Coord {
    let loc_dir = get_dest_axisdirection(&cur_axis, move_direction);
    cur_pos + &loc_dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis_dot() {
        let cases = [
            (AxisDirection::Xm, AxisDirection::Xm, Rotation::Y0),
            (AxisDirection::Xp, AxisDirection::Zm, Rotation::Y270),
            (AxisDirection::Xp, AxisDirection::Xm, Rotation::Y180),
            (AxisDirection::Xp, AxisDirection::Zp, Rotation::Y90)
            
            ];
        for case in &cases {
            let (lhs, rhs, rot) = case;            
            let result = AxisDirection::dot(lhs, rhs);
            assert_eq!(rot, &result);
        }
    }
}