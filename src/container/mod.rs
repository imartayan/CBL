#![allow(unused_imports)]

mod plain_vec;
mod semi_sorted_vec;

use crate::compact_int::CompactInt;
pub use plain_vec::PlainVec;
pub use semi_sorted_vec::SemiSortedVec;

pub trait Container<T> {
    fn new() -> Self;
    fn new_with_one(x: T) -> Self;
    fn len(&self) -> usize;
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn contains(&self, x: &T) -> bool;
    fn insert(&mut self, x: T) -> bool;
    fn remove(&mut self, x: &T) -> bool;
    fn insert_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I);
    fn remove_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I);
    fn iter<'a>(&'a self) -> impl ExactSizeIterator<Item = &'a T>
    where
        T: 'a;
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    type T = usize;
    const N: usize = 100;

    fn test_container<ContainerT: Container<T>>() {
        let mut container = ContainerT::new();
        let v0 = (0..(2 * N)).step_by(2).collect_vec();
        let v1 = (0..(2 * N)).skip(1).step_by(2).collect_vec();
        for &i in v0.iter() {
            assert!(container.insert(i), "failed to insert {i}");
        }
        for i in v0.iter() {
            assert!(container.contains(i), "false negative {i}");
        }
        for i in v1.iter() {
            assert!(!container.contains(i), "false positive {i}");
        }
        for i in v0.iter() {
            assert_eq!(container.len(), N - i / 2, "wrong len");
            assert!(container.remove(i), "failed to remove {i}");
        }
        assert!(container.is_empty());
    }

    fn test_container_iter<ContainerT: Container<T>>() {
        let mut container = ContainerT::new();
        let v0 = (0..(2 * N)).step_by(2).collect_vec();
        let v1 = (0..(2 * N)).skip(1).step_by(2).collect_vec();
        container.insert_iter(v0.iter().copied());
        for i in v0.iter() {
            assert!(container.contains(i), "false negative {i}");
        }
        for i in v1.iter() {
            assert!(!container.contains(i), "false positive {i}");
        }
        container.remove_iter(v0.iter().copied());
        assert!(container.is_empty());
    }

    #[test]
    fn test_plain_vec() {
        for _ in 0..10000 {
            test_container::<PlainVec<T>>();
            test_container_iter::<PlainVec<T>>();
        }
    }

    #[test]
    fn test_semi_sorted_vec() {
        for _ in 0..10000 {
            test_container::<SemiSortedVec<T, 32>>();
            test_container_iter::<SemiSortedVec<T, 32>>();
        }
    }
}
