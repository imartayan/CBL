// Adapted from https://github.com/Daniel-Liu-c0deb0t/simple-saca/blob/main/src/compact_vec.rs

use core::cmp::Ordering;
use num_traits::sign::Unsigned;
use num_traits::PrimInt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct CompactInt<const BYTES: usize>([u8; BYTES]);

impl<const BYTES: usize> CompactInt<BYTES> {
    #[inline(always)]
    pub fn new() -> Self {
        Self([0u8; BYTES])
    }

    #[inline(always)]
    pub fn from_int<T: PrimInt + Unsigned>(val: T) -> Self {
        let mut res = Self::new();
        res.set(val);
        res
    }

    #[inline(always)]
    pub fn get<T: PrimInt + Unsigned>(&self) -> T {
        let mut res = T::zero();
        unsafe {
            std::ptr::copy_nonoverlapping(self.0.as_ptr(), &mut res as *mut _ as _, BYTES);
        }
        T::from_le(res) // make sure to read bytes as little endian
    }

    #[inline(always)]
    pub fn set<T: PrimInt + Unsigned>(&mut self, val: T) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                &val.to_le() as *const _ as _, // make sure to write bytes as little endian
                self.0.as_mut_ptr(),
                BYTES,
            );
        }
    }
}

impl<const BYTES: usize> Default for CompactInt<BYTES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BYTES: usize> Ord for CompactInt<BYTES> {
    #[inline]
    fn cmp(&self, other: &CompactInt<BYTES>) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl<const BYTES: usize> PartialOrd for CompactInt<BYTES> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTES: usize = 3;

    #[test]
    fn test_from_get() {
        let x = CompactInt::<BYTES>::from_int(442u32);
        assert_eq!(x.get::<u32>(), 442, "{:?}", x.0);
    }

    #[test]
    fn test_set_get() {
        let mut x = CompactInt::<BYTES>::new();
        x.set(631u32);
        assert_eq!(x.get::<u32>(), 631, "{:?}", x.0);
        x.set(363u32);
        assert_eq!(x.get::<u32>(), 363, "{:?}", x.0);
    }

    #[test]
    fn test_eq() {
        let x = CompactInt::<BYTES>::from_int(777u32);
        let y = CompactInt::<BYTES>::from_int(777u32);
        assert_eq!(x, y, "{:?} ≠ {:?}", x.0, y.0);
    }

    #[test]
    fn test_ord() {
        let x = CompactInt::<BYTES>::from_int(123u32);
        let y = CompactInt::<BYTES>::from_int(777u32);
        assert!(x < y, "{:?} ≥ {:?}", x.0, y.0);
    }
}
