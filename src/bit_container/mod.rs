use crate::ffi::{RankBV, UniquePtr, WithinUniquePtr};
use core::ops::*;

pub trait BitContainer {
    fn new_with_bitlength(bitlength: usize) -> Self;
    fn bitlength(&self) -> usize;
    fn contains(&self, index: usize) -> bool;
    fn insert(&mut self, index: usize) -> bool;
    fn remove(&mut self, index: usize) -> bool;
    fn rank(&self, index: usize) -> usize;
    fn count(&self) -> usize;
    fn iter(&self) -> impl Iterator<Item = usize>;
}

pub struct RankBitContainer {
    bv: UniquePtr<RankBV>,
}

impl BitContainer for RankBitContainer {
    #[inline]
    fn new_with_bitlength(bitlength: usize) -> Self {
        Self {
            bv: RankBV::new(1 << bitlength).within_unique_ptr(),
        }
    }

    #[inline]
    fn bitlength(&self) -> usize {
        self.bv.size().ilog2() as usize
    }

    #[inline]
    fn contains(&self, index: usize) -> bool {
        self.bv.get(index)
    }

    #[inline]
    fn insert(&mut self, index: usize) -> bool {
        !self.bv.set(index)
    }

    #[inline]
    fn remove(&mut self, index: usize) -> bool {
        self.bv.clear(index)
    }

    #[inline]
    fn rank(&self, index: usize) -> usize {
        self.bv.rank(index) as usize
    }

    #[inline]
    fn count(&self) -> usize {
        self.bv.count_ones()
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = usize> {
        RankBVIterator {
            bitvector: &self.bv,
            block_index: 0,
            block: self.bv.get_block(0),
        }
    }
}

struct RankBVIterator<'a> {
    bitvector: &'a UniquePtr<RankBV>,
    block_index: usize,
    block: u64,
}

impl<'a> Iterator for RankBVIterator<'a> {
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

impl BitOr<Self> for &RankBitContainer {
    type Output = RankBitContainer;

    fn bitor(self, other: Self) -> Self::Output {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        let res = Self::Output::new_with_bitlength(self.bitlength());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            res.bv.update_block(i, a | b);
        }
        res
    }
}

impl BitOrAssign<&Self> for RankBitContainer {
    fn bitor_assign(&mut self, other: &Self) {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            self.bv.update_block(i, a | b);
        }
    }
}

impl BitAnd<Self> for &RankBitContainer {
    type Output = RankBitContainer;

    fn bitand(self, other: Self) -> Self::Output {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        let res = Self::Output::new_with_bitlength(self.bitlength());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            res.bv.update_block(i, a & b);
        }
        res
    }
}

impl BitAndAssign<&Self> for RankBitContainer {
    fn bitand_assign(&mut self, other: &Self) {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            self.bv.update_block(i, a & b);
        }
    }
}

impl Sub<Self> for &RankBitContainer {
    type Output = RankBitContainer;

    fn sub(self, other: Self) -> Self::Output {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        let res = Self::Output::new_with_bitlength(self.bitlength());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            res.bv.update_block(i, a & !b);
        }
        res
    }
}

impl SubAssign<&Self> for RankBitContainer {
    fn sub_assign(&mut self, other: &Self) {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            self.bv.update_block(i, a & !b);
        }
    }
}

impl BitXor<Self> for &RankBitContainer {
    type Output = RankBitContainer;

    fn bitxor(self, other: Self) -> Self::Output {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        let res = Self::Output::new_with_bitlength(self.bitlength());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            res.bv.update_block(i, a ^ b);
        }
        res
    }
}

impl BitXorAssign<&Self> for RankBitContainer {
    fn bitxor_assign(&mut self, other: &Self) {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            self.bv.update_block(i, a ^ b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    const N: usize = 10000;
    const BITS: usize = 20;

    fn test_bit_container<BC: BitContainer>() {
        let mut bitset = BC::new_with_bitlength(BITS);
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
    fn test_rbv() {
        test_bit_container::<RankBitContainer>();
    }

    #[test]
    fn test_rbv_iter() {
        let mut bitset = RankBitContainer::new_with_bitlength(BITS);
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

    #[test]
    fn test_rbv_union() {
        let mut bitset = RankBitContainer::new_with_bitlength(BITS);
        let mut bitset2 = RankBitContainer::new_with_bitlength(BITS);
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            bitset.insert(i);
        }
        for &i in v1.iter() {
            bitset2.insert(i);
        }
        let res = &bitset | &bitset2;
        bitset |= &bitset2;
        for &i in v0.iter() {
            assert!(bitset.contains(i), "false negative");
        }
        for &i in v1.iter() {
            assert!(bitset.contains(i), "false negative");
        }
        for &i in v2.iter() {
            assert!(!bitset.contains(i), "false positive");
        }
        assert_eq!(bitset.iter().collect_vec(), res.iter().collect_vec());
    }

    #[test]
    fn test_rbv_intersection() {
        let mut bitset = RankBitContainer::new_with_bitlength(BITS);
        let mut bitset2 = RankBitContainer::new_with_bitlength(BITS);
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            bitset.insert(i);
        }
        for &i in v1.iter() {
            bitset.insert(i);
            bitset2.insert(i);
        }
        for &i in v2.iter() {
            bitset2.insert(i);
        }
        let res = &bitset & &bitset2;
        bitset &= &bitset2;
        for &i in v0.iter() {
            assert!(!bitset.contains(i), "false positive");
        }
        for &i in v1.iter() {
            assert!(bitset.contains(i), "false negative");
        }
        for &i in v2.iter() {
            assert!(!bitset.contains(i), "false positive");
        }
        assert_eq!(bitset.iter().collect_vec(), res.iter().collect_vec());
    }

    #[test]
    fn test_rbv_difference() {
        let mut bitset = RankBitContainer::new_with_bitlength(BITS);
        let mut bitset2 = RankBitContainer::new_with_bitlength(BITS);
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            bitset.insert(i);
        }
        for &i in v1.iter() {
            bitset.insert(i);
            bitset2.insert(i);
        }
        for &i in v2.iter() {
            bitset2.insert(i);
        }
        let res = &bitset - &bitset2;
        bitset -= &bitset2;
        for &i in v0.iter() {
            assert!(bitset.contains(i), "false negative");
        }
        for &i in v1.iter() {
            assert!(!bitset.contains(i), "false positive");
        }
        for &i in v2.iter() {
            assert!(!bitset.contains(i), "false positive");
        }
        assert_eq!(bitset.iter().collect_vec(), res.iter().collect_vec());
    }

    #[test]
    fn test_rbv_symmetric_difference() {
        let mut bitset = RankBitContainer::new_with_bitlength(BITS);
        let mut bitset2 = RankBitContainer::new_with_bitlength(BITS);
        let v0 = (0..(3 * N)).step_by(3).collect_vec();
        let v1 = (0..(3 * N)).skip(1).step_by(3).collect_vec();
        let v2 = (0..(3 * N)).skip(2).step_by(3).collect_vec();
        for &i in v0.iter() {
            bitset.insert(i);
        }
        for &i in v1.iter() {
            bitset.insert(i);
            bitset2.insert(i);
        }
        for &i in v2.iter() {
            bitset2.insert(i);
        }
        let res = &bitset ^ &bitset2;
        bitset ^= &bitset2;
        for &i in v0.iter() {
            assert!(bitset.contains(i), "false negative");
        }
        for &i in v1.iter() {
            assert!(!bitset.contains(i), "false positive");
        }
        for &i in v2.iter() {
            assert!(bitset.contains(i), "false negative");
        }
        assert_eq!(bitset.iter().collect_vec(), res.iter().collect_vec());
    }
}
