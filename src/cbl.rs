use crate::bit_container::*;
use crate::container::*;
use crate::kmer::{Base, Kmer, RawKmer};
use crate::necklace::{necklace_pos, NecklaceQueue};
use crate::tiered_vec::*;
use itertools::structs::GroupBy;
use itertools::Itertools;
use std::collections::{btree_map::Entry::Vacant, BTreeMap};

#[derive(Debug)]
pub struct CBL<
    const K: usize,
    KT: Base,
    ST: Ord,
    const M: usize = 9,
    const PREFIX_BITS: usize = 24,
    PrefixContainer: BitContainer = RankBitContainer,
    SuffixContainer: Container<ST> = SemiSortedVec<ST, 32>,
> where
    [(); 2 * K - M + 1]:,
{
    prefixes: PrefixContainer,
    tiered: UniquePtr<TieredVec28>,
    suffix_containers: Vec<SuffixContainer>,
    necklace_queue: NecklaceQueue<{ 2 * K }, { 2 * K - M + 1 }, KT>,
}

macro_rules! impl_cbl {
    ($KT:ty, $ST:ty) => {
        impl<
                const K: usize,
                const M: usize,
                const PREFIX_BITS: usize,
                PrefixContainer: BitContainer,
                SuffixContainer: Container<$ST>,
            > CBL<K, $KT, $ST, M, PREFIX_BITS, PrefixContainer, SuffixContainer>
        where
            [(); 2 * K - M + 1]:,
        {
            pub const WORD_BITS: usize = 2 * K;
            pub const POS_BITS: usize = Self::WORD_BITS.next_power_of_two().ilog2() as usize;
            pub const ITEM_BITS: usize = Self::WORD_BITS + Self::POS_BITS;
            pub const SUFFIX_BITS: usize = Self::ITEM_BITS - PREFIX_BITS;
            pub const SUFFIX_MASK: $KT = (1 << (Self::SUFFIX_BITS)) - 1;

            pub fn new() -> Self {
                Self {
                    prefixes: PrefixContainer::new_with_len(PREFIX_BITS),
                    tiered: new_tiered_vec_28(),
                    suffix_containers: Vec::new(),
                    necklace_queue: NecklaceQueue::<{ 2 * K }, { 2 * K - M + 1 }, $KT>::new(),
                }
            }

            #[inline]
            pub fn merge_necklace_pos(necklace: $KT, pos: usize) -> $KT {
                necklace * Self::WORD_BITS as $KT + pos as $KT
                // (necklace << P) | (pos as $KT)
            }

            #[inline]
            pub fn split_prefix_suffix(word: $KT) -> (usize, $ST) {
                (
                    (word >> Self::SUFFIX_BITS) as usize,
                    (word & Self::SUFFIX_MASK) as $ST,
                )
            }

            #[inline]
            pub fn single_prefix_suffix<KmerT: Kmer<K, $KT>>(kmer: KmerT) -> (usize, $ST) {
                let (necklace, pos) = necklace_pos::<{ 2 * K }, $KT>(kmer.to_int());
                Self::split_prefix_suffix(Self::merge_necklace_pos(necklace, pos))
            }

            pub fn contains<KmerT: Kmer<K, $KT>>(&self, kmer: KmerT) -> bool {
                let (prefix, suffix) = Self::single_prefix_suffix(kmer);
                if !self.prefixes.contains(prefix) {
                    return false;
                }
                let rank = self.prefixes.rank(prefix);
                let id = self.tiered.get(rank) as usize;
                self.suffix_containers[id].contains(suffix)
            }

            pub fn insert<KmerT: Kmer<K, $KT>>(&mut self, kmer: KmerT) {
                let (prefix, suffix) = Self::single_prefix_suffix(kmer);
                let absent = self.prefixes.insert(prefix);
                let rank = self.prefixes.rank(prefix);
                if absent {
                    let id = self.suffix_containers.len() as u32;
                    self.suffix_containers
                        .push(SuffixContainer::new_with_one(suffix));
                    self.tiered.insert(rank, id);
                } else {
                    let id = self.tiered.get(rank) as usize;
                    self.suffix_containers[id].insert(suffix);
                }
            }

            pub fn remove<KmerT: Kmer<K, $KT>>(&mut self, kmer: KmerT) {
                let (prefix, suffix) = Self::single_prefix_suffix(kmer);
                if self.prefixes.contains(prefix) {
                    let rank = self.prefixes.rank(prefix);
                    let id = self.tiered.get(rank) as usize;
                    self.suffix_containers[id].remove(suffix);
                }
            }

            pub fn get_seq_prefix_suffix(&mut self, seq: &[u8]) -> Vec<(usize, $ST)> {
                let mut res = Vec::with_capacity(seq.len() - K + 1);
                let kmer = RawKmer::<K, $KT>::from_nucs(&seq[..K]);
                self.necklace_queue.insert_full(kmer.to_int());
                let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                res.push(Self::split_prefix_suffix(Self::merge_necklace_pos(
                    necklace, pos,
                )));
                for base in seq[K..].iter().filter_map(<$KT>::from_nuc) {
                    self.necklace_queue.insert2(base);
                    let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                    res.push(Self::split_prefix_suffix(Self::merge_necklace_pos(
                        necklace, pos,
                    )));
                }
                res
            }

            fn _get_prefixes_suffixes<'a>(
                &'a mut self,
                seq: &'a [u8],
            ) -> impl Iterator<Item = (usize, $ST)> + 'a {
                let kmer = RawKmer::<K, $KT>::from_nucs(&seq[..(K - 1)]);
                self.necklace_queue.insert_full(kmer.to_int());
                seq[(K - 1)..]
                    .iter()
                    .filter_map(<$KT>::from_nuc)
                    .map(|base| {
                        self.necklace_queue.insert2(base);
                        let (necklace, pos) = self.necklace_queue.get_necklace_pos();
                        Self::split_prefix_suffix(Self::merge_necklace_pos(necklace, pos))
                    })
            }

            #[inline]
            #[allow(clippy::type_complexity)]
            fn _get_suffix_groups<'a>(
                &'a mut self,
                seq: &'a [u8],
            ) -> GroupBy<
                usize,
                impl Iterator<Item = (usize, $ST)> + 'a,
                for<'b> fn(&'b (usize, $ST)) -> usize,
            > {
                self._get_prefixes_suffixes(seq).group_by(|(p, _)| *p)
            }

            pub fn insert_seq(&mut self, seq: &[u8]) {
                assert!(seq.len() >= K);
                let seq_prefix_suffix = self.get_seq_prefix_suffix(seq);
                for group in seq_prefix_suffix.group_by(|(p1, _), (p2, _)| p1 == p2) {
                    let prefix = group[0].0;
                    let absent = self.prefixes.insert(prefix);
                    let rank = self.prefixes.rank(prefix);
                    let id = if absent {
                        let id = self.suffix_containers.len() as u32;
                        self.tiered.insert(rank, id);
                        self.suffix_containers.push(SuffixContainer::new());
                        id as usize
                    } else {
                        self.tiered.get(rank) as usize
                    };
                    if group.len() == 1 {
                        self.suffix_containers[id].insert(group[0].1);
                    } else {
                        self.suffix_containers[id]
                            .insert_iter(group.iter().map(|&(_, suffix)| suffix));
                    }
                }
            }

            pub fn remove_seq(&mut self, seq: &[u8]) {
                assert!(seq.len() >= K);
                let seq_prefix_suffix = self.get_seq_prefix_suffix(seq);
                for group in seq_prefix_suffix.group_by(|(p1, _), (p2, _)| p1 == p2) {
                    let prefix = group[0].0;
                    if self.prefixes.contains(prefix) {
                        let rank = self.prefixes.rank(prefix);
                        let id = self.tiered.get(rank) as usize;
                        if group.len() == 1 {
                            self.suffix_containers[id].remove(group[0].1);
                        } else {
                            self.suffix_containers[id]
                                .remove_iter(group.iter().map(|&(_, suffix)| suffix));
                        }
                        self.suffix_containers[id].shrink();
                    }
                }
            }

            pub fn iter(&self) {
                todo!()
            }

            pub fn prefix_load(&self) -> f64 {
                self.tiered.len() as f64 / (1 << PREFIX_BITS) as f64
            }

            pub fn suffix_load(&self) -> f64 {
                let total_size: usize = self
                    .suffix_containers
                    .iter()
                    .map(|container| container.len())
                    .sum();
                total_size as f64 / self.suffix_containers.len() as f64
            }

            pub fn suffix_load_repartition(&self) -> BTreeMap<usize, f64> {
                let mut size_count = BTreeMap::new();
                for size in self
                    .suffix_containers
                    .iter()
                    .map(|container| container.len())
                {
                    if let Vacant(e) = size_count.entry(size) {
                        e.insert(1usize);
                    } else {
                        *size_count.get_mut(&size).unwrap() += 1;
                    }
                }
                size_count
                    .iter()
                    .map(|(&k, &v)| (k, v as f64 / self.suffix_containers.len() as f64))
                    .collect()
            }
        }

        impl<
                const K: usize,
                const M: usize,
                const PREFIX_BITS: usize,
                PrefixContainer: BitContainer,
                SuffixContainer: Container<$ST>,
            > Default for CBL<K, $KT, $ST, M, PREFIX_BITS, PrefixContainer, SuffixContainer>
        where
            [(); 2 * K - M + 1]:,
        {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

impl_cbl!(u64, u32);
impl_cbl!(u64, u64);
impl_cbl!(u128, u64);
