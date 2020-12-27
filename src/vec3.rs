use std::ops::{Index, AddAssign, SubAssign, Add, Sub};


pub trait Vec3T<T>:
    Sub<Output=T>+Add<Output=T>+Copy
{
}

impl<T> Vec3T<T> for T where
    T: Sub<Output=T>+Add<Output=T>+Copy
{
}

pub struct Vec3<T> where T: Vec3T<T> {
    arr: [T; 3]
}

impl<T> Vec3<T> where T: Vec3T<T> {
    pub fn from_array(arr: [T; 3]) -> Self {
        Self {
            arr:arr
        }
    }

}

impl<T> Index<usize> for Vec3<T> where T: Vec3T<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.arr[index]
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
        for i in 0..3 {
            self.arr[i] = self.arr[i] + rhs.arr[i];
        }
    }
}

impl<'a, T> SubAssign<&'a Vec3<T>> for Vec3<T> where T: Vec3T<T> {
    fn sub_assign(&mut self, rhs: &'a Vec3<T>) {
        for i in 0..3 {
            self.arr[i] = self.arr[i] - rhs.arr[i];
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_addition() {
        let mut lhs: Vec3<i32> = Vec3::<i32>::from_array([1,2,3]);
        let rhs: Vec3<i32> = Vec3::<i32>{arr:[4,5,6]};
        lhs += &rhs;
        assert_eq!(lhs.arr, [5,7,9]);
    }
    #[test]
    fn test_subtraction() {
        let mut lhs: Vec3<i32> = Vec3::<i32>::from_array([1,2,3]);
        let rhs: Vec3<i32> = Vec3::<i32>{arr:[4,5,6]};
        lhs -= &rhs;
        assert_eq!(lhs.arr, [-3, -3, -3]);
    }
}