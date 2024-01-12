use super::Container;

#[derive(Debug, Clone)]
pub struct PlainVec<T: PartialEq> {
    vec: Vec<T>,
}

impl<T: PartialEq> Container<T> for PlainVec<T> {
    #[inline]
    fn new() -> Self {
        Self { vec: Vec::new() }
    }

    #[inline]
    fn new_with_one(x: T) -> Self {
        Self { vec: vec![x] }
    }

    #[inline]
    fn from_vec(vec: Vec<T>) -> Self {
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

    #[inline]
    fn contains(&self, x: &T) -> bool {
        self.vec.contains(x)
    }

    #[inline]
    fn insert(&mut self, x: T) -> bool {
        if !self.vec.contains(&x) {
            self.vec.push(x);
            return true;
        }
        false
    }

    #[inline]
    fn remove(&mut self, x: &T) -> bool {
        if let Some(i) = self.vec.iter().position(|y| y == x) {
            self.vec.swap_remove(i);
            return true;
        }
        false
    }

    #[inline]
    fn insert_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I) {
        for x in it {
            self.insert(x);
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
