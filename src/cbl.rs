//! Fully dynamic sets of *k*-mers.
#![allow(clippy::suspicious_arithmetic_impl)]

use crate::kmer::{Base, IntKmer, Kmer, RevComp};
use crate::necklace::*;
use crate::wordset::*;
use bincode::{DefaultOptions, Options};
use core::cmp::min;
use core::ops::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const M: usize = 9;

/// Size of a *k*-mer in bits
pub const fn kmer_bits<const K: usize>() -> usize {
    2 * K
}

/// Width of the necklace queue
pub const fn queue_width<const K: usize>() -> usize {
    kmer_bits::<K>().saturating_sub(M - 1)
}

/// Size of the suffixes in bits
pub const fn suffix_bits<const K: usize, const PREFIX_BITS: usize>() -> usize {
    (kmer_bits::<K>() + kmer_bits::<K>().next_power_of_two().ilog2() as usize)
        .saturating_sub(PREFIX_BITS)
}

/// A fully dynamic set of *k*-mers.
///
/// # Type Parameters
/// - `K`: the length of the *k*-mers, it must be ≤ 59.
/// - `T`: the integer type used to store *k*-mers, it must be large enough to store $2k + \lg(2k)$ bits.
/// - `PREFIX_BITS` (optional): the size of the prefixes in bits.
#[derive(Clone, Serialize, Deserialize)]
pub struct CBL<const K: usize, T: Base, const PREFIX_BITS: usize = 24>
where
    [(); kmer_bits::<K>()]:,
    [(); PREFIX_BITS.div_ceil(8)]:,
    [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
    [(); queue_width::<K>()]:,
{
    canonical: bool,
    wordset: WordSet<PREFIX_BITS, { suffix_bits::<K, PREFIX_BITS>() }>,
    #[serde(skip)]
    necklace_queue: NecklaceQueue<{ kmer_bits::<K>() }, T, { queue_width::<K>() }>,
    #[serde(skip)]
    necklace_queue_rev: NecklaceQueue<{ kmer_bits::<K>() }, T, { queue_width::<K>() }, true>,
}

macro_rules! impl_cbl {
    ($T:ty) => {
        impl<const K: usize, const PREFIX_BITS: usize> CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            const KMER_BITS: usize = 2 * K;
            const POS_BITS: usize = Self::KMER_BITS.next_power_of_two().ilog2() as usize;
            const CHUNK_SIZE: usize = 2048;

            /// Creates an empty [`CBL`].
            #[inline]
            pub fn new() -> Self {
                Self::new_with_wordset(WordSet::new(), false)
            }

            /// Creates an empty [`CBL`] for canonical *k*-mers.
            #[inline]
            pub fn new_canonical() -> Self {
                Self::new_with_wordset(WordSet::new(), true)
            }

            /// Creates a [`CBL`] with the given wordset.
            #[inline]
            fn new_with_wordset(
                wordset: WordSet<PREFIX_BITS, { suffix_bits::<K, PREFIX_BITS>() }>,
                canonical: bool,
            ) -> Self {
                assert!(
                    Self::KMER_BITS + Self::POS_BITS <= <$T>::BITS as usize,
                    "Cannot fit a {K}-mer and its length in a {}-bit integer",
                    <$T>::BITS
                );
                Self {
                    canonical,
                    wordset,
                    necklace_queue:
                        NecklaceQueue::<{ kmer_bits::<K>() }, $T, { queue_width::<K>() }>::new(),
                    necklace_queue_rev: NecklaceQueue::<
                        { kmer_bits::<K>() },
                        $T,
                        { queue_width::<K>() },
                        true,
                    >::new(),
                }
            }

            /// Saves the set to a file.
            pub fn save_to_file<P: AsRef<Path> + Copy>(&self, path: P) {
                let index_file = File::create(path).unwrap_or_else(|_| {
                    panic!("Failed to create {}", path.as_ref().to_str().unwrap())
                });
                let mut writer = BufWriter::new(index_file);
                DefaultOptions::new()
                    .with_varint_encoding()
                    .reject_trailing_bytes()
                    .serialize_into(&mut writer, self)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Failed to write index to {}",
                            path.as_ref().to_str().unwrap()
                        )
                    });
            }

            /// Loads the set from a file.
            pub fn load_from_file<P: AsRef<Path> + Copy>(path: P) -> Self {
                let index_file = File::open(path).unwrap_or_else(|_| {
                    panic!("Failed to open {}", path.as_ref().to_str().unwrap())
                });
                let reader = BufReader::new(index_file);
                DefaultOptions::new()
                    .with_varint_encoding()
                    .reject_trailing_bytes()
                    .deserialize_from(reader)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Failed to load index from {}",
                            path.as_ref().to_str().unwrap()
                        )
                    })
            }

            /// Returns `true` if the set stores canonical *k*-mers.
            #[inline]
            pub fn is_canonical(&self) -> bool {
                self.canonical
            }

            /// Counts the number of *k*-mers in the set.
            pub fn count(&self) -> usize {
                self.wordset.count()
            }

            /// Returns `true` if there are no *k*-mers in the set.
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.wordset.is_empty()
            }

            /// Packs a necklace and its position into a single integer.
            #[inline]
            fn merge_necklace_pos(necklace: $T, pos: usize) -> $T {
                // necklace * Self::KMER_BITS as $T + pos as $T
                (necklace << Self::POS_BITS) | (pos as $T)
            }

            /// Unpacks a necklace and its position from a single integer.
            #[inline]
            fn split_necklace_pos(word: $T) -> ($T, usize) {
                (
                    // word / (Self::KMER_BITS as $T),
                    // (word % (Self::KMER_BITS as $T)) as usize,
                    word >> Self::POS_BITS,
                    (word & ((1 << Self::POS_BITS) - 1)) as usize,
                )
            }

            /// Returns the necklace transformation of a *k*-mer.
            #[inline]
            fn get_word(&self, kmer: IntKmer<K, $T>) -> $T {
                let (necklace, pos) = necklace_pos::<{ kmer_bits::<K>() }, $T>(if self.canonical {
                    kmer.canonical().to_int()
                } else {
                    kmer.to_int()
                });
                Self::merge_necklace_pos(necklace, pos)
            }

            /// Recovers a *k*-mer from its necklace transformation.
            #[inline]
            fn recover_kmer(word: $T) -> IntKmer<K, $T> {
                let (necklace, pos) = Self::split_necklace_pos(word);
                IntKmer::<K, $T>::from_int(revert_necklace_pos::<{ kmer_bits::<K>() }, $T>(
                    necklace, pos,
                ))
            }

            /// Returns `true` if the set contains the given *k*-mer, the *k*-mer must be packed into an [`IntKmer`].
            #[inline]
            pub fn contains(&self, kmer: IntKmer<K, $T>) -> bool {
                self.wordset.contains(self.get_word(kmer))
            }

            /// Adds a *k*-mer to the set, the *k*-mer must be packed into an [`IntKmer`].
            /// Returns `true` the *k*-mer was absent from the set.
            #[inline]
            pub fn insert(&mut self, kmer: IntKmer<K, $T>) -> bool {
                self.wordset.insert(self.get_word(kmer))
            }

            /// Removes a *k*-mer to the set, the *k*-mer must be packed into an [`IntKmer`].
            /// Returns `true` the *k*-mer was present in the set.
            #[inline]
            pub fn remove(&mut self, kmer: IntKmer<K, $T>) -> bool {
                self.wordset.remove(self.get_word(kmer))
            }

            /// Splits a sequence into chunks of size `Self::CHUNK_SIZE` and returns an iterator over the chunks.
            #[inline]
            fn get_seq_chunks(seq: &'_ [u8]) -> impl Iterator<Item = &'_ [u8]> {
                (0..(seq.len() - K + 1))
                    .step_by(Self::CHUNK_SIZE)
                    .map(|start| &seq[start..min(start + Self::CHUNK_SIZE + K - 1, seq.len())])
            }

            /// Returns the necklace transformations of the *k*-mers contained in a sequence.
            #[inline]
            fn get_seq_words(&mut self, seq: &[u8]) -> Vec<$T> {
                if self.canonical {
                    let mut res = Vec::with_capacity(seq.len() - K + 1);
                    let mut res_rc = Vec::with_capacity(seq.len() - K + 1);
                    let mut kmer = IntKmer::<K, $T>::from_nucs(&seq[..K]);
                    self.necklace_queue.insert_full(kmer.to_int());
                    self.necklace_queue_rev
                        .insert_full(kmer.rev_comp().to_int());
                    if kmer.is_canonical() {
                        let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                        res.push(Self::merge_necklace_pos(necklace, pos));
                    } else {
                        let (necklace, pos) = self.necklace_queue_rev.get_necklace_pos();
                        res_rc.push(Self::merge_necklace_pos(necklace, pos));
                    }
                    for base in seq[K..].iter().filter_map(<$T>::from_nuc) {
                        kmer = kmer.append(base);
                        self.necklace_queue.insert2(base);
                        self.necklace_queue_rev.insert2(base.complement());
                        if kmer.is_canonical() {
                            let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                            res.push(Self::merge_necklace_pos(necklace, pos));
                        } else {
                            let (necklace, pos) = self.necklace_queue_rev.get_necklace_pos();
                            res_rc.push(Self::merge_necklace_pos(necklace, pos));
                        }
                    }
                    res.append(&mut res_rc);
                    res
                } else {
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
            }

            /// Returns `true` if the set contains all the *k*-mers of a sequence.
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

            /// For each *k*-mer of a sequence, returns `true` if it is contained in the set.
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

            /// Adds all the *k*-mers of a sequence to the set.
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

            /// Removes all the *k*-mers of a sequence from the set.
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

            /// Returns an iterator over the *k*-mers of the set.
            #[inline]
            pub fn iter(&self) -> impl Iterator<Item = IntKmer<K, $T>> + '_ {
                self.wordset.iter::<$T>().map(Self::recover_kmer)
            }

            /// Returns the proportion of available prefixes used in the set.
            #[inline]
            pub fn prefix_load(&self) -> f64 {
                self.wordset.prefix_load()
            }

            /// Returns an iterator over the prefixes of the set, with the size of the associated buckets.
            #[inline]
            pub fn buckets_sizes(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
                self.wordset.buckets_sizes()
            }

            /// Returns a Map storing the number of buckets of each size.
            #[inline]
            pub fn buckets_size_count(&self) -> BTreeMap<usize, usize> {
                self.wordset.buckets_size_count()
            }

            /// Returns a Map storing the proportion of buckets of each size.
            #[inline]
            pub fn buckets_load_repartition(&self) -> BTreeMap<usize, f64> {
                self.wordset.buckets_load_repartition()
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> Default for CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitOr<Self> for &mut CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            type Output = CBL<K, $T, PREFIX_BITS>;

            /// Perfom the union of two sets.
            fn bitor(self, other: Self) -> Self::Output {
                assert_eq!(
                    self.canonical, other.canonical,
                    "One of the index is canonical while the other isn't"
                );
                Self::Output::new_with_wordset(
                    &mut self.wordset | &mut other.wordset,
                    self.canonical,
                )
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitOrAssign<&mut Self>
            for CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            /// Perfom the union of `self` and `other` in place.
            fn bitor_assign(&mut self, other: &mut Self) {
                assert_eq!(
                    self.canonical, other.canonical,
                    "One of the index is canonical while the other isn't"
                );
                self.wordset |= &mut other.wordset;
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitAnd<Self> for &mut CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            type Output = CBL<K, $T, PREFIX_BITS>;

            /// Perfom the intersection of two sets.
            fn bitand(self, other: Self) -> Self::Output {
                assert_eq!(
                    self.canonical, other.canonical,
                    "One of the index is canonical while the other isn't"
                );
                Self::Output::new_with_wordset(
                    &mut self.wordset & &mut other.wordset,
                    self.canonical,
                )
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitAndAssign<&mut Self>
            for CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            /// Perform the intersection of `self` and `other` in place.
            fn bitand_assign(&mut self, other: &mut Self) {
                assert_eq!(
                    self.canonical, other.canonical,
                    "One of the index is canonical while the other isn't"
                );
                self.wordset &= &mut other.wordset;
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> Sub<Self> for &mut CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            type Output = CBL<K, $T, PREFIX_BITS>;

            /// Perfom the difference of two sets.
            fn sub(self, other: Self) -> Self::Output {
                assert_eq!(
                    self.canonical, other.canonical,
                    "One of the index is canonical while the other isn't"
                );
                Self::Output::new_with_wordset(
                    &mut self.wordset - &mut other.wordset,
                    self.canonical,
                )
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> SubAssign<&mut Self>
            for CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            /// Perform the difference of `self` and `other` in place.
            fn sub_assign(&mut self, other: &mut Self) {
                assert_eq!(
                    self.canonical, other.canonical,
                    "One of the index is canonical while the other isn't"
                );
                self.wordset -= &mut other.wordset;
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitXor<Self> for &mut CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            type Output = CBL<K, $T, PREFIX_BITS>;

            /// Perfom the intersection of two sets.
            fn bitxor(self, other: Self) -> Self::Output {
                assert_eq!(
                    self.canonical, other.canonical,
                    "One of the index is canonical while the other isn't"
                );
                Self::Output::new_with_wordset(
                    &mut self.wordset ^ &mut other.wordset,
                    self.canonical,
                )
            }
        }

        impl<const K: usize, const PREFIX_BITS: usize> BitXorAssign<&mut Self>
            for CBL<K, $T, PREFIX_BITS>
        where
            [(); kmer_bits::<K>()]:,
            [(); PREFIX_BITS.div_ceil(8)]:,
            [(); suffix_bits::<K, PREFIX_BITS>().div_ceil(8)]:,
            [(); queue_width::<K>()]:,
        {
            /// Perform the symmetric difference of `self` and `other` in place.
            fn bitxor_assign(&mut self, other: &mut Self) {
                assert_eq!(
                    self.canonical, other.canonical,
                    "One of the index is canonical while the other isn't"
                );
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
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use rand::{thread_rng, SeedableRng};

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
    fn test_random_insert_contains_remove() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut nucs = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut kmers = KmerT::iter_from_nucs(nucs.iter()).collect_vec();
        kmers.sort_unstable();
        kmers.dedup();
        kmers.shuffle(&mut rng);

        let mut set = CBL::<K, T>::new();
        for &kmer in kmers.iter() {
            assert!(set.insert(kmer));
        }
        kmers.shuffle(&mut rng);

        for (i, &kmer) in kmers.iter().enumerate() {
            assert!(
                set.contains(kmer),
                "kmer {i} false negative: {:0b}",
                kmer.to_int()
            );
        }
        kmers.shuffle(&mut rng);

        for &kmer in kmers.iter() {
            assert!(set.remove(kmer));
        }
        kmers.shuffle(&mut rng);

        for (i, &kmer) in kmers.iter().enumerate() {
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
    fn test_canonical() {
        let mut rng = thread_rng();
        let mut nucs = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut set = CBL::<K, T>::new_canonical();
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            set.insert(kmer);
        }
        for (i, kmer) in KmerT::iter_from_nucs(nucs.iter()).enumerate() {
            assert!(
                set.contains(kmer),
                "kmer {i} false negative: {:0b}",
                kmer.to_int()
            );
            assert!(
                set.contains(kmer.rev_comp()),
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
            assert!(
                !set.contains(kmer.rev_comp()),
                "kmer {i} false positive: {:0b}",
                kmer.to_int()
            );
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_canonical_batch() {
        let mut rng = thread_rng();
        let mut nucs = Vec::with_capacity(N);
        for _ in 0..N {
            nucs.push(u8::bases()[rng.gen_range(0..4)].to_nuc());
        }
        let mut set = CBL::<K, T>::new_canonical();
        set.insert_seq(&nucs);
        for (i, kmer) in KmerT::iter_from_nucs(nucs.iter()).enumerate() {
            assert!(
                set.contains(kmer),
                "kmer {i} false negative: {:0b}",
                kmer.to_int()
            );
            assert!(
                set.contains(kmer.rev_comp()),
                "kmer {i} false negative: {:0b}",
                kmer.to_int()
            );
        }
        set.remove_seq(&nucs);
        for (i, kmer) in KmerT::iter_from_nucs(nucs.iter()).enumerate() {
            assert!(
                !set.contains(kmer),
                "kmer {i} false positive: {:0b}",
                kmer.to_int()
            );
            assert!(
                !set.contains(kmer.rev_comp()),
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
        let mut res = &mut set | &mut set2;
        assert!(res.contains_seq(&nucs).iter().all(|&b| b));
        assert!(res.contains_seq(&nucs2).iter().all(|&b| b));
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
        let res = &mut set & &mut set2;
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            assert_eq!(res.contains(kmer), set2.contains(kmer));
        }
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
        let res = &mut set - &mut set2;
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            assert_eq!(res.contains(kmer), !set2.contains(kmer));
        }
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
        let res = &mut set ^ &mut set2;
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            assert_eq!(res.contains(kmer), !set2.contains(kmer));
        }
        set ^= &mut set2;
        for kmer in KmerT::iter_from_nucs(nucs.iter()) {
            assert_eq!(set.contains(kmer), !set2.contains(kmer));
        }
    }
}
