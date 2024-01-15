mod set_ops;

use crate::ffi::{RankBV, UniquePtr, WithinUniquePtr};

pub struct Bitvector {
    bv: UniquePtr<RankBV>,
}

impl Bitvector {
    #[inline]
    pub fn new_with_bitlength(bitlength: usize) -> Self {
        Self {
            bv: RankBV::new(1 << bitlength).within_unique_ptr(),
        }
    }

    #[inline]
    pub fn bitlength(&self) -> usize {
        self.bv.size().ilog2() as usize
    }

    #[inline]
    pub fn contains(&self, index: usize) -> bool {
        self.bv.get(index)
    }

    #[inline]
    pub fn insert(&mut self, index: usize) -> bool {
        !self.bv.set(index)
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> bool {
        self.bv.clear(index)
    }

    #[inline]
    pub fn rank(&self, index: usize) -> usize {
        self.bv.rank(index) as usize
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.bv.count_ones()
    }

    #[inline]
    pub fn iter(&self) -> BitvectorIterator {
        BitvectorIterator {
            bitvector: &self.bv,
            block_index: 0,
            block: self.bv.get_block(0),
        }
    }
}

pub struct BitvectorIterator<'a> {
    bitvector: &'a UniquePtr<RankBV>,
    block_index: usize,
    block: u64,
}

impl<'a> Iterator for BitvectorIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let num_blocks = self.bitvector.num_blocks();
        while self.block_index < num_blocks && self.block == 0 {
            self.block_index += 1;
            self.block = self.bitvector.get_block(self.block_index);
        }
        if self.block_index >= num_blocks {
            return None;
        }
        let bit_index = self.block.trailing_zeros() as usize;
        self.block -= 1 << bit_index;
        Some(self.block_index * 64 + bit_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    const N: usize = 10000;
    const BITS: usize = 20;

    #[test]
    fn test_bitvector() {
        let mut bitset = Bitvector::new_with_bitlength(BITS);
        let v0 = (0..(2 * N)).step_by(2).collect_vec();
        let v1 = (0..(2 * N)).skip(1).step_by(2).collect_vec();
        for &i in v0.iter() {
            bitset.insert(i);
        }
        for &i in v0.iter() {
            assert!(bitset.contains(i), "false negative");
        }
        for &i in v1.iter() {
            assert!(!bitset.contains(i), "false positive");
        }
        for &i in v0.iter() {
            assert_eq!(bitset.rank(i), i / 2, "wrong rank");
        }
        for &i in v0.iter() {
            assert_eq!(bitset.count(), N - i / 2, "wrong count");
            bitset.remove(i);
        }
        assert_eq!(bitset.count(), 0, "wrong count");
    }

    #[test]
    fn test_bitvector_iter() {
        let mut bitset = Bitvector::new_with_bitlength(BITS);
        bitset.insert(1);
        bitset.insert(3);
        bitset.insert(42);
        bitset.insert(101010);
        bitset.insert((1 << BITS) - 1);
        let mut iter = bitset.iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(42));
        assert_eq!(iter.next(), Some(101010));
        assert_eq!(iter.next(), Some((1 << BITS) - 1));
        assert_eq!(iter.next(), None);
    }
}
