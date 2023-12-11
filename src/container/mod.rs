mod plain_vec;
mod semi_sorted_vec;
// mod semi_tiered_vec;

pub use plain_vec::*;
pub use semi_sorted_vec::*;
// pub use semi_tiered_vec::*;

pub trait Container<T> {
    fn new() -> Self;
    fn new_with_one(x: T) -> Self;
    fn len(&self) -> usize;
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn contains(&self, x: T) -> bool;
    fn insert(&mut self, x: T);
    fn remove(&mut self, x: T);
    #[inline]
    fn insert_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I) {
        self.reserve(it.len());
        for x in it {
            self.insert(x);
        }
    }
    #[inline]
    fn remove_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I) {
        for x in it {
            self.remove(x);
        }
        self.shrink();
    }
    fn reserve(&mut self, additional: usize);
    fn shrink(&mut self);
}
