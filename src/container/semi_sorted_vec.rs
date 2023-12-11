use super::*;

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

    fn insert_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I) {
        self.reserve(it.len());
        if self.len() + it.len() >= THRESHOLD {
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
