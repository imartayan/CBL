mod set_ops;

use crate::bitvector::*;
use crate::compact_int::CompactInt;
use crate::container::*;
use crate::ffi::{TieredVec28, UniquePtr, WithinUniquePtr};
use core::slice::Iter;
use num_traits::cast::AsPrimitive;
use num_traits::sign::Unsigned;
use num_traits::PrimInt;
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{btree_map::Entry::Vacant, BTreeMap};

pub struct WordSet<const PREFIX_BITS: usize, const SUFFIX_BITS: usize>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    prefixes: Bitvector,
    tiered: UniquePtr<TieredVec28>,
    suffix_containers: Vec<SemiSortedVec<CompactInt<{ SUFFIX_BITS.div_ceil(8) }>, 32>>,
    empty_containers: Vec<usize>,
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    const PREFIX_BITS: usize = PREFIX_BITS;
    const SUFFIX_BITS: usize = SUFFIX_BITS;

    pub fn new() -> Self {
        assert!(
            PREFIX_BITS <= 28,
            "PREFIX_BITS={PREFIX_BITS} but it should be ≤ 28"
        );
        assert!(SUFFIX_BITS > 0, "SUFFIX_BITS should be ≠ 0");
        Self {
            prefixes: Bitvector::new_with_bitlength(Self::PREFIX_BITS),
            tiered: TieredVec28::new().within_unique_ptr(),
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

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.prefixes.count() == 0
    }

    #[inline]
    pub fn split_prefix_suffix<T: PrimInt + Unsigned + AsPrimitive<usize>>(
        word: T,
    ) -> (usize, CompactInt<{ SUFFIX_BITS.div_ceil(8) }>) {
        let suffix_mask: T = (T::one() << Self::SUFFIX_BITS) - T::one();
        (
            (word >> Self::SUFFIX_BITS).as_(),
            CompactInt::<{ SUFFIX_BITS.div_ceil(8) }>::from_int(word & suffix_mask),
        )
    }

    #[inline]
    pub fn contains<T: PrimInt + Unsigned + AsPrimitive<usize>>(&self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        if !self.prefixes.contains(prefix) {
            return false;
        }
        let rank = self.prefixes.rank(prefix);
        let id = self.tiered.get(rank) as usize;
        self.suffix_containers[id].contains(&suffix)
    }

    pub fn insert<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        let mut absent = self.prefixes.insert(prefix);
        let rank = self.prefixes.rank(prefix);
        if absent {
            match self.empty_containers.pop() {
                Some(id) => {
                    self.suffix_containers[id].insert(suffix);
                    self.tiered.insert(rank, id as u32);
                }
                None => {
                    let id = self.suffix_containers.len();
                    self.suffix_containers.push(SemiSortedVec::<
                        CompactInt<{ SUFFIX_BITS.div_ceil(8) }>,
                        32,
                    >::new_with_one(suffix));
                    self.tiered.insert(rank, id as u32);
                }
            };
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
            present = self.suffix_containers[id].remove(&suffix);
            if self.suffix_containers[id].is_empty() {
                self.empty_containers.push(id);
                self.tiered.remove(rank);
                self.prefixes.remove(prefix);
            }
        }
        present
    }

    pub fn contains_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(
        &mut self,
        words: &[T],
    ) -> Vec<bool> {
        let mut res = Vec::with_capacity(words.len());
        let prefixes_suffixes: Vec<_> = words
            .iter()
            .map(|&word| Self::split_prefix_suffix(word))
            .collect();
        for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
            let prefix = group[0].0;
            if !self.prefixes.contains(prefix) {
                res.resize(res.len() + group.len(), false);
                continue;
            }
            let rank = self.prefixes.rank(prefix);
            let id = self.tiered.get(rank) as usize;
            for (_, suffix) in group.iter() {
                res.push(self.suffix_containers[id].contains(suffix));
            }
        }
        res
    }

    pub fn insert_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, words: &[T]) {
        let prefixes_suffixes: Vec<_> = words
            .iter()
            .map(|&word| Self::split_prefix_suffix(word))
            .collect();
        for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
            let prefix = group[0].0;
            let absent = self.prefixes.insert(prefix);
            let rank = self.prefixes.rank(prefix);
            let id = if absent {
                match self.empty_containers.pop() {
                    Some(id) => {
                        self.tiered.insert(rank, id as u32);
                        id
                    }
                    None => {
                        let id = self.suffix_containers.len();
                        self.suffix_containers
                            .push(
                                SemiSortedVec::<CompactInt<{ SUFFIX_BITS.div_ceil(8) }>, 32>::new(),
                            );
                        self.tiered.insert(rank, id as u32);
                        id
                    }
                }
            } else {
                self.tiered.get(rank) as usize
            };
            if group.len() == 1 {
                self.suffix_containers[id].insert(group[0].1);
            } else {
                self.suffix_containers[id].insert_iter(group.iter().map(|&(_, suffix)| suffix));
            }
        }
    }

    pub fn remove_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, words: &[T]) {
        let prefixes_suffixes: Vec<_> = words
            .iter()
            .map(|&word| Self::split_prefix_suffix(word))
            .collect();
        for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
            let prefix = group[0].0;
            if self.prefixes.contains(prefix) {
                let rank = self.prefixes.rank(prefix);
                let id = self.tiered.get(rank) as usize;
                if group.len() == 1 {
                    self.suffix_containers[id].remove(&group[0].1);
                } else {
                    self.suffix_containers[id].remove_iter(group.iter().map(|&(_, suffix)| suffix));
                }
                if self.suffix_containers[id].is_empty() {
                    // self.suffix_containers[id] = SemiSortedVec::new();
                    self.empty_containers.push(id);
                    self.tiered.remove(rank);
                    self.prefixes.remove(prefix);
                }
            }
        }
    }

    #[inline]
    pub fn iter<T: PrimInt + Unsigned + AsPrimitive<usize>>(&self) -> impl Iterator<Item = T> + '_
    where
        usize: AsPrimitive<T>,
    {
        WordSetIterator {
            wordset: &self,
            prefix_iter: self.prefixes.iter(),
            prefix: None,
            suffix_iter: [].iter(),
            suffix: None,
        }
    }

    #[inline]
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

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> Default
    for WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn default() -> Self {
        Self::new()
    }
}

struct WordSetIterator<
    'a,
    T: PrimInt + Unsigned + AsPrimitive<usize>,
    const PREFIX_BITS: usize,
    const SUFFIX_BITS: usize,
> where
    [(); SUFFIX_BITS.div_ceil(8)]:,
    usize: AsPrimitive<T>,
{
    wordset: &'a WordSet<PREFIX_BITS, SUFFIX_BITS>,
    prefix_iter: BitvectorIterator<'a>,
    prefix: Option<usize>,
    suffix_iter: Iter<'a, CompactInt<{ SUFFIX_BITS.div_ceil(8) }>>,
    suffix: Option<&'a CompactInt<{ SUFFIX_BITS.div_ceil(8) }>>,
}

impl<
        'a,
        T: PrimInt + Unsigned + AsPrimitive<usize>,
        const PREFIX_BITS: usize,
        const SUFFIX_BITS: usize,
    > Iterator for WordSetIterator<'a, T, PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
    usize: AsPrimitive<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        while self.suffix.is_none() {
            self.prefix = self.prefix_iter.next();
            let rank = self.wordset.prefixes.rank(self.prefix?);
            let id = self.wordset.tiered.get(rank) as usize;
            self.suffix_iter = self.wordset.suffix_containers[id].iter();
            self.suffix = self.suffix_iter.next();
        }
        let prefix: T = self.prefix?.as_();
        let suffix: T = self.suffix?.get();
        self.suffix = self.suffix_iter.next();
        Some((prefix << SUFFIX_BITS) | suffix)
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> Serialize
    for WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.tiered.len()))?;
        for (rank, prefix) in self.prefixes.iter().enumerate() {
            let id = self.tiered.get(rank) as usize;
            map.serialize_entry(&prefix, &self.suffix_containers[id])?;
        }
        map.end()
    }
}

impl<'de, const PREFIX_BITS: usize, const SUFFIX_BITS: usize> Deserialize<'de>
    for WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    const N: usize = 1_000_000;
    const PREFIX_BITS: usize = 24;
    const SUFFIX_BITS: usize = 8;

    #[test]
    fn test_insert_contains_remove() {
        let mut v0 = (0..(2 * N)).step_by(2).collect_vec();
        let mut v1 = (0..(2 * N)).skip(1).step_by(2).collect_vec();
        let mut rng = thread_rng();
        v0.shuffle(&mut rng);
        v1.shuffle(&mut rng);
        let mut set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        for &i in v0.iter() {
            set.insert(i);
        }
        for &i in v0.iter() {
            assert!(set.contains(i));
        }
        for &i in v1.iter() {
            assert!(!set.contains(i));
        }
        for &i in v0.iter() {
            set.remove(i);
        }
        for &i in v0.iter() {
            assert!(!set.contains(i));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_batch_operations() {
        let mut set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        let v0 = (0..(2 * N)).step_by(2).collect_vec();
        let v1 = (0..(2 * N)).skip(1).step_by(2).collect_vec();
        set.insert_batch(&v0);
        assert!(set.contains_batch(&v0).iter().all(|&b| b));
        for &i in v1.iter() {
            assert!(!set.contains(i));
        }
        set.remove_batch(&v0);
        for &i in v0.iter() {
            assert!(!set.contains(i));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_wordset_iter() {
        let mut set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        set.insert(1u64);
        set.insert(42u64);
        set.insert((1 << SUFFIX_BITS) - 1u64);
        set.insert((1 << SUFFIX_BITS) + 10u64);
        set.insert(10 * (1 << SUFFIX_BITS) + 10u64);
        let mut iter = set.iter::<u64>();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(42));
        assert_eq!(iter.next(), Some((1 << SUFFIX_BITS) - 1u64));
        assert_eq!(iter.next(), Some((1 << SUFFIX_BITS) + 10));
        assert_eq!(iter.next(), Some(10 * (1 << SUFFIX_BITS) + 10));
        assert_eq!(iter.next(), None);
    }
}
