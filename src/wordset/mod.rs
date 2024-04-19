mod set_ops;

use crate::bitvector::*;
use crate::ffi::{TieredVec28, UniquePtr, WithinUniquePtr};
use crate::sliced_int::SlicedInt;
use crate::trievec::*;
use num_traits::cast::AsPrimitive;
use num_traits::sign::Unsigned;
use num_traits::PrimInt;
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::collections::BTreeMap;

pub struct WordSet<const PREFIX_BITS: usize, const SUFFIX_BITS: usize>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    prefixes: Bitvector,
    tiered: UniquePtr<TieredVec28>,
    suffix_containers: Vec<TrieVec<{ SUFFIX_BITS.div_ceil(8) }>>,
    empty_containers: Vec<usize>,
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    const PREFIX_BITS: usize = PREFIX_BITS;
    const SUFFIX_BITS: usize = SUFFIX_BITS;
    const THRESHOLD: usize = 1024;

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
    ) -> (usize, SlicedInt<{ SUFFIX_BITS.div_ceil(8) }>) {
        let suffix_mask: T = (T::one() << Self::SUFFIX_BITS) - T::one();
        (
            (word >> Self::SUFFIX_BITS).as_(),
            SlicedInt::<{ SUFFIX_BITS.div_ceil(8) }>::from_int(word & suffix_mask),
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
                    self.suffix_containers
                        .push(TrieVec::<{ SUFFIX_BITS.div_ceil(8) }>::new_with_one(suffix));
                    self.tiered.insert(rank, id as u32);
                }
            };
        } else {
            let id = self.tiered.get(rank) as usize;
            absent = self.suffix_containers[id].insert(suffix);
            self.adapt_container_grow(id);
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
            self.adapt_container_shrink(id);
            if self.suffix_containers[id].is_empty() {
                self.empty_containers.push(id);
                self.tiered.remove(rank);
                self.prefixes.remove(prefix);
            }
        }
        present
    }

    pub fn contains_all<T: PrimInt + Unsigned + AsPrimitive<usize>>(
        &mut self,
        words: &[T],
    ) -> bool {
        let prefixes_suffixes: Vec<_> = words
            .iter()
            .map(|&word| Self::split_prefix_suffix(word))
            .collect();
        for group in prefixes_suffixes.chunk_by(|(p1, _), (p2, _)| p1 == p2) {
            let prefix = group[0].0;
            if !self.prefixes.contains(prefix) {
                return false;
            }
            let rank = self.prefixes.rank(prefix);
            let id = self.tiered.get(rank) as usize;
            for (_, suffix) in group.iter() {
                if !self.suffix_containers[id].contains(suffix) {
                    return false;
                }
            }
        }
        true
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
        for group in prefixes_suffixes.chunk_by(|(p1, _), (p2, _)| p1 == p2) {
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
        for group in prefixes_suffixes.chunk_by(|(p1, _), (p2, _)| p1 == p2) {
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
                            .push(TrieVec::<{ SUFFIX_BITS.div_ceil(8) }>::new());
                        self.tiered.insert(rank, id as u32);
                        id
                    }
                }
            } else {
                self.tiered.get(rank) as usize
            };
            self.suffix_containers[id].insert_iter(group.iter().map(|&(_, suffix)| suffix));
            self.adapt_container_grow(id);
        }
    }

    pub fn remove_batch<T: PrimInt + Unsigned + AsPrimitive<usize>>(&mut self, words: &[T]) {
        let prefixes_suffixes: Vec<_> = words
            .iter()
            .map(|&word| Self::split_prefix_suffix(word))
            .collect();
        for group in prefixes_suffixes.chunk_by(|(p1, _), (p2, _)| p1 == p2) {
            let prefix = group[0].0;
            if self.prefixes.contains(prefix) {
                let rank = self.prefixes.rank(prefix);
                let id = self.tiered.get(rank) as usize;
                self.suffix_containers[id].remove_iter(group.iter().map(|&(_, suffix)| suffix));
                if self.suffix_containers[id].is_empty() {
                    self.empty_containers.push(id);
                    self.tiered.remove(rank);
                    self.prefixes.remove(prefix);
                }
                self.adapt_container_shrink(id);
            }
        }
    }

    fn adapt_container_grow(&mut self, id: usize) {
        if self.suffix_containers[id].len() > Self::THRESHOLD {
            self.suffix_containers[id].as_trie();
        }
    }

    fn adapt_container_shrink(&mut self, id: usize) {
        if self.suffix_containers[id].len() <= Self::THRESHOLD {
            self.suffix_containers[id].as_vec();
        }
    }

    #[inline]
    pub fn prefix_load(&self) -> f64 {
        self.tiered.len() as f64 / (1 << Self::PREFIX_BITS) as f64
    }

    pub fn buckets_sizes(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.prefixes.iter().enumerate().map(|(rank, prefix)| {
            let id = self.tiered.get(rank) as usize;
            (prefix, self.suffix_containers[id].len())
        })
    }

    pub fn buckets_size_count(&self) -> BTreeMap<usize, usize> {
        let mut size_count = BTreeMap::new();
        for (_, size) in self.buckets_sizes() {
            size_count.entry(size).and_modify(|e| *e += 1).or_insert(1);
        }
        size_count
    }

    pub fn buckets_load_repartition(&self) -> BTreeMap<usize, f64> {
        let size_count = self.buckets_size_count();
        let total_size: usize = size_count.iter().map(|(&s, &c)| s * c).sum();
        size_count
            .iter()
            .map(|(&s, &c)| (s, (s * c) as f64 / total_size as f64))
            .collect()
    }

    #[inline]
    pub fn iter<T: PrimInt + Unsigned + AsPrimitive<usize>>(&self) -> impl Iterator<Item = T> + '_
    where
        usize: AsPrimitive<T>,
    {
        WordSetIterator {
            wordset: self,
            prefix_iter: self.prefixes.iter(),
            prefix: None,
            suffix_iter: None,
            suffix: None,
        }
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
    suffix_iter: Option<TrieVecIterator<'a, { SUFFIX_BITS.div_ceil(8) }>>,
    suffix: Option<SlicedInt<{ SUFFIX_BITS.div_ceil(8) }>>,
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
            self.suffix_iter = Some(self.wordset.suffix_containers[id].iter());
            self.suffix = self.suffix_iter.as_mut().unwrap().next();
        }
        let prefix: T = self.prefix?.as_();
        let suffix: T = self.suffix?.get();
        self.suffix = self.suffix_iter.as_mut().unwrap().next();
        Some((prefix << SUFFIX_BITS) | suffix)
    }
}

impl<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> Clone for WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn clone(&self) -> Self {
        let tiered = TieredVec28::new().within_unique_ptr();
        for i in 0..self.tiered.len() {
            tiered.insert(i, self.tiered.get(i));
        }
        Self {
            prefixes: self.prefixes.clone(),
            tiered,
            suffix_containers: self.suffix_containers.clone(),
            empty_containers: self.empty_containers.clone(),
        }
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
            let prefix = prefix as u32;
            let id = self.tiered.get(rank) as usize;
            map.serialize_entry(&prefix, &self.suffix_containers[id])?;
        }
        map.end()
    }
}

struct WordSetVisitor<const PREFIX_BITS: usize, const SUFFIX_BITS: usize> {}

impl<'de, const PREFIX_BITS: usize, const SUFFIX_BITS: usize> Visitor<'de>
    for WordSetVisitor<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    type Value = WordSet<PREFIX_BITS, SUFFIX_BITS>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a wordset")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
        let mut wordset = WordSet {
            prefixes: Bitvector::new_with_bitlength(PREFIX_BITS),
            tiered: TieredVec28::new().within_unique_ptr(),
            suffix_containers: Vec::with_capacity(access.size_hint().unwrap_or(0)),
            empty_containers: Vec::new(),
        };
        while let Some((prefix, suffix_container)) = access.next_entry::<u32, _>()? {
            let prefix = prefix as usize;
            let rank = wordset.suffix_containers.len();
            wordset.prefixes.insert(prefix);
            wordset.tiered.insert(rank, rank as u32);
            wordset.suffix_containers.push(suffix_container);
        }
        Ok(wordset)
    }
}

impl<'de, const PREFIX_BITS: usize, const SUFFIX_BITS: usize> Deserialize<'de>
    for WordSet<PREFIX_BITS, SUFFIX_BITS>
where
    [(); SUFFIX_BITS.div_ceil(8)]:,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(WordSetVisitor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::{thread_rng, SeedableRng};

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
            assert!(set.remove(i));
        }
        for &i in v0.iter() {
            assert!(!set.contains(i));
        }
        assert!(set.is_empty());
    }

    #[test]
    fn test_random_insert_contains_remove() {
        let mut v0 = (0..(2 * N)).step_by(2).collect_vec();
        let mut rng = StdRng::seed_from_u64(42);
        v0.shuffle(&mut rng);

        let mut set = WordSet::<PREFIX_BITS, SUFFIX_BITS>::new();
        for &i in v0.iter() {
            set.insert(i);
        }
        assert_eq!(set.count(), N);
        v0.shuffle(&mut rng);

        for &i in v0.iter() {
            assert!(set.contains(i));
        }
        v0.shuffle(&mut rng);

        for &i in v0.iter() {
            assert!(set.remove(i));
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
