use crate::bit_container::*;
use crate::compact_int::CompactInt;
use crate::container::*;
use crate::tiered_vec::*;
use num_traits::cast::AsPrimitive;
use num_traits::sign::Unsigned;
use num_traits::PrimInt;
use std::collections::{btree_map::Entry::Vacant, BTreeMap};

pub struct BitWordSet<const BITS: usize, const PREFIX_BITS: usize = 24>
where
    [(); BITS.saturating_sub(PREFIX_BITS).div_ceil(8)]:,
{
    prefixes: RankBitContainer,
    tiered: UniquePtr<TieredVec28>,
    suffix_containers:
        Vec<SemiSortedVec<CompactInt<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>, 32>>,
    empty_containers: Vec<usize>,
}

impl<const BITS: usize, const PREFIX_BITS: usize> BitWordSet<BITS, PREFIX_BITS>
where
    [(); BITS.saturating_sub(PREFIX_BITS).div_ceil(8)]:,
{
    pub const BITS: usize = BITS;
    pub const PREFIX_BITS: usize = PREFIX_BITS;
    pub const SUFFIX_BITS: usize = Self::BITS.saturating_sub(Self::PREFIX_BITS);
    pub const SUFFIX_BYTES: usize = Self::SUFFIX_BITS.div_ceil(8);
    const CHUNK_SIZE: usize = 1024;

    pub fn new() -> Self {
        Self {
            prefixes: RankBitContainer::new_with_len(Self::PREFIX_BITS),
            tiered: new_tiered_vec_28(),
            suffix_containers: Vec::new(),
            empty_containers: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.suffix_containers
            .iter()
            .map(|container| container.len())
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.prefixes.count() == 0
    }

    #[inline]
    pub fn split_prefix_suffix<T: PrimInt + Unsigned + AsPrimitive<usize>>(
        word: T,
    ) -> (
        usize,
        CompactInt<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>,
    ) {
        let suffix_mask: T = (T::one() << Self::SUFFIX_BITS) - T::one();
        (
            (word >> Self::SUFFIX_BITS).as_(),
            CompactInt::<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>::from_int(
                word & suffix_mask,
            ),
        )
    }

    pub fn contains<T: PrimInt + Unsigned + AsPrimitive<usize>>(&self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        if !self.prefixes.contains(prefix) {
            return false;
        }
        let rank = self.prefixes.rank(prefix);
        let id = self.tiered.get(rank) as usize;
        self.suffix_containers[id].contains(suffix)
    }

    pub fn insert<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        let mut absent = self.prefixes.insert(prefix);
        let rank = self.prefixes.rank(prefix);
        if absent {
            let id = self.suffix_containers.len() as u32;
            self.suffix_containers.push(SemiSortedVec::<
                CompactInt<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>,
                32,
            >::new_with_one(suffix));
            self.tiered.insert(rank, id);
        } else {
            let id = self.tiered.get(rank) as usize;
            absent = self.suffix_containers[id].insert(suffix);
        }
        absent
    }

    pub fn remove<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        let mut present = self.prefixes.contains(prefix);
        if present {
            let rank = self.prefixes.rank(prefix);
            let id = self.tiered.get(rank) as usize;
            present = self.suffix_containers[id].remove(suffix);
            if self.suffix_containers[id].is_empty() {
                self.suffix_containers[id] = SemiSortedVec::new();
                self.empty_containers.push(id);
                self.tiered.remove(rank);
                self.prefixes.remove(prefix);
            }
        }
        present
    }

    pub fn insert_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, words: &[T]) {
        for chunk in words.chunks(Self::CHUNK_SIZE) {
            let prefixes_suffixes: Vec<_> = chunk
                .iter()
                .map(|&word| Self::split_prefix_suffix(word))
                .collect();
            for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
                let prefix = group[0].0;
                let absent = self.prefixes.insert(prefix);
                let rank = self.prefixes.rank(prefix);
                let id = if absent {
                    let id = self.suffix_containers.len() as u32;
                    self.tiered.insert(rank, id);
                    self.suffix_containers.push(SemiSortedVec::<
                        CompactInt<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>,
                        32,
                    >::new());
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
            //         let id = self.suffix_containers.len() as $ST;
            //         self.tiered.insert(rank, id);
            //         self.suffix_containers.push(SuffixContainer::new());
            //         id as usize
            //     } else {
            //         self.tiered.get(rank) as usize
            //     };
            //     self.suffix_containers[id].insert_iter(group.into_iter().map(|(_, suffix)| suffix));
            // }
        }
    }

    pub fn remove_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, words: &[T]) {
        for chunk in words.chunks(Self::CHUNK_SIZE) {
            let prefixes_suffixes: Vec<_> = chunk
                .iter()
                .map(|&word| Self::split_prefix_suffix(word))
                .collect();
            for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
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
                    if self.suffix_containers[id].is_empty() {
                        self.suffix_containers[id] = SemiSortedVec::new();
                        self.empty_containers.push(id);
                        self.tiered.remove(rank);
                        self.prefixes.remove(prefix);
                    }
                }
            }
        }
    }

    // pub fn iter(&self) {
    //     todo!()
    // }

    pub fn prefix_load(&self) -> f64 {
        self.tiered.len() as f64 / (1 << Self::PREFIX_BITS) as f64
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

impl<const BITS: usize, const PREFIX_BITS: usize> Default for BitWordSet<BITS, PREFIX_BITS>
where
    [(); BITS.saturating_sub(PREFIX_BITS).div_ceil(8)]:,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    const N: usize = 1_000_000;
    const BITS: usize = 32;

    #[test]
    fn test_insert_contains_remove() {
        let mut positive: Vec<_> = (0..(2 * N)).step_by(2).collect();
        let mut negative: Vec<_> = (0..(2 * N)).skip(1).step_by(2).collect();
        let mut rng = thread_rng();
        positive.shuffle(&mut rng);
        negative.shuffle(&mut rng);
        let mut set = BitWordSet::<BITS>::new();
        for &i in positive.iter() {
            set.insert(i);
        }
        for &i in positive.iter() {
            assert!(set.contains(i));
        }
        for &i in negative.iter() {
            assert!(!set.contains(i));
        }
        for &i in positive.iter() {
            // assert_eq!(set.count(), N - i / 2);
            set.remove(i);
        }
        for &i in positive.iter() {
            assert!(!set.contains(i));
        }
        // assert!(set.is_empty());
    }

    #[test]
    fn test_batch_operations() {
        let mut set = BitWordSet::<BITS>::new();
        let words: Vec<_> = (0..(2 * N)).step_by(2).collect();
        set.insert_batch(&words);
        for i in (0..(2 * N)).step_by(2) {
            assert!(set.contains(i));
        }
        for i in (0..(2 * N)).skip(1).step_by(2) {
            assert!(!set.contains(i));
        }
        set.remove_batch(&words);
        for i in (0..(2 * N)).step_by(2) {
            assert!(!set.contains(i));
        }
        // assert!(set.is_empty());
    }
}
