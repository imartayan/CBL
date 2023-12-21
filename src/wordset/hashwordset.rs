use crate::compact_int::CompactInt;
use crate::container::*;
use ahash::AHashMap;
use num_traits::sign::Unsigned;
use num_traits::PrimInt;
use std::collections::hash_map::Entry;
use std::collections::{btree_map::Entry::Vacant, BTreeMap};

pub struct HashWordSet<const BITS: usize, const PREFIX_BITS: usize = 24>
where
    [(); PREFIX_BITS.div_ceil(8)]:,
    [(); BITS.saturating_sub(PREFIX_BITS).div_ceil(8)]:,
{
    containers: AHashMap<
        CompactInt<{ PREFIX_BITS.div_ceil(8) }>,
        SemiSortedVec<CompactInt<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>, 32>,
    >,
}

impl<const BITS: usize, const PREFIX_BITS: usize> HashWordSet<BITS, PREFIX_BITS>
where
    [(); PREFIX_BITS.div_ceil(8)]:,
    [(); BITS.saturating_sub(PREFIX_BITS).div_ceil(8)]:,
{
    pub const BITS: usize = BITS;
    pub const PREFIX_BITS: usize = PREFIX_BITS;
    pub const SUFFIX_BITS: usize = Self::BITS.saturating_sub(Self::PREFIX_BITS);
    pub const PREFIX_BYTES: usize = Self::PREFIX_BITS.div_ceil(8);
    pub const SUFFIX_BYTES: usize = Self::SUFFIX_BITS.div_ceil(8);
    const CHUNK_SIZE: usize = 1024;

    pub fn new() -> Self {
        Self {
            containers: AHashMap::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.containers
            .iter()
            .map(|(_, container)| container.len())
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.containers.is_empty()
    }

    #[inline]
    pub fn split_prefix_suffix<T: PrimInt + Unsigned>(
        word: T,
    ) -> (
        CompactInt<{ PREFIX_BITS.div_ceil(8) }>,
        CompactInt<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>,
    ) {
        let suffix_mask: T = (T::one() << Self::SUFFIX_BITS) - T::one();
        (
            CompactInt::<{ PREFIX_BITS.div_ceil(8) }>::from_int(word >> Self::SUFFIX_BITS),
            CompactInt::<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>::from_int(
                word & suffix_mask,
            ),
        )
    }

    pub fn contains<T: PrimInt + Unsigned>(&self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        match self.containers.get(&prefix) {
            None => false,
            Some(container) => container.contains(suffix),
        }
    }

    pub fn insert<T: PrimInt + Unsigned>(&mut self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        match self.containers.entry(prefix) {
            Entry::Vacant(entry) => {
                entry.insert(SemiSortedVec::<
                    CompactInt<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>,
                    32,
                >::new_with_one(suffix));
                true
            }
            Entry::Occupied(entry) => entry.into_mut().insert(suffix),
        }
    }

    pub fn remove<T: PrimInt + Unsigned>(&mut self, word: T) -> bool {
        let (prefix, suffix) = Self::split_prefix_suffix(word);
        match self.containers.entry(prefix) {
            Entry::Vacant(_) => false,
            Entry::Occupied(mut entry) => {
                let container = entry.get_mut();
                let present = container.remove(suffix);
                if container.is_empty() {
                    entry.remove();
                }
                present
            }
        }
    }

    pub fn insert_batch<T: PrimInt + Unsigned>(&mut self, words: &[T]) {
        for chunk in words.chunks(Self::CHUNK_SIZE) {
            let prefixes_suffixes: Vec<_> = chunk
                .iter()
                .map(|&word| Self::split_prefix_suffix(word))
                .collect();
            for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
                let prefix = group[0].0;
                let container = match self.containers.entry(prefix) {
                    Entry::Vacant(entry) => entry.insert(SemiSortedVec::<
                        CompactInt<{ BITS.saturating_sub(PREFIX_BITS).div_ceil(8) }>,
                        32,
                    >::new()),
                    Entry::Occupied(entry) => entry.into_mut(),
                };
                if group.len() == 1 {
                    container.insert(group[0].1);
                } else {
                    container.insert_iter(group.iter().map(|&(_, suffix)| suffix));
                }
            }
        }
    }

    pub fn remove_batch<T: PrimInt + Unsigned>(&mut self, words: &[T]) {
        for chunk in words.chunks(Self::CHUNK_SIZE) {
            let prefixes_suffixes: Vec<_> = chunk
                .iter()
                .map(|&word| Self::split_prefix_suffix(word))
                .collect();
            for group in prefixes_suffixes.group_by(|(p1, _), (p2, _)| p1 == p2) {
                let prefix = group[0].0;
                match self.containers.entry(prefix) {
                    Entry::Vacant(_) => (),
                    Entry::Occupied(mut entry) => {
                        let container = entry.get_mut();
                        if group.len() == 1 {
                            container.remove(group[0].1);
                        } else {
                            container.remove_iter(group.iter().map(|&(_, suffix)| suffix));
                        }
                        if container.is_empty() {
                            entry.remove();
                        }
                    }
                }
            }
        }
    }

    // pub fn iter(&self) {
    //     todo!()
    // }

    pub fn prefix_load(&self) -> f64 {
        self.containers.len() as f64 / (1 << Self::PREFIX_BITS) as f64
    }

    pub fn suffix_load(&self) -> f64 {
        let total_size: usize = self
            .containers
            .iter()
            .map(|(_, container)| container.len())
            .sum();
        total_size as f64 / self.containers.len() as f64
    }

    pub fn suffix_load_repartition(&self) -> BTreeMap<usize, f64> {
        let mut size_count = BTreeMap::new();
        for size in self.containers.iter().map(|(_, container)| container.len()) {
            if let Vacant(e) = size_count.entry(size) {
                e.insert(1usize);
            } else {
                *size_count.get_mut(&size).unwrap() += 1;
            }
        }
        size_count
            .iter()
            .map(|(&k, &v)| (k, v as f64 / self.containers.len() as f64))
            .collect()
    }
}

impl<const BITS: usize, const PREFIX_BITS: usize> Default for HashWordSet<BITS, PREFIX_BITS>
where
    [(); PREFIX_BITS.div_ceil(8)]:,
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
        let mut set = HashWordSet::<BITS>::new();
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
        let mut set = HashWordSet::<BITS>::new();
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
