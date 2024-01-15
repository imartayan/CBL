use crate::kmer::{Base, Kmer, RawKmer};
use crate::necklace::*;
use crate::wordset::*;
use core::cmp::min;
use core::ops::*;
use std::collections::BTreeMap;

pub struct CBL<const K: usize, KT: Base, const PREFIX_BITS: usize = 24, const M: usize = 9>
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
    necklace_queue: NecklaceQueue<{ 2 * K }, KT, { (2 * K).saturating_sub(M - 1) }>,
}

macro_rules! impl_cbl {
    ($KT:ty) => {
        impl<const K: usize, const PREFIX_BITS: usize, const M: usize> CBL<K, $KT, PREFIX_BITS, M>
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
                    necklace_queue: NecklaceQueue::<
                        { 2 * K },
                        $KT,
                        { (2 * K).saturating_sub(M - 1) },
                    >::new(),
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
            fn merge_necklace_pos(necklace: $KT, pos: usize) -> $KT {
                necklace * Self::KMER_BITS as $KT + pos as $KT
                // (necklace << P) | (pos as $KT)
            }

            #[inline]
            fn split_necklace_pos(word: $KT) -> ($KT, usize) {
                (
                    word / (Self::KMER_BITS as $KT),
                    (word % (Self::KMER_BITS as $KT)) as usize,
                )
            }

            #[inline]
            fn get_word<KmerT: Kmer<K, $KT>>(kmer: KmerT) -> $KT {
                let (necklace, pos) = necklace_pos::<{ 2 * K }, $KT>(kmer.to_int());
                Self::merge_necklace_pos(necklace, pos)
            }

            #[inline]
            fn recover_kmer(word: $KT) -> RawKmer<K, $KT> {
                let (necklace, pos) = Self::split_necklace_pos(word);
                RawKmer::<K, $KT>::from_int(revert_necklace_pos::<{ 2 * K }, $KT>(necklace, pos))
            }

            #[inline]
            pub fn contains<KmerT: Kmer<K, $KT>>(&self, kmer: KmerT) -> bool {
                self.wordset.contains(Self::get_word(kmer))
            }

            #[inline]
            pub fn insert<KmerT: Kmer<K, $KT>>(&mut self, kmer: KmerT) -> bool {
                self.wordset.insert(Self::get_word(kmer))
            }

            #[inline]
            pub fn remove<KmerT: Kmer<K, $KT>>(&mut self, kmer: KmerT) -> bool {
                self.wordset.remove(Self::get_word(kmer))
            }

            #[inline]
            fn get_seq_chunks(seq: &'_ [u8]) -> impl Iterator<Item = &'_ [u8]> {
                (0..(seq.len() - K + 1))
                    .step_by(Self::CHUNK_SIZE)
                    .map(|start| &seq[start..min(start + Self::CHUNK_SIZE + K - 1, seq.len())])
            }

            #[inline]
            fn get_seq_words(&mut self, seq: &[u8]) -> Vec<$KT> {
                let mut res = Vec::with_capacity(seq.len() - K + 1);
                let kmer = RawKmer::<K, $KT>::from_nucs(&seq[..K]);
                self.necklace_queue.insert_full(kmer.to_int());
                let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                res.push(Self::merge_necklace_pos(necklace, pos));
                for base in seq[K..].iter().filter_map(<$KT>::from_nuc) {
                    self.necklace_queue.insert2(base);
                    let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                    res.push(Self::merge_necklace_pos(necklace, pos));
                }
                res
            }

            #[inline]
            pub fn contains_seq(&mut self, seq: &[u8]) -> Vec<bool> {
                assert!(seq.len() >= K);
                let mut res = Vec::with_capacity(seq.len() - K + 1);
                for chunk in Self::get_seq_chunks(seq) {
                    let words = self.get_seq_words(chunk);
                    res.append(&mut self.wordset.contains_batch(&words));
                }
                res
            }

            #[inline]
            pub fn insert_seq(&mut self, seq: &[u8]) {
                assert!(seq.len() >= K);
                for chunk in Self::get_seq_chunks(seq) {
                    let words = self.get_seq_words(chunk);
                    self.wordset.insert_batch(&words);
                }
            }

            #[inline]
            pub fn remove_seq(&mut self, seq: &[u8]) {
                assert!(seq.len() >= K);
                for chunk in Self::get_seq_chunks(seq) {
                    let words = self.get_seq_words(chunk);
                    self.wordset.remove_batch(&words);
                }
            }

            #[inline]
            pub fn iter(&self) -> impl Iterator<Item = RawKmer<K, $KT>> + '_ {
                self.wordset.iter::<$KT>().map(Self::recover_kmer)
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

        impl<const K: usize, const PREFIX_BITS: usize, const M: usize> Default
            for CBL<K, $KT, PREFIX_BITS, M>
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

        impl<const K: usize, const PREFIX_BITS: usize, const M: usize> BitOrAssign<&mut Self>
            for CBL<K, $KT, PREFIX_BITS, M>
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

        impl<const K: usize, const PREFIX_BITS: usize, const M: usize> BitAndAssign<&mut Self>
            for CBL<K, $KT, PREFIX_BITS, M>
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

        impl<const K: usize, const PREFIX_BITS: usize, const M: usize> SubAssign<&mut Self>
            for CBL<K, $KT, PREFIX_BITS, M>
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

        impl<const K: usize, const PREFIX_BITS: usize, const M: usize> BitXorAssign<&mut Self>
            for CBL<K, $KT, PREFIX_BITS, M>
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
    type KmerT = RawKmer<K, T>;
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
