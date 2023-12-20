mod plain_vec;
mod semi_sorted_vec;

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
    fn contains(&self, x: T) -> bool;
    fn insert(&mut self, x: T) -> bool;
    fn remove(&mut self, x: T) -> bool;
    fn insert_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I);
    fn remove_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I);
}

#[cfg(test)]
mod tests {
    use super::*;

    type T = usize;
    const N: usize = 10000;

    fn test_container<ContainerT: Container<T>>() {
        let mut container = ContainerT::new();
        for i in (0..(2 * N)).step_by(2) {
            container.insert(i);
        }
        for i in (0..(2 * N)).step_by(2) {
            assert!(container.contains(i));
        }
        for i in (0..(2 * N)).skip(1).step_by(2) {
            assert!(!container.contains(i));
        }
        for i in (0..(2 * N)).step_by(2) {
            assert_eq!(container.len(), N - i / 2);
            container.remove(i);
        }
        assert!(container.is_empty());
    }

    fn test_container_iter<ContainerT: Container<T>>() {
        let mut container = ContainerT::new();
        container.insert_iter((0..(2 * N)).step_by(2));
        for i in (0..(2 * N)).step_by(2) {
            assert!(container.contains(i));
        }
        for i in (0..(2 * N)).skip(1).step_by(2) {
            assert!(!container.contains(i));
        }
        container.remove_iter((0..(2 * N)).step_by(2));
        assert!(container.is_empty());
    }

    #[test]
    fn test_plain_vec() {
        test_container::<PlainVec<T>>();
        test_container_iter::<PlainVec<T>>();
    }

    #[test]
    fn test_semi_sorted_vec() {
        test_container::<SemiSortedVec<T, 32>>();
        test_container_iter::<SemiSortedVec<T, 32>>();
    }
}
