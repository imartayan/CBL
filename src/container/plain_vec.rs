use super::Container;

#[derive(Debug)]
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
    fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    fn contains(&self, x: T) -> bool {
        self.vec.contains(&x)
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
    fn remove(&mut self, x: T) -> bool {
        if let Some(i) = self.vec.iter().position(|y| y == &x) {
            self.vec.swap_remove(i);
            return true;
        }
        false
    }

    #[inline]
    fn insert_iter<I: Iterator<Item = T>>(&mut self, it: I) {
        // self.reserve(it.len());
        for x in it {
            self.insert(x);
        }
    }

    #[inline]
    fn remove_iter<I: ExactSizeIterator<Item = T>>(&mut self, it: I) {
        for x in it {
            self.remove(x);
        }
        // self.shrink();
        // self.vec.shrink_to_fit();
    }

    // #[inline]
    // fn reserve(&mut self, additional: usize) {
    //     self.vec.reserve(additional);
    //     // self.vec.reserve_exact(additional);
    // }

    // #[inline]
    // fn shrink(&mut self) {
    //     self.vec.shrink_to_fit();
    // }
}
