use crate::kmer::{Base, IntKmer, Kmer};
use crate::necklace::*;
use crate::wordset::*;
use core::cmp::min;
use core::ops::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const M: usize = 9;

#[derive(Serialize, Deserialize)]
pub struct CBL<const K: usize, T: Base, const PREFIX_BITS: usize = 24>
where
    [(); (2 * K + (2 * K).next_power_of_two().ilog2() as usize)
        .saturating_sub(PREFIX_BITS)
        .div_ceil(8)]:,
    [(); PREFIX_BITS.div_ceil(8)]:,
    [(); (2 * K).saturating_sub(M - 1)]:,
{
    wordset: WordSet<
        PREFIX_BITS,
        { (2 * K + (2 * K).next_power_of_two().ilog2() as usize).saturating_sub(PREFIX_BITS) },
    >,
    #[serde(skip)]
    necklace_queue: NecklaceQueue<{ 2 * K }, T, { (2 * K).saturating_sub(M - 1) }>,
}

macro_rules! impl_cbl {
    ($T:ty) => {
        impl<const K: usize, const PREFIX_BITS: usize> CBL<K, $T, PREFIX_BITS>
        where
            [(); (2 * K + (2 * K).next_power_of_two().ilog2() as usize)
                .saturating_sub(PREFIX_BITS)
                .div_ceil(8)]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); (2 * K).saturating_sub(M - 1)]:,
        {
            const KMER_BITS: usize = 2 * K;
            const CHUNK_SIZE: usize = 2048;

            pub fn new() -> Self {
                Self {
                    wordset: WordSet::new(),
                    necklace_queue:
                        NecklaceQueue::<{ 2 * K }, $T, { (2 * K).saturating_sub(M - 1) }>::new(),
                }
            }

            pub fn count(&self) -> usize {
                self.wordset.count()
            }

            #[inline]
            pub fn is_empty(&self) -> bool {
                self.wordset.is_empty()
            }

            #[inline]
            fn merge_necklace_pos(necklace: $T, pos: usize) -> $T {
                necklace * Self::KMER_BITS as $T + pos as $T
                // (necklace << P) | (pos as $T)
            }

            #[inline]
            fn split_necklace_pos(word: $T) -> ($T, usize) {
                (
                    word / (Self::KMER_BITS as $T),
                    (word % (Self::KMER_BITS as $T)) as usize,
                )
            }

            #[inline]
            fn get_word(kmer: IntKmer<K, $T>) -> $T {
                let (necklace, pos) = necklace_pos::<{ 2 * K }, $T>(kmer.to_int());
                Self::merge_necklace_pos(necklace, pos)
            }

            #[inline]
            fn recover_kmer(word: $T) -> IntKmer<K, $T> {
                let (necklace, pos) = Self::split_necklace_pos(word);
                IntKmer::<K, $T>::from_int(revert_necklace_pos::<{ 2 * K }, $T>(necklace, pos))
            }

            #[inline]
            pub fn contains(&self, kmer: IntKmer<K, $T>) -> bool {
                self.wordset.contains(Self::get_word(kmer))
            }

            #[inline]
            pub fn insert(&mut self, kmer: IntKmer<K, $T>) -> bool {
                self.wordset.insert(Self::get_word(kmer))
            }

            #[inline]
            pub fn remove(&mut self, kmer: IntKmer<K, $T>) -> bool {
                self.wordset.remove(Self::get_word(kmer))
            }

            #[inline]
            fn get_seq_chunks(seq: &'_ [u8]) -> impl Iterator<Item = &'_ [u8]> {
                (0..(seq.len() - K + 1))
                    .step_by(Self::CHUNK_SIZE)
                    .map(|start| &seq[start..min(start + Self::CHUNK_SIZE + K - 1, seq.len())])
            }

            #[inline]
            fn get_seq_words(&mut self, seq: &[u8]) -> Vec<$T> {
                let mut res = Vec::with_capacity(seq.len() - K + 1);
                let kmer = IntKmer::<K, $T>::from_nucs(&seq[..K]);
                self.necklace_queue.insert_full(kmer.to_int());
                let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                res.push(Self::merge_necklace_pos(necklace, pos));
                for base in seq[K..].iter().filter_map(<$T>::from_nuc) {
                    self.necklace_queue.insert2(base);
                    let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                    res.push(Self::merge_necklace_pos(necklace, pos));
                }
                res
            }

            #[inline]
            pub fn contains_all(&mut self, seq: &[u8]) -> bool {
                assert!(
                    seq.len() >= K,
                    "Sequence size ({}) is smaller than K ({})",
                    seq.len(),
                    K
                );
                for chunk in Self::get_seq_chunks(seq) {
                    let words = self.get_seq_words(chunk);
                    if !self.wordset.contains_all(&words) {
                        return false;
                    }
                }
                true
            }

            #[inline]
            pub fn contains_seq(&mut self, seq: &[u8]) -> Vec<bool> {
                assert!(
                    seq.len() >= K,
                    "Sequence size ({}) is smaller than K ({})",
                    seq.len(),
                    K
                );
                let mut res = Vec::with_capacity(seq.len() - K + 1);
                for chunk in Self::get_seq_chunks(seq) {
                    let words = self.get_seq_words(chunk);
                    res.append(&mut self.wordset.contains_batch(&words));
                }
                res
            }

            #[inline]
            pub fn insert_seq(&mut self, seq: &[u8]) {
                assert!(
                    seq.len() >= K,
                    "Sequence size ({}) is smaller than K ({})",
                    seq.len(),
                    K
                );
                for chunk in Self::get_seq_chunks(seq) {
                    let words = self.get_seq_words(chunk);
                    self.wordset.insert_batch(&words);
                }
            }

            #[inline]
            pub fn remove_seq(&mut self, seq: &[u8]) {
                assert!(
                    seq.len() >= K,
                    "Sequence size ({}) is smaller than K ({})",
                    seq.len(),
                    K
                );
                for chunk in Self::get_seq_chunks(seq) {
                    let words = self.get_seq_words(chunk);
                    self.wordset.remove_batch(&words);
                }
            }

            #[inline]
            pub fn iter(&self) -> impl Iterator<Item = IntKmer<K, $T>> + '_ {
                self.wordset.iter::<$T>().map(Self::recover_kmer)
            }

            #[inline]
            pub fn prefix_load(&self) -> f64 {
                self.wordset.prefix_load()
            }

            #[inline]
            pub fn suffix_load(&self) -> f64 {
                self.wordset.suffix_load()
            }

            #[inline]
            pub fn suffix_load_repartition(&self) -> BTreeMap<usize, f64> {
                self.wordset.suffix_load_repartition()
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> Default for CBL<K, $T, PREFIX_BITS>
        where
            [(); (2 * K + (2 * K).next_power_of_two().ilog2() as usize)
                .saturating_sub(PREFIX_BITS)
                .div_ceil(8)]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); (2 * K).saturating_sub(M - 1)]:,
        {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitOrAssign<&mut Self>
            for CBL<K, $T, PREFIX_BITS>
        where
            [(); (2 * K + (2 * K).next_power_of_two().ilog2() as usize)
                .saturating_sub(PREFIX_BITS)
                .div_ceil(8)]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); (2 * K).saturating_sub(M - 1)]:,
        {
            fn bitor_assign(&mut self, other: &mut Self) {
                self.wordset |= &mut other.wordset;
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitAndAssign<&mut Self>
            for CBL<K, $T, PREFIX_BITS>
        where
            [(); (2 * K + (2 * K).next_power_of_two().ilog2() as usize)
                .saturating_sub(PREFIX_BITS)
                .div_ceil(8)]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); (2 * K).saturating_sub(M - 1)]:,
        {
            fn bitand_assign(&mut self, other: &mut Self) {
                self.wordset &= &mut other.wordset;
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> SubAssign<&mut Self>
            for CBL<K, $T, PREFIX_BITS>
        where
            [(); (2 * K + (2 * K).next_power_of_two().ilog2() as usize)
                .saturating_sub(PREFIX_BITS)
                .div_ceil(8)]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); (2 * K).saturating_sub(M - 1)]:,
        {
            fn sub_assign(&mut self, other: &mut Self) {
                self.wordset -= &mut other.wordset;
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitXorAssign<&mut Self>
            for CBL<K, $T, PREFIX_BITS>
        where
            [(); (2 * K + (2 * K).next_power_of_two().ilog2() as usize)
                .saturating_sub(PREFIX_BITS)
                .div_ceil(8)]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); (2 * K).saturating_sub(M - 1)]:,
        {
            fn bitxor_assign(&mut self, other: &mut Self) {
                self.wordset ^= &mut other.wordset;
            }
        }
    };
}

impl_cbl!(u32);
impl_cbl!(u64);
impl_cbl!(u128);

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use rand::thread_rng;
    use rand::Rng;

    const K: usize = 59;
    type T = u128;
    type KmerT = IntKmer<K, T>;
    const N: usize = 1_000_000;

    #[test]
    fn test_insert_contains_remove() {
        let mut rng = thread_rng();
        let mut nucs = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut set = CBL::<K, T>::new();
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            set.insert(kmer);
        }
        for (i, kmer) in KmerT::iter_from_nucs(nucs.iter()).enumerate() {
            assert!(
                set.contains(kmer),
                "kmer {i} false negative: {:0b}",
                kmer.to_int()
            );
        }
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            set.remove(kmer);
        }
        for (i, kmer) in KmerT::iter_from_nucs(nucs.iter()).enumerate() {
            assert!(
                !set.contains(kmer),
                "kmer {i} false positive: {:0b}",
                kmer.to_int()
            );
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_batch_operations() {
        let mut rng = thread_rng();
        let mut nucs = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut set = CBL::<K, T>::new();
        set.insert_seq(&nucs);
        assert!(set.contains_seq(&nucs).iter().all(|&b| b));
        set.remove_seq(&nucs);
        for (i, kmer) in KmerT::iter_from_nucs(nucs.iter()).enumerate() {
            assert!(
                !set.contains(kmer),
                "kmer {i} false positive: {:0b}",
                kmer.to_int()
            );
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut set = CBL::<K, T>::new();
        let kmers = (0..1000u128).step_by(7).map(KmerT::from_int).collect_vec();
        for &kmer in kmers.iter() {
            set.insert(kmer);
        }
        let mut res = set.iter().collect_vec();
        res.sort_unstable();
        assert_eq!(res, kmers);
    }

    #[test]
    fn test_union() {
        let mut rng = thread_rng();
        let mut nucs = Vec::with_capacity(N);
        let mut nucs2 = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
            nucs2.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut set = CBL::<K, T>::new();
        let mut set2 = CBL::<K, T>::new();
        set.insert_seq(&nucs);
        set2.insert_seq(&nucs2);
        set |= &mut set2;
        assert!(set.contains_seq(&nucs).iter().all(|&b| b));
        assert!(set.contains_seq(&nucs2).iter().all(|&b| b));
    }

    #[test]
    fn test_intersection() {
        let mut rng = thread_rng();
        let mut nucs = Vec::with_capacity(N);
        let mut nucs2 = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
            nucs2.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut set = CBL::<K, T>::new();
        let mut set2 = CBL::<K, T>::new();
        set.insert_seq(&nucs);
        set2.insert_seq(&nucs2);
        set &= &mut set2;
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            assert_eq!(set.contains(kmer), set2.contains(kmer));
        }
    }

    #[test]
    fn test_difference() {
        let mut rng = thread_rng();
        let mut nucs = Vec::with_capacity(N);
        let mut nucs2 = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
            nucs2.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut set = CBL::<K, T>::new();
        let mut set2 = CBL::<K, T>::new();
        set.insert_seq(&nucs);
        set2.insert_seq(&nucs2);
        set -= &mut set2;
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            assert_eq!(set.contains(kmer), !set2.contains(kmer));
        }
    }

    #[test]
    fn test_symmetric_difference() {
        let mut rng = thread_rng();
        let mut nucs = Vec::with_capacity(N);
        let mut nucs2 = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
            nucs2.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut set = CBL::<K, T>::new();
        let mut set2 = CBL::<K, T>::new();
        set.insert_seq(&nucs);
        set2.insert_seq(&nucs2);
        set ^= &mut set2;
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            assert_eq!(set.contains(kmer), !set2.contains(kmer));
        }
    }
}
