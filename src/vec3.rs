use std::ops::{Index, AddAssign, SubAssign, Add, Sub, Neg, Mul};
use serde_derive::{Deserialize, Serialize};


pub trait Vec3T<T>:
    Sub<Output=T>+Add<Output=T>+Copy+Neg<Output=T>+Default+Mul<Output=T>

{
}

impl<T> Vec3T<T> for T where
    T: Sub<Output=T>+Add<Output=T>+Copy+Neg<Output=T>+Default+Mul<Output=T>
{
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Vec3<T>(pub T, pub T, pub T) where T: Vec3T<T>;

impl<T> Vec3<T> where T: Vec3T<T> +  {
    pub fn from_array(arr: [T; 3]) -> Self {
        Self{0: arr[0], 1:arr[1], 2:arr[2]}
    }
    pub fn new(x: T, y: T, z: T) -> Self {
        Vec3::<T>(x,y,z)
    }

    pub fn zero() -> Self {
        Vec3::<T>(Default::default(), Default::default(), Default::default())
    }

    pub fn dot(&self, rhs: &Self) -> T {
        self.0*rhs.0 + self.1*rhs.1 + self.2*rhs.2
    }
}

impl<T> Vec3<T> where T: Vec3T<T> + std::cmp::PartialOrd {
    pub fn abs_sum(&self) -> T {
        (if self.0 >= T::default() {
            self.0
        } else {
            -self.0
        }) +
        (if self.1 >= T::default() {
            self.1
        } else {
            -self.1
        }) +
        (if self.2 >= T::default() {
            self.2
        } else {
            -self.2
        })
    }
}


impl<T> Index<usize> for Vec3<T> where T: Vec3T<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Index out of bounds!")
        }
    }
}

// impl<T> AddAssign<&Vec3<T>> for &Vec3<T> where T: Add+Sub + Add<Output = T> + Copy {
    
//     fn add_assign(&mut self, rhs: &Vec3<T>) {
//         for i in 0..3 {
//             self.arr[i] = self.arr[i] + rhs[i];
//         }
//     }
// }

impl<'a, T> AddAssign<&'a Vec3<T>> for Vec3<T> where T: Vec3T<T> {
    fn add_assign(&mut self, rhs: &'a Vec3<T>) {
        // for i in 0..3 {
        //     self.index(i) = self.index(i) + rhs.index(i);
        // }
        self.0 = self.0 + rhs.0;
        self.1 = self.1 + rhs.1;
        self.2 = self.2 + rhs.2;
    }
}

impl<'a, T> SubAssign<&'a Vec3<T>> for Vec3<T> where T: Vec3T<T> {
    fn sub_assign(&mut self, rhs: &'a Vec3<T>) {
        // for i in 0..3 {
        //     self.arr[i] = self.arr[i] - rhs.arr[i];
        // }
        self.0 = self.0 - rhs.0;
        self.1 = self.1 - rhs.1;
        self.2 = self.2 - rhs.2;
    }
}

impl<'a, 'b, T> Sub<&'b Vec3<T>> for &'a Vec3<T> where T: Vec3T<T> {
    type Output = Vec3<T>;

    fn sub(self, rhs: &'b Vec3<T>) -> Self::Output {
        Vec3::<T>(self.0-rhs.0, self.1-rhs.1, self.2-rhs.2)
    }
}

impl<'a, 'b, T> Add<&'b Vec3<T>> for &'a Vec3<T> where T: Vec3T<T> {
    type Output = Vec3<T>;

    fn add(self, rhs: &'b Vec3<T>) -> Self::Output {
        Vec3::<T>(self.0+rhs.0, self.1+rhs.1, self.2+rhs.2)
    }
}

impl<T> Neg for Vec3<T> where T: Vec3T<T> {
    type Output = Vec3<T>;

    fn neg(self) -> Self::Output {
        Vec3::<T>(-self.0, -self.1, -self.2)
    }
}


// fn rotate_y()

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_addassign() {
        let mut lhs: Vec3<i32> = Vec3::<i32>::from_array([1,2,3]);
        let rhs: Vec3<i32> = Vec3::<i32>::from_array([4,5,6]);
        lhs += &rhs;
        assert_eq!(lhs.0, 5);

    }
    #[test]
    fn test_subassign() {
        let mut lhs: Vec3<i32> = Vec3::<i32>::from_array([1,2,3]);
        let rhs: Vec3<i32> = Vec3::<i32>::from_array([4,5,6]);
        lhs -= &rhs;
        assert_eq!(lhs, Vec3::<i32>(-3, -3, -3));
    }

    #[test]
    fn test_subref() {
        let lhs = Vec3::<i32>(1,2,3);
        assert_eq!(Vec3::<i32>::zero(), &lhs-&lhs);
        let rhs = Vec3::<i32>(2,4,6);
        
        let result = &lhs-&rhs;
        assert_eq!(Vec3::<i32>(-1,-2,-3), result);

        
    }

    
    #[test]
    fn test_addref() {
        let lhs = Vec3::<i32>(1,2,3);
        let rhs = Vec3::<i32>(2,3,4);
        
        let result = &lhs+&rhs;
        assert_eq!(Vec3::<i32>(3,5,7), result);
    }

    #[test]
    fn test_absum() {
        let vec = Vec3(-1, 3, -9);
        let sum = vec.abs_sum();
        assert_eq!(13, sum);
    }

}