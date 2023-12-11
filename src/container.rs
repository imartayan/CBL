// use crate::tiered_vec::{medium, UniquePtr};
// use core::cmp::Ordering::{Equal, Greater, Less};
// use core::ops::Range;

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
    fn insert_iter<I: Iterator<Item = T>>(&mut self, it: I) {
        // self.reserve(it.len());
        for x in it {
            self.insert(x);
        }
    }
    #[inline]
    fn remove_iter<I: Iterator<Item = T>>(&mut self, it: I) {
        for x in it {
            self.remove(x);
        }
        // self.shrink();
    }
    fn reserve(&mut self, additional: usize);
    fn shrink(&mut self);
}

pub struct PlainVec<T: Ord> {
    vec: Vec<T>,
}

impl<T: Ord> Container<T> for PlainVec<T> {
    #[inline]
    fn new() -> Self {
        Self { vec: Vec::new() }
    }

    #[inline]
    fn new_with_one(x: T) -> Self {
        Self { vec: vec![x] }
    }

    #[inline]
    fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    fn contains(&self, x: T) -> bool {
        self.vec.contains(&x)
    }

    #[inline]
    fn insert(&mut self, x: T) {
        if !self.vec.contains(&x) {
            self.vec.push(x);
        }
    }

    #[inline]
    fn remove(&mut self, x: T) {
        if let Some(i) = self.vec.iter().position(|y| y == &x) {
            self.vec.swap_remove(i);
        }
    }

    fn insert_iter<I: Iterator<Item = T>>(&mut self, it: I) {
        // self.reserve(it.len());
        for x in it {
            self.insert(x);
        }
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
        // self.vec.reserve_exact(additional);
    }

    #[inline]
    fn shrink(&mut self) {
        self.vec.shrink_to_fit();
    }
}

pub struct SortedVec<T: Ord> {
    vec: Vec<T>,
}

impl<T: Ord> Container<T> for SortedVec<T> {
    #[inline]
    fn new() -> Self {
        Self { vec: Vec::new() }
    }

    #[inline]
    fn new_with_one(x: T) -> Self {
        Self { vec: vec![x] }
    }

    #[inline]
    fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    fn contains(&self, x: T) -> bool {
        self.vec.binary_search(&x).is_ok()
    }

    #[inline]
    fn insert(&mut self, x: T) {
        if let Err(i) = self.vec.binary_search(&x) {
            self.vec.insert(i, x);
        }
    }

    #[inline]
    fn remove(&mut self, x: T) {
        if let Ok(i) = self.vec.binary_search(&x) {
            self.vec.remove(i);
        }
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
        // self.vec.reserve_exact(additional);
    }

    #[inline]
    fn shrink(&mut self) {
        self.vec.shrink_to_fit();
    }
}

pub struct SemiSortedVec<T: Ord + Copy, const THRESHOLD: usize> {
    vec: Vec<T>,
}

impl<T: Ord + Copy, const THRESHOLD: usize> Container<T> for SemiSortedVec<T, THRESHOLD> {
    #[inline]
    fn new() -> Self {
        Self { vec: Vec::new() }
    }

    #[inline]
    fn new_with_one(x: T) -> Self {
        Self { vec: vec![x] }
    }

    #[inline]
    fn len(&self) -> usize {
        self.vec.len()
    }

    fn contains(&self, x: T) -> bool {
        if self.len() >= THRESHOLD {
            self.vec.binary_search(&x).is_ok()
        } else {
            self.vec.contains(&x)
        }
    }

    fn insert(&mut self, x: T) {
        if self.len() >= THRESHOLD {
            if let Err(i) = self.vec.binary_search(&x) {
                self.vec.insert(i, x);
            }
        } else if !self.vec.contains(&x) {
            self.vec.push(x);
            if self.vec.len() == THRESHOLD {
                self.vec.sort_unstable();
            }
        }
    }

    fn remove(&mut self, x: T) {
        if self.len() >= THRESHOLD {
            if let Ok(i) = self.vec.binary_search(&x) {
                self.vec.remove(i);
            }
        } else if let Some(i) = self.vec.iter().position(|y| y == &x) {
            self.vec.swap_remove(i);
        }
    }

    fn insert_iter<I: Iterator<Item = T>>(&mut self, it: I) {
        // self.reserve(it.len());
        // if self.len() + it.len() >= THRESHOLD {
        if self.len() >= THRESHOLD {
            self.vec.extend(it);
            self.vec.sort_unstable();
            self.vec.dedup();
        } else {
            for x in it {
                self.insert(x);
            }
        }
        // if self.len() >= THRESHOLD {
        //     let mut values: Vec<T> = it.collect();
        //     values.sort_unstable();
        //     self.vec = merge_sorted_vec(&self.vec, &values);
        // } else if it.len() >= THRESHOLD {
        //     let mut values: Vec<T> = it.collect();
        //     self.vec.sort_unstable();
        //     values.sort_unstable();
        //     self.vec = merge_sorted_vec(&self.vec, &values);
        // } else {
        //     self.reserve(it.len());
        //     for x in it {
        //         self.insert(x);
        //     }
        // }
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
        // self.vec.reserve_exact(additional);
    }

    #[inline]
    fn shrink(&mut self) {
        self.vec.shrink_to_fit();
    }
}

// fn dedup_in_place<T: Ord + Copy>(v: &mut Vec<T>, mid: usize) {
//     v.dedup()
//     let (mut i, mut j) = (a.start, b.start);
//     while i < a.end && j < b.end {
//         let (x, y) = (v[i], v[j]);
//         match x.cmp(&y) {
//             Less => {
//                 i += 1;
//             }
//             Greater => {
//                 v.push(y);
//                 j += 1;
//             }
//             Equal => {
//                 v.push(x);
//                 i += 1;
//                 j += 1;
//             }
//         }
//     }
//     if i < v1.len() {
//         v.extend_from_slice(&v1[i..]);
//     } else if j < v2.len() {
//         v.extend_from_slice(&v2[j..]);
//     }
//     v
// }

// fn merge_sorted_vec<T: Ord + Copy>(v1: &[T], v2: &[T]) -> Vec<T> {
//     let mut v = Vec::with_capacity(v1.len() + v2.len());
//     let (mut i, mut j) = (0, 0);
//     while i < v1.len() && j < v2.len() {
//         let (x, y) = (v1[i], v2[j]);
//         match x.cmp(&y) {
//             Less => {
//                 v.push(x);
//                 i += 1;
//             }
//             Greater => {
//                 v.push(y);
//                 j += 1;
//             }
//             Equal => {
//                 v.push(x);
//                 i += 1;
//                 j += 1;
//             }
//         }
//     }
//     if i < v1.len() {
//         v.extend_from_slice(&v1[i..]);
//     } else if j < v2.len() {
//         v.extend_from_slice(&v2[j..]);
//     }
//     v
// }

// pub struct LazyTieredVec {
//     small_vec: SortedVec<u32>,
//     tiered_vec: UniquePtr<medium::TieredVec24>,
// }

// impl LazyTieredVec {
//     const GROWTH_THRESHOLD: usize = 128;
//     const SHRINK_THRESHOLD: usize = 128;

//     fn small_to_tiered(&mut self) {
//         self.tiered_vec = medium::new_tiered_vec_24();
//         for x in self.small_vec.vec.drain(..) {
//             self.tiered_vec.insert(self.tiered_vec.len(), x);
//         }
//         self.small_vec.vec.shrink_to_fit();
//     }

//     fn tiered_to_small(&mut self) {
//         todo!()
//     }
// }

// impl Container<u32> for LazyTieredVec {
//     fn new() -> Self {
//         Self {
//             small_vec: SortedVec::new(),
//             tiered_vec: UniquePtr::null(),
//         }
//     }

//     fn new_with_one(x: u32) -> Self {
//         Self {
//             small_vec: SortedVec::new_with_one(x),
//             tiered_vec: UniquePtr::null(),
//         }
//     }

//     fn len(&self) -> usize {
//         if self.tiered_vec.is_null() {
//             self.small_vec.len()
//         } else {
//             self.tiered_vec.len()
//         }
//     }

//     fn contains(&self, x: u32) -> bool {
//         if self.tiered_vec.is_null() {
//             self.small_vec.contains(x)
//         } else {
//             self.tiered_vec.contains_sorted(x)
//         }
//     }

//     fn insert(&mut self, x: u32) {
//         if self.tiered_vec.is_null() {
//             self.small_vec.insert(x);
//             if self.small_vec.len() >= Self::GROWTH_THRESHOLD {
//                 self.small_to_tiered();
//             }
//         } else {
//             self.tiered_vec.insert_sorted(x);
//         }
//     }

//     fn remove(&mut self, x: u32) {
//         if self.tiered_vec.is_null() {
//             self.small_vec.remove(x);
//             if self.small_vec.len() <= Self::SHRINK_THRESHOLD {
//                 self.tiered_to_small();
//             }
//         } else {
//             todo!()
//         }
//     }
// }
