// Inspired by https://github.com/Daniel-Liu-c0deb0t/simple-saca/blob/main/src/compact_vec.rs

#![allow(dead_code)]

use core::cmp::Ordering;
use num_traits::sign::Unsigned;
use num_traits::PrimInt;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SlicedInt<const BYTES: usize>([u8; BYTES]);

impl<const BYTES: usize> SlicedInt<BYTES> {
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
    pub fn from_le_bytes(bytes: &[u8]) -> Self {
        let mut res = Self::new();
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), res.0.as_mut_ptr(), BYTES);
        }
        res
    }

    #[inline(always)]
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        let mut le_bytes = [0u8; BYTES];
        le_bytes.copy_from_slice(bytes);
        le_bytes.reverse();
        Self::from_le_bytes(&le_bytes)
    }

    #[inline(always)]
    pub fn to_le_bytes(self) -> [u8; BYTES] {
        self.0
    }

    #[inline(always)]
    pub fn to_be_bytes(self) -> [u8; BYTES] {
        let mut bytes = self.0;
        bytes.reverse();
        bytes
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

impl<const BYTES: usize> Default for SlicedInt<BYTES> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BYTES: usize> Ord for SlicedInt<BYTES> {
    #[inline]
    fn cmp(&self, other: &SlicedInt<BYTES>) -> Ordering {
        for i in (0..BYTES).rev() {
            match self.0[i].cmp(&other.0[i]) {
                Ordering::Equal => (),
                non_eq => return non_eq,
            }
        }
        Ordering::Equal
    }
}

impl<const BYTES: usize> PartialOrd for SlicedInt<BYTES> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const BYTES: usize> Serialize for SlicedInt<BYTES> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

struct SlicedIntVisitor<const BYTES: usize> {}

impl<'de, const BYTES: usize> Visitor<'de> for SlicedIntVisitor<BYTES> {
    type Value = SlicedInt<BYTES>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("an integer sliced into bytes")
    }

    fn visit_bytes<E: std::error::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
        Ok(SlicedInt::from_le_bytes(bytes))
    }
}

impl<'de, const BYTES: usize> Deserialize<'de> for SlicedInt<BYTES> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_bytes(SlicedIntVisitor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BYTES: usize = 3;

    #[test]
    fn test_from_get() {
        let x = SlicedInt::<BYTES>::from_int(442u32);
        assert_eq!(x.get::<u32>(), 442, "{:?}", x.0);
    }

    #[test]
    fn test_set_get() {
        let mut x = SlicedInt::<BYTES>::new();
        x.set(631u32);
        assert_eq!(x.get::<u32>(), 631, "{:?}", x.0);
        x.set(363u32);
        assert_eq!(x.get::<u32>(), 363, "{:?}", x.0);
    }

    #[test]
    fn test_eq() {
        let x = SlicedInt::<BYTES>::from_int(777u32);
        let y = SlicedInt::<BYTES>::from_int(777u32);
        assert_eq!(x, y, "{:?} ≠ {:?}", x.0, y.0);
    }

    #[test]
    fn test_ord() {
        let x = SlicedInt::<BYTES>::from_int(123u32);
        let y = SlicedInt::<BYTES>::from_int(777u32);
        assert!(x < y, "{:?} ≥ {:?}", x.0, y.0);
    }
}
