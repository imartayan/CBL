use super::Container;

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
    fn iter<'a>(&'a self) -> impl ExactSizeIterator<Item = &'a T>
    where
        T: 'a,
    {
        self.vec.iter()
    }
}
