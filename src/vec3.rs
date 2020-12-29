use std::ops::{Index, AddAssign, SubAssign, Add, Sub, Neg};


pub trait Vec3T<T>:
    Sub<Output=T>+Add<Output=T>+Copy+Neg<Output=T>+Default

{
}

impl<T> Vec3T<T> for T where
    T: Sub<Output=T>+Add<Output=T>+Copy+Neg<Output=T>+Default
{
}

#[derive(PartialEq, Debug)]
pub struct Vec3<T>(pub T, pub T, pub T) where T: Vec3T<T>;

impl<T> Vec3<T> where T: Vec3T<T> {
    pub fn from_array(arr: [T; 3]) -> Self {
        Self{0: arr[0], 1:arr[1], 2:arr[2]}
    }
    pub fn new(x: T, y: T, z: T) -> Self {
        Vec3::<T>(x,y,z)
    }

    pub fn zero() -> Self {
        Vec3::<T>(Default::default(), Default::default(), Default::default())
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

impl<T> Neg for Vec3<T> where T: Vec3T<T> {
    type Output = Vec3<T>;

    fn neg(self) -> Self::Output {
        Vec3::<T>(-self.0, -self.1, -self.2)
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_addition() {
        let mut lhs: Vec3<i32> = Vec3::<i32>::from_array([1,2,3]);
        let rhs: Vec3<i32> = Vec3::<i32>::from_array([4,5,6]);
        lhs += &rhs;
        assert_eq!(lhs.0, 5);

    }
    #[test]
    fn test_subtraction() {
        let mut lhs: Vec3<i32> = Vec3::<i32>::from_array([1,2,3]);
        let rhs: Vec3<i32> = Vec3::<i32>::from_array([4,5,6]);
        lhs -= &rhs;
        // assert_eq!(lhs.arr, [-3, -3, -3]);
    }
}