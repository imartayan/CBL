use super::Container;
use core::cmp::Ordering;
use core::ops::*;
use core::slice::Iter;

#[derive(Debug, Clone)]
pub struct SemiSortedVec<T: Ord, const THRESHOLD: usize> {
    vec: Vec<T>,
}

impl<T: Ord, const THRESHOLD: usize> Container<T> for SemiSortedVec<T, THRESHOLD> {
    #[inline]
    fn new() -> Self {
        Self { vec: Vec::new() }
    }

    #[inline]
    fn new_with_one(x: T) -> Self {
        Self { vec: vec![x] }
    }

    #[inline]
    fn from_vec(mut vec: Vec<T>) -> Self {
        if vec.len() >= THRESHOLD {
            vec.sort_unstable();
        }
        Self { vec }
    }

    #[inline]
    unsafe fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self { vec }
    }

    #[inline]
    fn to_vec(self) -> Vec<T> {
        self.vec
    }

    #[inline]
    fn len(&self) -> usize {
        self.vec.len()
    }

    fn contains(&self, x: &T) -> bool {
        if self.len() >= THRESHOLD {
            self.vec.binary_search(x).is_ok()
        } else {
            self.vec.contains(x)
        }
    }

    fn insert(&mut self, x: T) -> bool {
        if self.len() >= THRESHOLD {
            if let Err(i) = self.vec.binary_search(&x) {
                self.vec.insert(i, x);
                return true;
            }
        } else if !self.vec.contains(&x) {
            self.vec.push(x);
            if self.vec.len() == THRESHOLD {
                self.vec.sort_unstable();
            }
            return true;
        }
        false
    }

    fn remove(&mut self, x: &T) -> bool {
        if self.len() >= THRESHOLD {
            if let Ok(i) = self.vec.binary_search(x) {
                self.vec.remove(i);
                return true;
            }
        } else if let Some(i) = self.vec.iter().position(|y| y == x) {
            self.vec.swap_remove(i);
            return true;
        }
        false
    }

    fn insert_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I) {
        if self.len() + it.len() >= THRESHOLD {
            self.vec.extend(it);
            self.vec.sort_unstable();
            self.vec.dedup();
        } else {
            for x in it {
                self.insert(x);
            }
        }
    }

    #[inline]
    fn remove_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I) {
        for x in it {
            self.remove(&x);
        }
    }

    #[inline]
    fn iter<'a>(&'a self) -> Iter<'a, T>
    where
        T: 'a,
    {
        self.vec.iter()
    }
}

impl<T: Copy + Ord, const THRESHOLD: usize> BitOr<Self> for &mut SemiSortedVec<T, THRESHOLD> {
    type Output = SemiSortedVec<T, THRESHOLD>;

    fn bitor(self, other: Self) -> Self::Output {
        let mut vec = Vec::new();
        if self.vec.len() < THRESHOLD {
            self.vec.sort_unstable();
        }
        if other.vec.len() < THRESHOLD {
            other.vec.sort_unstable();
        }
        let mut self_iter = self.vec.iter();
        let mut other_iter = other.vec.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(b) {
                Ordering::Less => {
                    vec.push(*a);
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    vec.push(*b);
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    vec.push(*a);
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        while let Some(a) = x {
            vec.push(*a);
            x = self_iter.next();
        }
        while let Some(b) = y {
            vec.push(*b);
            y = other_iter.next();
        }
        unsafe { Self::Output::from_vec_unchecked(vec) }
    }
}

impl<T: Copy + Ord, const THRESHOLD: usize> BitAnd<Self> for &mut SemiSortedVec<T, THRESHOLD> {
    type Output = SemiSortedVec<T, THRESHOLD>;

    fn bitand(self, other: Self) -> Self::Output {
        let mut vec = Vec::new();
        if self.vec.len() < THRESHOLD {
            self.vec.sort_unstable();
        }
        if other.vec.len() < THRESHOLD {
            other.vec.sort_unstable();
        }
        let mut self_iter = self.vec.iter();
        let mut other_iter = other.vec.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(b) {
                Ordering::Less => {
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    vec.push(*a);
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        unsafe { Self::Output::from_vec_unchecked(vec) }
    }
}

impl<T: Copy + Ord, const THRESHOLD: usize> Sub<Self> for &mut SemiSortedVec<T, THRESHOLD> {
    type Output = SemiSortedVec<T, THRESHOLD>;

    fn sub(self, other: Self) -> Self::Output {
        let mut vec = Vec::new();
        if self.vec.len() < THRESHOLD {
            self.vec.sort_unstable();
        }
        if other.vec.len() < THRESHOLD {
            other.vec.sort_unstable();
        }
        let mut self_iter = self.vec.iter();
        let mut other_iter = other.vec.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(b) {
                Ordering::Less => {
                    vec.push(*a);
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        while let Some(a) = x {
            vec.push(*a);
            x = self_iter.next();
        }
        unsafe { Self::Output::from_vec_unchecked(vec) }
    }
}

impl<T: Copy + Ord, const THRESHOLD: usize> BitXor<Self> for &mut SemiSortedVec<T, THRESHOLD> {
    type Output = SemiSortedVec<T, THRESHOLD>;

    fn bitxor(self, other: Self) -> Self::Output {
        let mut vec = Vec::new();
        if self.vec.len() < THRESHOLD {
            self.vec.sort_unstable();
        }
        if other.vec.len() < THRESHOLD {
            other.vec.sort_unstable();
        }
        let mut self_iter = self.vec.iter();
        let mut other_iter = other.vec.iter();
        let mut x = self_iter.next();
        let mut y = other_iter.next();
        while let (Some(a), Some(b)) = (x, y) {
            match a.cmp(b) {
                Ordering::Less => {
                    vec.push(*a);
                    x = self_iter.next();
                }
                Ordering::Greater => {
                    vec.push(*b);
                    y = other_iter.next();
                }
                Ordering::Equal => {
                    x = self_iter.next();
                    y = other_iter.next();
                }
            }
        }
        while let Some(a) = x {
            vec.push(*a);
            x = self_iter.next();
        }
        while let Some(b) = y {
            vec.push(*b);
            y = other_iter.next();
        }
        unsafe { Self::Output::from_vec_unchecked(vec) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    const N: usize = 1000;

    #[test]
    fn test_union() {
        let mut container = SemiSortedVec::<usize, 32>::new();
        let mut container2 = SemiSortedVec::<usize, 32>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            container.insert(i);
        }
        for &i in v1.iter() {
            container2.insert(i);
        }
        container = &mut container | &mut container2;
        for i in v0.iter() {
            assert!(container.contains(i), "false negative for {i}");
        }
        for i in v1.iter() {
            assert!(container.contains(i), "false negative for {i}");
        }
        for i in v2.iter() {
            assert!(!container.contains(i), "false positive for {i}");
        }
    }

    #[test]
    fn test_intersection() {
        let mut container = SemiSortedVec::<usize, 32>::new();
        let mut container2 = SemiSortedVec::<usize, 32>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            container.insert(i);
        }
        for &i in v1.iter() {
            container.insert(i);
            container2.insert(i);
        }
        for &i in v2.iter() {
            container2.insert(i);
        }
        container = &mut container & &mut container2;
        for i in v0.iter() {
            assert!(!container.contains(i), "false positive for {i}");
        }
        for i in v1.iter() {
            assert!(container.contains(i), "false negative for {i}");
        }
        for i in v2.iter() {
            assert!(!container.contains(i), "false positive for {i}");
        }
    }

    #[test]
    fn test_difference() {
        let mut container = SemiSortedVec::<usize, 32>::new();
        let mut container2 = SemiSortedVec::<usize, 32>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            container.insert(i);
        }
        for &i in v1.iter() {
            container.insert(i);
            container2.insert(i);
        }
        for &i in v2.iter() {
            container2.insert(i);
        }
        container = &mut container - &mut container2;
        for i in v0.iter() {
            assert!(container.contains(i), "false negative for {i}");
        }
        for i in v1.iter() {
            assert!(!container.contains(i), "false positive for {i}");
        }
        for i in v2.iter() {
            assert!(!container.contains(i), "false positive for {i}");
        }
    }

    #[test]
    fn test_symmetric_difference() {
        let mut container = SemiSortedVec::<usize, 32>::new();
        let mut container2 = SemiSortedVec::<usize, 32>::new();
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            container.insert(i);
        }
        for &i in v1.iter() {
            container.insert(i);
            container2.insert(i);
        }
        for &i in v2.iter() {
            container2.insert(i);
        }
        container = &mut container ^ &mut container2;
        for i in v0.iter() {
            assert!(container.contains(i), "false negative for {i}");
        }
        for i in v1.iter() {
            assert!(!container.contains(i), "false positive for {i}");
        }
        for i in v2.iter() {
            assert!(container.contains(i), "false negative for {i}");
        }
    }
}
