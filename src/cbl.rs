use crate::bit_container::*;
use crate::container::*;
use crate::kmer::{Base, Kmer, RawKmer};
use crate::necklace::{necklace_pos, NecklaceQueue};
use crate::tiered_vec::*;
use itertools::structs::GroupBy;
use itertools::Itertools;
use std::collections::{btree_map::Entry::Vacant, BTreeMap};

struct _CBLState<const N: usize, const P: usize, const W: usize, const K: usize, KT: Base> {
    pub necklace_queue: NecklaceQueue<N, W, KT>,
    pub current_prefix: usize,
    pub current_rank: usize,
    pub current_id: usize,
}

pub struct CBL<
    const N: usize,
    const P: usize,
    const W: usize,
    const K: usize,
    KT: Base,
    PrefixContainer: BitContainer,
    SuffixContainer: Container<u32>,
    const PREFIX_SIZE: usize = 24,
> {
    prefixes: PrefixContainer,
    tiered: UniquePtr<TieredVec24>,
    suffix_containers: Vec<SuffixContainer>,
    necklace_queue: NecklaceQueue<N, W, KT>,
}

// macro_rules! impl_cbl {
// ($($T:ty),+) => {$(
// )*}}

// impl_cbl!(u8, u16, u32, u64, u128);

impl<
        const N: usize,
        const P: usize,
        const W: usize,
        const K: usize,
        PrefixContainer: BitContainer,
        SuffixContainer: Container<u32>,
        const PREFIX_SIZE: usize,
    > CBL<N, P, W, K, u64, PrefixContainer, SuffixContainer, PREFIX_SIZE>
{
    pub const ITEM_SIZE: usize = (N + P);
    pub const SUFFIX_SIZE: usize = Self::ITEM_SIZE - PREFIX_SIZE;
    pub const SUFFIX_MASK: u64 = (1 << (Self::SUFFIX_SIZE)) - 1;

    pub fn new() -> Self {
        Self {
            prefixes: PrefixContainer::new_with_len(PREFIX_SIZE),
            tiered: new_tiered_vec_24(),
            suffix_containers: Vec::new(),
            necklace_queue: NecklaceQueue::<N, W, u64>::new(),
        }
    }

    #[inline]
    pub fn merge_necklace_pos(necklace: u64, pos: usize) -> u64 {
        necklace * N as u64 + pos as u64
    }

    #[inline]
    pub fn split_prefix_suffix(word: u64) -> (usize, u32) {
        (
            (word >> Self::SUFFIX_SIZE) as usize,
            (word & Self::SUFFIX_MASK) as u32,
        )
    }

    #[inline]
    pub fn single_prefix_suffix<KmerT: Kmer<K, u64>>(kmer: KmerT) -> (usize, u32) {
        let (necklace, pos) = necklace_pos::<N, u64>(kmer.to_int());
        Self::split_prefix_suffix(Self::merge_necklace_pos(necklace, pos))
    }

    pub fn contains<KmerT: Kmer<K, u64>>(&self, kmer: KmerT) -> bool {
        let (prefix, suffix) = Self::single_prefix_suffix(kmer);
        if !self.prefixes.contains(prefix) {
            return false;
        }
        let rank = self.prefixes.rank(prefix);
        let id = self.tiered.get(rank) as usize;
        self.suffix_containers[id].contains(suffix)
    }

    pub fn insert<KmerT: Kmer<K, u64>>(&mut self, kmer: KmerT) {
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

    pub fn remove<KmerT: Kmer<K, u64>>(&mut self, kmer: KmerT) {
        let (prefix, suffix) = Self::single_prefix_suffix(kmer);
        if self.prefixes.contains(prefix) {
            let rank = self.prefixes.rank(prefix);
            let id = self.tiered.get(rank) as usize;
            self.suffix_containers[id].remove(suffix);
            // if self.suffix_containers[id].is_empty() {
            //     self.suffix_containers.swap_remove(id);
            //     self.tiered.remove(rank);
            //     self.prefixes.remove(prefix);
            // }
        }
    }

    pub fn get_seq_prefix_suffix(&mut self, seq: &[u8]) -> Vec<(usize, u32)> {
        let mut res = Vec::with_capacity(seq.len() - K + 1);
        let kmer = RawKmer::<K, u64>::from_nucs(&seq[..K]);
        self.necklace_queue.insert_full(kmer.to_int());
        let (necklace, pos) = self.necklace_queue.get_necklace_pos();
        res.push(Self::split_prefix_suffix(Self::merge_necklace_pos(
            necklace, pos,
        )));
        for base in seq[K..].iter().filter_map(u64::from_nuc) {
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
    ) -> impl Iterator<Item = (usize, u32)> + 'a {
        let kmer = RawKmer::<K, u64>::from_nucs(&seq[..(K - 1)]);
        self.necklace_queue.insert_full(kmer.to_int());
        seq[(K - 1)..].iter().filter_map(u64::from_nuc).map(|base| {
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
        impl Iterator<Item = (usize, u32)> + 'a,
        for<'b> fn(&'b (usize, u32)) -> usize,
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
                self.suffix_containers[id].insert_iter(group.iter().map(|&(_, suffix)| suffix));
            }
        }

        // for (prefix, group) in self.get_suffix_groups(seq).into_iter() {
        //     let absent = self.prefixes.insert(prefix);
        //     let rank = self.prefixes.rank(prefix);
        //     let id = if absent {
        //         let id = self.suffix_containers.len() as u32;
        //         self.tiered.insert(rank, id);
        //         self.suffix_containers.push(SuffixContainer::new());
        //         id as usize
        //     } else {
        //         self.tiered.get(rank) as usize
        //     };
        //     self.suffix_containers[id].insert_iter(group.into_iter().map(|(_, suffix)| suffix));
        // }
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
                    self.suffix_containers[id].remove_iter(group.iter().map(|&(_, suffix)| suffix));
                }
                self.suffix_containers[id].shrink();
                // if self.suffix_containers[id].is_empty() {
                //     // let rank_with_max_id = self.get_rank_with_max_id();
                //     // self.suffix_containers.swap_remove(id);
                //     // self.tiered.update(rank_with_max_id, id as u32);
                //     self.tiered.remove(rank);
                //     self.prefixes.remove(prefix);
                // }
            }
        }
    }

    // fn get_rank_with_max_id(&self) -> usize {
    //     (0..self.suffix_containers.len())
    //         .map(|rank| (self.tiered.get(rank), rank))
    //         .max()
    //         .expect("Empty structure")
    //         .1
    // }

    pub fn iter(&self) {
        todo!()
    }

    pub fn prefix_load(&self) -> f64 {
        self.tiered.len() as f64 / (1 << PREFIX_SIZE) as f64
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
        const N: usize,
        const P: usize,
        const W: usize,
        const K: usize,
        PrefixContainer: BitContainer,
        SuffixContainer: Container<u32>,
        const PREFIX_SIZE: usize,
    > Default for CBL<N, P, W, K, u64, PrefixContainer, SuffixContainer, PREFIX_SIZE>
{
    fn default() -> Self {
        Self::new()
    }
}
