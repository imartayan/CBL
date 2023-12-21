pub mod minimizer;
pub mod queue;
pub mod rank;

use crate::kmer::{Base, Kmer};
use num_traits::int::PrimInt;
use queue::NecklaceQueue;

pub fn necklace_pos<const BITS: usize, T: PrimInt>(word: T) -> (T, usize) {
    let mut necklace = word;
    let mut rot = word;
    let mut pos = 0;
    for i in 1..BITS {
        rot = ((rot & T::one()) << (BITS - 1)) | (rot >> 1);
        if rot < necklace {
            necklace = rot;
            pos = i;
        }
    }
    (necklace, pos)
}

#[derive(Debug)]
pub struct KmerNecklaceFactory<const K: usize, const BITS: usize, T: Base, const M: usize = 9>
where
    [(); BITS - M + 1]:,
{
    necklace_queue: NecklaceQueue<BITS, T, { BITS - M + 1 }>,
}

macro_rules! impl_necklace_factory {
($($T:ty),+) => {$(
impl<const K: usize, const BITS: usize, const M: usize> KmerNecklaceFactory<K, BITS, $T, M>
where
    [(); BITS - M + 1]:,
{
    pub fn new() -> Self {
        Self {
            necklace_queue: NecklaceQueue::<BITS, $T, { BITS - M + 1 }>::new(),
        }
    }

    pub fn from_kmer<KmerT: Kmer<K, $T>>(&self, kmer: KmerT) -> ($T, usize) {
        necklace_pos::<BITS, $T>(kmer.to_int())
    }

    pub fn from_bases(&mut self, bases: &[$T]) -> Vec<($T, usize)> {
        let mut res = Vec::with_capacity(bases.len() - K + 1);
        let mut word = 0;
        for &base in &bases[..K] {
            word = (word << 2) | base;
        }
        self.necklace_queue.insert_full(word);
        res.push(self.necklace_queue.get_necklace_pos());
        for &base in &bases[K..] {
            self.necklace_queue.insert2(base);
            res.push(self.necklace_queue.get_necklace_pos());
        }
        res
    }
}

impl<const K: usize, const BITS: usize, const M: usize> Default
    for KmerNecklaceFactory<K, BITS, $T, M>
where
    [(); BITS - M + 1]:,
{
    fn default() -> Self {
        Self::new()
    }
}
)*}}

impl_necklace_factory!(u8, u16, u32, u64, u128);
