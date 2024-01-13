use super::Bitvector;
use core::ops::*;

impl BitOr<Self> for &Bitvector {
    type Output = Bitvector;

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

impl BitOrAssign<&Self> for Bitvector {
    fn bitor_assign(&mut self, other: &Self) {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            self.bv.update_block(i, a | b);
        }
    }
}

impl BitAnd<Self> for &Bitvector {
    type Output = Bitvector;

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

impl BitAndAssign<&Self> for Bitvector {
    fn bitand_assign(&mut self, other: &Self) {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            self.bv.update_block(i, a & b);
        }
    }
}

impl Sub<Self> for &Bitvector {
    type Output = Bitvector;

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

impl SubAssign<&Self> for Bitvector {
    fn sub_assign(&mut self, other: &Self) {
        assert_eq!(self.bv.num_blocks(), other.bv.num_blocks());
        for i in 0..self.bv.num_blocks() {
            let a = self.bv.get_block(i);
            let b = other.bv.get_block(i);
            self.bv.update_block(i, a & !b);
        }
    }
}

impl BitXor<Self> for &Bitvector {
    type Output = Bitvector;

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

impl BitXorAssign<&Self> for Bitvector {
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

    #[test]
    fn test_union() {
        let mut bitset = Bitvector::new_with_bitlength(BITS);
        let mut bitset2 = Bitvector::new_with_bitlength(BITS);
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
    fn test_intersection() {
        let mut bitset = Bitvector::new_with_bitlength(BITS);
        let mut bitset2 = Bitvector::new_with_bitlength(BITS);
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
    fn test_difference() {
        let mut bitset = Bitvector::new_with_bitlength(BITS);
        let mut bitset2 = Bitvector::new_with_bitlength(BITS);
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
    fn test_symmetric_difference() {
        let mut bitset = Bitvector::new_with_bitlength(BITS);
        let mut bitset2 = Bitvector::new_with_bitlength(BITS);
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
