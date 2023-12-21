use crate::kmer::{Base, Kmer, RawKmer};
use crate::necklace::queue::{necklace_pos, NecklaceQueue};
use crate::wordset::*;
use std::collections::BTreeMap;

pub struct CBL<const K: usize, KT: Base, const PREFIX_BITS: usize = 24, const M: usize = 9>
where
    [(); (2 * K + (2 * K).next_power_of_two().ilog2() as usize)
        .saturating_sub(PREFIX_BITS)
        .div_ceil(8)]:,
    [(); PREFIX_BITS.div_ceil(8)]:,
    [(); (2 * K).saturating_sub(M - 1)]:,
{
    wordset: BitWordSet<
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

            pub fn new() -> Self {
                Self {
                    wordset: BitWordSet::new(),
                    necklace_queue: NecklaceQueue::<
                        { 2 * K },
                        $KT,
                        { (2 * K).saturating_sub(M - 1) },
                    >::new(),
                }
            }

            #[inline]
            fn merge_necklace_pos(necklace: $KT, pos: usize) -> $KT {
                necklace * Self::KMER_BITS as $KT + pos as $KT
                // (necklace << P) | (pos as $KT)
            }

            #[inline]
            fn get_word<KmerT: Kmer<K, $KT>>(kmer: KmerT) -> $KT {
                let (necklace, pos) = necklace_pos::<{ 2 * K }, $KT>(kmer.to_int());
                Self::merge_necklace_pos(necklace, pos)
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
            pub fn insert_seq(&mut self, seq: &[u8]) {
                assert!(seq.len() >= K);
                let words = self.get_seq_words(seq);
                self.wordset.insert_batch(&words);
            }

            #[inline]
            pub fn remove_seq(&mut self, seq: &[u8]) {
                assert!(seq.len() >= K);
                let words = self.get_seq_words(seq);
                self.wordset.remove_batch(&words);
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
    };
}

impl_cbl!(u32);
impl_cbl!(u64);
impl_cbl!(u128);
