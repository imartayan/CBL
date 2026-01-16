#![allow(dead_code)]

use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Clone)]
pub struct TinyBitvector([u64; 4]);

impl TinyBitvector {
    #[inline(always)]
    pub fn new() -> Self {
        Self([0u64; 4])
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    #[inline(always)]
    pub fn count(&self) -> usize {
        (self.0[0].count_ones()
            + self.0[1].count_ones()
            + self.0[2].count_ones()
            + self.0[3].count_ones()) as usize
    }

    #[inline(always)]
    pub fn contains(&self, index: u8) -> bool {
        self.0[index as usize / 64] & (1 << (index as u64 % 64)) != 0
    }

    #[inline]
    pub fn insert(&mut self, index: u8) -> bool {
        let old = self.0[index as usize / 64];
        self.0[index as usize / 64] = old | (1 << (index as u64 % 64));
        self.0[index as usize / 64] != old
    }

    #[inline]
    pub fn remove(&mut self, index: u8) -> bool {
        let old = self.0[index as usize / 64];
        self.0[index as usize / 64] = old & !(1 << (index as u64 % 64));
        self.0[index as usize / 64] != old
    }

    #[inline]
    pub fn rank(&self, index: u8) -> usize {
        let rank = (self.0[index as usize / 64] & ((1 << (index as u64 % 64)) - 1)).count_ones();
        match index / 64 {
            0 => rank as usize,
            1 => (rank + self.0[0].count_ones()) as usize,
            2 => (rank + self.0[0].count_ones() + self.0[1].count_ones()) as usize,
            3 => {
                (rank + self.0[0].count_ones() + self.0[1].count_ones() + self.0[2].count_ones())
                    as usize
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> TinyBitvectorIterator<'_> {
        TinyBitvectorIterator {
            blocks: &self.0,
            block_index: 0,
            block: self.0[0],
        }
    }
}

pub struct TinyBitvectorIterator<'a> {
    blocks: &'a [u64; 4],
    block_index: usize,
    block: u64,
}

impl<'a> Iterator for TinyBitvectorIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        while self.block == 0 {
            self.block_index += 1;
            if self.block_index >= 4 {
                return None;
            }
            self.block = self.blocks[self.block_index];
        }
        let bit_index = self.block.trailing_zeros() as u8;
        self.block -= 1 << bit_index;
        Some(self.block_index as u8 * 64 + bit_index)
    }
}

impl Serialize for TinyBitvector {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.count()))?;
        for index in self.iter() {
            seq.serialize_element(&index)?;
        }
        seq.end()
    }
}

struct TinyBitvectorVisitor {}

impl<'de> Visitor<'de> for TinyBitvectorVisitor {
    type Value = TinyBitvector;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a small bitvector")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut access: S) -> Result<Self::Value, S::Error> {
        let mut bv = TinyBitvector::new();
        while let Some(index) = access.next_element()? {
            bv.insert(index);
        }
        Ok(bv)
    }
}

impl<'de> Deserialize<'de> for TinyBitvector {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(TinyBitvectorVisitor {})
    }
}
